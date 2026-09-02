use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::*;

#[test]
fn managed_token_normalization_removes_only_pasted_line_breaks() {
    assert_eq!(
        remove_line_breaks("fixture-token\r\ncontinued\nvalue".to_string()),
        "fixture-tokencontinuedvalue"
    );
    assert_eq!(
        remove_line_breaks("fixture token\tvalue".to_string()),
        "fixture token\tvalue"
    );
}

#[tokio::test]
async fn login_output_reader_bounds_unterminated_lines_before_allocation() {
    let oversized = vec![b'x'; MAX_LOGIN_LINE_BYTES * 4];
    let mut reader = BufReader::new(oversized.as_slice());

    let line = read_bounded_output_line(&mut reader)
        .await
        .expect("read bounded output")
        .expect("output line");

    assert_eq!(line.len(), MAX_LOGIN_LINE_BYTES + 1);
}

#[tokio::test]
async fn login_output_reader_preserves_the_oversize_signal_across_a_utf8_boundary() {
    let oversized = "€".repeat(MAX_LOGIN_LINE_BYTES);
    let mut reader = BufReader::new(oversized.as_bytes());

    let line = read_bounded_output_line(&mut reader)
        .await
        .expect("read bounded UTF-8 output")
        .expect("output line");

    assert!(line.len() > MAX_LOGIN_LINE_BYTES);
    assert!(line.contains(char::REPLACEMENT_CHARACTER));
}

fn render_selection(params: SelectionViewParams, width: u16) -> String {
    let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);
    let view = crate::bottom_pane::ListSelectionView::new(
        params,
        app_event_tx,
        crate::keymap::RuntimeKeymap::defaults().list,
    );
    let area = Rect::new(0, 0, width, view.desired_height(width));
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer);
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn auth_method_choice_is_recommended_by_default_and_dispatches_exact_actions() {
    let params = auth_method_choice_params();
    assert_eq!(params.initial_selected_idx, Some(0));
    assert!(params.items[0].name.contains("Recommended"));
    assert!(!params.allow_number_shortcuts);

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);
    (params.items[0].actions[0])(&app_event_tx);
    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::RunClaudeSetupToken)
    ));
    (params.items[1].actions[0])(&app_event_tx);
    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::UseClaudeCodePlanLogin)
    ));
}

#[test]
fn account_authority_accepts_optional_status_metadata_but_requires_email() {
    let status = ClaudeCodePlanStatus::SignedIn {
        email: Some("fixture@example.invalid".to_string()),
        organization_id: None,
        subscription: None,
    };
    assert!(status_authority_id(&status).is_ok());
    let missing_email = ClaudeCodePlanStatus::SignedIn {
        email: None,
        organization_id: Some("org-fixture".to_string()),
        subscription: Some("max".to_string()),
    };
    assert!(status_authority_id(&missing_email).is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn relative_config_dir_matches_runtime_keychain_profile_identity() {
    let relative = std::path::PathBuf::from("target/claude-relative-profile-fixture");
    let absolute = std::path::absolute(&relative).expect("absolute fixture profile");
    let home = std::path::Path::new("/unused-home");

    assert_eq!(
        macos_platform_login_source_id(home, Some(relative), false).expect("relative source id"),
        macos_platform_login_source_id(home, Some(absolute), false).expect("absolute source id"),
    );
}

#[test]
fn selected_environment_token_status_requires_the_bound_token() {
    let selection = ClaudeAuthSelection::new_environment_token("selected-environment-token")
        .expect("selection");

    assert!(environment_token_matches_selection_value(
        &selection,
        Some(" selected-environment-token "),
    ));
    assert!(!environment_token_matches_selection_value(
        &selection,
        Some("replaced-environment-token"),
    ));
    assert!(!environment_token_matches_selection_value(
        &selection,
        Some("   ")
    ));

    let legacy = ClaudeAuthSelection::new_at(
        ClaudeAuthSource::EnvironmentToken,
        ENVIRONMENT_CLAUDE_AUTH_SOURCE_ID,
        10,
    )
    .expect("legacy selection");
    assert!(!environment_token_matches_selection_value(
        &legacy,
        Some("selected-environment-token"),
    ));
}

#[test]
fn claude_auth_method_choice_snapshot() {
    insta::assert_snapshot!(
        "claude_auth_method_choice",
        render_selection(auth_method_choice_params(), 76)
    );
}

#[test]
fn claude_auth_recovery_snapshot() {
    insta::assert_snapshot!(
        "claude_auth_recovery",
        render_selection(
            auth_recovery_params(
                "Selected token missing. Restore it or explicitly choose another method."
                    .to_string(),
            ),
            76,
        )
    );
}

#[test]
fn auth_recovery_exposes_explicit_legacy_environment_selection() {
    let params = auth_recovery_params("recover".to_string());
    let item = params
        .items
        .iter()
        .find(|item| item.name.contains("CLAUDE_CODE_OAUTH_TOKEN"))
        .expect("legacy environment recovery item");
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);

    (item.actions[0])(&app_event_tx);

    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::UseLegacyClaudeEnvironmentToken)
    ));
}

