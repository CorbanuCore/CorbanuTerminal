//! Flagged `/security` picker (PF-24-S03). A human chooses Permissive or
//! Aggressive; the differences are shown first and only the accept key saves.
//! A saved level takes effect at the next start, where it is verified before
//! being shown as active.

use std::cell::Cell;
use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::key_hint;
use crate::key_hint::KeyBindingListExt;
use crate::keymap::ListKeymap;
use crate::legacy_core::protected_preflight::Disposition;
use crate::legacy_core::protected_preflight::FindingKind;
use crate::legacy_core::protected_preflight::LIMITS;
use crate::legacy_core::protected_preflight::Preflight;
use crate::legacy_core::protected_preflight::migration;
use crate::legacy_core::protected_preflight::migration::CredentialStore;
use crate::legacy_core::protected_preflight::migration::MigrationOutcome;
use crate::legacy_core::protected_preflight::migration::MigrationPlan;
use crate::security::aggressive;
use crate::security::current::CurrentValues;
use crate::security::level;
use crate::security::level::ChosenLevel;
use crate::security::level::LevelContext;
use crate::security::level::NestedAgents;
use crate::security::level::StoredLevel;
use crate::security::migration::VaultStore;
use crate::security::preflight;
use crate::security::preflight::PreflightInput;
use crate::wrapping::RtOptions;
use crate::wrapping::word_wrap_lines;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Permissive,
    Moderate,
    Aggressive,
}

const ROWS: [Row; 3] = [Row::Permissive, Row::Moderate, Row::Aggressive];

