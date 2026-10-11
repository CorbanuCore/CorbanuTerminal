//! PF-84-S04: named-account actions from `/providers`.

use codex_vault::DEFAULT_PROVIDER_ACCOUNT;
use codex_vault::ProviderAccountName;

use super::App;
use super::AppEvent;
use super::AppServerSession;
use crate::legacy_core::config::edit::ConfigEdit;
use crate::legacy_core::config::edit::ConfigEditsBuilder;
use crate::provider_named_accounts::AccountsView;
use crate::provider_named_accounts::ProviderAccountEvent;
use crate::tui;

fn account_label(name: Option<&ProviderAccountName>) -> &str {
    name.map_or(DEFAULT_PROVIDER_ACCOUNT, ProviderAccountName::as_str)
}

impl App {
    pub(super) async fn handle_provider_account_event(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        event: ProviderAccountEvent,
    ) {
        match event {
            ProviderAccountEvent::OpenActions { provider_id, name } => {
                let Some(accounts) = self.provider_manager_accounts_view() else {
                    return;
                };
                let (Some(entry), Some(account)) = (
                    self.provider_account_catalog_entry(&provider_id),
                    accounts.find(&provider_id, name.as_str()).cloned(),
                ) else {
                    return;
                };
                self.chat_widget
                    .open_provider_account_actions(&entry, &account, &accounts);
            }
            ProviderAccountEvent::AddStart { provider_id } => {
                let methods = self.provider_account_methods(&provider_id);
                if methods.is_empty() {
                    self.chat_widget.add_error_message(format!(
                        "{provider_id} cannot hold named accounts yet."
                    ));
                    return;
                }
                let display_name = self.provider_account_display_name(&provider_id);
                self.chat_widget.open_provider_account_name_prompt(
                    &display_name,
                    provider_id,
                    /*rename_from*/ None,
                    methods,
                );
            }
            ProviderAccountEvent::OpenMethodChoice {
                provider_id,
                name,
                methods,
            } => {
                self.chat_widget
                    .open_provider_account_method_choice(provider_id, name, &methods);
            }
            ProviderAccountEvent::AddNamed {
                provider_id,
                name,
                method,
            } => {
                if self.provider_account_exists(&provider_id, &name) {
                    self.chat_widget.add_error_message(format!(
                        "{provider_id} already has an account `{name}`. Choose another name, or remove it first."
                    ));
                    return;
                }
                if method == crate::provider_named_accounts::AddAccountMethod::Command {
                    self.app_event_tx
                        .send(AppEvent::ProviderAccount(ProviderAccountEvent::Save {
                            provider_id,
                            name,
                            method,
                            value: crate::provider_named_accounts::AccountValue::new(String::new()),
                        }));
                    return;
                }
                let display_name = self.provider_account_display_name(&provider_id);
                self.chat_widget.open_provider_account_value_entry(
                    &display_name,
                    provider_id,
                    name,
                    method,
                );
            }
            ProviderAccountEvent::Save {
                provider_id,
                name,
                method,
                value,
            } => {
                if !self
                    .provider_account_methods(&provider_id)
                    .contains(&method)
                {
                    self.chat_widget.add_error_message(format!(
                        "{provider_id} does not take that kind of account."
                    ));
                    return;
                }
                let codex_home = self.config.codex_home.to_path_buf();
                self.run_provider_account_job(move || {
                    crate::provider_named_accounts::save_account(
                        codex_home,
                        &provider_id,
                        &name,
                        method,
                        value,
                    )
                    .map(|message| {
                        format!(
                            "{message} Use it from /providers, `--account {name}` or a spawn `account`."
                        )
                    })
                });
            }
            ProviderAccountEvent::UseForSession { provider_id, name } => {
                self.use_provider_account_for_session(tui, app_server, provider_id, name)
                    .await;
            }
            ProviderAccountEvent::MakeDefault { provider_id, name } => {
                match self
                    .persist_default_provider_account(&provider_id, name.as_ref())
                    .await
                {
                    Ok(()) => self.chat_widget.add_info_message(
                        format!(
                            "New sessions use {provider_id} account `{}`. This session is unchanged.",
                            account_label(name.as_ref())
                        ),
                        /*hint*/ None,
                    ),
                    Err(error) => self.chat_widget.add_error_message(error),
                }
                self.schedule_provider_manager_refresh();
            }
            ProviderAccountEvent::RenameStart { provider_id, name } => {
                let accounts = self.provider_manager_accounts_view().unwrap_or_default();
                if accounts.session_account(&provider_id) == name.as_str() {
                    self.chat_widget.add_error_message(format!(
                        "This session uses {provider_id} account `{name}`. Use another account for this session first, then rename it."
                    ));
                    return;
                }
                let display_name = self.provider_account_display_name(&provider_id);
                self.chat_widget.open_provider_account_name_prompt(
                    &display_name,
                    provider_id,
                    Some(name),
                    Vec::new(),
                );
            }
            ProviderAccountEvent::Rename {
                provider_id,
                from,
                to,
            } => {
                let accounts = self.provider_manager_accounts_view().unwrap_or_default();
                if accounts.session_account(&provider_id) == from.as_str() {
                    return;
                }
                let codex_home = self.config.codex_home.to_path_buf();
                let result = tokio::task::spawn_blocking({
                    let (provider_id, from, to) = (provider_id.clone(), from.clone(), to.clone());
                    move || {
                        crate::provider_named_accounts::rename_account(
                            codex_home,
                            &provider_id,
                            &from,
                            &to,
                        )
                    }
                })
                .await
                .unwrap_or_else(|error| Err(error.to_string()));
                // A default that named the old account follows the rename.
                let result = match result {
                    Ok(message) if accounts.default_account(&provider_id) == from.as_str() => self
                        .persist_default_provider_account(&provider_id, Some(&to))
                        .await
                        .map(|()| format!("{message} It stays the default for new sessions.")),
                    other => other,
                };
                self.provider_account_finished(result);
            }
            ProviderAccountEvent::RemoveStart { provider_id, name } => {
                let accounts = self.provider_manager_accounts_view().unwrap_or_default();
                let Some(entry) = self.provider_account_catalog_entry(&provider_id) else {
                    return;
                };
                self.chat_widget.open_provider_account_removal(
                    &entry,
                    provider_id,
                    name,
                    &accounts,
                );
            }
            ProviderAccountEvent::Remove {
                provider_id,
                name,
                replacement,
            } => {
                self.remove_provider_account(tui, app_server, provider_id, name, replacement)
                    .await;
            }
            ProviderAccountEvent::Finished { result } => self.provider_account_finished(result),
        }
    }

