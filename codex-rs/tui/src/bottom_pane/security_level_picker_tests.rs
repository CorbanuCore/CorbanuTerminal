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
        preflight_enabled: false,
        boundary: None,
    }
}

/// Config C from the frozen code-blind design (case B-06), as the picker
/// would see it; `current_values` itself is covered in `aggressive_tests`.
fn current() -> CurrentValues {
    [
        "commands can write to the current folder, /tmp, /var/folders/tmp, /tmp/extra; approved or allow-listed commands can run outside the sandbox; permission-request tools are off",
        "on-request (reviewer: you)",
        "on for agent commands; web search live",
        "the vault store and sign-in file are readable to agent commands; secret-like environment variables are passed through (KEY, SECRET, TOKEN, VAULT, PASSWORD, PASSPHRASE, CREDENTIAL); login profiles or shell snapshots are used",
        "spawned agents get this session's values",
    ]
    .map(str::to_string)
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
        current(),
        /*preflight_input*/ None,
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
        current(),
        /*preflight_input*/ None,
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
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Aggressive),
        current(),
        /*preflight_input*/ None,
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
    // The session is still Aggressive, so its vault rule file stays until the
    // restart that activates Permissive (#203).
    assert_eq!(
        (
            level::load(home.path()),
            level::rules_path(home.path()).exists()
        ),
        (StoredLevel::Chosen(ChosenLevel::Permissive), true)
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
        current(),
        /*preflight_input*/ None,
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
        current(),
        /*preflight_input*/ None,
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

/// A review taller than the pane scrolls with the move keys, says so in the
/// footer, and stops at its last line (narrow or short terminals).
#[test]
fn tall_review_scrolls_to_its_last_line() {
    let home = tempfile::tempdir().unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Permissive),
        current(),
        /*preflight_input*/ None,
        RuntimeKeymap::defaults().list,
    );
    picker.handle_key_event(key(KeyCode::Up));
    picker.handle_key_event(key(KeyCode::Enter));
    let line_count = picker.lines(80).len();
    let body_height = 12;
    let max = (line_count - usize::from(body_height)) as u16;
    assert_eq!(picker.scroll_for(line_count, body_height), 0);
    for _ in 0..line_count * 2 {
        picker.handle_key_event(key(KeyCode::Down));
    }
    assert_eq!(picker.scroll_for(line_count, body_height), max);
    let mut visible = picker.lines(80).split_off(usize::from(max));
    visible.push(Line::from(picker.footer()));
    let area = Rect::new(0, 0, 80, visible.len() as u16);
    let mut buffer = Buffer::empty(area);
    Paragraph::new(visible).render(area, &mut buffer);
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
    insta::assert_snapshot!("security_level_picker_review_scrolled_to_end", rendered);

    picker.handle_key_event(key(KeyCode::Up));
    assert_eq!(
        (picker.scroll_for(line_count, body_height), &picker.screen),
        (max - 1, &Screen::Review(ChosenLevel::Aggressive))
    );
}

/// A saved Aggressive reopens its review so only a person in `/security` can
/// switch nested agent launches between refuse and pass.
#[test]
fn nested_agents_switch_from_the_aggressive_review() {
    let home = tempfile::tempdir().unwrap();
    level::save(home.path(), ChosenLevel::Aggressive, NestedAgents::Refuse).unwrap();
    let mut picker = SecurityLevelPicker::new(
        &context(home.path(), ChosenLevel::Aggressive),
        current(),
        /*preflight_input*/ None,
        RuntimeKeymap::defaults().list,
    );
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
    picker.handle_key_event(key(KeyCode::Enter));
    assert_eq!(
        picker.screen,
        Screen::List {
            note: Some("Aggressive is already saved. Nothing changed.".to_string())
        }
    );

    picker.handle_key_event(key(KeyCode::Enter));
    picker.handle_key_event(key(KeyCode::Char('n')));
    let review = render(&picker, 80);
    assert!(
        review.contains("Nested agents: pass.") && review.contains("n refuse/pass"),
        "{review}"
    );
    picker.handle_key_event(key(KeyCode::Enter));
    insta::assert_snapshot!(
        "security_level_picker_saved_nested_pass",
        render(&picker, 80)
    );
    assert_eq!(
        level::load_state(home.path()),
        (
            StoredLevel::Chosen(ChosenLevel::Aggressive),
            NestedAgents::Pass
        )
    );
}