impl Row {
    fn chosen(self) -> Option<ChosenLevel> {
        match self {
            Self::Permissive => Some(ChosenLevel::Permissive),
            Self::Moderate => None,
            Self::Aggressive => Some(ChosenLevel::Aggressive),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Permissive => "Permissive",
            Self::Moderate => "Moderate",
            Self::Aggressive => "Aggressive",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Screen {
    List {
        note: Option<String>,
    },
    Review(ChosenLevel),
    Saved(Result<ChosenLevel, String>),
    /// PF-29-S02: what a credential migration would do.
    MigrationPreview,
    MigrationResult(Result<MigrationOutcome, String>),
}

pub(crate) struct SecurityLevelPicker {
    codex_home: PathBuf,
    active: ChosenLevel,
    current: CurrentValues,
    stored: StoredLevel,
    /// Stored nested-launch setting.
    nested: NestedAgents,
    /// The nested-launch setting the Aggressive review will save.
    nested_choice: NestedAgents,
    selected: usize,
    screen: Screen,
    keymap: ListKeymap,
    /// Rows scrolled off the top of a review that is taller than the pane.
    scroll: u16,
    /// Largest useful `scroll`, recorded by the last render.
    max_scroll: Cell<u16>,
    pub(super) closed: bool,
    /// PF-29-S01: present when `protected_mode_preflight` is on.
    preflight_input: Option<PreflightInput>,
    preflight: Option<Preflight>,
    preflight_note: Option<String>,
    /// PF-29-S02: the previewed plan, and a store tests can replace.
    migration: Option<MigrationPlan>,
    pub(super) store: Option<Rc<dyn CredentialStore>>,
}

impl SecurityLevelPicker {
    pub(crate) fn new(
        context: &LevelContext,
        current: CurrentValues,
        preflight_input: Option<PreflightInput>,
        keymap: ListKeymap,
    ) -> Self {
        let (stored, nested) = level::load_state(&context.codex_home);
        let selected = match stored.enforced() {
            ChosenLevel::Permissive => 0,
            ChosenLevel::Aggressive => 2,
        };
        Self {
            codex_home: context.codex_home.clone(),
            active: context.active,
            current,
            stored,
            nested,
            nested_choice: nested,
            selected,
            screen: Screen::List { note: None },
            keymap,
            scroll: 0,
            max_scroll: Cell::new(0),
            closed: false,
            preflight_input,
            preflight: None,
            preflight_note: None,
            migration: None,
            store: None,
        }
    }

    pub(crate) fn handle_key_event(&mut self, key: KeyEvent) {
        let cancel =
            key_hint::plain(KeyCode::Esc).is_press(key) || self.keymap.cancel.is_pressed(key);
        let accept = self.keymap.accept.is_pressed(key);
        match self.screen.clone() {
            Screen::List { .. } => {
                if cancel {
                    self.closed = true;
                } else if self.keymap.move_up.is_pressed(key) {
                    self.selected = (self.selected + ROWS.len() - 1) % ROWS.len();
                    self.screen = Screen::List { note: None };
                } else if self.keymap.move_down.is_pressed(key) {
                    self.selected = (self.selected + 1) % ROWS.len();
                    self.screen = Screen::List { note: None };
                } else if accept {
                    self.screen = self.choose(ROWS[self.selected]);
                    self.nested_choice = self.nested;
                    self.scroll = 0;
                    self.preflight_note = None;
                    // Only a move to Aggressive is a transition; a saved
                    // Aggressive reopens its review for the nested setting.
                    self.preflight = match (&self.screen, &self.preflight_input) {
                        (Screen::Review(ChosenLevel::Aggressive), Some(input))
                            if self.stored != StoredLevel::Chosen(ChosenLevel::Aggressive) =>
                        {
                            Some(Preflight::run(&input.sources, input.flags))
                        }
                        _ => None,
                    };
                }
            }
            Screen::Review(target) => {
                if self.keymap.move_up.is_pressed(key) {
                    self.scroll = self.scroll.min(self.max_scroll.get()).saturating_sub(1);
                } else if self.keymap.move_down.is_pressed(key) {
                    self.scroll = (self.scroll + 1).min(self.max_scroll.get());
                } else if cancel {
                    self.screen = Screen::List {
                        note: Some("Cancelled. Nothing changed.".to_string()),
                    };
                } else if target == ChosenLevel::Aggressive
                    && key_hint::plain(KeyCode::Char('n')).is_press(key)
                {
                    self.nested_choice = self.nested_choice.toggled();
                } else if target == ChosenLevel::Aggressive
                    && key_hint::plain(KeyCode::Char('m')).is_press(key)
                    && self.migration_available()
                {
                    self.open_migration();
                } else if target == ChosenLevel::Aggressive
                    && key_hint::plain(KeyCode::Char('r')).is_press(key)
                    && self.migration_unfinished()
                {
                    let result = migration::recover(&self.codex_home, &*self.store())
                        .map(Option::unwrap_or_default)
                        .map_err(|err| err.to_string());
                    self.screen = Screen::MigrationResult(result);
                } else if accept
                    && self.stored == StoredLevel::Chosen(target)
                    && self.nested_choice == self.nested
                {
                    self.screen = Screen::List {
                        note: Some(format!(
                            "{} is already saved. Nothing changed.",
                            target.name()
                        )),
                    };
                } else if accept {
                    if !self.preflight_allows_save(target) {
                        return;
                    }
                    let result = self.save(target);
                    (self.stored, self.nested) = level::load_state(&self.codex_home);
                    self.screen = Screen::Saved(result);
                }
            }
            Screen::Saved(_) => {
                if cancel || accept {
                    self.closed = true;
                }
            }
            Screen::MigrationPreview => {
                if self.keymap.move_up.is_pressed(key) {
                    self.scroll = self.scroll.min(self.max_scroll.get()).saturating_sub(1);
                } else if self.keymap.move_down.is_pressed(key) {
                    self.scroll = (self.scroll + 1).min(self.max_scroll.get());
                } else if cancel {
                    self.migration = None;
                    self.preflight_note = Some("Migration cancelled. Nothing changed.".to_string());
                    self.screen = Screen::Review(ChosenLevel::Aggressive);
                    self.scroll = 0;
                } else if accept {
                    self.confirm_migration();
                }
            }
            Screen::MigrationResult(_) => {
                if cancel || accept {
                    self.back_to_review_after_migration();
                }
            }
        }
    }

    fn store(&self) -> Rc<dyn CredentialStore> {
        self.store
            .clone()
            .unwrap_or_else(|| Rc::new(VaultStore::new(&self.codex_home)))
    }

    /// A blocker this build can move into the vault.
    fn migration_available(&self) -> bool {
        self.preflight.as_ref().is_some_and(|preflight| {
            !preflight.inventory.migration_unfinished
                && preflight
                    .inventory
                    .blocking()
                    .any(|finding| finding.kind == FindingKind::ShellProfileExport)
        })
    }

    fn migration_unfinished(&self) -> bool {
        self.preflight
            .as_ref()
            .is_some_and(|preflight| preflight.inventory.migration_unfinished)
    }

    fn open_migration(&mut self) {
        let Some(preflight) = &self.preflight else {
            return;
        };
        match self.store().labels() {
            Ok(taken) => {
                self.migration = Some(MigrationPlan::from_preflight(preflight, &taken));
                self.screen = Screen::MigrationPreview;
                self.scroll = 0;
            }
            Err(err) => {
                self.preflight_note = Some(format!(
                    "The vault cannot be read ({err}). Nothing changed."
                ));
            }
        }
    }

    /// Confirm: recheck the reviewed preflight, then move the values.
    fn confirm_migration(&mut self) {
        let (Some(input), Some(reviewed), Some(plan)) = (
            &self.preflight_input,
            &self.preflight,
            self.migration.clone(),
        ) else {
            return;
        };
        let (next, drift) = reviewed.recheck(&input.sources, input.flags);
        if !drift.is_empty() {
            self.preflight = Some(next);
            self.migration = None;
            self.preflight_note = Some(
                "Not moved: something changed since the preview. Review it again; nothing changed."
                    .to_string(),
            );
            self.screen = Screen::Review(ChosenLevel::Aggressive);
            self.scroll = 0;
            return;
        }
        let result = migration::run(
            &self.codex_home,
            &plan,
            &*self.store(),
            crate::security::migration::injected_failure(),
        )
        .map_err(|err| err.to_string());
        self.screen = Screen::MigrationResult(result);
    }

    /// Back to the review with a fresh preflight (the re-audit).
    fn back_to_review_after_migration(&mut self) {
        let note = match &self.screen {
            Screen::MigrationResult(Ok(outcome)) if outcome.moved.is_empty() => {
                "Nothing was moved.".to_string()
            }
            Screen::MigrationResult(Ok(outcome)) => format!(
                "Moved {} credential{} into the vault. Rotate {} at the provider. Check the preflight below before saving Aggressive.",
                outcome.moved.len(),
                if outcome.moved.len() == 1 { "" } else { "s" },
                outcome.rotate.join(", ")
            ),
            _ => {
                "The migration stopped; Aggressive stays unsaved. Press r to finish it.".to_string()
            }
        };
        if let Some(input) = &self.preflight_input {
            self.preflight = Some(Preflight::run(&input.sources, input.flags));
        }
        self.migration = None;
        self.preflight_note = Some(note);
        self.screen = Screen::Review(ChosenLevel::Aggressive);
        self.scroll = 0;
    }

    /// Aggressive under the preflight flag: the review must have been clean
    /// and nothing may have changed since. Otherwise stay on the review.
    fn preflight_allows_save(&mut self, target: ChosenLevel) -> bool {
        let (Some(input), ChosenLevel::Aggressive) = (&self.preflight_input, target) else {
            return true;
        };
        // No preflight: Aggressive was saved with a receipt and only the
        // nested setting changes, which is not a transition. Read both again:
        // another Corbanu process may have changed them since this opened.
        let Some(reviewed) = &self.preflight else {
            let (stored, _) = level::load_state(&self.codex_home);
            if stored == StoredLevel::Chosen(ChosenLevel::Aggressive)
                && preflight::receipt_path(&self.codex_home).exists()
            {
                return true;
            }
            self.preflight = Some(Preflight::run(&input.sources, input.flags));
            self.preflight_note = Some(
                "Not saved: Aggressive has no preflight on record. Review the preflight above."
                    .to_string(),
            );
            return false;
        };
        let (next, drift) = reviewed.recheck(&input.sources, input.flags);
        let note = if !drift.is_empty() {
            let count = drift.added.len() + drift.removed.len() + drift.changed.len();
            Some(format!(
                "Not saved: {count} item{} changed since you reviewed this{}. Review it again.",
                if count == 1 { "" } else { "s" },
                if drift.readiness_changed {
                    ", and readiness changed"
                } else {
                    ""
                }
            ))
        } else if !next.is_clean() {
            Some("Not saved: resolve the blockers above first. Nothing changed.".to_string())
        } else {
            None
        };
        self.preflight = Some(next);
        self.preflight_note = note;
        self.preflight_note.is_none()
    }

    fn save(&self, target: ChosenLevel) -> Result<ChosenLevel, String> {
        // The receipt goes first: without Aggressive stored it is ignored,
        // while Aggressive without it would start unverified.
        match (target, &self.preflight) {
            (ChosenLevel::Aggressive, Some(passed)) => {
                preflight::save_receipt(&self.codex_home, passed).map_err(|err| err.to_string())?;
            }
            (ChosenLevel::Aggressive, None) => {}
            (ChosenLevel::Permissive, _) => {
                preflight::remove_receipt(&self.codex_home).map_err(|err| err.to_string())?;
            }
        }
        level::save(&self.codex_home, target, self.nested_choice)
            .map(|()| target)
            .map_err(|err| {
                // Never leave a receipt for a level that was not saved.
                if target == ChosenLevel::Aggressive && self.preflight.is_some() {
                    let _ = preflight::remove_receipt(&self.codex_home);
                }
                err.to_string()
            })
    }

    fn preflight_lines(&self, preflight: &Preflight) -> Vec<String> {
        let ready = preflight
            .readiness
            .iter()
            .filter(|item| item.is_ready())
            .count();
        let mut lines = vec![if preflight.is_clean() {
            "Preflight passed: no known raw-secret route stays open to agents or the model."
                .to_string()
        } else {
            "Preflight blocked: Aggressive cannot be saved until these are resolved.".to_string()
        }];
        lines.push(format!(
            "Controls ready: {ready} of {}",
            preflight.readiness.len()
        ));
        lines.extend(
            preflight
                .blockers()
                .into_iter()
                .map(|line| format!("✗ {line}")),
        );
        // Credential files by location; Corbanu's own history and state by name.
        let (own, credentials): (Vec<_>, Vec<_>) = preflight
            .inventory
            .findings
            .iter()
            .filter(|finding| finding.disposition == Disposition::Isolate)
            .partition(|finding| {
                finding
                    .paths
                    .first()
                    .is_some_and(|path| path.starts_with(&self.codex_home))
            });
        if !credentials.is_empty() {
            lines.push(format!(
                "Credential files denied to agent commands after restart: {}",
                credentials
                    .iter()
                    .map(|finding| finding.location.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !own.is_empty() {
            let names = own
                .iter()
                .filter_map(|finding| finding.paths.first()?.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            let databases = names.iter().filter(|name| name.contains(".sqlite")).count();
            let mut shown = names
                .into_iter()
                .filter(|name| !name.contains(".sqlite"))
                .collect::<Vec<_>>();
            if databases > 0 {
                shown.push(format!(
                    "{databases} database{}",
                    if databases == 1 { "" } else { "s" }
                ));
            }
            lines.push(format!(
                "Corbanu history and state denied to agent commands after restart: {}",
                shown.join(", ")
            ));
        }
        let routes = preflight
            .inventory
            .findings
            .iter()
            .filter(|finding| finding.disposition == Disposition::NotContained)
            .count();
        if routes > 0 {
            lines.push(format!(
                "Not contained: {routes} MCP server, hook or notify route{} run outside the sandbox.",
                if routes == 1 { "" } else { "s" }
            ));
        }
        lines.push(
            "Conversations recorded before the restart cannot be resumed under Aggressive."
                .to_string(),
        );
        lines.push(format!("Limits: {}", LIMITS.join(" ")));
        lines
    }

    fn choose(&self, row: Row) -> Screen {
        match row.chosen() {
            None => Screen::List {
                note: Some("Moderate is not available yet.".to_string()),
            },
            // A saved Aggressive opens its review again so the nested-agent
            // setting can be changed.
            Some(ChosenLevel::Permissive)
                if self.stored == StoredLevel::Chosen(ChosenLevel::Permissive) =>
            {
                Screen::List {
                    note: Some("Permissive is already saved. Nothing changed.".to_string()),
                }
            }
            Some(ChosenLevel::Permissive) if self.stored == StoredLevel::Absent => Screen::List {
                note: Some("Permissive is already in effect. Nothing changed.".to_string()),
            },
            Some(target) => Screen::Review(target),
        }
    }

    fn status_lines(&self) -> Vec<String> {
        let saved = self.stored.enforced();
        let mut lines = vec![format!("Active in this session: {}", self.active.name())];
        match &self.stored {
            StoredLevel::Invalid(reason) => lines.push(format!(
                "Stored level is unreadable ({reason}); Aggressive is enforced. Choose a level to repair it."
            )),
            StoredLevel::Absent | StoredLevel::Chosen(_) if saved != self.active => lines.push(
                format!(
                    "Saved: {}. It takes effect when you restart Corbanu Terminal.",
                    saved.name()
                ),
            ),
            StoredLevel::Absent | StoredLevel::Chosen(_) => {}
        }
        lines
    }

    pub(crate) fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let width = usize::from(width.max(1));
        let wrap = |text: &str| -> Vec<Line<'static>> {
            textwrap::wrap(text, width)
                .into_iter()
                .map(|line| Line::from(line.into_owned()))
                .collect()
        };
        let mut lines: Vec<Line<'static>> = Vec::new();
        match &self.screen {
            Screen::List { note } => {
                lines.push("Security level".bold().into());
                for status in self.status_lines() {
                    lines.extend(wrap(&status));
                }
                lines.push(Line::default());
                for (index, row) in ROWS.iter().enumerate() {
                    let marker = if index == self.selected { ">" } else { " " };
                    let suffix = match row.chosen() {
                        None => "  not available yet",
                        Some(level) if level == self.active => "  (active)",
                        Some(_) => "",
                    };
                    let text = format!("{marker} {}{suffix}", row.name());
                    lines.push(match (index == self.selected, row.chosen()) {
                        (_, None) => text.dim().into(),
                        (true, Some(_)) => text.cyan().bold().into(),
                        (false, Some(_)) => text.into(),
                    });
                }
                lines.push(Line::default());
                let summary = match ROWS[self.selected] {
                    Row::Permissive => {
                        "Today's behaviour. Your approval, sandbox, network, vault and agent settings apply exactly as configured."
                    }
                    Row::Moderate => {
                        "Protection around untrusted content and secrets while normal work continues. Not available yet."
                    }
                    Row::Aggressive => {
                        "Sensitive access denied by default, built from existing sandbox, approval, network and exec-policy controls."
                    }
                };
                lines.extend(wrap(summary));
                if let Some(note) = note {
                    lines.extend(wrap(note).into_iter().map(Stylize::cyan));
                }
            }
            Screen::Review(ChosenLevel::Aggressive) => {
                lines.push("Switch to Aggressive?".bold().into());
                lines.extend(wrap(
                    "These settings replace yours in Corbanu Terminal sessions and their child agents:",
                ));
                lines.extend(
                    wrap("\"now\" shows this session; web search, environment, shell and roles are as loaded at launch.")
                        .into_iter()
                        .map(Stylize::dim),
                );
                // Indents only when there is room for text after them.
                let (indent, hanging) = if width < 20 { ("", "") } else { ("  ", "    ") };
                for ((control, value), current) in aggressive::ROWS.iter().zip(&self.current) {
                    let now: Line<'static> = vec![
                        format!("• {control}").bold(),
                        format!(" now: {current}").into(),
                    ]
                    .into();
                    lines.extend(word_wrap_lines(
                        [now],
                        RtOptions::new(width).subsequent_indent(hanging.into()),
                    ));
                    lines.extend(word_wrap_lines(
                        [Line::from(format!("Aggressive: {value}"))],
                        RtOptions::new(width)
                            .initial_indent(indent.into())
                            .subsequent_indent(hanging.into()),
                    ));
                }
                lines.extend(word_wrap_lines(
                    [Line::from(vec![
                        "• ".bold(),
                        aggressive::nested_row(self.nested_choice).into(),
                    ])],
                    RtOptions::new(width).subsequent_indent(hanging.into()),
                ));
                lines.extend(
                    wrap("Press n to switch between refuse and pass. Nested launches read this when they start.")
                        .into_iter()
                        .map(Stylize::dim),
                );
                if let Some(preflight) = &self.preflight {
                    for line in self.preflight_lines(preflight) {
                        lines.extend(wrap(&line));
                    }
                }
                if let Some(note) = &self.preflight_note {
                    lines.extend(wrap(note).into_iter().map(Stylize::cyan));
                }
                lines.extend(wrap(aggressive::UNCHANGED));
                lines.extend(wrap(if self.active == ChosenLevel::Aggressive {
                    "This session is already Aggressive; saving keeps it after restart. Your config.toml is not modified."
                } else {
                    "Takes effect when you restart Corbanu Terminal; this session keeps its current settings until then. Your config.toml is not modified."
                }));
            }
            Screen::Review(ChosenLevel::Permissive) => {
                lines.push("Return to Permissive?".bold().into());
                lines.extend(wrap(
                    "Your own approval, sandbox, network, web search and environment settings from config apply again exactly as saved. The Aggressive vault rule file is removed.",
                ));
                lines.extend(wrap(if self.active == ChosenLevel::Aggressive {
                    "Takes effect when you restart Corbanu Terminal; this session stays Aggressive until then, and its pending approvals end with it."
                } else {
                    "This session is already Permissive; saving cancels the Aggressive level saved for the next start."
                }));
            }
            Screen::Saved(Ok(level)) => {
                let saved = match level {
                    ChosenLevel::Aggressive => {
                        format!("Saved: Aggressive (nested agents: {})", self.nested.name())
                    }
                    ChosenLevel::Permissive => format!("Saved: {}", level.name()),
                };
                lines.push(saved.bold().into());
                let message = if *level == self.active {
                    format!("{} is active in this session.", level.name())
                } else {
                    format!(
                        "Restart Corbanu Terminal to activate {}. Until then this session stays {}. Resume this conversation with /resume after restarting.",
                        level.name(),
                        self.active.name()
                    )
                };
                lines.extend(wrap(&message));
            }
            Screen::Saved(Err(error)) => {
                lines.push("Security level not saved".bold().red().into());
                lines.extend(wrap(&format!(
                    "{error}. The stored level is now: {}.",
                    self.stored.enforced().name()
                )));
            }
            Screen::MigrationPreview => {
                if let Some(plan) = &self.migration {
                    let mut text = super::security_migration::preview_lines(plan).into_iter();
                    if let Some(title) = text.next() {
                        lines.push(title.bold().into());
                    }
                    for line in text {
                        lines.extend(wrap(&line));
                    }
                }
            }
            Screen::MigrationResult(result) => {
                let mut text = super::security_migration::result_lines(result).into_iter();
                if let Some(title) = text.next() {
                    lines.push(match result {
                        Ok(_) => title.bold().into(),
                        Err(_) => title.bold().red().into(),
                    });
                }
                for line in text {
                    lines.extend(wrap(&line));
                }
            }
        }
        lines
    }

    /// Rows to skip when `body_height` rows are visible. Records the limit
    /// so the review can be scrolled until its last line is shown.
    pub(crate) fn scroll_for(&self, line_count: usize, body_height: u16) -> u16 {
        let max = u16::try_from(line_count)
            .unwrap_or(u16::MAX)
            .saturating_sub(body_height);
        let max = if matches!(self.screen, Screen::Review(_) | Screen::MigrationPreview) {
            max
        } else {
            0
        };
        self.max_scroll.set(max);
        self.scroll.min(max)
    }

    pub(crate) fn footer(&self) -> String {
        let label = |bindings: &[key_hint::KeyBinding]| {
            bindings
                .first()
                .map(key_hint::KeyBinding::display_label)
                .unwrap_or_else(|| "unbound".into())
        };
        let accept = label(&self.keymap.accept);
        match self.screen {
            Screen::List { .. } => format!(
                "{}/{} move · {accept} choose · esc close",
                label(&self.keymap.move_up),
                label(&self.keymap.move_down),
            ),
            Screen::Review(target) => {
                let scroll = if self.max_scroll.get() > 0 {
                    format!(
                        "{}/{} scroll · ",
                        label(&self.keymap.move_up),
                        label(&self.keymap.move_down),
                    )
                } else {
                    String::new()
                };
                let nested = if target == ChosenLevel::Aggressive {
                    "n refuse/pass · "
                } else {
                    ""
                };
                let migrate = if self.migration_unfinished() {
                    "r finish the migration · ".to_string()
                } else if self.migration_available() {
                    "m move credentials to the vault · ".to_string()
                } else {
                    String::new()
                };
                if target == ChosenLevel::Aggressive
                    && self
                        .preflight
                        .as_ref()
                        .is_some_and(|preflight| !preflight.is_clean())
                {
                    format!("{scroll}{migrate}esc back, nothing changes")
                } else {
                    format!("{scroll}{nested}{accept} confirm and save · esc back, nothing changes")
                }
            }
            Screen::Saved(_) => format!("{accept} or esc close"),
            Screen::MigrationPreview => {
                let count = self.migration.as_ref().map_or(0, |plan| plan.moves.len());
                format!(
                    "{}/{} scroll · {accept} move {count} credential{} · esc back, nothing changes",
                    label(&self.keymap.move_up),
                    label(&self.keymap.move_down),
                    if count == 1 { "" } else { "s" }
                )
            }
            Screen::MigrationResult(_) => format!("{accept} or esc back to the review"),
        }
    }
}

#[cfg(test)]
#[path = "security_level_picker_tests.rs"]
mod tests;
