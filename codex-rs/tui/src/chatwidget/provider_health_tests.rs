use super::*;
use codex_provider_auth::*;
use pretty_assertions::assert_eq;

#[test]
fn only_definite_typed_auth_rejections_require_reauthentication() {
    for status in [400, 401, 403, 404, 408, 429, 500, 503] {
        assert_eq!(
            is_credential_rejection(Some(&CodexErrorInfo::HttpConnectionFailed {
                http_status_code: Some(status),
            })),
            status == 401
        );
    }
    assert!(is_credential_rejection(Some(&CodexErrorInfo::Unauthorized)));
    assert!(!is_credential_rejection(None));
    assert!(!is_credential_rejection(Some(
        &CodexErrorInfo::ResponseStreamDisconnected {
            http_status_code: None,
        }
    )));
}

#[tokio::test]
async fn late_turn_failure_names_original_provider_and_ignores_recovered_attempt() {
    let (mut chat, _tx, mut rx, _op_rx) =
        super::super::tests::make_chatwidget_manual_with_sender().await;
    let host = crate::provider_status_host::ProviderStatusHost::from_config(
        chat.config_ref(),
        crate::provider_status_host::ProviderAccountMetadata {
            openai: codex_login::OpenAiAuthMetadata::Account,
            claude: ClaudeCredentialMetadata::Configured {
                source: ClaudeCredentialSource::Managed,
            },
            ..Default::default()
        },
    );
    chat.model_catalog.set_provider_policy(
        super::super::provider_model_policy::ProviderModelPolicy::new(
            host.clone(),
            ProviderRuntimeAuthorizations::default(),
        ),
    );
    host.begin_credential_attempt("turn:thread:old".into(), "openai");
    chat.config.model_provider_id = "claude-plan".into();
    while rx.try_recv().is_ok() {}
    let failure = |turn: &str| {
        ServerNotification::Error(codex_app_server_protocol::ErrorNotification {
            thread_id: "thread".into(),
            turn_id: turn.into(),
            will_retry: false,
            error: codex_app_server_protocol::TurnError {
                message: "synthetic rejection".into(),
                codex_error_info: Some(CodexErrorInfo::Unauthorized),
                additional_details: None,
            },
        })
    };
    chat.observe_provider_health(&failure("old"));
    let warnings: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok())
        .filter_map(|event| match event {
            crate::app_event::AppEvent::InsertHistoryCell(cell) => Some(
                cell.display_lines(200)
                    .into_iter()
                    .map(|line| line.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            _ => None,
        })
        .collect();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("OpenAI (openai) credential was rejected"));
    assert!(!warnings[0].contains("Claude"));
    host.credential_changed("openai");
    host.begin_credential_attempt("turn:thread:replaced".into(), "openai");
    host.credential_changed("openai");
    chat.observe_provider_health(&failure("replaced"));
    assert!(
        !std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, crate::app_event::AppEvent::InsertHistoryCell(_)))
    );
    assert_eq!(
        host.resolve_provider("openai").unwrap().configuration,
        ProviderConfigurationState::Configured
    );
    assert_eq!(
        host.resolve_provider("claude-plan").unwrap().configuration,
        ProviderConfigurationState::Configured
    );
}

#[tokio::test]
async fn apps_rejection_marks_openai_not_current_claude_and_r_opens_recovery() {
    let (mut chat, _tx, mut rx, _op_rx) =
        super::super::tests::make_chatwidget_manual_with_sender().await;
    let host = crate::provider_status_host::ProviderStatusHost::from_config(
        chat.config_ref(),
        crate::provider_status_host::ProviderAccountMetadata {
            openai: codex_login::OpenAiAuthMetadata::Account,
            claude: ClaudeCredentialMetadata::Configured {
                source: ClaudeCredentialSource::Managed,
            },
            ..Default::default()
        },
    );
    chat.model_catalog.set_provider_policy(
        super::super::provider_model_policy::ProviderModelPolicy::new(
            host.clone(),
            ProviderRuntimeAuthorizations::default(),
        ),
    );
    let update = |status, failure_reason| {
        ServerNotification::McpServerStatusUpdated(
            codex_app_server_protocol::McpServerStatusUpdatedNotification {
                thread_id: None,
                name: "codex_apps".into(),
                status,
                error: None,
                failure_reason,
            },
        )
    };
    chat.observe_provider_health(&update(McpServerStartupState::Starting, None));
    // A late MCP failure still belongs to OpenAI after switching the chat route.
    chat.config.model_provider_id = "claude-plan".into();
    chat.observe_provider_health(&update(
        McpServerStartupState::Failed,
        Some(McpServerStartupFailureReason::OpenAiAccountReauthenticationRequired),
    ));
    let warning = std::iter::from_fn(|| rx.try_recv().ok())
        .filter_map(|event| match event {
            crate::app_event::AppEvent::InsertHistoryCell(cell) => Some(cell.display_lines(80)),
            _ => None,
        })
        .flatten()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("rejected_provider_identity", warning);
    let statuses = host.resolve();
    assert_eq!(
        statuses.get("openai").unwrap().configuration,
        ProviderConfigurationState::RecoveryRequired
    );
    assert_eq!(
        statuses.get("claude-plan").unwrap().configuration,
        ProviderConfigurationState::Configured
    );
    let entry = host.catalog().get("openai").unwrap();
    chat.open_provider_manager(host.catalog(), statuses.entries(), Some(&entry.id));
    while rx.try_recv().is_ok() {}
    chat.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('r'),
        crossterm::event::KeyModifiers::NONE,
    ));
    assert!(
        matches!(rx.try_recv(), Ok(crate::app_event::AppEvent::OpenProviderManagerRecovery { provider_id }) if provider_id == entry.id)
    );
    chat.open_provider_recovery(entry, statuses.get("openai").unwrap());
    insta::assert_snapshot!(
        "openai_reauthentication",
        super::super::tests::helpers::render_bottom_popup(&chat, 90)
    );
    insta::assert_snapshot!(
        "openai_reauthentication_narrow",
        super::super::tests::helpers::render_bottom_popup(&chat, 45)
    );
    chat.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
    ));
    assert_eq!(
        host.resolve().get("openai").unwrap().configuration,
        ProviderConfigurationState::RecoveryRequired
    );
}
