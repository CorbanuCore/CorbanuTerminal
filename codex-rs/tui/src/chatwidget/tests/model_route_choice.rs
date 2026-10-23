//! #653: `/model` offers each connection of a model served by several.

use super::*;
use codex_model_provider_info::ZAI_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::ZAI_PROVIDER_ID;
use pretty_assertions::assert_eq;

const FLASH: &str = "glm-5.3-flash";
const OPENAI_STYLE: &str = "GLM 5.3 Flash · Z.AI (OpenAI-style)";
const CLAUDE_STYLE: &str = "GLM 5.3 Flash · Z.AI (Claude-style)";

async fn zai_session(
    provider: &str,
) -> (
    ChatWidget,
    tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    tokio::sync::mpsc::UnboundedReceiver<Op>,
) {
    let (mut chat, rx, op_rx) = make_chatwidget_manual(Some(FLASH)).await;
    chat.config.model_provider_id = provider.to_string();
    chat.thread_id = Some(ThreadId::new());
    (chat, rx, op_rx)
}

fn open_all_models(chat: &mut ChatWidget) -> String {
    let presets = chat.model_catalog.try_list_models().expect("catalog");
    chat.open_all_models_popup(presets);
    render_bottom_popup_with_height(chat, /*width*/ 110, /*height*/ 36)
}

/// Opens the picker the way `/model` does, filtering the catalog first.
fn open_model_command(chat: &mut ChatWidget) -> String {
    let presets = chat.model_catalog.try_list_models().expect("catalog");
    chat.open_model_popup_with_presets(presets);
    render_bottom_popup_with_height(chat, /*width*/ 110, /*height*/ 36)
}

/// Selects the row named `label` in the open tab and returns the preset the
/// picker sends to the effort step.
fn select_row(
    chat: &mut ChatWidget,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    label: &str,
) -> ModelPreset {
    for _ in 0..12 {
        let popup = render_bottom_popup_with_height(chat, /*width*/ 110, /*height*/ 36);
        if popup
            .lines()
            .any(|line| line.contains('›') && line.contains(label))
        {
            chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
            return std::iter::from_fn(|| rx.try_recv().ok())
                .find_map(|event| match event {
                    AppEvent::OpenReasoningPopup { model, .. } => Some(model),
                    _ => None,
                })
                .expect("a model row opens the effort step");
        }
        chat.handle_key_event(KeyEvent::from(KeyCode::Down));
    }
    panic!("no row {label}");
}

fn chosen(events: &[AppEvent]) -> (Option<String>, Option<String>) {
    let update = events.iter().find_map(|event| match event {
        AppEvent::UpdateModelSelection { model, provider } if model == FLASH => provider.clone(),
        _ => None,
    });
    let persist = events.iter().find_map(|event| match event {
        AppEvent::PersistModelSelection {
            model, provider, ..
        } if model == FLASH => provider.clone(),
        _ => None,
    });
    (update, persist)
}

#[tokio::test]
async fn zai_tab_lists_both_connections_openai_style_first() {
    let (mut chat, _rx, _op_rx) = zai_session(ZAI_PROVIDER_ID).await;
    let popup = open_all_models(&mut chat);
    let openai = popup.find(OPENAI_STYLE).expect(OPENAI_STYLE);
    let claude = popup.find(CLAUDE_STYLE).expect(CLAUDE_STYLE);
    assert!(openai < claude, "{popup}");
    assert!(
        popup.contains(&format!("{OPENAI_STYLE} (current)")),
        "{popup}"
    );
    assert_eq!(popup.matches("(current)").count(), 1, "{popup}");
    assert!(!popup.contains("via zai-anthropic"), "{popup}");
    assert_chatwidget_snapshot!("model_route_choice_zai_tab", popup);
}

#[tokio::test]
async fn a_claude_style_session_opens_on_its_row_in_the_zai_tab() {
    let (mut chat, _rx, _op_rx) = zai_session(ZAI_ANTHROPIC_PROVIDER_ID).await;
    // The runtime row the app adds for the session's connection is folded in.
    crate::model_catalog::ModelCatalog::sync_runtime_models(
        &chat.model_catalog,
        [ZAI_ANTHROPIC_PROVIDER_ID],
        Some(FLASH),
        ZAI_ANTHROPIC_PROVIDER_ID,
    );
    let popup = open_all_models(&mut chat);
    assert!(popup.contains("[Z.AI]"), "{popup}");
    assert!(
        !popup.contains("[Other]") && !popup.contains("Other"),
        "{popup}"
    );
    assert!(
        popup.contains(&format!("{CLAUDE_STYLE} (current)")),
        "{popup}"
    );
    assert_eq!(popup.matches("(current)").count(), 1, "{popup}");
    assert_eq!(popup.matches(CLAUDE_STYLE).count(), 1, "{popup}");
    assert_chatwidget_snapshot!("model_route_choice_claude_style_current", popup);
}

