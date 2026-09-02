use codex_provider_auth::ProviderActivationPolicy;
use codex_provider_auth::ProviderManagementAction;
use codex_provider_auth::ProviderManagementEffect;
use codex_provider_auth::ProviderManagementPersistenceResult;
use codex_provider_auth::ProviderManagementPhase;
use codex_provider_auth::ProviderManagementTransition;

use super::App;
use super::AppEvent;
use super::AppServerSession;

impl App {
    pub(super) fn open_provider_manager_actions(
        &mut self,
        provider_id: codex_provider_auth::ProviderCatalogId,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        host.remember_focused_provider(provider_id.clone());
        let Some(entry) = host
            .status_host()
            .catalog()
            .get(provider_id.as_str())
            .cloned()
        else {
            return;
        };
        let Some(status) = host
            .statuses()
            .iter()
            .find(|status| status.id == provider_id)
            .cloned()
        else {
            return;
        };
        self.chat_widget
            .open_provider_manager_actions(&entry, &status);
    }

    pub(super) fn provider_manager_request_policy(
        &mut self,
        provider_id: codex_provider_auth::ProviderCatalogId,
        policy: ProviderActivationPolicy,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let transition = host.dispatch(ProviderManagementAction::RequestPolicy {
            provider_id,
            policy,
        });
        self.apply_provider_management_transition(transition);
    }

