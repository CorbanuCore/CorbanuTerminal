//! #1047 (follow-ups of #1015): Moderate confirmed before this window's
//! session starts says that session starts under Core's saved Moderate, and a
//! window on a remote app server (its sessions run in another process) is
//! not told that its session starts here.

use pretty_assertions::assert_eq;

use super::issue_443_tests::key;
use super::issue_443_tests::picker;
use super::issue_443_tests::render;
use super::issue_443_tests::text;
use super::*;
use crate::legacy_core::security_level_change::ApprovalsAtConfirmation;
use crate::legacy_core::security_level_change::LevelChangeReport;

fn saved_moderate(picker: &mut SecurityLevelPicker, not_saved: Option<&str>) {
    saved_moderate_as(picker, LevelChangeKind::Stricter, not_saved);
}

fn saved_moderate_as(
    picker: &mut SecurityLevelPicker,
    kind: LevelChangeKind,
    not_saved: Option<&str>,
) {
    picker.stored = StoredLevel::Chosen(ChosenLevel::Moderate);
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Moderate,
        core: Some(LevelChangeReport {
            kind,
            in_force: SecurityLevel::Moderate,
            next_start: SecurityLevel::Moderate,
            live: false,
            other_sessions: false,
            not_saved: not_saved.map(str::to_string),
            approvals: None,
        }),
    }));
}

fn saved_aggressive(picker: &mut SecurityLevelPicker) {
    picker.stored = StoredLevel::Chosen(ChosenLevel::Aggressive);
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Aggressive,
        core: Some(LevelChangeReport {
            kind: LevelChangeKind::Stricter,
            in_force: SecurityLevel::Aggressive,
            next_start: SecurityLevel::Aggressive,
            live: false,
            other_sessions: false,
            not_saved: None,
            approvals: Some(ApprovalsAtConfirmation::default()),
        }),
    }));
}

/// The Moderate review in a window whose session has not started: it
/// starts here under the saved level; other sessions here take it now.
#[test]
fn issue_1047_moderate_review_before_the_window_session_starts() {
    let root = tempfile::tempdir().unwrap();
    for sessions_here in [false, true] {
        let mut picker = picker(&root, /*live*/ false);
        if sessions_here {
            picker.sessions_run_here = |_, _| true;
        }
        picker.handle_key_event(key(KeyCode::Down));
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Moderate));
        let review = text(&picker);
        assert!(
            review.contains(&format!(
                "Core's level is saved as Moderate as soon as you confirm: {}. {WINDOW_SESSION_TAKES_MODERATE}",
                crate::security::moderate::EFFECTS
            )),
            "{review}"
        );
        assert!(
            review.contains("Model keys through the credential broker start when you restart Corbanu Terminal (you can restart right after saving)."),
            "{review}"
        );
        assert!(
            !review.contains("from the next start (this session's"),
            "{review}"
        );
        assert_eq!(
            review.contains(OTHER_SESSIONS_HERE_TAKE_MODERATE_NOW),
            sessions_here,
            "{review}"
        );
    }
}

/// The Saved screen: not "Until then this session stays Permissive", but
/// Core's Moderate rules when it starts, and the broker at the restart.
#[test]
fn issue_1047_moderate_saved_before_the_window_session_starts() {
    let root = tempfile::tempdir().unwrap();
    let mut picker = picker(&root, /*live*/ false);
    saved_moderate(&mut picker, /*not_saved*/ None);
    let saved = text(&picker);
    assert!(!saved.contains("stays Permissive"), "{saved}");
    assert!(
        saved.contains(&format!(
            "{WINDOW_SESSION_TAKES_MODERATE} Starts when you restart: model keys held by the credential broker"
        )),
        "{saved}"
    );
    assert!(
        saved.contains("Core's level is Moderate for every session that starts from now on."),
        "{saved}"
    );
    assert!(saved.contains(MODERATE_REACHES_OTHER_TERMINALS), "{saved}");
    insta::assert_snapshot!(
        "issue_1047_saved_moderate_before_the_window_session_starts",
        render(&picker, /*width*/ 80)
    );

    // Not saved: nothing holds it, so the session does not start under it.
    saved_moderate(&mut picker, Some("disk full"));
    let unsaved = text(&picker);
    assert!(
        !unsaved.contains("This window's session has not started yet"),
        "{unsaved}"
    );
    assert!(unsaved.contains("Core's level is unchanged."), "{unsaved}");
    assert!(
        !unsaved.contains(MODERATE_REACHES_OTHER_TERMINALS),
        "{unsaved}"
    );

    // Core's level was above Moderate (recovered Aggressive, or saved by
    // another Corbanu Terminal): saving Moderate lowers it only from the next
    // start, so nothing says it takes Moderate now or reaches other
    // terminals.
    // Already Moderate in Core's saved record: the session starts under it.
    saved_moderate_as(
        &mut picker,
        LevelChangeKind::Unchanged,
        /*not_saved*/ None,
    );
    let unchanged = text(&picker);
    assert!(
        unchanged.contains(WINDOW_SESSION_TAKES_MODERATE),
        "{unchanged}"
    );
    saved_moderate_as(
        &mut picker,
        LevelChangeKind::Downgrade,
        /*not_saved*/ None,
    );
    let lowered = text(&picker);
    for absent in [
        "This window's session has not started yet",
        MODERATE_REACHES_OTHER_TERMINALS,
    ] {
        assert!(!lowered.contains(absent), "{absent}: {lowered}");
    }
}