#[tokio::test]
async fn picking_a_connection_switches_and_saves_that_provider_both_ways() {
    for (from, label, to) in [
        (ZAI_PROVIDER_ID, CLAUDE_STYLE, ZAI_ANTHROPIC_PROVIDER_ID),
        (ZAI_ANTHROPIC_PROVIDER_ID, OPENAI_STYLE, ZAI_PROVIDER_ID),
    ] {
        let (mut chat, mut rx, _op_rx) = zai_session(from).await;
        chat.set_reasoning_effort(Some(ReasoningEffortConfig::High));
        open_all_models(&mut chat);
        let preset = select_row(&mut chat, &mut rx, label);
        assert_eq!(preset.model, FLASH);
        assert_eq!(preset.provider_id.as_deref(), Some(to));

        chat.open_reasoning_popup(preset);
        let effort = render_bottom_popup(&chat, /*width*/ 100);
        assert!(effort.contains(label), "{effort}");
        // Another connection of the session's model is not the current choice.
        assert!(!effort.contains("(current)"), "{effort}");
        chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
        let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
        assert_eq!(
            chosen(&events),
            (Some(to.to_string()), Some(to.to_string())),
            "{from} -> {to}"
        );
        // The session's effort is kept.
        assert!(
            events.iter().any(|event| matches!(
                event,
                AppEvent::PersistModelSelection {
                    effort: Some(ReasoningEffortConfig::High),
                    ..
                }
            )),
            "{events:?}"
        );
    }
}

#[tokio::test]
async fn only_available_connections_are_offered() {
    use crate::provider_status_host::ProviderAccountMetadata;
    use crate::provider_status_host::ProviderStatusHost;
    use codex_provider_auth::ProviderActivationPolicy;
    use codex_provider_auth::ProviderRuntimeAuthorizations;

    let (mut chat, _rx, _op_rx) = zai_session(ZAI_PROVIDER_ID).await;
    let home = tempfile::tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.cli_auth_credentials_store_mode = codex_login::AuthCredentialsStoreMode::File;
    config.model_provider_id = ZAI_PROVIDER_ID.to_string();
    codex_login::login_with_provider_api_key(
        home.path(),
        "ZAI_API_KEY",
        "synthetic-route-choice-key",
        config.cli_auth_credentials_store_mode,
        config.auth_keyring_backend_kind(),
    )
    .unwrap();
    let host = ProviderStatusHost::from_config(&config, ProviderAccountMetadata::default());
    chat.model_catalog.set_provider_policy(
        super::super::provider_model_policy::ProviderModelPolicy::new(
            host.clone(),
            ProviderRuntimeAuthorizations::default(),
        ),
    );
    let both = open_all_models(&mut chat);
    assert!(
        both.contains(OPENAI_STYLE) && both.contains(CLAUDE_STYLE),
        "{both}"
    );

    // Both connections use the Z.AI key, so /providers turns them off together.
    chat.handle_key_event(KeyEvent::from(KeyCode::Esc));
    host.persist_policy(ZAI_PROVIDER_ID, ProviderActivationPolicy::Inactive)
        .unwrap();
    chat.model_catalog.refresh_provider_policy();
    let none = open_all_models(&mut chat);
    assert!(
        !none.contains("OpenAI-style") && !none.contains("Claude-style"),
        "{none}"
    );

    // Nor is one this session has no provider entry for.
    chat.handle_key_event(KeyEvent::from(KeyCode::Esc));
    host.persist_policy(ZAI_PROVIDER_ID, ProviderActivationPolicy::Active)
        .unwrap();
    chat.model_catalog.refresh_provider_policy();
    chat.config
        .model_providers
        .remove(ZAI_ANTHROPIC_PROVIDER_ID);
    let unconfigured = open_all_models(&mut chat);
    assert!(
        unconfigured.contains("Z.AI GLM 5.3 Flash (current)"),
        "{unconfigured}"
    );
    assert!(!unconfigured.contains("-style"), "{unconfigured}");
}

