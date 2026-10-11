//! PF-84-S04: one onboarding row per configured provider. Choosing it offers
//! "Use configured credentials", "Add another account" (named accounts on)
//! and the provider's setup methods as "Replace with ...".

use codex_vault::ProviderAccountName;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;

use super::AuthModeWidget;
use super::SignInOption;
use super::SignInState;
use super::keys;
use super::masked_api_key;
use crate::key_hint::KeyBindingListExt;
use crate::provider_named_accounts::AccountValue;
use crate::provider_named_accounts::AddAccountMethod;

/// One choice behind a configured provider's row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConfiguredChoice {
    UseExisting,
    AddAccount,
    Replace(SignInOption),
}

impl AuthModeWidget {
    /// The setup rows a catalog entry would show while not configured.
    pub(super) fn entry_setup_options(&self, entry_index: usize) -> Vec<SignInOption> {
        self.provider_status_host
            .catalog()
            .entries()
            .get(entry_index)
            .map(|entry| super::entry_setup_options(entry, &self.api_key_provider_options))
            .unwrap_or_default()
    }

    /// The provider id and method a new named account of this entry uses.
    fn entry_account_target(&self, entry_index: usize) -> Option<(String, AddAccountMethod)> {
        let entry = self
            .provider_status_host
            .catalog()
            .entries()
            .get(entry_index)?;
        entry.runtime_provider_ids.iter().find_map(|id| {
            self.named_account_methods
                .get(id.as_str())
                .and_then(|methods| methods.first())
                .map(|method| (id.to_string(), *method))
        })
    }

    pub(crate) fn configured_choices(&self, entry_index: usize) -> Vec<ConfiguredChoice> {
        let mut choices = vec![ConfiguredChoice::UseExisting];
        if self.entry_account_target(entry_index).is_some() {
            choices.push(ConfiguredChoice::AddAccount);
        }
        choices.extend(
            self.entry_setup_options(entry_index)
                .into_iter()
                .map(ConfiguredChoice::Replace),
        );
        choices
    }

    /// Opens the choice for a configured provider that also has setup rows.
    /// Returns false when there is nothing to choose (select it directly).
    pub(super) fn open_configured_choice(&mut self, entry_index: usize) -> bool {
        if self.configured_choices(entry_index).len() < 2 {
            return false;
        }
        self.set_error(/*message*/ None);
        *self.sign_in_state.write().unwrap() = SignInState::ConfiguredProviderChoice {
            entry_index,
            highlighted: 0,
        };
        self.request_frame.schedule_frame();
        true
    }

