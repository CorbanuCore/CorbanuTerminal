//! Application-level events used to coordinate UI actions.
//!
//! `AppEvent` is the internal message bus between UI components and the top-level `App` loop.
//! Widgets emit events to request actions that must be handled at the app layer (like opening
//! pickers, persisting configuration, or shutting down the agent), without needing direct access to
//! `App` internals.
//!
//! Exit is modelled explicitly via `AppEvent::Exit(ExitMode)` so callers can request shutdown-first
//! quits without reaching into the app loop or coupling to shutdown/exit sequencing.

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio_util::sync::CancellationToken;

use crate::inline_visualization::InlineVisualizationContext;
use codex_app_server_protocol::AddCreditsNudgeCreditType;
use codex_app_server_protocol::AddCreditsNudgeEmailStatus;
use codex_app_server_protocol::ConsumeAccountRateLimitResetCreditResponse;
use codex_app_server_protocol::DynamicToolCallResponse;
use codex_app_server_protocol::GetAccountRateLimitsResponse;
use codex_app_server_protocol::MarketplaceAddResponse;
use codex_app_server_protocol::MarketplaceRemoveResponse;
use codex_app_server_protocol::MarketplaceUpgradeResponse;
use codex_app_server_protocol::McpServerOauthLoginResponse;
use codex_app_server_protocol::McpServerStatus;
use codex_app_server_protocol::McpServerStatusDetail;
use codex_app_server_protocol::PluginInstallResponse;
use codex_app_server_protocol::PluginListResponse;
use codex_app_server_protocol::PluginMarketplaceEntry;
use codex_app_server_protocol::PluginReadParams;
use codex_app_server_protocol::PluginReadResponse;
use codex_app_server_protocol::PluginUninstallResponse;
use codex_app_server_protocol::RequestId as AppServerRequestId;
use codex_app_server_protocol::SkillsListResponse;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadAgentMessageParams;
use codex_app_server_protocol::ThreadGoalStatus;
use codex_app_server_protocol::ThreadItemsListResponse;
use codex_connectors::AppInfo;
use codex_file_search::FileMatch;
use codex_message_history::HistoryBatchCursor;
use codex_protocol::ThreadId;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ReasoningEffort;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_approval_presets::ApprovalPreset;
use codex_utils_path_uri::PathUri;
use strum_macros::IntoStaticStr;
use uuid::Uuid;

use crate::app_command::AppCommand;
use crate::app_server_session::AppServerStartedThread;
use crate::bottom_pane::ApprovalRequest;
use crate::bottom_pane::StatusLineItem;
use crate::bottom_pane::TerminalTitleItem;
use crate::chatwidget::AstraModelPickerAction;
use crate::chatwidget::ConfiguredPetLoadGeneration;
use crate::chatwidget::ConnectorScopeGeneration;
use crate::chatwidget::ThreadUsageOutcome;
use crate::chatwidget::UserMessage;
use crate::experimental_features::FeatureWriteResult;
use crate::goal_files::GoalDraft;
use codex_app_server_protocol::AskForApproval;
use codex_config::types::ApprovalsReviewer;
use codex_features::Feature;
use codex_plugin::PluginCapabilitySummary;
use codex_protocol::config_types::CollaborationModeMask;
use codex_protocol::models::ActivePermissionProfile;
<<<<<<< HEAD
use codex_protocol::openai_models::ReasoningEffort;
use zeroize::Zeroize;
||||||| 413492cd6c
use codex_protocol::openai_models::ReasoningEffort;
=======
use codex_realtime_webrtc::StartedRealtimeWebrtcSession;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

use crate::history_cell::HistoryCell;
<<<<<<< HEAD
use crate::mkdocs_viewer::MkDocsSite;
||||||| 413492cd6c

=======

#[derive(Debug)]
pub(crate) struct RealtimeWebrtcStartupFailure {
    pub message: String,
    pub cause: codex_realtime_webrtc::ConnectionError,
}

/// Global voice controls always apply to the one call's owner.
#[derive(Clone, Copy, Debug)]
pub(crate) enum VoiceControl {
    Toggle,
    Stop,
    Mute,
}

/// Confirmed server lifecycle operations available from the agents dashboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AgentsOverviewAction {
    Archive,
    Delete,
}

/// Whether a managed checkout starts fresh or preserves the current conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagedWorktreeMode {
    New,
    Fork,
}

/// Checkout creation result returned from the blocking Git task to the TUI event loop.
/// Prepared checkout and destination configuration consumed on a fresh event-loop stack.
#[derive(Debug)]
pub(crate) struct ManagedWorktreeTransition {
    pub(crate) source_thread_id: ThreadId,
    pub(crate) source_cwd: AbsolutePathBuf,
    pub(crate) manager: codex_worktree::WorktreeManager,
    pub(crate) checkout: codex_worktree::ManagedWorktree,
    pub(crate) config: Box<crate::legacy_core::config::Config>,
    pub(crate) mode: ManagedWorktreeMode,
    pub(crate) name: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ManagedWorktreeCreated {
    pub(crate) source_thread_id: ThreadId,
    pub(crate) source_cwd: AbsolutePathBuf,
    pub(crate) mode: ManagedWorktreeMode,
    pub(crate) name: Option<String>,
    pub(crate) result: Result<
        (
            codex_worktree::WorktreeManager,
            codex_worktree::ManagedWorktree,
        ),
        String,
    >,
}

>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThreadGoalSetMode {
    ConfirmIfExists,
    ReplaceExisting,
    UpdateExisting {
        status: ThreadGoalStatus,
        token_budget: Option<i64>,
    },
}

/// One absolute history offset returned by a batch lookup.
///
/// Malformed rows retain their offset with `entry` set to `None` so the composer can cache the gap
/// without shifting every older record.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HistoryBatchEntryResponse {
    pub(crate) offset: usize,
    pub(crate) entry: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GpuAuthorizationPromptState {
    pub(crate) maximum_hourly_microusd: Option<i64>,
    pub(crate) maximum_total_microusd: Option<i64>,
    pub(crate) validation_error: Option<String>,
}

/// Persistent-history data routed back to the thread that requested it.
///
/// Batch responses preserve absolute offsets and malformed-row gaps so the composer can cache the
/// data independently of whichever search query is active when the response arrives.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum HistoryLookupResponse {
    Entry {
        offset: usize,
        log_id: u64,
        entry: Option<String>,
    },
    Batch {
        cursor: HistoryBatchCursor,
        log_id: u64,
        entries: Vec<HistoryBatchEntryResponse>,
        next_older_cursor: Option<HistoryBatchCursor>,
    },
    BatchError {
        cursor: HistoryBatchCursor,
        log_id: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConsolidationScrollbackReflow {
    IfResizeReflowRan,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) enum WindowsSandboxEnableMode {
    Elevated,
    Legacy,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) struct ConnectorsSnapshot {
    pub(crate) connectors: Vec<AppInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PluginLocation {
    Local { marketplace_path: AbsolutePathBuf },
    Remote { marketplace_name: String },
}

impl PluginLocation {
    pub(crate) fn into_request_params(self) -> (Option<AbsolutePathBuf>, Option<String>) {
        match self {
            PluginLocation::Local { marketplace_path } => (Some(marketplace_path), None),
            PluginLocation::Remote { marketplace_name } => (None, Some(marketplace_name)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PluginRemoteSectionError {
    pub(crate) section_id: String,
    pub(crate) label: String,
    pub(crate) message: String,
}

/// Distinguishes why a rate-limit refresh was requested so the completion
/// handler can route the result correctly.
///
/// A `StartupPrefetch` fires once, concurrently with the rest of TUI init, and
/// updates the cached snapshots and any available reset-credit notice (no
/// status card to finalize). A `StatusCommand` is tied to a specific `/status`
/// invocation and must call `finish_status_rate_limit_refresh` when done so the
/// card stops showing a "refreshing" state. A `UsageMenu` refreshes a cached
/// zero reset count so the disabled menu entry can become available without a
/// restart. A `ResetPicker` refreshes the rate limits and detailed reset-credit
/// rows before showing redemption choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RateLimitRefreshOrigin {
    /// Eagerly fetched after bootstrap for `/status` data and reset availability.
    StartupPrefetch { reset_hint_request_id: u64 },
    /// User-initiated via `/status`; the `request_id` correlates with the
    /// status card that should be updated when the fetch completes.
    StatusCommand { request_id: u64 },
    /// User reopened `/usage` while the cached reset-credit count was zero.
    UsageMenu { request_id: u64 },
    /// User opened the reset-credit picker.
    ResetPicker { request_id: u64 },
    /// Refresh requested after a reset credit was successfully consumed.
    ResetConsume { request_id: u64 },
    /// Refresh backend recovery after an inference limit error.
    Recovery,
    /// Background account usage read, scheduled more frequently near exhaustion.
    Periodic,
}

pub(crate) struct ProviderApiKeySecret(String);

impl ProviderApiKeySecret {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for ProviderApiKeySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted provider API key>")
    }
}

pub(crate) struct ClaudeSubscriptionTokenSecret(zeroize::Zeroizing<String>);

impl ClaudeSubscriptionTokenSecret {
    pub(crate) fn new(value: String) -> Self {
        Self(zeroize::Zeroizing::new(value))
    }

    pub(crate) fn into_inner(self) -> String {
        self.0.to_string()
    }
}

impl fmt::Debug for ClaudeSubscriptionTokenSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted Claude subscription token>")
    }
}

pub(crate) struct TelegramBotSecret(String);

impl TelegramBotSecret {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub(crate) fn into_inner(mut self) -> String {
        std::mem::take(&mut self.0)
    }
}

impl Drop for TelegramBotSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for TelegramBotSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted Telegram bot token>")
    }
}

pub(crate) struct WalletSecret(String);

impl WalletSecret {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(mut self) -> String {
        std::mem::take(&mut self.0)
    }
}

impl Drop for WalletSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for WalletSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted wallet secret>")
    }
}

pub(crate) struct VaultSecret(String);

impl VaultSecret {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub(crate) fn into_inner(mut self) -> String {
        std::mem::take(&mut self.0)
    }
}

impl Drop for VaultSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for VaultSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted vault secret>")
    }
}

