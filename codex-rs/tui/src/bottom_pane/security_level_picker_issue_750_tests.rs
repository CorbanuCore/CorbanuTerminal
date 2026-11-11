//! #750: Moderate can be chosen in `/security`, with a review of what it
//! adds and what is not available (D1), a confirmation that saves it with
//! Core's record (Core's rules now, the credential broker at the next start:
//! D4), "restart now", and the moves back to Permissive and to and from
//! Aggressive. Esc changes nothing.

use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::*;
use crate::keymap::RuntimeKeymap;
use crate::security::moderate::Facts;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn render(picker: &SecurityLevelPicker, width: u16) -> String {
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
fn text(picker: &SecurityLevelPicker) -> String {
    render(picker, /*width*/ 1000)
}

const ALL_ON: Facts = Facts {
    untrusted_content: true,
    secretless_launch: true,
    output_gate: true,
};

fn open_picker(home: &std::path::Path, active: ChosenLevel, facts: Facts) -> SecurityLevelPicker {
    let context = LevelContext {
        codex_home: home.to_path_buf(),
        picker_enabled: true,
        active,
        preflight_enabled: false,
        boundary: None,
    };
    let current = [
        "commands can write to the current folder; permission-request tools are off",
        "on-request (reviewer: you)",
        "off for agent commands; web search cached",
        "the vault store and sign-in file are readable to agent commands",
        "held by the isolated credential broker (broker_model_auth on)",
        "spawned agents get this session's values",
    ]
    .map(str::to_string);
    let mut picker = SecurityLevelPicker::new(
        &context,
        current,
        /*preflight_input*/ None,
        RuntimeKeymap::defaults().list,
    );
    picker.set_moderate_facts(facts);
    picker
}

fn choose_moderate(picker: &mut SecurityLevelPicker) {
    while ROWS[picker.selected] != Row::Moderate {
        picker.handle_key_event(key(KeyCode::Down));
    }
    picker.handle_key_event(key(KeyCode::Enter));
}

fn confirmed(home: &std::path::Path) -> level::ConfirmedLevel {
    level::load_confirmed(home)
}

#[test]
fn issue_750_moderate_is_offered_reviewed_and_esc_changes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Permissive, ALL_ON);
    let list = text(&picker);
    assert!(!list.contains("not available yet"), "{list}");
    choose_moderate(&mut picker);
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Moderate));
    insta::assert_snapshot!(
        "security_level_picker_review_moderate",
        render(&picker, /*width*/ 80)
    );
    let review = text(&picker);
    for expected in [
        "Switch to Moderate?",
        "even when you approve a request to run outside it",
        "Not available in this build:",
        "Financial preview and approvals (PF-38, PF-39)",
        "Isolated retrieval (PF-31)",
        "Untrusted-content detectors (PF-35, PF-36)",
        "Agent Sweep (PF-40)",
        "Your config.toml is not modified.",
        "confirm and save",
    ] {
        assert!(review.contains(expected), "{expected}: {review}");
    }
    assert!(!review.contains("can run outside the sandbox"), "{review}");
    picker.handle_key_event(key(KeyCode::Esc));
    assert_eq!(
        picker.screen,
        Screen::List {
            note: Some("Cancelled. Nothing changed.".to_string())
        }
    );
    assert_eq!(level::load(home.path()), StoredLevel::Absent);
    assert_eq!(confirmed(home.path()), level::ConfirmedLevel::Absent);
}

#[test]
fn issue_750_confirming_moderate_saves_it_with_cores_record_and_offers_restart() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Permissive, ALL_ON);
    choose_moderate(&mut picker);
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(
        level::load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Moderate)
    );
    assert_eq!(
        confirmed(home.path()),
        level::ConfirmedLevel::Level("moderate".to_string())
    );
    insta::assert_snapshot!(
        "security_level_picker_saved_moderate",
        render(&picker, /*width*/ 80)
    );
    let saved = text(&picker);
    assert!(saved.contains("Saved: Moderate"), "{saved}");
    // #1047: this picker's window has no session yet (no thread): it starts
    // under Core's saved Moderate.
    assert!(
        saved.contains("This window's session has not started yet: when it starts here, it takes Core's Moderate rules"),
        "{saved}"
    );
    assert!(
        picker.footer().contains("r restart now"),
        "{}",
        picker.footer()
    );
    // Choosing it again changes nothing.
    picker.handle_key_event(key(KeyCode::Esc));
    let mut again = open_picker(home.path(), ChosenLevel::Permissive, ALL_ON);
    choose_moderate(&mut again);
    assert_eq!(
        again.screen,
        Screen::List {
            note: Some("Moderate is already saved. Nothing changed.".to_string())
        }
    );
}

