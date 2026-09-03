use codex_provider_auth::CredentialControl;
use codex_provider_auth::ExplicitProviderSelection;
use codex_provider_auth::ProviderActivationPolicy;
use codex_provider_auth::ProviderAvailabilityState;
use codex_provider_auth::ProviderCatalog;
use codex_provider_auth::ProviderCatalogEntry;
use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderCurrentState;
use codex_provider_auth::ProviderEligibilityState;
use codex_provider_auth::ProviderMethodState;
use codex_provider_auth::ProviderSetupCapability;
use codex_provider_auth::ProviderStatusSnapshot;

use super::*;

const MANAGER_VIEW_ID: &str = "provider-manager";

impl ChatWidget {
    pub(crate) fn open_provider_manager_api_key(
        &mut self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target: codex_provider_auth::ApiKeyAuthTarget,
        display_name: String,
    ) {
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            format!("provider:{}", target.provider_id),
            format!("Add {display_name}"),
            "API key — masked".to_string(),
            "Stored through the selected provider credential backend".to_string(),
            Box::new(move |_label, secret| {
                tx.send(AppEvent::SaveProviderManagerApiKey {
                    attempt_id,
                    target: target.clone(),
                    api_key: crate::app_event::ProviderApiKeySecret::new(secret),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_provider_manager(
        &mut self,
        catalog: &ProviderCatalog,
        statuses: &[ProviderStatusSnapshot],
        focused_provider: Option<&codex_provider_auth::ProviderCatalogId>,
    ) {
        let manager_present = self.provider_manager_selected_index().is_some();
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Providers".bold()));
        header.push(Line::from(
            "Configure providers and control whether they are eligible for use.",
        ));
        let items = statuses
            .iter()
            .filter_map(|status| {
                let entry = catalog.get(status.id.as_str())?;
                let provider_id = status.id.clone();
                Some(SelectionItem {
                    name: entry.display_name.clone(),
                    description: Some(status_description(status)),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenProviderManagerActions {
                            provider_id: provider_id.clone(),
                        });
                    })],
                    dismiss_on_select: false,
                    ..Default::default()
                })
            })
            .collect();
        let params = SelectionViewParams {
            view_id: Some(MANAGER_VIEW_ID),
            header: Box::new(header),
            items,
            initial_selected_idx: focused_provider.and_then(|provider_id| {
                statuses.iter().position(|status| status.id == *provider_id)
            }),
            ..Default::default()
        };
        if manager_present {
            let _replaced = self.replace_selection_view_if_present(MANAGER_VIEW_ID, params);
        } else {
            self.show_selection_view(params);
        }
    }

    pub(crate) fn provider_manager_selected_index(&self) -> Option<usize> {
        self.selected_index_for_present_view(MANAGER_VIEW_ID)
    }

    pub(crate) fn open_provider_manager_actions(
        &mut self,
        entry: &ProviderCatalogEntry,
        status: &ProviderStatusSnapshot,
    ) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from(entry.display_name.clone().bold()));
        header.push(Line::from(status_description(status)));
        if let Some(note) = credential_control_note(status) {
            header.push(Line::from(note));
        }
        let mut items = Vec::new();
        match (status.configuration, status.eligibility) {
            (ProviderConfigurationState::Configured, ProviderEligibilityState::Active) => {
                items.push(policy_item(
                    status.id.clone(),
                    ProviderActivationPolicy::Inactive,
                    "Deactivate",
                    if status.current == ProviderCurrentState::Current {
                        "Choose an exact usable replacement first; credentials are kept."
                    } else {
                        "Exclude this provider from use; credentials are kept."
                    },
                ));
            }
            (ProviderConfigurationState::Configured, ProviderEligibilityState::Inactive) => {
                items.push(policy_item(
                    status.id.clone(),
                    ProviderActivationPolicy::Active,
                    "Reactivate",
                    "Make this configured provider eligible for use again.",
                ));
            }
            _ => {
                for capability in entry.setup_capabilities.iter() {
                    if interactive(capability) {
                        let provider_id = status.id.clone();
                        let capability = capability.clone();
                        items.push(SelectionItem {
                            name: setup_label(status, &capability),
                            description: Some(setup_description(&capability)),
                            actions: vec![Box::new(move |tx| {
                                tx.send(AppEvent::ProviderManagerBeginAuthentication {
                                    provider_id: provider_id.clone(),
                                    capability: capability.clone(),
                                });
                            })],
                            dismiss_on_select: true,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        self.show_selection_view(SelectionViewParams {
            header: Box::new(header),
            items,
            ..Default::default()
        });
    }

    pub(crate) fn open_provider_replacement(
        &mut self,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
        catalog: &ProviderCatalog,
        candidates: Vec<ExplicitProviderSelection>,
    ) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Choose replacement".bold()));
        header.push(Line::from(
            "The replacement is persisted and made current before deactivation.",
        ));
        let items = candidates
            .into_iter()
            .filter_map(|replacement| {
                let entry = catalog.get(replacement.provider_id.as_str())?;
                let name = format!("{} — {}", entry.display_name, replacement.model);
                let target_provider_id = target_provider_id.clone();
                Some(SelectionItem {
                    name,
                    description: Some(format!(
                        "Exact provider: {}",
                        replacement.runtime_provider_id.as_str()
                    )),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::ProviderManagerChooseReplacement {
                            target_provider_id: target_provider_id.clone(),
                            replacement: replacement.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                })
            })
            .collect();
        let cancelled_target = target_provider_id;
        self.show_selection_view(SelectionViewParams {
            header: Box::new(header),
            items,
            on_cancel: Some(Box::new(move |tx| {
                tx.send(AppEvent::ProviderManagerCancelReplacement {
                    target_provider_id: cancelled_target.clone(),
                });
            })),
            ..Default::default()
        });
    }
}

fn policy_item(
    provider_id: codex_provider_auth::ProviderCatalogId,
    policy: ProviderActivationPolicy,
    name: &str,
    description: &str,
) -> SelectionItem {
    SelectionItem {
        name: name.to_string(),
        description: Some(description.to_string()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::ProviderManagerRequestPolicy {
                provider_id: provider_id.clone(),
                policy,
            });
        })],
        dismiss_on_select: true,
        ..Default::default()
    }
}

fn status_description(status: &ProviderStatusSnapshot) -> String {
    let state = match status.configuration {
        ProviderConfigurationState::Configured => match status.eligibility {
            ProviderEligibilityState::Active => "Active",
            ProviderEligibilityState::Inactive => "Inactive",
            _ => "Configured",
        },
        ProviderConfigurationState::RecoveryRequired => "Recovery required",
        ProviderConfigurationState::NotConfigured => "Not configured",
        ProviderConfigurationState::Checking => "Checking",
        ProviderConfigurationState::Unavailable => "Unavailable",
    };
    let current = (status.current == ProviderCurrentState::Current).then_some(" · current");
    let unavailable =
        (status.availability != ProviderAvailabilityState::Ready).then_some(" · unavailable");
    format!(
        "{state}{}{}",
        current.unwrap_or(""),
        unavailable.unwrap_or("")
    )
}

fn credential_control_note(status: &ProviderStatusSnapshot) -> Option<&'static str> {
    status.methods.iter().find_map(|method| match method.state {
        ProviderMethodState::Configured {
            control: CredentialControl::ExternalEnvironment,
            ..
        } => Some("Environment-backed credential: deactivate here; unset it outside Corbanu."),
        ProviderMethodState::Configured {
            control: CredentialControl::ExternalProvider,
            ..
        } => Some("Externally managed credential: deactivate here; remove it at its provider."),
        ProviderMethodState::Configured {
            control: CredentialControl::ManagedByCorbanu,
            ..
        } => Some("Deactivation keeps the managed credential; this screen never deletes it."),
        _ => None,
    })
}

fn interactive(capability: &ProviderSetupCapability) -> bool {
    matches!(
        capability,
        ProviderSetupCapability::OpenAiAccount
            | ProviderSetupCapability::ApiKey { .. }
            | ProviderSetupCapability::ClaudeAccount
            | ProviderSetupCapability::CorbanuPlan
    )
}

fn setup_label(status: &ProviderStatusSnapshot, capability: &ProviderSetupCapability) -> String {
    let verb = if status.configuration == ProviderConfigurationState::RecoveryRequired {
        "Recover"
    } else {
        "Set up"
    };
    format!("{verb} with {}", setup_description(capability))
}

fn setup_description(capability: &ProviderSetupCapability) -> String {
    match capability {
        ProviderSetupCapability::OpenAiAccount => "OpenAI account".to_string(),
        ProviderSetupCapability::ApiKey { .. } => "API key".to_string(),
        ProviderSetupCapability::ClaudeAccount => "Claude account".to_string(),
        ProviderSetupCapability::CorbanuPlan => "Corbanu API".to_string(),
        ProviderSetupCapability::Local { .. } => "local provider".to_string(),
        ProviderSetupCapability::CommandAuth { .. } => "external command".to_string(),
        ProviderSetupCapability::StatusOnly { .. } => "external configuration".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn provider_manager_uses_shared_status_copy_snapshot() {
        let (mut chat, _tx, _rx, _op_rx) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        let catalog = ProviderCatalog::from_runtime_providers(&chat.config_ref().model_providers);
        let statuses = catalog
            .entries()
            .iter()
            .take(3)
            .enumerate()
            .map(|(index, entry)| ProviderStatusSnapshot {
                id: entry.id.clone(),
                methods: Vec::new(),
                configuration: if index == 2 {
                    ProviderConfigurationState::RecoveryRequired
                } else {
                    ProviderConfigurationState::Configured
                },
                eligibility: if index == 1 {
                    ProviderEligibilityState::Inactive
                } else {
                    ProviderEligibilityState::Active
                },
                current: if index == 0 {
                    ProviderCurrentState::Current
                } else {
                    ProviderCurrentState::NotCurrent
                },
                availability: ProviderAvailabilityState::Ready,
            })
            .collect::<Vec<_>>();
        chat.open_provider_manager(&catalog, &statuses, None);
        let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 80);
        insta::assert_snapshot!("provider_manager_shared_status", rendered);
    }

    #[tokio::test]
    async fn refresh_preserves_provider_selection_by_identity_and_falls_back_when_removed() {
        let (mut chat, _tx, _rx, _op_rx) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        let catalog = ProviderCatalog::from_runtime_providers(&chat.config_ref().model_providers);
        let statuses = catalog
            .entries()
            .iter()
            .take(3)
            .map(|entry| ProviderStatusSnapshot {
                id: entry.id.clone(),
                methods: Vec::new(),
                configuration: ProviderConfigurationState::Configured,
                eligibility: ProviderEligibilityState::Active,
                current: ProviderCurrentState::NotCurrent,
                availability: ProviderAvailabilityState::Ready,
            })
            .collect::<Vec<_>>();
        let focused_provider = statuses[2].id.clone();

        chat.open_provider_manager(&catalog, &statuses, None);
        chat.open_provider_manager(&catalog, &statuses, Some(&focused_provider));
        assert_eq!(chat.provider_manager_selected_index(), Some(2));

        chat.open_provider_manager(&catalog, &statuses[..2], Some(&focused_provider));
        assert_eq!(chat.provider_manager_selected_index(), Some(0));
    }

    #[tokio::test]
    async fn opening_provider_actions_keeps_manager_as_the_parent_view() {
        let (mut chat, _tx, mut rx, _op_rx) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        let catalog = ProviderCatalog::from_runtime_providers(&chat.config_ref().model_providers);
        let entry = &catalog.entries()[0];
        let statuses = [ProviderStatusSnapshot {
            id: entry.id.clone(),
            methods: Vec::new(),
            configuration: ProviderConfigurationState::Configured,
            eligibility: ProviderEligibilityState::Active,
            current: ProviderCurrentState::NotCurrent,
            availability: ProviderAvailabilityState::Ready,
        }];
        chat.open_provider_manager(&catalog, &statuses, None);

        chat.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));

        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenProviderManagerActions { provider_id }) if provider_id == entry.id
        ));
        assert_eq!(chat.provider_manager_selected_index(), Some(0));
    }

    #[test]
    fn external_credential_copy_never_claims_deletion() {
        let status = ProviderStatusSnapshot {
            id: ProviderCatalog::from_runtime_providers(
                &codex_model_provider_info::built_in_model_providers(None),
            )
            .entries()[0]
                .id
                .clone(),
            methods: vec![codex_provider_auth::ProviderMethodStatus {
                capability: ProviderSetupCapability::OpenAiAccount,
                state: ProviderMethodState::Configured {
                    source: codex_provider_auth::ProviderCredentialSource::Environment,
                    control: CredentialControl::ExternalEnvironment,
                    availability: codex_provider_auth::ConfiguredAvailability::Ready,
                },
            }],
            configuration: ProviderConfigurationState::Configured,
            eligibility: ProviderEligibilityState::Active,
            current: ProviderCurrentState::NotCurrent,
            availability: ProviderAvailabilityState::Ready,
        };
        let copy = credential_control_note(&status).unwrap();
        assert!(copy.contains("unset it outside Corbanu"));
        assert!(!copy.to_ascii_lowercase().contains("delete"));
    }
}
