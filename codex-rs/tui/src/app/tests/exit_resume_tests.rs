//! Corbanu exit resume hints: the thread on screen, then the primary session (#1412), then a
//! thread whose saved pane layout can still be resumed. Ported from main's
//! `app/tests/session_summary.rs`, which upstream removed with `session_summary`.

use super::*;
use crate::claude_panes::PANE_LAYOUT_VERSION;
use crate::claude_panes::PaneLayoutState;
use crate::claude_panes::persist_pane_layout;
use pretty_assertions::assert_eq;

fn session_with_rollout(
    thread_id: ThreadId,
    thread_name: Option<String>,
    rollout_path: Option<std::path::PathBuf>,
) -> ThreadSessionState {
    let mut session = test_thread_session(thread_id, test_path_buf("/tmp/project"));
    session.thread_name = thread_name;
    session.rollout_path = rollout_path;
    session
}

fn exit_lines(resume_hint: Option<ResumableThread>) -> Vec<String> {
    AppExitInfo {
        token_usage: TokenUsage::default(),
        thread_id: resume_hint.as_ref().map(|thread| thread.thread_id),
        resume_hint,
        disconnect_info: None,
        update_action: None,
        exit_reason: ExitReason::UserRequested,
    }
    .format_exit_messages(/*color_enabled*/ false)
}

#[tokio::test]
async fn exit_resume_hint_falls_back_to_primary_when_active_codex_pane_has_no_rollout() {
    let mut app = make_test_app().await;
    let temp_dir = tempdir().expect("temp dir");
    let primary_rollout_path = temp_dir.path().join("primary-rollout.jsonl");
    std::fs::write(&primary_rollout_path, "{}\n").expect("write primary rollout");
    let primary_thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174001").expect("thread id");
    let side_thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174002").expect("thread id");

    app.primary_thread_id = Some(primary_thread_id);
    app.primary_session_configured = Some(session_with_rollout(
        primary_thread_id,
        /*thread_name*/ None,
        Some(primary_rollout_path),
    ));
    app.active_thread_id = Some(side_thread_id);
    app.chat_widget.handle_thread_session(session_with_rollout(
        side_thread_id,
        Some("Codex 1".to_string()),
        /*rollout_path*/ None,
    ));

    let exit_thread = app.exit_resumable_thread().expect("primary fallback");
    assert_eq!(
        exit_thread,
        ResumableThread {
            thread_id: primary_thread_id,
            thread_name: None,
        }
    );
    assert_eq!(
        exit_lines(Some(exit_thread)),
        vec![
            "To continue this session, run:".to_string(),
            "  corbanu resume 123e4567-e89b-12d3-a456-426614174001".to_string(),
        ]
    );
}

#[tokio::test]
async fn exit_resume_hint_prefers_active_codex_pane_when_it_is_resumable() {
    let mut app = make_test_app().await;
    let temp_dir = tempdir().expect("temp dir");
    let primary_rollout_path = temp_dir.path().join("primary-rollout.jsonl");
    let side_rollout_path = temp_dir.path().join("side-rollout.jsonl");
    std::fs::write(&primary_rollout_path, "{}\n").expect("write primary rollout");
    std::fs::write(&side_rollout_path, "{}\n").expect("write side rollout");
    let primary_thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174003").expect("thread id");
    let side_thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174004").expect("thread id");

    app.primary_thread_id = Some(primary_thread_id);
    app.primary_session_configured = Some(session_with_rollout(
        primary_thread_id,
        /*thread_name*/ None,
        Some(primary_rollout_path),
    ));
    app.active_thread_id = Some(side_thread_id);
    app.chat_widget.handle_thread_session(session_with_rollout(
        side_thread_id,
        Some("Codex 1".to_string()),
        Some(side_rollout_path),
    ));

    let exit_thread = app.exit_resumable_thread().expect("active thread");
    assert_eq!(
        exit_thread,
        ResumableThread {
            thread_id: side_thread_id,
            thread_name: Some("Codex 1".to_string()),
        }
    );
    assert_eq!(
        exit_lines(Some(exit_thread)),
        vec![
            "To continue this session, run:".to_string(),
            "  corbanu resume 123e4567-e89b-12d3-a456-426614174004".to_string(),
            "Or run corbanu resume and select Codex 1.".to_string(),
        ]
    );
}

#[test]
fn resume_hint_requires_a_nonempty_rollout() {
    let thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174000").expect("thread id");
    let temp_dir = tempdir().expect("temp dir");
    let rollout_path = temp_dir.path().join("rollout.jsonl");

    let missing = crate::app::resumable_thread(
        Some(thread_id),
        /*thread_name*/ None,
        Some(&rollout_path),
    );
    std::fs::write(&rollout_path, "").expect("write empty rollout");
    let empty = crate::app::resumable_thread(
        Some(thread_id),
        /*thread_name*/ None,
        Some(&rollout_path),
    );
    std::fs::write(&rollout_path, "{}\n").expect("write rollout");
    let persisted = crate::app::resumable_thread(
        Some(thread_id),
        Some("my-session".to_string()),
        Some(&rollout_path),
    );

    assert_eq!(missing, None);
    assert_eq!(empty, None);
    assert_eq!(
        persisted,
        Some(ResumableThread {
            thread_id,
            thread_name: Some("my-session".to_string()),
        })
    );
}

#[tokio::test]
async fn pane_layout_thread_resume_hint_does_not_require_rollout() {
    let app = make_test_app().await;
    let thread_id =
        ThreadId::from_string("123e4567-e89b-12d3-a456-426614174010").expect("thread id");
    let layout = PaneLayoutState {
        version: PANE_LAYOUT_VERSION,
        codex_thread_id: Some(thread_id.to_string()),
        active_user_pane_id: Some("claude-child".to_string()),
        spawn_nazgul_pane_id: None,
        claude_pane_ids: vec!["claude-child".to_string()],
        ..Default::default()
    };
    persist_pane_layout(app.config.codex_home.as_ref(), &layout).expect("persist layout");

    assert_eq!(
        crate::app::exit_summary::pane_layout_resumable_thread(
            app.config.codex_home.as_ref(),
            Some(thread_id),
        ),
        Some(ResumableThread {
            thread_id,
            thread_name: None,
        })
    );
}
