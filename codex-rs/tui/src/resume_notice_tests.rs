use super::*;
use crate::history_cell::HistoryCell;
use crate::resume_picker::SessionTarget;
use clap::Parser;
use codex_protocol::ThreadId;
use pretty_assertions::assert_eq;
use std::path::Path;

#[test]
fn resume_fallback_notice_wraps_and_names_lookup_scope() {
    let mut cli = Cli::parse_from(["corbanu"]);
    cli.resume_last = true;
    let message = resume_fallback_notice(
        &cli,
        &SessionSelection::StartFresh,
        Some("qwen"),
        Some(Path::new("/project")),
    )
    .expect("automatic fallback should explain why");
    let cell = crate::history_cell::new_transcript_warning_event(message);
    let rendered = cell
        .display_lines(/*width*/ 48)
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!(rendered);
}

#[test]
fn resume_fallback_notice_omits_filters_that_were_not_applied() {
    let mut cli = Cli::parse_from(["corbanu"]);
    cli.resume_last = true;
    cli.resume_show_all = true;
    cli.resume_include_non_interactive = true;
    assert_eq!(
        resume_fallback_notice(
            &cli,
            &SessionSelection::StartFresh,
            /*model_provider*/ None,
            /*cwd*/ None,
        ),
        Some("No resumable conversation found. Starting a new conversation.".to_string()),
    );
}

#[test]
fn resume_fallback_notice_is_absent_for_success_cancel_and_ordinary_start() {
    let mut cli = Cli::parse_from(["corbanu"]);
    let fresh = SessionSelection::StartFresh;
    assert_eq!(
        resume_fallback_notice(&cli, &fresh, Some("qwen"), /*cwd*/ None),
        None
    );
    cli.resume_picker = true;
    assert_eq!(
        resume_fallback_notice(&cli, &fresh, Some("qwen"), /*cwd*/ None),
        None
    );
    cli.resume_picker = false;
    cli.resume_last = true;
    let target = SessionTarget {
        backend_id: crate::backend_id::BackendId::new(),
        thread_id: ThreadId::new(),
        path: None,
        cwd: None,
        history_mode: None,
    };
    assert_eq!(
        resume_fallback_notice(
            &cli,
            &SessionSelection::Resume(target),
            Some("qwen"),
            /*cwd*/ None
        ),
        None,
    );
    assert_eq!(
        resume_fallback_notice(
            &cli,
            &SessionSelection::Exit,
            Some("qwen"),
            /*cwd*/ None
        ),
        None,
    );
    cli.fork_last = true;
    assert_eq!(
        resume_fallback_notice(&cli, &fresh, Some("qwen"), /*cwd*/ None),
        None
    );
}
