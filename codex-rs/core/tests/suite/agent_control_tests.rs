//! D682: policy-less host controllers fail closed (D530), including cold resume.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use codex_core::AgentConfigUpdate;
use codex_core::AgentControl;
use codex_core::AgentExecutionGuard;
use codex_core::AgentInfo;
use codex_core::AgentTarget;
use codex_core::AgentTurnOutcome;
use codex_core::DeliveryReceipt;
use codex_core::GuardianRootSnapshot;
use codex_core::LiveAgent;
use codex_core::PendingAgentMessage;
use codex_core::RolloutBudgetReminder;
use codex_core::SendRequest;
use codex_core::SpawnRequest;
use codex_core::ThreadConfigSnapshot;
use codex_features::Feature;
use codex_protocol::AgentPath;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::TokenUsage;
use codex_rollout_trace::ThreadTraceContext;
use core_test_support::responses;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use futures::future::BoxFuture;
use pretty_assertions::assert_eq;
use test_case::test_case;

pub(super) struct TestAgentControl {
    thread_id: ThreadId,
    service_tier: Mutex<Option<String>>,
    mail: Mutex<Vec<PendingAgentMessage>>,
    mailbox_state: tokio::sync::watch::Sender<bool>,
    agents: Mutex<HashMap<String, ThreadId>>,
}

impl TestAgentControl {
    pub(super) fn new(thread_id: ThreadId) -> Self {
        Self {
            thread_id,
            service_tier: Mutex::default(),
            mail: Mutex::default(),
            mailbox_state: tokio::sync::watch::channel(/*init*/ false).0,
            agents: Mutex::new(HashMap::from([("/root".into(), thread_id)])),
        }
    }
}

impl AgentControl for TestAgentControl {
    fn identity(&self) -> SessionId {
        SessionId::from(self.thread_id)
    }

    fn resolve<'a>(
        &'a self,
        _caller: ThreadId,
        _parent: Option<ThreadId>,
        _source: &'a SessionSource,
        target: &'a str,
    ) -> BoxFuture<'a, CodexResult<ThreadId>> {
        Box::pin(async move {
            if let Some(id) = self.agents.lock().expect("agents lock").get(target) {
                return Ok(*id);
            }
            ThreadId::from_string(target)
                .map_err(|error| CodexErr::InvalidRequest(error.to_string()))
        })
    }

    fn spawn(
        &self,
        request: SpawnRequest,
    ) -> BoxFuture<'_, CodexResult<(LiveAgent, ThreadConfigSnapshot)>> {
        assert_eq!(request.caller, self.thread_id);
        Box::pin(async { Err(CodexErr::InvalidRequest("host spawn rejection".to_string())) })
    }

    fn send(&self, request: SendRequest) -> BoxFuture<'_, CodexResult<DeliveryReceipt>> {
        assert_eq!(request.caller, self.thread_id);
        Box::pin(async { Err(CodexErr::InvalidRequest("host send rejection".to_string())) })
    }

    fn take_mailbox(&self, _agent: ThreadId) -> Vec<PendingAgentMessage> {
        let messages = std::mem::take(&mut *self.mail.lock().expect("mail lock"));
        self.mailbox_state.send_if_modified(std::mem::take);
        messages
    }

    fn watch_mailbox(&self, _agent: ThreadId) -> tokio::sync::watch::Receiver<bool> {
        self.mailbox_state.subscribe()
    }

    fn ensure_child_loaded(
        &self,
        parent: ThreadId,
        child: ThreadId,
    ) -> BoxFuture<'_, CodexResult<()>> {
        assert_eq!(parent, self.thread_id);
        Box::pin(async move {
            Err(CodexErr::InvalidRequest(format!(
                "host resume rejection for {child}"
            )))
        })
    }

    fn interrupt(
        &self,
        _caller: ThreadId,
        _target: AgentTarget,
        _version: MultiAgentVersion,
    ) -> BoxFuture<'_, CodexResult<AgentInfo>> {
        panic!("unexpected agent interrupt")
    }

    fn list<'a>(
        &'a self,
        caller: ThreadId,
        _parent: Option<ThreadId>,
        _source: &'a SessionSource,
        _path_prefix: Option<&'a str>,
    ) -> BoxFuture<'a, CodexResult<Vec<LiveAgent>>> {
        assert_eq!(caller, self.thread_id);
        Box::pin(async { Err(CodexErr::InvalidRequest("host list rejection".to_string())) })
    }

    fn child_agent_paths(&self, _parent: ThreadId) -> BoxFuture<'_, Vec<AgentPath>> {
        Box::pin(async { Vec::new() })
    }

    fn check_turn_admission(
        &self,
        _version: MultiAgentVersion,
        _source: &SessionSource,
    ) -> CodexResult<()> {
        Ok(())
    }

    fn admit_turn(
        &self,
        _version: MultiAgentVersion,
        _source: &SessionSource,
    ) -> Option<AgentExecutionGuard> {
        None
    }

    fn record_usage(&self, _usage: TokenUsage) -> BoxFuture<'_, CodexResult<()>> {
        Box::pin(async { Ok(()) })
    }

    fn turn_finished<'a>(
        &'a self,
        _outcome: AgentTurnOutcome,
        _trace: &'a ThreadTraceContext,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }

    fn service_tier(&self) -> Option<String> {
        self.service_tier.lock().expect("service tier lock").clone()
    }

    fn propagate_config_update(&self, update: AgentConfigUpdate) {
        let AgentConfigUpdate::ServiceTier(tier) = update;
        *self.service_tier.lock().expect("service tier lock") = tier;
    }

    fn get_guardian_package(
        &self,
        _agent: ThreadId,
    ) -> BoxFuture<'_, Option<GuardianRootSnapshot>> {
        Box::pin(async { None })
    }

    fn pending_budget_reminder<'a>(
        &'a self,
        _agent: ThreadId,
        _window: &'a str,
    ) -> BoxFuture<'a, Option<RolloutBudgetReminder>> {
        Box::pin(async { None })
    }

    fn mark_budget_reminder_delivered<'a>(
        &'a self,
        _agent: ThreadId,
        _window: &'a str,
        _reminder: RolloutBudgetReminder,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

