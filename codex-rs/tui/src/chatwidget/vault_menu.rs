//! Vault action menu and secret-copy helpers.

use super::*;

const VAULT_MENU_VIEW_ID: &str = "vault-menu";
const VAULT_CREDENTIALS_VIEW_ID: &str = "vault-credentials";
const VAULT_CREDENTIAL_ACTIONS_VIEW_ID: &str = "vault-credential-actions";

impl ChatWidget {
    pub(crate) fn add_vault_credential(
        &mut self,
        label: String,
        secret: crate::app_event::VaultSecret,
    ) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        let auth_store_mode = self.config.cli_auth_credentials_store_mode;
        let keyring_backend_kind = self.config.auth_keyring_backend_kind();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let task_label = label.clone();
            let task = tokio::task::spawn_blocking(move || {
                let secret = secret.into_inner();
                if let Some(provider_key_id) =
                    codex_login::provider_api_key_id_from_vault_label(&task_label)
                {
                    codex_login::login_with_provider_api_key(
                        &codex_home,
                        &provider_key_id,
                        &secret,
                        auth_store_mode,
                        keyring_backend_kind,
                    )
                    .map_err(|error| error.to_string())
                } else {
                    codex_vault::Vault::new(codex_home)
                        .add(codex_vault::AddCredential {
                            label: task_label,
                            credential_type: codex_vault::CredentialType::ManualSecret,
                            provider: None,
                            notes: None,
                            revocation_notes: None,
                            secret,
                        })
                        .map_err(|error| error.to_string())
                }
            });
            let result = match task.await {
                Ok(result) => result,
                Err(error) => Err(format!("Vault add task failed: {error}")),
            };
            tx.send(AppEvent::VaultCredentialAdded { label, result });
        });
    }

    pub(crate) fn on_vault_credential_added(&mut self, label: String, result: Result<(), String>) {
        match result {
            Ok(()) => self.add_info_message(
                format!("Added vault credential {label:?}."),
                /*hint*/ None,
            ),
            Err(error) => {
                self.add_error_message(format!("Failed to add vault credential {label:?}: {error}"))
            }
        }
    }

    pub(crate) fn open_vault_menu(&mut self) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        self.show_selection_view(SelectionViewParams {
            view_id: Some(VAULT_MENU_VIEW_ID),
            footer_hint: Some(standard_popup_hint_line()),
            is_searchable: true,
            search_placeholder: Some("Search vault actions".to_string()),
            items: vault_action_items(codex_home.clone(), /*credential_result*/ None),
            header: vault_header(/*credential_count*/ None),
            ..Default::default()
        });
        self.load_vault_credentials(codex_home, /*menu*/ true);
    }

    pub(crate) fn open_vault_credentials_list(&mut self) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        self.show_selection_view(SelectionViewParams {
            view_id: Some(VAULT_CREDENTIALS_VIEW_ID),
            footer_hint: Some(standard_popup_hint_line()),
            is_searchable: true,
            search_placeholder: Some("Search credentials".to_string()),
            items: vault_credential_items(/*credential_result*/ None),
            header: vault_credentials_header(/*credential_count*/ None),
            ..Default::default()
        });
        self.load_vault_credentials(codex_home, /*menu*/ false);
    }

    pub(crate) fn open_vault_credential_actions(&mut self, label: String) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        let display_name = credential_display_name_for_label(&label);
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Vault credential".bold()));
        header.push(Line::from(display_name.cyan()));
        header.push(Line::from(
            "Choose an action. Secrets are never printed to chat.".dim(),
        ));

        self.show_selection_view(SelectionViewParams {
            view_id: Some(VAULT_CREDENTIAL_ACTIONS_VIEW_ID),
            footer_hint: Some(standard_popup_hint_line()),
            is_searchable: false,
            items: vault_credential_action_items(codex_home, label),
            header: Box::new(header),
            ..Default::default()
        });
    }

    pub(crate) fn copy_vault_secret_to_clipboard(&mut self, label: String) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let task = tokio::task::spawn_blocking({
                let label = label.clone();
                move || {
                    let vault = codex_vault::Vault::new(codex_home);
                    let secret = vault.reveal(&label).map_err(|err| {
                        format!("Failed to read vault credential {label:?}: {err}")
                    })?;
                    crate::clipboard_copy::copy_to_clipboard(&secret)
                }
            });
            let result = match task.await {
                Ok(result) => result,
                Err(err) => Err(format!("Vault copy task failed: {err}")),
            };
            tx.send(AppEvent::VaultCopySecretFinished { label, result });
        });
    }

    pub(crate) fn reveal_vault_secret(&mut self, label: String) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let task = tokio::task::spawn_blocking({
                let label = label.clone();
                move || {
                    codex_vault::Vault::new(codex_home)
                        .reveal(&label)
                        .map(crate::app_event::VaultSecret::new)
                        .map_err(|err| {
                            format!("Failed to reveal vault credential {label:?}: {err}")
                        })
                }
            });
            let result = match task.await {
                Ok(result) => result,
                Err(err) => Err(format!("Vault reveal task failed: {err}")),
            };
            tx.send(AppEvent::VaultRevealSecretFinished { label, result });
        });
    }

    pub(crate) fn on_vault_reveal_secret_finished(
        &mut self,
        label: String,
        result: Result<crate::app_event::VaultSecret, String>,
    ) {
        match result {
            Ok(secret) => self.bottom_pane.show_view(Box::new(
                crate::bottom_pane::vault_secret_reveal::VaultSecretRevealView::new(
                    label,
                    secret.into_inner(),
                ),
            )),
            Err(error) => self.add_error_message(error),
        }
    }

    pub(crate) fn open_vault_replace_secret(&mut self, label: String) {
        if label == codex_vault::MANAGED_CLAUDE_TOKEN_LABEL {
            self.add_error_message(
                "Replace the managed Claude subscription token from Providers so validation and selection stay transactional."
                    .to_string(),
            );
            return;
        }
        let tx = self.app_event_tx.clone();
        let submitted_label = label.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            label.clone(),
            "Replace vault credential".to_string(),
            format!("Credential {label:?} — masked replacement"),
            "New secret value (masked — never stored in chat)".to_string(),
            Box::new(move |_, secret| {
                tx.send(AppEvent::VaultCredentialReplaceRequested {
                    label: submitted_label.clone(),
                    secret: crate::app_event::VaultSecret::new(secret),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn replace_vault_credential(
        &mut self,
        label: String,
        secret: crate::app_event::VaultSecret,
    ) {
        if label == codex_vault::MANAGED_CLAUDE_TOKEN_LABEL {
            drop(secret);
            self.on_vault_credential_replaced(
                label,
                Err(
                    "managed Claude tokens can only be replaced from Providers with dedicated validation"
                        .to_string(),
                ),
            );
            return;
        }
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        let auth_store_mode = self.config.cli_auth_credentials_store_mode;
        let keyring_backend_kind = self.config.auth_keyring_backend_kind();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let task_label = label.clone();
            let task = tokio::task::spawn_blocking(move || {
                let secret = secret.into_inner();
                if let Some(provider_key_id) =
                    codex_login::provider_api_key_id_from_vault_label(&task_label)
                {
                    codex_login::login_with_provider_api_key(
                        &codex_home,
                        &provider_key_id,
                        &secret,
                        auth_store_mode,
                        keyring_backend_kind,
                    )
                    .map_err(|error| error.to_string())
                } else {
                    codex_vault::Vault::new(codex_home)
                        .update(
                            &task_label,
                            Some(secret),
                            /*provider*/ None,
                            /*notes*/ None,
                            /*revocation_notes*/ None,
                        )
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                }
            });
            let result = match task.await {
                Ok(result) => result,
                Err(error) => Err(format!("Vault replacement task failed: {error}")),
            };
            tx.send(AppEvent::VaultCredentialReplaced { label, result });
        });
    }

    pub(crate) fn on_vault_credential_replaced(
        &mut self,
        label: String,
        result: Result<(), String>,
    ) {
        match result {
            Ok(()) => self.add_info_message(
                format!("Replaced vault credential {label:?}."),
                /*hint*/ None,
            ),
            Err(error) => self.add_error_message(format!(
                "Failed to replace vault credential {label:?}: {error}"
            )),
        }
    }

    pub(crate) fn confirm_vault_credential_delete(&mut self, label: String) {
        let delete_label = label.clone();
        self.show_selection_view(SelectionViewParams {
            title: Some(format!("Delete vault credential {label:?}?")),
            subtitle: Some(
                "This permanently removes the encrypted local credential and cannot be undone."
                    .to_string(),
            ),
            footer_hint: Some(standard_popup_hint_line()),
            items: vec![
                SelectionItem {
                    name: "Cancel".to_string(),
                    description: Some("Keep the credential".to_string()),
                    dismiss_on_select: true,
                    ..Default::default()
                },
                SelectionItem {
                    name: "Delete credential".to_string(),
                    description: Some("Permanently remove it from this vault".to_string()),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::VaultCredentialDeleteRequested {
                            label: delete_label.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                },
            ],
            initial_selected_idx: Some(0),
            allow_number_shortcuts: false,
            ..Default::default()
        });
    }

    pub(crate) fn delete_vault_credential(&mut self, label: String) {
        let codex_home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let task = tokio::task::spawn_blocking({
                let label = label.clone();
                move || {
                    if label == codex_vault::MANAGED_CLAUDE_TOKEN_LABEL {
                        codex_vault::Vault::new(codex_home)
                            .remove_managed_claude_subscription_token()
                            .map_err(|error| error.to_string())
                    } else if let Some(provider_key_id) =
                        codex_login::provider_api_key_id_from_vault_label(&label)
                    {
                        codex_login::delete_provider_api_key(&codex_home, &provider_key_id)
                            .map_err(|error| error.to_string())
                    } else {
                        codex_vault::Vault::new(codex_home)
                            .delete(&label)
                            .map_err(|error| error.to_string())
                    }
                }
            });
            // A running `spawn_blocking` task cannot be cancelled by dropping a timeout future.
            // Await its real terminal result so the UI never reports failure while deletion keeps
            // mutating the vault in the background.
            let result = match task.await {
                Ok(result) => result,
                Err(error) => Err(format!("Vault delete task failed: {error}")),
            };
            tx.send(AppEvent::VaultCredentialDeleted { label, result });
        });
    }

    pub(crate) fn on_vault_credential_deleted(
        &mut self,
        label: String,
        result: Result<bool, String>,
    ) {
        match result {
            Ok(true) => self.add_info_message(
                format!("Deleted vault credential {label:?}."),
                /*hint*/ None,
            ),
            Ok(false) => self.add_error_message(format!("No vault credential labeled {label:?}.")),
            Err(error) => self.add_error_message(format!(
                "Failed to delete vault credential {label:?}: {error}"
            )),
        }
        self.open_vault_menu();
    }

    pub(crate) fn on_vault_copy_secret_finished(
        &mut self,
        label: String,
        result: Result<Option<crate::clipboard_copy::ClipboardLease>, String>,
    ) {
        match result {
            Ok(lease) => {
                self.clipboard_lease = lease;
                self.add_info_message(
                    format!("Copied vault credential {label:?} to clipboard."),
                    /*hint*/ None,
                );
            }
            Err(err) => self.add_error_message(err),
        }
    }

    fn load_vault_credentials(&self, codex_home: PathBuf, menu: bool) {
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let task = tokio::task::spawn_blocking(move || {
                sorted_vault_credentials(&codex_home).map_err(|err| err.to_string())
            });
            let result = match task.await {
                Ok(result) => result,
                Err(err) => Err(format!("Vault list task failed: {err}")),
            };
            if menu {
                tx.send(AppEvent::VaultMenuCredentialsReady { result });
            } else {
                tx.send(AppEvent::VaultCredentialsReady { result });
            }
        });
    }

    pub(crate) fn on_vault_menu_credentials_ready(
        &mut self,
        result: Result<Vec<codex_vault::VaultCredentialMeta>, String>,
    ) {
        let selected = self
            .bottom_pane
            .selected_index_for_active_view(VAULT_MENU_VIEW_ID);
        let count = result.as_ref().ok().map(Vec::len);
        let mut params = SelectionViewParams {
            view_id: Some(VAULT_MENU_VIEW_ID),
            footer_hint: Some(standard_popup_hint_line()),
            is_searchable: true,
            search_placeholder: Some("Search vault actions".to_string()),
            items: vault_action_items(self.config.codex_home.as_path().to_path_buf(), Some(result)),
            header: vault_header(count),
            ..Default::default()
        };
        params.initial_selected_idx = selected;
        self.bottom_pane
            .replace_selection_view_if_present(VAULT_MENU_VIEW_ID, params);
    }

    pub(crate) fn on_vault_credentials_ready(
        &mut self,
        result: Result<Vec<codex_vault::VaultCredentialMeta>, String>,
    ) {
        let selected = self
            .bottom_pane
            .selected_index_for_active_view(VAULT_CREDENTIALS_VIEW_ID);
        let count = result.as_ref().ok().map(Vec::len);
        let mut params = SelectionViewParams {
            view_id: Some(VAULT_CREDENTIALS_VIEW_ID),
            footer_hint: Some(standard_popup_hint_line()),
            is_searchable: true,
            search_placeholder: Some("Search credentials".to_string()),
            items: vault_credential_items(Some(result)),
            header: vault_credentials_header(count),
            ..Default::default()
        };
        params.initial_selected_idx = selected;
        self.bottom_pane
            .replace_selection_view_if_present(VAULT_CREDENTIALS_VIEW_ID, params);
    }
}

