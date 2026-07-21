use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::*;

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
    let area = Rect::new(0, 0, 60, view.desired_height(60));
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

    let _input_tx = start_with_executable(app_event_tx, &fake_claude);
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
        }) if message.contains("login complete")
    ));
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