/// D4: when the session's policy runs in this process, Core's rules apply as
/// soon as the person confirms, and the credential broker at the restart.
#[test]
fn issue_750_a_live_session_takes_cores_rules_now_and_the_broker_at_restart() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Permissive, ALL_ON);
    choose_moderate(&mut picker);
    picker.basis.live = true;
    let review = text(&picker);
    assert!(
        review.contains("Core's level becomes Moderate as soon as you confirm"),
        "{review}"
    );
    assert!(
        review.contains("Model keys through the credential broker start when you restart"),
        "{review}"
    );
    // #842: other Corbanu Terminals on the home take it too.
    assert!(
        review.contains(
            "within about a second in other Corbanu Terminals running on this Corbanu home"
        ),
        "{review}"
    );
    // What the live tree reports after the commit.
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Moderate,
        core: None,
    }));
    picker.stored = StoredLevel::Chosen(ChosenLevel::Moderate);
    picker.basis.in_force = SecurityLevel::Moderate;
    picker.basis.next_start = SecurityLevel::Moderate;
    let saved = text(&picker);
    assert!(
        saved.contains("Active now in this session: agent commands always stay in the sandbox"),
        "{saved}"
    );
    assert!(
        saved.contains(
            "Other Corbanu Terminals running on this Corbanu home take Core's Moderate rules"
        ),
        "{saved}"
    );
    assert!(
        saved.contains("Starts when you restart: model keys held by the credential broker"),
        "{saved}"
    );
    // #842 review: not saved, other Corbanu Terminals cannot take it.
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Moderate,
        core: Some(
            crate::legacy_core::security_level_change::LevelChangeReport {
                kind: crate::legacy_core::security_level_change::LevelChangeKind::Stricter,
                in_force: SecurityLevel::Moderate,
                next_start: SecurityLevel::Moderate,
                live: true,
                other_sessions: false,
                not_saved: Some("disk full".to_string()),
                approvals: None,
            },
        ),
    }));
    let unsaved = text(&picker);
    assert!(unsaved.contains("could not be saved"), "{unsaved}");
    assert!(!unsaved.contains("Other Corbanu Terminals"), "{unsaved}");
    picker.screen = Screen::List { note: None };
    let list = text(&picker);
    assert!(
        list.contains("Saved: Moderate. Core's Moderate rules apply in this session now"),
        "{list}"
    );

    // Back to Permissive before restarting: Core's Moderate stays on until
    // the restart, so Permissive is not "active" yet.
    picker.screen = Screen::Saved(Ok(TransitionOutcome {
        level: ChosenLevel::Permissive,
        core: None,
    }));
    picker.stored = StoredLevel::Chosen(ChosenLevel::Permissive);
    picker.basis.next_start = SecurityLevel::Permissive;
    let saved = text(&picker);
    assert!(
        saved.contains("Core's Moderate rules stay on in this session until you restart Corbanu Terminal; then Permissive applies."),
        "{saved}"
    );
    assert!(
        !saved.contains("Permissive is active in this session"),
        "{saved}"
    );
    assert!(
        picker.footer().contains("r restart now"),
        "{}",
        picker.footer()
    );
}

/// Core refuses every model request under Moderate without the
/// untrusted-content labels, so Moderate is not saved without them.
#[test]
fn issue_750_moderate_is_not_saved_without_the_untrusted_content_labels() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Permissive, Facts::default());
    choose_moderate(&mut picker);
    let review = text(&picker);
    assert!(
        review
            .contains("Moderate needs the untrusted-content labels (the source_envelopes feature)"),
        "{review}"
    );
    assert!(
        review.contains(
            "agent commands get your environment as configured (secretless_agent_launch off)"
        ),
        "{review}"
    );
    assert!(
        !picker.footer().contains("confirm and save"),
        "{}",
        picker.footer()
    );
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Moderate));
    assert!(text(&picker).contains("Not saved: Moderate needs"));
    assert_eq!(level::load(home.path()), StoredLevel::Absent);
    assert_eq!(confirmed(home.path()), level::ConfirmedLevel::Absent);
}