// D682 retires the upstream host-dispatch/mailbox/lifecycle success contracts.
// A controller without Corbanu's policy must never gain a runnable thread.
#[test_case(Feature::Collab, false; "new_collaboration")]
#[test_case(Feature::AgentMessageBoard, false; "new_mailbox")]
#[test_case(Feature::WakeOnAgentMessages, false; "new_mail_wake")]
#[test_case(Feature::Collab, true; "resumed_collaboration")]
#[test_case(Feature::AgentMessageBoard, true; "resumed_mailbox")]
#[test_case(Feature::WakeOnAgentMessages, true; "resumed_mail_wake")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn policyless_host_controller_refuses_startup(
    feature: Feature,
    resume: bool,
) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = responses::start_mock_server().await;
    let controls = Arc::new(Mutex::new(Vec::new()));
    let mut builder = test_codex()
        .with_config(move |config| {
            config
                .features
                .enable(Feature::MultiAgentV2)
                .expect("enable V2");
            config
                .features
                .enable(feature)
                .expect("enable host capability");
            config.service_tier = Some("priority".into());
        })
        .with_thread_manager({
            let controls = Arc::clone(&controls);
            move |manager| {
                manager.with_agent_control_factory(move |thread_id| {
                    let controller = Arc::new(TestAgentControl::new(thread_id));
                    controls
                        .lock()
                        .expect("controls lock")
                        .push(Arc::clone(&controller));
                    let control: Arc<dyn AgentControl> = controller;
                    async move { Ok(control) }
                })
            }
        });
    let result = if resume {
        let seed = test_codex().build_with_auto_env(&server).await?;
        seed.codex.ensure_rollout_materialized().await;
        seed.codex.flush_rollout().await?;
        builder.restart_with_auto_env(&server, &seed).await
    } else {
        builder.build_with_auto_env(&server).await
    };
    let error = result
        .err()
        .expect("policy-less host controller must not start");
    let error = error
        .downcast_ref::<CodexErr>()
        .expect("typed startup error");
    assert!(
        matches!(error.details(), CodexErrorDetails::UnsupportedOperation(message)
            if message == "this agent controller has no Corbanu security policy"),
        "{error:?}"
    );
    {
        let controls = controls.lock().expect("controls lock");
        assert_eq!(controls.len(), 1);
        assert_eq!(
            controls[0].service_tier(),
            None,
            "rejection must not publish root settings"
        );
        assert!(controls[0].mail.lock().expect("mail lock").is_empty());
    }
    assert!(
        server
            .received_requests()
            .await
            .expect("recorded requests")
            .iter()
            .all(|request| !request.url.path().ends_with("/responses"))
    );
    Ok(())
}
