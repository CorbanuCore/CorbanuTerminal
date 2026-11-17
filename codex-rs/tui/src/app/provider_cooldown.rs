use super::App;
use super::AppServerSession;
use crate::legacy_core::config::Config;

impl App {
    pub(super) async fn clear_provider_cooldown(
        &mut self,
        app_server: &AppServerSession,
        config: &Config,
        model: &str,
    ) {
        if !app_server.uses_embedded_app_server() {
            self.chat_widget.add_error_message(
                "Clear the cooldown on the app-server host with `corbanu -m MODEL model clear-cooldown`, using the same provider and profile.".to_string(),
            );
            return;
        }
        let Some(state_db) = self.state_db.as_ref() else {
            self.chat_widget.add_error_message(
                "Provider cooldown state is unavailable in this session.".to_string(),
            );
            return;
        };
        let auth = codex_login::AuthManager::shared_from_config(
            config, /*enable_codex_api_key_env*/ false,
        )
        .await;
        match crate::legacy_core::clear_provider_cooldown(state_db, config, model, auth.auth_mode()).await {
            Ok(()) => self.chat_widget.add_info_message(
                format!(
                    "Cleared the local cooldown for {}/{}. The next request will probe the provider.",
                    config.model_provider_id, model
                ),
                /*hint*/ None,
            ),
            Err(err) => self
                .chat_widget
                .add_error_message(format!("Could not clear the provider cooldown: {err}")),
        }
    }
}