/// #665: with the connections on different accounts, each is listed only when
/// its own credential is there: the named account's enrollment, or the
/// default Z.AI key for a connection without an account.
#[tokio::test]
async fn connections_on_different_accounts_are_offered_only_with_their_own_credential() {
    for (named, named_row, default_row) in [
        (ZAI_PROVIDER_ID, OPENAI_STYLE, CLAUDE_STYLE),
        (ZAI_ANTHROPIC_PROVIDER_ID, CLAUDE_STYLE, OPENAI_STYLE),
    ] {
        // Enrolled named account, no default key: only its connection.
        let (mut chat, _rx, _op_rx, home) = mixed_account_session(named, /*enrolled*/ true).await;
        let popup = open_all_models(&mut chat);
        assert!(popup.contains(named_row), "{named}: {popup}");
        assert!(!popup.contains(default_row), "{named}: {popup}");

        // With a default Z.AI key stored, the other connection can sign in too.
        chat.handle_key_event(KeyEvent::from(KeyCode::Esc));
        store_default_zai_key(&chat, home.path());
        chat.model_catalog.refresh_provider_policy();
        let popup = open_all_models(&mut chat);
        assert!(
            popup.contains(named_row) && popup.contains(default_row),
            "{named}: {popup}"
        );

        // A default key and an unenrolled named account: only the default
        // account's connection, for every GLM model `/model` lists.
        let (mut chat, _rx, _op_rx, home) = mixed_account_session(named, /*enrolled*/ false).await;
        store_default_zai_key(&chat, home.path());
        chat.model_catalog.refresh_provider_policy();
        let popup = open_model_command(&mut chat);
        assert!(popup.contains(default_row), "{named}: {popup}");
        assert!(!popup.contains(named_row), "{named}: {popup}");
        let other_model_row = default_row.replace("GLM 5.3 Flash", "GLM 5.3");
        assert!(popup.contains(&other_model_row), "{named}: {popup}");
    }
}

/// A Z.AI session whose connection `named` runs on account `work` (enrolled
/// or not) and the other on the default account, with no Z.AI key in the
/// environment. Returns the chat and the home holding its credentials.
async fn mixed_account_session(
    named: &str,
    enrolled: bool,
) -> (
    ChatWidget,
    tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    tokio::sync::mpsc::UnboundedReceiver<Op>,
    tempfile::TempDir,
) {
    use crate::provider_status_host::ProviderAccountMetadata;
    use crate::provider_status_host::ProviderStatusHost;
    use codex_provider_auth::ProviderRuntimeAuthorizations;

    let (mut chat, rx, op_rx) = zai_session(named).await;
    let home = tempfile::tempdir().unwrap();
    let mut config = crate::legacy_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .build()
        .await
        .unwrap();
    config.cli_auth_credentials_store_mode = codex_login::AuthCredentialsStoreMode::File;
    config.model_provider_id = named.to_string();
    let account = codex_model_provider_info::NamedProviderAccount {
        provider_id: named.to_string(),
        name: "work".to_string(),
    };
    for providers in [
        &mut config.model_providers,
        &mut chat.config.model_providers,
    ] {
        providers.get_mut(named).expect(named).account = Some(account.clone());
    }
    let host = ProviderStatusHost::from_config_with_account_registry(
        &config,
        ProviderAccountMetadata::default(),
        move |held, kind| {
            enrolled && held == &account && kind == codex_login::ProviderAccountKind::ApiKey
        },
    )
    .with_environment(|_| codex_provider_auth::EnvironmentCredentialMetadata::Missing);
    chat.model_catalog.set_provider_policy(
        super::super::provider_model_policy::ProviderModelPolicy::new(
            host,
            ProviderRuntimeAuthorizations::default(),
        ),
    );
    (chat, rx, op_rx, home)
}

fn store_default_zai_key(chat: &ChatWidget, home: &std::path::Path) {
    codex_login::login_with_provider_api_key(
        home,
        "ZAI_API_KEY",
        "synthetic-route-choice-key",
        codex_login::AuthCredentialsStoreMode::File,
        chat.config.auth_keyring_backend_kind(),
    )
    .unwrap();
}

