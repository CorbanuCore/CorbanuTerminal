//! Issue #443 (milestone AGG-04/AGG-37): when confirming Aggressive raises
//! Core's level in the running session, the review, screen 07 and the list
//! say which protections hold now and which wait for the restart, and never
//! that this session stays Permissive.

use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::*;
use crate::keymap::RuntimeKeymap;
use crate::legacy_core::protected_preflight::InventorySources;
use crate::legacy_core::protected_preflight::ReadinessFlags;
use crate::legacy_core::security_level_change::ApprovalsAtConfirmation;
use crate::legacy_core::security_level_change::LevelChangeReport;

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

pub(super) fn render(picker: &SecurityLevelPicker, width: u16) -> String {
    let mut lines = picker.lines(width);
    lines.push(Line::from(picker.footer()));
    let area = Rect::new(0, 0, width, lines.len() as u16);
    let mut buffer = Buffer::empty(area);
    Paragraph::new(lines).render(area, &mut buffer);
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

/// One line per paragraph, so phrases never break across lines.
pub(super) fn text(picker: &SecurityLevelPicker) -> String {
    render(picker, /*width*/ 1000)
}

/// A Permissive session with the activation preflight on. With `live`, its
/// policy tree runs in this process, as in a running TUI session (the tests
/// set it directly: building a tree needs Core internals).
pub(super) fn picker(root: &tempfile::TempDir, live: bool) -> SecurityLevelPicker {
    let corbanu = root.path().join("corbanu");
    for dir in ["corbanu", "home", "work"] {
        std::fs::create_dir_all(root.path().join(dir)).unwrap();
    }
    let input = PreflightInput {
        sources: InventorySources {
            codex_home: corbanu.clone(),
            home: Some(root.path().join("home")),
            cwd: root.path().join("work"),
            env: Vec::new(),
            config_layers: Vec::new(),
        },
        flags: ReadinessFlags {
            secretless_launch: true,
            credential_broker: true,
            output_gate: true,
            untrusted_content: true,
        },
    };
    let context = LevelContext {
        codex_home: corbanu,
        picker_enabled: true,
        active: ChosenLevel::Permissive,
        preflight_enabled: true,
        boundary: None,
    };
    let current = [
        "commands can write to the current folder",
        "on-request (reviewer: you)",
        "on for agent commands; web search live",
        "the vault store and sign-in file are readable to agent commands",
        "Corbanu reads provider keys itself (broker_model_auth off)",
        "spawned agents get this session's values",
    ]
    .map(str::to_string);
    let mut picker = SecurityLevelPicker::new(
        &context,
        current,
        Some(input),
        RuntimeKeymap::defaults().list,
    );
    picker.basis.live = live;
    picker
}

/// What a live session's tree reports after the commit: Aggressive now.
pub(super) fn raised_live(picker: &mut SecurityLevelPicker) {
    picker.basis.live = true;
    picker.basis.in_force = SecurityLevel::Aggressive;
    picker.basis.next_start = SecurityLevel::Aggressive;
}

#[test]
fn issue_443_review_says_core_protections_apply_on_confirm() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    // Choosing reads the basis again; a running session's tree is live.
    picker.basis.live = true;
    let review = text(&picker);
    assert!(
        review.contains(
            "Core's protections below apply as soon as you confirm, and approval for every command from each pane's next turn."
        ),
        "{review}"
    );
    assert!(
        review.contains("agent commands cannot run `corbanu vault …`"),
        "{review}"
    );
    assert!(
        !review.contains("this session keeps its current settings until then"),
        "{review}"
    );
}

#[test]
fn issue_443_review_without_a_live_tree_waits_for_the_restart() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ false);
    // The session's thread is known but its policy does not run here (a
    // remote app server).
    picker.core.thread = Some(codex_protocol::ThreadId::new());
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    let review = text(&picker);
    assert!(
        review.contains("this session keeps its current settings until then"),
        "{review}"
    );
    assert!(
        review.contains("Core's level becomes Aggressive from the next start"),
        "{review}"
    );
    // #978: no session of this process runs on the home (as with a remote
    // app server): nothing is claimed for other sessions.
    assert!(
        !review.contains("The other sessions of this Corbanu Terminal"),
        "{review}"
    );
}

