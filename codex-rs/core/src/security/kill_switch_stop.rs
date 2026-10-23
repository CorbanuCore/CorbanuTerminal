//! Issue #507: turning the security kill switch on stops what is already
//! running, in every session of this process on the Corbanu home (each pane,
//! and the child agents they started), not only the next turn (#445).
//!
//! - The turn in progress is interrupted: a reply still streaming is
//!   cancelled, approvals it is waiting for end, and the commands it started
//!   are stopped with it.
//! - Background terminals (unified exec processes) and an open realtime
//!   conversation are ended too.
//! - A tool call that reaches the router while the switch is on is not run
//!   ([`refuse_tool_call`]), so nothing slips through between the switch and
//!   the interruption.
//!
//! Another Corbanu Terminal process running on the same home takes the saved
//! switch within about a second (#507, Travis 2026-10-12: its watch of the
//! saved state, `effective_policy::transition::kill_switch_watch`) and stops
//! its work the same way.
//!
//! Agent messages that arrive while the switch is on keep waiting (#445), and
//! turning it off starts no turn for them either: they go with the next turn
//! someone starts.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::Weak;

use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::TurnAbortReason;
use codex_protocol::protocol::WarningEvent;

use super::effective_policy::EffectivePolicyView;
use super::transition::RevocationSink;
use crate::function_tool::FunctionCallError;
use crate::session::session::Session;

/// Shown in a session whose work the kill switch stopped.
pub const KILL_SWITCH_STOPPED: &str = "Stopped: the security kill switch was turned on, so Corbanu ended this reply and the commands it was running, and sends nothing to the model until the switch is turned off. Turning it off restarts nothing: messages and tasks from other agents wait for the next turn someone starts (a Claude pane keeps only the newest task; a task for a Corbanu pane you opened yourself in /panes is refused instead). To turn it off, open /security, press g, choose \"Turn the kill switch off\" and confirm.";

/// What the model is told for a tool call that was not run.
const KILL_SWITCH_TOOL_REFUSAL: &str = "Not run: the security kill switch is on, so Corbanu runs no tool calls until a person turns it off in /security.";

/// The revocation sink of one session: it stops the session's work when the
/// kill switch is turned on. Owned by the session (in its extension data);
/// the policy tree holds it weakly.
pub(crate) struct KillSwitchStop {
    session: Weak<Session>,
    /// The session's policy tree.
    policy: EffectivePolicyView,
    /// The kill-switch event this session last saw: only a new one stops it.
    seen: Mutex<Option<String>>,
    /// The commit runs on a plain thread; the stop runs on the session's runtime.
    runtime: tokio::runtime::Handle,
}

impl KillSwitchStop {
    pub(crate) fn new(
        session: Weak<Session>,
        policy: EffectivePolicyView,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        let seen = Mutex::new(policy.kill_switch_event());
        Self {
            session,
            policy,
            seen,
            runtime,
        }
    }
}

impl RevocationSink for KillSwitchStop {
    fn revoke(&self) {
        // Sinks are also told of "Revoke all" and stricter levels: only a
        // kill switch turned on since this session last looked stops it.
        let event = self.policy.kill_switch_event();
        let fresh = {
            let mut seen = self.seen.lock().unwrap_or_else(PoisonError::into_inner);
            let fresh = event.is_some() && *seen != event;
            *seen = event;
            fresh
        };
        if !fresh {
            return;
        }
        let Some(session) = self.session.upgrade() else {
            return;
        };
        self.runtime.spawn(async move { stop(&session).await });
    }
}

/// Interrupt the session's turn, telling the person why, and end its
/// background terminals.
async fn stop(session: &Arc<Session>) {
    let turn_context = session
        .active_turn
        .lock()
        .await
        .as_ref()
        .and_then(|turn| turn.task.as_ref())
        .map(|task| Arc::clone(&task.turn_context));
    if let Some(turn_context) = turn_context {
        tracing::info!(
            target: "codex_core::security::transition",
            turn_id = turn_context.sub_id.as_str(),
            "kill switch on: interrupting the turn in progress"
        );
        session
            .send_event(
                turn_context.as_ref(),
                EventMsg::Warning(WarningEvent {
                    message: KILL_SWITCH_STOPPED.to_string(),
                }),
            )
            .await;
        session.abort_all_tasks(TurnAbortReason::Interrupted).await;
    }
    session.close_unified_exec_processes().await;
    if session.conversation.running_state().await.is_some() {
        let sub_id = session.next_internal_sub_id();
        crate::realtime_conversation::handle_close(session, sub_id).await;
    }
}

/// A tool call is not run while the kill switch is on (anywhere in the
/// session's policy tree).
pub(crate) fn refuse_tool_call(session: &Session) -> Result<(), FunctionCallError> {
    if session
        .services
        .agent_control
        .effective_security_policy()
        .kill_switch_event()
        .is_some()
    {
        Err(FunctionCallError::RespondToModel(
            KILL_SWITCH_TOOL_REFUSAL.to_string(),
        ))
    } else {
        Ok(())
    }
}
