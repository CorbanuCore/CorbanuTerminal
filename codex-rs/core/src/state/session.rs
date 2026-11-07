//! Session-wide mutable state.

use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::models::BaseInstructionsProvenance;
#[cfg(test)]
use codex_protocol::models::ResponseItem;
<<<<<<< HEAD
||||||| 413492cd6c
use codex_sandboxing::policy_transforms::merge_permission_profiles;
use std::collections::HashMap;
=======
use codex_protocol::openai_models::ReasoningEffort;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;

use super::AdditionalContextStore;
use super::auto_compact_window::AutoCompactWindow;
use super::auto_compact_window::AutoCompactWindowIds;
use super::auto_compact_window::AutoCompactWindowSnapshot;
use crate::TurnStartOptions;
use crate::context_manager::ContextManager;
use crate::context_manager::HistoryReplacement;
use crate::session::PreviousTurnSettings;
use crate::session::session::SessionConfiguration;
use crate::session::startup_prewarm::SessionStartupPrewarmHandle;
use crate::session::time_reminder::CurrentTimeReminderState;
<<<<<<< HEAD
use crate::session_startup_prewarm::SessionStartupPrewarmHandle;
use codex_protocol::protocol::AccountTokenUsage;
||||||| 413492cd6c
use crate::session_startup_prewarm::SessionStartupPrewarmHandle;
=======
use codex_history::ResponseItemEnvelope;
use codex_history::TurnAttribution;
use codex_protocol::SessionId;
use codex_protocol::ThreadId;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_protocol::protocol::RateLimitSnapshot;
use codex_protocol::protocol::TokenUsage;
use codex_protocol::protocol::TokenUsageInfo;
use codex_protocol::protocol::TokenUsageRecord;
use codex_protocol::protocol::TurnContextItem;
use tokio_util::sync::CancellationToken;
use tokio_util::task::AbortOnDropHandle;

/// Runtime request effort, inherited from a parent or established by prewarm or sampling.
/// Successful compaction allows a fresh baseline without an override.
#[derive(Clone)]
pub(crate) enum ReasoningEffortPin {
    Unset,
    Compacted,
    Active {
        model: String,
        effort: ReasoningEffort,
    },
}

impl ReasoningEffortPin {
    pub(crate) fn get(&self, model: &str) -> Option<ReasoningEffort> {
        match self {
            Self::Active {
                model: pinned_model,
                effort,
            } if pinned_model == model => Some(effort.clone()),
            Self::Unset | Self::Compacted | Self::Active { .. } => None,
        }
    }

    pub(crate) fn pin(&mut self, model: &str, effort: ReasoningEffort) -> ReasoningEffort {
        if let Some(pinned) = self.get(model) {
            return pinned;
        }
        *self = Self::Active {
            model: model.to_owned(),
            effort: effort.clone(),
        };
        effort
    }
}