    pub(super) fn handle_named_account_key_event(&mut self, key_event: &KeyEvent) -> bool {
        let state = self.sign_in_state.read().unwrap().clone();
        match state {
            SignInState::ConfiguredProviderChoice {
                entry_index,
                highlighted,
            } => {
                let choices = self.configured_choices(entry_index);
                let count = choices.len().max(1);
                if keys::MOVE_UP.is_pressed(*key_event) || keys::MOVE_DOWN.is_pressed(*key_event) {
                    let delta = if keys::MOVE_UP.is_pressed(*key_event) {
                        count - 1
                    } else {
                        1
                    };
                    *self.sign_in_state.write().unwrap() = SignInState::ConfiguredProviderChoice {
                        entry_index,
                        highlighted: (highlighted + delta) % count,
                    };
                } else if keys::CONFIRM.is_pressed(*key_event) {
                    match choices.get(highlighted).copied() {
                        Some(ConfiguredChoice::UseExisting) => {
                            *self.sign_in_state.write().unwrap() = SignInState::PickMode;
                            self.select_existing_provider(entry_index);
                        }
                        Some(ConfiguredChoice::AddAccount) => {
                            *self.sign_in_state.write().unwrap() = SignInState::AccountNameEntry {
                                entry_index,
                                value: String::new(),
                            };
                        }
                        Some(ConfiguredChoice::Replace(option)) => {
                            *self.sign_in_state.write().unwrap() = SignInState::PickMode;
                            self.handle_sign_in_option(option);
                        }
                        None => {}
                    }
                } else if keys::CANCEL.is_pressed(*key_event) {
                    *self.sign_in_state.write().unwrap() = SignInState::PickMode;
                }
                self.request_frame.schedule_frame();
                true
            }
            SignInState::AccountNameEntry {
                entry_index,
                mut value,
            } => {
                if keys::CANCEL.is_pressed(*key_event) {
                    self.set_error(/*message*/ None);
                    *self.sign_in_state.write().unwrap() = SignInState::ConfiguredProviderChoice {
                        entry_index,
                        highlighted: 0,
                    };
                } else if keys::CONFIRM.is_pressed(*key_event) {
                    match ProviderAccountName::parse(&value) {
                        Ok(name) => self.account_name_entered(entry_index, name),
                        Err(error) => self.set_error(Some(error.to_string())),
                    }
                } else {
                    if edit_text(&mut value, key_event) {
                        self.set_error(/*message*/ None);
                    }
                    *self.sign_in_state.write().unwrap() =
                        SignInState::AccountNameEntry { entry_index, value };
                }
                self.request_frame.schedule_frame();
                true
            }
            SignInState::AccountValueEntry {
                entry_index,
                name,
                mut value,
            } => {
                if keys::CANCEL.is_pressed(*key_event) {
                    self.set_error(/*message*/ None);
                    *self.sign_in_state.write().unwrap() = SignInState::AccountNameEntry {
                        entry_index,
                        value: name.to_string(),
                    };
                } else if keys::CONFIRM.is_pressed(*key_event) {
                    if value.trim().is_empty() {
                        self.set_error(Some("The value cannot be empty.".to_string()));
                    } else {
                        self.save_named_account(entry_index, name, AccountValue::new(value));
                    }
                } else {
                    edit_text(&mut value, key_event);
                    *self.sign_in_state.write().unwrap() = SignInState::AccountValueEntry {
                        entry_index,
                        name,
                        value,
                    };
                }
                self.request_frame.schedule_frame();
                true
            }
            SignInState::AccountSaved { .. } => {
                if keys::CONFIRM.is_pressed(*key_event) || keys::CANCEL.is_pressed(*key_event) {
                    *self.sign_in_state.write().unwrap() = SignInState::PickMode;
                    self.request_frame.schedule_frame();
                }
                true
            }
            SignInState::AccountSaving => true,
            _ => false,
        }
    }

    pub(super) fn handle_named_account_paste(&mut self, pasted: &str) -> bool {
        let mut guard = self.sign_in_state.write().unwrap();
        match &mut *guard {
            SignInState::AccountValueEntry { value, .. } => {
                value.push_str(pasted.trim());
            }
            SignInState::AccountNameEntry { value, .. } => {
                value.push_str(pasted.trim());
            }
            _ => return false,
        }
        drop(guard);
        self.request_frame.schedule_frame();
        true
    }

    fn account_name_entered(&mut self, entry_index: usize, name: ProviderAccountName) {
        let Some((_, method)) = self.entry_account_target(entry_index) else {
            self.set_error(Some(
                "This provider cannot hold named accounts.".to_string(),
            ));
            return;
        };
        if method.is_secret() {
            *self.sign_in_state.write().unwrap() = SignInState::AccountValueEntry {
                entry_index,
                name,
                value: String::new(),
            };
        } else {
            self.save_named_account(entry_index, name, AccountValue::new(String::new()));
        }
    }

    fn save_named_account(
        &mut self,
        entry_index: usize,
        name: ProviderAccountName,
        value: AccountValue,
    ) {
        let Some((provider_id, method)) = self.entry_account_target(entry_index) else {
            self.set_error(Some(
                "This provider cannot hold named accounts.".to_string(),
            ));
            return;
        };
        *self.sign_in_state.write().unwrap() = SignInState::AccountSaving;
        let codex_home = self.codex_home.clone();
        let sign_in_state = self.sign_in_state.clone();
        let error = self.error.clone();
        let request_frame = self.request_frame.clone();
        tokio::task::spawn_blocking(move || {
            let result = crate::provider_named_accounts::save_account(
                codex_home,
                &provider_id,
                &name,
                method,
                value,
            );
            let mut state = sign_in_state.write().unwrap();
            if !matches!(&*state, SignInState::AccountSaving) {
                return;
            }
            match result {
                Ok(message) => {
                    *state = SignInState::AccountSaved {
                        message: format!(
                            "{message} Use it with `--account {name}` or choose it in /providers."
                        ),
                    };
                }
                Err(message) => {
                    *error.write().unwrap() = Some(message);
                    *state = SignInState::AccountNameEntry {
                        entry_index,
                        value: name.to_string(),
                    };
                }
            }
            drop(state);
            request_frame.schedule_frame();
        });
    }

