//! Turn-scoped state and active turn metadata scaffolding.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tokio_util::task::AbortOnDropHandle;

use codex_extension_api::ExtensionData;
use codex_protocol::dynamic_tools::DynamicToolResponse;
use codex_protocol::protocol::TurnEnvironmentSelection;
use codex_protocol::request_permissions::RequestPermissionProfile;
use codex_protocol::request_permissions::RequestPermissionsResponse;
use codex_protocol::request_user_input::RequestUserInputResponse;
use codex_rmcp_client::ElicitationResponse;
use rmcp::model::RequestId;
use tokio::sync::oneshot;

use crate::agent::control::AgentExecutionGuard;
use crate::agent::control::NativeAutoTurnGuard;
use crate::mcp_tool_call::McpToolApprovalMetadata;
use crate::session::TurnInputQueue;
use crate::session::turn_context::TurnContext;
use crate::tasks::AnySessionTask;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::TokenUsage;

/// Metadata about the currently running turn.
pub(crate) struct ActiveTurn {
    pub(crate) task: Option<RunningTask>,
    pub(crate) turn_state: Arc<Mutex<TurnState>>,
}

/// Whether mailbox deliveries should still be folded into the current turn.
///
/// State machine:
/// - A turn starts in `CurrentTurn`, so queued child mail can join the next
///   model request for that turn.
/// - After user-visible terminal output is recorded, we switch to `NextTurn`
///   to leave late child mail queued instead of extending an already shown
///   answer.
/// - If the same task later gets explicit same-turn work again (a steered user
///   prompt or a tool call after an untagged preamble), we reopen `CurrentTurn`
///   so that pending child mail is drained into that follow-up request.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum MailboxDeliveryPhase {
    /// Incoming mailbox messages can still be consumed by the current turn.
    #[default]
    CurrentTurn,
    /// The current turn already emitted visible final answer text; mailbox
    /// messages should remain queued for a later turn.
    NextTurn,
}

