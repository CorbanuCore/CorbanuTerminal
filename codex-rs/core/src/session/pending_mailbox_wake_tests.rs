use super::TurnInput;
use super::tests::make_session_and_context;
use crate::security::EffectivePolicyInitialization;
use crate::security::EffectivePolicyView;
use crate::security::PersistedHumanSecurityState;
use crate::security::TrustedSecurityController;
use crate::state::ActiveTurn;
use codex_protocol::AgentPath;
use codex_protocol::SessionId;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::turn_input::TurnStartOptions;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use futures::FutureExt;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use test_case::test_case;

#[tokio::test]
async fn pending_mailbox_wake_checks_idle_before_preparing_a_turn() {
    for busy in [true, false] {
        let (session, _turn_context) = make_session_and_context().await;
        let session = Arc::new(session);
        let active_turn = busy.then(ActiveTurn::default);
        let original_turn_state = active_turn
            .as_ref()
            .map(|turn| Arc::clone(&turn.turn_state));
        *session.active_turn.lock().await = active_turn;
        let mail = InterAgentCommunication::new(
            AgentPath::root(),
            AgentPath::try_from("/root/worker").expect("agent path"),
            Vec::new(),
            "continue the queued work".to_string(),
            /*trigger_turn*/ true,
        );
        session
            .input_queue
            .enqueue_mailbox_communication(
                mail.clone().into(),
                TurnStartOptions {
                    parent_turn_id: Some("parent-turn".to_string()),
                    ..Default::default()
                },
            )
            .await;

        // Turn preparation first locks session state. Keep it unavailable so one poll proves
        // the busy wake returns before preparation (and therefore before the auth read), while
        // the idle control still reaches preparation. This needs no timeout or native vault.
        let outcome = {
            let _state = session.state.lock().await;
            tokio::task::unconstrained(
                session.maybe_start_turn_for_pending_work_with_sub_id("mailbox-wake".to_string()),
            )
            .now_or_never()
        };
        assert_eq!(outcome, busy.then_some(()));

        {
            let active_turn = session.active_turn.lock().await;
            match (active_turn.as_ref(), original_turn_state.as_ref()) {
                (Some(turn), Some(original)) => assert!(Arc::ptr_eq(&turn.turn_state, original)),
                (Some(turn), None) => assert!(turn.task.is_none()),
                _ => panic!("a busy wake must preserve its turn; an idle wake must reserve one"),
            }
        }
        // Upstream's `TurnStartOptions` (which now carries the parent turn) has
        // no `PartialEq`; compare the parent turn it carries.
        let (items, start_options) = session.input_queue.drain_mailbox_input_items().await;
        assert_eq!(
            (items, start_options.parent_turn_id),
            (
                vec![TurnInput::InterAgentCommunication(mail.into())],
                Some("parent-turn".to_string()),
            )
        );
    }
}

#[test_case(false; "clears_own_reservation")]
#[test_case(true; "preserves_replacement_reservation")]
#[tokio::test]
async fn pending_mailbox_wake_refusal_clears_only_its_own_reservation(replace_reservation: bool) {
    let (session, _turn_context) = make_session_and_context().await;
    let session = Arc::new(session);
    let mail = InterAgentCommunication::new(
        AgentPath::root(),
        AgentPath::try_from("/root/worker").expect("agent path"),
        Vec::new(),
        "work held by the refusal".to_string(),
        /*trigger_turn*/ true,
    );
    session
        .input_queue
        .enqueue_mailbox_communication(
            mail.clone().into(),
            TurnStartOptions {
                parent_turn_id: Some("held-parent-turn".to_string()),
                ..Default::default()
            },
        )
        .await;

    // Stop after reservation but before turn preparation. No Interrupt is submitted:
    // only the later refusal can release this wake's reservation.
    let mut wake = Box::pin(tokio::task::unconstrained(
        session.maybe_start_turn_for_pending_work_with_sub_id("refused-wake".to_string()),
    ));
    let replacement = {
        let _state = session.state.lock().await;
        assert!(wake.as_mut().now_or_never().is_none());
        let reservation = {
            let active_turn = session
                .active_turn
                .try_lock()
                .expect("preparation released admission lock");
            let turn = active_turn.as_ref().expect("wake reserved an idle turn");
            assert!(turn.task.is_none());
            Arc::clone(&turn.turn_state)
        };

        // Install a stopped policy entirely in memory; the fixture uses dummy auth.
        let human = PolicyPrincipal::new(PrincipalKind::Human, "human:test-operator")
            .expect("human principal");
        let mut revocations = RevocationState::new();
        revocations
            .apply(
                &RevocationEvent::new(
                    human.clone(),
                    RevocationTarget::KillSwitch { active: true },
                    RevocationReason::KillSwitch,
                    /*created_at_unix_seconds*/ 100,
                )
                .expect("kill event"),
            )
            .expect("apply kill");
        let policy = EffectivePolicyView::default();
        let _controller = TrustedSecurityController::initialize(
            &policy,
            PersistedHumanSecurityState::new(
                SecuritySettings::new(SecurityLevel::Moderate),
                human,
                revocations,
            )
            .expect("stopped policy"),
            session.thread_id,
            SessionId::from(session.thread_id),
            EffectivePolicyInitialization::Root,
        )
        .expect("initialize policy");
        let client = (*session.services.model_client())
            .clone()
            .with_ingress_policy(SecurityLevel::Moderate, policy);
        session.services.replace_model_client(client);
        assert!(session.services.model_client().kill_switch_on());

        if replace_reservation {
            let turn = ActiveTurn::default();
            let replacement = Arc::clone(&turn.turn_state);
            assert!(!Arc::ptr_eq(&reservation, &replacement));
            *session
                .active_turn
                .try_lock()
                .expect("paused wake released admission lock") = Some(turn);
            Some(replacement)
        } else {
            None
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(10), wake)
        .await
        .expect("refused wake must finish without starting work");

    {
        let active_turn = session.active_turn.lock().await;
        match replacement {
            Some(replacement) => {
                let turn = active_turn
                    .as_ref()
                    .expect("replacement reservation survives");
                assert!(turn.task.is_none());
                assert!(Arc::ptr_eq(&turn.turn_state, &replacement));
            }
            None => assert!(
                active_turn.is_none(),
                "refusal must release its own reservation"
            ),
        }
    }
    let (items, start_options) = session.input_queue.drain_mailbox_input_items().await;
    assert_eq!(
        (items, start_options.parent_turn_id),
        (
            vec![TurnInput::InterAgentCommunication(mail.into())],
            Some("held-parent-turn".to_string()),
        )
    );
}
