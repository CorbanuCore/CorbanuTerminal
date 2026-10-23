//! Thread settings sync between TUI-local state and app-server thread state.

use super::App;
use crate::app_command::AppCommand;
use crate::app_server_session::AppServerSession;
use crate::session_state::ThreadSessionState;
use codex_app_server_protocol::ApprovalsReviewer as AppServerApprovalsReviewer;
use codex_app_server_protocol::ThreadSettings;
use codex_app_server_protocol::ThreadSettingsUpdateParams;
use codex_protocol::ThreadId;
use codex_protocol::config_types::ModeKind;
use codex_protocol::models::ActivePermissionProfile;
use codex_protocol::models::PermissionProfile;

impl App {
    pub(super) async fn sync_active_thread_model_setting(
        &mut self,
        app_server: &mut AppServerSession,
        model: String,
    ) {
        let Some(params) = self.active_thread_model_setting_update_params(model) else {
            return;
        };
        self.send_thread_settings_update(app_server, params).await;
    }

    pub(super) fn active_thread_model_setting_update_params(
        &self,
        model: String,
    ) -> Option<ThreadSettingsUpdateParams> {
        let thread_id = self.active_thread_id?;
        Some(ThreadSettingsUpdateParams {
            thread_id: thread_id.to_string(),
            model: Some(model),
            collaboration_mode: Some(self.chat_widget.effective_collaboration_mode()),
            ..ThreadSettingsUpdateParams::default()
        })
    }

    pub(super) async fn sync_active_thread_model_selection(
        &mut self,
        app_server: &mut AppServerSession,
        model: String,
        model_provider: Option<String>,
        confirm: bool,
    ) -> SettingsUpdateOutcome {
        let Some(thread_id) = self.active_thread_id else {
            return SettingsUpdateOutcome::NotSent;
        };
        let params = ThreadSettingsUpdateParams {
            thread_id: thread_id.to_string(),
            confirm,
            model: Some(model),
            model_provider,
            collaboration_mode: Some(self.chat_widget.effective_collaboration_mode()),
            ..ThreadSettingsUpdateParams::default()
        };
        self.send_thread_settings_update(app_server, params).await
    }

    pub(super) async fn sync_active_thread_reasoning_setting(
        &mut self,
        app_server: &mut AppServerSession,
        effort: Option<codex_protocol::openai_models::ReasoningEffort>,
    ) {
        let Some(params) = self.active_thread_reasoning_setting_update_params(effort) else {
            return;
        };
        self.send_thread_settings_update(app_server, params).await;
    }

    pub(super) fn active_thread_reasoning_setting_update_params(
        &self,
        effort: Option<codex_protocol::openai_models::ReasoningEffort>,
    ) -> Option<ThreadSettingsUpdateParams> {
        let thread_id = self.active_thread_id?;
        Some(ThreadSettingsUpdateParams {
            thread_id: thread_id.to_string(),
            effort,
            collaboration_mode: Some(self.chat_widget.current_collaboration_mode().clone()),
            ..ThreadSettingsUpdateParams::default()
        })
    }

    pub(super) async fn sync_active_thread_plan_mode_reasoning_setting(
        &mut self,
        app_server: &mut AppServerSession,
    ) {
        let Some(thread_id) = self.active_thread_id else {
            return;
        };
        let params = ThreadSettingsUpdateParams {
            thread_id: thread_id.to_string(),
            collaboration_mode: Some(self.chat_widget.effective_collaboration_mode()),
            ..ThreadSettingsUpdateParams::default()
        };
        self.send_thread_settings_update(app_server, params).await;
    }

    pub(super) async fn sync_active_thread_personality_setting(
        &mut self,
        app_server: &mut AppServerSession,
        personality: codex_protocol::config_types::Personality,
    ) {
        let Some(thread_id) = self.active_thread_id else {
            return;
        };
        let params = ThreadSettingsUpdateParams {
            thread_id: thread_id.to_string(),
            personality: Some(personality),
            ..ThreadSettingsUpdateParams::default()
        };
        self.send_thread_settings_update(app_server, params).await;
    }

    pub(super) async fn sync_override_turn_context_settings(
        &mut self,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
        op: &AppCommand,
    ) {
        let AppCommand::OverrideTurnContext {
            cwd,
            approval_policy,
            approvals_reviewer,
            permission_profile: _,
            active_permission_profile,
            windows_sandbox_level: _,
            model,
            effort,
            summary,
            service_tier,
            collaboration_mode,
            personality,
        } = op
        else {
            return;
        };

        let params = ThreadSettingsUpdateParams {
            thread_id: thread_id.to_string(),
            cwd: cwd.clone(),
            approval_policy: *approval_policy,
            approvals_reviewer: approvals_reviewer.map(AppServerApprovalsReviewer::from),
            permissions: active_permission_profile
                .as_ref()
                .map(|profile| profile.id.clone()),
            model: model.clone(),
            effort: effort.clone().unwrap_or_default(),
            summary: *summary,
            service_tier: service_tier.clone(),
            collaboration_mode: collaboration_mode.clone(),
            personality: *personality,
            ..ThreadSettingsUpdateParams::default()
        };
        self.send_thread_settings_update(app_server, params).await;
    }