fn remote(picker: &mut SecurityLevelPicker, thread: Option<codex_protocol::ThreadId>) {
    picker.restart_blocked = None;
    picker.set_core_levels(CoreLevels {
        thread,
        sessions_in_this_process: false,
        ..CoreLevels::default()
    });
}

/// A window on a remote app server: its sessions do not run in this
/// process, so neither screen says its session starts here, and a restart
/// (refused against such a server while a level above Permissive is saved)
/// is not offered.
#[test]
fn issue_1047_remote_window_is_not_told_it_starts_here() {
    let root = tempfile::tempdir().unwrap();

    let mut picker = picker(&root, /*live*/ false);
    remote(&mut picker, /*thread*/ None);
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
    let review = text(&picker);
    assert!(
        review.contains(&format!(
            "Core's level is saved as Aggressive as soon as you confirm: {CORE_EFFECTS}. {}",
            remote_window_sessions(ChosenLevel::Aggressive, /*reaches*/ true)
        )),
        "{review}"
    );
    assert!(
        review.contains("The rows above apply in a Corbanu Terminal started locally (below); this window keeps its current settings."),
        "{review}"
    );
    for absent in [
        WINDOW_SESSION_TAKES_AGGRESSIVE,
        "when it starts here",
        "you can restart right after saving",
    ] {
        assert!(!review.contains(absent), "{absent}: {review}");
    }

    saved_aggressive(&mut picker);
    let saved = text(&picker);
    assert!(
        saved.contains(&remote_window_sessions(
            ChosenLevel::Aggressive,
            /*reaches*/ true
        )),
        "{saved}"
    );
    assert!(saved.contains(REMOTE_RESTART_REFUSED), "{saved}");
    assert!(!picker.restart_useful());
    assert!(
        saved.contains("Core's level is Aggressive from the next start."),
        "{saved}"
    );
    for absent in [
        "when it starts here",
        "stays Permissive",
        "Restart Corbanu Terminal to activate",
        "Press r to restart now",
    ] {
        assert!(!saved.contains(absent), "{absent}: {saved}");
    }
    insta::assert_snapshot!(
        "issue_1047_saved_aggressive_remote_window",
        render(&picker, /*width*/ 80)
    );

    // Core's level is above Moderate: the review offers no restart, which a
    // remote app server would refuse.
    let mut picker = super::issue_443_tests::picker(&root, /*live*/ false);
    remote(&mut picker, /*thread*/ None);
    picker.handle_key_event(key(KeyCode::Down));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Moderate));
    // Choosing reads the basis again; the review shows it as set here.
    picker.basis.in_force = SecurityLevel::Aggressive;
    let review = text(&picker);
    assert!(
        review.contains("Moderate applies from the next start."),
        "{review}"
    );
    assert!(
        !review.contains("you can restart right after saving"),
        "{review}"
    );

    // Also once the window's session is known (#978 wording before).
    let mut picker = super::issue_443_tests::picker(&root, /*live*/ false);
    remote(&mut picker, Some(codex_protocol::ThreadId::new()));
    picker.handle_key_event(key(KeyCode::Down));
    picker.handle_key_event(key(KeyCode::Enter));
    let review = text(&picker);
    assert!(
        review.contains(&format!(
            "Core's level is saved as Moderate as soon as you confirm: {}. {}",
            crate::security::moderate::EFFECTS,
            remote_window_sessions(ChosenLevel::Moderate, /*reaches*/ true)
        )),
        "{review}"
    );
    assert!(!review.contains("when it starts here"), "{review}");
    saved_moderate(&mut picker, /*not_saved*/ None);
    let saved = text(&picker);
    assert!(
        saved.contains(&remote_window_sessions(
            ChosenLevel::Moderate,
            /*reaches*/ true
        )),
        "{saved}"
    );
    assert!(
        !saved.contains("Restart Corbanu Terminal to activate"),
        "{saved}"
    );
    assert!(saved.contains(MODERATE_REACHES_OTHER_TERMINALS), "{saved}");
    assert!(!saved.contains("when it starts here"), "{saved}");

    // Not saved, or a lowered Core level: nothing reaches the server or
    // other terminals; the restart sentence stays.
    for (kind, not_saved) in [
        (LevelChangeKind::Stricter, Some("disk full")),
        (LevelChangeKind::Downgrade, None),
    ] {
        saved_moderate_as(&mut picker, kind, not_saved);
        let saved = text(&picker);
        assert!(
            saved.contains(&remote_window_sessions(
                ChosenLevel::Moderate,
                /*reaches*/ false
            )),
            "{saved}"
        );
        for absent in [
            "they take Core's saved level",
            MODERATE_REACHES_OTHER_TERMINALS,
        ] {
            assert!(!saved.contains(absent), "{absent}: {saved}");
        }
    }
}