    pub(super) fn provider_manager_begin_authentication(
        &mut self,
        provider_id: codex_provider_auth::ProviderCatalogId,
        capability: codex_provider_auth::ProviderSetupCapability,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let Some(entry) = host
            .status_host()
            .catalog()
            .get(provider_id.as_str())
            .cloned()
        else {
            return;
        };
        let transition = host.dispatch(ProviderManagementAction::BeginAuthentication {
            provider_id: provider_id.clone(),
        });
        if !transition.applied {
            return;
        }
        let Some(attempt_id) = transition.effects.iter().find_map(|effect| match effect {
            ProviderManagementEffect::BeginAuthentication { attempt_id, .. } => Some(*attempt_id),
            _ => None,
        }) else {
            return;
        };
        match capability {
            capability @ codex_provider_auth::ProviderSetupCapability::ApiKey { .. } => {
                match codex_provider_auth::ApiKeyAuthTarget::from_catalog_capability(
                    &entry,
                    &capability,
                ) {
                    Ok(target) => self.chat_widget.open_provider_manager_api_key(
                        attempt_id,
                        target,
                        entry.display_name,
                    ),
                    Err(_) => self.provider_manager_authentication_cancelled(provider_id),
                }
            }
            codex_provider_auth::ProviderSetupCapability::OpenAiAccount => {
                let Ok(target) =
                    codex_provider_auth::OpenAiAccountTarget::from_catalog_entry(&entry)
                else {
                    self.provider_manager_authentication_cancelled(provider_id);
                    return;
                };
                let Some(status) = self
                    .provider_management_host
                    .as_ref()
                    .and_then(|host| host.status_host().resolve_provider(provider_id.as_str()))
                else {
                    return;
                };
                self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
                    codex_provider_auth::OpenAiAccountAction::Start(
                        codex_provider_auth::OpenAiAccountFlowStart {
                            target,
                            method: codex_provider_auth::OpenAiAccountMethod::DeviceCode,
                            context:
                                codex_provider_auth::OpenAiAccountLoginContext::ProviderEnrollment,
                            status,
                        },
                    )
                    .into(),
                ));
            }
            codex_provider_auth::ProviderSetupCapability::ClaudeAccount => {
                let Ok(target) = codex_provider_auth::claude_account_flow::ClaudeAccountTarget::from_catalog_entry(&entry)
                else {
                    self.provider_manager_authentication_cancelled(provider_id);
                    return;
                };
                let Some(status) = self
                    .provider_management_host
                    .as_ref()
                    .and_then(|host| host.status_host().resolve_provider(provider_id.as_str()))
                else {
                    return;
                };
                if status.configuration
                    == codex_provider_auth::ProviderConfigurationState::RecoveryRequired
                {
                    let codex_home = self.config.codex_home.to_path_buf();
                    let tx = self.app_event_tx.clone();
                    tokio::spawn(async move {
                        let source =
                            crate::chatwidget::claude_code_login::selected_claude_recovery_source(
                                codex_home,
                            )
                            .await;
                        tx.send(AppEvent::ProviderManagerClaudeRecoverySourceResolved {
                            attempt_id,
                            target,
                            status,
                            source,
                        });
                    });
                } else {
                    self.provider_manager_claude_recovery_source_resolved(
                        attempt_id,
                        target,
                        status,
                        codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource::Unknown,
                    );
                }
            }
            codex_provider_auth::ProviderSetupCapability::CorbanuPlan => {
                self.app_event_tx.send(AppEvent::OpenWallet);
            }
            _ => self.provider_manager_authentication_cancelled(provider_id),
        }
    }

    pub(super) fn provider_manager_claude_recovery_source_resolved(
        &mut self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target: codex_provider_auth::claude_account_flow::ClaudeAccountTarget,
        status: codex_provider_auth::ProviderStatusSnapshot,
        source: codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource,
    ) {
        let matches = self.provider_management_host.as_ref().and_then(
            super::super::provider_management_host::ProviderManagementHost::authenticating_attempt,
        ) == Some((attempt_id, target.provider_id.clone()));
        if !matches {
            return;
        }
        let Some(intent) = claude_intent_for_status(&status, source) else {
            self.provider_manager_authentication_cancelled(target.provider_id);
            return;
        };
        self.app_event_tx.send(AppEvent::SharedProviderAuthAction(
            codex_provider_auth::claude_account_flow::ClaudeAccountAction::Start(
                codex_provider_auth::claude_account_flow::ClaudeAccountFlowStart {
                    target,
                    intent,
                    status,
                },
            )
            .into(),
        ));
    }

    pub(super) fn provider_manager_cancel_replacement(
        &mut self,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let transition =
            host.dispatch(ProviderManagementAction::CancelReplacement { target_provider_id });
        self.apply_provider_management_transition(transition);
    }

    pub(super) fn provider_manager_choose_replacement(
        &mut self,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
        replacement: codex_provider_auth::ExplicitProviderSelection,
        app_server: &AppServerSession,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let transition = host.dispatch(ProviderManagementAction::ChooseReplacement {
            target_provider_id,
            replacement,
        });
        self.apply_provider_management_transition_with_server(transition, app_server);
    }

    pub(super) fn provider_manager_persistence_finished(
        &mut self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        result: ProviderManagementPersistenceResult,
    ) {
        let Some(host) = self.provider_management_host.as_mut() else {
            return;
        };
        let transition =
            host.dispatch(ProviderManagementAction::PersistenceFinished { attempt_id, result });
        if !transition.applied {
            return;
        }
        match result {
            ProviderManagementPersistenceResult::Applied => {}
            ProviderManagementPersistenceResult::ReplacementAppliedDeactivationFailed => {
                self.chat_widget.add_error_message(
                    "Replacement is current, but the previous provider remains active.".to_string(),
                )
            }
            ProviderManagementPersistenceResult::Failed => self
                .chat_widget
                .add_error_message("Provider settings could not be persisted.".to_string()),
        }
        self.apply_provider_management_transition(transition);
    }

    pub(super) fn provider_manager_replacement_persisted(
        &mut self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
        replacement: codex_provider_auth::ExplicitProviderSelection,
        success: bool,
    ) {
        if !self.provider_manager_attempt_matches(attempt_id, &target_provider_id) {
            return;
        }
        if !success {
            self.provider_manager_persistence_finished(
                attempt_id,
                ProviderManagementPersistenceResult::Failed,
            );
            return;
        }
        if let Some(host) = self.provider_management_host.as_ref() {
            host.status_host()
                .set_current_runtime(replacement.runtime_provider_id.to_string());
        }
        self.app_event_tx.send(AppEvent::UpdateModelSelection {
            model: replacement.model,
            provider: Some(replacement.runtime_provider_id.to_string()),
        });
        self.app_event_tx
            .send(AppEvent::ProviderManagerDeactivateAfterReplacement {
                attempt_id,
                target_provider_id,
            });
    }

    pub(super) fn provider_manager_deactivate_after_replacement(
        &mut self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target_provider_id: codex_provider_auth::ProviderCatalogId,
    ) {
        if !self.provider_manager_attempt_matches(attempt_id, &target_provider_id) {
            return;
        }
        let Some(host) = self.provider_management_host.as_ref() else {
            return;
        };
        let status_host = host.status_host().clone();
        let tx = self.app_event_tx.clone();
        tokio::task::spawn_blocking(move || {
            let result = if status_host
                .persist_policy(
                    target_provider_id.as_str(),
                    ProviderActivationPolicy::Inactive,
                )
                .is_ok()
            {
                ProviderManagementPersistenceResult::Applied
            } else {
                ProviderManagementPersistenceResult::ReplacementAppliedDeactivationFailed
            };
            tx.send(AppEvent::ProviderManagerPersistenceFinished { attempt_id, result });
        });
    }

    fn provider_manager_attempt_matches(
        &self,
        attempt_id: codex_provider_auth::ProviderManagementAttemptId,
        target_provider_id: &codex_provider_auth::ProviderCatalogId,
    ) -> bool {
        matches!(
            self.provider_management_host.as_ref().map(super::super::provider_management_host::ProviderManagementHost::phase),
            Some(ProviderManagementPhase::Persisting {
                attempt_id: expected,
                mutation: codex_provider_auth::ProviderManagementMutation::ReplacementThenDeactivate {
                    target_provider_id: target,
                    ..
                },
            }) if *expected == attempt_id && target == target_provider_id
        )
    }

    pub(super) fn apply_provider_management_transition(
        &mut self,
        transition: ProviderManagementTransition,
    ) {
        for effect in transition.effects {
            match effect {
                ProviderManagementEffect::PersistEligibility {
                    attempt_id,
                    provider_id,
                    policy,
                } => {
                    let Some(host) = self.provider_management_host.as_ref() else {
                        continue;
                    };
                    let status_host = host.status_host().clone();
                    let tx = self.app_event_tx.clone();
                    tokio::task::spawn_blocking(move || {
                        let result = if status_host
                            .persist_policy(provider_id.as_str(), policy)
                            .is_ok()
                        {
                            ProviderManagementPersistenceResult::Applied
                        } else {
                            ProviderManagementPersistenceResult::Failed
                        };
                        tx.send(AppEvent::ProviderManagerPersistenceFinished {
                            attempt_id,
                            result,
                        });
                    });
                }
                ProviderManagementEffect::PresentReplacement { target_provider_id } => {
                    let Some(host) = self.provider_management_host.as_ref() else {
                        continue;
                    };
                    let catalog = host.status_host().catalog().clone();
                    let candidates = host.replacement_candidates(self.config.model.clone());
                    self.chat_widget.open_provider_replacement(
                        target_provider_id,
                        &catalog,
                        candidates,
                    );
                }
                ProviderManagementEffect::Refresh => {
                    self.schedule_provider_manager_refresh();
                }
                ProviderManagementEffect::BeginAuthentication { .. }
                | ProviderManagementEffect::PersistReplacementThenDeactivate { .. } => {}
            }
        }
    }

    fn apply_provider_management_transition_with_server(
        &mut self,
        transition: ProviderManagementTransition,
        app_server: &AppServerSession,
    ) {
        for effect in &transition.effects {
            if let ProviderManagementEffect::PersistReplacementThenDeactivate {
                attempt_id,
                target_provider_id,
                replacement,
            } = effect
            {
                let request_handle = app_server.request_handle();
                let tx = self.app_event_tx.clone();
                let attempt_id = *attempt_id;
                let target_provider_id = target_provider_id.clone();
                let replacement = replacement.clone();
                tokio::spawn(async move {
                    let success = crate::config_update::write_config_batch(
                        request_handle,
                        crate::config_update::build_model_selection_edits(
                            replacement.model.as_str(),
                            Some(replacement.runtime_provider_id.as_str()),
                            None::<String>,
                        ),
                    )
                    .await
                    .is_ok();
                    tx.send(AppEvent::ProviderManagerReplacementPersisted {
                        attempt_id,
                        target_provider_id,
                        replacement,
                        success,
                    });
                });
            }
        }
        self.apply_provider_management_transition(transition);
    }
}

pub(super) fn claude_intent_for_status(
    status: &codex_provider_auth::ProviderStatusSnapshot,
    source: codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource,
) -> Option<codex_provider_auth::claude_account_flow::ClaudeAccountIntent> {
    use codex_provider_auth::ProviderConfigurationState;
    use codex_provider_auth::claude_account_flow::ClaudeAccountIntent;
    use codex_provider_auth::claude_account_flow::ClaudeUnauthorizedRecoverySource;

    match status.configuration {
        ProviderConfigurationState::NotConfigured => Some(ClaudeAccountIntent::Add),
        ProviderConfigurationState::RecoveryRequired
            if source != ClaudeUnauthorizedRecoverySource::Unknown =>
        {
            Some(ClaudeAccountIntent::UnauthorizedRecovery { source })
        }
        _ => None,
    }
}
