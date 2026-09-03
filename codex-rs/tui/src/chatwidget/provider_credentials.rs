//! Provider API-key picker and masked entry flow.

use super::*;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::ViewCompletion;
use crate::chatwidget::claude_code_login::ClaudeCodePlanStatus;
pub(crate) use crate::chatwidget::pfterminal_plan_status::PfTerminalPlanStatus;
use crate::status::StatusAccountDisplay;
use codex_model_provider_info::AMBIENT_API_KEY_ENV_VAR;
use codex_model_provider_info::AMBIENT_PROVIDER_ID;
use codex_model_provider_info::ANTHROPIC_API_KEY_ENV_VAR;
use codex_model_provider_info::ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::BASETEN_API_KEY_ENV_VAR;
use codex_model_provider_info::BASETEN_PROVIDER_ID;
use codex_model_provider_info::DEEPSEEK_API_KEY_ENV_VAR;
use codex_model_provider_info::DEEPSEEK_PROVIDER_ID;
use codex_model_provider_info::KIMI_CODE_API_KEY_ENV_VAR;
use codex_model_provider_info::KIMI_CODE_PROVIDER_ID;
use codex_model_provider_info::META_API_KEY_ENV_VAR;
use codex_model_provider_info::META_PROVIDER_ID;
use codex_model_provider_info::OPENROUTER_API_KEY_ENV_VAR;
use codex_model_provider_info::OPENROUTER_PROVIDER_ID;
use codex_model_provider_info::VERCEL_API_KEY_ENV_VAR;
use codex_model_provider_info::VERCEL_PROVIDER_ID;
use codex_model_provider_info::ZAI_API_KEY_ENV_VAR;
use codex_model_provider_info::ZAI_PROVIDER_ID;
use std::path::Path;
use std::time::Duration;

const PROVIDER_CREDENTIALS_VIEW_ID: &str = "provider-credentials";
const CODEX_ACCOUNT_DEVICE_LOGIN_VIEW_ID: &str = "codex-account-device-login";
const PROVIDER_API_KEY_SAVE_VIEW_ID: &str = "provider-api-key-save";
const PROVIDER_STATUS_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderApiKeyStatus {
    Checking,
    AvailableFromEnvironment,
    Stored,
    NotConfigured,
    Unavailable,
}

#[derive(Debug, Clone, Copy)]
enum ProviderCredentialOption {
    CodexAccount,
    ClaudeCodePlan,
    PfTerminalPlan,
    ProviderApiKey {
        provider_id: &'static str,
        provider_name: &'static str,
        env_key: &'static str,
    },
}

const PROVIDER_CREDENTIAL_OPTIONS: &[ProviderCredentialOption] = &[
    ProviderCredentialOption::CodexAccount,
    ProviderCredentialOption::ClaudeCodePlan,
    ProviderCredentialOption::PfTerminalPlan,
    ProviderCredentialOption::ProviderApiKey {
        provider_id: ANTHROPIC_PROVIDER_ID,
        provider_name: "Anthropic",
        env_key: ANTHROPIC_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: AMBIENT_PROVIDER_ID,
        provider_name: "Ambient",
        env_key: AMBIENT_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: KIMI_CODE_PROVIDER_ID,
        provider_name: "Kimi Code",
        env_key: KIMI_CODE_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: ZAI_PROVIDER_ID,
        provider_name: "Z.AI",
        env_key: ZAI_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: DEEPSEEK_PROVIDER_ID,
        provider_name: "DeepSeek",
        env_key: DEEPSEEK_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: OPENROUTER_PROVIDER_ID,
        provider_name: "OpenRouter",
        env_key: OPENROUTER_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: META_PROVIDER_ID,
        provider_name: "Meta",
        env_key: META_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: BASETEN_PROVIDER_ID,
        provider_name: "Baseten",
        env_key: BASETEN_API_KEY_ENV_VAR,
    },
    ProviderCredentialOption::ProviderApiKey {
        provider_id: VERCEL_PROVIDER_ID,
        provider_name: "Vercel",
        env_key: VERCEL_API_KEY_ENV_VAR,
    },
];

impl ChatWidget {
    pub(crate) fn open_provider_credentials_menu(&mut self) {
        let params = self.provider_credentials_params(
            &ClaudeCodePlanStatus::Checking,
            &PfTerminalPlanStatus::Checking,
            &[],
        );
        self.show_selection_view(params);
        self.refresh_provider_credentials_status_in_background();
    }