impl Default for ActiveTurn {
    fn default() -> Self {
        Self {
            task: None,
            turn_state: Arc::new(Mutex::new(TurnState::default())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TaskKind {
    Regular,
    Review,
    Compact,
}

pub(crate) struct RunningTask {
    pub(crate) done: Arc<Notify>,
    pub(crate) kind: TaskKind,
    pub(crate) task: Arc<dyn AnySessionTask>,
    pub(crate) cancellation_token: CancellationToken,
    pub(crate) handle: AbortOnDropHandle<()>,
    pub(crate) turn_context: Arc<TurnContext>,
    pub(crate) turn_extension_data: Arc<ExtensionData>,
    pub(crate) _agent_execution_guard: Option<AgentExecutionGuard>,
    /// Covers the complete admitted task, including mailbox input moved out of InputQueue before
    /// the model loop starts.
    pub(crate) _native_auto_turn_guard: NativeAutoTurnGuard,
    // Timer recorded when the task drops to capture the full turn duration.
    pub(crate) _timer: Option<codex_otel::Timer>,
}

/// Mutable state for a single turn.
#[derive(Default)]
pub(crate) struct TurnState {
    pending_approvals: HashMap<String, PendingApproval>,
    pending_request_permissions: HashMap<String, PendingRequestPermissions>,
    pending_user_input: HashMap<String, oneshot::Sender<RequestUserInputResponse>>,
    pending_elicitations: HashMap<(String, RequestId), oneshot::Sender<ElicitationResponse>>,
    mcp_tool_approval_metadata: HashMap<String, McpToolApprovalMetadata>,
    pending_dynamic_tools: HashMap<String, oneshot::Sender<DynamicToolResponse>>,
    pub(crate) pending_input: TurnInputQueue,
    mailbox_delivery_phase: MailboxDeliveryPhase,
    /// D51 (#1001): bound to the policy marker they were granted under.
    granted_permissions: super::GrantedPermissions,
    /// D51 (#1001): the policy marker strict auto-review was turned on
    /// under, with the grant it came with; it ends with that grant. Ending it
    /// also ends the automatic review of commands that would run unasked:
    /// deliberately, as the new level's approval rules then govern.
    strict_auto_review_under: Option<super::AuthorityMarker>,
    pub(crate) tool_calls: u64,
    pub(crate) has_memory_citation: bool,
    pub(crate) token_usage_at_turn_start: TokenUsage,
    mailbox_message_ids: HashSet<String>,
}

/// An open approval prompt. #945: it remembers the policy marker (epoch
/// and revocation generation) it was shown under, so an answer that comes
/// after a security transition can be told apart.
struct PendingApproval {
    tx: oneshot::Sender<ReviewDecision>,
    shown_under: Option<(u64, u64)>,
}

pub(crate) struct PendingRequestPermissions {
    pub(crate) tx_response: oneshot::Sender<RequestPermissionsResponse>,
    pub(crate) requested_permissions: RequestPermissionProfile,
    pub(crate) environment: TurnEnvironmentSelection,
    /// #945: the policy marker the request was shown under.
    pub(crate) shown_under: Option<(u64, u64)>,
}

impl TurnState {
    pub(crate) fn track_mailbox_message(&mut self, message_id: String) {
        self.mailbox_message_ids.insert(message_id);
    }

    pub(crate) fn take_mailbox_message_ids(&mut self) -> Vec<String> {
        self.mailbox_message_ids.drain().collect()
    }

    pub(crate) fn insert_pending_approval(
        &mut self,
        key: String,
        tx: oneshot::Sender<ReviewDecision>,
        shown_under: Option<(u64, u64)>,
    ) -> Option<oneshot::Sender<ReviewDecision>> {
        self.pending_approvals
            .insert(key, PendingApproval { tx, shown_under })
            .map(|previous| previous.tx)
    }

    pub(crate) fn remove_pending_approval(
        &mut self,
        key: &str,
    ) -> Option<oneshot::Sender<ReviewDecision>> {
        self.pending_approvals.remove(key).map(|pending| pending.tx)
    }

    /// #945: the policy marker the open prompt `key` was shown under.
    pub(crate) fn pending_approval_shown_under(&self, key: &str) -> Option<Option<(u64, u64)>> {
        self.pending_approvals
            .get(key)
            .map(|pending| pending.shown_under)
    }

    pub(crate) fn clear_pending_waiters(&mut self) {
        self.pending_approvals.clear();
        self.pending_request_permissions.clear();
        self.pending_user_input.clear();
        self.pending_elicitations.clear();
        self.mcp_tool_approval_metadata.clear();
        self.pending_dynamic_tools.clear();
    }

    pub(crate) fn insert_pending_request_permissions(
        &mut self,
        key: String,
        pending_request_permissions: PendingRequestPermissions,
    ) -> Option<PendingRequestPermissions> {
        self.pending_request_permissions
            .insert(key, pending_request_permissions)
    }

    pub(crate) fn remove_pending_request_permissions(
        &mut self,
        key: &str,
    ) -> Option<PendingRequestPermissions> {
        self.pending_request_permissions.remove(key)
    }

    pub(crate) fn insert_pending_user_input(
        &mut self,
        key: String,
        tx: oneshot::Sender<RequestUserInputResponse>,
    ) -> Option<oneshot::Sender<RequestUserInputResponse>> {
        self.pending_user_input.insert(key, tx)
    }

    pub(crate) fn remove_pending_user_input(
        &mut self,
        key: &str,
    ) -> Option<oneshot::Sender<RequestUserInputResponse>> {
        self.pending_user_input.remove(key)
    }

    pub(crate) fn insert_pending_elicitation(
        &mut self,
        server_name: String,
        request_id: RequestId,
        tx: oneshot::Sender<ElicitationResponse>,
    ) -> Option<oneshot::Sender<ElicitationResponse>> {
        self.pending_elicitations
            .insert((server_name, request_id), tx)
    }

    pub(crate) fn remove_pending_elicitation(
        &mut self,
        server_name: &str,
        request_id: &RequestId,
    ) -> Option<oneshot::Sender<ElicitationResponse>> {
        self.pending_elicitations
            .remove(&(server_name.to_string(), request_id.clone()))
    }

    pub(crate) fn insert_mcp_tool_approval_metadata(
        &mut self,
        call_id: String,
        metadata: McpToolApprovalMetadata,
    ) {
        self.mcp_tool_approval_metadata.insert(call_id, metadata);
    }

    pub(crate) fn mcp_tool_approval_metadata(
        &self,
        call_id: &str,
    ) -> Option<McpToolApprovalMetadata> {
        self.mcp_tool_approval_metadata.get(call_id).cloned()
    }

    pub(crate) fn insert_pending_dynamic_tool(
        &mut self,
        key: String,
        tx: oneshot::Sender<DynamicToolResponse>,
    ) -> Option<oneshot::Sender<DynamicToolResponse>> {
        self.pending_dynamic_tools.insert(key, tx)
    }

    pub(crate) fn remove_pending_dynamic_tool(
        &mut self,
        key: &str,
    ) -> Option<oneshot::Sender<DynamicToolResponse>> {
        self.pending_dynamic_tools.remove(key)
    }

    pub(crate) fn accept_mailbox_delivery_for_current_turn(&mut self) {
        self.set_mailbox_delivery_phase(MailboxDeliveryPhase::CurrentTurn);
    }

    pub(crate) fn accepts_mailbox_delivery_for_current_turn(&self) -> bool {
        self.mailbox_delivery_phase == MailboxDeliveryPhase::CurrentTurn
    }

    pub(crate) fn set_mailbox_delivery_phase(&mut self, phase: MailboxDeliveryPhase) {
        self.mailbox_delivery_phase = phase;
    }

    pub(crate) fn record_granted_permissions(
        &mut self,
        environment_id: &str,
        permissions: AdditionalPermissionProfile,
        granted_under: super::AuthorityMarker,
        current: super::AuthorityMarker,
    ) -> bool {
        self.granted_permissions
            .record(environment_id, permissions, granted_under, current)
    }

    pub(crate) fn granted_permissions(
        &mut self,
        environment_id: &str,
        current: super::AuthorityMarker,
    ) -> Option<AdditionalPermissionProfile> {
        self.granted_permissions.get(environment_id, current)
    }

    pub(crate) fn enable_strict_auto_review(&mut self, under: super::AuthorityMarker) {
        self.strict_auto_review_under = Some(under);
    }

    /// Whether strict auto-review is on under the policy in force
    /// (`current`): not after a security transition (D51).
    pub(crate) fn strict_auto_review_enabled(&self, current: super::AuthorityMarker) -> bool {
        self.strict_auto_review_under == Some(current)
    }
}
