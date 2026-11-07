mod github;
mod openai;

use super::destination::CredentialDestination;
use rama_http::HeaderMap;
use rama_http::HeaderValue;
use rand::Rng as _;
use std::collections::HashMap;
<<<<<<< HEAD
use zeroize::Zeroizing;
||||||| 413492cd6c
=======
use url::Url;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

const DUMMY_ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

type TranslateRequestHeader = fn(&HeaderMap, &str, &str) -> Option<HeaderValue>;

/// Describes how one credential family is recognized and injected.
///
/// Providers must be declared as `static` values because the broker uses their addresses as stable
/// identities when deduplicating credential records.
pub(super) struct CredentialProvider {
    pub(super) context_env_vars: &'static [&'static str],
    pub(super) credential_prefixes: &'static [&'static str],
    pub(super) ignored_credential_prefixes: &'static [&'static str],
    pub(super) credential_watermark: Option<&'static str>,
    pub(super) minimum_credential_len: usize,
    sources: &'static [CredentialSource],
    pub(super) reset_on_configuration_change: bool,
    dummy_value: fn(&str) -> String,
    translate_request_header: TranslateRequestHeader,
    request_header_value: fn(&str) -> Option<HeaderValue>,
    insert_request_header: fn(&mut HeaderMap, HeaderValue),
    /// PF-28-S02: request paths (query removed) the credential may be sent
    /// to on a bound host.
    allows_path: fn(host: &str, path: &str) -> bool,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum CredentialHostBinding {
    ExactHost(String),
    ExactHosts(Vec<String>),
    HostPattern {
        exact_hosts: &'static [&'static str],
        suffixes: &'static [&'static str],
    },
    ConfiguredHosts(Vec<CredentialDestination>),
}

pub(super) struct CredentialSource {
    pub(super) env_vars: &'static [&'static str],
    pub(super) binding_env_vars: &'static [&'static str],
    // An explicit invalid hint can clear dynamic destinations while leaving a static fallback.
    pub(super) invalidates_host_binding: fn(&HashMap<String, String>) -> bool,
    pub(super) host_binding:
        fn(&HashMap<String, String>, Option<&str>) -> Option<CredentialHostBinding>,
}

const CREDENTIAL_PROVIDERS: &[&CredentialProvider] = &[&github::PROVIDER, &openai::PROVIDER];

impl CredentialProvider {
    pub(super) fn sources(&self) -> &[CredentialSource] {
        self.sources
    }

    pub(super) fn dummy_value(&self, real_value: &str) -> String {
        (self.dummy_value)(real_value)
    }

    pub(super) fn translate_request_header(
        &self,
        headers: &HeaderMap,
        expected_value: &str,
        replacement_value: &str,
    ) -> Option<HeaderValue> {
        (self.translate_request_header)(headers, expected_value, replacement_value)
    }

    pub(super) fn request_header_value(&self, value: &str) -> Option<HeaderValue> {
        (self.request_header_value)(value)
    }

    pub(super) fn insert_request_header(&self, headers: &mut HeaderMap, value: HeaderValue) {
        (self.insert_request_header)(headers, value);
    }

    /// PF-28-S02: whether a credential of this provider may be sent with
    /// this request (its host binding is checked separately). HTTPS on 443,
    /// ordinary methods, a plain path the provider uses.
    pub(super) fn allows_request(
        &self,
        scheme: &str,
        host: &str,
        port: u16,
        method: &str,
        path: &str,
    ) -> Result<(), super::ScopedCredentialInjectionError> {
        use super::ScopedCredentialInjectionError as Denied;
        if scheme != "https" {
            return Err(Denied::SchemeDenied);
        }
        if port != 443 {
            return Err(Denied::PortDenied);
        }
        if !matches!(method, "GET" | "HEAD" | "POST" | "PUT" | "PATCH" | "DELETE") {
            return Err(Denied::MethodDenied);
        }
        let path = path.split_once('?').map_or(path, |(path, _)| path);
        if !plain_path(path) || !(self.allows_path)(host, path) {
            return Err(Denied::PathDenied);
        }
        Ok(())
    }
}

/// An absolute path with no dot segments, empty segments, backslashes, path
/// parameters or encoded separators (single or double), so the origin cannot
/// resolve it outside the paths a provider allows.
pub(super) fn plain_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    path.starts_with('/')
        && path.len() <= 4096
        && path
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'\\' | b'#' | b';'))
        && !["%2e", "%2f", "%5c", "%25", "//"]
            .iter()
            .any(|bad| lower.contains(bad))
        && !path
            .split('/')
            .skip(1)
            .any(|segment| matches!(segment, "." | ".."))
}

impl CredentialHostBinding {
    pub(super) fn bypasses_proxy_with_local_binding(&self) -> bool {
        use crate::policy::is_default_proxy_bypass_host;

        match self {
            Self::ExactHost(host) => is_default_proxy_bypass_host(host),
            Self::ExactHosts(hosts) => hosts.iter().any(|host| is_default_proxy_bypass_host(host)),
            Self::HostPattern { exact_hosts, .. } => exact_hosts
                .iter()
                .any(|host| is_default_proxy_bypass_host(host)),
            Self::ConfiguredHosts(destinations) => destinations
                .iter()
                .any(CredentialDestination::bypasses_proxy_with_local_binding),
        }
    }

