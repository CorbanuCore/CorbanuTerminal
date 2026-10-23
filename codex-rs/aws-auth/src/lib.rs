mod config;
mod named_profile;
mod signing;

use std::time::SystemTime;

use aws_credential_types::provider::ProvideCredentials;
use aws_credential_types::provider::SharedCredentialsProvider;
use bytes::Bytes;
use http::HeaderMap;
use http::Method;
use thiserror::Error;

/// AWS auth configuration used to resolve credentials and sign requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsAuthConfig {
    pub profile: Option<String>,
    pub region: Option<String>,
    pub service: String,
}

/// Generic HTTP request shape consumed by SigV4 signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsRequestToSign {
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Bytes,
}

/// Signed request parts returned to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsSignedRequest {
    pub url: String,
    pub headers: HeaderMap,
}

/// Errors returned by credential loading or SigV4 signing.
/// `Debug` prints [`AwsAuthError::detailed_message`], so a `{error:?}` in a
/// log line cannot carry a credential helper's stderr (#548).
#[derive(Error)]
pub enum AwsAuthError {
    #[error("AWS service name must not be empty")]
    EmptyService,
    #[error("AWS SDK config did not resolve a credentials provider")]
    MissingCredentialsProvider,
    #[error("AWS SDK config did not resolve a region")]
    MissingRegion,
    #[error(
        "AWS profile `{0}` sets no region, or is not defined in the AWS config files; set `region` in that profile or `aws.region` for the provider"
    )]
    MissingProfileRegion(String),
    #[error(
        "`credential_source = {0}` is not allowed for a named account: it would use environment, instance or container credentials; give the profile its own keys, a `source_profile`, SSO or `credential_process`"
    )]
    RefusedCredentialSource(String),
    #[error("failed to load AWS credentials: {0}")]
    Credentials(#[from] aws_credential_types::provider::error::CredentialsError),
    #[error("request URL is not a valid URI: {0}")]
    InvalidUri(#[source] http::uri::InvalidUri),
    #[error("failed to construct HTTP request for signing: {0}")]
    BuildHttpRequest(#[source] http::Error),
    #[error("request contains a non-UTF8 header value: {0}")]
    InvalidHeaderValue(#[source] http::header::ToStrError),
    #[error("failed to build signable request: {0}")]
    SigningRequest(#[source] aws_sigv4::http_request::SigningError),
    #[error("failed to build SigV4 signing params: {0}")]
    SigningParams(String),
    #[error("SigV4 signing failed: {0}")]
    SigningFailure(#[source] aws_sigv4::http_request::SigningError),
}

/// Loaded AWS auth context that can sign outbound HTTP requests.
#[derive(Clone)]
pub struct AwsAuthContext {
    credentials_provider: SharedCredentialsProvider,
    region: String,
    service: String,
}

impl std::fmt::Debug for AwsAuthContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AwsAuthContext")
            .field("region", &self.region)
            .field("service", &self.service)
            .finish_non_exhaustive()
    }
}

impl AwsAuthContext {
    pub async fn load(config: AwsAuthConfig) -> Result<Self, AwsAuthError> {
        let sdk_config = config::load_sdk_config(&config).await?;
        let credentials_provider = config::credentials_provider(&sdk_config)?;
        let region = config::resolved_region(&sdk_config)?;

        Ok(Self {
            credentials_provider,
            region,
            service: config.service.trim().to_string(),
        })
    }

    /// PF-84-S06: credentials and region from `profile` only, never from
    /// environment keys, `AWS_PROFILE` or another profile. Credentials are
    /// resolved once here (and again for each signature), so a missing or
    /// broken profile fails before any request.
    pub async fn load_named_profile(
        profile: &str,
        region_override: Option<String>,
        service: &str,
    ) -> Result<Self, AwsAuthError> {
        named_profile::load_named_profile(profile, region_override, service, /*files*/ None).await
    }

    /// PF-84-S06: the region a named profile uses (`region_override`, then
    /// the profile's `region`), without resolving credentials.
    pub async fn named_profile_region(
        profile: &str,
        region_override: Option<String>,
    ) -> Result<String, AwsAuthError> {
        named_profile::named_profile_region(profile, region_override, /*files*/ None).await
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub async fn sign(&self, request: AwsRequestToSign) -> Result<AwsSignedRequest, AwsAuthError> {
        self.sign_at(request, SystemTime::now()).await
    }

    async fn sign_at(
        &self,
        request: AwsRequestToSign,
        time: SystemTime,
    ) -> Result<AwsSignedRequest, AwsAuthError> {
        let credentials = self.credentials_provider.provide_credentials().await?;
        signing::sign_request(&credentials, &self.region, &self.service, request, time)
    }
}

impl std::fmt::Debug for AwsAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detailed_message())
    }
}

impl AwsAuthError {
    /// This error and its sources, for messages that must say why (the
    /// SDK's top-level credential errors are terse). A `credential_process`
    /// helper's stderr, which the SDK copies into its error after `Stderr:`,
    /// is cut off: it may hold anything. Other sources report names, file
    /// locations and service errors, not file contents.
    pub fn detailed_message(&self) -> String {
        const HELPER_OUTPUT: &str = "Stderr:";
        let mut message = self.to_string();
        let mut source = std::error::Error::source(self);
        while let Some(error) = source {
            let mut text = error.to_string();
            if let Some(index) = text.find(HELPER_OUTPUT) {
                text.truncate(index);
                text.push_str("(helper output withheld)");
            }
            if !message.contains(&text) {
                message.push_str(": ");
                message.push_str(&text);
            }
            source = error.source();
        }
        // Other SDK credential errors can quote what they failed to parse.
        codex_log_guard::redact_credentials(&message).into_owned()
    }

    /// Returns whether retrying the outbound request can reasonably recover from this auth error.
    pub fn is_retryable(&self) -> bool {
        match self {
            AwsAuthError::Credentials(error) => matches!(
                error,
                aws_credential_types::provider::error::CredentialsError::ProviderTimedOut(_)
                    | aws_credential_types::provider::error::CredentialsError::ProviderError(_)
            ),
            AwsAuthError::EmptyService
            | AwsAuthError::MissingCredentialsProvider
            | AwsAuthError::MissingRegion
            | AwsAuthError::MissingProfileRegion(_)
            | AwsAuthError::RefusedCredentialSource(_)
            | AwsAuthError::InvalidUri(_)
            | AwsAuthError::BuildHttpRequest(_)
            | AwsAuthError::InvalidHeaderValue(_)
            | AwsAuthError::SigningRequest(_)
            | AwsAuthError::SigningParams(_)
            | AwsAuthError::SigningFailure(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::time::UNIX_EPOCH;

    use aws_credential_types::Credentials;
    use aws_credential_types::provider::error::CredentialsError;
    use pretty_assertions::assert_eq;

    use super::*;

    fn test_context(session_token: Option<&str>) -> AwsAuthContext {
        AwsAuthContext {
            credentials_provider: SharedCredentialsProvider::new(Credentials::new(
                "AKIDEXAMPLE",
                "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
                session_token.map(str::to_string),
                /*expires_after*/ None,
                "unit-test",
            )),
            region: "us-east-1".to_string(),
            service: "bedrock".to_string(),
        }
    }

    fn test_request() -> AwsRequestToSign {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        headers.insert("x-test-header", http::HeaderValue::from_static("present"));
        AwsRequestToSign {
            method: Method::POST,
            url: "https://bedrock-runtime.us-east-1.amazonaws.com/v1/responses".to_string(),
            headers,
            body: Bytes::from_static(br#"{"model":"openai.gpt-oss-120b-1:0"}"#),
        }
    }

    #[tokio::test]
    async fn sign_adds_sigv4_headers_and_preserves_existing_headers() {
        let signed = test_context(/*session_token*/ None)
            .sign_at(
                test_request(),
                UNIX_EPOCH + Duration::from_secs(1_700_000_000),
            )
            .await
            .expect("request should sign");

        assert_eq!(
            signing::header_value(&signed.headers, http::header::CONTENT_TYPE.as_str()),
            Some("application/json".to_string())
        );
        assert_eq!(
            signing::header_value(&signed.headers, "x-test-header"),
            Some("present".to_string())
        );
        assert_eq!(
            signed.url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/v1/responses"
        );
        assert!(
            signing::header_value(&signed.headers, http::header::AUTHORIZATION.as_str())
                .is_some_and(|value| value.starts_with("AWS4-HMAC-SHA256 "))
        );
        assert!(signing::header_value(&signed.headers, "x-amz-date").is_some());
    }

    #[test]
    fn credentials_provider_failures_are_retryable() {
        assert!(
            AwsAuthError::Credentials(CredentialsError::provider_error("temporarily unavailable"))
                .is_retryable()
        );
        assert!(
            AwsAuthError::Credentials(CredentialsError::provider_timed_out(Duration::from_secs(1)))
                .is_retryable()
        );
    }

    #[test]
    fn deterministic_aws_auth_errors_are_not_retryable() {
        assert!(!AwsAuthError::EmptyService.is_retryable());
        assert!(
            !AwsAuthError::Credentials(CredentialsError::not_loaded_no_source()).is_retryable()
        );
        assert!(
            !AwsAuthError::Credentials(CredentialsError::invalid_configuration("bad profile"))
                .is_retryable()
        );
        assert!(
            !AwsAuthError::Credentials(CredentialsError::unhandled("unexpected response"))
                .is_retryable()
        );
    }

    #[tokio::test]
    async fn sign_includes_session_token_when_credentials_have_one() {
        let signed = test_context(Some("session-token"))
            .sign_at(
                test_request(),
                UNIX_EPOCH + Duration::from_secs(1_700_000_000),
            )
            .await
            .expect("request should sign");

        assert_eq!(
            signing::header_value(&signed.headers, "x-amz-security-token"),
            Some("session-token".to_string())
        );
    }

    /// Every event as the logs database and `/feedback` capture them
    /// (TRACE for every target, no `RUST_LOG` needed).
    #[derive(Clone, Default)]
    struct CaptureAll(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for CaptureAll {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl CaptureAll {
        fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + use<> {
            use tracing_subscriber::Layer;
            use tracing_subscriber::layer::SubscriberExt;
            let writer = self.clone();
            tracing_subscriber::registry().with(
                tracing_subscriber::fmt::layer()
                    .with_writer(move || writer.clone())
                    .with_ansi(false)
                    .with_filter(
                        tracing_subscriber::filter::Targets::new()
                            .with_default(tracing::Level::TRACE),
                    ),
            )
        }

        fn contains(&self, needle: &str) -> bool {
            String::from_utf8_lossy(&self.0.lock().unwrap()).contains(needle)
        }
    }

    /// #380 review: aws-sigv4 logs the canonical request, session token
    /// included, at TRACE on every signed request.
    #[tokio::test]
    async fn guarded_sinks_never_see_the_session_token() {
        const SESSION_TOKEN: &str = "fake-aws-session-token-380-7c2d41";
        let sign = || async {
            test_context(Some(SESSION_TOKEN))
                .sign_at(
                    test_request(),
                    UNIX_EPOCH + Duration::from_secs(1_700_000_000),
                )
                .await
                .expect("request should sign")
        };

        // Parallel tests without a subscriber can register these callsites
        // as `never`: sign once so they exist, then rebuild their interest.
        sign().await;
        tracing::callsite::rebuild_interest_cache();

        let unguarded = CaptureAll::default();
        {
            let _default = tracing::subscriber::set_default(unguarded.subscriber());
            sign().await;
        }
        assert!(
            unguarded.contains(SESSION_TOKEN),
            "control: aws-sigv4 should log the token at TRACE"
        );

        let guarded = CaptureAll::default();
        let signed = {
            let _default =
                tracing::subscriber::set_default(codex_log_guard::guard(guarded.subscriber()));
            sign().await
        };
        assert_eq!(
            signing::header_value(&signed.headers, "x-amz-security-token"),
            Some(SESSION_TOKEN.to_string())
        );
        assert!(!guarded.contains(SESSION_TOKEN), "session token was logged");
    }

    #[tokio::test]
    async fn load_rejects_empty_service_name() {
        let err = AwsAuthContext::load(AwsAuthConfig {
            profile: None,
            region: None,
            service: "   ".to_string(),
        })
        .await
        .expect_err("empty service should be rejected");

        assert_eq!(err.to_string(), "AWS service name must not be empty");
    }
}