fn sorted_vault_credentials(
    codex_home: &Path,
) -> Result<Vec<codex_vault::VaultCredentialMeta>, codex_vault::VaultError> {
    let mut credentials = codex_vault::Vault::new(codex_home.to_path_buf()).list()?;
    credentials.sort_by(|left, right| left.label.cmp(&right.label));
    Ok(credentials)
}

fn vault_header(credential_count: Option<usize>) -> Box<dyn Renderable> {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Vault".bold()));
    header.push(Line::from(
        "Add credentials, inspect metadata, or copy secrets without sending them to chat.".dim(),
    ));
    if let Some(count) = credential_count {
        header.push(Line::from(format!("{count} credential(s) stored").dim()));
    }
    Box::new(header)
}

fn vault_credentials_header(credential_count: Option<usize>) -> Box<dyn Renderable> {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("View credentials".bold()));
    header.push(Line::from("Select a credential to inspect or copy.".dim()));
    if let Some(count) = credential_count {
        header.push(Line::from(format!("{count} credential(s) stored").dim()));
    }
    Box::new(header)
}

fn vault_action_items(
    codex_home: PathBuf,
    credential_result: Option<Result<Vec<codex_vault::VaultCredentialMeta>, String>>,
) -> Vec<SelectionItem> {
    let view_description = match credential_result {
        None => "Loading stored credentials...".to_string(),
        Some(Ok(credentials)) if credentials.is_empty() => "No credentials stored yet".to_string(),
        Some(Ok(credentials)) => format!("View {} stored credential(s)", credentials.len()),
        Some(Err(err)) => format!("Credential list unavailable: {err}"),
    };
    vec![
        SelectionItem {
            name: "Add credential".to_string(),
            description: Some("Open masked label and secret entry".to_string()),
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::OpenVaultCredentialAdd);
            })],
            dismiss_on_select: true,
            ..Default::default()
        },
        SelectionItem {
            name: "View credentials".to_string(),
            description: Some(view_description),
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::OpenVaultCredentialsList);
            })],
            dismiss_on_select: false,
            ..Default::default()
        },
        vault_history_item(
            "Vault status",
            "Show lock state, backend, and credential count",
            codex_home,
            "status".to_string(),
        ),
    ]
}