#[test]
fn managed_token_app_event_debug_is_redacted() {
    let canary = "claude-token-canary-never-render";
    let event = AppEvent::SaveClaudeManagedSubscriptionToken {
        token: ClaudeSubscriptionTokenSecret::new(canary.to_string()),
    };
    assert!(!format!("{event:?}").contains(canary));
}

#[test]
fn extracts_url_from_claude_osc8_link() {
    let line = concat!(
        "If the browser didn't open, visit: \u{1b}]8;;",
        "https://claude.com/oauth?state=private\u{7}",
        "https://claude.com/oauth?state=private\u{1b}]8;;\u{7}"
    );
    assert_eq!(
        extract_https_url(line),
        Some("https://claude.com/oauth?state=private".to_string())
    );
}

#[test]
fn ready_view_opens_masked_code_entry() {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);
    let (input_tx, _input_rx) = mpsc::unbounded_channel();
    let mut view = ClaudeCodePlanLoginView::ready(
        app_event_tx,
        "https://claude.com/oauth".to_string(),
        input_tx,
    );

    view.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::OpenClaudeCodePlanLoginCodeEntry { .. })
    ));
    assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
}

#[test]
fn escape_cancels_claude_subprocess() {
    let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);
    let (input_tx, mut input_rx) = mpsc::unbounded_channel();
    let mut view = ClaudeCodePlanLoginView::pending(app_event_tx, input_tx);

    view.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

    assert!(matches!(
        input_rx.try_recv(),
        Ok(ClaudeCodeLoginInput::Cancel)
    ));
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
}

#[test]
fn claude_login_ready_view_snapshot() {
    let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);
    let (input_tx, _input_rx) = mpsc::unbounded_channel();
    let view = ClaudeCodePlanLoginView::ready(
        app_event_tx,
        "https://claude.com/oauth/authorize?code=true&state=example".to_string(),
        input_tx,
    );
    let area = Rect::new(0, 0, 60, view.desired_height(/*width*/ 60));
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
}

#[cfg(unix)]
#[tokio::test]
async fn claude_cli_owns_token_exchange_and_status_verification() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    let source_id = current_platform_login_source_id().expect("source id");
    std::fs::write(
        &fake_claude,
        format!(
            r#"#!/bin/sh
if [ "$1" = "internal-claude-login-health" ]; then
  printf '%s\n' '{source_id}'
  exit 0
fi
if [ "$1 $2" = "auth login" ]; then
  printf 'If the browser did not open: https://claude.example/oauth?state=test\n'
  IFS= read -r code
  [ "$code" = "one-time-code" ]
  exit
fi
if [ "$1 $2 $3" = "auth status --json" ]; then
  printf '{{"loggedIn":true,"authMethod":"claude.ai","email":"fixture@example.invalid","orgId":"org-fixture","subscriptionType":"max"}}\n'
  exit 0
fi
exit 2
"#
        ),
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make fake claude executable");
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);

    let _input_tx = start_with_executable(
        app_event_tx,
        &fake_claude,
        Some(&fake_claude),
        temp_dir.path().to_path_buf(),
    );
    let input_tx = match event_rx.recv().await {
        Some(AppEvent::ClaudeCodePlanLoginReady {
            verification_url,
            input_tx,
        }) => {
            assert_eq!(verification_url, "https://claude.example/oauth?state=test");
            input_tx
        }
        other => panic!("expected ready event, got {other:?}"),
    };
    input_tx
        .send(ClaudeCodeLoginInput::AuthorizationCode(
            "one-time-code".to_string(),
        ))
        .expect("send authorization code");

    let finished = event_rx.recv().await;
    assert!(
        matches!(
            finished,
            Some(AppEvent::ClaudeCodePlanLoginFinished {
                result: Some(Ok(ref message))
            }) if message.contains("login selected")
        ),
        "unexpected login result: {finished:?}"
    );
    let selection = Vault::new(temp_dir.path().to_path_buf())
        .load_claude_auth_selection()
        .expect("load selection")
        .expect("selection persisted");
    assert_eq!(selection.source, ClaudeAuthSource::ClaudeCodeLogin);
    assert!(selection.authority_id.is_some());
}

