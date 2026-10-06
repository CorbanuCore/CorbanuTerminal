use crossterm::event::KeyModifiers;
use pretty_assertions::assert_eq;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::*;
use crate::keymap::RuntimeKeymap;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn context(home: &std::path::Path, active: ChosenLevel) -> LevelContext {
    LevelContext {
        codex_home: home.to_path_buf(),
        picker_enabled: true,
        active,
    }
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

fn files(home: &std::path::Path) -> Vec<String> {
    let mut names = walk(home);
    names.sort();
    names
}

fn walk(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flat_map(|entry| {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        walk(&path)
                    } else {
                        vec![path.display().to_string()]
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn list_review_and_saved_screens() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Permissive),
        RuntimeKeymap::defaults().list,
    );
    insta::assert_snapshot!("security_level_picker_list", render(&picker, 80));
    picker.handle_key_event(key(KeyCode::Down));
    picker.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!(
        "security_level_picker_moderate_unavailable",
        render(&picker, 80)
    );
    picker.handle_key_event(key(KeyCode::Down));
    picker.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!(
        "security_level_picker_review_aggressive",
        render(&picker, 80)
    );
    picker.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!(
        "security_level_picker_saved_aggressive",
        render(&picker, 80)
    );
    assert_eq!(
        level::load(home.path()),
        StoredLevel::Chosen(ChosenLevel::Aggressive)
    );
    picker.handle_key_event(key(KeyCode::Enter));
    assert!(picker.closed);
}

#[test]
fn escape_and_typed_text_change_nothing() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Permissive),
        RuntimeKeymap::defaults().list,
    );
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
    for typed in ['y', 'p', 'a', ' ', '1'] {
        picker.handle_key_event(key(KeyCode::Char(typed)));
    }
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
    picker.handle_key_event(key(KeyCode::Esc));
    assert_eq!(
        picker.screen,
        Screen::List {
            note: Some("Cancelled. Nothing changed.".to_string())
        }
    );
    picker.handle_key_event(key(KeyCode::Esc));
    assert_eq!(
        (picker.closed, level::load(home.path()), files(home.path())),
        (true, StoredLevel::Absent, Vec::<String>::new())
    );
}

#[test]
fn return_to_permissive_while_aggressive_is_active() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive).unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Aggressive),
        RuntimeKeymap::defaults().list,
    );
    assert_eq!(picker.selected, 2);
    picker.handle_key_event(key(KeyCode::Down));
    picker.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!(
        "security_level_picker_review_permissive",
        render(&picker, 80)
    );
    picker.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!(
        "security_level_picker_saved_permissive_pending",
        render(&picker, 80)
    );
    assert_eq!(
        (
            level::load(home.path()),
            level::rules_path(home.path()).exists()
        ),
        (StoredLevel::Chosen(ChosenLevel::Permissive), false)
    );
}

#[test]
fn unreadable_state_is_reported_and_aggressive_selected() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        level::state_path(home.path()),
        "version = 1\nlevel = \"max\"\n",
    )
    .unwrap();
    let picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Aggressive),
        RuntimeKeymap::defaults().list,
    );
    let rendered = render(&picker, 400).replace(&home.path().display().to_string(), "<home>");
    insta::assert_snapshot!("security_level_picker_unreadable", rendered);
}

#[test]
fn choosing_the_saved_level_is_a_no_op() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Permissive),
        RuntimeKeymap::defaults().list,
    );
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(
        (picker.screen.clone(), files(home.path())),
        (
            Screen::List {
                note: Some("Permissive is already in effect. Nothing changed.".to_string())
            },
            Vec::<String>::new()
        )
    );
}