fn vault_credential_items(
    credential_result: Option<Result<Vec<codex_vault::VaultCredentialMeta>, String>>,
) -> Vec<SelectionItem> {
    match credential_result {
        None => vec![SelectionItem {
            name: "Loading credentials...".to_string(),
            is_disabled: true,
            dismiss_on_select: false,
            ..Default::default()
        }],
        Some(Ok(credentials)) if credentials.is_empty() => vec![SelectionItem {
            name: "No credentials stored".to_string(),
            description: Some("Use Add credential from the vault menu.".to_string()),
            is_disabled: true,
            dismiss_on_select: false,
            ..Default::default()
        }],
        Some(Ok(credentials)) => credentials.into_iter().map(vault_credential_item).collect(),
        Some(Err(err)) => vec![SelectionItem {
            name: "Credential list unavailable".to_string(),
            description: Some(err),
            is_disabled: true,
            dismiss_on_select: false,
            ..Default::default()
        }],
    }
}

fn vault_credential_item(credential: codex_vault::VaultCredentialMeta) -> SelectionItem {
    let label = credential.label;
    let name = credential_display_name(&label, credential.provider.as_deref());
    let description = match credential.provider {
        Some(provider) => format!("Stored as {provider}; vault label {label}"),
        None => credential.credential_type.description().to_string(),
    };
    SelectionItem {
        name,
        description: Some(description),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::OpenVaultCredentialActions {
                label: label.clone(),
            });
        })],
        dismiss_on_select: false,
        ..Default::default()
    }
}