mod pf_29_s01 {
    use super::*;
    use crate::legacy_core::protected_preflight::InventorySources;
    use crate::legacy_core::protected_preflight::ReadinessFlags;
    use pretty_assertions::assert_eq;

    struct Machine {
        root: tempfile::TempDir,
    }

    impl Machine {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            for dir in ["corbanu", "home", "work"] {
                std::fs::create_dir_all(root.path().join(dir)).unwrap();
            }
            Self { root }
        }

        fn corbanu(&self) -> std::path::PathBuf {
            self.root.path().join("corbanu")
        }

        fn home(&self) -> std::path::PathBuf {
            self.root.path().join("home")
        }

        fn picker(&self, flags_on: bool, active: ChosenLevel) -> SecurityLevelPicker {
            let input = PreflightInput {
                sources: InventorySources {
                    codex_home: self.corbanu(),
                    home: Some(self.home()),
                    cwd: self.root.path().join("work"),
                    env: Vec::new(),
                    config_layers: Vec::new(),
                },
                flags: ReadinessFlags {
                    secretless_launch: flags_on,
                    credential_broker: flags_on,
                    output_gate: flags_on,
                },
            };
            let mut context = context(&self.corbanu(), active);
            context.preflight_enabled = true;
            SecurityLevelPicker::new(
                &context,
                current(),
                Some(input),
                RuntimeKeymap::defaults().list,
            )
        }
    }

    fn review_aggressive(picker: &mut SecurityLevelPicker) {
        picker.handle_key_event(key(KeyCode::Up));
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
    }

    fn text(picker: &SecurityLevelPicker) -> String {
        render(picker, 400)
    }

    #[test]
    fn pf_29_s01_blockers_keep_aggressive_unsaved() {
        let machine = Machine::new();
        let mut picker = machine.picker(/*flags_on*/ false, ChosenLevel::Permissive);
        review_aggressive(&mut picker);
        let review = text(&picker);
        assert!(review.contains("Preflight blocked"), "{review}");
        assert!(review.contains("turn on the `secretless_agent_launch` feature"));
        assert!(!review.contains("confirm and save"));

        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        assert!(text(&picker).contains("Not saved: resolve the blockers above first."));
        assert_eq!(files(&machine.corbanu()), Vec::<String>::new());

        picker.handle_key_event(key(KeyCode::Esc));
        assert_eq!(files(&machine.corbanu()), Vec::<String>::new());
    }

    #[test]
    fn pf_29_s01_clean_preflight_saves_level_and_receipt() {
        let machine = Machine::new();
        std::fs::write(machine.home().join(".netrc"), "machine x password fake").unwrap();
        let mut picker = machine.picker(/*flags_on*/ true, ChosenLevel::Permissive);
        review_aggressive(&mut picker);
        let review = text(&picker);
        assert!(review.contains("Preflight passed"), "{review}");
        assert!(
            review.contains("Credential files denied to agent commands after restart: ~/.netrc")
        );
        assert!(!review.contains("fake"));

        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Saved(Ok(ChosenLevel::Aggressive)));
        assert_eq!(
            level::load(&machine.corbanu()),
            StoredLevel::Chosen(ChosenLevel::Aggressive)
        );
        let receipt = preflight::load_receipt(&machine.corbanu())
            .unwrap()
            .unwrap();
        assert_eq!(receipt.findings.len(), 1);
        assert!(
            !std::fs::read_to_string(preflight::receipt_path(&machine.corbanu()))
                .unwrap()
                .contains("netrc"),
            "receipt holds IDs, not paths"
        );
    }

    #[test]
    fn pf_29_s01_drift_after_review_refuses_save() {
        let machine = Machine::new();
        std::fs::write(machine.home().join(".netrc"), "one").unwrap();
        let mut picker = machine.picker(/*flags_on*/ true, ChosenLevel::Permissive);
        review_aggressive(&mut picker);
        std::fs::write(machine.home().join(".netrc"), "two").unwrap();
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        assert!(text(&picker).contains("Not saved: 1 item changed since you reviewed this."));
        assert_eq!(level::load(&machine.corbanu()), StoredLevel::Absent);

        // The refreshed review can now be confirmed.
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Saved(Ok(ChosenLevel::Aggressive)));
    }

    /// Toggling nested agents on a saved Aggressive skips the preflight only
    /// while a receipt is on record (review 3).
    #[test]
    fn pf_29_s01_saved_aggressive_without_receipt_needs_a_preflight() {
        let machine = Machine::new();
        level::save(
            &machine.corbanu(),
            ChosenLevel::Aggressive,
            NestedAgents::Refuse,
        )
        .unwrap();
        let mut picker = machine.picker(/*flags_on*/ true, ChosenLevel::Aggressive);
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        picker.handle_key_event(key(KeyCode::Char('n')));
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        assert!(text(&picker).contains("no preflight on record"));
        assert!(!preflight::receipt_path(&machine.corbanu()).exists());

        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Saved(Ok(ChosenLevel::Aggressive)));
        assert!(preflight::receipt_path(&machine.corbanu()).exists());
    }

    #[test]
    fn pf_29_s01_returning_to_permissive_removes_the_receipt() {
        let machine = Machine::new();
        let mut picker = machine.picker(/*flags_on*/ true, ChosenLevel::Permissive);
        review_aggressive(&mut picker);
        picker.handle_key_event(key(KeyCode::Enter));
        assert!(preflight::receipt_path(&machine.corbanu()).exists());

        let mut picker = machine.picker(/*flags_on*/ true, ChosenLevel::Aggressive);
        picker.handle_key_event(key(KeyCode::Down));
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Permissive));
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Saved(Ok(ChosenLevel::Permissive)));
        assert!(!preflight::receipt_path(&machine.corbanu()).exists());
    }
}