/// #978: confirmed where this session's policy does not run in this
/// process: the Saved screen says the other sessions of this Corbanu
/// Terminal took Core's level and approval for every command now.
#[test]
fn issue_978_saved_without_a_live_tree_names_the_other_sessions() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ false);
    picker.stored = StoredLevel::Chosen(ChosenLevel::Aggressive);
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Aggressive,
        core: Some(LevelChangeReport {
            kind: LevelChangeKind::Stricter,
            in_force: SecurityLevel::Aggressive,
            next_start: SecurityLevel::Aggressive,
            live: false,
            other_sessions: true,
            not_saved: None,
            approvals: Some(ApprovalsAtConfirmation {
                held: 2,
                kept: Vec::new(),
                pending: 0,
            }),
        }),
    }));
    let saved = text(&picker);
    // #1015: this window's session has not started; it starts under it.
    assert!(!saved.contains("stays Permissive"), "{saved}");
    assert!(
        saved.contains(
            "This window's session has not started yet: when it starts here before you restart, it takes Core's Aggressive protections and approval for every command, and its other settings stay Permissive until the restart."
        ),
        "{saved}"
    );
    assert!(
        saved.contains(
            "Core's level is Aggressive now in the other sessions of this Corbanu Terminal, and from every start."
        ),
        "{saved}"
    );
    assert!(
        saved.contains("approve every command that is not known read-only (reviewer: you), now in 2 threads of this Corbanu Terminal"),
        "{saved}"
    );
    insta::assert_snapshot!(
        "issue_978_saved_aggressive_without_a_live_tree",
        render(&picker, /*width*/ 80)
    );

    // Review r1 nits: not saved, no "from every start"; chosen again, the
    // sessions below it are named.
    let report = |kind, not_saved: Option<&str>| LevelChangeReport {
        kind,
        in_force: SecurityLevel::Aggressive,
        next_start: SecurityLevel::Aggressive,
        live: false,
        other_sessions: true,
        not_saved: not_saved.map(str::to_string),
        approvals: None,
    };
    assert_eq!(
        core_outcome_line(&report(LevelChangeKind::Stricter, Some("disk full"))),
        "Core's level is Aggressive now in the other sessions of this Corbanu Terminal."
    );
    assert_eq!(
        core_outcome_line(&report(LevelChangeKind::Unchanged, /*not_saved*/ None)),
        "Core's level: Aggressive, now also in the other sessions of this Corbanu Terminal that were below it."
    );
}

#[test]
fn issue_443_screen_07_names_what_applies_now_and_at_restart() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    saved_live(&mut picker);
    let saved = text(&picker);
    assert!(!saved.contains("stays Permissive"), "{saved}");
    assert!(
        saved.contains("Active now in this session: agent commands cannot run `corbanu vault …`"),
        "{saved}"
    );
    assert!(saved.contains("Starts when you restart:"), "{saved}");
    assert!(
        saved.contains("commands already running keep the permissions they started with"),
        "{saved}"
    );
    insta::assert_snapshot!(
        "issue_443_saved_aggressive_live",
        render(&picker, /*width*/ 80)
    );

    // Esc, then /security again: the list says the same.
    picker.screen = Screen::List { note: None };
    let list = text(&picker);
    assert!(
        list.contains("Saved: Aggressive. Core's protections (vault refusal, protected paths) apply in this session now"),
        "{list}"
    );
    assert!(!list.contains("It takes effect when you restart"), "{list}");
    assert_eq!(picker.active, ChosenLevel::Permissive);
}

/// Screen 07 as a running session's tree reports it after the commit.
pub(super) fn saved_live(picker: &mut SecurityLevelPicker) {
    saved_live_with(
        picker,
        Some(ApprovalsAtConfirmation {
            held: 2,
            kept: Vec::new(),
            pending: 0,
        }),
    );
}

pub(super) fn saved_live_with(
    picker: &mut SecurityLevelPicker,
    approvals: Option<ApprovalsAtConfirmation>,
) {
    raised_live(picker);
    picker.stored = StoredLevel::Chosen(ChosenLevel::Aggressive);
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Aggressive,
        core: Some(LevelChangeReport {
            kind: LevelChangeKind::Stricter,
            in_force: SecurityLevel::Aggressive,
            next_start: SecurityLevel::Aggressive,
            live: true,
            other_sessions: false,
            not_saved: None,
            approvals,
        }),
    }));
}

/// Without the untrusted-content labels the protected-path denials do not
/// run, so only the vault refusal is claimed as active now.
#[test]
fn issue_443_screen_07_claims_only_the_vault_rule_without_source_envelopes() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    if let Some(input) = picker.preflight_input.as_mut() {
        input.flags.untrusted_content = false;
    }
    saved_live(&mut picker);
    let saved = text(&picker);
    assert!(
        saved
            .contains("Active now in this session: agent commands cannot run `corbanu vault …`.\n"),
        "{saved}"
    );
    assert!(!saved.contains("protected paths"), "{saved}");
    picker.screen = Screen::List { note: None };
    let list = text(&picker);
    assert!(
        list.contains("Core's protections (vault refusal) apply"),
        "{list}"
    );
}