    pub(crate) fn refresh_provider_credentials_status(
        &mut self,
        claude_status: ClaudeCodePlanStatus,
        pfterminal_plan_status: PfTerminalPlanStatus,
        api_key_statuses: Vec<(String, ProviderApiKeyStatus)>,
    ) {
        let selected_index = self
            .bottom_pane
            .selected_index_for_active_view(PROVIDER_CREDENTIALS_VIEW_ID);
        let mut params = self.provider_credentials_params(
            &claude_status,
            &pfterminal_plan_status,
            &api_key_statuses,
        );
        params.initial_selected_idx = selected_index;
        self.bottom_pane
            .replace_selection_view_if_present(PROVIDER_CREDENTIALS_VIEW_ID, params);
    }

    fn refresh_provider_credentials_status_in_background(&self) {
        let codex_home = self.config.codex_home.clone();
        let plan_home = codex_home.clone();
        let auth_store_mode = self.config.cli_auth_credentials_store_mode;
        let keyring_backend_kind = self.config.auth_keyring_backend_kind();
        let app_event_tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let claude_status = crate::chatwidget::claude_code_login::current_status_with_timeout(
                plan_home.as_path(),
                PROVIDER_STATUS_TIMEOUT,
            );
            let api_key_statuses = tokio::task::spawn_blocking(move || {
                provider_api_key_statuses(codex_home.as_path())
            });
            let pfterminal_plan_status = super::pfterminal_plan_status::load(
                plan_home.to_path_buf(),
                auth_store_mode,
                keyring_backend_kind,
            );
            let (claude_status, pfterminal_plan_status, api_key_statuses) = tokio::join!(
                claude_status,
                tokio::time::timeout(PROVIDER_STATUS_TIMEOUT, pfterminal_plan_status),
                tokio::time::timeout(PROVIDER_STATUS_TIMEOUT, api_key_statuses)
            );
            let pfterminal_plan_status =
                pfterminal_plan_status.unwrap_or(PfTerminalPlanStatus::Unavailable);
            let api_key_statuses = match api_key_statuses {
                Ok(Ok(statuses)) => statuses,
                Ok(Err(_)) | Err(_) => provider_api_key_unavailable_statuses(),
            };
            app_event_tx.send(AppEvent::ProviderCredentialStatusesReady {
                claude_status,
                pfterminal_plan_status,
                api_key_statuses,
            });
        });
    }

    fn provider_credentials_params(
        &self,
        claude_status: &ClaudeCodePlanStatus,
        pfterminal_plan_status: &PfTerminalPlanStatus,
        api_key_statuses: &[(String, ProviderApiKeyStatus)],
    ) -> SelectionViewParams {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Providers".bold()));
        header.push(Line::from(
            "Select a provider to sign in or replace credentials.".dim(),
        ));

        SelectionViewParams {
            view_id: Some(PROVIDER_CREDENTIALS_VIEW_ID),
            footer_hint: Some(standard_popup_hint_line()),
            is_searchable: true,
            search_placeholder: Some("Search providers".to_string()),
            items: provider_credential_items(
                &self.codex_account_status_description(),
                &claude_status_description(claude_status),
                &super::pfterminal_plan_status::description(pfterminal_plan_status),
                |env_key| provider_api_key_status_description(env_key, api_key_statuses),
            ),
            header: Box::new(header),
            ..Default::default()
        }
    }

    fn codex_account_status_description(&self) -> String {
        match self.status_account_display() {
            Some(StatusAccountDisplay::ChatGpt { email, plan }) => {
                signed_in_description(email.as_deref(), plan.as_deref())
            }
            _ if self.has_codex_backend_auth() => "Signed in".to_string(),
            _ => "Not signed in".to_string(),
        }
    }

    pub(crate) fn open_provider_api_key_add(
        &mut self,
        provider_id: String,
        provider_name: String,
        env_key: String,
    ) {
        let display_name = provider_credential_display_name(&provider_name, &env_key);
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            provider_vault_label(&env_key),
            format!("Add {display_name}"),
            "API key — masked".to_string(),
            format!("{env_key} (masked - not shown, not stored in chat)"),
            Box::new(move |_label: String, secret: String| {
                tx.send(AppEvent::SaveProviderApiKey {
                    provider_id,
                    display_name,
                    api_key: crate::app_event::ProviderApiKeySecret::new(secret),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_provider_api_key_save_pending(
        &mut self,
        save_id: String,
        display_name: String,
    ) {
        self.pending_provider_api_key_save_id = Some(save_id.clone());
        self.bottom_pane
            .show_view(Box::new(ProviderApiKeySavePendingView::new(
                self.app_event_tx.clone(),
                save_id,
                display_name,
            )));
    }

    pub(crate) fn on_provider_api_key_save_dismissed(&mut self, save_id: String) {
        clear_matching_provider_api_key_save(&mut self.pending_provider_api_key_save_id, &save_id);
    }

    pub(crate) fn on_provider_api_key_save_finished(
        &mut self,
        save_id: String,
        display_name: String,
        result: Result<(), String>,
    ) {
        let was_active = clear_matching_provider_api_key_save(
            &mut self.pending_provider_api_key_save_id,
            &save_id,
        );
        if was_active {
            self.bottom_pane
                .dismiss_view_by_id(PROVIDER_API_KEY_SAVE_VIEW_ID);
        }
        match (was_active, result) {
            (true, Ok(())) => self.add_info_message(
                format!("Stored {display_name} in the vault."),
                /*hint*/ None,
            ),
            (true, Err(message)) => {
                self.add_error_message(format!("Failed to store {display_name}: {message}"));
            }
            (false, Ok(())) => self.add_info_message(
                format!("Background save completed: stored {display_name} in the vault."),
                /*hint*/ None,
            ),
            (false, Err(message)) => {
                self.add_error_message(format!(
                    "Background save failed for {display_name}: {message}"
                ));
            }
        }
    }

    pub(crate) fn open_codex_account_device_login_pending(&mut self) {
        self.pending_provider_codex_login_id = None;
        self.bottom_pane
            .show_view(Box::new(CodexAccountDeviceLoginView::pending(
                self.app_event_tx.clone(),
            )));
    }

    pub(crate) fn open_codex_account_device_login_ready(
        &mut self,
        login_id: String,
        verification_url: String,
        user_code: String,
    ) {
        self.pending_provider_codex_login_id = Some(login_id.clone());
        self.bottom_pane.replace_active_view_by_id(
            CODEX_ACCOUNT_DEVICE_LOGIN_VIEW_ID,
            Box::new(CodexAccountDeviceLoginView::ready(
                self.app_event_tx.clone(),
                login_id,
                verification_url,
                user_code,
            )),
        );
    }

    pub(crate) fn on_codex_account_device_login_failed(&mut self, message: String) {
        self.pending_provider_codex_login_id = None;
        self.bottom_pane
            .dismiss_view_by_id(CODEX_ACCOUNT_DEVICE_LOGIN_VIEW_ID);
        self.add_error_message(format!("OpenAI Codex account login failed: {message}"));
    }

    #[allow(dead_code)]
    pub(crate) fn on_codex_account_login_completed(
        &mut self,
        notification: codex_app_server_protocol::AccountLoginCompletedNotification,
    ) {
        let Some(login_id) = notification.login_id else {
            return;
        };
        self.on_codex_account_login_result(login_id, notification.success, notification.error);
    }

    pub(crate) fn on_codex_account_login_result(
        &mut self,
        login_id: String,
        success: bool,
        failure_message: Option<String>,
    ) {
        if self.pending_provider_codex_login_id.as_deref() != Some(login_id.as_str()) {
            return;
        }
        self.pending_provider_codex_login_id = None;
        self.bottom_pane
            .dismiss_view_by_id(CODEX_ACCOUNT_DEVICE_LOGIN_VIEW_ID);

        if success {
            self.add_info_message(
                "OpenAI Codex account login complete.".to_string(),
                /*hint*/ None,
            );
        } else {
            self.add_error_message(
                failure_message
                    .unwrap_or_else(|| "OpenAI Codex account login did not complete.".to_string()),
            );
        }
    }
}

fn provider_credential_items(
    codex_account_status: &str,
    claude_status: &str,
    pfterminal_plan_status: &str,
    api_key_status: impl Fn(&str) -> String,
) -> Vec<SelectionItem> {
    PROVIDER_CREDENTIAL_OPTIONS
        .iter()
        .map(|option| {
            provider_credential_item(
                option,
                codex_account_status,
                claude_status,
                pfterminal_plan_status,
                &api_key_status,
            )
        })
        .collect()
}

fn provider_api_key_statuses(codex_home: &Path) -> Vec<(String, ProviderApiKeyStatus)> {
    let stored_labels = match codex_vault::Vault::new(codex_home.to_path_buf()).list() {
        Ok(credentials) => credentials
            .into_iter()
            .map(|credential| credential.label)
            .collect::<std::collections::HashSet<_>>(),
        Err(_) => return provider_api_key_unavailable_statuses(),
    };
    PROVIDER_CREDENTIAL_OPTIONS
        .iter()
        .filter_map(|option| {
            let ProviderCredentialOption::ProviderApiKey { env_key, .. } = option else {
                return None;
            };
            let status = if std::env::var(env_key)
                .ok()
                .is_some_and(|value| !value.trim().is_empty())
            {
                ProviderApiKeyStatus::AvailableFromEnvironment
            } else if stored_labels.contains(&provider_vault_label(env_key)) {
                ProviderApiKeyStatus::Stored
            } else {
                ProviderApiKeyStatus::NotConfigured
            };
            Some((env_key.to_string(), status))
        })
        .collect()
}

fn provider_api_key_unavailable_statuses() -> Vec<(String, ProviderApiKeyStatus)> {
    PROVIDER_CREDENTIAL_OPTIONS
        .iter()
        .filter_map(|option| match option {
            ProviderCredentialOption::ProviderApiKey { env_key, .. } => {
                Some((env_key.to_string(), ProviderApiKeyStatus::Unavailable))
            }
            ProviderCredentialOption::CodexAccount
            | ProviderCredentialOption::ClaudeCodePlan
            | ProviderCredentialOption::PfTerminalPlan => None,
        })
        .collect()
}

fn provider_api_key_status_description(
    env_key: &str,
    statuses: &[(String, ProviderApiKeyStatus)],
) -> String {
    match statuses
        .iter()
        .find(|(status_env_key, _)| status_env_key == env_key)
        .map(|(_, status)| *status)
        .unwrap_or(ProviderApiKeyStatus::Checking)
    {
        ProviderApiKeyStatus::Checking => "Checking...".to_string(),
        ProviderApiKeyStatus::AvailableFromEnvironment => "Available from environment".to_string(),
        ProviderApiKeyStatus::Stored => "Stored in vault".to_string(),
        ProviderApiKeyStatus::NotConfigured => "Not configured".to_string(),
        ProviderApiKeyStatus::Unavailable => "Status unavailable".to_string(),
    }
}

fn provider_credential_item(
    option: &ProviderCredentialOption,
    codex_account_status: &str,
    claude_status: &str,
    pfterminal_plan_status: &str,
    api_key_status: &impl Fn(&str) -> String,
) -> SelectionItem {
    match option {
        ProviderCredentialOption::CodexAccount => {
            let name = "Provider: OpenAI Codex Account".to_string();
            SelectionItem {
                name: name.clone(),
                search_value: Some(name),
                description: Some(codex_account_status.to_string()),
                actions: vec![Box::new(|tx| {
                    tx.send(AppEvent::OpenCodexAccountDeviceLogin);
                })],
                dismiss_on_select: true,
                ..Default::default()
            }
        }
        ProviderCredentialOption::ClaudeCodePlan => {
            let name = "Provider: Claude Code Plan".to_string();
            SelectionItem {
                name: name.clone(),
                search_value: Some(name),
                description: Some(claude_status.to_string()),
                actions: vec![Box::new(|tx| {
                    tx.send(AppEvent::OpenClaudeCodePlanLogin);
                })],
                dismiss_on_select: true,
                ..Default::default()
            }
        }
        ProviderCredentialOption::PfTerminalPlan => {
            let name = "Provider: Corbanu API".to_string();
            SelectionItem {
                name: name.clone(),
                search_value: Some(name),
                description: Some(pfterminal_plan_status.to_string()),
                actions: vec![Box::new(|tx| tx.send(AppEvent::OpenWallet))],
                dismiss_on_select: true,
                ..Default::default()
            }
        }
        ProviderCredentialOption::ProviderApiKey {
            provider_id,
            provider_name,
            env_key,
        } => {
            let provider_id = provider_id.to_string();
            let provider_name = provider_name.to_string();
            let env_key = env_key.to_string();
            let name = provider_credential_display_name(&provider_name, &env_key);
            SelectionItem {
                name: name.clone(),
                search_value: Some(name),
                description: Some(api_key_status(&env_key)),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenProviderApiKeyAdd {
                        provider_id: provider_id.clone(),
                        provider_name: provider_name.clone(),
                        env_key: env_key.clone(),
                    });
                })],
                dismiss_on_select: true,
                ..Default::default()
            }
        }
    }
}