    pub(super) async fn apply_thread_settings_to_cached_session(
        &mut self,
        thread_id: ThreadId,
        settings: &ThreadSettings,
    ) {
        if self.primary_thread_id == Some(thread_id)
            && let Some(session) = self.primary_session_configured.as_mut()
        {
            apply_thread_settings_to_session(session, settings);
        }

        if let Some(channel) = self.thread_event_channels.get(&thread_id) {
            let mut store = channel.store.lock().await;
            if let Some(session) = store.session.as_mut() {
                apply_thread_settings_to_session(session, settings);
            }
        }
    }

    /// Whether core applied the update before replying (`Confirmed`, only
    /// for `params.confirm` answered `Confirmed { Applied }`), accepted it
    /// without that (`Accepted`: unconfirmed, uncertain, or a server without
    /// the method), it failed, or nothing was sent (`NotSent`).
    pub(super) async fn send_thread_settings_update(
        &mut self,
        app_server: &mut AppServerSession,
        params: ThreadSettingsUpdateParams,
    ) -> SettingsUpdateOutcome {
        if !thread_settings_update_has_changes(&params) {
            return SettingsUpdateOutcome::NotSent;
        }
        let mut params = params;
        let mut result = app_server.thread_settings_update(params.clone()).await;
        match &result {
            Ok(true) => return SettingsUpdateOutcome::Confirmed,
            Err(err) if params.confirm => {
                // PF-84: confirmation needs a live listener; the update itself
                // must still apply, as it did before it was confirmed.
                tracing::warn!(
                    "confirmed thread settings update failed, sending it unconfirmed: {err}"
                );
                params.confirm = false;
                result = app_server.thread_settings_update(params).await;
            }
            _ => {}
        }
        match result {
            Ok(_) => SettingsUpdateOutcome::Accepted,
            Err(err) => {
                tracing::warn!("failed to update app-server thread settings from TUI: {err}");
                self.chat_widget
                    .add_error_message(format!("Failed to update thread settings: {err}"));
                SettingsUpdateOutcome::Failed
            }
        }
    }
}

/// How a thread settings update went (PF-84: an account switch refetches the
/// model list only once core confirmed it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingsUpdateOutcome {
    /// Nothing changed, or there is no active thread.
    NotSent,
    /// Core applied it before the reply.
    Confirmed,
    /// Accepted for core to apply later (not confirmed).
    Accepted,
    Failed,
}

/// #437: [`ThreadSettings`] carries only the legacy sandbox projection of a
/// thread's permissions, which drops denied reads (the protected paths of the
/// Aggressive profile). Rebuilding the profile from it after every turn made
/// `/security` claim that approved commands leave the sandbox and that the
/// vault is readable. The profile already known is kept while its projection
/// and active profile match the update.
/// `materialized` is `current` with the runtime workspace roots applied, the
/// form the app server projects.
pub(crate) fn permission_profile_for_thread_settings(
    current: &PermissionProfile,
    materialized: Option<&PermissionProfile>,
    current_active: Option<&ActivePermissionProfile>,
    settings: &ThreadSettings,
) -> PermissionProfile {
    let sandbox_policy = settings.sandbox_policy.to_core();
    let active = settings.active_permission_profile.clone().map(Into::into);
    let unchanged = |profile: &PermissionProfile| {
        codex_sandboxing::compatibility_sandbox_policy_for_permission_profile(
            profile,
            settings.cwd.as_path(),
        ) == sandbox_policy
    };
    if current_active == active.as_ref()
        && (unchanged(current) || materialized.is_some_and(unchanged))
    {
        return current.clone();
    }
    PermissionProfile::from_legacy_sandbox_policy_for_cwd(&sandbox_policy, settings.cwd.as_path())
}

fn apply_thread_settings_to_session(session: &mut ThreadSessionState, settings: &ThreadSettings) {
    if settings.collaboration_mode.mode == ModeKind::Default {
        session.model = settings.model.clone();
        session.reasoning_effort = settings.effort.clone();
    }
    session.model_provider_id = settings.model_provider.clone();
    session.service_tier = settings.service_tier.clone();
    session.approval_policy = settings.approval_policy;
    session.approvals_reviewer = settings.approvals_reviewer.to_core();
    session.permission_profile = permission_profile_for_thread_settings(
        &session.permission_profile,
        /*materialized*/ None,
        session.active_permission_profile.as_ref(),
        settings,
    );
    session.active_permission_profile = settings.active_permission_profile.clone().map(Into::into);
    session.set_cwd_retargeting_implicit_runtime_workspace_root(settings.cwd.clone());
    session.personality = settings.personality;
    let mut collaboration_mode = settings.collaboration_mode.clone();
    collaboration_mode
        .settings
        .model
        .clone_from(&settings.model);
    collaboration_mode.settings.reasoning_effort = settings.effort.clone();
    session.collaboration_mode = Some(Box::new(collaboration_mode));
}

fn thread_settings_update_has_changes(params: &ThreadSettingsUpdateParams) -> bool {
    params.cwd.is_some()
        || params.approval_policy.is_some()
        || params.approvals_reviewer.is_some()
        || params.sandbox_policy.is_some()
        || params.permissions.is_some()
        || params.model.is_some()
        || params.model_provider.is_some()
        || params.service_tier.is_some()
        || params.effort.is_some()
        || params.summary.is_some()
        || params.collaboration_mode.is_some()
        || params.personality.is_some()
}