#[test]
fn issue_750_return_to_permissive_from_moderate() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Moderate, NestedAgents::Refuse).unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Moderate, ALL_ON);
    assert_eq!(ROWS[picker.selected], Row::Moderate);
    assert!(text(&picker).contains("> Moderate  (active)"));
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Permissive));
    insta::assert_snapshot!(
        "security_level_picker_review_permissive_from_moderate",
        render(&picker, /*width*/ 80)
    );
    let review = text(&picker);
    assert!(
        review.contains("Protections removed at the next start:"),
        "{review}"
    );
    assert!(
        review.contains("Takes effect when you restart Corbanu Terminal; Core's Moderate rules stay on in this session until then."),
        "{review}"
    );
    assert!(!review.contains("vault rule file"), "{review}");
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(
        level::load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Permissive)
    );
    let saved = text(&picker);
    assert!(saved.contains("Saved: Permissive"), "{saved}");
    assert!(
        saved.contains("Restart Corbanu Terminal to activate Permissive. Until then this session stays Moderate."),
        "{saved}"
    );
}

#[test]
fn issue_750_moderate_to_aggressive_and_back() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Moderate, NestedAgents::Refuse).unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Moderate, ALL_ON);
    picker.handle_key_event(key(KeyCode::Down));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(saved_level(&picker), Some(ChosenLevel::Aggressive));
    assert_eq!(
        level::load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Aggressive)
    );

    // From an active Aggressive back to Moderate: what goes at the restart.
    let mut picker = open_picker(home.path(), ChosenLevel::Aggressive, ALL_ON);
    choose_moderate(&mut picker);
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Moderate));
    let review = text(&picker);
    assert!(
        review.contains("Aggressive protections removed at the next start:"),
        "{review}"
    );
    assert!(
        review.contains("This session stays Aggressive until you restart Corbanu Terminal"),
        "{review}"
    );
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(saved_level(&picker), Some(ChosenLevel::Moderate));
    assert_eq!(
        level::load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Moderate)
    );
    // Aggressive's rule file stays until the next launch.
    assert!(level::rules_path(home.path()).exists());
    let saved = text(&picker);
    assert!(
        saved.contains("Restart Corbanu Terminal to activate Moderate. Until then this session stays Aggressive."),
        "{saved}"
    );
}

fn saved_level(picker: &SecurityLevelPicker) -> Option<ChosenLevel> {
    match &picker.screen {
        Screen::Saved(Ok(outcome)) => Some(outcome.level),
        _ => None,
    }
}

/// Review 1 (#750): Moderate that Core keeps (from config, or Core's record
/// over a Permissive level file after a save that did not finish) is the
/// next start's level: no "Saved: Permissive", no pointless restart, and
/// choosing Permissive opens the review that lowers Core.
#[test]
fn issue_750_moderate_kept_by_core_is_the_next_start() {
    let home = tempfile::tempdir().unwrap();
    for stored in [None, Some(ChosenLevel::Permissive)] {
        if let Some(level) = stored {
            level::save(home.path(), level, NestedAgents::Refuse).unwrap();
        }
        let mut picker = open_picker(home.path(), ChosenLevel::Moderate, ALL_ON);
        picker.basis.live = true;
        picker.basis.in_force = SecurityLevel::Moderate;
        picker.basis.next_start = SecurityLevel::Moderate;
        let list = text(&picker);
        assert!(!list.contains("Saved:"), "{stored:?}: {list}");
        assert!(
            !picker.footer().contains("r restart now"),
            "{stored:?}: {}",
            picker.footer()
        );
        while ROWS[picker.selected] != Row::Permissive {
            picker.handle_key_event(key(KeyCode::Up));
        }
        // Choosing reads Core's state again; keep the simulated live tree.
        picker.screen = picker.choose(Row::Permissive);
        assert_eq!(
            picker.screen,
            Screen::Review(ChosenLevel::Permissive),
            "{stored:?}"
        );
        assert!(
            text(&picker).contains("Protections removed at the next start:"),
            "{stored:?}"
        );
    }
}

