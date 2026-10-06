mod github;
mod openai;

use rama_http::HeaderMap;
use rama_http::HeaderValue;
use rand::Rng as _;
use std::collections::HashMap;
use zeroize::Zeroizing;

const DUMMY_ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

type RequestHeader = for<'a> fn(&'a HeaderMap) -> Option<&'a HeaderValue>;

/// Describes how one credential family is recognized and injected.
///
/// Providers must be declared as `static` values because the broker uses their addresses as stable
/// identities when deduplicating credential records.
pub(super) struct CredentialProvider {
    context_env_vars: &'static [&'static str],
    sources: &'static [CredentialSource],
    dummy_value: fn(&str) -> String,
    request_header: RequestHeader,
    request_header_value: fn(&str) -> Option<HeaderValue>,
    insert_request_header: fn(&mut HeaderMap, HeaderValue),
    /// PF-28-S02: request paths (query removed) the credential may be sent
    /// to on a bound host.
    allows_path: fn(host: &str, path: &str) -> bool,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum CredentialHostBinding {
    ExactHost(String),
    HostPattern {
        exact_hosts: &'static [&'static str],
        suffixes: &'static [&'static str],
    },
}

pub(super) struct CredentialSource {
    pub(super) env_vars: &'static [&'static str],
    pub(super) host_binding: fn(&HashMap<String, String>) -> Option<CredentialHostBinding>,
}

const CREDENTIAL_PROVIDERS: &[&CredentialProvider] = &[&github::PROVIDER, &openai::PROVIDER];

impl CredentialProvider {
    pub(super) fn sources(&self) -> &[CredentialSource] {
        self.sources
    }

    pub(super) fn dummy_value(&self, real_value: &str) -> String {
        (self.dummy_value)(real_value)
    }

    pub(super) fn request_header<'a>(&self, headers: &'a HeaderMap) -> Option<&'a HeaderValue> {
        (self.request_header)(headers)
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
fn plain_path(path: &str) -> bool {
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
    pub(super) fn matches_host(&self, host: &str) -> bool {
        match self {
            Self::ExactHost(expected_host) => host == expected_host,
            Self::HostPattern {
                exact_hosts,
                suffixes,
            } => {
                exact_hosts.contains(&host) || suffixes.iter().any(|suffix| host.ends_with(suffix))
            }
        }
    }
}

pub(super) fn credential_broker_env_keys() -> impl Iterator<Item = &'static str> {
    credential_providers()
        .flat_map(|provider| provider.context_env_vars.iter().copied())
        .chain(
            credential_providers()
                .flat_map(CredentialProvider::sources)
                .flat_map(|source| source.env_vars.iter().copied()),
        )
}

pub(super) fn credential_providers() -> impl Iterator<Item = &'static CredentialProvider> {
    CREDENTIAL_PROVIDERS.iter().copied()
}

pub(super) fn openai_provider() -> &'static CredentialProvider {
    &openai::PROVIDER
}

#[cfg(unix)]
pub(super) fn provider_by_id(
    id: super::isolated::protocol::ProviderId,
) -> &'static CredentialProvider {
    match id {
        super::isolated::protocol::ProviderId::Github => &github::PROVIDER,
        super::isolated::protocol::ProviderId::Openai => &openai::PROVIDER,
    }
}

#[cfg(unix)]
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
}

fn shaped_dummy_value(real_value: &str, prefix: &str, minimum_len: usize) -> String {
    let target_len = real_value.len().max(minimum_len).max(prefix.len() + 16);
    let mut rng = rand::rng();
    let mut dummy = String::with_capacity(target_len);
    dummy.push_str(prefix);
    for index in prefix.len()..target_len {
        let character = match real_value.as_bytes().get(index).copied() {
            Some(template) if !template.is_ascii_alphanumeric() => template,
            _ => DUMMY_ALPHANUMERIC[rng.random_range(0..DUMMY_ALPHANUMERIC.len())],
        };
        dummy.push(char::from(character));
    }
    dummy
}