    fn provider_account_finished(&mut self, result: Result<String, String>) {
        match result {
            Ok(message) => self.chat_widget.add_info_message(message, /*hint*/ None),
            Err(message) => self.chat_widget.add_error_message(message),
        }
        self.schedule_provider_manager_refresh();
    }

    fn run_provider_account_job(
        &self,
        job: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) {
        let tx = self.app_event_tx.clone();
        tokio::task::spawn_blocking(move || {
            let result = job();
            tx.send(AppEvent::ProviderAccount(ProviderAccountEvent::Finished {
                result,
            }));
        });
    }

    fn provider_account_methods(
        &self,
        provider_id: &str,
    ) -> Vec<crate::provider_named_accounts::AddAccountMethod> {
        self.config
            .model_providers
            .get(provider_id)
            .map(|provider| crate::provider_named_accounts::add_methods(provider_id, provider))
            .unwrap_or_default()
    }

    fn provider_account_exists(&self, provider_id: &str, name: &ProviderAccountName) -> bool {
        self.provider_manager_accounts_view()
            .is_some_and(|accounts| accounts.find(provider_id, name.as_str()).is_some())
    }

    fn provider_account_catalog_entry(
        &self,
        provider_id: &str,
    ) -> Option<codex_provider_auth::ProviderCatalogEntry> {
        let host = self.provider_management_host.as_ref()?;
        host.status_host()
            .catalog()
            .entries()
            .iter()
            .find(|entry| {
                entry
                    .runtime_provider_ids
                    .iter()
                    .any(|id| id.as_str() == provider_id)
            })
            .cloned()
    }

