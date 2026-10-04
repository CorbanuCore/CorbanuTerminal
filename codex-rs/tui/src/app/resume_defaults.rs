//! Remember an acknowledged interactive resume without persisting invocation overrides.

use super::*;
use crate::app_server_session::ResumeModelSettings;
use crate::session_state::ThreadSessionState;

pub(super) async fn remember_resumed_model(
    app_server: &AppServerSession,
    config: &Config,
    settings: ResumeModelSettings,
    session: &ThreadSessionState,
) -> Result<()> {
    // Explicit CLI/profile selection remains invocation-scoped. Unknown remote providers must
    // not become broken local defaults. The server owns the active profile's write destination.
    if settings != ResumeModelSettings::RestoreFromThread
        || crate::app_server_session::has_explicit_resume_service_tier(config)
        || app_server.thread_params_mode() != crate::app_server_session::ThreadParamsMode::Embedded
        || !config
            .model_providers
            .contains_key(&session.model_provider_id)
    {
        return Ok(());
    }
    let mut edits = crate::config_update::build_model_selection_edits(
        &session.model,
        Some(&session.model_provider_id),
        session.reasoning_effort.as_ref(),
    );
    edits.extend(crate::config_update::build_service_tier_selection_edits(
        Some(
            session
                .service_tier
                .as_deref()
                .unwrap_or(codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE),
        ),
    ));
    crate::config_update::write_config_batch(app_server.request_handle(), edits).await?;
    Ok(())
}

#[cfg(test)]
#[path = "resume_defaults_tests.rs"]
mod tests;