fn claude_status_description(status: &ClaudeCodePlanStatus) -> String {
    match status {
        ClaudeCodePlanStatus::Checking => "Checking sign-in...".to_string(),
        ClaudeCodePlanStatus::ManagedToken { stored: true } => {
            "Selected · long-lived subscription token".to_string()
        }
        ClaudeCodePlanStatus::ManagedToken { stored: false } => {
            "Recovery needed · selected token is missing".to_string()
        }
        ClaudeCodePlanStatus::EnvironmentToken { available: true } => {
            "Selected · CLAUDE_CODE_OAUTH_TOKEN".to_string()
        }
        ClaudeCodePlanStatus::EnvironmentToken { available: false } => {
            "Recovery needed · selected environment token is missing".to_string()
        }
        ClaudeCodePlanStatus::SelectionRequired {
            existing_source_detected: true,
        } => "Choose method · existing credentials detected".to_string(),
        ClaudeCodePlanStatus::SelectionRequired {
            existing_source_detected: false,
        } => "Choose authentication method".to_string(),
        ClaudeCodePlanStatus::InvalidSelection => {
            "Recovery needed · selected method is not valid on this platform".to_string()
        }
        ClaudeCodePlanStatus::NeedsReauthorization => {
            "Recovery needed · selected login needs reauthorization".to_string()
        }
        ClaudeCodePlanStatus::SignedIn {
            email,
            subscription,
            ..
        } => signed_in_description(email.as_deref(), subscription.as_deref()),
        ClaudeCodePlanStatus::SignedOut => "Not signed in".to_string(),
        ClaudeCodePlanStatus::Unavailable => "Claude Code not installed".to_string(),
        ClaudeCodePlanStatus::Error => "Status unavailable".to_string(),
    }
}