fn credential_display_name_for_label(label: &str) -> String {
    credential_display_name(label, /*provider*/ None)
}

fn credential_display_name(label: &str, provider: Option<&str>) -> String {
    let key_id = provider
        .or_else(|| label.strip_prefix("provider/"))
        .unwrap_or(label);
    match key_id.to_ascii_uppercase().as_str() {
        "AMBIENT_API_KEY" => "Provider: Ambient API Key".to_string(),
        "KIMI_API_KEY" => "Provider: Kimi Code API Key".to_string(),
        "ZAI_API_KEY" => "Provider: Z.AI API Key".to_string(),
        "DEEPSEEK_API_KEY" => "Provider: DeepSeek API Key".to_string(),
        "OPENROUTER_API_KEY" => "Provider: OpenRouter API Key".to_string(),
        "MODEL_API_KEY" => "Provider: Meta API Key".to_string(),
        "BASETEN_API_KEY" => "Provider: Baseten API Key".to_string(),
        "AI_GATEWAY_API_KEY" => "Provider: Vercel API Key".to_string(),
        _ if label.starts_with("provider/") => format!("Provider: {key_id}"),
        _ => label.to_string(),
    }
}

fn vault_credential_action_items(codex_home: PathBuf, label: String) -> Vec<SelectionItem> {
    let reveal_label = label.clone();
    let copy_label = label.clone();
    let delete_label = label.clone();
    let mut items = vec![vault_history_item(
        "Show metadata",
        "Inspect metadata only; secret remains hidden",
        codex_home,
        format!("show {label}"),
    )];
    if label != codex_vault::MANAGED_CLAUDE_TOKEN_LABEL {
        items.push(SelectionItem {
            name: "Reveal secret".to_string(),
            description: Some("Show raw secret only in a transient secure view".to_string()),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenVaultRevealSecret {
                    label: reveal_label.clone(),
                });
            })],
            dismiss_on_select: true,
            ..Default::default()
        });
        items.push(SelectionItem {
            name: "Copy secret".to_string(),
            description: Some(
                "Copy raw secret to clipboard; it is not printed to chat".to_string(),
            ),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenVaultCopySecret {
                    label: copy_label.clone(),
                });
            })],
            dismiss_on_select: true,
            ..Default::default()
        });
        let replace_label = label;
        items.push(SelectionItem {
            name: "Replace secret".to_string(),
            description: Some("Enter a masked replacement for this credential".to_string()),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenVaultReplaceSecret {
                    label: replace_label.clone(),
                });
            })],
            dismiss_on_select: true,
            ..Default::default()
        });
    }
    items.push(SelectionItem {
        name: "Delete credential".to_string(),
        description: Some("Open a safe confirmation before permanent removal".to_string()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::ConfirmVaultCredentialDelete {
                label: delete_label.clone(),
            });
        })],
        dismiss_on_select: true,
        ..Default::default()
    });
    items
}

