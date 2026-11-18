//! D530: reject a provided non-local pane controller before entering child startup.
use super::*;
use crate::AgentConfigUpdate;
use crate::AgentControl;
use crate::AgentExecutionGuard;
use crate::AgentInfo;
use crate::AgentTarget;
use crate::AgentTurnOutcome;
use crate::DeliveryReceipt;
use crate::GuardianRootSnapshot;
use crate::LiveAgent;
use crate::PendingAgentMessage;
use crate::RolloutBudgetReminder;
use crate::SendRequest;
use crate::SpawnRequest;
use crate::ThreadConfigSnapshot;
use codex_protocol::SessionId;
use codex_protocol::protocol::MultiAgentVersion;
use codex_protocol::protocol::SessionConfiguredEvent;
use codex_rollout_trace::ThreadTraceContext;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;

/// The guard may inspect backend availability; no startup or agent operation is valid.
struct NonLocalPaneController;

impl AgentControl for NonLocalPaneController {
    fn identity(&self) -> SessionId {
        panic!("non-local pane must be refused before controller initialization")
    }

    fn resolve<'a>(
        &'a self,
        _caller: ThreadId,
        _parent: Option<ThreadId>,
        _source: &'a SessionSource,
        _target: &'a str,
    ) -> BoxFuture<'a, CodexResult<ThreadId>> {
        unreachable!("no agent operation before pane admission")
    }

    fn spawn(
        &self,
        _request: SpawnRequest,
    ) -> BoxFuture<'_, CodexResult<(LiveAgent, ThreadConfigSnapshot)>> {
        unreachable!("no child spawn before pane admission")
    }

    fn send(&self, _request: SendRequest) -> BoxFuture<'_, CodexResult<DeliveryReceipt>> {
        unreachable!("no agent operation before pane admission")
    }

    fn take_mailbox(&self, _agent: ThreadId) -> Vec<PendingAgentMessage> {
        unreachable!("no mailbox consumption before pane admission")
    }

    fn watch_mailbox(&self, _agent: ThreadId) -> tokio::sync::watch::Receiver<bool> {
        unreachable!("no mailbox subscription before pane admission")
    }

    fn ensure_child_loaded(
        &self,
        _parent: ThreadId,
        _child: ThreadId,
    ) -> BoxFuture<'_, CodexResult<()>> {
        unreachable!("no child loading before pane admission")
    }

    fn interrupt(
        &self,
        _caller: ThreadId,
        _target: AgentTarget,
        _version: MultiAgentVersion,
    ) -> BoxFuture<'_, CodexResult<AgentInfo>> {
        unreachable!("no agent operation before pane admission")
    }

    fn list<'a>(
        &'a self,
        _caller: ThreadId,
        _parent: Option<ThreadId>,
        _source: &'a SessionSource,
        _path_prefix: Option<&'a str>,
    ) -> BoxFuture<'a, CodexResult<Vec<LiveAgent>>> {
        unreachable!("no agent operation before pane admission")
    }

    fn child_agent_paths(&self, _parent: ThreadId) -> BoxFuture<'_, Vec<AgentPath>> {
        unreachable!("no agent operation before pane admission")
    }

    fn check_turn_admission(
        &self,
        _version: MultiAgentVersion,
        _source: &SessionSource,
    ) -> CodexResult<()> {
        unreachable!("no turn admission before pane admission")
    }

    fn admit_turn(
        &self,
        _version: MultiAgentVersion,
        _source: &SessionSource,
    ) -> Option<AgentExecutionGuard> {
        unreachable!("no execution before pane admission")
    }

    fn record_usage(&self, _usage: TokenUsage) -> BoxFuture<'_, CodexResult<()>> {
        unreachable!("no usage before pane admission")
    }

    fn turn_finished<'a>(
        &'a self,
        _outcome: AgentTurnOutcome,
        _trace: &'a ThreadTraceContext,
    ) -> BoxFuture<'a, ()> {
        unreachable!("no turn before pane admission")
    }

    fn service_tier(&self) -> Option<String> {
        unreachable!("no controller settings access before pane admission")
    }

    fn propagate_config_update(&self, _update: AgentConfigUpdate) {
        unreachable!("no controller settings publication before pane admission")
    }

    fn get_guardian_package(
        &self,
        _agent: ThreadId,
    ) -> BoxFuture<'_, Option<GuardianRootSnapshot>> {
        unreachable!("no guardian initialization before pane admission")
    }

    fn pending_budget_reminder<'a>(
        &'a self,
        _agent: ThreadId,
        _window: &'a str,
    ) -> BoxFuture<'a, Option<RolloutBudgetReminder>> {
        unreachable!("no budget initialization before pane admission")
    }

    fn mark_budget_reminder_delivered<'a>(
        &'a self,
        _agent: ThreadId,
        _window: &'a str,
        _reminder: RolloutBudgetReminder,
    ) -> BoxFuture<'a, ()> {
        unreachable!("no budget publication before pane admission")
    }
}

