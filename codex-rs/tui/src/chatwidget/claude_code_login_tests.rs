use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::*;

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
    std::fs::write(
        &fake_claude,
        r#"#!/bin/sh
if [ "$1 $2" = "auth login" ]; then
  printf 'If the browser did not open: https://claude.example/oauth?state=test\n'
  IFS= read -r code
  [ "$code" = "one-time-code" ]
  exit
fi
if [ "$1 $2 $3" = "auth status --json" ]; then
  printf '{"loggedIn":true,"authMethod":"claude.ai"}\n'
  exit 0
fi
exit 2
"#,
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make fake claude executable");
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let app_event_tx = AppEventSender::new(event_tx);

    let _input_tx =
        start_with_executable(app_event_tx, &fake_claude, temp_dir.path().to_path_buf());
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

    assert!(matches!(
        event_rx.recv().await,
        Some(AppEvent::ClaudeCodePlanLoginFinished {
            result: Some(Ok(message))
        }) if message.contains("login selected")
    ));
    let selection = Vault::new(temp_dir.path().to_path_buf())
        .load_claude_auth_selection()
        .expect("load selection")
        .expect("selection persisted");
    assert_eq!(selection.source, ClaudeAuthSource::ClaudeCodeLogin);
}

#[cfg(unix)]
#[tokio::test]
async fn existing_claude_login_is_selected_without_reauthorization() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let fake_claude = temp_dir.path().join("claude");
    std::fs::write(
        &fake_claude,
        "#!/bin/sh\n[ \"$1 $2 $3\" = \"auth status --json\" ] || exit 2\nprintf '{\"loggedIn\":true,\"authMethod\":\"claude.ai\"}\\n'\n",
    )
    .expect("write fake claude");
    let mut permissions = std::fs::metadata(&fake_claude)
        .expect("fake claude metadata")
        .permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&fake_claude, permissions).expect("make executable");

    assert!(
        select_existing_claude_code_login(&fake_claude, temp_dir.path(), Duration::from_secs(1),)
            .await
            .expect("select existing login")
    );
    let selection = Vault::new(temp_dir.path().to_path_buf())
        .load_claude_auth_selection()
        .expect("load selection")
        .expect("selection persisted");
    assert_eq!(selection.source, ClaudeAuthSource::ClaudeCodeLogin);
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
        current_status_with_timeout(temp_dir.path(), Duration::from_millis(25)).await,
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
