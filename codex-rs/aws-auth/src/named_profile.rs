//! PF-84-S06: credentials and region from one named AWS profile only.
//!
//! The SDK's default chain reads `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`
//! before any profile, so a named account must not use it. This loads only
//! the shared config and credentials files (honouring `AWS_CONFIG_FILE` and
//! `AWS_SHARED_CREDENTIALS_FILE`), never `AWS_PROFILE`, and never falls back
//! to another profile. A profile's `credential_source` (`Environment`,
//! `Ec2InstanceMetadata`, `EcsContainer`) is refused too, so environment keys
//! and instance or container credentials are never used. Static keys,
//! `source_profile` role chains, SSO and `credential_process` profiles work
//! as the SDK resolves them.

// The SDK keeps these names as deprecated aliases of `aws-runtime` types;
// using them avoids a direct dependency on that crate.
#![allow(deprecated)]

use aws_config::meta::region::ProvideRegion;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::profile::ProfileFileRegionProvider;
use aws_config::profile::profile_file::ProfileFiles;
use aws_config::provider_config::ProviderConfig;
use aws_credential_types::provider::ProvideCredentials;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::provider::future;
use aws_types::region::Region;
use std::sync::Arc;
use std::sync::Mutex;

use crate::AwsAuthContext;
use crate::AwsAuthError;

/// The `credential_source` values a named account refuses.
const REFUSED_CREDENTIAL_SOURCES: [&str; 3] =
    ["Environment", "Ec2InstanceMetadata", "EcsContainer"];

/// Stands in for a refused `credential_source` and records that it was
/// asked, so the refusal is reported as such (the SDK wraps it).
#[derive(Debug)]
struct RefusedCredentialSource {
    source: &'static str,
    asked: Arc<Mutex<Option<&'static str>>>,
}

impl ProvideCredentials for RefusedCredentialSource {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        if let Ok(mut asked) = self.asked.lock() {
            *asked = Some(self.source);
        }
        future::ProvideCredentials::ready(Err(CredentialsError::invalid_configuration(format!(
            "credential_source = {} is not allowed for a named account",
            self.source
        ))))
    }
}

/// The region a named profile uses: `region_override` when set, otherwise
/// the profile's own `region`. No environment fallback. `files` replaces
/// the default AWS files (tests only).
pub(crate) async fn named_profile_region(
    profile: &str,
    region_override: Option<String>,
    files: Option<ProfileFiles>,
) -> Result<String, AwsAuthError> {
    if let Some(region) = region_override
        .map(|region| region.trim().to_string())
        .filter(|region| !region.is_empty())
    {
        return Ok(region);
    }
    let mut builder = ProfileFileRegionProvider::builder().profile_name(profile);
    if let Some(files) = files {
        builder = builder.profile_files(files);
    }
    builder
        .build()
        .region()
        .await
        .map(|region| region.to_string())
        .ok_or_else(|| AwsAuthError::MissingProfileRegion(profile.to_string()))
}

/// Loads `profile` and resolves its credentials once, so a missing profile
/// or failing credentials are reported before any request is signed.
pub(crate) async fn load_named_profile(
    profile: &str,
    region_override: Option<String>,
    service: &str,
    files: Option<ProfileFiles>,
) -> Result<AwsAuthContext, AwsAuthError> {
    if service.trim().is_empty() {
        return Err(AwsAuthError::EmptyService);
    }
    let region = named_profile_region(profile, region_override, files.clone()).await;
    // Role chains call STS, which needs the region.
    let provider_config = ProviderConfig::without_region()
        .with_region(region.as_ref().ok().cloned().map(Region::new));
    let mut builder = ProfileFileCredentialsProvider::builder()
        .configure(&provider_config)
        .profile_name(profile);
    let asked = Arc::new(Mutex::new(None));
    for source in REFUSED_CREDENTIAL_SOURCES {
        builder = builder.with_custom_provider(
            source,
            RefusedCredentialSource {
                source,
                asked: Arc::clone(&asked),
            },
        );
    }
    if let Some(files) = files {
        builder = builder.profile_files(files);
    }
    let provider = builder.build();
    // A missing profile reports itself before a missing region would.
    if let Err(error) = provider.provide_credentials().await {
        let refused = asked.lock().ok().and_then(|asked| *asked);
        return Err(match (refused, region) {
            (Some(source), _) => AwsAuthError::RefusedCredentialSource(source.to_string()),
            // The profile exists but its credentials could not be fetched
            // and it has no region (for a role chain, STS needs one): report
            // the region first; the other cause shows once it is set.
            (None, Err(region_error))
                if !matches!(
                    error,
                    CredentialsError::InvalidConfiguration(_)
                        | CredentialsError::CredentialsNotLoaded(_)
                ) =>
            {
                region_error
            }
            (None, _) => error.into(),
        });
    }
    Ok(AwsAuthContext {
        credentials_provider: SharedCredentialsProvider::new(provider),
        region: region?,
        service: service.trim().to_string(),
    })
}

#[cfg(test)]
#[path = "named_profile_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "log_sinks_tests.rs"]
mod log_sinks_tests;
