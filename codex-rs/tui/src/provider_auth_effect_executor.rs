use std::time::Duration;

use codex_app_server_client::AppServerRequestHandle;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::LoginAccountParams;
use codex_app_server_protocol::LoginAccountResponse;
use codex_app_server_protocol::RequestId;
use codex_provider_auth::ApiKeyAuthTarget;
use codex_provider_auth::ApiKeyFlowIntent;
use codex_provider_auth::ApiKeyFlowStart;
use codex_provider_auth::ApiKeyPersistenceResult;
use codex_provider_auth::ApiKeySecret;
use codex_provider_auth::ApiKeyStorage;
use codex_provider_auth::ProviderAuthAction;
use codex_provider_auth::ProviderAuthCompletion;
use codex_provider_auth::ProviderAuthController;
use codex_provider_auth::ProviderAuthEffect;
use codex_provider_auth::ProviderAuthFailureReason;
use codex_provider_auth::ProviderAuthFlowSnapshot;
use codex_provider_auth::ProviderStatusSnapshot;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::provider_status_host::ProviderStatusHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderAuthExecutorError {
    UnsupportedEffect,
    ActionReceiverClosed,
    FlowRejected,
    SettlementFailed(ProviderAuthFailureReason),
}

/// Shared renderer-independent host for PF-50 API-key effects.
#[derive(Clone)]
pub(crate) struct ProviderAuthEffectExecutor {
    request_handle: AppServerRequestHandle,
    action_tx: UnboundedSender<ProviderAuthAction>,
    status_host: ProviderStatusHost,
}

impl ProviderAuthEffectExecutor {
    pub(crate) fn new(
        request_handle: AppServerRequestHandle,
        action_tx: UnboundedSender<ProviderAuthAction>,
        status_host: ProviderStatusHost,
    ) -> Self {
        Self {
            request_handle,
            action_tx,
            status_host,
        }
    }

    pub(crate) fn execute(
        &self,
        effect: ProviderAuthEffect,
    ) -> Result<(), ProviderAuthExecutorError> {
        match effect {
            ProviderAuthEffect::PersistApiKey {
                attempt_id,
                target,
                secret,
            } => {
                let openai_storage = matches!(target.storage, ApiKeyStorage::OpenAiAuth);
                let request = ClientRequest::LoginAccount {
                    request_id: RequestId::String(Uuid::new_v4().to_string()),
                    params: login_params(target, secret),
                };
                let request_handle = self.request_handle.clone();
                let action_tx = self.action_tx.clone();
                let status_host = self.status_host.clone();
                tokio::spawn(async move {
                    let result = match request_handle
                        .request_typed::<LoginAccountResponse>(request)
                        .await
                    {
                        Ok(LoginAccountResponse::ApiKey {}) => {
                            if openai_storage {
                                status_host.mark_openai_api_key();
                            }
                            ApiKeyPersistenceResult::Stored
                        }
                        Ok(_) => ApiKeyPersistenceResult::Rejected,
                        Err(codex_app_server_client::TypedRequestError::Transport { .. }) => {
                            ApiKeyPersistenceResult::StorageUnavailable
                        }
                        Err(_) => ApiKeyPersistenceResult::Rejected,
                    };
                    let _ = action_tx
                        .send(ProviderAuthAction::PersistenceFinished { attempt_id, result });
                });
                Ok(())
            }
            ProviderAuthEffect::ScheduleTimeout {
                attempt_id,
                timeout,
            } => {
                self.schedule_timeout(attempt_id, timeout);
                Ok(())
            }
            ProviderAuthEffect::RefreshProviderStatus { attempt_id, target } => {
                let status = self.status_host.resolve_target(&target);
                self.action_tx
                    .send(ProviderAuthAction::StatusResolved { attempt_id, status })
                    .map_err(|_| ProviderAuthExecutorError::ActionReceiverClosed)
            }
            ProviderAuthEffect::Complete(_)
            | ProviderAuthEffect::OpenAiAccount(_)
            | ProviderAuthEffect::ClaudeAccount(_) => {
                Err(ProviderAuthExecutorError::UnsupportedEffect)
            }
        }
    }

    pub(crate) async fn persist_api_key(
        request_handle: AppServerRequestHandle,
        status_host: ProviderStatusHost,
        target: ApiKeyAuthTarget,
        secret: ApiKeySecret,
    ) -> Result<ProviderStatusSnapshot, ProviderAuthExecutorError> {
        let metadata = status_host.api_key_metadata(&target);
        let intent = if matches!(
            metadata.managed,
            codex_provider_auth::ManagedApiKeyMetadata::Stored { .. }
        ) {
            ApiKeyFlowIntent::Replace
        } else {
            ApiKeyFlowIntent::Add
        };
        let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel();
        let executor = Self::new(request_handle, action_tx, status_host);
        let mut controller = ProviderAuthController::default();
        for action in [
            ProviderAuthAction::StartApiKey(ApiKeyFlowStart {
                target,
                intent,
                metadata,
            }),
            ProviderAuthAction::SetApiKey(secret),
            ProviderAuthAction::Submit,
        ] {
            if let Some(status) = apply_transition(&executor, controller.dispatch(action))? {
                return Ok(status);
            }
        }
        while let Some(action) = action_rx.recv().await {
            if let Some(status) = apply_transition(&executor, controller.dispatch(action))? {
                return Ok(status);
            }
        }
        Err(ProviderAuthExecutorError::ActionReceiverClosed)
    }

