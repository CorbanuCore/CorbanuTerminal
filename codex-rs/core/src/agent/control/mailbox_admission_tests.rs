//! Real DB/session interleavings at the unloaded-admission/publication boundary.
use super::*;
use crate::thread_manager::mailbox_admission::AdmissionPause;
use crate::thread_manager::mailbox_admission::AdmissionStage;
use codex_extension_api::ExtensionRegistryBuilder;
use codex_extension_api::MessageLifecycleContributor;
use codex_extension_api::MessageLifecycleInput;
use codex_state::AgentMailboxPhase;
use pretty_assertions::assert_eq;
use std::sync::Mutex;
use tokio::sync::Notify;

#[derive(Default)]
struct Lifecycle(Mutex<Vec<(&'static str, ResponseItemId)>>);

impl MessageLifecycleContributor for Lifecycle {
    fn on_publish(&self, input: &MessageLifecycleInput) {
        self.0
            .lock()
            .unwrap()
            .push(("published", input.message_id.clone()));
    }

    fn on_consumed(&self, input: &MessageLifecycleInput, _recipient: ThreadId) {
        self.0
            .lock()
            .unwrap()
            .push(("consumed", input.message_id.clone()));
    }
}

#[derive(Clone, Copy)]
enum Cancel {
    Neither,
    Sender,
    Reload,
    LateSender,
}

async fn admission_reload(stage: AdmissionStage, cancel: Cancel) {
    let (home, mut config) = test_config().await;
    config.features.enable(Feature::MultiAgentV2).unwrap();
    config.features.enable(Feature::Sqlite).unwrap();
    let mut harness = AgentControlHarness::new_with_config(home, config).await;
    let lifecycle = Arc::new(Lifecycle::default());
    let mut extensions = ExtensionRegistryBuilder::<Config>::new();
    extensions.message_lifecycle_contributor(lifecycle.clone());
    harness.manager = ThreadManager::new(
        &harness.config,
        harness.manager.auth_manager(),
        harness.manager.get_models_manager(),
        crate::CodexAppsToolsCache::default(),
        SessionSource::Exec,
        harness.manager.environment_manager(),
        Arc::new(extensions.build()),
        Arc::new(crate::test_support::EmptyUserInstructionsProvider),
        /*analytics_events_client*/ None,
        harness.manager.image_store(),
        crate::thread_manager::thread_store_from_config(&harness.config, harness.state_db.clone()),
        /*agent_graph_store*/ None,
        "mailbox-admission-test".to_string(),
        /*attestation_provider*/ None,
        /*external_time_provider*/ None,
    );
    let (parent_id, parent) = harness.start_paginated_thread().await;
    let control = parent
        .session
        .services
        .local_agent_runtime
        .control(parent.session.session_id());
    let child =
        spawn_v2_reload_test_child(&control, harness.config.clone(), &parent, "worker").await;
    let child_id = child.thread_id;
    let thread = harness.manager.get_thread(child_id).await.unwrap();
    thread
        .inject_response_items(vec![assistant_message(
            "child persisted",
            Some(MessagePhase::FinalAnswer),
        )])
        .await
        .unwrap();
    // This fixture has no task runner to finish the initial turn.
    thread.session.mark_interrupted();
    *thread.session.active_turn.lock().await = None;
    assert_eq!(
        harness.manager.try_evict_v2_thread(thread).await.unwrap(),
        ThreadEvictionOutcome::Evicted,
    );
    lifecycle.0.lock().unwrap().clear();

    let state = control.runtime.upgrade().unwrap();
    // Discard the notifications from initial root/child publication.
    state.mailbox_admission.publishing.notified().await;
    let pause = Arc::new(AdmissionPause {
        stage,
        entered: Notify::new(),
        release: Notify::new(),
    });
    *state.mailbox_admission.pause.lock().unwrap() = Some(pause.clone());
    let mut communication = InterAgentCommunication::new(
        AgentPath::root(),
        child.metadata.agent_path.unwrap(),
        Vec::new(),
        "mail admitted while unloaded".to_string(),
        /*trigger_turn*/ false,
    );
    let publication_id = ResponseItemId::new("amsg");
    communication.id = Some(publication_id.clone());
    let message_id = communication.message_id.clone().unwrap();
    let late_communication = communication.clone();
    let sender_control = control.clone();
    let mut send = Some(tokio::spawn(async move {
        sender_control
            .send_inter_agent_communication_without_capacity_check(
                child_id,
                communication,
                AgentCommunicationContext::new(AgentCommunicationKind::Message, parent_id),
                TurnStartOptions::default(),
            )
            .await
    }));
    timeout(Duration::from_secs(10), pause.entered.notified())
        .await
        .unwrap();

    // A stalled admission used to hold the global map read guard. This synchronous
    // assertion fails on that implementation; no scheduler/timing assumption is needed.
    {
        let mut threads = state
            .threads
            .try_write()
            .expect("unrelated map writers must proceed");
        let unrelated = ThreadId::new();
        threads.insert(unrelated, parent.clone());
        assert!(threads.remove(&unrelated).is_some());
        assert!(!threads.contains_key(&child_id));
    }
    let expected_phase = match stage {
        AdmissionStage::BeforeWrite => None,
        AdmissionStage::BeforeRetention => Some(AgentMailboxPhase::Ready),
    };
    assert_eq!(
        harness
            .state_db
            .as_ref()
            .unwrap()
            .get_agent_message(&message_id)
            .await
            .unwrap()
            .map(|row| row.phase),
        expected_phase,
    );

    let reload_control = control.clone();
    let reload_config = harness.config.clone();
    let reload = tokio::spawn(async move {
        reload_control
            .ensure_v2_agent_loaded(reload_config, child_id, /*parent*/ None)
            .await
    });
    timeout(
        Duration::from_secs(10),
        state.mailbox_admission.publishing.notified(),
    )
    .await
    .unwrap();
    // Publication has reached the recipient gate. It cannot become visible or
    // begin recovery while either DB admission or delivery retention is incomplete.
    assert!(!state.threads.read().await.contains_key(&child_id));
    assert!(!reload.is_finished());
    // An arrival queued behind publication must recheck residency and deliver to
    // the newly live session, rather than leave a Ready row after recovery.
    let late_sender = if matches!(cancel, Cancel::LateSender) {
        let late_control = control.clone();
        let mut late_communication = late_communication;
        late_communication.message_id = Some("late-mail".to_string());
        late_communication.id = Some(ResponseItemId::with_suffix("amsg", "late"));
        Some(tokio::spawn(async move {
            late_control
                .send_inter_agent_communication_without_capacity_check(
                    child_id,
                    late_communication,
                    AgentCommunicationContext::new(AgentCommunicationKind::Message, parent_id),
                    TurnStartOptions::default(),
                )
                .await
        }))
    } else {
        None
    };
    tokio::task::yield_now().await;
    match cancel {
        Cancel::Sender => {
            let send = send.take().unwrap();
            send.abort();
            assert!(send.await.unwrap_err().is_cancelled());
        }
        Cancel::Neither | Cancel::Reload | Cancel::LateSender => {}
    }
    let reload = if matches!(cancel, Cancel::Reload) {
        reload.abort();
        match reload.await {
            Err(err) => assert!(err.is_cancelled()),
            Ok(_) => panic!("reload should be cancelled while waiting for admission"),
        }
        None
    } else {
        Some(reload)
    };
    *state.mailbox_admission.pause.lock().unwrap() = None;
    pause.release.notify_one();
    let pin = if let Some(reload) = reload {
        timeout(Duration::from_secs(10), reload)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    } else {
        timeout(
            Duration::from_secs(10),
            control.ensure_v2_agent_loaded(harness.config.clone(), child_id, /*parent*/ None),
        )
        .await
        .unwrap()
        .unwrap()
    };
    if let Some(send) = send {
        assert_eq!(
            timeout(Duration::from_secs(10), send)
                .await
                .unwrap()
                .unwrap()
                .unwrap(),
            message_id,
        );
    }
    // Reacquiring the gate proves a cancelled sender released its reservation.
    drop(
        timeout(
            Duration::from_secs(10),
            state.mailbox_admission.lock(child_id),
        )
        .await
        .unwrap(),
    );
    let expected_count = if let Some(late_sender) = late_sender {
        timeout(Duration::from_secs(10), late_sender)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        2
    } else {
        1
    };
    let reloaded = harness.manager.get_thread(child_id).await.unwrap();
    assert_eq!(
        harness
            .state_db
            .as_ref()
            .unwrap()
            .get_agent_message(&message_id)
            .await
            .unwrap()
            .unwrap()
            .phase,
        AgentMailboxPhase::Submitted,
    );
    // Recovery again must neither duplicate delivery nor republish its lifecycle.
    control
        .recover_inter_agent_communications(child_id)
        .await
        .unwrap();
    // Wait for the submission loop to hand the recovered mail to the queue.
    let mails = timeout(Duration::from_secs(10), async {
        let mut mails = Vec::new();
        loop {
            mails.extend(reloaded.session.input_queue.drain_mailbox().await);
            if mails.len() >= expected_count {
                break mails;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(mails.len(), expected_count);
    let mut ids: Vec<_> = mails
        .iter()
        .map(|mail| mail.communication.message_id.clone().unwrap())
        .collect();
    ids.sort();
    let mut expected_ids = vec![message_id.clone()];
    let mut expected_events = vec![("published", publication_id.clone())];
    if matches!(cancel, Cancel::LateSender) {
        expected_ids.push("late-mail".to_string());
        expected_events.push(("published", ResponseItemId::with_suffix("amsg", "late")));
    }
    expected_ids.sort();
    assert_eq!(ids, expected_ids);
    assert_eq!(*lifecycle.0.lock().unwrap(), expected_events);
    expected_events.extend(
        mails
            .iter()
            .map(|mail| ("consumed", mail.communication.id.clone().unwrap())),
    );
    let turn = reloaded.session.new_default_turn().await;
    for mail in mails {
        reloaded
            .session
            .record_inter_agent_communication(&turn, turn.model_info(), mail.communication)
            .await;
    }
    assert_eq!(*lifecycle.0.lock().unwrap(), expected_events);
    assert!(
        reloaded
            .session
            .input_queue
            .drain_mailbox()
            .await
            .is_empty()
    );
    assert!(
        control
            .runtime
            .mailboxes
            .durable_delivery(child_id, &message_id)
            .is_none()
    );
    drop(pin);
}

#[tokio::test]
async fn admission_before_write_does_not_block_other_writers_or_lose_reload_mail() {
    admission_reload(AdmissionStage::BeforeWrite, Cancel::Neither).await;
}

#[tokio::test]
async fn admission_before_retention_cannot_be_overtaken_by_reload() {
    admission_reload(AdmissionStage::BeforeRetention, Cancel::Neither).await;
}

#[tokio::test]
async fn cancelled_admission_before_write_finishes_before_reload() {
    admission_reload(AdmissionStage::BeforeWrite, Cancel::Sender).await;
}

#[tokio::test]
async fn cancelled_admission_after_write_retains_one_delivery() {
    admission_reload(AdmissionStage::BeforeRetention, Cancel::Sender).await;
}

#[tokio::test]
async fn admission_queued_behind_publication_uses_the_live_session() {
    admission_reload(AdmissionStage::BeforeWrite, Cancel::LateSender).await;
}

#[tokio::test]
async fn cancelled_reload_waiter_does_not_strand_admission() {
    admission_reload(AdmissionStage::BeforeWrite, Cancel::Reload).await;
}