/// Returning to Permissive before the restart: Core's Aggressive level stays
/// in force in this session, so the review does not call it Permissive.
#[test]
fn issue_443_permissive_review_before_restart_keeps_core_protections() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    saved_live(&mut picker);
    picker.screen = Screen::Review(ChosenLevel::Permissive);
    let review = text(&picker);
    assert!(
        review.contains("Core's Aggressive protections stay on in this session until you restart"),
        "{review}"
    );
    assert!(
        !review.contains("This session is already Permissive"),
        "{review}"
    );
}

/// #505: confirming through a live tree requires approval for every command
/// from each pane's next turn; the review says so, and that the rest waits
/// for the restart.
#[test]
fn issue_505_review_says_approvals_apply_on_confirm() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    picker.basis.live = true;
    let review = text(&picker);
    assert!(
        review.contains(
            "Core's protections below apply as soon as you confirm, and approval for every command from each pane's next turn."
        ),
        "{review}"
    );
    assert!(
        review.contains("this session keeps its current sandbox, network, environment, model-key and child-agent settings"),
        "{review}"
    );
    assert!(
        review.contains("Approval for every command: from each pane's next turn you approve every command that is not known read-only (reviewer: you), in every Corbanu pane"),
        "{review}"
    );
    assert!(
        review.contains(
            "a pane whose managed configuration rules this out keeps its own approval settings"
        ),
        "{review}"
    );
}

/// #505: without a live tree nothing changes before the restart, approvals
/// included.
#[test]
fn issue_505_review_without_a_live_tree_keeps_approvals_for_the_restart() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ false);
    // A remote app server: the session runs in another process.
    picker.core.thread = Some(codex_protocol::ThreadId::new());
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    let review = text(&picker);
    assert!(!review.contains("Approval for every command:"), "{review}");
    assert!(
        review.contains("this session keeps its current settings until then"),
        "{review}"
    );
}

/// #505: screen 07 reports what the confirmation did: approvals now, the
/// rest at restart; a thread a managed requirement kept is named.
#[test]
fn issue_505_screen_07_reports_approvals_and_kept_threads() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    saved_live(&mut picker);
    let saved = text(&picker);
    assert!(
        saved.contains("Approval for every command: from each pane's next turn you approve every command that is not known read-only (reviewer: you), now in 2 threads of this Corbanu Terminal"),
        "{saved}"
    );
    assert!(
        !saved.contains("approval for every command that is not known read-only, network"),
        "approvals are no longer listed for the restart: {saved}"
    );
    assert!(!saved.contains("was not applied"), "{saved}");

    saved_live_with(
        &mut picker,
        Some(ApprovalsAtConfirmation {
            held: 0,
            kept: vec!["thread-1: managed reviewer".to_string()],
            pending: 1,
        }),
    );
    let saved = text(&picker);
    assert!(
        saved.contains("Approval for every command was not applied in 1 thread (their approval settings stay as they are): thread-1: managed reviewer."),
        "{saved}"
    );
    assert!(
        saved.contains(
            "Still applying in 1 thread: not answered yet; it applies there as soon as it answers."
        ),
        "{saved}"
    );
    // Held nowhere: nothing claims it, and the restart list keeps it.
    assert!(!saved.contains("now in 0 threads"), "{saved}");
    assert!(
        saved.contains("approval for every command that is not known read-only, network"),
        "{saved}"
    );

    // A commit that did not reach this process's sessions claims nothing.
    saved_live_with(&mut picker, /*approvals*/ None);
    let saved = text(&picker);
    assert!(!saved.contains("Approval for every command:"), "{saved}");
    assert!(
        saved.contains("approval for every command that is not known read-only, network"),
        "{saved}"
    );
}

/// #919 (Travis, 2026-10-16, D20): the Aggressive review and screen 07 say
/// that other Corbanu Terminals running on the home take Core's Aggressive
/// protections too; screen 07 does not promise it when the level could not
/// be saved.
#[test]
fn issue_919_aggressive_review_and_saved_say_other_terminals_take_it() {
    const OTHERS: &str = "Other Corbanu Terminals running on this Corbanu home take Core's Aggressive protections and approval for every command within about a second";
    for live in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let mut picker = picker(&root, live);
        picker.handle_key_event(key(KeyCode::Up));
        picker.handle_key_event(key(KeyCode::Enter));
        picker.basis.live = live;
        let review = text(&picker);
        assert!(review.contains(OTHERS), "{review}");
    }

    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ true);
    saved_live(&mut picker);
    let saved = text(&picker);
    assert!(saved.contains(OTHERS), "{saved}");
    assert!(
        saved.contains("the rest of Aggressive starts there when they restart"),
        "{saved}"
    );

    if let Screen::Saved(Ok(outcome)) = &mut picker.screen
        && let Some(report) = outcome.core.as_mut()
    {
        report.not_saved = Some("disk full".to_string());
    }
    let unsaved = text(&picker);
    assert!(unsaved.contains("could not be saved"), "{unsaved}");
    assert!(!unsaved.contains("Other Corbanu Terminals"), "{unsaved}");
}

