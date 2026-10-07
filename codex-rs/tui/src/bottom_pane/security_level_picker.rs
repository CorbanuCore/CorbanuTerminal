//! Flagged `/security` picker (PF-24-S03). A human chooses Permissive or
//! Aggressive; the differences are shown first and only the accept key saves.
//! A saved level takes effect at the next start, where it is verified before
//! being shown as active.
//!
//! PF-24-S02: the accept key on a review is the one human-origin transition
//! event. It saves the level file and commits Core's transition off the UI
//! thread ([`crate::security::confirm`]): a stricter Core level applies to
//! running sessions now, a downgrade at the next start. A failed save keeps
//! the view open and changes nothing; a saved level that needs a restart
//! offers one.

use std::cell::Cell;
use std::path::PathBuf;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
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
use crate::legacy_core::security_level_change::LevelBasis;
use crate::legacy_core::security_level_change::LevelChangeKind;
use crate::legacy_core::security_level_change::SecurityLevel;
use crate::legacy_core::security_level_change::StoredSecurityState;
use crate::security::aggressive;
use crate::security::confirm;
use crate::security::confirm::Pending;
use crate::security::confirm::TransitionFailure;
use crate::security::confirm::TransitionOutcome;
use crate::security::confirm::TransitionRequest;
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
    /// The confirmation is being saved and committed.
    Saving(ChosenLevel),
    Saved(Result<TransitionOutcome, TransitionFailure>),
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
    /// Core's level settings from the session config (PF-24-S02).
    core: CoreLevels,
    /// Core's state as last read; the review commits only while it holds.
    basis: LevelBasis,
    pending: Option<Pending>,
    /// Run the commit on the calling thread (tests).
    pub(super) commit_inline: bool,
    /// Sends the "restart now" request.
    app_event_tx: Option<AppEventSender>,
    /// PF-25-S02: `k` asked for the grants and kill switch view.
    pub(super) open_revocations: bool,
}

