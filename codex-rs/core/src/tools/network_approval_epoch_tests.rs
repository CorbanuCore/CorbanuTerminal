//! #945: a network approval answered after a security transition is for a
//! stale prompt. These tests drive the real session path with a controlled
//! order: the prompt is shown, the level rises, then the answer arrives.

use super::*;
use crate::security::EffectivePolicyInitialization;
use crate::security::PersistedHumanSecurityState;
use crate::security::TrustedSecurityController;
use crate::session::TurnInput;
use crate::session::tests::make_session_and_context_with_rx;
use crate::session::turn_context::TurnContext;
use crate::state::TaskKind;
use crate::tasks::SessionTask;
use crate::tasks::SessionTaskContext;
use crate::tasks::SessionTaskResult;
use codex_network_proxy::NetworkPolicyRequestArgs;
use codex_protocol::SessionId;
use codex_protocol::approvals::NetworkPolicyAmendment;
use codex_protocol::protocol::ExecApprovalRequestEvent;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use pretty_assertions::assert_eq;
use std::time::Duration;
use tokio::task::JoinHandle;

const HOST: &str = "stale-prompt.example";
const WAIT: Duration = Duration::from_secs(10);

/// Keeps a turn active, so network requests have a turn to prompt in.
#[derive(Clone, Copy)]
struct WaitingTask;

impl SessionTask for WaitingTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.issue_945_waiting"
    }

    async fn run(
        self: Arc<Self>,
        _session: Arc<SessionTaskContext>,
        _ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        cancellation_token.cancelled().await;
        Ok(None)
    }
}

struct Fixture {
    session: Arc<Session>,
    events: async_channel::Receiver<Event>,
    controller: TrustedSecurityController,
    environment_id: String,
}

impl Fixture {
    /// A Permissive session with a running turn that may ask about the
    /// network.
    async fn new() -> Self {
        let (session, turn_context, events) = make_session_and_context_with_rx().await;
        assert!(
            permission_profile_allows_network_approval_flow(&turn_context.permission_profile())
                && allows_network_approval_flow(turn_context.approval_policy.value()),
            "the test turn may ask about the network"
        );
        let controller = TrustedSecurityController::initialize(
            &session.services.agent_control.effective_security_policy(),
            PersistedHumanSecurityState::new(
                SecuritySettings::new(SecurityLevel::Permissive),
                PolicyPrincipal::new(PrincipalKind::Human, "human:issue-945").unwrap(),
                RevocationState::new(),
            )
            .unwrap(),
            session.thread_id,
            SessionId::from(session.thread_id),
            EffectivePolicyInitialization::Root,
        )
        .expect("initialize the session's policy tree");
        let environment_id = turn_context
            .environments
            .primary()
            .expect("a primary environment")
            .environment_id
            .clone();
        // A remembered host could be saved here.
        std::fs::create_dir_all(session.codex_home().await).unwrap();
        session
            .spawn_task(turn_context, Vec::new(), WaitingTask)
            .await;
        Self {
            session,
            events,
            controller,
            environment_id,
        }
    }

    fn request(&self) -> JoinHandle<NetworkDecision> {
        let service = Arc::clone(&self.session.services.network_approval);
        let session = Arc::clone(&self.session);
        let request = NetworkPolicyRequest::new(NetworkPolicyRequestArgs {
            protocol: NetworkProtocol::Http,
            host: HOST.to_string(),
            port: 80,
            environment_id: None,
            client_addr: None,
            method: Some("GET".to_string()),
            command: None,
            exec_policy_hint: None,
        });
        tokio::spawn(async move { service.handle_inline_policy_request(session, request).await })
    }

    /// The next network prompt, or `None` when `decision` finished first.
    async fn prompt_or_decision(
        &self,
        decision: &mut JoinHandle<NetworkDecision>,
    ) -> Result<ExecApprovalRequestEvent, NetworkDecision> {
        let prompt = async {
            loop {
                let event = self.events.recv().await.expect("event channel open");
                if let EventMsg::ExecApprovalRequest(prompt) = event.msg {
                    return prompt;
                }
            }
        };
        tokio::time::timeout(WAIT, async {
            tokio::select! {
                prompt = prompt => Ok(prompt),
                decision = decision => Err(decision.expect("request task")),
            }
        })
        .await
        .expect("a prompt or a decision")
    }