#[cfg(unix)]
#[tokio::test]
async fn existing_claude_login_is_selected_without_reauthorization() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    let source_id = current_platform_login_source_id().expect("source id");
    std::fs::write(
        &fake_claude,
        format!("#!/bin/sh\nif [ \"$1\" = \"internal-claude-login-health\" ]; then printf '%s\\n' '{source_id}'; exit 0; fi\n[ \"$1 $2 $3\" = \"auth status --json\" ] || exit 2\nprintf '{{\"loggedIn\":true,\"authMethod\":\"claude.ai\",\"email\":\"fixture@example.invalid\",\"orgId\":\"org-fixture\",\"subscriptionType\":\"max\"}}\\n'\n"),
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make executable");

    assert!(
        select_existing_claude_code_login(
            &fake_claude,
            Some(&fake_claude),
            temp_dir.path(),
            Duration::from_secs(1),
        )
        .await
        .expect("select existing login")
    );
    let selection = Vault::new(temp_dir.path().to_path_buf())
        .load_claude_auth_selection()
        .expect("load selection")
        .expect("selection persisted");
    assert_eq!(selection.source, ClaudeAuthSource::ClaudeCodeLogin);
    assert!(selection.authority_id.is_some());
}

#[cfg(unix)]
#[tokio::test]
async fn unhealthy_platform_record_does_not_replace_the_previous_selection() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    std::fs::write(
        &fake_claude,
        "#!/bin/sh\n[ \"$1\" = \"internal-claude-login-health\" ] && exit 1\n[ \"$1 $2 $3\" = \"auth status --json\" ] || exit 2\nprintf '{\"loggedIn\":true,\"authMethod\":\"claude.ai\",\"email\":\"fixture@example.invalid\",\"orgId\":\"org-fixture\",\"subscriptionType\":\"max\"}\\n'\n",
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make executable");
    let vault = Vault::new(temp_dir.path().to_path_buf());
    let previous = ClaudeAuthSelection::new(
        ClaudeAuthSource::ManagedSubscriptionToken,
        MANAGED_CLAUDE_AUTH_SOURCE_ID,
    )
    .expect("previous selection");
    vault
        .save_claude_auth_selection(&previous)
        .expect("save previous selection");

    let error = select_existing_claude_code_login(
        &fake_claude,
        Some(&fake_claude),
        temp_dir.path(),
        Duration::from_secs(1),
    )
    .await
    .expect_err("unhealthy platform record must fail before persistence");

    assert!(error.contains("needs reauthorization"));
    assert_eq!(
        vault.load_claude_auth_selection().expect("load selection"),
        Some(previous)
    );
}

#[cfg(unix)]
#[tokio::test]
async fn providers_status_reports_selected_login_reauthorization_need() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    std::fs::write(
        &fake_claude,
        "#!/bin/sh\n[ \"$1\" = \"internal-claude-login-health\" ] && exit 1\n[ \"$1 $2 $3\" = \"auth status --json\" ] || exit 2\nprintf '{\"loggedIn\":true,\"authMethod\":\"claude.ai\",\"email\":\"fixture@example.invalid\",\"orgId\":\"org-fixture\",\"subscriptionType\":\"max\"}\\n'\n",
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make executable");
    let selection = ClaudeAuthSelection::new(
        ClaudeAuthSource::ClaudeCodeLogin,
        current_platform_login_source_id().expect("current source id"),
    )
    .expect("selection");
    Vault::new(temp_dir.path().to_path_buf())
        .save_claude_auth_selection(&selection)
        .expect("save selection");

    let status = current_status_with_executables(
        temp_dir.path(),
        Duration::from_secs(1),
        &fake_claude,
        Some(&fake_claude),
    )
    .await;

    assert_eq!(status, ClaudeCodePlanStatus::NeedsReauthorization);
}

#[cfg(unix)]
#[tokio::test]
async fn providers_status_reports_health_probe_timeout_as_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let hanging_health = temp_dir.path().join("hanging-health");
    std::fs::write(&hanging_health, "#!/bin/sh\nsleep 30\n").expect("write health fixture");
    let mut permissions = std::fs::metadata(&hanging_health)
        .expect("health fixture metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&hanging_health, permissions).expect("make health fixture executable");
    let selection = ClaudeAuthSelection::new(
        ClaudeAuthSource::ClaudeCodeLogin,
        current_platform_login_source_id().expect("current source id"),
    )
    .expect("selection");
    Vault::new(temp_dir.path().to_path_buf())
        .save_claude_auth_selection(&selection)
        .expect("save selection");

    let status = current_status_with_executables(
        temp_dir.path(),
        Duration::from_millis(25),
        Path::new("claude"),
        Some(&hanging_health),
    )
    .await;

    assert_eq!(status, ClaudeCodePlanStatus::Error);
}

