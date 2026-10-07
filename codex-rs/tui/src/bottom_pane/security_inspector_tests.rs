use codex_protocol::ThreadId;
use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::keymap::RuntimeKeymap;
use crate::legacy_core::security_inspection::ContractFacts;
use crate::legacy_core::security_inspection::ControlFacts;
use crate::legacy_core::security_inspection::SecurityLevel;
use crate::legacy_core::security_inspection::TaintFacts;
use crate::security::inspector::tests::NOW;
use crate::security::inspector::tests::healthy;
use crate::security::inspector::tests::input;
use crate::security::inspector::tests::saved;
use crate::security::level::ChosenLevel;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn at_now() -> i64 {
    NOW
}

fn stale_now() -> i64 {
    NOW + 120
}

fn inspector(facts: RuntimeFacts, clock: fn() -> i64) -> SecurityInspector {
    let mut inspector = SecurityInspector::with_clock(
        input(ChosenLevel::Aggressive),
        RuntimeKeymap::defaults().list,
        clock,
    );
    inspector.saved = saved(ChosenLevel::Aggressive);
    inspector.facts = facts;
    inspector
}

/// The whole inspector, in a pane tall enough not to scroll.
fn render(inspector: &SecurityInspector, width: u16) -> String {
    let (header, body) = inspector.content(width);
    let footer = inspector.footer(width);
    let area = Rect::new(
        0,
        0,
        width,
        (header.len() + body.len() + footer.len()) as u16,
    );
    let mut buffer = Buffer::empty(area);
    inspector.render(area, &mut buffer);
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
fn pf_41_s01_inspector_healthy_aggressive() {
    let inspector = inspector(healthy(ThreadId::new()), at_now);
    insta::assert_snapshot!(
        "pf_41_s01_inspector_healthy_aggressive",
        render(&inspector, /*width*/ 100)
    );
}

#[test]
fn pf_41_s01_inspector_degraded_and_stale() {
    let facts = RuntimeFacts {
        policy: PolicyFacts::Stored {
            level: SecurityLevel::Aggressive,
            kill_switch: false,
            unreadable: false,
        },
        taint: TaintFacts::Generation(2),
        launch_contract: ContractFacts::Armed { hardened: false },
        output_gate: ControlFacts::Off,
        model_broker: ControlFacts::Degraded("no broker is running; provider keys are not sent"),
        ..healthy(ThreadId::new())
    };
    insta::assert_snapshot!(
        "pf_41_s01_inspector_degraded_stale",
        render(&inspector(facts, stale_now), /*width*/ 100)
    );
}

#[test]
fn pf_41_s01_inspector_blocked_by_unreadable_state() {
    let facts = RuntimeFacts {
        policy: PolicyFacts::Stored {
            level: SecurityLevel::Aggressive,
            kill_switch: true,
            unreadable: true,
        },
        ..healthy(ThreadId::new())
    };
    let rendered = render(&inspector(facts, at_now), /*width*/ 100);
    insta::assert_snapshot!(
        "pf_41_s01_inspector_blocked",
        rendered.lines().take(8).collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn pf_41_s01_inspector_keys_only_read() {
    let mut inspector = inspector(healthy(ThreadId::new()), at_now);
    let home = tempfile::TempDir::new().unwrap();
    inspector.input.codex_home = home.path().to_path_buf();
    for code in [
        KeyCode::Enter,
        KeyCode::Char('y'),
        KeyCode::Char('g'),
        KeyCode::Char('k'),
        KeyCode::Down,
        KeyCode::Up,
    ] {
        inspector.handle_key_event(key(code));
        assert!(!inspector.closed, "{code:?}");
    }
    // `r` reads again: here the live facts of this test process.
    inspector.handle_key_event(key(KeyCode::Char('r')));
    assert_eq!(
        inspector.facts.policy,
        PolicyFacts::Stored {
            level: SecurityLevel::Permissive,
            kill_switch: false,
            unreadable: false,
        }
    );
    // Nothing was written to the Corbanu home.
    assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
    inspector.handle_key_event(key(KeyCode::Esc));
    assert!(inspector.closed);
}

#[test]
fn pf_41_s01_inspector_scrolls_under_a_pinned_header() {
    let mut inspector = inspector(healthy(ThreadId::new()), at_now);
    let height = inspector.desired_height(/*width*/ 100);
    let area = Rect::new(0, 0, 100, height);
    let screen = |inspector: &SecurityInspector| {
        let mut buffer = Buffer::empty(area);
        inspector.render(area, &mut buffer);
        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
    };
    for _ in 0..200 {
        inspector.handle_key_event(key(KeyCode::Down));
        screen(&inspector);
    }
    let bottom = screen(&inspector);
    assert!(bottom[0].starts_with("Security inspector (read only)"));
    assert!(bottom[1].starts_with("● Protected: Aggressive"));
    assert!(bottom.iter().any(|line| line.contains("Recent denials")));
    assert!(bottom[bottom.len() - 1].contains("esc back to /security"));
}