mod pf_29_s02 {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;
    use zeroize::Zeroizing;

    use super::*;
    use crate::legacy_core::protected_preflight::InventorySources;
    use crate::legacy_core::protected_preflight::ReadinessFlags;
    use crate::legacy_core::protected_preflight::migration::journal_path;

    const VALUE: &str = "fake-openai-value-0001";

    #[derive(Default)]
    struct FakeStore {
        values: RefCell<BTreeMap<String, String>>,
        locked: Cell<bool>,
    }

    impl CredentialStore for FakeStore {
        fn put(&self, label: &str, _name: &str, _origin: &str, value: &str) -> Result<(), String> {
            if self.locked.get() {
                return Err("the vault is locked".to_string());
            }
            self.values
                .borrow_mut()
                .insert(label.to_string(), value.to_string());
            Ok(())
        }

        fn get(&self, label: &str) -> Result<Option<Zeroizing<String>>, String> {
            Ok(self.values.borrow().get(label).cloned().map(Zeroizing::new))
        }
    }

    struct Machine {
        root: tempfile::TempDir,
        store: Rc<FakeStore>,
    }

    impl Machine {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            for dir in ["corbanu", "home", "work"] {
                std::fs::create_dir_all(root.path().join(dir)).unwrap();
            }
            std::fs::write(
                root.path().join("home/.zshrc"),
                format!("alias ll='ls -l'\nexport OPENAI_API_KEY={VALUE}\n"),
            )
            .unwrap();
            Self {
                root,
                store: Rc::new(FakeStore::default()),
            }
        }

        fn path(&self, name: &str) -> std::path::PathBuf {
            self.root.path().join(name)
        }

        fn zshrc(&self) -> String {
            std::fs::read_to_string(self.path("home/.zshrc")).unwrap()
        }

