use std::collections::HashMap;

use codex_model_provider_info::DEEPSEEK_API_KEY_ENV_VAR;
use codex_model_provider_info::DEEPSEEK_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_model_provider_info::built_in_model_providers;
use pretty_assertions::assert_eq;

use super::*;
use crate::*;

#[test]
fn target_derivation_covers_builtin_custom_shared_and_unsupported_entries() {
    let builtins = ProviderCatalog::from_runtime_providers(&built_in_model_providers(None));
    assert_eq!(
        ApiKeyAuthTarget::from_catalog_entry(builtins.get(DEEPSEEK_PROVIDER_ID).unwrap()),
        Ok(target(DEEPSEEK_PROVIDER_ID, DEEPSEEK_API_KEY_ENV_VAR))
    );
    assert_eq!(
        ApiKeyAuthTarget::from_catalog_entry(builtins.get(OPENAI_PROVIDER_ID).unwrap()),
        Err(ApiKeyTargetError::UnsupportedCapability)
    );
    let catalog = ProviderCatalog::from_runtime_providers(&HashMap::from([
        ("z-wire".into(), custom("Shared", "SHARED_KEY")),
        ("a-wire".into(), custom("Shared", "SHARED_KEY")),
        ("solo".into(), custom("Solo", "SOLO_KEY")),
    ]));
    assert_eq!(
        catalog
            .entries()
            .iter()
            .map(ApiKeyAuthTarget::from_catalog_entry)
            .collect::<Result<Vec<_>, _>>(),
        Ok(vec![
            target("a-wire", "SHARED_KEY"),
            target("solo", "SOLO_KEY")
        ])
    );
}

#[test]
fn fake_host_consumes_only_persistence_effect_into_existing_request_shape() {
    #[derive(Debug, PartialEq, Eq)]
    struct Request {
        provider: String,
        api_key: String,
    }
    let target = target("custom", "CUSTOM_KEY");
    let mut controller = ProviderAuthController::default();
    controller.dispatch(ProviderAuthAction::StartApiKey(ApiKeyFlowStart {
        target,
        intent: ApiKeyFlowIntent::Add,
        metadata: ApiKeyCredentialMetadata {
            environment: EnvironmentCredentialMetadata::Missing,
            managed: ManagedApiKeyMetadata::Missing,
        },
    }));
    controller.dispatch(ProviderAuthAction::SetApiKey(ApiKeySecret::new("canary")));
    let request = controller
        .dispatch(ProviderAuthAction::Submit)
        .effects
        .into_iter()
        .find_map(|effect| match effect {
            ProviderAuthEffect::PersistApiKey { target, secret, .. } => Some(Request {
                provider: target.runtime_provider_id.to_string(),
                api_key: secret.expose_secret().into(),
            }),
            _ => None,
        });
    assert_eq!(
        request,
        Some(Request {
            provider: "custom".into(),
            api_key: "canary".into()
        })
    );
}

fn custom(name: &str, env: &str) -> ModelProviderInfo {
    ModelProviderInfo {
        name: name.into(),
        env_key: Some(env.into()),
        ..Default::default()
    }
}

fn target(id: &str, env_key: &str) -> ApiKeyAuthTarget {
    ApiKeyAuthTarget {
        provider_id: ProviderCatalogId(id.into()),
        runtime_provider_id: ProviderRuntimeId(id.into()),
        env_key: env_key.into(),
    }
}