fn signed_in_description(email: Option<&str>, plan: Option<&str>) -> String {
    let mut parts = vec!["Signed in".to_string()];
    if let Some(email) = email.filter(|email| !email.trim().is_empty()) {
        parts.push(email.to_string());
    }
    if let Some(plan) = plan.filter(|plan| !plan.trim().is_empty()) {
        let mut characters = plan.chars();
        let display = characters
            .next()
            .map(|first| format!("{}{} plan", first.to_uppercase(), characters.as_str()))
            .unwrap_or_default();
        parts.push(display);
    }
    parts.join(" · ")
}

fn provider_credential_display_name(provider_name: &str, env_key: &str) -> String {
    let key_name = match env_key {
        "ANTHROPIC_API_KEY" => "API Key",
        "AMBIENT_API_KEY" => "API Key",
        "KIMI_API_KEY" => "API Key",
        "ZAI_API_KEY" => "API Key",
        "DEEPSEEK_API_KEY" => "API Key",
        "OPENROUTER_API_KEY" => "API Key",
        "MODEL_API_KEY" => "API Key",
        "BASETEN_API_KEY" => "API Key",
        "AI_GATEWAY_API_KEY" => "API Key",
        _ => env_key,
    };
    format!("Provider: {provider_name} {key_name}")
}