    async fn prompt(&self, decision: &mut JoinHandle<NetworkDecision>) -> ExecApprovalRequestEvent {
        match self.prompt_or_decision(decision).await {
            Ok(prompt) => prompt,
            Err(decision) => {
                panic!("expected a network prompt, the request was decided: {decision:?}")
            }
        }
    }

    /// Waits until another request has joined the open prompt for the host,
    /// so it is certain to receive that prompt's answer.
    async fn waiter_joined_open_prompt(&self) {
        let mut joined = {
            let approvals = self
                .session
                .services
                .network_approval
                .pending_host_approvals
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut open = approvals.iter().filter(|(key, _)| key.host.host == HOST);
            let (_, pending) = open.next().expect("an open prompt for the host");
            assert!(open.next().is_none(), "one open prompt for the host");
            pending.joined_waiters.subscribe()
        };
        tokio::time::timeout(WAIT, joined.wait_for(|joined| *joined >= 1))
            .await
            .expect("a request joined the open prompt")
            .expect("the open prompt is alive");
    }

    async fn answer(&self, prompt: &ExecApprovalRequestEvent, decision: ReviewDecision) {
        self.session
            .notify_approval(&prompt.effective_approval_id(), decision)
            .await;
    }

    /// A stricter level is now in force; like the cross-window raise, the
    /// revocation sinks have not run yet.
    fn raise_level(&self) {
        let change = self
            .controller
            .confirm_level_change(SecurityLevel::Moderate, RevocationState::new())
            .unwrap();
        self.controller.apply_confirmed_change(change).unwrap();
    }

    async fn host_approved_for_session(&self) -> bool {
        let key = HostApprovalKey {
            environment_id: self.environment_id.clone(),
            host: HOST.to_string(),
            protocol: "http",
            port: 80,
        };
        self.session
            .services
            .network_approval
            .session_approved_hosts
            .lock()
            .await
            .contains(&key, self.session.services.authority_marker())
    }
}

async fn decided(decision: JoinHandle<NetworkDecision>) -> NetworkDecision {
    tokio::time::timeout(WAIT, decision)
        .await
        .expect("the request was decided")
        .expect("request task")
}

/// "Allow and don't ask again" for the host.
fn remember() -> ReviewDecision {
    ReviewDecision::NetworkPolicyAmendment {
        network_policy_amendment: NetworkPolicyAmendment {
            host: HOST.to_string(),
            action: NetworkPolicyRuleAction::Allow,
        },
    }
}

fn denied() -> NetworkDecision {
    NetworkDecision::deny("not_allowed")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_for_session_answer_after_a_level_change_is_asked_again() {
    let fixture = Fixture::new().await;
    let mut decision = fixture.request();
    let shown = fixture.prompt(&mut decision).await;

    fixture.raise_level();
    fixture
        .answer(&shown, ReviewDecision::ApprovedForSession)
        .await;

    // The answer was for the Permissive prompt: asked again under Moderate.
    let again = fixture.prompt(&mut decision).await;
    assert_ne!(again.call_id, shown.call_id);
    assert!(
        again
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("security level changed")),
        "{:?}",
        again.reason
    );
    assert!(
        !fixture.host_approved_for_session().await,
        "the stale answer added no host"
    );

    fixture.answer(&again, ReviewDecision::denied("no")).await;
    assert_eq!(decided(decision).await, denied());
    assert!(!fixture.host_approved_for_session().await);
}