/// Persistent, session-scoped state previously stored directly on `Session`.
pub(crate) struct SessionState {
    pub(crate) session_configuration: SessionConfiguration,
    /// Catalog to carry across compaction, independent of an explicit resume override.
    pub(crate) persisted_dynamic_tools: Arc<[DynamicToolSpec]>,
    /// Plugin selection of the last admitted task; settings updates take effect on the next task.
    pub(crate) active_disabled_plugin_ids: Vec<String>,
    /// Persisted origin of the session base instructions, when known.
    pub(crate) base_instructions_provenance: Option<BaseInstructionsProvenance>,
    pub(crate) history: ContextManager,
    /// Cancels work bound to discarded history or a superseded Guardian evidence policy.
    pub(crate) history_reset: CancellationToken,
    pub(crate) latest_rate_limits: Option<RateLimitSnapshot>,
    pub(crate) latest_token_usage_record: Option<TokenUsageRecord>,
    pub(crate) mcp_dependency_prompted: HashSet<String>,
    pub(crate) additional_context: AdditionalContextStore,
    /// Settings used by the latest regular user turn, used for turn-to-turn
    /// model/realtime handling on subsequent regular turns (including full-context
    /// reinjection after resume or `/compact`).
    previous_turn_settings: Option<PreviousTurnSettings>,
    /// Latest task admitted in this runtime, retained across completion and history edits.
    /// Cleared by standalone settings changes to invalidate pending continuation.
    pub(crate) last_started_turn_id: Option<String>,
    /// Latest regular turn, retained across completion, settings changes, and compaction.
    pub(crate) turn_attribution: Option<TurnAttribution>,
    /// Runtime accounting state for the active auto-compaction window.
    auto_compact_window: AutoCompactWindow,
    pub(crate) async_compaction: crate::session::async_compaction::AsyncCompactionState,
    /// Original request effort for the current model while configuration updates remain active.
    pub(crate) reasoning_effort_pin: ReasoningEffortPin,
    /// Set under the state lock before shutdown takes the last warmup handle.
    pub(crate) shutting_down: bool,
    /// Background model warmup scheduled at startup or while resuming an idle thread.
    pub(crate) startup_prewarm: Option<SessionStartupPrewarmHandle>,
    /// Retained after completion so later turns do not repeat speculative captures.
    pub(crate) shell_snapshot_prewarm: Option<AbortOnDropHandle<()>>,
    pub(crate) current_time_reminder: CurrentTimeReminderState,
    pub(crate) active_connector_selection: HashSet<String>,
    pub(crate) pending_session_start_sources: VecDeque<codex_hooks::SessionStartSource>,
<<<<<<< HEAD
    /// D51 (#1001): bound to the policy marker they were granted under.
    granted_permissions: super::GrantedPermissions,
||||||| 413492cd6c
    granted_permissions_by_environment_id: HashMap<String, AdditionalPermissionProfile>,
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    next_turn_is_first: bool,
<<<<<<< HEAD
    /// Whether the latest model request was authorised by the `OPENAI_API_KEY`
    /// fallback; the notice is shown only when this turns on.
    openai_api_key_env_fallback_in_use: bool,
    /// PF-84: provider-reported usage per (provider, named account), kept only
    /// while named accounts are on. Names only, never credential material.
    account_token_usage: Vec<AccountTokenUsage>,
    /// #688: tools already reported as left out by the tool-count limit.
    omitted_tools_reported: HashSet<codex_protocol::ToolName>,
||||||| 413492cd6c
=======
    initial_settings_pending: bool,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

impl SessionState {
    /// Create a new session state mirroring previous `State::default()` semantics.
    #[cfg(test)]
    pub(crate) fn new(session_configuration: SessionConfiguration) -> Self {
        Self::new_with_auto_compact_window_ids(
            session_configuration,
            AutoCompactWindowIds::new_initial(),
            ContextManager::new(),
        )
    }

