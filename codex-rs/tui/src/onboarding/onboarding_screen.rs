//! Onboarding screen orchestration and top-level keyboard routing.
//!
//! The onboarding flow is a small state machine over visible steps
//! (welcome/auth/trust/GovCloud). This module decides which step receives key/paste
//! events and enforces flow-level safety rules that cut across individual step
//! widgets. Folder-entry consent reuses this same event loop for startup and in-app navigation.
//!
//! In particular, onboarding quit handling has a text-entry guard: while any
//! text field has focus (an API key, a Claude token or code, or a named
//! account's name or value) or its value is saving, the printable `q` quit key
//! is text, even in an empty field. Control chords (Ctrl+C, Ctrl+D) remain explicit exits. Quitting
//! on a printable key mid-entry would drop the rest of a typed or non-bracketed
//! pasted secret onto the user's shell (#461).

use codex_app_server_client::AppServerEvent;
use codex_app_server_client::AppServerRequestHandle;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ConfigBatchWriteParams;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerNotification;
use codex_exec_server::LOCAL_FS;
use codex_git_utils::resolve_root_git_project_for_trust;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::widgets::Clear;
use ratatui::widgets::WidgetRef;

use codex_model_provider_info::AMBIENT_API_KEY_ENV_VAR;
use codex_model_provider_info::ANTHROPIC_API_KEY_ENV_VAR;
use codex_model_provider_info::BASETEN_API_KEY_ENV_VAR;
use codex_model_provider_info::DEEPSEEK_API_KEY_ENV_VAR;
use codex_model_provider_info::KIMI_CODE_API_KEY_ENV_VAR;
use codex_model_provider_info::META_API_KEY_ENV_VAR;
use codex_model_provider_info::OPENROUTER_API_KEY_ENV_VAR;
use codex_model_provider_info::VERCEL_API_KEY_ENV_VAR;
use codex_model_provider_info::ZAI_API_KEY_ENV_VAR;
use codex_protocol::config_types::ForcedLoginMethod;

use crate::LoginStatus;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::app_server_session::AppServerSession;
use crate::clipboard_copy::CopyFormat;
use crate::clipboard_copy::CopyStatus;
use crate::config_update::RemoteProjectTrust;
use crate::config_update::format_config_error;
use crate::config_update::replace_config_value;
use crate::config_update::write_trusted_project;
use crate::empty_state_animation::Presentation;
use crate::key_hint::KeyBindingListExt;
use crate::legacy_core::config::Config;
use crate::onboarding::auth::ApiKeyInputState;
use crate::onboarding::auth::ApiKeyProviderOption;
use crate::onboarding::auth::AuthModeWidget;
use crate::onboarding::auth::SignInOption;
use crate::onboarding::auth::SignInState;
<<<<<<< HEAD
use crate::onboarding::auth::catalog_sign_in_options_from_statuses;
||||||| 413492cd6c
=======
use crate::onboarding::bedrock::BedrockState;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::onboarding::keys;
use crate::onboarding::provider_setup::DeferredProviderSetup;
use crate::onboarding::provider_setup::ProviderSetupSession;
use crate::onboarding::trust_directory::TrustDirectorySelection;
use crate::onboarding::trust_directory::TrustDirectoryWidget;
use crate::onboarding::welcome::WelcomeWidget;
<<<<<<< HEAD
use crate::provider_auth_effect_executor::ProviderAuthEffectExecutor;
use crate::provider_status_host::ProviderAccountMetadata;
use crate::provider_status_host::ProviderStatusHost;
||||||| 413492cd6c
=======
use crate::startup_draft::StartupCancelled;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::tui::FrameRequester;
use crate::tui::MouseCapture;
use crate::tui::Tui;
use crate::tui::TuiEvent;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;

#[path = "directory_trust.rs"]
mod directory_trust;
pub(crate) use directory_trust::DirectoryTrustOptions;
pub(crate) use directory_trust::check_directory_trust;

#[allow(clippy::large_enum_variant)]
enum Step {
    Welcome(WelcomeWidget),
    Auth(AuthModeWidget),
    TrustDirectory(TrustDirectoryWidget),
    GovCloudGuidance {
        state: BedrockState,
        acknowledged: bool,
    },
}

pub(crate) trait KeyboardHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent);
    fn handle_paste(&mut self, _pasted: String) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepState {
    Hidden,
    InProgress,
    Complete,
}

pub(crate) trait StepStateProvider {
    fn get_step_state(&self) -> StepState;
}

pub(crate) struct OnboardingScreen {
    request_frame: FrameRequester,
    steps: Vec<Step>,
    remote_trust_key: Option<String>,
    is_done: bool,
    should_exit: bool,
    provider_auth_controller: codex_provider_auth::ProviderAuthController,
    provider_auth_executor: Option<ProviderAuthEffectExecutor>,
    provider_auth_action_rx:
        tokio::sync::mpsc::UnboundedReceiver<codex_provider_auth::ProviderAuthAction>,
}

pub(crate) struct OnboardingScreenArgs {
    pub show_trust_screen: bool,
    pub remote_project_trust: Option<RemoteProjectTrust>,
    pub show_login_screen: bool,
    pub bedrock_setup_enabled: bool,
    pub gcp_setup_enabled: bool,
    pub login_status: LoginStatus,
    pub app_server_target: crate::AppServerTarget,
    pub app_server_request_handle: Option<AppServerRequestHandle>,
    pub config: Config,
}

#[derive(Default)]
pub(crate) struct OnboardingResult {
    pub directory_trust_persisted: bool,
    pub should_exit: bool,
    pub configured_provider: Option<String>,
    pub deferred_provider_setup: Option<DeferredProviderSetup>,
}

<<<<<<< HEAD
fn provider_api_key_options(host: &ProviderStatusHost) -> Vec<ApiKeyProviderOption> {
    host.catalog()
        .entries()
        .iter()
        .flat_map(|entry| {
            entry.setup_capabilities.iter().filter_map(|capability| {
                let codex_provider_auth::ProviderSetupCapability::ApiKey { .. } = capability else {
                    return None;
                };
                let target = codex_provider_auth::ApiKeyAuthTarget::from_catalog_capability(
                    entry, capability,
                )
                .ok()?;
                Some(ApiKeyProviderOption {
                    target,
                    capability: capability.clone(),
                    name: entry.display_name.clone(),
                })
            })
        })
        .collect()
}

pub(crate) fn provider_api_key_display_name(provider: &ApiKeyProviderOption) -> String {
    let codex_provider_auth::ApiKeyStorage::EnvironmentVariable { env_key } =
        &provider.target.storage
    else {
        return "Provider: OpenAI API Key".to_string();
    };
    match env_key.as_str() {
        ANTHROPIC_API_KEY_ENV_VAR => "Provider: Anthropic API Key".to_string(),
        AMBIENT_API_KEY_ENV_VAR => "Provider: Ambient API Key".to_string(),
        KIMI_CODE_API_KEY_ENV_VAR => "Provider: Kimi Code API Key".to_string(),
        ZAI_API_KEY_ENV_VAR => "Provider: Z.AI API Key".to_string(),
        DEEPSEEK_API_KEY_ENV_VAR => "Provider: DeepSeek API Key".to_string(),
        OPENROUTER_API_KEY_ENV_VAR => "Provider: OpenRouter API Key".to_string(),
        META_API_KEY_ENV_VAR => "Provider: Meta API Key".to_string(),
        BASETEN_API_KEY_ENV_VAR => "Provider: Baseten API Key".to_string(),
        VERCEL_API_KEY_ENV_VAR => "Provider: Vercel API Key".to_string(),
        _ => format!("Provider: {} {}", provider.name, env_key),
    }
}