/// Core's `[security]` levels from the session's config layers, and the
/// session the picker was opened in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CoreLevels {
    /// The session's thread, once it has started: its policy tree is the one
    /// a confirmation commits through.
    pub(crate) thread: Option<codex_protocol::ThreadId>,
    /// The strictest level any layer sets (without the stored state).
    pub(crate) configured: SecurityLevel,
    /// The strictest level a layer other than the user's `config.toml` sets.
    pub(crate) outside_user_config: SecurityLevel,
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
            core: CoreLevels::default(),
            basis: LevelBasis::read(
                &context.codex_home,
                SecurityLevel::Permissive,
                /*thread*/ None,
            ),
            pending: None,
            commit_inline: cfg!(test),
            app_event_tx: None,
            open_revocations: false,
        }
    }

    /// PF-25-S02: whether the grants and kill switch view is offered: Core
    /// enforces a protected level, the kill switch is on, or grants are held.
    fn revocations_useful(&self) -> bool {
        self.basis.in_force != SecurityLevel::Permissive
            || self.basis.kill_switch_active
            || !crate::legacy_core::security_grant::held_grants().is_empty()
    }

    /// Where the grants and kill switch view commits, and its restart sender.
    pub(super) fn revocation_target(
        &self,
    ) -> (
        crate::security::revocation_view::RevocationTarget,
        Option<AppEventSender>,
    ) {
        (
            crate::security::revocation_view::RevocationTarget {
                codex_home: self.codex_home.clone(),
                configured: self.core.configured,
                thread: self.core.thread,
            },
            self.app_event_tx.clone(),
        )
    }

    /// Back from the grants and kill switch view: read Core's state again.
    pub(super) fn revocations_closed(&mut self) {
        self.open_revocations = false;
        self.basis = self.read_basis();
    }

    pub(crate) fn set_core_levels(&mut self, core: CoreLevels) {
        self.core = core;
        self.basis = LevelBasis::read(&self.codex_home, core.configured, core.thread);
    }

    fn read_basis(&self) -> LevelBasis {
        LevelBasis::read(&self.codex_home, self.core.configured, self.core.thread)
    }

    pub(crate) fn set_app_event_tx(&mut self, app_event_tx: AppEventSender) {
        self.app_event_tx = Some(app_event_tx);
    }

    /// Whether Core's record still needs the saved Aggressive confirmed:
    /// it was saved without it, or a step did not finish.
    fn core_behind(&self) -> bool {
        self.preflight_input.is_some()
            && self.stored == StoredLevel::Chosen(ChosenLevel::Aggressive)
            && confirm::core_commit_needed(
                &self.basis,
                ChosenLevel::Aggressive,
                /*raise*/ true,
            )
    }

    /// Collect a finished commit. Returns whether the screen changed.
    pub(crate) fn poll(&mut self) -> bool {
        let Some(result) = self.pending.as_mut().and_then(Pending::poll) else {
            return false;
        };
        self.pending = None;
        (self.stored, self.nested) = level::load_state(&self.codex_home);
        self.basis = self.read_basis();
        self.screen = Screen::Saved(result);
        true
    }

    pub(crate) fn saving(&self) -> bool {
        matches!(self.screen, Screen::Saving(_))
    }

    pub(crate) fn handle_key_event(&mut self, key: KeyEvent) {
        let cancel =
            key_hint::plain(KeyCode::Esc).is_press(key) || self.keymap.cancel.is_pressed(key);
        let accept = self.keymap.accept.is_pressed(key);
        match self.screen.clone() {
            Screen::List { .. } => {
                if cancel {
                    self.closed = true;
                } else if key_hint::plain(KeyCode::Char('r')).is_press(key) && self.restart_useful()
                {
                    self.restart();
                } else if key_hint::plain(KeyCode::Char('k')).is_press(key)
                    && self.revocations_useful()
                {
                    self.open_revocations = true;
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
                    self.basis = self.read_basis();
                    self.preflight = match (&self.screen, &self.preflight_input) {
                        (Screen::Review(ChosenLevel::Aggressive), Some(input))
                            if self.stored != StoredLevel::Chosen(ChosenLevel::Aggressive)
                                || self.core_behind() =>
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
                    && !self.core_behind()
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
                    self.confirm(target);
                }
            }
            // Nothing can be cancelled halfway; the result comes from `poll`.
            Screen::Saving(_) => {}
            Screen::Saved(Err(failure)) => {
                // The view stays open: back to the review to try again.
                if cancel {
                    self.screen = Screen::List {
                        note: Some("Nothing changed.".to_string()),
                    };
                } else if accept {
                    self.back_to_review(&failure);
                }
            }
            Screen::Saved(Ok(_)) => {
                if key_hint::plain(KeyCode::Char('r')).is_press(key) && self.restart_useful() {
                    self.restart();
                } else if cancel || accept {
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

    /// The confirm key: one typed transition request, committed off the UI
    /// thread. The receipt goes first (without Aggressive stored it is
    /// ignored); Core's level is raised only after a passed preflight.
    fn confirm(&mut self, target: ChosenLevel) {
        let raise_core = target == ChosenLevel::Aggressive
            && self.preflight_input.is_some()
            && (self.preflight.is_some() || !self.core_behind());
        let request = TransitionRequest {
            codex_home: self.codex_home.clone(),
            target,
            nested: self.nested_choice,
            configured: self.core.configured,
            thread: self.core.thread,
            reviewed: self.basis.clone(),
            passed_preflight: match target {
                ChosenLevel::Aggressive => self.preflight.clone(),
                ChosenLevel::Permissive => None,
            },
            raise_core,
        };
        self.screen = Screen::Saving(target);
        self.pending = Some(confirm::start(request, self.commit_inline));
        self.poll();
    }

    /// After a failed save: the review again, with fresh state.
    fn back_to_review(&mut self, failure: &TransitionFailure) {
        let Some(target) = self.last_target() else {
            self.screen = Screen::List { note: None };
            return;
        };
        self.basis = self.read_basis();
        if target == ChosenLevel::Aggressive
            && let Some(input) = &self.preflight_input
            && (failure.review_again || self.preflight.is_some())
        {
            self.preflight = Some(Preflight::run(&input.sources, input.flags));
        }
        self.preflight_note = Some(if failure.review_again {
            "The state changed; this review shows it now.".to_string()
        } else {
            "Not saved before; confirm to try again.".to_string()
        });
        self.scroll = 0;
        self.screen = Screen::Review(target);
    }

    fn last_target(&self) -> Option<ChosenLevel> {
        ROWS[self.selected].chosen()
    }

    /// A restart activates what was saved, or ends this session's
    /// Core state that only a new start clears.
    fn restart_useful(&self) -> bool {
        let pending =
            self.stored.enforced() != self.active || self.basis.in_force != self.basis.next_start;
        match &self.screen {
            Screen::Saved(Ok(_)) | Screen::List { .. } => pending,
            _ => false,
        }
    }

    fn restart(&mut self) {
        if let Some(tx) = &self.app_event_tx {
            tx.send(AppEvent::RestartForSecurityLevel);
        }
        self.closed = true;
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
        let ready_labels = preflight
            .readiness
            .iter()
            .filter(|item| item.is_ready())
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        lines.push(if ready_labels.is_empty() {
            format!("Controls ready: 0 of {}", preflight.readiness.len())
        } else {
            format!(
                "Controls ready: {ready} of {} ({})",
                preflight.readiness.len(),
                ready_labels.join(", ")
            )
        });
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
                "Stored level is unreadable or was changed outside /security ({reason}); Aggressive is enforced. Choose a level to repair it."
            )),
            StoredLevel::Absent | StoredLevel::Chosen(_) if saved != self.active => lines.push(
                format!(
                    "Saved: {}. It takes effect when you restart Corbanu Terminal.",
                    saved.name()
                ),
            ),
            StoredLevel::Absent | StoredLevel::Chosen(_) => {}
        }
        if let Some(line) = core_status(&self.basis) {
            lines.push(line);
        }
        // PF-25-S01: grants held now, with their scope and expiry.
        lines.extend(crate::security::grant_view::held_lines(
            &crate::legacy_core::security_grant::held_grants(),
        ));
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
                    "The rows above take effect when you restart Corbanu Terminal (you can restart right after saving); this session keeps its current settings until then. Your config.toml is not modified."
                }));
                for line in self.core_lines_for_aggressive() {
                    lines.extend(wrap(&line));
                }
            }
            Screen::Review(ChosenLevel::Permissive) => {
                lines.push("Return to Permissive?".bold().into());
                if self.active == ChosenLevel::Aggressive {
                    lines.push("Protections removed at the next start:".bold().red().into());
                    lines.extend(
                        wrap("Your own settings from config apply instead of each of these.")
                            .into_iter()
                            .map(Stylize::dim),
                    );
                    let hanging = if width < 20 { "" } else { "    " };
                    for (control, value) in aggressive::ROWS {
                        lines.extend(word_wrap_lines(
                            [Line::from(vec![
                                format!("• {control}:").bold(),
                                format!(" {value}").into(),
                            ])],
                            RtOptions::new(width).subsequent_indent(hanging.into()),
                        ));
                    }
                }
                for line in self.core_lines_for_permissive() {
                    lines.extend(wrap(&line));
                }
                lines.extend(wrap(
                    "Your own approval, sandbox, network, web search and environment settings from config apply again exactly as saved. The Aggressive vault rule file is removed at the next start.",
                ));
                lines.extend(wrap(if self.active == ChosenLevel::Aggressive {
                    "Takes effect when you restart Corbanu Terminal; this session stays Aggressive until then, and its pending approvals end with it."
                } else {
                    "This session is already Permissive; saving cancels the Aggressive level saved for the next start."
                }));
                if let Some(note) = &self.preflight_note {
                    lines.extend(wrap(note).into_iter().map(Stylize::cyan));
                }
            }
            Screen::Saving(level) => {
                lines.push(format!("Saving {}…", level.name()).bold().into());
                lines.extend(wrap(
                    "Writing the level and confirming it with Core. Nothing is active until this finishes.",
                ));
            }
            Screen::Saved(Ok(outcome)) => {
                let level = outcome.level;
                let saved = match level {
                    ChosenLevel::Aggressive => {
                        format!("Saved: Aggressive (nested agents: {})", self.nested.name())
                    }
                    ChosenLevel::Permissive => format!("Saved: {}", level.name()),
                };
                lines.push(saved.bold().into());
                if level == self.active {
                    lines.extend(wrap(&format!(
                        "{} is active in this session.",
                        level.name()
                    )));
                } else {
                    lines.extend(wrap(&format!(
                        "Restart Corbanu Terminal to activate {}. Until then this session stays {}.",
                        level.name(),
                        self.active.name()
                    )));
                }
                if let Some(report) = &outcome.core {
                    lines.extend(wrap(&core_outcome_line(report)));
                    if let Some(reason) = &report.not_saved {
                        lines.extend(
                            wrap(&format!(
                                "Core's level applies now but could not be saved ({reason}); it holds until this process ends, and the next start reports it."
                            ))
                            .into_iter()
                            .map(Stylize::red),
                        );
                    }
                }
                if self.basis.kill_switch_active
                    && matches!(self.basis.stored, StoredSecurityState::Level(_))
                {
                    lines.extend(wrap(
                        "The kill switch stays on in this session until you restart.",
                    ));
                }
                if self.restart_useful() {
                    // Resuming earlier conversations is refused only under
                    // the protected boundary (PF-29-S01).
                    lines.extend(wrap(if level == ChosenLevel::Aggressive
                        && self.preflight_input.is_some()
                    {
                        "Press r to restart now: Corbanu Terminal closes this session and starts again in this folder with the same options. Conversations from before cannot be resumed under Aggressive."
                    } else {
                        "Press r to restart now: Corbanu Terminal closes this session and starts again in this folder with the same options. Resume this conversation with /resume afterwards."
                    }));
                }
                if let Some(non_user) = self.outside_user_config_floor(level) {
                    lines.extend(wrap(&non_user).into_iter().map(Stylize::red));
                }
            }
            Screen::Saved(Err(failure)) => {
                lines.push("Security level not saved".bold().red().into());
                lines.extend(wrap(&format!("{}.", failure.message)));
                lines.extend(wrap(&format!(
                    "Saved for the next start: {}. Active in this session: {}.",
                    self.stored.enforced().name(),
                    self.active.name()
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
                "{}/{} move · {accept} choose · {}{}esc close",
                label(&self.keymap.move_up),
                label(&self.keymap.move_down),
                if self.restart_useful() {
                    "r restart now · "
                } else {
                    ""
                },
                if self.revocations_useful() {
                    "k grants and kill switch · "
                } else {
                    ""
                },
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
            Screen::Saving(_) => "saving…".to_string(),
            Screen::Saved(Ok(_)) if self.restart_useful() => {
                format!("r restart now · {accept} or esc close")
            }
            Screen::Saved(Ok(_)) => format!("{accept} or esc close"),
            Screen::Saved(Err(_)) => format!("{accept} review again · esc back, nothing changes"),
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

impl SecurityLevelPicker {
    /// What confirming Aggressive does to Core's level (PF-23).
    fn core_lines_for_aggressive(&self) -> Vec<String> {
        if self.preflight_input.is_none() {
            return vec![format!(
                "Core's level stays {}: raising it needs the activation preflight (the protected_mode_preflight feature).",
                name(self.basis.in_force)
            )];
        }
        if !confirm::core_commit_needed(&self.basis, ChosenLevel::Aggressive, /*raise*/ true) {
            return Vec::new();
        }
        let effects = "agent commands cannot open protected paths, sensitive surfaces need a grant, external content reaches the model labelled untrusted, and memory summaries stop";
        vec![
            if self.basis.live && self.basis.in_force == SecurityLevel::Aggressive {
                "Core's level is already Aggressive in this session; confirming saves it for every start and raises the other sessions of this Corbanu Terminal that are below it.".to_string()
            } else if self.basis.live {
                format!(
                    "Core's level becomes Aggressive as soon as you confirm, in this session and the others of this Corbanu Terminal: {effects}; grants and \"for session\" approvals end."
                )
            } else {
                format!(
                    "Core's level becomes Aggressive from the next start (this session's policy does not run in this process yet): {effects}."
                )
            },
        ]
    }

    /// What confirming Permissive does to Core's level.
    fn core_lines_for_permissive(&self) -> Vec<String> {
        if !confirm::core_commit_needed(&self.basis, ChosenLevel::Permissive, /*raise*/ false) {
            return Vec::new();
        }
        let saved_above = match self.basis.stored {
            StoredSecurityState::Level(stored) if stored > SecurityLevel::Permissive => {
                Some(stored)
            }
            StoredSecurityState::Level(_)
            | StoredSecurityState::Absent
            | StoredSecurityState::Unreadable(_) => None,
        };
        let mut lines = vec![if self.basis.live
            && self.basis.in_force == SecurityLevel::Permissive
            && let Some(saved) = saved_above
        {
            format!(
                "The Core level saved for the next start becomes Permissive (it is {} now, saved by another session); this session stays Permissive. Grants and \"for session\" approvals in this session end as soon as you confirm.",
                name(saved)
            )
        } else if self.basis.in_force == SecurityLevel::Permissive {
            "Core's saved record is set to Permissive (it is repaired if unreadable); this session stays Permissive.".to_string()
        } else if self.basis.live {
            format!(
                "Core's level stays {} in this session and becomes Permissive from the next start. Grants, \"for session\" approvals and child agents' authority in this session end as soon as you confirm.",
                name(self.basis.in_force)
            )
        } else {
            "Core's level becomes Permissive from the next start.".to_string()
        }];
        if crate::legacy_core::security_level_change::downgrade_rewrites_user_config(
            &self.codex_home,
            SecurityLevel::Permissive,
        ) {
            lines.push(
                "Your config.toml sets a stricter [security] level; confirming rewrites it to \"permissive\".".to_string(),
            );
        }
        if let Some(line) = self.outside_user_config_floor(ChosenLevel::Permissive) {
            lines.push(line);
        }
        lines
    }

    /// A level a config layer other than the user's sets above `level`.
    fn outside_user_config_floor(&self, level: ChosenLevel) -> Option<String> {
        (self.core.outside_user_config > confirm::core_level(level)).then(|| {
            format!(
                "A configuration layer other than your config.toml (a project's .codex/config.toml, a profile, -c or managed configuration) sets [security] level = \"{}\"; Core's next start still enforces it there.",
                self.core.outside_user_config
            )
        })
    }
}

fn name(level: SecurityLevel) -> &'static str {
    crate::security::view::profile_name(level)
}

fn core_status(basis: &LevelBasis) -> Option<String> {
    let stored = match &basis.stored {
        StoredSecurityState::Unreadable(_) => {
            return Some(
                "Core's security state is unreadable: Aggressive and the kill switch are enforced. Choose a level to repair it.".to_string(),
            );
        }
        StoredSecurityState::Absent => None,
        StoredSecurityState::Level(level) => Some(*level),
    };
    if basis.in_force == SecurityLevel::Permissive && stored.is_none() {
        return None;
    }
    Some(if basis.in_force == basis.next_start {
        format!("Core's level: {}", name(basis.in_force))
    } else {
        format!(
            "Core's level: {} now, {} from the next start",
            name(basis.in_force),
            name(basis.next_start)
        )
    })
}

fn core_outcome_line(
    report: &crate::legacy_core::security_level_change::LevelChangeReport,
) -> String {
    match report.kind {
        LevelChangeKind::Stricter if report.live => format!(
            "Core's level is {} now in this session, and from every start.",
            name(report.in_force)
        ),
        LevelChangeKind::Stricter => {
            format!(
                "Core's level is {} from the next start.",
                name(report.next_start)
            )
        }
        LevelChangeKind::Downgrade if !report.live => {
            format!(
                "Core's level is {} from the next start.",
                name(report.next_start)
            )
        }
        LevelChangeKind::Downgrade => format!(
            "Core's level stays {} in this session and is {} from the next start; grants and \"for session\" approvals ended now.",
            name(report.in_force),
            name(report.next_start)
        ),
        LevelChangeKind::Unchanged => format!("Core's level: {}.", name(report.in_force)),
    }
}

#[cfg(test)]
#[path = "security_level_picker_tests.rs"]
mod tests;