        fn picker(&self) -> SecurityLevelPicker {
            let input = PreflightInput {
                sources: InventorySources {
                    codex_home: self.path("corbanu"),
                    home: Some(self.path("home")),
                    cwd: self.path("work"),
                    env: Vec::new(),
                    config_layers: Vec::new(),
                },
                flags: ReadinessFlags {
                    secretless_launch: true,
                    credential_broker: true,
                    output_gate: true,
                },
            };
            let mut context = context(&self.path("corbanu"), ChosenLevel::Permissive);
            context.preflight_enabled = true;
            let mut picker = SecurityLevelPicker::new(
                &context,
                current(),
                Some(input),
                RuntimeKeymap::defaults().list,
            );
            picker.store = Some(self.store.clone());
            picker.handle_key_event(key(KeyCode::Up));
            picker.handle_key_event(key(KeyCode::Enter));
            assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
            picker
        }
    }

    fn text(picker: &SecurityLevelPicker) -> String {
        render(picker, 400)
    }

    #[test]
    fn pf_29_s02_preview_then_esc_changes_nothing() {
        let machine = Machine::new();
        let before = machine.zshrc();
        let mut picker = machine.picker();
        assert!(picker.footer().contains("m move credentials to the vault"));
        picker.handle_key_event(key(KeyCode::Char('m')));
        assert_eq!(picker.screen, Screen::MigrationPreview);
        let preview = text(&picker);
        assert!(
            preview.contains("~/.zshrc:2 OPENAI_API_KEY → vault label migrated/openai_api_key")
        );
        assert!(preview.contains("readable only by you (0600)"));
        assert!(preview.contains("Rotate every value"));
        assert!(!preview.contains(VALUE));

        picker.handle_key_event(key(KeyCode::Esc));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        assert!(text(&picker).contains("Migration cancelled. Nothing changed."));
        assert_eq!(machine.zshrc(), before);
        assert!(machine.store.values.borrow().is_empty());
        assert!(!journal_path(&machine.path("corbanu")).exists());
    }

    #[test]
    fn pf_29_s02_confirm_moves_reaudits_and_then_saves() {
        let machine = Machine::new();
        let mut picker = machine.picker();
        picker.handle_key_event(key(KeyCode::Char('m')));
        picker.handle_key_event(key(KeyCode::Enter));
        let result = text(&picker);
        assert!(
            result.contains("Moved 1 credential into the vault"),
            "{result}"
        );
        assert!(result.contains("Rotate at the provider: OPENAI_API_KEY"));
        assert!(!result.contains(VALUE));
        assert_eq!(
            machine.zshrc(),
            "alias ll='ls -l'\nexport OPENAI_API_KEY=\"$(corbanu vault auth-helper migrated/openai_api_key)\"\n"
        );

        picker.handle_key_event(key(KeyCode::Enter));
        let review = text(&picker);
        assert!(review.contains("Preflight passed"), "{review}");
        assert!(review.contains("Moved 1 credential into the vault. Rotate OPENAI_API_KEY"));
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Saved(Ok(ChosenLevel::Aggressive)));
        assert!(preflight::receipt_path(&machine.path("corbanu")).exists());
    }

    #[test]
    fn pf_29_s02_failure_locks_aggressive_until_recovery() {
        let machine = Machine::new();
        machine.store.locked.set(true);
        let mut picker = machine.picker();
        picker.handle_key_event(key(KeyCode::Char('m')));
        picker.handle_key_event(key(KeyCode::Enter));
        assert!(text(&picker).contains("Migration stopped"));
        picker.handle_key_event(key(KeyCode::Enter));
        let review = text(&picker);
        assert!(
            review.contains("a credential migration did not finish"),
            "{review}"
        );
        assert!(picker.footer().contains("r finish the migration"));
        // Saving stays refused while the migration is unfinished.
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        assert_eq!(level::load(&machine.path("corbanu")), StoredLevel::Absent);

        machine.store.locked.set(false);
        picker.handle_key_event(key(KeyCode::Char('r')));
        assert!(text(&picker).contains("Moved 1 credential into the vault"));
        picker.handle_key_event(key(KeyCode::Enter));
        assert!(text(&picker).contains("Preflight passed"));
        assert!(!journal_path(&machine.path("corbanu")).exists());
    }

    #[test]
    fn pf_29_s02_change_after_preview_moves_nothing() {
        let machine = Machine::new();
        let mut picker = machine.picker();
        picker.handle_key_event(key(KeyCode::Char('m')));
        std::fs::write(
            machine.path("home/.zshrc"),
            "export OPENAI_API_KEY=fake-other-value-0002\n",
        )
        .unwrap();
        picker.handle_key_event(key(KeyCode::Enter));
        assert_eq!(picker.screen, Screen::Review(ChosenLevel::Aggressive));
        assert!(text(&picker).contains("Not moved: something changed since the preview"));
        assert!(machine.store.values.borrow().is_empty());
        assert_eq!(
            machine.zshrc(),
            "export OPENAI_API_KEY=fake-other-value-0002\n"
        );
    }
}
