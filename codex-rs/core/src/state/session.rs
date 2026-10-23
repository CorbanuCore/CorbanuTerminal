//! Session-wide mutable state.

use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::models::ResponseItem;
use std::collections::HashSet;
use std::collections::VecDeque;

use super::AdditionalContextStore;
use super::auto_compact_window::AutoCompactWindow;
use super::auto_compact_window::AutoCompactWindowIds;
use super::auto_compact_window::AutoCompactWindowSnapshot;
use crate::context_manager::ContextManager;
use crate::session::PreviousTurnSettings;
use crate::session::session::SessionConfiguration;
use crate::session::time_reminder::CurrentTimeReminderState;
use crate::session_startup_prewarm::SessionStartupPrewarmHandle;
use codex_protocol::protocol::AccountTokenUsage;
use codex_protocol::protocol::RateLimitSnapshot;
use codex_protocol::protocol::TokenUsage;
use codex_protocol::protocol::TokenUsageInfo;
use codex_protocol::protocol::TurnContextItem;
use codex_utils_output_truncation::TruncationPolicy;

/// Persistent, session-scoped state previously stored directly on `Session`.
pub(crate) struct SessionState {
    pub(crate) session_configuration: SessionConfiguration,
    pub(crate) history: ContextManager,
    pub(crate) latest_rate_limits: Option<RateLimitSnapshot>,
    pub(crate) server_reasoning_included: bool,
    pub(crate) mcp_dependency_prompted: HashSet<String>,
    pub(crate) additional_context: AdditionalContextStore,
    /// Settings used by the latest regular user turn, used for turn-to-turn
    /// model/realtime handling on subsequent regular turns (including full-context
    /// reinjection after resume or `/compact`).
    previous_turn_settings: Option<PreviousTurnSettings>,
    /// Runtime accounting state for the active auto-compaction window.
    auto_compact_window: AutoCompactWindow,
    /// Startup prewarmed session prepared during session initialization.
    pub(crate) startup_prewarm: Option<SessionStartupPrewarmHandle>,
    pub(crate) current_time_reminder: CurrentTimeReminderState,
    pub(crate) active_connector_selection: HashSet<String>,
    pub(crate) pending_session_start_sources: VecDeque<codex_hooks::SessionStartSource>,
    /// D51 (#1001): bound to the policy marker they were granted under.
    granted_permissions: super::GrantedPermissions,
    next_turn_is_first: bool,
    /// Whether the latest model request was authorised by the `OPENAI_API_KEY`
    /// fallback; the notice is shown only when this turns on.
    openai_api_key_env_fallback_in_use: bool,
    /// PF-84: provider-reported usage per (provider, named account), kept only
    /// while named accounts are on. Names only, never credential material.
    account_token_usage: Vec<AccountTokenUsage>,
    /// #688: tools already reported as left out by the tool-count limit.
    omitted_tools_reported: HashSet<codex_protocol::ToolName>,
}

impl SessionState {
    /// Create a new session state mirroring previous `State::default()` semantics.
    #[cfg(test)]
    pub(crate) fn new(session_configuration: SessionConfiguration) -> Self {
        Self::new_with_auto_compact_window_ids(
            session_configuration,
            AutoCompactWindowIds::new_initial(),
        )
    }

    pub(crate) fn new_with_auto_compact_window_ids(
        session_configuration: SessionConfiguration,
        auto_compact_window_ids: AutoCompactWindowIds,
    ) -> Self {
        let history = ContextManager::new();
        Self {
            session_configuration,
            history,
            latest_rate_limits: None,
            server_reasoning_included: false,
            mcp_dependency_prompted: HashSet::new(),
            additional_context: AdditionalContextStore::default(),
            previous_turn_settings: None,
            auto_compact_window: AutoCompactWindow::new_with_ids(auto_compact_window_ids),
            startup_prewarm: None,
            current_time_reminder: CurrentTimeReminderState::default(),
            active_connector_selection: HashSet::new(),
            pending_session_start_sources: VecDeque::new(),
            granted_permissions: super::GrantedPermissions::default(),
            next_turn_is_first: true,
            openai_api_key_env_fallback_in_use: false,
            account_token_usage: Vec::new(),
            omitted_tools_reported: HashSet::new(),
        }
    }

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
    }

    pub(crate) fn take_next_turn_is_first(&mut self) -> bool {
        let is_first_turn = self.next_turn_is_first;
        self.next_turn_is_first = false;
        is_first_turn
    }

    pub(crate) fn clone_history(&self) -> ContextManager {
        self.history.clone()
    }

    pub(crate) fn replace_history(
        &mut self,
        items: Vec<ResponseItem>,
        reference_context_item: Option<TurnContextItem>,
    ) {
        self.history.replace(items);
        self.history
            .set_reference_context_item(reference_context_item);
        self.auto_compact_window.clear_prefill();
    }

    pub(crate) fn set_token_info(&mut self, info: Option<TokenUsageInfo>) {
        self.history.set_token_info(info);
    }

    pub(crate) fn set_reference_context_item(&mut self, item: Option<TurnContextItem>) {
        self.history.set_reference_context_item(item);
    }

    pub(crate) fn reference_context_item(&self) -> Option<TurnContextItem> {
        self.history.reference_context_item()
    }

    // Token/rate limit helpers
    pub(crate) fn update_token_info_from_usage(
        &mut self,
        usage: &TokenUsage,
        model_context_window: Option<i64>,
    ) {
        self.history.update_token_info(usage, model_context_window);
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

    pub(crate) fn get_total_token_usage(&self, server_reasoning_included: bool) -> i64 {
        self.history
            .get_total_token_usage(server_reasoning_included)
    }

    pub(crate) fn set_server_reasoning_included(&mut self, included: bool) {
        self.server_reasoning_included = included;
    }

    pub(crate) fn server_reasoning_included(&self) -> bool {
        self.server_reasoning_included
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