    fn provider_account_display_name(&self, provider_id: &str) -> String {
        self.provider_account_catalog_entry(provider_id)
            .map_or_else(|| provider_id.to_string(), |entry| entry.display_name)
    }

    /// Writes `[provider_accounts] <provider> = <name>` (or clears it for the
    /// `default` account) and reloads the config new sessions start from.
    async fn persist_default_provider_account(
        &mut self,
        provider_id: &str,
        name: Option<&ProviderAccountName>,
    ) -> Result<(), String> {
        let segments = vec!["provider_accounts".to_string(), provider_id.to_string()];
        let edit = match name {
            Some(name) => ConfigEdit::SetPath {
                segments,
                value: name.as_str().into(),
            },
            None => ConfigEdit::ClearPath { segments },
        };
        ConfigEditsBuilder::for_config(&self.config)
            .with_edits([edit])
            .apply()
            .await
            .map_err(|error| {
                format!("Could not save the default {provider_id} account: {error}")
            })?;
        self.refresh_in_memory_config_from_disk_best_effort("changing the default account")
            .await;
        Ok(())
    }

    /// Starts a new session whose requests to `provider_id` use `name`. The
    /// running thread keeps its account: a live thread never changes account.
    /// Nothing changes when the new config cannot be built.
    async fn use_provider_account_for_session(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        provider_id: String,
        name: Option<ProviderAccountName>,
    ) -> bool {
        let selection = format!("{provider_id}:{}", account_label(name.as_ref()));
        let previous = self.harness_overrides.provider_account.replace(selection);
        if let Err(error) = self.refresh_in_memory_config_from_disk().await {
            self.harness_overrides.provider_account = previous;
            self.chat_widget.add_error_message(format!(
                "Could not use {provider_id} account `{}`: {error}",
                account_label(name.as_ref())
            ));
            return false;
        }
        self.start_fresh_session_with_summary_hint(
            tui, app_server, /*session_start_source*/ None,
            /*initial_user_message*/ None, /*new_thread_name*/ None,
        )
        .await;
        self.chat_widget.add_info_message(
            format!(
                "New session: requests to {provider_id} use account `{}`.",
                account_label(name.as_ref())
            ),
            /*hint*/ None,
        );
        true
    }

    async fn remove_provider_account(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        provider_id: String,
        name: ProviderAccountName,
        replacement: Option<Option<ProviderAccountName>>,
    ) {
        let accounts: AccountsView = self.provider_manager_accounts_view().unwrap_or_default();
        if accounts.in_use(&provider_id, name.as_str()) {
            // PF-54: an account in use is replaced before it is removed.
            let Some(replacement) = replacement else {
                self.chat_widget.add_error_message(format!(
                    "{provider_id} account `{name}` is in use; choose a replacement first."
                ));
                return;
            };
            if accounts.default_account(&provider_id) == name.as_str()
                && let Err(error) = self
                    .persist_default_provider_account(&provider_id, replacement.as_ref())
                    .await
            {
                self.chat_widget.add_error_message(error);
                return;
            }
            if accounts.session_account(&provider_id) == name.as_str()
                && !self
                    .use_provider_account_for_session(
                        tui,
                        app_server,
                        provider_id.clone(),
                        replacement,
                    )
                    .await
            {
                return;
            }
        }
        let codex_home = self.config.codex_home.to_path_buf();
        self.run_provider_account_job(move || {
            crate::provider_named_accounts::remove_account(codex_home, &provider_id, &name)
        });
    }
}