/// PF-84 accounts are per provider: connections on different accounts say so.
#[tokio::test]
async fn connections_on_different_named_accounts_name_them() {
    let (mut chat, _rx, _op_rx) = zai_session(ZAI_PROVIDER_ID).await;
    chat.config
        .model_providers
        .get_mut(ZAI_PROVIDER_ID)
        .expect("zai")
        .account = Some(codex_model_provider_info::NamedProviderAccount {
        provider_id: ZAI_PROVIDER_ID.to_string(),
        name: "work".to_string(),
    });
    let popup = open_all_models(&mut chat);
    assert!(
        popup.contains(&format!("{OPENAI_STYLE} · account work")),
        "{popup}"
    );
    assert!(
        popup.contains(&format!("{CLAUDE_STYLE} · default account")),
        "{popup}"
    );
}

/// Switching connection keeps the session's effort, even an advanced one:
/// chosen explicitly, or GLM 5.3 Flash's default (Max) when none was chosen.
#[tokio::test]
async fn switching_connection_keeps_the_sessions_max_effort() {
    for session_effort in [Some(ReasoningEffortConfig::Max), None] {
        switch_to_claude_style_keeping_max(session_effort).await;
    }
}

async fn switch_to_claude_style_keeping_max(session_effort: Option<ReasoningEffortConfig>) {
    let (mut chat, mut rx, _op_rx) = zai_session(ZAI_PROVIDER_ID).await;
    chat.set_reasoning_effort(session_effort);
    open_all_models(&mut chat);
    let preset = select_row(&mut chat, &mut rx, CLAUDE_STYLE);
    chat.open_reasoning_popup(preset);
    let effort = render_bottom_popup(&chat, /*width*/ 100);
    assert!(
        effort
            .lines()
            .any(|line| line.contains('›') && line.contains("More reasoning")),
        "{effort}"
    );
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    let advanced = std::iter::from_fn(|| rx.try_recv().ok())
        .find_map(|event| match event {
            AppEvent::OpenAdvancedReasoningPopup { model, .. } => Some(model),
            _ => None,
        })
        .expect("the advanced step opens");
    chat.open_advanced_reasoning_popup(advanced);
    let popup = render_bottom_popup(&chat, /*width*/ 100);
    assert!(!popup.contains("(current)"), "{popup}");
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
    assert_eq!(
        chosen(&events),
        (
            Some(ZAI_ANTHROPIC_PROVIDER_ID.to_string()),
            Some(ZAI_ANTHROPIC_PROVIDER_ID.to_string())
        )
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            AppEvent::PersistModelSelection {
                effort: Some(ReasoningEffortConfig::Max),
                ..
            }
        )),
        "{events:?}"
    );
}

/// A default model's connections stay together when the tab lifts it first,
/// and every connection row has its own id.
#[tokio::test]
async fn a_default_models_connections_stay_together_with_distinct_ids() {
    let (mut chat, _rx, _op_rx) = zai_session(ZAI_ANTHROPIC_PROVIDER_ID).await;
    crate::model_catalog::ModelCatalog::sync_runtime_models(
        &chat.model_catalog,
        [ZAI_ANTHROPIC_PROVIDER_ID],
        Some(FLASH),
        ZAI_ANTHROPIC_PROVIDER_ID,
    );
    let mut presets = chat.model_catalog.try_list_models().expect("catalog");
    for preset in &mut presets {
        preset.is_default = preset.model == "glm-5.2";
    }

    let rows = chat.with_model_route_rows(presets.clone());
    let zai_ids = rows
        .iter()
        .filter(|row| row.model.starts_with("glm-"))
        .map(|row| row.id.clone())
        .collect::<Vec<_>>();
    let unique = zai_ids.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), zai_ids.len(), "{zai_ids:?}");

    chat.open_all_models_popup(presets);
    let popup = render_bottom_popup_with_height(&chat, /*width*/ 110, /*height*/ 36);
    let position = |label: &str| {
        popup
            .find(label)
            .unwrap_or_else(|| panic!("{label}: {popup}"))
    };
    let openai = position("GLM 5.2 · Z.AI (OpenAI-style)");
    let claude = position("GLM 5.2 · Z.AI (Claude-style)");
    let flash = position(OPENAI_STYLE);
    assert!(openai < claude && claude < flash, "{popup}");
}