fn provider_vault_label(env_key: &str) -> String {
    format!("provider/{}", env_key.to_ascii_lowercase())
}

fn clear_matching_provider_api_key_save(
    pending_save_id: &mut Option<String>,
    completed_save_id: &str,
) -> bool {
    if pending_save_id.as_deref() != Some(completed_save_id) {
        return false;
    }
    *pending_save_id = None;
    true
}

struct ProviderApiKeySavePendingView {
    app_event_tx: AppEventSender,
    save_id: String,
    display_name: String,
    complete: bool,
}

impl ProviderApiKeySavePendingView {
    fn new(app_event_tx: AppEventSender, save_id: String, display_name: String) -> Self {
        Self {
            app_event_tx,
            save_id,
            display_name,
            complete: false,
        }
    }

    fn cancel(&mut self) {
        if self.complete {
            return;
        }
        self.complete = true;
        self.app_event_tx
            .send(AppEvent::ProviderApiKeySaveDismissed {
                save_id: self.save_id.clone(),
            });
    }

    fn lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(format!("Saving {} securely...", self.display_name).bold()),
            Line::from(""),
            Line::from("Keep Corbanu Terminal open while the encrypted vault is updated.").dim(),
        ]
    }
}

impl BottomPaneView for ProviderApiKeySavePendingView {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        if key_event.code == KeyCode::Esc {
            self.cancel();
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.complete.then_some(ViewCompletion::Cancelled)
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(PROVIDER_API_KEY_SAVE_VIEW_ID)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.cancel();
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }
}