    pub(super) fn matches_host(&self, host: &str, port: u16) -> bool {
        match self {
            Self::ExactHost(expected_host) => host == expected_host,
            Self::ExactHosts(expected_hosts) => {
                expected_hosts.iter().any(|expected| host == expected)
            }
            Self::HostPattern {
                exact_hosts,
                suffixes,
            } => {
                exact_hosts.contains(&host) || suffixes.iter().any(|suffix| host.ends_with(suffix))
            }
            Self::ConfiguredHosts(destinations) => destinations
                .iter()
                .any(|destination| destination.matches_host(host, port)),
        }
    }

    // Hooks apply to the host regardless of each credential destination's port or path.
    pub(super) fn matches_hostname(&self, host: &str) -> bool {
        match self {
            Self::ConfiguredHosts(destinations) => destinations
                .iter()
                .any(|destination| destination.matches_hostname(host)),
            Self::ExactHost(_) | Self::ExactHosts(_) | Self::HostPattern { .. } => {
                self.matches_host(host, /*port*/ 443)
            }
        }
    }

    pub(super) fn requires_mitm(&self, host: &str, port: u16) -> bool {
        match self {
            Self::ConfiguredHosts(destinations) => destinations
                .iter()
                .any(|destination| destination.requires_mitm(host, port)),
            Self::ExactHost(_) | Self::ExactHosts(_) | Self::HostPattern { .. } => {
                self.matches_host(host, port)
            }
        }
    }

    pub(super) fn matches_request(&self, host: &str, request: Option<&Url>) -> bool {
        match self {
            Self::ConfiguredHosts(destinations) => destinations
                .iter()
                .any(|destination| destination.matches_request(host, request)),
            Self::ExactHost(_) | Self::ExactHosts(_) | Self::HostPattern { .. } => {
                request.is_none_or(|request| request.scheme() == "https")
                    && self.matches_host(
                        host,
                        request.and_then(Url::port_or_known_default).unwrap_or(443),
                    )
            }
        }
    }
}

pub(super) fn credential_context_env_keys(
    brokered_keys: &[String],
) -> impl Iterator<Item = &'static str> + '_ {
    credential_providers()
        .filter(move |provider| {
            provider.sources().iter().any(|source| {
                source.env_vars.iter().any(|key| {
                    brokered_keys
                        .iter()
                        .any(|brokered_key| super::env_key_matches(brokered_key, key))
                })
            })
        })
        .flat_map(|provider| provider.context_env_vars.iter().copied())
}

pub(super) fn credential_binding_env_keys(
    brokered_keys: &[String],
) -> impl Iterator<Item = &'static str> + '_ {
    credential_providers()
        .flat_map(CredentialProvider::sources)
        .filter(move |source| {
            source.env_vars.iter().any(|key| {
                brokered_keys
                    .iter()
                    .any(|brokered_key| super::env_key_matches(brokered_key, key))
            })
        })
        .flat_map(|source| source.binding_env_vars.iter().copied())
}

pub(super) fn credential_env_keys() -> impl Iterator<Item = &'static str> {
    credential_providers()
        .flat_map(CredentialProvider::sources)
        .flat_map(|source| source.env_vars.iter().copied())
}

pub(super) fn credential_providers() -> impl Iterator<Item = &'static CredentialProvider> {
    CREDENTIAL_PROVIDERS.iter().copied()
}

<<<<<<< HEAD
pub(super) fn openai_provider() -> &'static CredentialProvider {
    &openai::PROVIDER
}

#[cfg(any(unix, windows))]
pub(super) fn provider_by_id(
    id: super::isolated::protocol::ProviderId,
) -> &'static CredentialProvider {
    match id {
        super::isolated::protocol::ProviderId::Github => &github::PROVIDER,
        super::isolated::protocol::ProviderId::Openai => &openai::PROVIDER,
    }
}

#[cfg(any(unix, windows))]
pub(super) fn provider_id(
    provider: &'static CredentialProvider,
) -> Option<super::isolated::protocol::ProviderId> {
    if std::ptr::eq(provider, &github::PROVIDER) {
        Some(super::isolated::protocol::ProviderId::Github)
    } else if std::ptr::eq(provider, &openai::PROVIDER) {
        Some(super::isolated::protocol::ProviderId::Openai)
    } else {
        None
    }
}

// Only the protected scoped route uses this path. Legacy provider behavior and
// dummy shaping remain unchanged. The final wire header is necessarily a copy;
// its sensitive flag prevents Debug disclosure, not memory persistence.
pub(super) fn scoped_openai_header_value(value: &str) -> Option<HeaderValue> {
    let mut bearer = Zeroizing::new(String::with_capacity("Bearer ".len() + value.len()));
    bearer.push_str("Bearer ");
    bearer.push_str(value);
    let mut header = HeaderValue::from_str(&bearer).ok()?;
    header.set_sensitive(true);
    Some(header)
||||||| 413492cd6c
=======
pub(super) fn translate_standard_request_header(
    headers: &HeaderMap,
    expected_value: &str,
    replacement_value: &str,
) -> Option<HeaderValue> {
    github::translate_request_header(headers, expected_value, replacement_value)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

fn shaped_dummy_value(real_value: &str, prefix: &str, minimum_len: usize) -> String {
    let mut dummy = format!("{prefix}masked");
    // Keep over 128 bits of randomness after the visible marker.
    let target_len = real_value.len().max(minimum_len).max(dummy.len() + 22);
    dummy.reserve(target_len - dummy.len());
    let mut rng = rand::rng();
    for _ in dummy.len()..target_len {
        let character = DUMMY_ALPHANUMERIC[rng.random_range(0..DUMMY_ALPHANUMERIC.len())];
        dummy.push(char::from(character));
    }
    dummy
}