/// #739 (option B): with protected launch on, the Permissive summary and its
/// review say that no command runs outside the sandbox, so none is offered;
/// without it they are unchanged.
#[test]
fn issue_739_permissive_says_protected_launch_keeps_commands_in_the_sandbox() {
    let home = tempfile::tempdir().unwrap();
    for (facts, protected) in [(ALL_ON, true), (Facts::default(), false)] {
        level::save(home.path(), ChosenLevel::Moderate, NestedAgents::Refuse).unwrap();
        let mut picker = open_picker(home.path(), ChosenLevel::Moderate, facts);
        while ROWS[picker.selected] != Row::Permissive {
            picker.handle_key_event(key(KeyCode::Up));
        }
        let list = text(&picker);
        assert_eq!(
            list.contains("agent commands never run outside the sandbox, even when you approve"),
            protected,
            "{list}"
        );
        assert_eq!(list.contains("Today's behaviour."), !protected, "{list}");
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Permissive));
        let review = text(&picker);
        assert_eq!(
            review.contains("so no approval to run outside it is offered"),
            protected,
            "{review}"
        );
        assert!(!review.contains("can run outside the sandbox"), "{review}");
        // Moderate's sandbox rule is listed as removed only without it.
        assert_eq!(
            review.contains("a command you approve to run outside the sandbox"),
            !protected,
            "{review}"
        );
    }
}

/// #783 (Travis, 2026-10-14): on Windows without its sandbox set up, Moderate
/// cannot be chosen either, exactly like Aggressive (#691): the list says so,
/// Enter explains how to set it up, and nothing is saved. Permissive stays.
#[test]
fn issue_783_moderate_waits_for_the_windows_sandbox() {
    const STEPS: &str = "The Windows sandbox is not set up. Open /permissions, choose Default \
                         and pick \"Set up default sandbox\" (it needs Administrator permission)";
    let home = tempfile::tempdir().unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Permissive, ALL_ON);
    picker.set_windows_sandbox_steps(Some(STEPS.to_string()));
    choose_moderate(&mut picker);
    insta::assert_snapshot!(
        "security_level_picker_moderate_needs_windows_sandbox",
        render(&picker, /*width*/ 80)
    );
    assert_eq!(
        picker.screen,
        Screen::List {
            note: Some(format!(
                "Moderate needs the default Windows sandbox. {STEPS}, then choose Moderate \
                 again. Nothing changed."
            ))
        }
    );
    assert_eq!(level::load(home.path()), StoredLevel::Absent);
    assert_eq!(confirmed(home.path()), level::ConfirmedLevel::Absent);

    // Permissive is still offered (and already in effect).
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(
        picker.screen,
        Screen::List {
            note: Some("Permissive is already in effect. Nothing changed.".to_string())
        }
    );

    // Once it is set up, Moderate's review opens as before.
    picker.set_windows_sandbox_steps(/*steps*/ None);
    choose_moderate(&mut picker);
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Moderate));
}

/// #783: with Moderate saved and active, its row says it needs the sandbox,
/// choosing it again changes nothing, and switching to Permissive still works.
#[test]
fn issue_783_saved_moderate_waits_for_the_windows_sandbox_and_permissive_stays() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Moderate, NestedAgents::Refuse).unwrap();
    let mut picker = open_picker(home.path(), ChosenLevel::Moderate, ALL_ON);
    picker.set_windows_sandbox_steps(Some("Run /setup-default-sandbox".to_string()));
    let list = text(&picker);
    assert!(
        list.contains("> Moderate  (active), needs the Windows sandbox set up"),
        "{list}"
    );
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(
        picker.screen,
        Screen::List {
            note: Some(
                "Moderate needs the default Windows sandbox. Run /setup-default-sandbox, then \
                 choose Moderate again. Nothing changed."
                    .to_string()
            )
        }
    );
    assert_eq!(
        level::load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Moderate)
    );
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Permissive));
}