impl Renderable for ProviderApiKeySavePendingView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines())
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        3
    }
}

struct CodexAccountDeviceLoginView {
    app_event_tx: AppEventSender,
    login_id: Option<String>,
    verification_url: Option<String>,
    user_code: Option<String>,
    complete: bool,
    completion: Option<ViewCompletion>,
}

impl CodexAccountDeviceLoginView {
    fn pending(app_event_tx: AppEventSender) -> Self {
        Self {
            app_event_tx,
            login_id: None,
            verification_url: None,
            user_code: None,
            complete: false,
            completion: None,
        }
    }

    fn ready(
        app_event_tx: AppEventSender,
        login_id: String,
        verification_url: String,
        user_code: String,
    ) -> Self {
        Self {
            app_event_tx,
            login_id: Some(login_id),
            verification_url: Some(verification_url),
            user_code: Some(user_code),
            complete: false,
            completion: None,
        }
    }

    fn cancel(&mut self) {
        if self.complete {
            return;
        }
        if let Some(login_id) = self.login_id.take() {
            self.app_event_tx
                .send(AppEvent::CancelCodexAccountDeviceLogin { login_id });
        }
        self.complete = true;
        self.completion = Some(ViewCompletion::Cancelled);
    }

    fn accept(&mut self) {
        if self.complete {
            return;
        }
        self.login_id = None;
        self.complete = true;
        self.completion = Some(ViewCompletion::Accepted);
    }

    fn lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("OpenAI Codex Account".bold()), Line::from("")];
        if let (Some(verification_url), Some(user_code)) = (&self.verification_url, &self.user_code)
        {
            lines.push(Line::from("1. Open this link in your browser and sign in"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                verification_url.clone().cyan().underlined(),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from("2. Enter this one-time code"));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![user_code.clone().cyan().bold()]));
            lines.push(Line::from(""));
            lines.push(
                Line::from("Device codes are a common phishing target. Never share this code.")
                    .dim(),
            );
            lines.push(Line::from(""));
            lines.push(Line::from("Press Esc to cancel").dim());
        } else {
            lines.push(Line::from("Requesting a one-time device code...").dim());
            lines.push(Line::from(""));
            lines.push(Line::from("Press Esc to cancel").dim());
        }
        lines
    }
}

impl BottomPaneView for CodexAccountDeviceLoginView {
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.cancel(),
            KeyCode::Enter => self.accept(),
            _ => {}
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }

    fn view_id(&self) -> Option<&'static str> {
        Some(CODEX_ACCOUNT_DEVICE_LOGIN_VIEW_ID)
    }

    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.cancel();
        CancellationEvent::Handled
    }

    fn prefer_esc_to_handle_key_event(&self) -> bool {
        true
    }
}