/// A second request for the same host waits on the first one's prompt; when
/// that answer is stale, both are decided again, under one new prompt.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_waiters_on_a_stale_prompt_are_decided_again() {
    let fixture = Fixture::new().await;
    let mut first = fixture.request();
    let shown = fixture.prompt(&mut first).await;
    let mut second = fixture.request();
    // The second request joins the open prompt: no prompt of its own.
    fixture.waiter_joined_open_prompt().await;
    assert!(fixture.events.try_recv().is_err(), "no second prompt");
    assert!(!second.is_finished());

    fixture.raise_level();
    fixture
        .answer(&shown, ReviewDecision::ApprovedForSession)
        .await;
    let again = fixture.prompt(&mut first).await;
    assert_ne!(again.call_id, shown.call_id);
    // #1054: both stale requests are asked again; whichever reaches the new
    // round second joins the new prompt. Answer only once it has, so a late
    // waiter cannot miss this answer and ask a third time.
    fixture.waiter_joined_open_prompt().await;
    fixture.answer(&again, ReviewDecision::denied("no")).await;
    assert_eq!(decided(first).await, denied());
    assert_eq!(
        fixture.prompt_or_decision(&mut second).await.err(),
        Some(denied()),
        "the waiter was decided under the new prompt"
    );
    assert!(!fixture.host_approved_for_session().await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_answer_under_the_new_level_counts() {
    let fixture = Fixture::new().await;
    let mut decision = fixture.request();
    let shown = fixture.prompt(&mut decision).await;
    fixture.raise_level();
    fixture.answer(&shown, ReviewDecision::Approved).await;

    // A one-time approval of the stale prompt is asked again too; the
    // answer under the new level is stored.
    let again = fixture.prompt(&mut decision).await;
    fixture
        .answer(&again, ReviewDecision::ApprovedForSession)
        .await;
    assert_eq!(decided(decision).await, NetworkDecision::Allow);
    assert!(fixture.host_approved_for_session().await);

    // And used: the next request passes without a prompt.
    let mut next = fixture.request();
    assert_eq!(
        fixture.prompt_or_decision(&mut next).await.err(),
        Some(NetworkDecision::Allow)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_remembered_host_answer_after_a_level_change_is_not_saved() {
    let fixture = Fixture::new().await;
    let mut decision = fixture.request();
    let shown = fixture.prompt(&mut decision).await;

    fixture.raise_level();
    fixture.answer(&shown, remember()).await;

    let again = fixture.prompt(&mut decision).await;
    fixture.answer(&again, ReviewDecision::denied("no")).await;
    assert_eq!(decided(decision).await, denied());
    let codex_home = fixture.session.codex_home().await;
    let rules = || {
        std::fs::read_to_string(crate::exec_policy::default_policy_path(&codex_home))
            .unwrap_or_default()
    };
    assert!(!rules().contains(HOST), "no rule was saved: {}", rules());
    assert!(!fixture.host_approved_for_session().await);

    // The same answer under the level in force is saved.
    let mut decision = fixture.request();
    let shown = fixture.prompt(&mut decision).await;
    fixture.answer(&shown, remember()).await;
    assert_eq!(decided(decision).await, NetworkDecision::Allow);
    assert!(rules().contains(HOST), "{}", rules());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_945_host_approved_before_a_level_change_ends_with_it() {
    let fixture = Fixture::new().await;
    let mut decision = fixture.request();
    let shown = fixture.prompt(&mut decision).await;
    fixture
        .answer(&shown, ReviewDecision::ApprovedForSession)
        .await;
    assert_eq!(decided(decision).await, NetworkDecision::Allow);
    // Permissive as before: the host is approved for the session.
    let mut next = fixture.request();
    assert_eq!(
        fixture.prompt_or_decision(&mut next).await.err(),
        Some(NetworkDecision::Allow)
    );

    // The level rises; before the revocation sink runs, the approval is
    // already gone.
    fixture.raise_level();
    let mut after = fixture.request();
    let prompt = fixture.prompt(&mut after).await;
    fixture.answer(&prompt, ReviewDecision::denied("no")).await;
    assert_eq!(decided(after).await, denied());
}

#[tokio::test]
async fn issue_945_session_hosts_follow_the_policy_marker() {
    let key = HostApprovalKey {
        environment_id: "local".to_string(),
        host: HOST.to_string(),
        protocol: "https",
        port: 443,
    };
    let mut hosts = SessionApprovedHosts::default();
    let shown = Some((1, 0));
    let raised = Some((2, 0));
    // Answered after the level changed: not stored.
    assert!(!hosts.insert(key.clone(), shown, raised));
    assert!(!hosts.contains(&key, raised));
    // Answered under the policy in force: stored, and dropped once it moves.
    assert!(hosts.insert(key.clone(), raised, raised));
    assert!(hosts.contains(&key, raised));
    assert!(!hosts.contains(&key, Some((3, 0))));

    // A sync copies only hosts still valid for the source.
    let source = NetworkApprovalService::default();
    source
        .session_approved_hosts
        .lock()
        .await
        .insert(key.clone(), shown, shown);
    let target = NetworkApprovalService::default();
    source
        .sync_session_approved_hosts_to(&target, shown, raised)
        .await;
    assert!(
        target
            .session_approved_hosts
            .lock()
            .await
            .contains(&key, raised)
    );
    source
        .sync_session_approved_hosts_to(&target, raised, raised)
        .await;
    assert!(
        !target
            .session_approved_hosts
            .lock()
            .await
            .contains(&key, raised)
    );
}