    pub(super) fn render_named_account_state(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &SignInState,
    ) {
        let display_name = |entry_index: usize| {
            self.provider_status_host
                .catalog()
                .entries()
                .get(entry_index)
                .map_or_else(String::new, |entry| entry.display_name.clone())
        };
        let mut lines: Vec<Line> = Vec::new();
        match state {
            SignInState::ConfiguredProviderChoice {
                entry_index,
                highlighted,
            } => {
                lines.push(format!("> {}", display_name(*entry_index)).bold().into());
                lines.push("Configured · active · ready".dim().into());
                lines.push("".into());
                for (index, choice) in self.configured_choices(*entry_index).iter().enumerate() {
                    let label = match choice {
                        ConfiguredChoice::UseExisting => "Use configured credentials".to_string(),
                        ConfiguredChoice::AddAccount => "Add another account".to_string(),
                        ConfiguredChoice::Replace(option) => {
                            format!("Replace with {}", self.sign_in_option_method(*option))
                        }
                    };
                    lines.push(if index == *highlighted {
                        format!("> {label}").cyan().into()
                    } else {
                        format!("  {label}").into()
                    });
                }
            }
            SignInState::AccountNameEntry { entry_index, value } => {
                lines.push(
                    format!("> Add a {} account", display_name(*entry_index))
                        .bold()
                        .into(),
                );
                lines.push("".into());
                lines.push("Name it: lowercase letters, digits or '-' (for example work).".into());
                lines.push("".into());
                lines.push(if value.is_empty() {
                    "  Account name".dim().into()
                } else {
                    format!("  {value}").into()
                });
            }
            SignInState::AccountValueEntry {
                entry_index,
                name,
                value,
            } => {
                let method = self
                    .entry_account_target(*entry_index)
                    .map_or("Credential", |(_, method)| method.label());
                lines.push(
                    format!("> {} account `{name}`", display_name(*entry_index))
                        .bold()
                        .into(),
                );
                lines.push("".into());
                lines.push(
                    format!("Paste the {method}. It is saved to the vault and never shown.").into(),
                );
                lines.push("".into());
                lines.push(if value.is_empty() {
                    "  Paste it here (masked)".dim().into()
                } else {
                    format!("  {}", masked_api_key(value)).into()
                });
            }
            SignInState::AccountSaving => {
                lines.push("  Saving the account securely…".into());
            }
            SignInState::AccountSaved { message } => {
                lines.push(format!("✓ {message}").green().into());
                lines.push("".into());
            }
            _ => return,
        }
        lines.push("".into());
        lines.push(Line::from(vec![
            "  Press ".dim(),
            self.confirm_binding().into(),
            " to continue · Press ".dim(),
            self.cancel_binding().into(),
            " to go back".dim(),
        ]));
        if let Some(error) = self.error_message() {
            lines.push("".into());
            lines.push(error.red().into());
        }
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

/// Plain single-line editing; returns whether the text changed.
fn edit_text(value: &mut String, key_event: &KeyEvent) -> bool {
    match key_event.code {
        KeyCode::Backspace => value.pop().is_some(),
        KeyCode::Char(character)
            if key_event.kind == KeyEventKind::Press
                && !key_event.modifiers.intersects(
                    KeyModifiers::SUPER | KeyModifiers::CONTROL | KeyModifiers::ALT,
                ) =>
        {
            value.push(character);
            true
        }
        _ => false,
    }
}
