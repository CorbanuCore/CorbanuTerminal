use std::path::Path;
use std::sync::Arc;

use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_login::auth::BedrockApiKeyAuth;
use codex_model_provider_info::ModelProviderAwsAuthInfo;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use codex_protocol::account::ProviderAccount;
use pretty_assertions::assert_eq;

use super::super::AmazonBedrockModelProvider;
use super::ProfileSource;
use crate::provider::ModelProvider;
use crate::provider::ProviderAccountState;

const MISSING_PROFILE: &str = "pf84-s06-profile-that-does-not-exist";

fn missing_profile(_: &Path, _: &str, _: &str) -> std::io::Result<Option<String>> {
    Ok(Some(MISSING_PROFILE.to_string()))
}

fn no_profile(_: &Path, _: &str, _: &str) -> std::io::Result<Option<String>> {
    Ok(None)
}

/// A Bedrock provider on account `work`, with a managed Bedrock key the
/// account must ignore.
fn named_provider(region: Option<&str>, source: ProfileSource) -> AmazonBedrockModelProvider {
    let mut info =
        ModelProviderInfo::create_amazon_bedrock_provider(Some(ModelProviderAwsAuthInfo {
            profile: Some("configured-profile-the-account-overrides".to_string()),
            region: region.map(str::to_string),
        }));
    info.account = Some(NamedProviderAccount {
        provider_id: "amazon-bedrock".to_string(),
        name: "work".to_string(),
    });
    let auth_manager =
        AuthManager::from_auth_for_testing(CodexAuth::BedrockApiKey(BedrockApiKeyAuth {
            api_key: "managed-bedrock-api-key".to_string(),
            region: "us-east-1".to_string(),
        }));
    let mut provider = AmazonBedrockModelProvider::new(info, Some(auth_manager));
    provider
        .named_account
        .as_mut()
        .expect("named account")
        .set_profile_source(source);
    provider
}

fn error_text<T>(result: codex_protocol::error::Result<T>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(error) => error.to_string(),
    }
}

#[tokio::test]
async fn named_account_ignores_the_managed_key_and_uses_its_profile_region() {
    let provider = named_provider(Some("us-west-2"), missing_profile);
    assert_eq!(
        (
            provider.auth_manager().is_none(),
            provider.auth().await,
            provider.account_state(),
            provider
                .runtime_base_url()
                .await
                .expect("configured region"),
        ),
        (
            true,
            None,
            Ok(ProviderAccountState {
                account: Some(ProviderAccount::AmazonBedrock {
                    uses_codex_managed_credentials: false,
                }),
                requires_openai_auth: false,
            }),
            Some("https://bedrock-mantle.us-west-2.api.aws/openai/v1".to_string()),
        )
    );
}

/// A profile that cannot be used names the account, the profile and the
/// recovery, and says nothing falls back. (Loading itself is tested with
/// injected AWS files in `codex-aws-auth`; this test reads no AWS files.)
#[test]
fn unusable_profile_errors_name_the_recovery() {
    let provider = named_provider(Some("us-west-2"), missing_profile);
    let account = provider.named_account.as_ref().expect("named account");
    let message = account
        .profile_error(
            MISSING_PROFILE,
            &codex_aws_auth::AwsAuthError::MissingProfileRegion(MISSING_PROFILE.to_string()),
        )
        .to_string();
    assert!(
        message.contains(&format!(
            "account `work` of provider `amazon-bedrock` uses AWS profile `{MISSING_PROFILE}`, \
             which cannot be used"
        )) && message.contains("never falls back"),
        "{message}"
    );
}

#[tokio::test]
async fn account_without_a_profile_is_refused() {
    let provider = named_provider(Some("us-west-2"), no_profile);
    assert_eq!(
        error_text(provider.api_auth().await),
        "Fatal error: account `work` of provider `amazon-bedrock` holds no AWS profile; add one \
         with `corbanu account add amazon-bedrock work --kind aws-profile --value <profile>` or \
         pick another account with `--account` (see `corbanu account list`)"
    );
}

#[test]
fn default_account_keeps_the_managed_key() {
    let provider = AmazonBedrockModelProvider::new(
        ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None),
        Some(AuthManager::from_auth_for_testing(
            CodexAuth::BedrockApiKey(BedrockApiKeyAuth {
                api_key: "managed-bedrock-api-key".to_string(),
                region: "us-east-1".to_string(),
            }),
        )),
    );
    assert_eq!(
        (
            provider.named_account.is_none(),
            provider.managed_auth().is_some()
        ),
        (true, true)
    );
    assert!(Arc::ptr_eq(
        &provider.auth_manager().expect("managed manager"),
        provider.auth_manager.as_ref().expect("manager"),
    ));
}
