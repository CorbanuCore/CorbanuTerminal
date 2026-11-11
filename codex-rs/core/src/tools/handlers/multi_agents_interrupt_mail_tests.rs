//! #1952 (D485): a spawned agent whose first turn is interrupted before it records its first
//! message, and which is then unloaded, must not get that message again when a queue-only
//! `send_message` reloads it. Mailbox recovery requeues a `Submitted` row of a replaced
//! session (crash recovery, #1058/#1545); the interrupted turn now cancels the rows it took but
//! never recorded, so there is nothing to requeue. Based on dec-1109d's reproduction
//! (`dec-1109d-redelivery-repro`, 20/20 re-delivered before this change).
use super::*;
use pretty_assertions::assert_eq;

fn agent_ops(manager: &ThreadManager, agent_id: ThreadId) -> Vec<Op> {
    manager
        .captured_ops()
        .into_iter()
        .filter(|(thread_id, _)| *thread_id == agent_id)
        .map(|(_, op)| op)
        .collect()
}

fn deliveries_of(ops: &[Op], message_id: &str) -> usize {
    ops.iter()
        .filter(|op| {
            matches!(op, Op::InterAgentCommunication { communication }
                if communication.message_id.as_deref() == Some(message_id))
        })
        .count()
}

struct Outcome {
    first_phase_before_reload: codex_state::AgentMailboxPhase,
    redelivered: usize,
    status_after_reload: AgentStatus,
}

async fn spawn_unload_and_reload(interrupt: bool) -> Outcome {
    let (mut session, mut turn) = make_session_and_context().await;
    let mut config = (*turn.config).clone();
    config.multi_agent_v2.max_concurrent_threads_per_session = 4;
    config
        .features
        .enable(Feature::MultiAgentV2)
        .expect("test config should allow feature update");
    config
        .features
        .enable(Feature::Sqlite)
        .expect("test config should allow sqlite");
    let state_db = init_state_db(&config)
        .await
        .expect("sqlite state db should initialize");
    let manager = ThreadManager::with_models_provider_home_and_state_for_tests(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
        config.codex_home.to_path_buf(),
        Arc::new(codex_exec_server::EnvironmentManager::default_for_tests()),
        Some(state_db.clone()),
    );
    let root = manager
        .start_thread(StartThreadOptions::new(config.clone()))
        .await
        .expect("root thread should start");
    session.services.agent_control = manager.agent_control();
    session.thread_id = root.thread_id;
    set_turn_config(&mut turn, config);
    let session = Arc::new(session);
    let turn = Arc::new(turn);
    SpawnAgentHandlerV2::default()
        .handle(invocation(
            session.clone(),
            turn.clone(),
            "spawn_agent",
            function_payload(json!({"message": "boot worker", "task_name": "worker"})),
        ))
        .await
        .expect("spawn worker");
    let agent_id = session
        .services
        .agent_control
        .resolve_agent_reference(session.thread_id, &turn.session_source, "worker")
        .await
        .expect("worker should resolve");
    let first_id = agent_ops(&manager, agent_id)
        .iter()
        .find_map(|op| match op {
            Op::InterAgentCommunication { communication } if communication.trigger_turn => {
                communication.message_id.clone()
            }
            _ => None,
        })
        .expect("the spawn hands the first message over");
    if interrupt {
        // The parent interrupts the just-spawned agent (D435 pending path).
        let output = InterruptAgentHandler
            .handle(invocation(
                session.clone(),
                turn.clone(),
                "interrupt_agent",
                function_payload(json!({"target": "worker"})),
            ))
            .await
            .expect("interrupt_agent should succeed");
        let _ = expect_text_output(output);
    }
    // Wait until the first turn has ended (interrupted, or on its own).
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let status = manager.agent_control().get_status(agent_id).await;
        if !matches!(status, AgentStatus::PendingInit | AgentStatus::Running) {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "agent did not settle"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    let first_phase_before_reload = state_db
        .get_agent_message(&first_id)
        .await
        .expect("mailbox row")
        .expect("the first message has a mailbox row")
        .phase;
    let ops_before_reload = agent_ops(&manager, agent_id);

    let unloaded = manager
        .remove_thread(&agent_id)
        .await
        .expect("worker thread should be loaded before removal");
    unloaded
        .submit(Op::Shutdown {})
        .await
        .expect("removed worker should accept shutdown");
    unloaded.wait_until_terminated().await;
    // Queue-only: on its own this never starts a turn.
    SendMessageHandlerV2
        .handle(invocation(
            session.clone(),
            turn.clone(),
            "send_message",
            function_payload(json!({"target": "worker", "message": "for later"})),
        ))
        .await
        .expect("send_message should reload the worker");
    tokio::time::sleep(Duration::from_millis(2000)).await;
    let ops_after_reload: Vec<Op> = agent_ops(&manager, agent_id)
        .into_iter()
        .skip(ops_before_reload.len())
        .collect();
    let redelivered = deliveries_of(&ops_after_reload, &first_id);
    let status_after_reload = manager.agent_control().get_status(agent_id).await;
    Outcome {
        first_phase_before_reload,
        redelivered,
        status_after_reload,
    }
}

#[tokio::test]
async fn reload_does_not_redeliver_an_interrupted_first_message() {
    let outcome = spawn_unload_and_reload(/*interrupt*/ true).await;
    // Cancelled when the interrupt beat the turn to recording the message; quarantined
    // (UnknownOutcome) when the turn recorded it first. Before #1952 the row stayed Submitted.
    assert!(
        matches!(
            outcome.first_phase_before_reload,
            codex_state::AgentMailboxPhase::Cancelled
                | codex_state::AgentMailboxPhase::UnknownOutcome
        ),
        "unexpected phase after interrupt: {:?}",
        outcome.first_phase_before_reload
    );
    assert_eq!(outcome.redelivered, 0);
    assert_eq!(outcome.status_after_reload, AgentStatus::PendingInit);
}

#[tokio::test]
async fn reload_does_not_redeliver_a_first_message_whose_turn_ended_on_its_own() {
    let outcome = spawn_unload_and_reload(/*interrupt*/ false).await;
    assert_eq!(
        outcome.first_phase_before_reload,
        codex_state::AgentMailboxPhase::Completed
    );
    assert_eq!(outcome.redelivered, 0);
    assert_eq!(outcome.status_after_reload, AgentStatus::PendingInit);
}