fn vault_history_item(
    name: impl Into<String>,
    description: impl Into<String>,
    codex_home: PathBuf,
    args: String,
) -> SelectionItem {
    SelectionItem {
        name: name.into(),
        description: Some(description.into()),
        actions: vec![Box::new(move |tx| {
            let tx = tx.clone();
            let codex_home = codex_home.clone();
            let args = args.clone();
            tokio::spawn(async move {
                let lines = tokio::task::spawn_blocking(move || {
                    crate::vault_command::handle_vault_command(&codex_home, &args)
                })
                .await
                .unwrap_or_else(|err| vec![Line::from(format!("Vault task failed: {err}"))]);
                tx.send(AppEvent::InsertHistoryCell(Box::new(
                    PlainHistoryCell::new(lines),
                )));
            });
        })],
        dismiss_on_select: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_vault::CredentialType;
    use codex_vault::StorageBackend;
    use codex_vault::VaultCredentialMeta;

    #[test]
    fn vault_menu_view_id_is_stable() {
        assert_eq!(VAULT_MENU_VIEW_ID, "vault-menu");
    }

    #[test]
    fn top_level_vault_actions_do_not_include_per_credential_actions() {
        let items = vault_action_items(
            PathBuf::from("/tmp/codex-home"),
            Some(Ok(vec![VaultCredentialMeta {
                label: "provider/ambient_api_key".to_string(),
                credential_type: CredentialType::ApiKey,
                provider: Some("AMBIENT_API_KEY".to_string()),
                notes: None,
                revocation_notes: None,
                created_at: 1,
                updated_at: 1,
                storage_backend: StorageBackend::EncryptedSecrets,
            }])),
        );
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec!["Add credential", "View credentials", "Vault status"]
        );
        assert!(!names.iter().any(|name| name.starts_with("Copy secret")));
        assert!(!names.iter().any(|name| name.starts_with("Show provider/")));
    }

    #[test]
    fn credential_tab_shows_one_row_per_credential() {
        let items = vault_credential_items(Some(Ok(vec![VaultCredentialMeta {
            label: "provider/ambient_api_key".to_string(),
            credential_type: CredentialType::ApiKey,
            provider: Some("AMBIENT_API_KEY".to_string()),
            notes: None,
            revocation_notes: None,
            created_at: 1,
            updated_at: 1,
            storage_backend: StorageBackend::EncryptedSecrets,
        }])));

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Provider: Ambient API Key");
        assert!(
            items[0]
                .description
                .as_deref()
                .is_some_and(|description| description.contains("AMBIENT_API_KEY"))
        );
    }

    #[test]
    fn provider_credentials_render_as_human_provider_names() {
        assert_eq!(
            credential_display_name("provider/ambient_api_key", /*provider*/ None),
            "Provider: Ambient API Key"
        );
        assert_eq!(
            credential_display_name("provider/zai_api_key", /*provider*/ None),
            "Provider: Z.AI API Key"
        );
        assert_eq!(
            credential_display_name("provider/kimi_api_key", /*provider*/ None),
            "Provider: Kimi Code API Key"
        );
        assert_eq!(
            credential_display_name("provider/deepseek_api_key", /*provider*/ None),
            "Provider: DeepSeek API Key"
        );
        assert_eq!(
            credential_display_name("provider/openrouter_api_key", /*provider*/ None),
            "Provider: OpenRouter API Key"
        );
        assert_eq!(
            credential_display_name("provider/model_api_key", /*provider*/ None),
            "Provider: Meta API Key"
        );
        assert_eq!(
            credential_display_name("provider/baseten_api_key", /*provider*/ None),
            "Provider: Baseten API Key"
        );
        assert_eq!(
            credential_display_name("provider/ai_gateway_api_key", /*provider*/ None),
            "Provider: Vercel API Key"
        );
    }

    #[test]
    fn credential_actions_are_scoped_to_selected_credential() {
        let items = vault_credential_action_items(
            PathBuf::from("/tmp/codex-home"),
            "provider/zai_api_key".to_string(),
        );
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "Show metadata",
                "Reveal secret",
                "Copy secret",
                "Replace secret",
                "Delete credential"
            ]
        );
    }

    #[test]
    fn managed_claude_token_actions_are_metadata_only() {
        let items = vault_credential_action_items(
            PathBuf::from("/tmp/codex-home"),
            codex_vault::MANAGED_CLAUDE_TOKEN_LABEL.to_string(),
        );
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["Show metadata", "Delete credential"]);
    }
}