/// #1015: the window's own session has not started (no thread yet) when
/// Aggressive is confirmed without a live tree: the review says it starts
/// under Core's level and approval for every command, not that it keeps its
/// settings until the restart; with other sessions here, they are named.
#[test]
fn issue_1015_review_before_the_window_session_starts() {
    let root = tempfile::tempdir().unwrap();
    for sessions_here in [false, true] {
        let mut picker = picker(&root, /*live*/ false);
        if sessions_here {
            picker.sessions_run_here = |_, _| true;
        }
        picker.handle_key_event(key(KeyCode::Up));
        picker.handle_key_event(key(KeyCode::Enter));
        let review = text(&picker);
        assert!(
            !review.contains("this session keeps its current settings until then"),
            "{review}"
        );
        assert!(
            review.contains(
                "until then this window's session keeps its current settings, apart from Core's level and approval for every command (below)."
            ),
            "{review}"
        );
        assert!(
            review.contains(&format!(
                "Core's level is saved as Aggressive as soon as you confirm: {CORE_EFFECTS}. {WINDOW_SESSION_TAKES_AGGRESSIVE}"
            )),
            "{review}"
        );
        assert!(
            !review.contains("from the next start (this session's"),
            "{review}"
        );
        assert_eq!(
            review.contains(OTHER_SESSIONS_HERE_TAKE_AGGRESSIVE_NOW),
            sessions_here,
            "{review}"
        );
        if sessions_here {
            // From the paragraph this changes; the preflight rows and the
            // footer above and below it differ by platform.
            let rendered = render(&picker, /*width*/ 80);
            let lines: Vec<&str> = rendered.lines().collect();
            let from = lines
                .iter()
                .position(|line| line.starts_with("The rows above take effect"))
                .expect("the restart paragraph");
            insta::assert_snapshot!(
                "issue_1015_review_before_the_window_session_starts",
                lines[from..lines.len() - 1].join("\n")
            );
        }
    }
}

/// #1015 item 2: the not-live review when other sessions of this process
/// run on the home and this session's thread is known (its policy does not
/// run here yet).
#[test]
fn issue_1015_review_names_the_other_sessions_here() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ false);
    picker.core.thread = Some(codex_protocol::ThreadId::new());
    picker.sessions_run_here = |_, thread| thread.is_some();
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    let review = text(&picker);
    assert!(
        review.contains(&format!(
            "Core's level becomes Aggressive from the next start (this session's policy does not run in this process yet): {CORE_EFFECTS}. {OTHER_SESSIONS_HERE_TAKE_AGGRESSIVE}"
        )),
        "{review}"
    );
    assert!(
        review.contains("this session keeps its current settings until then"),
        "{review}"
    );
}

/// #1015 item 3: no live tree, no other session here and the save failed:
/// nothing holds Core's level, so the Saved screen claims neither the next
/// start nor that it applies now.
#[test]
fn issue_1015_saved_not_held_and_not_saved_claims_nothing() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ false);
    picker.stored = StoredLevel::Chosen(ChosenLevel::Aggressive);
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Aggressive,
        core: Some(LevelChangeReport {
            kind: LevelChangeKind::Stricter,
            in_force: SecurityLevel::Aggressive,
            next_start: SecurityLevel::Aggressive,
            live: false,
            other_sessions: false,
            not_saved: Some("disk full".to_string()),
            approvals: Some(ApprovalsAtConfirmation::default()),
        }),
    }));
    let saved = text(&picker);
    for absent in [
        "from the next start",
        "for every session that starts",
        "applies now",
        "Core's Aggressive protections and",
        "stays Permissive.",
    ] {
        assert!(!saved.contains(absent), "{absent}: {saved}");
    }
    assert!(saved.contains("Core's level is unchanged."), "{saved}");
    assert!(
        saved.contains(
            "Core's level could not be saved (disk full), and no running session of this Corbanu Terminal holds it. Choose the level again to retry."
        ),
        "{saved}"
    );
    assert!(
        saved.contains(
            "when it starts here before you restart, it takes approval for every command"
        ),
        "{saved}"
    );
}
