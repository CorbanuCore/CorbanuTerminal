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
    chat.observe_provider_health(&update(
        McpServerStartupState::Failed,
        Some(McpServerStartupFailureReason::OpenAiAccountReauthenticationRequired),
    ));
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
