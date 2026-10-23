//! PF-84-S06: a named Amazon Bedrock account stores only an AWS profile name
//! (not secret). Its requests are signed with that profile's credentials and
//! nothing else: no managed Bedrock key, no `AWS_BEARER_TOKEN_BEDROCK`, no
//! `AWS_PROFILE` and no environment keys. A missing or broken profile is
//! refused with recovery text before any request; it never falls back.

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use codex_aws_auth::AwsAuthContext;
use codex_aws_auth::AwsAuthError;
use codex_model_provider_info::ModelProviderAwsAuthInfo;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;

use super::mantle::BEDROCK_MANTLE_SERVICE_NAME;
use super::mantle::region_from_config;

/// Reads the profile name a named account stores (`Ok(None)`: none).
pub(super) type ProfileSource = fn(&Path, &str, &str) -> std::io::Result<Option<String>>;

#[derive(Clone, Debug)]
pub(super) struct NamedBedrockAccount {
    provider_id: String,
    name: String,
    home: Option<PathBuf>,
    profile_source: ProfileSource,
    /// The signing context of the profile last loaded. Its credentials
    /// provider (with the SDK's SSO token cache) is kept for the session;
    /// credentials are still fetched for every request, as on the default
    /// path. Edits to `~/.aws` apply from the next session.
    loaded: Arc<std::sync::Mutex<Option<(String, AwsAuthContext)>>>,
}

impl NamedBedrockAccount {
    /// The provider's named account, unless it has none (`default`) or uses
    /// command auth (which serves its own accounts).
    pub(super) fn for_provider(info: &ModelProviderInfo, home: Option<PathBuf>) -> Option<Self> {
        if info.has_command_auth() {
            return None;
        }
        let account = info.account.as_ref()?;
        Some(Self {
            provider_id: account.provider_id.clone(),
            name: account.name.clone(),
            home,
            profile_source: codex_login::provider_account_aws_profile,
            loaded: Arc::default(),
        })
    }

    #[cfg(test)]
    pub(super) fn set_profile_source(&mut self, profile_source: ProfileSource) {
        self.profile_source = profile_source;
    }

    /// The account's profile name, read from the vault on every use so a
    /// rename or removal takes effect at once.
    fn profile(&self) -> Result<String> {
        let Self {
            provider_id, name, ..
        } = self;
        let Some(home) = self.home.as_deref() else {
            return Err(CodexErr::Fatal(format!(
                "account `{name}` of provider `{provider_id}` cannot be verified: no Corbanu home \
                 is available"
            )));
        };
        match (self.profile_source)(home, provider_id, name) {
            Ok(Some(profile)) if !profile.trim().is_empty() => Ok(profile.trim().to_string()),
            Ok(_) => Err(CodexErr::Fatal(format!(
                "account `{name}` of provider `{provider_id}` holds no AWS profile; add one with \
                 `corbanu account add {provider_id} {name} --kind aws-profile --value <profile>` \
                 or pick another account with `--account` (see `corbanu account list`)"
            ))),
            Err(error) => Err(CodexErr::Fatal(format!(
                "account `{name}` of provider `{provider_id}` is unavailable: {error}"
            ))),
        }
    }

    /// The Mantle region: `aws.region` when configured, else the profile's.
    pub(super) async fn region(&self, aws: &ModelProviderAwsAuthInfo) -> Result<String> {
        let profile = self.profile()?;
        if let Some(context) = self.loaded_context(&profile) {
            return Ok(context.region().to_string());
        }
        AwsAuthContext::named_profile_region(&profile, region_from_config(aws))
            .await
            .map_err(|error| self.profile_error(&profile, &error))
    }

    /// The profile's signing context. When the profile is first loaded (or
    /// the account's profile changes) its credentials are resolved once
    /// before any request, so a broken profile is refused up front.
    pub(super) async fn context(&self, aws: &ModelProviderAwsAuthInfo) -> Result<AwsAuthContext> {
        let profile = self.profile()?;
        if let Some(context) = self.loaded_context(&profile) {
            return Ok(context);
        }
        // Not locked while loading; a concurrent first load just loads twice.
        let context = AwsAuthContext::load_named_profile(
            &profile,
            region_from_config(aws),
            BEDROCK_MANTLE_SERVICE_NAME,
        )
        .await
        .map_err(|error| self.profile_error(&profile, &error))?;
        if let Ok(mut loaded) = self.loaded.lock() {
            *loaded = Some((profile, context.clone()));
        }
        Ok(context)
    }

    /// The cached context, when it was loaded for `profile`.
    fn loaded_context(&self, profile: &str) -> Option<AwsAuthContext> {
        let loaded = self.loaded.lock().ok()?;
        loaded
            .as_ref()
            .filter(|(loaded_profile, _)| loaded_profile == profile)
            .map(|(_, context)| context.clone())
    }

    fn profile_error(&self, profile: &str, error: &AwsAuthError) -> CodexErr {
        let Self {
            provider_id, name, ..
        } = self;
        // The file and sign-in hints fit only failures to load credentials.
        let hint = if matches!(error, AwsAuthError::Credentials(_)) {
            format!(
                " Check that the profile is in ~/.aws/config or ~/.aws/credentials (or the files \
                 AWS_CONFIG_FILE and AWS_SHARED_CREDENTIALS_FILE name) and that its credentials \
                 work; for an SSO profile run `aws sso login --profile {profile}`."
            )
        } else {
            String::new()
        };
        CodexErr::Fatal(format!(
            "account `{name}` of provider `{provider_id}` uses AWS profile `{profile}`, which \
             cannot be used: {}.{hint} Corbanu never falls back to another profile or to \
             environment credentials.",
            error.detailed_message()
        ))
    }
}

#[cfg(test)]
#[path = "named_account_tests.rs"]
mod tests;