fn initial_sign_in_option(
    catalog_options: &[SignInOption],
    forced_login_method: Option<ForcedLoginMethod>,
) -> SignInOption {
    match forced_login_method {
        Some(ForcedLoginMethod::Api) => SignInOption::ApiKey,
        Some(ForcedLoginMethod::Chatgpt) => SignInOption::ChatGpt,
        None => catalog_options
            .first()
            .copied()
            .unwrap_or(SignInOption::ChatGpt),
    }
||||||| 413492cd6c
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ApiKeyEntryContext {
    /// True when onboarding is currently rendering the API-key entry state.
    active: bool,
    /// True when the API-key input field currently contains user text.
    has_text: bool,
=======
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextEntryContext {
    /// Whether onboarding is currently rendering a text-entry state.
    active: bool,
    /// Whether printable quit shortcuts should be consumed as text input.
    has_text: bool,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

impl OnboardingScreen {
    pub(crate) async fn new(
        tui: &mut Tui,
        args: OnboardingScreenArgs,
        claude_event_tx: AppEventSender,
    ) -> Self {
        let OnboardingScreenArgs {
            show_trust_screen,
            remote_project_trust,
            show_login_screen,
            bedrock_setup_enabled,
            gcp_setup_enabled,
            login_status,
            app_server_target,
            app_server_request_handle,
            config,
        } = args;
        let local_settings = crate::local_settings::LocalSettings::from(&config);
        let cwd = config.cwd.to_path_buf();
<<<<<<< HEAD
        let provider_status_host = ProviderStatusHost::from_config(
            &config,
            ProviderAccountMetadata {
                openai: openai_metadata(login_status),
                claude: codex_provider_auth::ClaudeCredentialMetadata::Checking,
                ..ProviderAccountMetadata::default()
            },
        );
        let provider_statuses = provider_status_host.resolve();
        let provider_setup_session = Arc::new(RwLock::new(ProviderSetupSession::from_statuses(
            provider_statuses.entries(),
        )));
        let (provider_auth_action_tx, provider_auth_action_rx) =
            tokio::sync::mpsc::unbounded_channel();
        let discovery_config = config.clone();
        let discovery_tx = claude_event_tx.clone();
        tokio::spawn(async move {
            let metadata = ProviderAccountMetadata::discover(&discovery_config).await;
            discovery_tx.send(AppEvent::ProviderAccountMetadataResolved { metadata });
        });
        let provider_auth_executor = app_server_request_handle.clone().map(|request_handle| {
            ProviderAuthEffectExecutor::new(
                request_handle,
                provider_auth_action_tx.clone(),
                provider_status_host.clone(),
            )
        });
        let codex_home = config.codex_home.to_path_buf();
        let forced_login_method = config.forced_login_method;
        let mut api_key_provider_id = config.model_provider_id.clone();
        let mut api_key_provider_name = config.model_provider.name.clone();
        let mut api_key_env_var = config.model_provider.env_key.clone();
        let api_key_provider_options = provider_api_key_options(&provider_status_host);
        // The first option stands in only when the screen will name that option too (the
        // override below). Otherwise the screen names the configured provider, and saving the
        // key to another provider would report success for the wrong one.
        let first_option_stands_in =
            api_key_env_var.is_none() || config.model_provider.requires_openai_auth;
        let selected_provider_index = api_key_provider_options
            .iter()
            .position(|provider| provider.target.provider_id.as_str() == api_key_provider_id)
            .or_else(|| first_option_stands_in.then_some(0));
        let selected_api_key_option = selected_provider_index
            .and_then(|index| api_key_provider_options.get(index))
            .cloned();
        if let Some(provider) = selected_api_key_option.as_ref()
            && first_option_stands_in
        {
            api_key_provider_id = provider.target.provider_id.as_str().to_string();
            api_key_provider_name = provider.name.clone();
            api_key_env_var = provider.environment_variable().map(str::to_string);
        }
||||||| 413492cd6c
        let forced_login_method = config.forced_login_method;
=======
        let remote_trust_key = remote_project_trust
            .as_ref()
            .map(|project| project.trust_target.to_string_lossy().into_owned());
        let auth_config = config.auth_config();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let mut steps: Vec<Step> = Vec::new();
        steps.push(Step::Welcome(WelcomeWidget::new(
            !matches!(login_status, LoginStatus::NotAuthenticated),
            tui.frame_requester(),
            local_settings.tui.animations && local_settings.tui.effects.welcome,
        )));
        #[cfg(target_os = "windows")]
        let show_windows_create_sandbox_hint = if show_trust_screen
            && remote_project_trust.is_none()
            && let Some(handle) = &app_server_request_handle
        {
            crate::windows_sandbox::WindowsSandboxConfig::read(
                handle.clone(),
                config.cwd.display().to_string(),
            )
            .await
            .is_ok_and(|state| !state.is_enabled())
        } else {
            false
        };
        #[cfg(not(target_os = "windows"))]
        let show_windows_create_sandbox_hint = false;
        if show_login_screen {
<<<<<<< HEAD
            let catalog_options = catalog_sign_in_options_from_statuses(
                &provider_status_host,
                &provider_statuses,
                &api_key_provider_options,
            );
            let has_multiple_provider_options = catalog_options.len() > 1;
            // #1563: a chosen but unset Corbanu Plan starts on its own setup row.
            let highlighted_mode = if forced_login_method.is_none()
                && config.model_provider_id
                    == codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID
                && catalog_options.contains(&SignInOption::CorbanuPlan)
            {
                SignInOption::CorbanuPlan
            } else {
                initial_sign_in_option(&catalog_options, forced_login_method)
            };
            let initial_sign_in_state =
                if matches!(forced_login_method, Some(ForcedLoginMethod::Api))
                    || (!has_multiple_provider_options && api_key_env_var.is_some())
                {
                    SignInState::ApiKeyEntry(ApiKeyInputState::from_env(api_key_env_var.as_deref()))
                } else {
                    SignInState::PickMode
                };
            let named_account_methods = if config
                .features
                .enabled(codex_features::Feature::NamedAccounts)
            {
                config
                    .model_providers
                    .iter()
                    .map(|(id, provider)| {
                        (
                            id.clone(),
                            crate::provider_named_accounts::add_methods(id, provider)
                                .into_iter()
                                // Onboarding takes pasted secrets; paths, AWS profiles and OpenAI sign-ins are in /providers.
                                .filter(|method| {
                                    !matches!(
                                        method,
                                        crate::provider_named_accounts::AddAccountMethod::ClaudeConfigDir
                                            | crate::provider_named_accounts::AddAccountMethod::AwsProfile
                                            | crate::provider_named_accounts::AddAccountMethod::ChatgptDeviceCode
                                            | crate::provider_named_accounts::AddAccountMethod::OpenaiApiKey
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .filter(|(_, methods)| !methods.is_empty())
                    .collect()
            } else {
                std::collections::BTreeMap::new()
            };
||||||| 413492cd6c
            let highlighted_mode = match forced_login_method {
                Some(ForcedLoginMethod::Api) => SignInOption::ApiKey,
                _ => SignInOption::ChatGpt,
            };
=======
            let highlighted_mode =
                if auth_config.is_login_method_allowed(ForcedLoginMethod::Chatgpt) {
                    SignInOption::ChatGpt
                } else {
                    SignInOption::OtherMethods
                };
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            if let Some(app_server_request_handle) = app_server_request_handle {
                let mut widget = AuthModeWidget {
                    request_frame: tui.frame_requester(),
                    highlighted_mode,
                    activation_key_guard: None,
                    error: Arc::new(RwLock::new(None)),
                    sign_in_state: Arc::new(RwLock::new(initial_sign_in_state)),
                    login_status,
<<<<<<< HEAD
                    replace_provider_with_openai_on_login: config.model_provider_id
                        == codex_model_provider_info::OPENAI_PROVIDER_ID
                        || !config.model_provider.requires_openai_auth,
||||||| 413492cd6c
=======
                    app_server_target,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    app_server_request_handle,
<<<<<<< HEAD
                    forced_login_method,
                    api_key_provider_id,
                    api_key_provider_name,
                    api_key_env_var,
                    api_key_provider_options,
                    selected_api_key_option,
                    animations_enabled: config.animations,
||||||| 413492cd6c
                    forced_login_method,
                    animations_enabled: config.animations,
=======
                    auth_config,
                    bedrock_setup_enabled,
                    gcp_setup_enabled,
                    animations_enabled: local_settings.tui.animations
                        && local_settings.tui.effects.shimmer,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    animations_suppressed: std::cell::Cell::new(false),
                    codex_home,
                    claude_event_tx,
                    provider_setup_session,
                    provider_status_host,
                    provider_statuses: Arc::new(RwLock::new(provider_statuses)),
                    provider_auth_action_tx,
                    named_account_methods,
                };
                widget.begin_initial_api_key_entry();
                steps.push(Step::Auth(widget));
            } else {
                tracing::warn!("skipping onboarding login step without app-server request handle");
            }
        }
        let highlighted = TrustDirectorySelection::Trust;
        if show_trust_screen {
            let (cwd, trust_target) = match remote_project_trust {
                Some(RemoteProjectTrust {
                    cwd, trust_target, ..
                }) => (cwd, trust_target),
                None => {
                    let trust_target =
                        resolve_root_git_project_for_trust(LOCAL_FS.as_ref(), &config.cwd)
                            .await
                            .map(Into::into)
                            .unwrap_or_else(|| cwd.clone());
                    (cwd, trust_target)
                }
            };
            steps.push(Step::TrustDirectory(TrustDirectoryWidget {
                restricted: false,
                existing_task: false,
                cancel: super::trust_directory::TrustCancelAction::Quit,
                cwd,
                trust_target,
                show_windows_create_sandbox_hint,
                should_quit: false,
                selection: None,
                highlighted,
                error: None,
            }))
        }
        Self {
            request_frame: tui.frame_requester(),
            steps,
            remote_trust_key,
            is_done: false,
            should_exit: false,
            provider_auth_controller: codex_provider_auth::ProviderAuthController::default(),
            provider_auth_executor,
            provider_auth_action_rx,
        }
    }

    fn current_steps_mut(&mut self) -> Vec<&mut Step> {
        let states = self
            .steps
            .iter()
            .map(StepStateProvider::get_step_state)
            .collect::<Vec<_>>();
        let welcome_index = self
            .steps
            .iter()
            .position(|step| matches!(step, Step::Welcome(_)));
        let visible_indices = visible_step_indices(&states, welcome_index);
        self.steps
            .iter_mut()
            .enumerate()
            .filter_map(|(index, step)| visible_indices.contains(&index).then_some(step))
            .collect()
    }

    fn current_steps(&self) -> Vec<&Step> {
        let states = self
            .steps
            .iter()
            .map(StepStateProvider::get_step_state)
            .collect::<Vec<_>>();
        let welcome_index = self
            .steps
            .iter()
            .position(|step| matches!(step, Step::Welcome(_)));
        let visible_indices = visible_step_indices(&states, welcome_index);
        self.steps
            .iter()
            .enumerate()
            .filter_map(|(index, step)| visible_indices.contains(&index).then_some(step))
            .collect()
    }

    fn should_suppress_animations(&self) -> bool {
        // Freeze the whole onboarding screen when auth is showing copyable login
        // material so terminal selection is not interrupted by redraws.
        self.current_steps().into_iter().any(|step| match step {
            Step::Auth(widget) => widget.should_suppress_animations(),
            Step::Welcome(_) | Step::TrustDirectory(_) | Step::GovCloudGuidance { .. } => false,
        })
    }

    fn is_auth_in_progress(&self) -> bool {
        self.steps.iter().any(|step| {
            matches!(step, Step::Auth(_)) && matches!(step.get_step_state(), StepState::InProgress)
        })
    }

    pub(crate) fn is_done(&self) -> bool {
        self.is_done
            || !self
                .steps
                .iter()
                .any(|step| matches!(step.get_step_state(), StepState::InProgress))
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

<<<<<<< HEAD
    fn configured_provider(&self) -> Option<String> {
        self.steps.iter().find_map(|step| {
            if let Step::Auth(widget) = step {
                return widget.configured_provider();
            }
            None
        })
    }

    fn deferred_provider_setup(&self) -> Option<DeferredProviderSetup> {
        self.steps.iter().find_map(|step| match step {
            Step::Auth(widget) => widget.deferred_provider_setup(),
            Step::Welcome(_) | Step::TrustDirectory(_) => None,
        })
||||||| 413492cd6c
=======
    /// Check whether the first unfinished onboarding step can grant directory trust.
    fn is_trust_step_active(&self) -> bool {
        self.steps
            .iter()
            .find(|step| matches!(step.get_step_state(), StepState::InProgress))
            .is_some_and(|step| matches!(step, Step::TrustDirectory(_)))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    fn cancel_auth_if_active(&self) {
        for step in &self.steps {
            if let Step::Auth(widget) = step {
                widget.cancel_active_attempt();
            }
        }
    }

    fn auth_widget_mut(&mut self) -> Option<&mut AuthModeWidget> {
        self.steps.iter_mut().find_map(|step| match step {
            Step::Auth(widget) => Some(widget),
            Step::Welcome(_) | Step::TrustDirectory(_) | Step::GovCloudGuidance { .. } => None,
        })
    }

    fn handle_app_server_notification(&mut self, notification: ServerNotification) {
        match notification {
            ServerNotification::AccountLoginCompleted(notification) => {
                if let Some(widget) = self.auth_widget_mut() {
                    widget.on_account_login_completed(notification);
                }
            }
            ServerNotification::AccountUpdated(notification) => {
                if let Some(widget) = self.auth_widget_mut() {
                    widget.on_account_updated(notification);
                }
            }
            _ => {}
        }
    }

<<<<<<< HEAD
    fn handle_claude_event(&mut self, event: AppEvent) {
        if let Some(widget) = self.auth_widget_mut() {
            widget.handle_claude_event(event);
        }
    }

    fn handle_provider_auth_action(&mut self, action: codex_provider_auth::ProviderAuthAction) {
        let transition = self.provider_auth_controller.dispatch(action);
        if let Some(widget) = self.auth_widget_mut() {
            widget.on_provider_auth_snapshot(&transition.snapshot);
        }
        for effect in transition.effects {
            match effect {
                codex_provider_auth::ProviderAuthEffect::Complete(
                    codex_provider_auth::ProviderAuthCompletion::Configured { target, .. },
                ) => {
                    if let Some(widget) = self.auth_widget_mut() {
                        widget.on_api_key_configured(target.provider_id.as_str());
                    }
||||||| 413492cd6c
    fn api_key_entry_context(&self) -> ApiKeyEntryContext {
        self.steps
            .iter()
            .find_map(|step| {
                if let Step::Auth(widget) = step {
                    Some(ApiKeyEntryContext {
                        active: widget.is_api_key_entry_active(),
                        has_text: widget.api_key_entry_has_text(),
                    })
                } else {
                    None
=======
    fn text_entry_context(&self) -> TextEntryContext {
        self.steps
            .iter()
            .find_map(|step| {
                if let Step::Auth(widget) = step {
                    Some(TextEntryContext {
                        active: widget.is_text_entry_active(),
                        has_text: widget.should_suppress_printable_quit(),
                    })
                } else {
                    None
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                }
                effect => {
                    if let Some(executor) = &self.provider_auth_executor {
                        let _ = executor.execute(effect);
                    }
                }
            }
        }
    }

    fn text_entry_active(&self) -> bool {
        self.steps.iter().any(|step| {
            if let Step::Auth(widget) = step {
                widget.is_text_entry_active()
            } else {
                false
            }
        })
    }
}

impl KeyboardHandler for OnboardingScreen {
    /// Route key events to onboarding steps while preserving text-entry safety.
    ///
<<<<<<< HEAD
    /// While a text field has focus, printable quit bindings are text. Control
    /// quit chords still work as emergency exits.
||||||| 413492cd6c
    /// In API-key entry mode, printable quit bindings are suppressed only after
    /// the user has started typing in the API-key field. This keeps the
    /// printable `q` quit key usable on an empty field while protecting in-progress
    /// text entry from accidental exits. Control/alt quit chords still work as
    /// emergency exits.
=======
    /// OpenAI API-key entry suppresses printable quit bindings after typing
    /// starts, while cloud provider fields suppress them from the first character.
    /// Control/alt quit chords still work as emergency exits.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind == KeyEventKind::Release {
            if let Some(widget) = self.auth_widget_mut() {
                widget.handle_key_event(key_event);
            }
            return;
        }
<<<<<<< HEAD
||||||| 413492cd6c
        let api_key_entry_context = self.api_key_entry_context();
=======
        let text_entry_context = self.text_entry_context();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let should_quit = key_event.kind == KeyEventKind::Press
            && keys::QUIT.is_pressed(key_event)
<<<<<<< HEAD
            && !is_text_key_in_text_field(key_event, self.text_entry_active());
||||||| 413492cd6c
            && !suppress_quit_while_typing_api_key(key_event, api_key_entry_context);
=======
            && !suppress_quit_while_typing(key_event, text_entry_context);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        if should_quit {
            if self.is_auth_in_progress() {
                self.cancel_auth_if_active();
                // If the user cancels the auth menu, exit the app rather than
                // leave the user at a prompt in an unauthed state.
                self.should_exit = true;
            } else if self.is_trust_step_active()
                || self
                    .steps
                    .iter()
                    .any(|step| matches!(step, Step::GovCloudGuidance { .. }))
            {
                self.should_exit = true;
            }
            self.is_done = true;
        } else {
            if let Some(Step::Welcome(widget)) = self
                .steps
                .iter_mut()
                .find(|step| matches!(step, Step::Welcome(_)))
            {
                widget.handle_key_event(key_event);
            }
            if let Some(active_step) = self.current_steps_mut().into_iter().last() {
                active_step.handle_key_event(key_event);
            }
            if self.steps.iter().any(|step| {
                if let Step::TrustDirectory(widget) = step {
                    widget.should_quit()
                } else {
                    false
                }
            }) {
                self.should_exit = true;
                self.is_done = true;
            }
        }
        self.request_frame.schedule_frame();
    }

    fn handle_paste(&mut self, pasted: String) {
        if pasted.is_empty() {
            return;
        }

        if let Some(active_step) = self.current_steps_mut().into_iter().last() {
            active_step.handle_paste(pasted);
        }
        self.request_frame.schedule_frame();
    }
}

<<<<<<< HEAD
/// Returns `true` when a key is text for the focused text field, so it must
/// never act as a shortcut: a printable character without control/alt.
fn is_text_key_in_text_field(key_event: KeyEvent, text_entry_active: bool) -> bool {
    text_entry_active
||||||| 413492cd6c
/// Returns `true` when a quit shortcut should be ignored as text input.
///
/// This only applies while API-key entry is active and the key is a printable
/// character without control/alt modifiers and there is already text in the
/// input field. Empty input intentionally does not trigger suppression so
/// the printable `q` quit key can still exit onboarding.
fn suppress_quit_while_typing_api_key(
    key_event: KeyEvent,
    api_key_entry_context: ApiKeyEntryContext,
) -> bool {
    api_key_entry_context.active
        && api_key_entry_context.has_text
=======
/// Returns `true` when a quit shortcut should be ignored as text input.
///
/// This only applies when the active text-entry state suppresses printable quit
/// shortcuts and the key has no control/alt modifiers.
fn suppress_quit_while_typing(key_event: KeyEvent, text_entry_context: TextEntryContext) -> bool {
    text_entry_context.active
        && text_entry_context.has_text
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        && matches!(key_event.code, KeyCode::Char(_))
        && !key_event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

impl WidgetRef for &OnboardingScreen {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let suppress_animations = self.should_suppress_animations();
        let logo_presentation = if suppress_animations {
            Presentation::Hidden
        } else if self.text_entry_context().active {
            Presentation::Faded
        } else {
            Presentation::Animated
        };
        for step in self.current_steps() {
            match step {
                Step::Welcome(widget) => widget.set_presentation(logo_presentation),
                Step::Auth(widget) => widget.set_animations_suppressed(suppress_animations),
                Step::TrustDirectory(_) | Step::GovCloudGuidance { .. } => {}
            }
        }

        Clear.render(area, buf);
        // Render steps top-to-bottom, measuring each step's height dynamically.
        let mut y = area.y;
        let bottom = area.y.saturating_add(area.height);
        let width = area.width;

        // Helper to scan a temporary buffer and return number of used rows.
        fn used_rows(tmp: &Buffer, width: u16, height: u16) -> u16 {
            if width == 0 || height == 0 {
                return 0;
            }
            let mut last_non_empty: Option<u16> = None;
            for yy in 0..height {
                let mut any = false;
                for xx in 0..width {
                    let cell = &tmp[(xx, yy)];
                    let has_symbol = !cell.symbol().trim().is_empty();
                    let has_style = cell.fg != Color::Reset
                        || cell.bg != Color::Reset
                        || !cell.modifier.is_empty();
                    if has_symbol || has_style {
                        any = true;
                        break;
                    }
                }
                if any {
                    last_non_empty = Some(yy);
                }
            }
            last_non_empty.map(|v| v + 2).unwrap_or(0)
        }

        let mut i = 0usize;
        let current_steps = self.current_steps();

        while i < current_steps.len() && y < bottom {
            let step = &current_steps[i];
            let max_h = bottom.saturating_sub(y);
            if max_h == 0 || width == 0 {
                break;
            }
            let scratch_area = Rect::new(0, 0, width, max_h);
            let mut scratch = Buffer::empty(scratch_area);
            if let Step::Welcome(widget) = step {
                widget.update_layout_area(scratch_area);
            }
            step.render_ref(scratch_area, &mut scratch);
            let h = used_rows(&scratch, width, max_h).min(max_h);
            if h > 0 {
                let target = Rect {
                    x: area.x,
                    y,
                    width,
                    height: h,
                };
                Clear.render(target, buf);
                step.render_ref(target, buf);
                y = y.saturating_add(h);
            }
            i += 1;
        }
    }
}

impl KeyboardHandler for Step {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self {
            Step::Welcome(widget) => widget.handle_key_event(key_event),
            Step::Auth(widget) => widget.handle_key_event(key_event),
            Step::TrustDirectory(widget) => widget.handle_key_event(key_event),
            Step::GovCloudGuidance {
                state,
                acknowledged,
            } => {
                *acknowledged = state.handle_guidance_key_event(&key_event);
            }
        }
    }

    fn handle_paste(&mut self, pasted: String) {
        match self {
            Step::Welcome(_) | Step::GovCloudGuidance { .. } => {}
            Step::Auth(widget) => widget.handle_paste(pasted),
            Step::TrustDirectory(widget) => widget.handle_paste(pasted),
        }
    }
}

impl StepStateProvider for Step {
    fn get_step_state(&self) -> StepState {
        match self {
            Step::Welcome(w) => w.get_step_state(),
            Step::Auth(w) => w.get_step_state(),
            Step::TrustDirectory(w) => w.get_step_state(),
            Step::GovCloudGuidance { acknowledged, .. } => {
                if *acknowledged {
                    StepState::Complete
                } else {
                    StepState::InProgress
                }
            }
        }
    }
}

fn visible_step_indices(states: &[StepState], welcome_index: Option<usize>) -> Vec<usize> {
    let active_index = states
        .iter()
        .position(|state| *state == StepState::InProgress);
    states
        .iter()
        .enumerate()
        .filter_map(|(index, state)| {
            if *state == StepState::Hidden {
                return None;
            }
            match active_index {
                Some(active_index) if Some(index) == welcome_index || index == active_index => {
                    Some(index)
                }
                Some(_) => None,
                None => Some(index),
            }
        })
        .collect()
}

impl WidgetRef for Step {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match self {
            Step::Welcome(widget) => {
                widget.render_ref(area, buf);
            }
            Step::Auth(widget) => {
                widget.render_ref(area, buf);
            }
            Step::TrustDirectory(widget) => {
                widget.render_ref(area, buf);
            }
            Step::GovCloudGuidance { state, .. } => state.render(area, buf, /*error*/ None),
        }
    }
}

pub(crate) async fn run_gov_cloud_guidance(tui: &mut Tui) -> Result<()> {
    let screen = OnboardingScreen {
        request_frame: tui.frame_requester(),
        steps: vec![Step::GovCloudGuidance {
            state: BedrockState::guidance(),
            acknowledged: false,
        }],
        remote_trust_key: None,
        is_done: false,
        should_exit: false,
    };
    let result = run_onboarding_screen(
        screen, /*app_server_request_handle*/ None, /*app_server*/ None, tui,
    )
    .await?;
    if result.should_exit {
        return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, StartupCancelled).into());
    }
    Ok(())
}

pub(crate) async fn run_onboarding_app(
    args: OnboardingScreenArgs,
    app_server: Option<&mut AppServerSession>,
    tui: &mut Tui,
) -> Result<OnboardingResult> {
    let request_handle = args.app_server_request_handle.clone();
    let screen = OnboardingScreen::new(tui, args).await;
    run_onboarding_screen(screen, request_handle, app_server, tui).await
}

async fn run_onboarding_screen(
    onboarding_screen: OnboardingScreen,
    app_server_request_handle: Option<AppServerRequestHandle>,
    app_server: Option<&mut AppServerSession>,
    tui: &mut Tui,
) -> Result<OnboardingResult> {
    // Let the terminal select and copy login URLs and device codes, even in fullscreen mode.
    let previous_mouse_capture = tui.mouse_capture;
    let result = match tui.set_mouse_capture(MouseCapture::Disabled) {
        Ok(()) => {
            run_onboarding_screen_inner(
                onboarding_screen,
                app_server_request_handle,
                app_server,
                tui,
            )
            .await
        }
        Err(error) => Err(error.into()),
    };
    let restore = tui.set_mouse_capture(previous_mouse_capture);
    result.and_then(|result| restore.map(|()| result).map_err(Into::into))
}

async fn run_onboarding_screen_inner(
    mut onboarding_screen: OnboardingScreen,
    app_server_request_handle: Option<AppServerRequestHandle>,
    mut app_server: Option<&mut AppServerSession>,
    tui: &mut Tui,
) -> Result<OnboardingResult> {
    use tokio_stream::StreamExt;
<<<<<<< HEAD

    let app_server_request_handle = args.app_server_request_handle.clone();
    let (claude_event_tx, mut claude_event_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut onboarding_screen =
        OnboardingScreen::new(tui, args, AppEventSender::new(claude_event_tx)).await;
||||||| 413492cd6c

    let app_server_request_handle = args.app_server_request_handle.clone();
    let mut onboarding_screen = OnboardingScreen::new(tui, args).await;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let mut directory_trust_persisted = false;
    // One-time guard to fully clear the screen after ChatGPT login success message is shown
    let mut did_full_clear_after_success = false;
    let mut pending_copy: Option<(u64, String)> = None;

    tui.draw(u16::MAX, |frame| {
        frame.render_widget_ref(&onboarding_screen, frame.area());
    })?;

    tui.discard_pending_input_before_interactive_screen()?;
    let tui_events = tui.event_stream();
    tokio::pin!(tui_events);
    let mut trust_step_was_active = onboarding_screen.is_trust_step_active();

    while !onboarding_screen.is_done() {
        tokio::select! {
            event = tui_events.next() => {
                if let Some(event) = event {
                    tui.screen_size_for_event(&event)?;
                    match event {
                        TuiEvent::Key(key_event) => {
                            let copy_link = if key_event.kind == KeyEventKind::Press
                                && keys::COPY_LINK.is_pressed(key_event)
                                && let Some(Step::Auth(widget)) = onboarding_screen.current_steps().last()
                                && let Ok(state) = widget.sign_in_state.read()
                                && let SignInState::ChatGptContinueInBrowser(state) = &*state
                                && !state.auth_url.is_empty()
                            {
                                Some((state.login_id.clone(), Arc::<str>::from(state.auth_url.as_str())))
                            } else {
                                None
                            };
                            if let Some((login_id, url)) = copy_link {
                                let result = tui.clipboard.copy(url, CopyFormat::PlainText, tui.frame_requester());
                                let message = match result {
                                    Ok(CopyStatus::Pending(id)) => {
                                        pending_copy = Some((id, login_id));
                                        "Copying link…".to_string()
                                    }
                                    Ok(CopyStatus::Busy) => "Clipboard busy; try again".to_string(),
                                    Ok(CopyStatus::Confirmed) => "Copied link to clipboard".to_string(),
                                    Ok(CopyStatus::Unconfirmed) => "Copy requested; check your clipboard".to_string(),
                                    Err(error) => format!("Could not copy link: {error}"),
                                };
                                if let Some(Step::Auth(widget)) = onboarding_screen.current_steps_mut().into_iter().last() {
                                    *widget.error.write().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(message);
                                }
                                tui.frame_requester().schedule_frame();
                            } else {
                                onboarding_screen.handle_key_event(key_event);
                            }
                            if !directory_trust_persisted {
                                directory_trust_persisted = persist_selected_trust(
                                    &mut onboarding_screen,
                                    app_server_request_handle.clone(),
                                )
                                .await;
                            }
                        }
                        TuiEvent::Paste(text) => {
                            onboarding_screen.handle_paste(text);
                        }
                        TuiEvent::Draw
                        | TuiEvent::Resume
                        | TuiEvent::Resize(_)
                        | TuiEvent::FocusGained
                        | TuiEvent::FocusLost => {
                            if let Some((id, result)) = tui.clipboard.poll().cloned()
                                && let Some((pending_id, login_id)) = &pending_copy
                                && id == *pending_id
                            {
                                if let Some(Step::Auth(widget)) = onboarding_screen.current_steps_mut().into_iter().last()
                                    && widget.sign_in_state.read().is_ok_and(|state| {
                                        matches!(&*state, SignInState::ChatGptContinueInBrowser(state) if state.login_id == *login_id)
                                    })
                                {
                                    let message = match result {
                                        Ok(CopyStatus::Confirmed) => "Copied link to clipboard".to_string(),
                                        Ok(CopyStatus::Unconfirmed) => "Copy requested; check your clipboard".to_string(),
                                        Ok(CopyStatus::Busy | CopyStatus::Pending(_)) => "Clipboard busy; try again".to_string(),
                                        Err(error) => format!("Could not copy link: {error}"),
                                    };
                                    *widget.error.write().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(message);
                                }
                                pending_copy = None;
                            }
                            for step in &onboarding_screen.steps {
                                if let Step::Welcome(widget) = step {
                                    if matches!(&event, TuiEvent::Resume) {
                                        widget.set_presentation(crate::empty_state_animation::Presentation::Hidden);
                                    }
                                    widget.set_focused(tui.is_terminal_focused());
                                }
                            }
                            if !did_full_clear_after_success
                                && onboarding_screen.steps.iter().any(|step| {
                                    if let Step::Auth(w) = step {
                                        w.sign_in_state.read().is_ok_and(|g| {
                                            matches!(&*g, super::auth::SignInState::ChatGptSuccessMessage)
                                        })
                                    } else {
                                        false
                                    }
                                })
                            {
                                // Reset any lingering SGR (underline/color) before clearing
                                let _ = ratatui::crossterm::execute!(
                                    std::io::stdout(),
                                    ratatui::crossterm::style::SetAttribute(
                                        ratatui::crossterm::style::Attribute::Reset
                                    ),
                                    ratatui::crossterm::style::SetAttribute(
                                        ratatui::crossterm::style::Attribute::NoUnderline
                                    ),
                                    ratatui::crossterm::style::SetForegroundColor(
                                        ratatui::crossterm::style::Color::Reset
                                    ),
                                    ratatui::crossterm::style::SetBackgroundColor(
                                        ratatui::crossterm::style::Color::Reset
                                    )
                                );
                                let _ = tui.terminal.clear();
                                did_full_clear_after_success = true;
                            }
                            tui.draw(u16::MAX, |frame| {
                                frame.render_widget_ref(&onboarding_screen, frame.area());
                            })?;
                        }
                        TuiEvent::Mouse(_) => {}
                    }
                } else {
                    return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, StartupCancelled).into());
                }
            }
            event = async {
                match app_server.as_mut() {
                    Some(app_server) => app_server.next_event().await,
                    None => None,
                }
            }, if app_server.is_some() => {
                if let Some(event) = event {
                    match event.event {
                        AppServerEvent::ServerNotification(notification) => {
                            onboarding_screen.handle_app_server_notification(*notification);
                        }
                        AppServerEvent::Disconnected { message } => {
                            return Err(color_eyre::eyre::eyre!(message));
                        }
                        AppServerEvent::Lagged { .. }
                        | AppServerEvent::ServerRequest(_) => {}
                    }
                }
            }
            event = claude_event_rx.recv() => {
                if let Some(event) = event {
                    onboarding_screen.handle_claude_event(event);
                }
            }
            action = onboarding_screen.provider_auth_action_rx.recv() => {
                if let Some(action) = action {
                    onboarding_screen.handle_provider_auth_action(action);
                }
            }
        }

        discard_pending_input_on_trust_step_transition(
            &onboarding_screen,
            &mut trust_step_was_active,
            |onboarding_screen| {
                tui.draw(u16::MAX, |frame| {
                    frame.render_widget_ref(onboarding_screen, frame.area());
                })?;
                tui.discard_pending_input_before_interactive_screen()?;
                Ok(())
            },
        )?;
    }
    // `TuiEventStream` shares the broker-owned crossterm source. The onboarding stream can leave
    // that source registered with its now-stale task waker when this sequential screen exits.
    // Recreate the source before App installs its own stream so the painted composer also accepts
    // keyboard input without requiring a process restart.
    tui.resume_events();
    Ok(OnboardingResult {
        directory_trust_persisted,
        should_exit: onboarding_screen.should_exit(),
        configured_provider: onboarding_screen.configured_provider(),
        deferred_provider_setup: onboarding_screen.deferred_provider_setup(),
    })
}

<<<<<<< HEAD
fn openai_metadata(login_status: LoginStatus) -> codex_login::OpenAiAuthMetadata {
    match login_status {
        LoginStatus::NotAuthenticated => codex_login::OpenAiAuthMetadata::Missing,
        LoginStatus::AuthMode(mode) if mode.has_chatgpt_account() => {
            codex_login::OpenAiAuthMetadata::Account
        }
        LoginStatus::AuthMode(codex_protocol::auth::AuthMode::ApiKey) => {
            codex_login::OpenAiAuthMetadata::ApiKey
        }
        LoginStatus::AuthMode(codex_protocol::auth::AuthMode::Headers)
        | LoginStatus::AuthMode(codex_protocol::auth::AuthMode::AgentIdentity) => {
            codex_login::OpenAiAuthMetadata::ExternallyManaged
        }
        LoginStatus::AuthMode(_) => codex_login::OpenAiAuthMetadata::Unsupported,
    }
||||||| 413492cd6c
=======
/// Render a newly activated trust step before discarding keys from the preceding screen.
///
/// Once the step is active, preserve fresh navigation and confirmation input.
fn discard_pending_input_on_trust_step_transition(
    onboarding_screen: &OnboardingScreen,
    trust_step_was_active: &mut bool,
    render_and_discard: impl FnOnce(&OnboardingScreen) -> Result<()>,
) -> Result<()> {
    let trust_step_is_active = onboarding_screen.is_trust_step_active();
    if trust_step_is_active && !*trust_step_was_active {
        render_and_discard(onboarding_screen)?;
    }
    *trust_step_was_active = trust_step_is_active;
    Ok(())
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

async fn persist_selected_trust(
    onboarding_screen: &mut OnboardingScreen,
    request_handle: Option<AppServerRequestHandle>,
) -> bool {
    let Some((trust_step_index, trust_target)) = onboarding_screen
        .steps
        .iter()
        .enumerate()
        .find_map(|(index, step)| {
            if let Step::TrustDirectory(widget) = step
                && widget.selection == Some(TrustDirectorySelection::Trust)
                && !widget.restricted
            {
                return Some((index, widget.trust_target.clone()));
            }
            None
        })
    else {
        return false;
    };

    let result = match (
        request_handle,
        onboarding_screen.remote_trust_key.as_deref(),
    ) {
        (Some(request_handle), Some(project_key)) => {
            let project_key = project_key.replace('\\', "\\\\").replace('"', "\\\"");
            request_handle
                .request_typed::<serde_json::Value>(ClientRequest::ConfigBatchWrite {
                    request_id: RequestId::String(format!(
                        "tui-project-trust-write-{}",
                        Uuid::new_v4()
                    )),
                    params: ConfigBatchWriteParams {
                        edits: vec![replace_config_value(
                            format!("projects.\"{project_key}\".trust_level"),
                            serde_json::json!("trusted"),
                        )],
                        file_path: None,
                        expected_version: None,
                        reload_user_config: true,
                    },
                })
                .await
                .map(|_| ())
                .wrap_err("config/batchWrite failed while persisting remote project trust")
        }
        (Some(request_handle), None) => write_trusted_project(request_handle, &trust_target)
            .await
            .map(|_| ()),
        (None, _) => Err(color_eyre::eyre::eyre!("app server unavailable")),
    };

    match result {
        Ok(()) => true,
        Err(error) => {
            let error = format_config_error(&error);
            tracing::error!(
                "failed to persist trusted project state for {}: {error}",
                trust_target.display()
            );
            if let Step::TrustDirectory(widget) = &mut onboarding_screen.steps[trust_step_index] {
                widget.selection = None;
                widget.error = Some(format!(
                    "Failed to set trust for {}: {error}",
                    trust_target.display()
                ));
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
<<<<<<< HEAD
    use super::KeyboardHandler;
||||||| 413492cd6c
    use super::ApiKeyEntryContext;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    use super::OnboardingScreen;
    use super::Step;
    use super::StepState;
    use super::StepStateProvider;
<<<<<<< HEAD
    use super::initial_sign_in_option;
    use super::is_text_key_in_text_field;
||||||| 413492cd6c
=======
    use super::TextEntryContext;
    use super::discard_pending_input_on_trust_step_transition;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    use super::persist_selected_trust;
<<<<<<< HEAD
    use super::provider_api_key_options;
    use super::visible_step_indices;
    use crate::onboarding::auth::SignInOption;
    use crate::onboarding::auth::catalog_sign_in_options;
||||||| 413492cd6c
    use super::suppress_quit_while_typing_api_key;
=======
    use super::suppress_quit_while_typing;
    use crate::onboarding::onboarding_screen::KeyboardHandler;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    use crate::onboarding::trust_directory::TrustDirectorySelection;
    use crate::onboarding::trust_directory::TrustDirectoryWidget;
    use crate::provider_status_host::ProviderStatusHost;
    use crate::tui::FrameRequester;
    use codex_protocol::config_types::ForcedLoginMethod;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
<<<<<<< HEAD
    fn printable_quit_key_is_text_in_a_text_field_even_when_empty() {
        // #461: an empty field must not quit either, or a value that starts
        // with `q` ends the session and the rest reaches the shell.
        assert!(is_text_key_in_text_field(
||||||| 413492cd6c
    fn suppresses_printable_quit_key_during_api_key_entry() {
        let suppressed = suppress_quit_while_typing_api_key(
=======
    fn suppresses_printable_quit_key_during_api_key_entry() {
        let suppressed = suppress_quit_while_typing(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
<<<<<<< HEAD
            /*text_entry_active*/ true,
        ));
        assert!(is_text_key_in_text_field(
            KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT),
            /*text_entry_active*/ true,
        ));
||||||| 413492cd6c
            ApiKeyEntryContext {
                active: true,
                has_text: true,
            },
        );
        assert!(suppressed);
=======
            TextEntryContext {
                active: true,
                has_text: true,
            },
        );
        assert!(suppressed);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    #[test]
<<<<<<< HEAD
    fn control_quit_chords_still_quit_from_a_text_field() {
        for key in ['c', 'd'] {
            assert!(!is_text_key_in_text_field(
                KeyEvent::new(KeyCode::Char(key), KeyModifiers::CONTROL),
                /*text_entry_active*/ true,
            ));
        }
    }

    #[test]
    fn printable_quit_key_is_a_shortcut_outside_text_fields() {
        assert!(!is_text_key_in_text_field(
||||||| 413492cd6c
    fn does_not_suppress_printable_quit_key_when_api_key_input_is_empty() {
        let suppressed = suppress_quit_while_typing_api_key(
=======
    fn does_not_suppress_printable_quit_key_when_api_key_input_is_empty() {
        let suppressed = suppress_quit_while_typing(
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
<<<<<<< HEAD
            /*text_entry_active*/ false,
        ));
||||||| 413492cd6c
            ApiKeyEntryContext {
                active: true,
                has_text: false,
            },
        );
        assert!(!suppressed);
=======
            TextEntryContext {
                active: true,
                has_text: false,
            },
        );
        assert!(!suppressed);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    #[test]
<<<<<<< HEAD
    fn forced_chatgpt_never_defaults_to_the_hidden_anthropic_option() {
        assert_eq!(
            initial_sign_in_option(
                &[SignInOption::AnthropicAccount],
                Some(ForcedLoginMethod::Chatgpt),
            ),
            SignInOption::ChatGpt
||||||| 413492cd6c
    fn does_not_suppress_control_quit_key_during_api_key_entry() {
        let suppressed = suppress_quit_while_typing_api_key(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            ApiKeyEntryContext {
                active: true,
                has_text: true,
            },
=======
    fn does_not_suppress_control_quit_key_during_api_key_entry() {
        let suppressed = suppress_quit_while_typing(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            TextEntryContext {
                active: true,
                has_text: true,
            },
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        );
    }

    #[test]
<<<<<<< HEAD
    fn forced_api_key_never_defaults_to_the_hidden_anthropic_option() {
        assert_eq!(
            initial_sign_in_option(
                &[SignInOption::AnthropicAccount],
                Some(ForcedLoginMethod::Api),
            ),
            SignInOption::ApiKey
        );
    }

    #[tokio::test]
    async fn catalog_surface_keeps_openai_alternatives_corbanu_deferred_and_custom_typed() {
        let home = tempfile::tempdir().unwrap();
        let mut config = crate::legacy_core::config::ConfigBuilder::default()
            .codex_home(home.path().to_path_buf())
            .build()
            .await
            .unwrap();
        config.model_providers.insert(
            "pf53-custom".to_string(),
            codex_model_provider_info::ModelProviderInfo {
                name: "PF53 Custom".to_string(),
                env_key: Some("PF53_CUSTOM_API_KEY".to_string()),
                ..Default::default()
||||||| 413492cd6c
    fn does_not_suppress_when_not_in_api_key_entry() {
        let suppressed = suppress_quit_while_typing_api_key(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            ApiKeyEntryContext {
                active: false,
                has_text: true,
=======
    fn does_not_suppress_when_not_in_api_key_entry() {
        let suppressed = suppress_quit_while_typing(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            TextEntryContext {
                active: false,
                has_text: true,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            },
        );
        let host = ProviderStatusHost::from_config(
            &config,
            crate::provider_status_host::ProviderAccountMetadata::default(),
        );
        let api_options = provider_api_key_options(&host);
        let visible = catalog_sign_in_options(&host, &api_options);

        assert!(matches!(visible.first(), Some(SignInOption::DeviceCode)));
        let Some(SignInOption::ProviderApiKey(openai_index)) = visible.get(1) else {
            panic!("OpenAI API key must immediately follow the account capability: {visible:?}");
        };
        assert_eq!(
            api_options[*openai_index].target.provider_id.as_str(),
            codex_model_provider_info::OPENAI_PROVIDER_ID
        );
        assert_eq!(
            api_options[*openai_index].target.storage,
            codex_provider_auth::ApiKeyStorage::OpenAiAuth
        );
        assert_eq!(
            visible
                .iter()
                .filter(|option| matches!(option, SignInOption::CorbanuPlan))
                .count(),
            1
        );
        assert!(!api_options.iter().any(|option| {
            option.target.provider_id.as_str()
                == codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID
        }));

        let custom = api_options
            .iter()
            .find(|option| option.target.provider_id.as_str() == "pf53-custom")
            .expect("custom catalog API-key option");
        assert_eq!(custom.target.runtime_provider_id.as_str(), "pf53-custom");
        assert_eq!(
            custom.target.storage,
            codex_provider_auth::ApiKeyStorage::EnvironmentVariable {
                env_key: "PF53_CUSTOM_API_KEY".to_string(),
            }
        );
        assert_eq!(
            custom.capability,
            codex_provider_auth::ProviderSetupCapability::ApiKey {
                storage: codex_provider_auth::ApiKeyStorage::EnvironmentVariable {
                    env_key: "PF53_CUSTOM_API_KEY".to_string(),
                },
            }
        );
    }

    #[test]
    fn onboarding_keeps_welcome_and_the_current_required_step_visible() {
        assert_eq!(
            visible_step_indices(
                &[
                    StepState::Complete,
                    StepState::Complete,
                    StepState::InProgress,
                ],
                Some(0),
            ),
            vec![0, 2],
            "a completed auth panel must not hide the active trust step"
        );
        assert_eq!(
            visible_step_indices(&[StepState::Complete, StepState::InProgress], Some(0),),
            vec![0, 1],
            "welcome and active auth should remain visible together"
        );
        assert_eq!(
            visible_step_indices(
                &[
                    StepState::Hidden,
                    StepState::Complete,
                    StepState::InProgress
                ],
                Some(0),
            ),
            vec![2],
            "hidden welcome and completed intermediate steps must not consume the viewport"
        );
        assert_eq!(
            visible_step_indices(&[StepState::Complete, StepState::Complete], Some(0),),
            vec![0, 1],
            "the final completed frame remains renderable before onboarding exits"
        );
    }

    #[test]
    fn buffered_enter_is_discarded_when_trust_step_becomes_active() {
        let mut onboarding_screen = OnboardingScreen {
            request_frame: FrameRequester::test_dummy(),
            steps: vec![Step::TrustDirectory(TrustDirectoryWidget {
                restricted: false,
                existing_task: false,
                cancel: super::super::trust_directory::TrustCancelAction::Quit,
                cwd: PathBuf::from("/workspace/project"),
                trust_target: PathBuf::from("/workspace/project"),
                show_windows_create_sandbox_hint: false,
                should_quit: false,
                selection: None,
                highlighted: TrustDirectorySelection::Trust,
                error: None,
            })],
            remote_trust_key: None,
            is_done: false,
            should_exit: false,
        };

        let mut buffered_keys = vec![KeyEvent::from(KeyCode::Enter)];
        let mut trust_step_was_active = false;
        let mut drain_count = 0;
        discard_pending_input_on_trust_step_transition(
            &onboarding_screen,
            &mut trust_step_was_active,
            |_| {
                drain_count += 1;
                buffered_keys.clear();
                Ok(())
            },
        )
        .expect("discard buffered trust input");
        for key in buffered_keys {
            onboarding_screen.handle_key_event(key);
        }

        assert!(onboarding_screen.is_trust_step_active());
        let Step::TrustDirectory(widget) = &onboarding_screen.steps[0] else {
            panic!("trust step should remain present");
        };
        assert_eq!(widget.selection, None);
        assert_eq!(widget.highlighted, TrustDirectorySelection::Trust);

        let mut navigation_keys = vec![KeyEvent::from(KeyCode::Down)];
        discard_pending_input_on_trust_step_transition(
            &onboarding_screen,
            &mut trust_step_was_active,
            |_| {
                drain_count += 1;
                navigation_keys.clear();
                Ok(())
            },
        )
        .expect("retain fresh trust navigation");
        assert_eq!(drain_count, 1);
        onboarding_screen.handle_key_event(navigation_keys.remove(0));
        let Step::TrustDirectory(widget) = &onboarding_screen.steps[0] else {
            panic!("trust step should remain present");
        };
        assert_eq!(widget.highlighted, TrustDirectorySelection::Quit);
        onboarding_screen.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(onboarding_screen.should_exit());
    }

    #[tokio::test]
    async fn restricted_acceptance_and_cancellation_do_not_persist_trust() {
        for key in [KeyCode::Enter, KeyCode::Esc] {
            let mut screen = OnboardingScreen {
                request_frame: FrameRequester::test_dummy(),
                steps: vec![Step::TrustDirectory(TrustDirectoryWidget {
                    restricted: true,
                    existing_task: false,
                    cancel: super::super::trust_directory::TrustCancelAction::AgentsOverview,
                    cwd: PathBuf::from("/workspace/project"),
                    trust_target: PathBuf::from("/workspace/project"),
                    show_windows_create_sandbox_hint: false,
                    should_quit: false,
                    selection: None,
                    highlighted: TrustDirectorySelection::Trust,
                    error: None,
                })],
                remote_trust_key: Some("/workspace/project".to_string()),
                is_done: false,
                should_exit: false,
            };
            screen.handle_key_event(key.into());
            assert!(!persist_selected_trust(&mut screen, /*request_handle*/ None).await);
            assert!(screen.is_done());
            assert_eq!(screen.should_exit(), key == KeyCode::Esc);
            let Step::TrustDirectory(widget) = &screen.steps[0] else {
                panic!("trust step")
            };
            assert_eq!(widget.error, None);
        }
    }

    #[tokio::test]
    async fn trust_persistence_failure_keeps_trust_step_in_progress() {
        let (_provider_auth_action_tx, provider_auth_action_rx) =
            tokio::sync::mpsc::unbounded_channel();
        let mut onboarding_screen = OnboardingScreen {
            request_frame: FrameRequester::test_dummy(),
            steps: vec![Step::TrustDirectory(TrustDirectoryWidget {
                restricted: false,
                existing_task: false,
                cancel: super::super::trust_directory::TrustCancelAction::Quit,
                cwd: PathBuf::from("/workspace/project"),
                trust_target: PathBuf::from("/workspace/project"),
                show_windows_create_sandbox_hint: false,
                should_quit: false,
                selection: Some(TrustDirectorySelection::Trust),
                highlighted: TrustDirectorySelection::Trust,
                error: None,
            })],
            remote_trust_key: None,
            is_done: false,
            should_exit: false,
            provider_auth_controller: codex_provider_auth::ProviderAuthController::default(),
            provider_auth_executor: None,
            provider_auth_action_rx,
        };

        let persisted =
            persist_selected_trust(&mut onboarding_screen, /*request_handle*/ None).await;

        assert!(!persisted);
        let Step::TrustDirectory(widget) = &onboarding_screen.steps[0] else {
            panic!("trust step should remain present");
        };
        assert_eq!(widget.selection, None);
        assert_eq!(widget.get_step_state(), super::StepState::InProgress);
        assert!(
            widget
                .error
                .as_deref()
                .is_some_and(|error| error.contains("app server unavailable"))
        );
    }

    fn auth_screen(widget: crate::onboarding::auth::AuthModeWidget) -> OnboardingScreen {
        let (_provider_auth_action_tx, provider_auth_action_rx) =
            tokio::sync::mpsc::unbounded_channel();
        OnboardingScreen {
            request_frame: FrameRequester::test_dummy(),
            steps: vec![Step::Auth(widget)],
            is_done: false,
            should_exit: false,
            provider_auth_controller: codex_provider_auth::ProviderAuthController::default(),
            provider_auth_executor: None,
            provider_auth_action_rx,
        }
    }

    /// #461: every onboarding text field takes `q` as text, from the first
    /// character on; nothing typed ends onboarding.
    #[tokio::test]
    async fn every_onboarding_text_field_types_q_instead_of_quitting() {
        use crate::onboarding::auth::SignInState;

        let typed = "qprobe-q-key-Q";
        let (claude_input_tx, _claude_input_rx) = tokio::sync::mpsc::unbounded_channel();
        let states = [
            ("API key", SignInState::ApiKeyEntry(Default::default())),
            (
                "Claude token",
                SignInState::ClaudeManagedTokenEntry(Default::default()),
            ),
            (
                "Claude code",
                SignInState::ClaudeCodeLoginCodeEntry {
                    input_tx: claude_input_tx,
                    value: String::new(),
                },
            ),
            (
                "account name",
                SignInState::AccountNameEntry {
                    entry_index: 0,
                    value: String::new(),
                },
            ),
            (
                "account value",
                SignInState::AccountValueEntry {
                    entry_index: 0,
                    name: codex_vault::ProviderAccountName::parse("probe").expect("name"),
                    value: zeroize::Zeroizing::new(String::new()),
                },
            ),
        ];
        for (field, state) in states {
            let (widget, _home) = crate::onboarding::auth::tests::widget_forced_chatgpt().await;
            *widget.sign_in_state.write().unwrap() = state;
            let mut screen = auth_screen(widget);
            for character in typed.chars() {
                let modifiers = if character.is_ascii_uppercase() {
                    KeyModifiers::SHIFT
                } else {
                    KeyModifiers::NONE
                };
                screen.handle_key_event(KeyEvent::new(KeyCode::Char(character), modifiers));
                assert!(
                    !screen.is_done() && !screen.should_exit(),
                    "{field}: {character:?} ended onboarding"
                );
            }
            let Step::Auth(widget) = &screen.steps[0] else {
                panic!("auth step");
            };
            assert_eq!(widget.text_entry_value().as_deref(), Some(typed), "{field}");
        }
    }

    /// A non-bracketed paste with a newline starts the save while the rest of
    /// the value is still arriving; those keys must not quit either.
    #[tokio::test]
    async fn saving_a_typed_value_does_not_quit_on_q() {
        use crate::onboarding::auth::SignInState;

        for state in [
            SignInState::ApiKeySaving,
            SignInState::ClaudeManagedTokenSaving,
            SignInState::AccountSaving,
        ] {
            let (widget, _home) = crate::onboarding::auth::tests::widget_forced_chatgpt().await;
            *widget.sign_in_state.write().unwrap() = state;
            let mut screen = auth_screen(widget);
            screen.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
            assert!(!screen.is_done() && !screen.should_exit());
        }
    }

    /// Ctrl+C still leaves onboarding from a text field.
    #[tokio::test]
    async fn ctrl_c_still_quits_from_an_onboarding_text_field() {
        use crate::onboarding::auth::SignInState;

        let (widget, _home) = crate::onboarding::auth::tests::widget_forced_chatgpt().await;
        *widget.sign_in_state.write().unwrap() = SignInState::AccountNameEntry {
            entry_index: 0,
            value: String::new(),
        };
        let mut screen = auth_screen(widget);
        screen.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(screen.is_done());
    }

    /// Outside a text field, `q` is still the quit shortcut.
    #[tokio::test]
    async fn q_still_quits_from_the_provider_list() {
        let (widget, _home) = crate::onboarding::auth::tests::widget_forced_chatgpt().await;
        let mut screen = auth_screen(widget);
        screen.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(screen.is_done());
    }

    /// #462: when config already names the provider, onboarding opens on its
    /// key entry. Saving must complete that provider's setup, not loop back.
    #[tokio::test]
    async fn preset_provider_key_entry_completes_setup_for_that_provider() {
        use crate::onboarding::auth::SignInState;
        use crate::onboarding::provider_setup::ProviderSetupAction;
        use crate::onboarding::provider_setup::ProviderSetupPhase;

        let (mut widget, _home) = crate::onboarding::auth::tests::widget_forced_chatgpt().await;
        widget.forced_login_method = None;
        let options = provider_api_key_options(&widget.provider_status_host);
        let kimi = options
            .iter()
            .find(|option| option.target.provider_id.as_str() != "openai")
            .expect("a non-OpenAI API-key provider")
            .clone();
        widget.api_key_provider_options = vec![kimi.clone()];
        widget.selected_api_key_option = Some(kimi.clone());
        *widget.sign_in_state.write().unwrap() = SignInState::ApiKeyEntry(Default::default());

        widget.begin_initial_api_key_entry();

        let entry = widget
            .provider_status_host
            .catalog()
            .get(kimi.target.provider_id.as_str())
            .expect("catalog entry");
        assert_eq!(
            widget
                .provider_setup_session
                .read()
                .unwrap()
                .snapshot()
                .phase,
            ProviderSetupPhase::Authenticating {
                provider_id: entry.id.clone()
            }
        );
        let configured = widget.provider_setup_session.write().unwrap().dispatch(
            ProviderSetupAction::AuthConfigured {
                provider_id: entry.id.clone(),
                runtime_provider_id: entry.runtime_provider_ids.first().expect("runtime").clone(),
            },
        );
        assert!(configured.applied, "the saved key's completion was ignored");
    }
}