#[cfg(unix)]
#[tokio::test]
async fn providers_status_preserves_unavailable_claude_after_a_healthy_probe() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let healthy_probe = temp_dir.path().join("healthy-probe");
    let source_id = current_platform_login_source_id().expect("current source id");
    std::fs::write(
        &healthy_probe,
        format!("#!/bin/sh\nprintf '%s\\n' '{source_id}'\n"),
    )
    .expect("write health fixture");
    let mut permissions = std::fs::metadata(&healthy_probe)
        .expect("health fixture metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&healthy_probe, permissions).expect("make health fixture executable");
    let authority_id = codex_vault::claude_login_authority_id(
        "fixture@example.invalid",
        Some("org-fixture"),
        Some("max"),
    )
    .expect("authority id");
    let selection =
        ClaudeAuthSelection::new_claude_code_login(source_id, authority_id).expect("selection");
    Vault::new(temp_dir.path().to_path_buf())
        .save_claude_auth_selection(&selection)
        .expect("save selection");

    let status = current_status_with_executables(
        temp_dir.path(),
        Duration::from_secs(1),
        &temp_dir.path().join("missing-claude"),
        Some(&healthy_probe),
    )
    .await;

    assert_eq!(status, ClaudeCodePlanStatus::Unavailable);
}

#[tokio::test]
async fn invalid_persisted_source_id_surfaces_recovery_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let selection = ClaudeAuthSelection::new(
        ClaudeAuthSource::ClaudeCodeLogin,
        "claude-login:copied-from-another-platform",
    )
    .expect("syntactically valid selection");
    Vault::new(temp_dir.path().to_path_buf())
        .save_claude_auth_selection(&selection)
        .expect("save selection");

    assert_eq!(
        current_status_with_timeout(temp_dir.path(), Duration::from_secs(30)).await,
        ClaudeCodePlanStatus::InvalidSelection
    );
}

#[cfg(unix)]
#[tokio::test]
async fn claude_status_timeout_resolves_to_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    std::fs::write(&fake_claude, "#!/bin/sh\nsleep 30\n").expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make fake claude executable");

    let status = tokio::time::timeout(
        Duration::from_secs(1),
        status_with_timeout(&fake_claude, Duration::from_millis(25)),
    )
    .await
    .expect("status check must remain bounded");

    assert_eq!(status, ClaudeCodePlanStatus::Error);
}

#[cfg(unix)]
#[tokio::test]
async fn provider_status_timeout_includes_vault_lock_contention() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let secrets_dir = temp_dir.path().join("secrets");
    std::fs::create_dir_all(&secrets_dir).expect("create secrets dir");
    let lock_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(secrets_dir.join(".vault.lock"))
        .expect("open vault lock");
    lock_file.lock().expect("hold vault lock");

    let status = tokio::time::timeout(
        Duration::from_secs(1),
        current_status_with_timeout(temp_dir.path(), Duration::from_millis(25)),
    )
    .await
    .expect("provider status must remain bounded while vault storage is locked");

    assert_eq!(status, ClaudeCodePlanStatus::Error);
    drop(lock_file);
}

#[cfg(unix)]
#[tokio::test]
async fn post_login_status_timeout_does_not_persist_a_selection() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    std::fs::write(&fake_claude, "#!/bin/sh\nsleep 30\n").expect("write hanging fake Claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake Claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make fake Claude executable");

    let error = tokio::time::timeout(
        Duration::from_secs(1),
        verify_login_with_timeout(
            &fake_claude,
            Some(&fake_claude),
            temp_dir.path(),
            Duration::from_millis(25),
        ),
    )
    .await
    .expect("post-login verification must remain bounded")
    .expect_err("hanging status must fail verification");

    assert!(error.contains("timed out"));
    assert_eq!(
        Vault::new(temp_dir.path().to_path_buf())
            .load_claude_auth_selection()
            .expect("load selection"),
        None
    );
}

#[cfg(unix)]
#[tokio::test]
async fn claude_status_preserves_custom_oauth_profile_identity() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    std::fs::write(
        &fake_claude,
        "#!/bin/sh\n[ \"$CLAUDE_CODE_CUSTOM_OAUTH_URL\" = \"https://oauth.example.invalid\" ] || exit 3\nprintf '{\"loggedIn\":true,\"authMethod\":\"claude.ai\",\"email\":\"fixture@example.invalid\",\"orgId\":\"org-fixture\",\"subscriptionType\":\"max\"}\\n'\n",
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make executable");

    assert!(matches!(
        read_status_with_profile(
            &fake_claude,
            None,
            Some(std::ffi::OsStr::new("https://oauth.example.invalid")),
        )
        .await
        .expect("status"),
        ClaudeCodePlanStatus::SignedIn { .. }
    ));
}