#[derive(Debug)]
pub(crate) struct WalletCreatedResult {
    pub(crate) address: String,
    pub(crate) recovery: WalletSecret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WalletPersistenceOperation {
    Create,
    Restore,
}

#[derive(Debug)]
pub(crate) struct WalletUnlockedResult {
    pub(crate) capability: WalletSecret,
    pub(crate) expires_in_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WalletUnlockContinuation {
    WalletMenu,
    OpenCorbanuApi {
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    // Only the unlock tests construct this; see WalletPlanPurchaseMode.
    #[cfg_attr(not(test), allow(dead_code))]
    OpenPlans {
        mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode,
    },
    CorbanuApiOperation {
        operation: codex_wallet::CorbanuApiOperation,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeferredWalletPreflight {
    Missing,
    Locked,
    Unlocked,
}

#[derive(Debug)]
pub(crate) struct WalletPlanProvisionedResult {
    pub(crate) plan_id: String,
    pub(crate) api_key: WalletSecret,
    pub(crate) purchase: Option<WalletPlanPurchaseSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WalletPlanProvisioningOperation {
    Purchase,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WalletPlanReceiptSelectionPolicy {
    SelectProviderOnSuccess,
    PreserveCurrentProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WalletPlanPersistenceAttemptId(u64);

impl WalletPlanPersistenceAttemptId {
    pub(crate) fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WalletPlanCredentialPersistenceError {
    StoreFailed,
    WorkerUnavailable,
}

impl fmt::Display for WalletPlanCredentialPersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StoreFailed => f.write_str("the credential could not be stored"),
            Self::WorkerUnavailable => {
                f.write_str("the credential storage worker stopped unexpectedly")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct WalletPlanPurchaseSummary {
    pub(crate) price_usdc: String,
    pub(crate) scheduled_start: Option<String>,
    pub(crate) transaction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeymapEditIntent {
    ReplaceAll,
    AddAlternate,
    ReplaceOne { old_key: String },
}

/// Kind of shortcut recorded by one `/keymap` capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeymapCaptureMode {
    SingleKey,
    Chord,
    LeaderChord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TranscriptExportDestination {
    Clipboard,
    File(PathBuf),
}

/// Deliver a generated title to its originating automatic rename or editable prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ThreadTitleDestination {
    /// Name the thread only if the user has not already named it.
    Automatic,
    /// Prefill only the still-active rename prompt with the matching request ID.
    RenameSuggestion { request_id: Uuid },
}

/// Identifies the policy that initiated a recap request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecapTrigger {
    Automatic,
    Manual,
}

#[derive(Debug)]
pub(crate) struct AgentsOverviewThreadRefresh {
    pub(crate) cleared_unread: Vec<ThreadId>,
    pub(crate) threads: std::collections::HashMap<ThreadId, Option<(Thread, bool)>>,
    pub(crate) last_messages: std::collections::HashMap<ThreadId, String>,
    pub(crate) recent_seed_complete: bool,
    pub(crate) discovery: Option<crate::app::agents_overview_discovery::AgentsOverviewDiscovery>,
    pub(crate) pinned_thread_ids: Option<Option<Vec<ThreadId>>>,
}

#[derive(Debug, Default)]
pub(crate) struct AgentPickerThreadRefresh {
    pub(crate) threads: Vec<Thread>,
    pub(crate) archived_thread_ids: std::collections::HashSet<ThreadId>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, IntoStaticStr)]
pub(crate) enum AppEvent {
    AccountEmailLoaded {
        request_id: uuid::Uuid,
        email: Option<String>,
        workspace: Option<String>,
    },
    SecuritySetupLoaded {
        request_id: uuid::Uuid,
        identity: crate::security_setup::Identity,
        notice: crate::security_setup::Notice,
    },
    OpenDaemonMenu,
    ConfirmDaemonRestart,
    RunDaemonRestart,
    ConfirmDaemonUpdate(crate::update_action::DaemonUpdateSource),
    RunDaemonUpdate(crate::update_action::DaemonUpdateSource),
    ReviewMisalignment(Arc<crate::chatwidget::MisalignmentReview>),
    ContinueMisalignment(Arc<crate::chatwidget::MisalignmentReview>),
    CloseMisalignmentReview,
    /// Open the live command center for recent and locally retained root sessions.
    OpenAgentsOverview,
    ShowMoreAgentsOverview,
    RefreshViewedThread {
        backend_id: crate::backend_id::BackendId,
        thread_id: ThreadId,
        revision: String,
    },
    /// Create an empty thread from the command center.
    NewAgentsOverviewSession {
        cwd: Option<AbsolutePathBuf>,
    },
    /// Update the agents overview after a background thread listing finishes.
    AgentsOverviewThreadsLoaded {
        request_id: Uuid,
        result: Result<AgentsOverviewThreadRefresh, String>,
    },
    /// Switch to a root session selected from the shared dashboard.
    SelectAgentsOverviewThread {
        thread_id: ThreadId,
    },
    /// Create an empty session in a worktree from the selected project's default branch.
    NewAgentsOverviewWorktree {
        cwd: Option<AbsolutePathBuf>,
    },
    AgentsOverviewWorktreeCreated(Result<crate::app::PendingWorktree, String>),
    /// Fork the selected dashboard conversation and open the new session.
    ForkAgentsOverviewThread {
        thread_id: ThreadId,
    },
    /// Run the existing fork action after selection events have been processed.
    ForkAgentsOverviewThreadReady {
        thread_id: ThreadId,
    },
    /// Rename a task directly from the shared dashboard.
    RenameAgentsOverviewThread {
        thread_id: ThreadId,
        name: String,
    },
    /// Move a task into or out of the shared pinned section.
    ToggleAgentsOverviewPin {
        thread_id: ThreadId,
        pinned: bool,
    },
    /// Finish moving a task into or out of the shared pinned section.
    AgentsOverviewPinToggled {
        request_id: Uuid,
        thread_id: ThreadId,
        pinned: bool,
        result: Result<(), String>,
    },
    /// Generate an editable title suggestion for the active rename prompt.
    SuggestThreadName {
        thread_id: ThreadId,
        request_id: Uuid,
    },
    /// Register a hidden title-generation thread started in the background.
    ThreadTitleStarted {
        cancellation: CancellationToken,
        thread_id: ThreadId,
        destination: ThreadTitleDestination,
        prompt: String,
        effort: Option<ReasoningEffort>,
        result: Result<String, String>,
    },
    /// Route a hidden title request to its automatic rename or editable prompt.
    GeneratedThreadTitle {
        cancellation: CancellationToken,
        thread_id: ThreadId,
        temporary_thread_id: ThreadId,
        destination: ThreadTitleDestination,
        result: Result<String, String>,
    },
    /// Interrupt a task directly from the shared dashboard.
    StopAgentsOverviewThread {
        thread_id: ThreadId,
    },
    /// Hide a dashboard row locally without stopping its task.
    HideAgentsOverviewThread {
        thread_id: ThreadId,
    },
    /// Confirm a server lifecycle action for the selected dashboard task.
    ConfirmAgentsOverviewAction {
        thread_id: ThreadId,
        action: AgentsOverviewAction,
    },
    /// Execute a confirmed dashboard lifecycle action.
    RunAgentsOverviewAction {
        thread_id: ThreadId,
        action: AgentsOverviewAction,
    },
    /// Open the agent picker for switching active threads.
    OpenAgentPicker,
    /// Merge a completed root-scoped agent-picker refresh without blocking terminal input.
    AgentPickerThreadsLoaded {
        primary_thread_id: ThreadId,
        request_id: Uuid,
        result: Result<AgentPickerThreadRefresh, String>,
    },
    /// Switch the active thread to the selected agent.
    SelectAgentThread(ThreadId),
    /// Open the user-pane picker for switching Codex/Claude panes.
    OpenPanePicker,
    /// `/security` "restart now" (PF-24-S02): shut down like `/quit`, then
    /// start Corbanu Terminal again so a saved level takes effect.
    RestartForSecurityLevel,
    /// #507: the kill switch was turned on in `/security`; stop the Claude
    /// panes' running turns (Core stops its own sessions).
    KillSwitchTurnedOn,
    /// #507: the kill switch was turned off. Nothing restarts; the person is
    /// told which Claude panes have a task waiting for their next turn.
    KillSwitchTurnedOff,
    /// #507 (Travis, 2026-10-12): another Corbanu Terminal process on this
    /// home turned the kill switch on or off; Core already applied it to
    /// this process's sessions.
    KillSwitchChangedElsewhere {
        on: bool,
    },
    /// #842 (Travis, 2026-10-15): Moderate confirmed in another Corbanu
    /// Terminal process on this home; Core already raised this process's
    /// sessions to it.
    SecurityLevelRaisedElsewhere {
        level: crate::legacy_core::security_revocation::SecurityLevel,
    },
    /// Open the `/spawn` role picker.
    OpenSpawnRolePicker,
    /// Open the `/spawn` Nazgul pane-binding picker.
    OpenSpawnNazgulPanePicker,
    /// Open the `/spawn` Nazgul role picker (create a new Nazgul pane or bind an existing one).
    OpenSpawnNazgulPicker,
    /// Bind an existing user pane as the Nazgul/root orchestration pane.
    BindSpawnNazgulPane {
        pane_id: String,
    },
    /// Choose the parent/supervisor for a spawn role.
    OpenSpawnParentPicker {
        role: crate::spawn_orchestration::SpawnRole,
    },
    /// Open the P0 harness picker for a spawn role.
    OpenSpawnHarnessPicker {
        role: crate::spawn_orchestration::SpawnRole,
        parent_node_id: Option<String>,
    },
    /// Open the inherited model/effort confirmation step for a spawn role.
    OpenSpawnModelPicker {
        role: crate::spawn_orchestration::SpawnRole,
        parent_node_id: Option<String>,
    },
    /// Open the Claude Code provider picker for a spawn role.
    OpenSpawnClaudeProfilePicker {
        role: crate::spawn_orchestration::SpawnRole,
        parent_node_id: Option<String>,
    },
    /// Create a native spawned-agent pane for a role/model selection.
    CreateSpawnAgent {
        role: crate::spawn_orchestration::SpawnRole,
        parent_node_id: Option<String>,
        agent_nickname: Option<String>,
        model: String,
        provider: Option<String>,
        effort: Option<ReasoningEffort>,
    },
    /// Create the standard persistent crew: Nazgul (root) -> Troll -> 3 Orcs. No task is started.
    CreateSpawnStandardCrew,
    /// Create the Corbanu API persistent crew: Kimi K3 Nazgul -> Luna Troll -> 3 Flash Orcs.
    CreateSpawnCorbanuApiCrew,
    /// Open a prompt that sends work to an existing spawned-agent pane.
    OpenSpawnAgentTaskPrompt {
        thread_id: codex_protocol::ThreadId,
    },
    /// Open a prompt that sends work to an existing Claude spawn pane.
    OpenSpawnClaudePaneTaskPrompt {
        pane_id: String,
    },
    /// An agent's task for a native pane, agent-opened or opened by the person
    /// (D106, #1176): admitted to the pane's Core mailbox as an inter-agent
    /// message, never as the person's input.
    SubmitSpawnAgentTask {
        thread_id: codex_protocol::ThreadId,
        task: String,
        /// #660: another agent sent it while the security kill switch was
        /// on: it waits in the mailbox for the pane's next turn even when it
        /// is handled after the switch was turned off.
        sent_while_kill_switch_on: bool,
    },
    /// D95 (#1146): a task the person typed into Spawn status's "Send task
    /// to" prompt. It starts a turn in that pane as the person's own input,
    /// as if typed there, not as a message from another agent. Only that
    /// prompt sends it; tasks from agents use [`AppEvent::SubmitSpawnAgentTask`].
    SubmitSpawnAgentTaskFromPerson {
        thread_id: codex_protocol::ThreadId,
        task: String,
    },
    /// Deliver an edge-adapter message through the canonical native mailbox.
    SendSpawnAgentMailboxMessage {
        params: ThreadAgentMessageParams,
    },
    /// Start a normal turn in an existing Claude spawn pane.
    SubmitSpawnClaudePaneTask {
        pane_id: String,
        task: String,
        /// #613: another agent sent it while the security kill switch was
        /// on: it waits for the pane's next turn even when it is handled
        /// after the switch was turned off.
        sent_while_kill_switch_on: bool,
    },
    /// Show the current orchestration tree.
    OpenSpawnStatus,
    /// Confirm permanent removal of the complete managed `/spawn` crew.
    OpenRemoveSpawnCrewConfirmation,
    /// Permanently remove every owned member of the managed `/spawn` crew.
    RemoveSpawnCrew,
    /// Handle a native `/orchestrate` whip command.
    HandleOrchestrateCommand {
        args: String,
    },
    /// Start the guided `/orchestrate attach` flow.
    OpenOrchestrateTargetPicker,
    /// Start the two-choice `/orchestrate` fast path.
    OpenOrchestrateFastTargetPicker,
    /// Choose a Manager for the two-choice `/orchestrate` fast path.
    OpenOrchestrateFastManagerPicker {
        target: String,
    },
    /// Bind an existing Manager using the fast-path defaults.
    AttachOrchestrateFastManager {
        target: String,
        manager_node_id: String,
    },
    /// Choose how long a guided whip should stay armed.
    OpenOrchestrateDurationPicker {
        target: String,
    },
    /// Choose the instruction document for a guided whip.
    OpenOrchestrateWhipPicker {
        target: String,
        duration_arg: String,
        duration_label: String,
    },
    /// Compose a new global whip instruction document from the guided flow.
    OpenOrchestrateWriteWhipPrompt {
        target: String,
        duration_arg: String,
        duration_label: String,
    },
    /// Prompt for the slug to save a composed global whip instruction document.
    OpenOrchestrateSaveWhipPrompt {
        target: String,
        duration_arg: String,
        duration_label: String,
        instructions: String,
    },
    /// Save a composed global whip and continue to guided confirm.
    SaveOrchestrateWhipAndConfirm {
        target: String,
        duration_arg: String,
        duration_label: String,
        requested_name: String,
        instructions: String,
    },
    /// Choose the conversational Manager for a guided assignment.
    OpenOrchestrateManagerPicker {
        target: String,
        duration_arg: String,
        duration_label: String,
        whip_name: String,
    },
    /// Create a native Codex Manager pane and attach the guided assignment to it.
    CreateOrchestrateManager {
        target: String,
        duration_arg: String,
        whip_name: String,
    },
    /// Confirm the guided whip attach command.
    OpenOrchestrateConfirm {
        target: String,
        duration_arg: String,
        duration_label: String,
        whip_name: String,
        manager_node_id: String,
    },
    /// Show actions/details for one whip row.
    OpenOrchestrateWhipDetails {
        whip_id: String,
    },
    /// Open a duration picker for extending one whip.
    OpenOrchestrateExtendDurationPicker {
        whip_id: String,
    },
    /// Periodic native whip sweep; intentionally low priority and no-op when no whips are armed.
    WhipSweepTick,
    /// Switch the active user pane.
    SelectUserPane {
        pane_id: String,
    },
    /// Open the model picker for creating a native Codex pane.
    OpenCodexPaneModelPicker,
    /// Create and switch to a native Codex pane.
    CreateCodexPane {
        model: String,
        provider: Option<String>,
        effort: Option<ReasoningEffort>,
        display_name: Option<String>,
    },
    /// Prompt for the display name of a native Codex pane before creation.
    OpenCodexPaneNamePrompt {
        model: String,
        provider: Option<String>,
        effort: Option<ReasoningEffort>,
    },
    /// Open the Claude Code provider picker.
    OpenClaudePaneProfilePicker,
    /// Create and switch to a Claude Code headless pane.
    CreateClaudePane {
        profile: crate::claude_panes::ClaudeProviderProfileKind,
        display_name: Option<String>,
    },
    /// Prompt for the display name of a Claude Code pane before creation.
    OpenClaudePaneNamePrompt {
        profile: crate::claude_panes::ClaudeProviderProfileKind,
    },
    /// Rename the currently visible pane display name.
    RenameCurrentPane {
        name: String,
    },
    /// Open a rename prompt for a native Codex pane row.
    OpenRenameCodexPanePrompt {
        thread_id: codex_protocol::ThreadId,
    },
    /// Open a rename prompt for a Claude Code pane row.
    OpenRenameClaudePanePrompt {
        pane_id: String,
    },
    /// Rename a specific native Codex pane row.
    RenameCodexPane {
        thread_id: codex_protocol::ThreadId,
        name: String,
    },
    /// Rename a specific Claude Code pane row.
    RenameClaudePane {
        pane_id: String,
        name: String,
    },
    /// Create and switch to a Claude Code headless pane for a spawn role.
    CreateSpawnClaudePane {
        role: crate::spawn_orchestration::SpawnRole,
        parent_node_id: Option<String>,
        profile: crate::claude_panes::ClaudeProviderProfileKind,
    },
    /// A Claude Code headless pane turn finished.
    ClaudePaneTurnFinished {
        pane_id: String,
        result: Result<crate::claude_panes::ClaudePaneTurnOutput, String>,
    },
    /// A contained Claude pane (#218) asks to use a tool; a person answers.
    ClaudePaneApprovalRequested(Box<crate::claude_panes::approval::ClaudeApprovalRequest>),
    /// Some contained Claude pane requests stopped waiting (the turn ended,
    /// was interrupted, timed out, or Claude Code cancelled them); their
    /// popups close.
    ClaudePaneApprovalsSettled,
    /// #554: a new contained Claude pane cannot run the Claude Code on PATH
    /// (too old, or it does not run); `problem` says why and what to do.
    ClaudePaneLaunchProblem {
        pane_id: String,
        problem: String,
    },
    /// A Claude Code headless pane emitted bounded live progress.
    ClaudePaneTurnProgress {
        progress: crate::claude_panes::ClaudePaneTurnProgress,
    },
    /// A pane bridge sent a model request upstream and is reporting it.
    ///
    /// The bridge posts to a provider from inside this process, which does not
    /// run the model client, so the request reaches no ledger unless it is
    /// reported. The server decides which account the route bills and whether
    /// to record it at all.
    PaneBridgeModelRequestSent {
        provider_id: String,
        base_url: String,
        path: String,
        model: String,
        /// The `usage` object the provider's response carried, if any.
        usage: Option<serde_json::Value>,
    },

    AgentControlTick {
        directory: PathBuf,
    },

    /// A response may only update the profile, credential and view which requested it.
    TaskNodeScopedResult {
        label: &'static str,
        generation: u64,
        scope: codex_tasknode_session::SessionScope,
        identity: Option<String>,
        view: Option<String>,
        event: Box<AppEvent>,
    },

    CampaignTrackerDocument {
        text: String,
    },
    CampaignTrackerTick,
    CampaignTrackerOpen {
        path: String,
        body: Option<serde_json::Value>,
    },
    CampaignTrackerResult {
        path: String,
        enrollment: Option<bool>,
        body: Option<serde_json::Value>,
        result: Result<serde_json::Value, String>,
    },
    CampaignTrackerSync {
        identity: String,
        result: Result<usize, String>,
    },
    /// Open the Task Node menu.
    OpenTaskNodeMenu,
    /// Task Node menu account/task counts loaded.
    TaskNodeMenuStatusResult {
        result: Result<serde_json::Value, String>,
    },
    /// Task Node menu task request count loaded.
    TaskNodeMenuRequestsResult {
        result: Result<serde_json::Value, String>,
    },
    /// Refresh Task Node menu counts while the menu is still active.
    TaskNodeMenuPoll {
        generation: u64,
    },
    /// Start or refresh GitHub-backed Task Node terminal linking.
    OpenTaskNodeLink,
    /// Task Node GitHub link request finished loading.
    TaskNodeLinkResult {
        result: Result<serde_json::Value, String>,
    },
    /// Show Task Node account/task status.
    OpenTaskNodeStatus,
    /// Task Node account/task status loaded.
    TaskNodeStatusResult {
        result: Result<serde_json::Value, String>,
    },
    /// Show a Task Node task tab.
    OpenTaskNodeTaskList {
        tab: String,
    },
    /// Task Node task tab loaded.
    TaskNodeTaskListResult {
        tab: String,
        result: Result<serde_json::Value, String>,
    },
    /// Open Task Node task detail/actions.
    OpenTaskNodeTaskActions {
        task_id: String,
    },
    /// Task Node task detail/actions loaded.
    TaskNodeTaskActionsResult {
        task_id: String,
        result: Result<serde_json::Value, String>,
    },
    /// Copy a Task Node task brief to the clipboard.
    CopyTaskNodeTaskBrief {
        task_id: String,
    },
    /// Task Node task brief detail loaded for clipboard copy.
    CopyTaskNodeTaskBriefResult {
        task_id: String,
        result: Result<serde_json::Value, String>,
    },
    /// Submit a Task Node lifecycle action.
    SubmitTaskNodeTaskAction {
        task_id: String,
        action: String,
    },
    /// Task Node lifecycle action finished.
    SubmitTaskNodeTaskActionResult {
        action: String,
        result: Result<serde_json::Value, String>,
    },
    /// Open the Task Node evidence prompt.
    OpenTaskNodeEvidencePrompt {
        task_id: String,
    },
    /// Task Node task detail loaded for the evidence prompt.
    OpenTaskNodeEvidencePromptResult {
        task_id: String,
        result: Result<serde_json::Value, String>,
    },
    /// Submit Task Node evidence text.
    SubmitTaskNodeEvidence {
        task_id: String,
        summary: String,
    },
    /// Task Node evidence submission finished.
    SubmitTaskNodeEvidenceResult {
        result: Result<serde_json::Value, String>,
    },
    /// Open the Task Node personal task request prompt.
    OpenTaskNodeTaskRequestPrompt,
    /// Submit a Task Node personal task request.
    SubmitTaskNodeTaskRequest {
        detail: String,
    },
    /// Task Node personal task request finished.
    SubmitTaskNodeTaskRequestResult {
        result: Result<serde_json::Value, String>,
    },
    /// Show the current Task Node context document.
    TaskNodeTeamContextDocument {
        text: String,
    },

    OpenTaskNodeTeamContext,

    OpenTaskNodeTeamContextResult {
        result: Result<serde_json::Value, String>,
    },

    OpenTaskNodeContext,
    /// Task Node context document loaded.
    OpenTaskNodeContextResult {
        result: Result<serde_json::Value, String>,
    },
    /// Open the Task Node context edit prompt.
    OpenTaskNodeContextEdit {
        title: String,
        body: String,
        revision: u64,
        body_format: String,
    },
    /// Save Task Node context edits.
    SubmitTaskNodeContextEdit {
        title: String,
        body: String,
        revision: u64,
    },
    /// Task Node context save finished.
    SubmitTaskNodeContextEditResult {
        result: Result<serde_json::Value, String>,
    },
    RetryTaskNodeRequest {
        request_id: String,
        attempt: u64,
    },
    RetryTaskNodeRequestResult {
        result: Result<serde_json::Value, String>,
    },
    /// Show active Task Node task requests.
    OpenTaskNodeRequestList,
    OpenTaskNodeRequestPage {
        cursor: Option<String>,
    },
    /// Active Task Node task requests loaded.
    OpenTaskNodeRequestListResult {
        result: Result<serde_json::Value, String>,
    },
    /// Show Task Node read-only balance.
    OpenTaskNodeBalance,
    /// Task Node read-only balance loaded.
    OpenTaskNodeBalanceResult {
        result: Result<serde_json::Value, String>,
    },
    /// Show recent Task Node rewards.
    OpenTaskNodeRewards,
    /// Recent Task Node rewards loaded.
    OpenTaskNodeRewardsResult {
        result: Result<serde_json::Value, String>,
    },
    /// Open Task Node chat threads.
    OpenTaskNodeChat,
    /// Task Node chat threads loaded.
    OpenTaskNodeChatConversationsResult {
        result: Result<serde_json::Value, String>,
    },
    /// Open Task Node chat history.
    OpenTaskNodeChatHistory {
        conversation_id: String,
        title: String,
    },
    /// Task Node chat history loaded.
    OpenTaskNodeChatHistoryResult {
        conversation_id: String,
        title: String,
        result: Result<serde_json::Value, String>,
    },
    /// Open the Task Node chat composer.
    OpenTaskNodeChatPrompt {
        conversation_id: String,
        title: String,
    },
    /// Submit a Task Node chat message.
    SubmitTaskNodeChat {
        conversation_id: String,
        title: String,
        message: String,
    },
    /// Task Node chat stream emitted visible assistant text.
    TaskNodeChatStreamDelta {
        stream_id: String,
        conversation_id: String,
        title: String,
        text: String,
    },
    /// Task Node chat stream finished.
    TaskNodeChatStreamDone {
        stream_id: String,
        conversation_id: String,
        title: String,
        result: Result<serde_json::Value, String>,
    },
    /// Remove the local Task Node terminal session.
    LogoutTaskNode,
    /// Local Task Node logout and best-effort remote revoke finished.
    TaskNodeLogoutResult {
        result: Result<String, String>,
    },

    /// Fork the current thread into a transient side conversation.
    StartSide {
        parent_thread_id: ThreadId,
        user_message: Option<UserMessage>,
    },

    /// Submit an op to the specified thread, regardless of current focus.
    SubmitThreadOp {
        thread_id: ThreadId,
        op: AppCommand,
    },

    /// Confirm retrying a safety-buffered turn with the server-selected model.
    ConfirmSafetyBufferedRetry {
        thread_id: ThreadId,
        turn_id: String,
        model: String,
        turn: AppCommand,
        prompt: UserMessage,
    },

    /// Sign the challenge associated with an approved elicitation.
    UserVerificationApproved {
        thread_id: ThreadId,
        server_name: String,
        request_id: AppServerRequestId,
    },
    /// Return a controller-owned verification result, never a native provider handle.
    UserVerificationFinished {
        thread_id: ThreadId,
        server_name: String,
        request_id: AppServerRequestId,
        attempt_id: Uuid,
        result: Result<
            codex_app_server_protocol::UserVerificationProof,
            crate::app_command::UserVerificationFailure,
        >,
    },

    /// Interrupt, fork, and retry a safety-buffered turn with the server-selected model.
    RetrySafetyBufferedTurn {
        thread_id: ThreadId,
        turn_id: String,
        model: String,
        turn: AppCommand,
        prompt: UserMessage,
    },

    /// Fork before a blocked turn and send the selected safer follow-up.
    ContinueSafetyRecourse {
        thread_id: ThreadId,
        turn_id: String,
        prompt: String,
    },

    /// Deliver a synthetic history lookup response to a specific thread channel.
    ThreadHistoryEntryResponse {
        thread_id: ThreadId,
        event: HistoryLookupResponse,
    },

    /// Refill terminal scrollback from older paginated history after its rows reflow.
    RequestOlderScrollbackHistory {
        thread_id: ThreadId,
    },

    /// One background-loaded page of older Ctrl+T transcript history.
    OlderThreadHistoryLoaded {
        thread_id: ThreadId,
        cursor: String,
        result: Result<ThreadItemsListResponse, String>,
    },

    /// Open the filename prompt for an on-demand Markdown transcript export.
    OpenTranscriptExportFilePrompt,
    /// Open retained warnings without changing the draft or transcript position.
    OpenWarnings,
    /// Copy a diagnostic and acknowledge in the footer, without appending history.
    CopyWarning(String),
    /// Apply the user's decisions for the frozen warning details, in viewer-close order.
    UpdateWarnings {
        transcript: Arc<()>,
        dismissed: Vec<crate::history_cell::WarningEntry>,
        kept: Vec<crate::history_cell::WarningEntry>,
    },

    /// Export all current-thread history to the selected destination.
    ExportTranscript {
        destination: TranscriptExportDestination,
    },

    /// Select a response or block directly in the owned transcript.
    SelectTranscriptCopy {
        guard: Arc<crate::copy_input_guard::CopyInputGuard>,
    },

    /// Retry queued input after the transcript copy owner is released.
    TranscriptCopyClosed,

    /// Copy text through the session clipboard worker.
    CopySelection {
        text: Arc<str>,
        label: String,
        format: crate::clipboard_copy::CopyFormat,
    },

    /// Persist a submitted prompt in the cross-session message history.
    AppendMessageHistoryEntry {
        thread_id: ThreadId,
        text: String,
    },

    /// Persist a branch discovered from an App git-action directive into thread metadata.
    SyncThreadGitBranch {
        thread_id: ThreadId,
        branch: String,
        cwd: PathBuf,
    },

    /// Fetch a persistent cross-session message history entry by offset.
    LookupMessageHistoryEntry {
        thread_id: ThreadId,
        offset: usize,
        log_id: u64,
    },

    /// Fetch a bounded batch of persistent history entries for reverse search.
    LookupMessageHistoryBatch {
        thread_id: ThreadId,
        cursor: HistoryBatchCursor,
        log_id: u64,
    },

    /// Start a new session, optionally assigning it a name.
    NewSession {
        name: Option<String>,
    },

    /// Continue a provider-locked conversation in a fresh chat using its saved history.
    ContinueProviderChat {
        thread_id: ThreadId,
    },

    /// Create a managed checkout and start or fork a session into it.
    StartManagedWorktree {
        mode: ManagedWorktreeMode,
        name: Option<String>,
    },
    /// Continue a checkout transition after synchronous Git work finishes off-loop.
    ManagedWorktreeCreated(Box<ManagedWorktreeCreated>),

    BrowseManagedWorktrees,
    ManagedWorktreesLoaded {
        request: crate::worktree_browser::Request,
        result: Result<Vec<crate::worktree_browser::Entry>, String>,
    },
    ManagedWorktreeAction {
        request: crate::worktree_browser::Request,
        action: crate::worktree_browser::Action,
    },
    ShowManagedWorktreeActions {
        request: crate::worktree_browser::Request,
        entry: crate::worktree_browser::Entry,
    },
    ConfirmManagedWorktreeRemoval {
        request: crate::worktree_browser::Request,
        root: PathBuf,
    },
    RemoveManagedWorktree {
        request: crate::worktree_browser::Request,
        root: PathBuf,
    },
    ManagedWorktreeRemoved {
        root: PathBuf,
        result: Result<(), String>,
    },

    /// Change the working directory of the originating idle primary thread.
    ChangeWorkingDirectory {
        thread_id: ThreadId,
        requested_cwd: PathBuf,
    },

    /// Result of the fresh startup thread that is attached after the input UI is live.
    StartupThreadStarted {
        backend_id: crate::backend_id::BackendId,
        result: color_eyre::Result<AppServerStartedThread>,
    },

    /// Register a tool-created or resumed background thread and its overview metadata.
    DynamicToolThreadStarted {
        thread: Thread,
        task_tools_available: bool,
        registered: tokio::sync::oneshot::Sender<()>,
    },

    /// Return a completed client-owned dynamic tool call to app server.
    DynamicToolCallCompleted {
        request_id: AppServerRequestId,
        response: DynamicToolCallResponse,
    },

    /// Register task tools inherited by a dynamically created thread.
    TaskToolsAvailable {
        thread_id: ThreadId,
    },

    /// Clear the terminal UI (screen + scrollback), start a fresh session, and keep the
    /// previous chat resumable.
    ClearUi {
        name: Option<String>,
    },

    /// Clear history queued by the previous thread before the new thread's replay events.
    ResetTranscriptForThreadSwitch,
    /// Reset queued history while keeping the startup draft visible until the next frame.
    ResetTranscriptForThreadSwitchPreservingScreen,

    /// Re-render the transcript using the selected scrollback rendering mode.
    RawOutputModeChanged {
        enabled: bool,
    },

    /// Clear the current context, start a fresh session, and submit an initial user message.
    ///
    /// This is the Plan Mode handoff path: the previous thread remains resumable, but the model
    /// sees only the explicit prompt carried in `text` once the new session is configured.
    ClearUiAndSubmitUserMessage {
        text: String,
    },

    /// Open the resume picker inside the running TUI session.
    OpenResumePicker,

    /// Open the Claude Code migration picker inside the running TUI session.
    OpenExternalAgentConfigMigration,

    /// Resume a thread by UUID or thread name inside the running TUI session.
    ResumeSessionByIdOrName(String),

    /// Attach a conversation through the registered backend that owns it.
    ResumeBackendSession(crate::resume_picker::SessionTarget),

    /// Archive the current active main thread and return to the command center.
    ArchiveCurrentThread,

    /// Permanently delete the current active main thread and return to the command center.
    DeleteCurrentThread,

    /// Fork the current session into a new thread, optionally assigning it a name.
    ForkCurrentSession {
        name: Option<String>,
    },

    /// Revert before a selected prompt, retaining its identity across queued history pages.
    RevertSessionForPromptEdit {
        thread_id: ThreadId,
        selected_cell: Arc<dyn HistoryCell>,
        prompt: UserMessage,
    },
    FinishPromptRevert {
        thread_id: ThreadId,
        nth_user_message: usize,
    },

    /// Request to exit the application.
    ///
    /// Use `ShutdownFirst` for user-initiated quits so core cleanup runs and the
    /// UI exits only after `ShutdownComplete`. `Immediate` is a last-resort
    /// escape hatch that skips shutdown and may drop in-flight work (e.g.,
    /// background tasks, rollout flush, or child process cleanup).
    Exit(ExitMode),
    /// The user accepted stopping other sessions owned by this embedded server.
    ConfirmedEmbeddedExit(ExitMode),

    /// Apply a choice from the running-task exit menu to its originating thread.
    RunningTaskExit {
        action: RunningTaskExitAction,
        thread_id: ThreadId,
    },

    /// Request app-server account logout, then exit after it succeeds.
    Logout,

    /// Request to exit the application due to a fatal error.
    #[allow(dead_code)]
    FatalExitRequest(String),

    /// Forward a command to the Agent. Using an `AppEvent` for this avoids
    /// bubbling channels through layers of widgets.
    CodexOp(AppCommand),

    /// A blocking image-preparation worker has finished; payload stays with its widget.
    ImagesPrepared(Uuid),

    /// Approve one retry of a recent auto-review denial selected in the TUI.
    ApproveRecentAutoReviewDenial {
        thread_id: ThreadId,
        id: String,
    },

    /// Kick off an asynchronous file search for the given query (text after
    /// the `@`). Previous searches may be cancelled by the app layer so there
    /// is at most one in-flight search.
    StartFileSearch(String),

    /// Result of a completed asynchronous file search. The `query` echoes the
    /// original search term so the UI can decide whether the results are
    /// still relevant.
    FileSearchResult {
        query: String,
        matches: Vec<FileMatch>,
    },

    /// Same-host task results for the active unified mention query.
    TaskSearchResult {
        thread_id: ThreadId,
        query: String,
        matches: Vec<crate::task_mentions::TaskMention>,
    },

    /// Refresh account rate limits in the background.
    RefreshRateLimits {
        origin: RateLimitRefreshOrigin,
    },

    /// Reconcile inherited account usage with an attached task before its queued input runs.
    ApplyBackendBannerFallback {
        thread_id: ThreadId,
    },

    /// Open the current thread goal summary/action menu.
    OpenThreadGoalMenu {
        thread_id: ThreadId,
    },

    /// Open an editor for the current thread goal objective.
    OpenThreadGoalEditor {
        thread_id: Option<ThreadId>,
    },

    /// Materialize and set or replace the current thread goal objective.
    SetThreadGoalDraft {
        thread_id: ThreadId,
        draft: GoalDraft,
        mode: ThreadGoalSetMode,
    },

    /// Pause or resume the current thread goal.
    SetThreadGoalStatus {
        thread_id: ThreadId,
        status: ThreadGoalStatus,
    },

    /// Clear the current thread goal.
    ClearThreadGoal {
        thread_id: ThreadId,
    },

    /// Result of refreshing rate limits.
    RateLimitsLoaded {
        request_id: u64,
        origin: RateLimitRefreshOrigin,
        hard_stop_generation: u64,
        result: Result<GetAccountRateLimitsResponse, String>,
    },

    /// Open the authenticated account analytics dashboard.
    OpenAnalytics {
        view: Option<crate::analytics::TokenActivityView>,
    },

    OpenAccountingInspector {
        day: i64,
    },
    LoadAccountingInspector {
        generation: uuid::Uuid,
        thread: Option<ThreadId>,
        day: i64,
        range: Option<codex_state::accounting::InspectionRange>,
    },
    AccountingInspectorLoaded {
        generation: uuid::Uuid,
        thread: Option<ThreadId>,
        day: i64,
        result: Result<codex_state::accounting::InspectionDay, String>,
    },
    NavigateAccountingInspector {
        generation: uuid::Uuid,
        page: usize,
    },
    CloseAccountingInspector {
        generation: uuid::Uuid,
    },
    RefreshAccountingInspector {
        generation: uuid::Uuid,
    },

    /// Open the reset-credit flow selected from the `/usage` menu.
    OpenRateLimitResetCredits,

    /// Confirm the reset credit selected from the reset-credit picker.
    OpenRateLimitResetConfirmation {
        picker_request_id: u64,
        confirmation_gate: Arc<AtomicBool>,
        credit_id: Option<String>,
        reset_title: String,
        reset_detail: Option<String>,
        reset_description: String,
    },

    /// Consume one reset credit using a stable idempotency key.
    ConsumeRateLimitResetCredit {
        idempotency_key: String,
        credit_id: Option<String>,
    },

    /// Result of consuming one reset credit.
    RateLimitResetCreditConsumed {
        request_id: u64,
        idempotency_key: String,
        credit_id: Option<String>,
        result: Result<ConsumeAccountRateLimitResetCreditResponse, String>,
    },

    /// Fetch backend-estimated usage for the currently visible enterprise thread.
    RefreshThreadUsage {
        thread_id: ThreadId,
        request_id: u64,
    },

    /// Result of fetching backend-estimated usage for a specific thread.
    ThreadUsageLoaded {
        thread_id: ThreadId,
        request_id: u64,
        result: Result<ThreadUsageOutcome, String>,
    },

    /// Result of fetching usage for the selected dashboard task.
    AgentsOverviewUsageLoaded {
        thread_id: ThreadId,
        request_id: Uuid,
        result: Result<ThreadUsageOutcome, String>,
    },

    /// Fetch workspace messages for the status-line headline item.
    RefreshStatusLineWorkspaceHeadline {
        request_id: u64,
    },

    /// Commit settled asynchronous usage output after active-output barriers clear.
    CommitPendingUsageOutput,

    /// Commit settled asynchronous usage output after stream shutdown.
    CommitPendingUsageOutputAfterStreamShutdown,

    /// Send a user-confirmed request to notify the workspace owner.
    SendAddCreditsNudgeEmail {
        credit_type: AddCreditsNudgeCreditType,
    },

    /// Result of notifying the workspace owner.
    AddCreditsNudgeEmailFinished {
        request_id: Uuid,
        result: Result<AddCreditsNudgeEmailStatus, String>,
    },

    /// Result of prefetching connectors.
    ConnectorsLoaded {
        thread_id: Option<ThreadId>,
        cwd: PathBuf,
        generation: ConnectorScopeGeneration,
        result: Result<ConnectorsSnapshot, String>,
        is_final: bool,
    },

    /// Thread-scoped installed applications that may actually be mentioned.
    InstalledConnectorMentionsLoaded {
        thread_id: Option<ThreadId>,
        cwd: PathBuf,
        generation: ConnectorScopeGeneration,
        result: Result<ConnectorsSnapshot, String>,
    },

    /// Result of computing a `/diff` command.
    DiffResult(PathBuf, String),

    /// Result of building the terminal-native MkDocs view for `/docs`.
    MkDocsResult(Result<MkDocsSite, String>),

    /// Open the app link view in the bottom pane.
    OpenAppLink {
        app_id: String,
        title: String,
        description: Option<String>,
        instructions: String,
        url: String,
        is_installed: bool,
        is_enabled: bool,
    },

    /// Open the provided URL in the user's browser.
    OpenUrlInBrowser {
        url: String,
    },

    /// Open the current thread in Codex Desktop.
    OpenDesktopThread {
        thread_id: ThreadId,
    },

    /// Persist a pet selection and reload the ambient pet.
    PetSelected {
        pet_id: String,
    },

    /// Persist terminal pets as disabled and remove the ambient pet.
    PetDisabled,

    /// Start loading the side preview for the pet picker.
    PetPreviewRequested {
        pet_id: String,
    },

    /// Result of loading the side preview for the pet picker.
    PetPreviewLoaded {
        request_id: u64,
        result: Result<crate::pets::AmbientPet, String>,
    },

    /// Result of loading the selected ambient pet before config persistence.
    PetSelectionLoaded {
        request_id: u64,
        pet_id: String,
        result: Result<Option<crate::pets::AmbientPet>, String>,
    },

    /// Result of restoring the configured ambient pet during startup.
    ConfiguredPetLoaded {
        generation: ConfiguredPetLoadGeneration,
        pet_id: String,
        result: Result<Option<crate::pets::AmbientPet>, String>,
    },

    /// Refresh app connector state and mention bindings.
    RefreshConnectors {
        force_refetch: bool,
    },

    /// Fetch apps only while the originating account, workspace, and thread remain current.
    FetchConnectorsList {
        force_refetch: bool,
        generation: ConnectorScopeGeneration,
    },

    /// Refresh callable installed applications without loading the app directory.
    FetchInstalledConnectorMentions {
        force_refresh: bool,
        generation: ConnectorScopeGeneration,
    },

    /// Fetch plugin marketplace state for the provided working directory.
    FetchPluginsList {
        cwd: PathBuf,
    },

    /// Fetch lifecycle hook inventory for the current thread or working directory.
    FetchHooksList {
        target: crate::hooks_rpc::HooksListTarget,
    },

    /// Result of fetching plugin marketplace state.
    PluginsLoaded {
        cwd: PathBuf,
        result: Result<PluginListResponse, String>,
    },

    /// Open the plugin list from an already cached response.
    OpenPluginsList {
        cwd: PathBuf,
        response: PluginListResponse,
    },

    /// Result of explicitly fetching remote-backed plugin sections.
    PluginRemoteSectionsLoaded {
        cwd: PathBuf,
        marketplaces: Vec<PluginMarketplaceEntry>,
        section_errors: Vec<PluginRemoteSectionError>,
    },

    /// Result of fetching lifecycle hook inventory.
    HooksLoaded {
        target: crate::hooks_rpc::HooksListTarget,
        result: Result<codex_app_server_protocol::HooksListEntry, String>,
    },

    /// Open the prompt for adding a marketplace source.
    OpenMarketplaceAddPrompt,

    /// Replace the plugins popup with a marketplace-add loading state.
    OpenMarketplaceAddLoading {
        source: String,
    },

    /// Add a marketplace from the provided source.
    FetchMarketplaceAdd {
        cwd: PathBuf,
        source: String,
    },

    /// Result of adding a marketplace.
    MarketplaceAddLoaded {
        cwd: PathBuf,
        source: String,
        result: Result<MarketplaceAddResponse, String>,
    },

    /// Open the confirmation prompt for removing a marketplace.
    OpenMarketplaceRemoveConfirm {
        marketplace_name: String,
        marketplace_display_name: String,
    },

    /// Replace the plugins popup with a marketplace-remove loading state.
    OpenMarketplaceRemoveLoading {
        marketplace_display_name: String,
    },

    /// Remove a marketplace by name.
    FetchMarketplaceRemove {
        cwd: PathBuf,
        marketplace_name: String,
        marketplace_display_name: String,
    },

    /// Result of removing a marketplace.
    MarketplaceRemoveLoaded {
        cwd: PathBuf,
        marketplace_name: String,
        marketplace_display_name: String,
        result: Result<MarketplaceRemoveResponse, String>,
    },

    /// Replace the plugins popup with a marketplace-upgrade loading state.
    OpenMarketplaceUpgradeLoading {
        marketplace_name: Option<String>,
    },

    /// Upgrade configured Git marketplaces.
    FetchMarketplaceUpgrade {
        cwd: PathBuf,
        marketplace_name: Option<String>,
    },

    /// Result of upgrading configured Git marketplaces.
    MarketplaceUpgradeLoaded {
        cwd: PathBuf,
        result: Result<MarketplaceUpgradeResponse, String>,
    },

    /// Replace the plugins popup with a plugin-detail loading state.
    OpenPluginDetailLoading {
        plugin_display_name: String,
    },

    /// Fetch detail for a specific plugin from a marketplace.
    FetchPluginDetail {
        cwd: PathBuf,
        params: PluginReadParams,
    },

    /// Result of fetching plugin detail.
    PluginDetailLoaded {
        cwd: PathBuf,
        result: Result<PluginReadResponse, String>,
    },

    /// Replace the plugins popup with an install loading state.
    OpenPluginInstallLoading {
        plugin_display_name: String,
    },

    /// Replace the plugins popup with an uninstall loading state.
    OpenPluginUninstallLoading {
        plugin_display_name: String,
    },

    /// Install a specific plugin from a marketplace.
    FetchPluginInstall {
        cwd: PathBuf,
        location: PluginLocation,
        plugin_name: String,
        plugin_display_name: String,
    },

    /// Result of installing a plugin.
    PluginInstallLoaded {
        cwd: PathBuf,
        location: PluginLocation,
        plugin_name: String,
        plugin_display_name: String,
        result: Result<PluginInstallResponse, String>,
    },

    /// Uninstall a specific plugin by canonical plugin id.
    FetchPluginUninstall {
        cwd: PathBuf,
        plugin_id: String,
        plugin_display_name: String,
    },

    /// Result of uninstalling a plugin.
    PluginUninstallLoaded {
        cwd: PathBuf,
        plugin_id: String,
        plugin_display_name: String,
        result: Result<PluginUninstallResponse, String>,
    },

    /// Enable or disable an installed plugin.
    SetPluginEnabled {
        cwd: PathBuf,
        plugin_id: String,
        enabled: bool,
    },

    /// Result of enabling or disabling a plugin.
    PluginEnabledSet {
        cwd: PathBuf,
        plugin_id: String,
        enabled: bool,
        result: Result<(), String>,
    },

    /// Refresh plugin mention bindings from the current config.
    RefreshPluginMentions,

    /// Result of refreshing plugin mention bindings.
    PluginMentionsLoaded {
        cwd: PathBuf,
        plugins: Option<Vec<PluginCapabilitySummary>>,
    },

    /// Advance the post-install plugin app-auth flow.
    PluginInstallAuthAdvance {
        refresh_connectors: bool,
    },

    /// Abandon the post-install plugin app-auth flow.
    PluginInstallAuthAbandon,

    /// Fetch MCP inventory via app-server RPCs and render it into history.
    FetchMcpInventory {
        detail: McpServerStatusDetail,
        thread_id: Option<ThreadId>,
    },

    StartMcpLogin {
        name: String,
        thread_id: ThreadId,
    },

    McpLoginStarted {
        request_id: String,
        result: Result<McpServerOauthLoginResponse, String>,
    },

    /// Result of fetching MCP inventory via app-server RPCs.
    McpInventoryLoaded {
        result: Result<Vec<McpServerStatus>, String>,
        detail: McpServerStatusDetail,
        thread_id: Option<ThreadId>,
    },

    /// Result of the startup skills refresh that runs after the first frame is scheduled.
    ///
    /// This event is startup-only. Interactive skills refreshes are handled synchronously through the app
    /// command path because those callers expect the visible skill state to be current when their command
    /// completes.
    SkillsListLoaded {
        cwd: PathBuf,
        result: Result<SkillsListResponse, String>,
    },

    /// Begin buffering initial resume replay rows before they are written to scrollback.
    BeginInitialHistoryReplayBuffer,

    /// Begin buffering thread-switch replay cells so the final scrollback write can reuse the
    /// resize-reflow tail renderer.
    BeginThreadSwitchHistoryReplayBuffer,

    /// Resume following the transcript after an explicit local command submission.
    /// Background output and later refreshes must preserve the user's reading position.
    FollowTranscript,

    InsertHistoryCell(Box<dyn HistoryCell>),
    /// FIFO barrier after the completed turn's history insertions.
    TurnTipReady {
        thread_id: ThreadId,
        turn_id: String,
    },

    /// Move visible completed voice captions into history in one app event.
    CommitRealtimeTranscriptHistory,

    VoiceControl {
        thread_id: Option<ThreadId>,
        control: VoiceControl,
    },
    RealtimeConversationStateChanged,
    BackgroundVoiceError {
        thread_id: ThreadId,
        message: String,
    },

    /// Finish buffering initial resume replay after all replay events have been queued.
    EndInitialHistoryReplayBuffer,

    /// Replace the contiguous run of streaming `AgentMessageCell`s at the end of
    /// the transcript with a single `AgentMarkdownCell` that stores the raw
    /// markdown source and re-renders from it on resize.
    ///
    /// Emitted by `ChatWidget::flush_answer_stream_with_separator` after stream
    /// finalization. The `App` handler walks backward through `transcript_cells`
    /// to find the `AgentMessageCell` run and splices in the consolidated cell.
    /// The `cwd` keeps local file-link display stable across the final re-render.
    /// `scrollback_reflow` lets table-tail finalization force the already-emitted
    /// terminal scrollback to be rebuilt from the consolidated source-backed cell.
    /// `deferred_history_cell` lets callers add the final stream tail to the
    /// transcript without first writing its provisional render to scrollback.
    ConsolidateAgentMessage {
        source: String,
        copy_source: Option<String>,
        cwd: PathBuf,
        inline_visualization_context: Option<InlineVisualizationContext>,
        scrollback_reflow: ConsolidationScrollbackReflow,
        deferred_history_cell: Option<Box<dyn HistoryCell>>,
    },

    /// Replace the contiguous run of streaming `ProposedPlanStreamCell`s at the
    /// end of the transcript with a single source-backed `ProposedPlanCell`.
    ///
    /// Emitted by `ChatWidget::on_plan_item_completed` after plan stream
    /// finalization.
    ConsolidateProposedPlan(String),

    StartCommitAnimation,
    StopCommitAnimation,

    /// Update the current reasoning effort in the running app and widget.
    UpdateReasoningEffort(Option<ReasoningEffort>),

    /// Change Reserve effort only on the task that opened the picker, without saving defaults.
    UpdateLunaReserveReasoning {
        thread_id: ThreadId,
        effort: Option<ReasoningEffort>,
    },

    /// Update the current model slug in the running app and widget.
    UpdateModel(String),

<<<<<<< HEAD
    /// Update model and provider together so active thread settings stay coherent.
    UpdateModelSelection {
        model: String,
        provider: Option<String>,
    },

    /// Update the current personality in the running app and widget.
    UpdatePersonality(Personality),
||||||| 413492cd6c
    /// Update the current personality in the running app and widget.
    UpdatePersonality(Personality),
=======
    /// Apply a model-picker selection to the active conversation and saved local defaults.
    ApplyModelSelection {
        model: String,
        effort: Option<ReasoningEffort>,
    },

    /// Apply an explicit Astra selection and offer the flourish only if it changed the model on
    /// its original task. Automatic model updates do not use this event.
    AstraSelectedFromModelPicker {
        thread_id: ThreadId,
        model: String,
        action: AstraModelPickerAction,
    },

    /// Result of creating a TUI-owned WebRTC offer for an active thread.
    RealtimeWebrtcOfferCreated {
        thread_id: ThreadId,
        attempt_id: u64,
        result: Result<StartedRealtimeWebrtcSession, RealtimeWebrtcStartupFailure>,
    },

    /// Result of establishing the WebRTC connection for an active voice attempt.
    RealtimeWebrtcConnected {
        thread_id: ThreadId,
        attempt_id: u64,
        result: Result<(), codex_realtime_webrtc::ConnectionError>,
    },

    /// Stop voice on its original thread after its chat widget is replaced.
    StopRealtimeConversation {
        thread_id: ThreadId,
    },
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

    /// Finish a settings selection after its preceding update events have been applied.
    SettingsSelectionClosed,
    /// Run after any nested settings events emitted while handling the close event.
    SettingsSelectionSettled,

    /// Persist the selected model and reasoning effort to the appropriate config.
    PersistModelSelection {
        model: String,
        provider: Option<String>,
        effort: Option<ReasoningEffort>,
    },

    /// Apply a model, effort, and optional speed only to the active session, preserving saved defaults.
    SelectSessionModel {
        model: String,
        effort: Option<ReasoningEffort>,
        service_tier: Option<String>,
    },

    OpenQuickDialEditor {
        target: crate::chatwidget::quick_dials::QuickDialEditorTarget,
        dial: Option<codex_config::types::QuickDial>,
    },
    OpenQuickDialSavePicker(codex_config::types::QuickDial),
    ConfirmQuickDial {
        slot: u8,
        dial: Option<codex_config::types::QuickDial>,
    },
    SaveQuickDial {
        slot: u8,
        dial: Option<codex_config::types::QuickDial>,
    },
    RecallQuickDial {
        slot: u8,
        thread_id: Option<ThreadId>,
    },

    /// Show the cyber auto-review notice after the model selection confirmation.
    CyberModelAutoReviewNotice,

    /// Read the owning server preference before showing the voice picker.
    OpenRealtimeSettings,
    OpenRealtimeSoundDevices,
    OpenRealtimeVoices,
    OpenRealtimeDevicePicker {
        kind: codex_realtime_webrtc::AudioDeviceKind,
    },
    OpenRealtimeInputChannels {
        device: codex_realtime_webrtc::AudioDevice,
    },
    RealtimeDevicesListed {
        origin: Option<ThreadId>,
        kind: codex_realtime_webrtc::AudioDeviceKind,
        result: Result<Vec<codex_realtime_webrtc::AudioDevice>, String>,
    },
    PersistRealtimeDevice {
        kind: codex_realtime_webrtc::AudioDeviceKind,
        name: Option<String>,
    },
    PersistRealtimeInputChannel {
        channel: Option<codex_config::config_toml::MicrophoneChannels>,
    },

    /// Save the voice for subsequent conversations through the app server.
    PersistRealtimeVoiceSelection {
        voice: codex_protocol::protocol::RealtimeVoice,
    },

    /// Persist the selected service tier to the appropriate config.
    PersistServiceTierSelection {
        service_tier: Option<String>,
    },

    /// Persist the current thread's Daybreak preference and the new-thread default.
    PersistDaybreakSelection {
        thread_id: ThreadId,
        enabled: bool,
    },

    /// Fetch the current catalog even when cached models produce no picker.
    FetchModels {
        request_id: uuid::Uuid,
    },
    ModelsLoaded {
        request_id: uuid::Uuid,
        result: Result<Vec<ModelPreset>, String>,
    },
    /// Finished the API-key compatibility check; do not check the same catalog again.
    ApiKeyCompatModelsLoaded {
        request_id: uuid::Uuid,
        result: Result<Vec<ModelPreset>, String>,
    },

    FetchPermissionProfiles {
        request_id: uuid::Uuid,
        thread_cwd: Option<PathBuf>,
    },
    PermissionProfilesLoaded {
        request_id: uuid::Uuid,
        result: Result<crate::permission_discovery::PermissionDiscovery, String>,
    },

    /// Open the reasoning selection popup after picking a model.
    OpenReasoningPopup {
        model: ModelPreset,
        purpose: crate::chatwidget::ModelSelectionPurpose,
    },

    /// Open the explicit Max/Ultra reasoning selection popup for a model.
    OpenAdvancedReasoningPopup {
        model: ModelPreset,
        purpose: crate::chatwidget::ModelSelectionPurpose,
    },

    /// Apply an advanced reasoning effort to the active conversation without changing defaults.
    ApplyAdvancedReasoning {
        model: String,
        effort: ReasoningEffort,
    },

    /// Open the Plan-mode reasoning scope prompt for the selected model/effort.
    OpenPlanReasoningScopePrompt {
        model: String,
        provider: Option<String>,
        effort: Option<ReasoningEffort>,
    },

    /// Open the full model picker (non-auto models).
    OpenAllModelsPopup,

    /// Open the durable GPU rental and recipe surface.
    OpenGpuMenu,
    OpenGpuRental {
        rental_id: String,
    },
    DisableGpuServing {
        rental_id: String,
    },
    TerminateGpuRental {
        rental_id: String,
    },
    OpenGpuAuthorizationPrompt {
        recipe_id: String,
        state: GpuAuthorizationPromptState,
    },
    SearchGpuOffers {
        recipe_id: String,
        maximum_hourly_microusd: i64,
        maximum_total_microusd: i64,
        ttl_minutes: i64,
    },
    GpuOffersLoaded {
        recipe_id: String,
        authorization: codex_gpu_market::RentalAuthorization,
        offers: Result<Vec<codex_gpu_market::GpuOffer>, String>,
    },
    OpenGpuConfirmation {
        recipe_id: String,
        authorization: codex_gpu_market::RentalAuthorization,
        offer: codex_gpu_market::GpuOffer,
    },
    ConfirmGpuRental {
        recipe_id: String,
        authorization: codex_gpu_market::RentalAuthorization,
        offer: codex_gpu_market::GpuOffer,
    },
    GpuRentalConfirmationFinished {
        result: Result<codex_state::GpuRental, String>,
    },
    OpenGpuProviderCredential {
        provider: String,
    },
    SaveGpuProviderCredential {
        provider: String,
        api_key: ProviderApiKeySecret,
    },

    /// Open masked entry for a provider API key.
    OpenProviderApiKeyAdd {
        provider_id: String,
        provider_name: String,
        env_key: String,
    },

    /// Store a provider API key through app-server account login.
    SaveProviderApiKey {
        provider_id: String,
        display_name: String,
        api_key: ProviderApiKeySecret,
    },
    OpenSharedProviderSetup,
    ProviderAccountMetadataResolved {
        metadata: crate::provider_status_host::ProviderAccountMetadata,
    },
    SharedProviderSetupBegin {
        provider_id: codex_provider_auth::ProviderCatalogId,
        capability: codex_provider_auth::ProviderSetupCapability,
    },
    SharedProviderSetupSelectExisting {
        provider_id: codex_provider_auth::ProviderCatalogId,
        runtime_provider_id: codex_provider_auth::ProviderRuntimeId,
    },
    SharedProviderSetupQueueCorbanu(bool),
    SaveSharedProviderApiKey {
        target: codex_provider_auth::ApiKeyAuthTarget,
        api_key: ProviderApiKeySecret,
    },
    SharedProviderApiKeyFinished {
        target: codex_provider_auth::ApiKeyAuthTarget,
        result: Result<codex_provider_auth::ProviderStatusSnapshot, String>,
    },
    SharedProviderSetupDone,
    SharedProviderSetupCancelled,
    ProviderAccountLoginCompleted {
        login_id: Option<String>,
        success: bool,
    },
    SharedProviderAuthAction(codex_provider_auth::ProviderAuthAction),

    OpenProviderManager,
    ProviderManagerStatusesResolved {
        generation: u64,
        status_host: crate::provider_status_host::ProviderStatusHost,
        statuses: Vec<codex_provider_auth::ProviderStatusSnapshot>,
        /// PF-84-S04: named accounts, when the `named_accounts` feature is on.
        accounts: Option<Vec<crate::provider_named_accounts::NamedAccountRow>>,
    },
    /// PF-84-S04: a named-account action from `/providers`.
    ProviderAccount(crate::provider_named_accounts::ProviderAccountEvent),
    /// PF-84 (#572): whether the vault holds a named Bedrock account (`None`
    /// when it could not be read), for the check numbered `generation`.
    NamedBedrockAccountChecked {
        generation: u64,
        offered: Option<bool>,
    },
    ProviderManagerMetadataResolved {
        generation: u64,
        metadata: crate::provider_status_host::ProviderAccountMetadata,
    },
    OpenProviderManagerActions {
        provider_id: codex_provider_auth::ProviderCatalogId,
    },
    OpenProviderManagerRecovery {
        provider_id: codex_provider_auth::ProviderCatalogId,
    },
    ProviderManagerApiKeyCancelled {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        provider_id: codex_provider_auth::ProviderCatalogId,
    },
    ProviderManagerBeginAuthentication {
        provider_id: codex_provider_auth::ProviderCatalogId,
        capability: codex_provider_auth::ProviderSetupCapability,
    },
    ProviderManagerClaudeRecoverySourceResolved {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target: codex_provider_auth::claude_account_flow::ClaudeAccountTarget,
        status: codex_provider_auth::ProviderStatusSnapshot,
        source: codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource,
    },
    SaveProviderManagerApiKey {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target: codex_provider_auth::ApiKeyAuthTarget,
        api_key: ProviderApiKeySecret,
    },
    ProviderManagerApiKeyFinished {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target: codex_provider_auth::ApiKeyAuthTarget,
        status: Option<codex_provider_auth::ProviderStatusSnapshot>,
    },
    ProviderManagerRequestPolicy {
        provider_id: codex_provider_auth::ProviderCatalogId,
        policy: codex_provider_auth::ProviderActivationPolicy,
    },
    ProviderManagerChooseReplacement {
        target_provider_id: codex_provider_auth::ProviderCatalogId,
        replacement: codex_provider_auth::ExplicitProviderSelection,
    },
    ProviderManagerCancelReplacement {
        target_provider_id: codex_provider_auth::ProviderCatalogId,
    },
    ProviderManagerPersistenceFinished {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        result: codex_provider_auth::ProviderManagementPersistenceResult,
    },
    ProviderManagerReplacementPersisted {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
        replacement: codex_provider_auth::ExplicitProviderSelection,
        success: bool,
    },
    ProviderManagerDeactivateAfterReplacement {
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
    },

    ProviderApiKeySaveDismissed {
        save_id: String,
    },

    /// Finish a provider API-key save after app-server has persisted and reloaded auth.
    ProviderApiKeySaveFinished {
        save_id: String,
        display_name: String,
        result: Result<(), String>,
    },

    OpenTelegram,
    OpenTelegramTokenEntry,
    ValidateTelegramToken {
        token: TelegramBotSecret,
    },
    TelegramTokenValidated {
        result: Result<crate::chatwidget::telegram_setup::TelegramBotIdentity, String>,
    },
    DiscoverTelegramChats,
    PollTelegramChats {
        generation: u64,
    },
    TelegramChatsDiscovered {
        generation: u64,
        result: Result<crate::chatwidget::telegram_setup::TelegramDiscovery, String>,
    },
    ConfirmTelegramChat {
        candidate: crate::chatwidget::telegram_setup::TelegramChatCandidate,
    },
    ConnectTelegramChat {
        candidate: crate::chatwidget::telegram_setup::TelegramChatCandidate,
        defaults: crate::chatwidget::telegram_setup::TelegramConnectionDefaults,
    },
    StartTelegramConnector,
    StopTelegramConnector,
    ReplaceTelegramBot,
    ConfirmTelegramDisconnect,
    DisconnectTelegram,
    TelegramOperationFinished {
        result: Result<String, String>,
    },
    TelegramStatusReady {
        result: Result<crate::chatwidget::telegram_setup::TelegramStatus, String>,
    },

    /// Start OpenAI Codex account device-code login from the Providers screen.
    OpenCodexAccountDeviceLogin,

    /// Open the explicit Claude Plan authentication-method choice.
    OpenClaudeCodePlanLogin,

    /// Open the safe handoff for the official long-lived `claude setup-token` flow.
    RunClaudeSetupToken,

    /// Use or repair Claude Code's rotating subscription login.
    UseClaudeCodePlanLogin,

    /// Persist an explicitly configured legacy environment token as the exact source.
    UseLegacyClaudeEnvironmentToken,

    /// Completion of the metadata-only legacy environment-token recovery.
    LegacyClaudeEnvironmentTokenSelected {
        result: Result<String, String>,
    },

    /// Completion of the off-event-loop Claude Code status probe.
    ClaudeCodePlanLoginSelectionChecked {
        result: Result<bool, String>,
    },

    /// Store and select a masked long-lived Claude subscription token.
    SaveClaudeManagedSubscriptionToken {
        token: ClaudeSubscriptionTokenSecret,
    },

    /// Completion of the encrypted managed-token enrollment transaction.
    ClaudeManagedSubscriptionTokenSaved {
        result: Result<String, String>,
    },

    /// Claude Code opened a browser login URL and is waiting for its authorization code.
    ClaudeCodePlanLoginReady {
        verification_url: String,
        input_tx: tokio::sync::mpsc::UnboundedSender<
            crate::chatwidget::claude_code_login::ClaudeCodeLoginInput,
        >,
    },

    /// Open masked entry for the Claude Code OAuth authorization code.
    OpenClaudeCodePlanLoginCodeEntry {
        input_tx: tokio::sync::mpsc::UnboundedSender<
            crate::chatwidget::claude_code_login::ClaudeCodeLoginInput,
        >,
    },

    /// Claude Code's login subprocess completed or failed.
    ClaudeCodePlanLoginFinished {
        result: Option<Result<String, String>>,
    },

    /// Latest provider credential statuses loaded away from the TUI event thread.
    // Unreachable since provider management was unified (3db6321294); remove
    // with the rest of this status plumbing or wire it back in.
    #[allow(dead_code)]
    ProviderCredentialStatusesReady {
        claude_status: crate::chatwidget::claude_code_login::ClaudeCodePlanStatus,
        pfterminal_plan_status: crate::chatwidget::provider_credentials::PfTerminalPlanStatus,
        api_key_statuses: Vec<(
            String,
            crate::chatwidget::provider_credentials::ProviderApiKeyStatus,
        )>,
    },

    /// Device-code login data returned by account/login/start.
    CodexAccountDeviceLoginReady {
        login_id: String,
        verification_url: String,
        user_code: String,
    },

    /// Device-code login failed before a user code was available.
    CodexAccountDeviceLoginFailed {
        message: String,
    },

    /// Cancel a pending OpenAI Codex account device-code login.
    CancelCodexAccountDeviceLogin {
        login_id: String,
    },

    /// Open the masked vault credential-entry overlay.
    OpenVaultCredentialAdd,

    VaultCredentialAddRequested {
        label: String,
        secret: VaultSecret,
    },

    VaultCredentialAdded {
        label: String,
        result: Result<(), String>,
    },

    OpenWallet,
    OpenWalletPlanUsage,
    WalletPlanUsageReady {
        result: Result<crate::chatwidget::wallet_usage::WalletPlanUsageOverview, String>,
    },
    OpenWalletCreate,
    OpenWalletRestore,
    OpenWalletRestorePasscode {
        recovery: WalletSecret,
    },
    OpenWalletRecoveryBackup,
    WalletRecoveryBackupFinished {
        result: Result<WalletCreatedResult, String>,
    },
    OpenWalletUnlock {
        policy: codex_wallet_daemon::UnlockPolicy,
        continuation: WalletUnlockContinuation,
    },
    WalletUnlockPreflightFinished {
        policy: codex_wallet_daemon::UnlockPolicy,
        continuation: WalletUnlockContinuation,
        result: Result<bool, String>,
    },
    OpenWalletCustomUnlock {
        validation_error: Option<String>,
        continuation: WalletUnlockContinuation,
    },
    WalletLockRequested,
    ConfirmWalletPlanDisconnect,
    WalletPlanDisconnectRequested,
    WalletPlanDisconnected {
        result: Result<bool, String>,
    },
    ConfirmWalletRemoval {
        address: String,
    },
    WalletRemoveRequested {
        address: String,
    },
    WalletRemoved {
        result: Result<(), String>,
    },
    WalletStatusReady {
        generation: u64,
        result: Result<crate::chatwidget::wallet_menu::WalletOverview, String>,
    },
    OpenCorbanuApi {
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    CorbanuApiLoaded {
        result: Result<crate::chatwidget::wallet_api::CorbanuApiView, String>,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
        /// The daemon refused the held wallet capability as lapsed.
        capability_lapsed: bool,
    },
    OpenCorbanuApiTopUp {
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    ConfirmCorbanuApiTopUp {
        amount_usd: String,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    ConfirmCorbanuApiKeyRevocation {
        key_id: String,
        display_prefix: String,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    CorbanuApiOperationRequested {
        operation: codex_wallet::CorbanuApiOperation,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    CorbanuApiOperationFinished {
        result: Result<codex_wallet::CorbanuApiOperationResult, String>,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    WalletCreateFinished {
        operation: WalletPersistenceOperation,
        result: Result<WalletCreatedResult, String>,
    },
    WalletUnlockFinished {
        policy: codex_wallet_daemon::UnlockPolicy,
        continuation: WalletUnlockContinuation,
        result: Result<WalletUnlockedResult, String>,
    },
    OpenWalletPlans {
        mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode,
    },
    BeginDeferredCorbanuPlan {
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    },
    DeferredWalletPreflightFinished {
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
        result: Result<DeferredWalletPreflight, String>,
    },
    WalletPlansReady {
        mode: crate::chatwidget::wallet_menu::WalletPlanPurchaseMode,
        result: Result<crate::chatwidget::wallet_menu::WalletPlanCatalog, String>,
    },
    ConfirmWalletPlanPurchase {
        plan: crate::chatwidget::wallet_menu::WalletPlanChoice,
    },
    WalletPlanPurchaseRequested {
        plan: crate::chatwidget::wallet_menu::WalletPlanChoice,
    },
    WalletPlanProvisioned {
        operation: WalletPlanProvisioningOperation,
        result: Result<WalletPlanProvisionedResult, String>,
        deferred_setup: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    },
    WalletPlanCredentialPersistenceFinished {
        attempt_id: WalletPlanPersistenceAttemptId,
        operation: WalletPlanProvisioningOperation,
        plan_id: String,
        purchase: Option<WalletPlanPurchaseSummary>,
        deferred_setup: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
        api_key: Option<WalletSecret>,
        result: Result<(), WalletPlanCredentialPersistenceError>,
    },
    DeferredCorbanuPlanCancelled {
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    },
    DeferredCorbanuPlanConfigured {
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    },
    WalletPlanReceiptReady {
        attempt_id: WalletPlanPersistenceAttemptId,
        selection_policy: WalletPlanReceiptSelectionPolicy,
        receipt: crate::chatwidget::wallet_receipt::WalletPlanReceipt,
    },
    // Unreachable since the legacy plan state was retired (66ff6579d7); remove
    // or wire back in.
    #[allow(dead_code)]
    OpenWalletPlanReceipt {
        receipt: crate::chatwidget::wallet_receipt::WalletPlanReceipt,
    },
    CloseWalletPlanReceipt,
    // Unreachable since the legacy plan state was retired (66ff6579d7); remove
    // or wire back in.
    #[allow(dead_code)]
    WalletRecoverPlanRequested,

    /// Open the vault credential list.
    OpenVaultCredentialsList,

    /// Open the per-credential vault action menu.
    OpenVaultCredentialActions {
        label: String,
    },

    /// Copy a vault credential secret to the clipboard without writing it to history.
    OpenVaultCopySecret {
        label: String,
    },

    /// Reveal a vault secret only inside a host-owned secure view.
    OpenVaultRevealSecret {
        label: String,
    },

    VaultRevealSecretFinished {
        label: String,
        result: Result<VaultSecret, String>,
    },

    OpenVaultReplaceSecret {
        label: String,
    },

    VaultCredentialReplaceRequested {
        label: String,
        secret: VaultSecret,
    },

    VaultCredentialReplaced {
        label: String,
        result: Result<(), String>,
    },

    ConfirmVaultCredentialDelete {
        label: String,
    },

    VaultCredentialDeleteRequested {
        label: String,
    },

    VaultCredentialDeleted {
        label: String,
        result: Result<bool, String>,
    },

    VaultMenuCredentialsReady {
        result: Result<Vec<codex_vault::VaultCredentialMeta>, String>,
    },

    VaultCredentialsReady {
        result: Result<Vec<codex_vault::VaultCredentialMeta>, String>,
    },

    VaultCopySecretFinished {
        label: String,
        result: Result<Option<crate::clipboard_copy::ClipboardLease>, String>,
    },

    /// Open the confirmation prompt before enabling full access mode.
    OpenFullAccessConfirmation {
        preset: ApprovalPreset,
        return_to_permissions: bool,
        profile_selection: Option<PermissionProfileSelection>,
    },

    /// Apply a permission shortcut only while its originating thread is displayed.
    ApplyPermissionShortcut {
        thread_id: ThreadId,
        selection: PermissionProfileSelection,
    },

    /// Refresh server-owned Windows state after selecting a thread or project.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    RefreshWindowsSandbox {
        thread_id: ThreadId,
    },

    /// Prompt to enable the Windows sandbox feature before using Agent mode.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    OpenWindowsSandboxEnablePrompt {
        preset: ApprovalPreset,
        profile_selection: Option<PermissionProfileSelection>,
    },

    /// Open the Windows sandbox fallback prompt after declining or failing elevation.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    OpenWindowsSandboxFallbackPrompt {
        preset: ApprovalPreset,
        profile_selection: Option<PermissionProfileSelection>,
    },

    /// Begin the elevated Windows sandbox setup flow.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    BeginWindowsSandboxElevatedSetup {
        preset: ApprovalPreset,
        profile_selection: Option<PermissionProfileSelection>,
    },

    /// Begin the non-elevated Windows sandbox setup flow.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    BeginWindowsSandboxLegacySetup {
        preset: ApprovalPreset,
        profile_selection: Option<PermissionProfileSelection>,
    },

    /// Enable the Windows sandbox feature and switch to Agent mode.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    EnableWindowsSandboxForAgentMode {
        preset: ApprovalPreset,
        mode: WindowsSandboxEnableMode,
        profile_selection: Option<PermissionProfileSelection>,
    },

    // Handled when sent; nothing constructs it in this build.
    #[allow(dead_code)]
    /// Update the current approval policy in the running app and widget.
    UpdateAskForApprovalPolicy(AskForApproval),

    // Handled when sent; nothing constructs it in this build.
    #[allow(dead_code)]
    /// Update the current built-in active permission profile in the running app and widget.
    UpdateActivePermissionProfile(ActivePermissionProfile),

    /// Select a named permission profile, optionally applying built-in mode settings too.
    SelectPermissionProfile(PermissionProfileSelection),
    SelectPermissionPreset(PermissionProfileSelection),
    PermissionConfirmationCompleted {
        selection_id: uuid::Uuid,
        result: crate::app_server_session::PermissionConfirmationResult,
    },

    /// Update the current approvals reviewer in the running app and widget.
    UpdateApprovalsReviewer(ApprovalsReviewer),

    /// Discover experimental features for the requesting popup only.
    FetchExperimentalFeatures {
        thread_id: ThreadId,
        response_tx: tokio::sync::oneshot::Sender<
            Result<Vec<codex_app_server_protocol::ExperimentalFeature>, String>,
        >,
    },

    /// Update feature flags and persist them to the top-level config.
    UpdateFeatureFlags {
        updates: Vec<(Feature, bool)>,
    },

    /// Save generic menu controls without changing running-task settings.
    SaveExperimentalFeatures {
        thread_id: ThreadId,
        updates: Vec<(String, bool)>,
        response_tx: tokio::sync::oneshot::Sender<Result<FeatureWriteResult, String>>,
    },

    /// Save an enable prompt on the app server without changing the current thread.
    EnableFeatureForNewThreads(Feature),

    /// Update memory settings and persist them to config.toml.
    UpdateMemorySettings {
        use_memories: bool,
        generate_memories: bool,
    },

    /// Clear all persisted local memory artifacts via the app-server.
    ResetMemories,

    /// Update whether the rate limit switch prompt has been acknowledged for the session.
    UpdateRateLimitSwitchPromptHidden(bool),

    /// Update the Plan-mode-specific reasoning effort in memory.
    UpdatePlanModeReasoningEffort(Option<ReasoningEffort>),

    /// Persist the acknowledgement flag for the rate limit switch prompt.
    PersistRateLimitSwitchPromptHidden,

    /// Persist the Plan-mode-specific reasoning effort.
    PersistPlanModeReasoningEffort(Option<ReasoningEffort>),

    /// Persist the acknowledgement flag for the model migration prompt.
    PersistModelMigrationPromptAcknowledged {
        from_model: String,
        to_model: String,
    },

    /// Re-open the approval presets popup.
    OpenApprovalsPopup,

    /// Open the skills list popup.
    OpenSkillsList,

    /// Open the skills enable/disable picker.
    OpenManageSkillsPopup,

    /// Enable or disable a skill by path.
    SetSkillEnabled {
        path: PathUri,
        enabled: bool,
    },

    /// Enable or disable an app by connector ID.
    SetAppEnabled {
        id: String,
        enabled: bool,
    },

    /// Enable or disable a hook by stable hook key.
    SetHookEnabled {
        key: String,
        enabled: bool,
    },

    /// Trust the current definition for a hook by stable hook key.
    TrustHook {
        key: String,
        current_hash: String,
    },

    /// Trust the current definitions for one or more hooks by stable hook key.
    TrustHooks {
        updates: Vec<crate::hooks_rpc::HookTrustUpdate>,
    },

    /// Result of persisting hook enabled state.
    HookEnabledSet {
        key: String,
        enabled: bool,
        result: Result<(), String>,
    },

    /// Result of persisting hook trust state.
    HookTrusted {
        result: Result<(), String>,
    },

    /// Notify that the manage skills popup was closed.
    ManageSkillsClosed,

    /// Re-open the permissions presets popup.
    OpenPermissionsPopup,

    /// Resolve an inline permission profile after any app-owned platform preflight.
    ResolvePermissionProfile {
        profile_id: String,
    },

    /// Open the branch picker option from the review popup.
    OpenReviewBranchPicker(PathBuf),

    /// Open the commit picker option from the review popup.
    OpenReviewCommitPicker(PathBuf),

    /// Open the custom prompt option from the review popup.
    OpenReviewCustomPrompt,

    /// Submit a user message with an explicit collaboration mask.
    SubmitUserMessageWithMode {
        text: String,
        collaboration_mode: CollaborationModeMask,
    },

    /// Open the approval popup.
    FullScreenApprovalRequest(ApprovalRequest),

    /// Inspect the complete verification request without deciding it.
    FullScreenUserVerificationRequest(
        crate::bottom_pane::user_verification::UserVerificationRequest,
    ),

    /// Open the feedback note entry overlay after the user selects a category.
    OpenFeedbackNote {
        category: FeedbackCategory,
        include_logs: bool,
    },

    /// Open the upload consent popup for feedback after selecting a category.
    OpenFeedbackConsent {
        category: FeedbackCategory,
    },

    /// Submit feedback for the current thread via the app-server feedback RPC.
    SubmitFeedback {
        category: FeedbackCategory,
        reason: Option<String>,
        turn_id: Option<String>,
        include_logs: bool,
    },

    /// Result of a feedback upload request initiated by the TUI.
    FeedbackSubmitted {
        origin_thread_id: Option<ThreadId>,
        category: FeedbackCategory,
        include_logs: bool,
        result: Result<String, String>,
    },

    /// Launch the external editor after a normal draw has completed.
    LaunchExternalEditor,

    /// Async update of the current git branch for status line rendering.
    StatusLineBranchUpdated {
        cwd: PathBuf,
        branch: Option<String>,
    },
    /// Async update of Git summary fields for status line rendering.
    StatusLineGitSummaryUpdated {
        cwd: PathBuf,
        summary: crate::chatwidget::StatusLineGitSummary,
    },
    /// Async update of the workspace notification headline for status line rendering.
    StatusLineWorkspaceHeadlineUpdated {
        request_id: u64,
        result: Result<crate::workspace_messages::WorkspaceHeadlineFetchResult, String>,
    },
    /// Apply a user-confirmed status-line item ordering/selection.
    StatusLineSetup {
        items: Vec<StatusLineItem>,
        use_theme_colors: bool,
    },
    /// Dismiss the status-line setup UI without changing config.
    StatusLineSetupCancelled,

    /// Apply a user-confirmed terminal-title item ordering/selection.
    TerminalTitleSetup {
        items: Vec<TerminalTitleItem>,
    },
    /// Apply a temporary terminal-title preview while the setup UI is open.
    TerminalTitleSetupPreview {
        items: Vec<TerminalTitleItem>,
    },
    /// Dismiss the terminal-title setup UI without changing config.
    TerminalTitleSetupCancelled,

    /// Remember the Command Center grouping across launches.
    PersistAgentsOverviewGrouping(codex_config::types::AgentsOverviewGrouping),

    /// Open an existing editor above the configuration panel.
    OpenConfigEditor(crate::config_panel::ConfigEditorHandoff),

    /// Schedule a notification preview with time to switch away from the terminal.
    TestConfigNotification,

    /// Send the matching scheduled preview regardless of the configured focus condition.
    SendConfigTestNotification {
        generation: u64,
    },

    /// Requests saving a local `/config` preference and refreshing its effective value.
    SaveConfigPreference(crate::config_panel::ConfigPreference),

    /// Save the transcript renderer preference for the next launch only.
    FullscreenTranscriptSelected {
        enabled: bool,
    },

    /// Apply a user-confirmed syntax theme selection.
    SyntaxThemeSelected {
        name: String,
    },

    /// Runtime syntax theme preview changed; refresh theme-derived UI colors.
    SyntaxThemePreviewed,

    /// Open set/remove actions for the selected keymap action.
    OpenKeymapActionMenu {
        context: String,
        action: String,
    },

    /// Open binding selection before replacing one binding for an action.
    OpenKeymapReplaceBindingMenu {
        context: String,
        action: String,
    },

    /// Open key capture for the selected keymap action.
    OpenKeymapCapture {
        context: String,
        action: String,
        intent: KeymapEditIntent,
        capture_mode: KeymapCaptureMode,
    },

    /// Open the keymap keypress inspector.
    OpenKeymapDebug,

    /// Apply a captured key to the selected keymap action.
    KeymapCaptured {
        context: String,
        action: String,
        key: String,
        intent: KeymapEditIntent,
    },

    /// Remove the custom root binding for the selected keymap action.
    KeymapCleared {
        context: String,
        action: String,
    },

    /// Generate a recap for the displayed idle thread at the user's request.
    GenerateRecap {
        thread_id: ThreadId,
    },

    /// Recheck whether an unfocused thread is ready for an automatic recap.
    CheckRecap {
        thread_id: ThreadId,
    },

    /// Deliver the result of starting a recap's temporary thread.
    RecapStarted {
        thread_id: ThreadId,
        request_id: Uuid,
        trigger: RecapTrigger,
        completed_turn_count: usize,
        turn_revision: usize,
        history: String,
        result: Result<String, String>,
    },

    /// Deliver the generated recap from a temporary structured turn.
    RecapGenerated {
        thread_id: ThreadId,
        request_id: Uuid,
        trigger: RecapTrigger,
        temporary_thread_id: ThreadId,
        completed_turn_count: usize,
        turn_revision: usize,
        result: Result<String, String>,
    },
}

/// Named profile selection to apply after any required UI guardrails complete.
#[derive(Debug, Clone)]
pub(crate) struct PermissionProfileSelection {
    pub profile_id: String,
    pub approval_policy: Option<AskForApproval>,
    pub approvals_reviewer: Option<ApprovalsReviewer>,
    pub display_label: String,
}

/// The exit strategy requested by the UI layer.
///
/// Most user-initiated exits should use `ShutdownFirst` so core cleanup runs and the UI exits only
/// after core acknowledges completion. `Immediate` is an escape hatch for cases where shutdown has
/// already completed (or is being bypassed) and the UI loop should terminate right away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExitMode {
    /// Shutdown core and exit after completion.
    ShutdownFirst,
    /// Unsubscribe and exit after the current turn was successfully interrupted.
    ShutdownAfterInterrupt,
    /// Exit the UI loop immediately without waiting for shutdown.
    ///
    /// This skips `Op::Shutdown`, so any in-flight work may be dropped and
    /// cleanup that normally runs before `ShutdownComplete` can be missed.
    Immediate,
}

/// Choice made when leaving a daemon-backed task that is still running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunningTaskExitAction {
    CancelTask,
    RunInBackground,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeedbackCategory {
    BadResult,
    GoodResult,
    Bug,
    SafetyCheck,
    Other,
}