    pub(crate) fn new_with_auto_compact_window_ids(
        session_configuration: SessionConfiguration,
        auto_compact_window_ids: AutoCompactWindowIds,
        history: ContextManager,
    ) -> Self {
        Self {
            active_disabled_plugin_ids: Vec::new(),
            persisted_dynamic_tools: Arc::clone(&session_configuration.dynamic_tools),
            session_configuration,
            base_instructions_provenance: None,
            history,
            history_reset: CancellationToken::new(),
            latest_rate_limits: None,
            latest_token_usage_record: None,
            mcp_dependency_prompted: HashSet::new(),
            additional_context: AdditionalContextStore::default(),
            previous_turn_settings: None,
            last_started_turn_id: None,
            turn_attribution: None,
            auto_compact_window: AutoCompactWindow::new_with_ids(auto_compact_window_ids),
            async_compaction: crate::session::async_compaction::AsyncCompactionState::default(),
            reasoning_effort_pin: ReasoningEffortPin::Unset,
            shutting_down: false,
            startup_prewarm: None,
            shell_snapshot_prewarm: None,
            current_time_reminder: CurrentTimeReminderState::default(),
            active_connector_selection: HashSet::new(),
            pending_session_start_sources: VecDeque::new(),
<<<<<<< HEAD
            granted_permissions: super::GrantedPermissions::default(),
||||||| 413492cd6c
            granted_permissions_by_environment_id: HashMap::new(),
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            next_turn_is_first: true,
<<<<<<< HEAD
            openai_api_key_env_fallback_in_use: false,
            account_token_usage: Vec::new(),
            omitted_tools_reported: HashSet::new(),
||||||| 413492cd6c
=======
            initial_settings_pending: true,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
    }

<<<<<<< HEAD
    /// Records whether the current request uses the `OPENAI_API_KEY` fallback and
    /// returns true when it newly does (first use, or a switch back to it).
    pub(crate) fn note_openai_api_key_env_fallback(&mut self, in_use: bool) -> bool {
        let newly_in_use = in_use && !self.openai_api_key_env_fallback_in_use;
        self.openai_api_key_env_fallback_in_use = in_use;
        newly_in_use
    }

    /// #688: the tools in `omitted` not reported yet this session.
    pub(crate) fn unreported_omitted_tools(
        &self,
        omitted: &[codex_protocol::ToolName],
    ) -> Vec<codex_protocol::ToolName> {
        omitted
            .iter()
            .filter(|tool| !self.omitted_tools_reported.contains(*tool))
            .cloned()
            .collect()
    }

    /// #688: records `tools` as reported to the person; each left-out tool
    /// is reported once per session.
    pub(crate) fn mark_omitted_tools_reported(
        &mut self,
        tools: impl IntoIterator<Item = codex_protocol::ToolName>,
    ) {
        self.omitted_tools_reported.extend(tools);
    }

    // History helpers
    pub(crate) fn record_items<I>(&mut self, items: I, policy: TruncationPolicy)
    where
        I: IntoIterator,
        I::Item: std::ops::Deref<Target = ResponseItem>,
    {
        self.history.record_items(items, policy);
    }

||||||| 413492cd6c
    // History helpers
    pub(crate) fn record_items<I>(&mut self, items: I, policy: TruncationPolicy)
    where
        I: IntoIterator,
        I::Item: std::ops::Deref<Target = ResponseItem>,
    {
        self.history.record_items(items, policy);
    }

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    pub(crate) fn previous_turn_settings(&self) -> Option<PreviousTurnSettings> {
        self.previous_turn_settings.clone()
    }
    pub(crate) fn set_previous_turn_settings(
        &mut self,
        previous_turn_settings: Option<PreviousTurnSettings>,
    ) {
        self.previous_turn_settings = previous_turn_settings;
    }

    pub(crate) fn set_next_turn_is_first(&mut self, value: bool) {
        self.next_turn_is_first = value;
        self.initial_settings_pending = value;
    }

    pub(crate) fn take_next_turn_is_first(&mut self) -> bool {
        let is_first_turn = self.next_turn_is_first;
        self.next_turn_is_first = false;
        is_first_turn
    }

    pub(crate) fn take_initial_settings_pending(&mut self) -> bool {
        let pending = self.initial_settings_pending;
        self.initial_settings_pending = false;
        pending
    }

    pub(crate) fn clone_history(&self) -> ContextManager {
        self.history.clone()
    }

    #[cfg(test)]
    pub(crate) fn replace_history(
        &mut self,
        items: Vec<ResponseItem>,
        reference_context_item: Option<TurnContextItem>,
    ) {
        self.replace_annotated_history(
            items.into_iter().map(ResponseItemEnvelope::new).collect(),
            reference_context_item,
            HistoryReplacement::Reset,
        );
    }

    pub(crate) fn replace_annotated_history(
        &mut self,
        items: Vec<ResponseItemEnvelope>,
        reference_context_item: Option<TurnContextItem>,
        replacement: HistoryReplacement,
    ) {
        let invalidate_reviews = match replacement {
            HistoryReplacement::Compaction {
                reviewer_compaction_hash,
            } => self
                .history
                .replace_compacted(items, reviewer_compaction_hash.as_deref()),
            HistoryReplacement::Reset => {
                self.turn_attribution = None;
                self.history.replace_annotated(items);
                true
            }
        };
        if invalidate_reviews {
            std::mem::take(&mut self.history_reset).cancel();
        }
        self.history
            .set_reference_context_item(reference_context_item);
        self.auto_compact_window.clear_prefill();
    }

    pub(crate) fn set_token_info(&mut self, info: Option<TokenUsageInfo>) {
        self.history.set_token_info(info);
    }

    pub(crate) fn record_token_usage(
        &mut self,
        thread_id: ThreadId,
        turn_id: &str,
        session_id: SessionId,
        root_turn_id: String,
        response_id: String,
        usage: &TokenUsage,
    ) -> TokenUsageRecord {
        let mut turn_token_usage = self
            .latest_token_usage_record
            .as_ref()
            .filter(|record| record.turn_id == turn_id)
            .map_or_else(TokenUsage::default, |record| {
                record.turn_token_usage.clone()
            });
        turn_token_usage.add_assign(usage);
        let mut thread_token_usage = self
            .latest_token_usage_record
            .as_ref()
            .map_or_else(TokenUsage::default, |record| {
                record.thread_token_usage.clone()
            });
        thread_token_usage.add_assign(usage);
        let record = TokenUsageRecord {
            thread_id,
            turn_id: turn_id.to_string(),
            session_id,
            root_turn_id,
            response_id,
            usage: usage.clone(),
            turn_token_usage,
            thread_token_usage,
        };
        self.latest_token_usage_record = Some(record.clone());
        record
    }

    pub(crate) fn set_reference_context_item(&mut self, item: Option<TurnContextItem>) {
        self.history.set_reference_context_item(item);
    }

    pub(crate) fn reference_context_item(&self) -> Option<TurnContextItem> {
        self.history.reference_context_item()
    }

    pub(crate) fn recovered_turn_start_options(&self, turn_id: &str) -> TurnStartOptions {
        self.turn_attribution
            .as_ref()
            .filter(|attribution| attribution.turn_id == turn_id)
            .map(TurnAttribution::start_options)
            .unwrap_or_else(|| TurnStartOptions {
                // Older rollouts only persisted the root on the model-context record.
                root_turn_id: self
                    .reference_context_item()
                    .filter(|context| context.turn_id.as_deref() == Some(turn_id))
                    .and_then(|context| context.root_turn_id),
                ..Default::default()
            })
    }

    // Token/rate limit helpers
    pub(crate) fn update_token_info_from_usage(
        &mut self,
        usage: &TokenUsage,
        model_context_window: Option<i64>,
        includes_reasoning: bool,
        input_item_count: usize,
    ) {
        self.history.update_token_info(
            usage,
            model_context_window,
            includes_reasoning,
            input_item_count,
        );
    }

    pub(crate) fn ensure_auto_compact_window_server_prefill_from_usage(
        &mut self,
        usage: &TokenUsage,
    ) {
        self.auto_compact_window
            .ensure_server_observed_prefill_from_usage(usage);
    }

    pub(crate) fn set_auto_compact_window_estimated_prefill(&mut self, tokens: i64) {
        self.auto_compact_window.set_estimated_prefill(tokens);
    }

    pub(crate) fn auto_compact_window_snapshot(&self) -> AutoCompactWindowSnapshot {
        self.auto_compact_window.snapshot()
    }

    pub(crate) fn claim_token_budget_reminder(&mut self) -> bool {
        self.auto_compact_window.claim_token_budget_reminder()
    }

    pub(crate) fn claim_auto_compact_fallback(&mut self) -> bool {
        self.auto_compact_window.claim_auto_compact_fallback()
    }

    pub(crate) fn auto_compact_window_number(&self) -> u64 {
        self.auto_compact_window.window_number()
    }

    pub(crate) fn auto_compact_window_ids(&self) -> AutoCompactWindowIds {
        self.auto_compact_window.ids()
    }

    pub(crate) fn restore_auto_compact_window(
        &mut self,
        window_number: u64,
        ids: AutoCompactWindowIds,
    ) {
        self.auto_compact_window.restore(window_number, ids);
    }

    pub(crate) fn advance_auto_compact_window(&mut self) -> (u64, AutoCompactWindowIds) {
        self.auto_compact_window.advance()
    }

    pub(crate) fn request_new_context_window(&mut self) {
        self.auto_compact_window.request_new_context_window();
    }

    pub(crate) fn take_new_context_window_request(&mut self) -> bool {
        self.auto_compact_window.take_new_context_window_request()
    }

    pub(crate) fn start_new_context_window(&mut self) -> (u64, AutoCompactWindowIds) {
        let window = self.auto_compact_window.advance();
        self.auto_compact_window.clear_prefill();
        window
    }

    pub(crate) fn token_info(&self) -> Option<TokenUsageInfo> {
        self.history.token_info()
    }

    pub(crate) fn set_rate_limits(&mut self, snapshot: RateLimitSnapshot) {
        self.latest_rate_limits = Some(merge_rate_limit_fields(
            self.latest_rate_limits.as_ref(),
            snapshot,
        ));
    }

    pub(crate) fn token_info_and_rate_limits(
        &self,
    ) -> (Option<TokenUsageInfo>, Option<RateLimitSnapshot>) {
        (self.token_info(), self.latest_rate_limits.clone())
    }

    /// PF-84: adds one request's provider-reported usage to the account that
    /// served it.
    pub(crate) fn record_account_token_usage(
        &mut self,
        provider_id: &str,
        account: &str,
        usage: &TokenUsage,
    ) {
        AccountTokenUsage::accumulate(&mut self.account_token_usage, provider_id, account, usage);
    }

    /// PF-84: this thread's usage per account; `None` until a named-accounts
    /// session records some, so other sessions' events keep their bytes.
    pub(crate) fn account_token_usage(&self) -> Option<Vec<AccountTokenUsage>> {
        (!self.account_token_usage.is_empty()).then(|| self.account_token_usage.clone())
    }

    pub(crate) fn set_account_token_usage(&mut self, usage: Vec<AccountTokenUsage>) {
        self.account_token_usage = usage;
    }

    pub(crate) fn set_token_usage_full(&mut self, context_window: i64) {
        self.history.set_token_usage_full(context_window);
    }

    pub(crate) fn get_total_token_usage(&self) -> i64 {
        self.history.get_total_token_usage()
    }

    pub(crate) fn record_mcp_dependency_prompted<I>(&mut self, names: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.mcp_dependency_prompted.extend(names);
    }

    pub(crate) fn mcp_dependency_prompted(&self) -> HashSet<String> {
        self.mcp_dependency_prompted.clone()
    }

    pub(crate) fn set_session_startup_prewarm(
        &mut self,
        startup_prewarm: SessionStartupPrewarmHandle,
    ) {
        self.startup_prewarm = Some(startup_prewarm);
    }

    pub(crate) fn take_session_startup_prewarm(&mut self) -> Option<SessionStartupPrewarmHandle> {
        self.startup_prewarm.take()
    }

    // Adds connector IDs to the active set and returns the merged selection.
    pub(crate) fn merge_connector_selection<I>(&mut self, connector_ids: I) -> HashSet<String>
    where
        I: IntoIterator<Item = String>,
    {
        self.active_connector_selection.extend(connector_ids);
        self.active_connector_selection.clone()
    }

    // Returns the current connector selection tracked on session state.
    pub(crate) fn get_connector_selection(&self) -> HashSet<String> {
        self.active_connector_selection.clone()
    }

    // Removes all currently tracked connector selections.
    pub(crate) fn clear_connector_selection(&mut self) {
        self.active_connector_selection.clear();
    }

    pub(crate) fn queue_pending_session_start_source(
        &mut self,
        value: codex_hooks::SessionStartSource,
    ) {
        self.pending_session_start_sources.push_back(value);
    }

    pub(crate) fn take_pending_session_start_source(
        &mut self,
    ) -> Option<codex_hooks::SessionStartSource> {
        self.pending_session_start_sources.pop_front()
    }
<<<<<<< HEAD

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
||||||| 413492cd6c

    pub(crate) fn record_granted_permissions(
        &mut self,
        environment_id: &str,
        permissions: AdditionalPermissionProfile,
    ) {
        let granted_permissions = merge_permission_profiles(
            self.granted_permissions_by_environment_id
                .get(environment_id),
            Some(&permissions),
        );
        if let Some(granted_permissions) = granted_permissions {
            self.granted_permissions_by_environment_id
                .insert(environment_id.to_string(), granted_permissions);
        }
    }

    pub(crate) fn granted_permissions(
        &self,
        environment_id: &str,
    ) -> Option<AdditionalPermissionProfile> {
        self.granted_permissions_by_environment_id
            .get(environment_id)
            .cloned()
    }
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

// Sometimes new snapshots don't include credits or plan information.
// Preserve those from the previous snapshot when missing. For `limit_id`, treat
// missing values as the default `"codex"` bucket.
fn merge_rate_limit_fields(
    previous: Option<&RateLimitSnapshot>,
    mut snapshot: RateLimitSnapshot,
) -> RateLimitSnapshot {
    if snapshot.limit_id.is_none() {
        snapshot.limit_id = Some("codex".to_string());
    }
    if snapshot.credits.is_none() {
        snapshot.credits = previous.and_then(|prior| prior.credits.clone());
    }
    if snapshot.individual_limit.is_none() {
        snapshot.individual_limit = previous.and_then(|prior| prior.individual_limit.clone());
    }
    if snapshot.spend_control_reached.is_none() {
        snapshot.spend_control_reached = previous.and_then(|prior| prior.spend_control_reached);
    }
    if snapshot.plan_type.is_none() {
        snapshot.plan_type = previous.and_then(|prior| prior.plan_type);
    }
    snapshot
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