struct StartupInstructionsWitness(AtomicUsize);

impl codex_extension_api::ThreadInstructionsProvider for StartupInstructionsWitness {
    fn load_thread_instructions(&self) -> codex_extension_api::LoadInstructionsFuture<'_> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Default::default() })
    }
}

#[tokio::test]
async fn provided_nonlocal_thread_spawn_is_rejected_before_child_startup() {
    // D682 prevents creating a policy-less parent through public startup. Install
    // one inert synthetic parent to exercise the earlier Provided-controller guard
    // itself, independently of the later factory replacement path (#2562).
    let (mut session, turn) = make_session_and_context().await;
    let parent_id = session.thread_id;
    let metadata = SessionConfiguredEvent {
        session_id: session.session_id(),
        thread_id: parent_id,
        forked_from_id: None,
        parent_thread_id: None,
        thread_source: None,
        thread_name: None,
        model: "test".into(),
        model_provider_id: "test".into(),
        service_tier: None,
        approval_policy: codex_protocol::protocol::AskForApproval::Never,
        approvals_reviewer: codex_protocol::config_types::ApprovalsReviewer::User,
        permission_profile: turn.permission_profile(),
        active_permission_profile: None,
        cwd: turn.config.cwd.clone(),
        reasoning_effort: None,
        network_proxy: None,
        rollout_path: None,
    };
    session.services.agent_control = Arc::new(NonLocalPaneController);
    let (tx_sub, _rx_sub) = async_channel::bounded(1);
    let (_tx_event, rx_event) = async_channel::bounded(1);
    let parent = Arc::new(CodexThread::new(
        Arc::new(session),
        crate::session::SessionIo {
            tx_sub,
            rx_event,
            agent_status: tokio::sync::watch::channel(AgentStatus::PendingInit).1,
            session_loop_termination: crate::session::completed_session_loop_termination(),
        },
        ThreadStartupMetadata::from(&metadata),
        /*rollout_path*/ None,
        SessionSource::Cli,
    ));
    let home = tempdir().expect("test home");
    let mut config = turn.config.as_ref().clone();
    config.codex_home = home.path().join("unstarted-child-home").abs();
    config.cwd = home.path().abs();
    let manager = ThreadManager::with_models_provider_and_home_for_tests(
        CodexAuth::from_api_key("dummy"),
        config.model_provider.clone(),
        home.path().to_path_buf(),
        Arc::new(codex_exec_server::EnvironmentManager::default_for_tests()),
    );
    manager
        .state
        .threads
        .write()
        .await
        .insert(parent_id, parent);
    let mut created = manager.subscribe_thread_created();
    let child_id = ThreadId::new();
    let instructions = Arc::new(StartupInstructionsWitness(AtomicUsize::new(0)));
    let child_home = config.codex_home.clone();
    let error = tokio::time::timeout(
        Duration::from_secs(5),
        manager.start_thread(StartThreadOptions {
            reserved_thread_id: Some(child_id),
            environments: Some(Vec::new()),
            thread_instructions_provider: Some(instructions.clone()),
            session_source: Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
                parent_thread_id: parent_id,
                depth: 1,
                agent_path: Some(AgentPath::try_from("/root/worker").expect("agent path")),
                agent_nickname: None,
                agent_role: None,
                agent_class: None,
            })),
            ..StartThreadOptions::new(config)
        }),
    )
    .await
    .expect("pre-spawn refusal must not enter asynchronous child startup")
    .err()
    .expect("provided non-local controller must be refused");
    assert!(
        matches!(error.details(), CodexErrorDetails::UnsupportedOperation(message)
            if message == "this agent operation needs Corbanu's local agent backend"),
        "{error:?}"
    );
    assert_eq!(instructions.0.load(Ordering::SeqCst), 0);
    assert_eq!(manager.list_thread_ids().await, vec![parent_id]);
    assert!(manager.get_thread(child_id).await.is_err());
    assert!(matches!(
        created.try_recv(),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty)
    ));
    assert!(
        !child_home.exists(),
        "no child home or rollout may be created"
    );
    manager.remove_thread(&parent_id).await;
}