    fn schedule_timeout(
        &self,
        attempt_id: codex_provider_auth::ProviderAuthAttemptId,
        timeout: Duration,
    ) {
        let action_tx = self.action_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            let _ = action_tx.send(ProviderAuthAction::TimeoutElapsed { attempt_id });
        });
    }
}

fn apply_transition(
    executor: &ProviderAuthEffectExecutor,
    transition: codex_provider_auth::ProviderAuthTransition,
) -> Result<Option<ProviderStatusSnapshot>, ProviderAuthExecutorError> {
    if matches!(
        transition.disposition,
        codex_provider_auth::ProviderAuthDisposition::Rejected(_)
    ) {
        return Err(ProviderAuthExecutorError::FlowRejected);
    }
    if let Some(error) = settlement_failure(&transition.snapshot) {
        return Err(error);
    }
    for effect in transition.effects {
        match effect {
            ProviderAuthEffect::Complete(ProviderAuthCompletion::Configured { status, .. }) => {
                return Ok(Some(status));
            }
            ProviderAuthEffect::Complete(_) => {
                return Err(ProviderAuthExecutorError::FlowRejected);
            }
            effect => executor.execute(effect)?,
        }
    }
    Ok(None)
}

fn settlement_failure(snapshot: &ProviderAuthFlowSnapshot) -> Option<ProviderAuthExecutorError> {
    match snapshot {
        ProviderAuthFlowSnapshot::Failed { reason, .. } => {
            Some(ProviderAuthExecutorError::SettlementFailed(*reason))
        }
        _ => None,
    }
}

fn login_params(target: ApiKeyAuthTarget, secret: ApiKeySecret) -> LoginAccountParams {
    match target.storage {
        ApiKeyStorage::OpenAiAuth => LoginAccountParams::ApiKey {
            api_key: secret.into_string(),
        },
        ApiKeyStorage::EnvironmentVariable { .. } => LoginAccountParams::ProviderApiKey {
            provider: target.runtime_provider_id.to_string(),
            api_key: secret.into_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use codex_model_provider_info::ModelProviderInfo;
    use codex_provider_auth::ProviderCatalog;
    use codex_provider_auth::ProviderSetupCapability;

    use super::*;

    #[test]
    fn request_mapping_covers_openai_and_custom_and_consumes_the_secret() {
        for (id, openai, expected_provider) in
            [("openai", true, None), ("custom", false, Some("custom"))]
        {
            let provider = ModelProviderInfo {
                name: id.into(),
                env_key: (!openai).then(|| "CUSTOM_KEY".into()),
                requires_openai_auth: openai,
                ..Default::default()
            };
            let catalog = ProviderCatalog::from_runtime_providers(
                &std::collections::HashMap::from([(id.into(), provider)]),
            );
            let entry = &catalog.entries()[0];
            let capability = entry
                .setup_capabilities
                .iter()
                .find(|capability| matches!(capability, ProviderSetupCapability::ApiKey { .. }))
                .unwrap();
            let target = ApiKeyAuthTarget::from_catalog_capability(entry, capability).unwrap();
            match login_params(target, ApiKeySecret::new("executor-canary")) {
                LoginAccountParams::ApiKey { api_key } => {
                    assert_eq!(expected_provider, None);
                    assert_eq!(api_key, "executor-canary");
                }
                LoginAccountParams::ProviderApiKey { provider, api_key } => {
                    assert_eq!(Some(provider.as_str()), expected_provider);
                    assert_eq!(api_key, "executor-canary");
                }
                _ => panic!("unexpected request"),
            }
        }
    }

    #[test]
    fn correlated_failed_snapshot_terminates_the_effect_executor_without_secret_data() {
        let provider = ModelProviderInfo {
            name: "custom".into(),
            env_key: Some("CUSTOM_KEY".into()),
            ..Default::default()
        };
        let catalog =
            ProviderCatalog::from_runtime_providers(&std::collections::HashMap::from([(
                "custom".into(),
                provider,
            )]));
        let entry = &catalog.entries()[0];
        let capability = entry
            .setup_capabilities
            .iter()
            .find(|capability| matches!(capability, ProviderSetupCapability::ApiKey { .. }))
            .unwrap();
        let target = ApiKeyAuthTarget::from_catalog_capability(entry, capability).unwrap();
        let snapshot = ProviderAuthFlowSnapshot::Failed {
            flow: codex_provider_auth::ApiKeyFlowContext {
                target,
                intent: ApiKeyFlowIntent::Replace,
            },
            reason: ProviderAuthFailureReason::StorageUnavailable,
        };

        assert_eq!(
            settlement_failure(&snapshot),
            Some(ProviderAuthExecutorError::SettlementFailed(
                ProviderAuthFailureReason::StorageUnavailable
            ))
        );
        assert!(!format!("{snapshot:?}").contains("secret-canary"));
    }
}
