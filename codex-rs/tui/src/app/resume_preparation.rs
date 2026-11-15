//! Prepare saved-thread attachment before replacing the active conversation UI.
//!
//! Stored-backend preparation leaves active settings unchanged; active-backend resume retains its
//! eager preference refresh. Neither detaches the active thread or replaces its composer. The
//! prepared result retains the connection identity and any provider error for read-only history.

use super::*;

/// Settings retained for a backend that is not currently displayed.
#[derive(Clone)]
pub(super) struct BackendSettings {
    pub(super) config: Config,
    pub(super) local_settings: crate::local_settings::LocalSettings,
    pub(super) target: AppServerTarget,
}

/// The settings source for a resume attempt.
///
/// The active source may refresh the displayed conversation's settings eagerly. A stored backend
/// is refreshed without adopting its settings until attachment succeeds.
#[derive(Clone, Copy)]
pub(super) enum ResumeSource<'a> {
    Active,
    Backend(&'a BackendSettings),
}

pub(super) struct PreparedResume {
    pub(super) backend_id: crate::backend_id::BackendId,
    pub(super) config: Config,
    pub(super) local_settings: crate::local_settings::LocalSettings,
    pub(super) thread: AppServerStartedThread,
    pub(super) read_only: bool,
    pub(super) history_notice: Option<&'static str>,
    pub(super) provider_error: Option<codex_app_server_protocol::ModelProviderError>,
    pub(super) model_settings: crate::app_server_session::ResumeModelSettings,
    /// PF-84 (#545): the named account the thread recorded, and the session's previous
    /// inherited account to restore if attachment fails.
    pub(super) recorded_account: Option<crate::session_resume::RecordedProviderAccount>,
    pub(super) previous_provider_account: Option<String>,
}

impl App {
    pub(super) fn backend_settings(&self) -> BackendSettings {
        BackendSettings {
            config: self.config.clone(),
            local_settings: self.local_settings.clone(),
            target: self.app_server_target.clone(),
        }
    }

    pub(super) async fn prepare_target_session(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        target_session: &SessionTarget,
        source: ResumeSource<'_>,
    ) -> std::result::Result<PreparedResume, AppRunControl> {
        if let Some(message) = crate::security::preflight::resume_refusal(&target_session.thread_id)
        {
            self.chat_widget.add_error_message(message);
            tui.frame_requester().schedule_frame();
            return Err(AppRunControl::Continue);
        }
        // PF-84 (#545): the thread resumes on the account it recorded, so the config this
        // session keeps for it names that account too.
        let recorded_account = self.resume_target_recorded_account(target_session).await;
        let previous_provider_account = std::mem::replace(
            &mut self.harness_overrides.inherited_provider_account,
            recorded_account
                .as_ref()
                .map(crate::session_resume::RecordedProviderAccount::selection),
        );
        let prepared = self
            .prepare_target_session_with_account(
                tui,
                app_server,
                target_session,
                source,
                recorded_account,
                previous_provider_account.clone(),
            )
            .await;
        if prepared.is_err() {
            self.harness_overrides.inherited_provider_account = previous_provider_account;
        }
        prepared
    }

    async fn prepare_target_session_with_account(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        target_session: &SessionTarget,
        source: ResumeSource<'_>,
        recorded_account: Option<crate::session_resume::RecordedProviderAccount>,
        previous_provider_account: Option<String>,
    ) -> std::result::Result<PreparedResume, AppRunControl> {
        let (mut resume_config, local_settings) = self
            .resume_config_for_target(tui, app_server, target_session, source)
            .await?;
        if let Some(recorded) = recorded_account.as_ref()
            && let Some(message) = crate::resume_launch::resume_account_error(
                &resume_config,
                recorded,
                codex_model_provider::AccountRecovery::Providers,
                crate::resume_launch::ThreadOpening::Resume,
            )
        {
            self.chat_widget.add_error_message(message);
            tui.frame_requester().schedule_frame();
            return Err(AppRunControl::Continue);
        }
        if let Err(error) = self.apply_runtime_policy_overrides(
            &mut resume_config,
            RuntimePolicyOverrideScope::ExplicitOnly,
        ) {
            self.add_session_picker_error(format!("{error:#}"));
            return Err(AppRunControl::Continue);
        }
        let permission_overrides = self.resume_permission_overrides(&resume_config);
        let model_settings = match source {
            ResumeSource::Active => self.resume_model_settings(),
            ResumeSource::Backend(_) => {
                super::config_persistence::resume_model_settings_for_overrides(
                    &resume_config,
                    &self.harness_overrides,
                )
            }
        };

        if let Some(history_mode) = target_session.history_mode {
            app_server.remember_thread_history_mode(target_session.thread_id, history_mode);
        }
        let resumed = app_server
            .resume_thread_with_permission_overrides(
                &local_settings,
                resume_config.clone(),
                target_session.thread_id,
                model_settings,
                permission_overrides,
            )
            .await;
        let mut history_notice = None;
        let mut provider_error = None;
        let (resumed, read_only) = match resumed {
            Ok(resumed) => (resumed, false),
            Err(err)
                if crate::app_server_session::is_active_writer_error(&err)
                    || crate::app_server_session::model_provider_error(&err).is_some() =>
            {
                provider_error = crate::app_server_session::model_provider_error(&err);
                match app_server
                    .read_thread_for_viewing(
                        &resume_config,
                        &local_settings,
                        target_session.thread_id,
                    )
                    .await
                {
                    Ok((thread, notice)) => {
                        history_notice = notice;
                        (thread, true)
                    }
                    Err(read_err) => {
                        self.add_session_picker_error(format!(
                            "Failed to load chat history: {read_err}"
                        ));
                        return Err(AppRunControl::Continue);
                    }
                }
            }
            Err(err) => {
                let path_display = target_session.display_label();
                self.add_session_picker_error(format!(
                    "Failed to resume session from {path_display}: {err}"
                ));
                return Err(AppRunControl::Continue);
            }
        };
        Ok(PreparedResume {
            backend_id: app_server.backend_id(),
            config: resume_config,
            local_settings,
            thread: resumed,
            read_only,
            history_notice,
            provider_error,
            model_settings,
            recorded_account,
            previous_provider_account,
        })
    }
}