impl Renderable for CodexAccountDeviceLoginView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines())
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn desired_height(&self, _width: u16) -> u16 {
        if self.verification_url.is_some() && self.user_code.is_some() {
            13
        } else {
            4
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_event_sender::AppEventSender;

    fn test_provider_rows() -> Vec<SelectionItem> {
        provider_credential_items(
            "Not signed in",
            "Signed in · user@example.com · Max plan",
            "Active · Starter plan · 3speRmS…JRwV5r",
            |_| "Not configured".to_string(),
        )
    }

    #[test]
    fn recommended_provider_rows_are_human_readable() {
        let rows = test_provider_rows();
        let names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "Provider: OpenAI Codex Account",
                "Provider: Claude Code Plan",
                "Provider: Corbanu API",
                "Provider: Anthropic API Key",
                "Provider: Ambient API Key",
                "Provider: Kimi Code API Key",
                "Provider: Z.AI API Key",
                "Provider: DeepSeek API Key",
                "Provider: OpenRouter API Key",
                "Provider: Meta API Key",
                "Provider: Baseten API Key",
                "Provider: Vercel API Key",
            ]
        );
        assert_eq!(rows[0].description.as_deref(), Some("Not signed in"));
        assert_eq!(
            rows[1].description.as_deref(),
            Some("Signed in · user@example.com · Max plan")
        );
        assert_eq!(
            rows[2].description.as_deref(),
            Some("Active · Starter plan · 3speRmS…JRwV5r")
        );
        assert_eq!(rows[3].description.as_deref(), Some("Not configured"));
        assert_eq!(rows[4].description.as_deref(), Some("Not configured"));
    }

    #[test]
    fn every_provider_row_is_searchable_by_its_visible_name() {
        for row in test_provider_rows() {
            assert_eq!(row.search_value.as_deref(), Some(row.name.as_str()));
        }
    }

    #[test]
    fn provider_rows_dispatch_expected_events() {
        let rows = test_provider_rows();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = AppEventSender::new(tx);

        (rows[0].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenCodexAccountDeviceLogin)
        ));

        (rows[1].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenClaudeCodePlanLogin)
        ));

        (rows[2].actions[0])(&sender);
        assert!(matches!(rx.try_recv(), Ok(AppEvent::OpenWallet)));

        (rows[3].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenProviderApiKeyAdd { provider_id, provider_name, env_key })
                if provider_id == ANTHROPIC_PROVIDER_ID
                    && provider_name == "Anthropic"
                    && env_key == ANTHROPIC_API_KEY_ENV_VAR
        ));

        (rows[4].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenProviderApiKeyAdd { provider_id, provider_name, env_key })
                if provider_id == AMBIENT_PROVIDER_ID
                    && provider_name == "Ambient"
                    && env_key == AMBIENT_API_KEY_ENV_VAR
        ));

        (rows[5].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenProviderApiKeyAdd { provider_id, provider_name, env_key })
                if provider_id == KIMI_CODE_PROVIDER_ID
                    && provider_name == "Kimi Code"
                    && env_key == KIMI_CODE_API_KEY_ENV_VAR
        ));

        (rows[7].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenProviderApiKeyAdd { provider_id, provider_name, env_key })
                if provider_id == DEEPSEEK_PROVIDER_ID
                    && provider_name == "DeepSeek"
                    && env_key == DEEPSEEK_API_KEY_ENV_VAR
        ));
    }

    #[test]
    fn provider_status_descriptions_are_explicit() {
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::SignedIn {
                email: Some("user@example.com".to_string()),
                organization_id: Some("org-fixture".to_string()),
                subscription: Some("max".to_string()),
            }),
            "Signed in · user@example.com · Max plan"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::SignedOut),
            "Not signed in"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::ManagedToken { stored: true }),
            "Selected · long-lived subscription token"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::ManagedToken { stored: false }),
            "Recovery needed · selected token is missing"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::SelectionRequired {
                existing_source_detected: true,
            }),
            "Choose method · existing credentials detected"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::InvalidSelection),
            "Recovery needed · selected method is not valid on this platform"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::NeedsReauthorization),
            "Recovery needed · selected login needs reauthorization"
        );
        assert_eq!(
            claude_status_description(&ClaudeCodePlanStatus::Unavailable),
            "Claude Code not installed"
        );
        assert_eq!(
            super::pfterminal_plan_status::description(&PfTerminalPlanStatus::Active {
                plan_id: "starter".to_string(),
                wallet_address: "3speRmSn2J3fhpxpY2B8eQQB8h9i569TajGifjJRwV5r".to_string(),
            }),
            "Active · Starter plan · 3speRmS…JRwV5r"
        );
        assert_eq!(
            super::pfterminal_plan_status::description(&PfTerminalPlanStatus::WalletOnly {
                wallet_address: "3speRmSn2J3fhpxpY2B8eQQB8h9i569TajGifjJRwV5r".to_string(),
            }),
            "Wallet connected · no plan credential · 3speRmS…JRwV5r"
        );
    }

    #[test]
    fn provider_status_rows_snapshot() {
        let rows = test_provider_rows();
        let rendered = rows
            .iter()
            .map(|row| {
                format!(
                    "{:<38} {}",
                    row.name,
                    row.description.as_deref().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn codex_account_device_login_escape_cancels_only_active_login() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = AppEventSender::new(tx);
        let esc = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        );

        let mut pending = CodexAccountDeviceLoginView::pending(sender.clone());
        pending.handle_key_event(esc);
        assert!(pending.is_complete());
        assert!(matches!(
            pending.completion(),
            Some(ViewCompletion::Cancelled)
        ));
        assert!(rx.try_recv().is_err());

        let mut ready = CodexAccountDeviceLoginView::ready(
            sender,
            "login-1".to_string(),
            "https://example.com/device".to_string(),
            "ABCD-EFGH".to_string(),
        );
        ready.handle_key_event(esc);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::CancelCodexAccountDeviceLogin { login_id }) if login_id == "login-1"
        ));

        ready.handle_key_event(esc);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn codex_account_device_login_accept_disarms_cancel() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = AppEventSender::new(tx);
        let mut view = CodexAccountDeviceLoginView::ready(
            sender,
            "login-1".to_string(),
            "https://example.com/device".to_string(),
            "ABCD-EFGH".to_string(),
        );

        view.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(matches!(view.completion(), Some(ViewCompletion::Accepted)));

        view.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn provider_api_key_save_view_describes_anthropic_progress() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let view = ProviderApiKeySavePendingView::new(
            AppEventSender::new(tx),
            "save-1".to_string(),
            "Provider: Anthropic API Key".to_string(),
        );
        let area = Rect::new(0, 0, 72, view.desired_height(/*width*/ 72));
        let mut buffer = Buffer::empty(area);
        view.render(area, &mut buffer);
        let rendered = (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!(rendered);
        assert_eq!(view.view_id(), Some(PROVIDER_API_KEY_SAVE_VIEW_ID));
    }

    #[test]
    fn provider_api_key_save_view_blocks_text_but_allows_dismissal() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = AppEventSender::new(tx);
        let mut esc_view = ProviderApiKeySavePendingView::new(
            sender.clone(),
            "save-esc".to_string(),
            "Provider: Anthropic API Key".to_string(),
        );
        esc_view.handle_key_event(crossterm::event::KeyEvent::new(
            KeyCode::Char('x'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(!esc_view.is_complete());
        assert!(rx.try_recv().is_err());

        esc_view.handle_key_event(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(esc_view.is_complete());
        assert_eq!(esc_view.completion(), Some(ViewCompletion::Cancelled));
        assert!(esc_view.prefer_esc_to_handle_key_event());
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::ProviderApiKeySaveDismissed { save_id }) if save_id == "save-esc"
        ));
        esc_view.handle_key_event(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(rx.try_recv().is_err());

        let mut ctrl_c_view = ProviderApiKeySavePendingView::new(
            sender,
            "save-ctrl-c".to_string(),
            "Provider: Anthropic API Key".to_string(),
        );
        assert_eq!(ctrl_c_view.on_ctrl_c(), CancellationEvent::Handled);
        assert!(ctrl_c_view.is_complete());
        assert_eq!(ctrl_c_view.completion(), Some(ViewCompletion::Cancelled));
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::ProviderApiKeySaveDismissed { save_id }) if save_id == "save-ctrl-c"
        ));
    }

    #[test]
    fn cancelled_save_completion_cannot_clear_a_newer_save() {
        let mut pending_save_id = Some("first-save".to_string());

        assert!(clear_matching_provider_api_key_save(
            &mut pending_save_id,
            "first-save"
        ));
        assert_eq!(pending_save_id, None);

        pending_save_id = Some("second-save".to_string());
        assert!(!clear_matching_provider_api_key_save(
            &mut pending_save_id,
            "first-save"
        ));
        assert_eq!(pending_save_id.as_deref(), Some("second-save"));
        assert!(clear_matching_provider_api_key_save(
            &mut pending_save_id,
            "second-save"
        ));
        assert_eq!(pending_save_id, None);
    }

    #[test]
    fn provider_vault_label_matches_provider_key_storage() {
        assert_eq!(
            provider_vault_label("ZAI_API_KEY"),
            "provider/zai_api_key".to_string()
        );
    }
}
