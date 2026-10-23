use super::CredentialHostBinding;
use super::CredentialProvider;
use super::CredentialSource;
use super::shaped_dummy_value;
use crate::policy::normalize_host;
use rama_http::HeaderMap;
use rama_http::HeaderValue;
use rama_http::header::AUTHORIZATION;
use std::collections::HashMap;

const GH_HOST_ENV_VAR: &str = "GH_HOST";
const GITHUB_TOKEN_PREFIXES: &[&str] = &["github_pat_", "ghp_", "gho_", "ghu_", "ghs_", "ghr_"];
const GITHUB_TOKEN_MIN_LEN: usize = 40;
const GITHUB_CLOUD_TOKEN_ENV_VARS: &[&str] = &["GH_TOKEN", "GITHUB_TOKEN"];
const GITHUB_ENTERPRISE_TOKEN_ENV_VARS: &[&str] =
    &["GH_ENTERPRISE_TOKEN", "GITHUB_ENTERPRISE_TOKEN"];
const GITHUB_CLOUD_HOSTS: &[&str] = &["api.github.com", "github.com"];
const GITHUB_CLOUD_HOST_SUFFIXES: &[&str] = &[".ghe.com"];

pub(super) static PROVIDER: CredentialProvider = CredentialProvider {
    context_env_vars: &[GH_HOST_ENV_VAR],
    sources: &[
        CredentialSource {
            env_vars: GITHUB_CLOUD_TOKEN_ENV_VARS,
            host_binding: github_cloud_binding,
        },
        CredentialSource {
            env_vars: GITHUB_ENTERPRISE_TOKEN_ENV_VARS,
            host_binding: github_enterprise_binding,
        },
    ],
    dummy_value,
    request_header,
    request_header_value,
    insert_request_header,
    allows_path,
};

/// API hosts take any path; web hosts take the API (GitHub Enterprise
/// Server's `/api/`), git smart HTTP and LFS.
fn allows_path(host: &str, path: &str) -> bool {
    host.starts_with("api.")
        || path.starts_with("/api/")
        || path.ends_with("/info/refs")
        || path.ends_with("/git-upload-pack")
        || path.ends_with("/git-receive-pack")
        || path.contains("/info/lfs/")
}

fn dummy_value(real_value: &str) -> String {
    shaped_dummy_value(
        real_value,
        github_token_prefix(real_value),
        GITHUB_TOKEN_MIN_LEN,
    )
}

fn request_header(headers: &HeaderMap) -> Option<&HeaderValue> {
    headers.get(AUTHORIZATION)
}

fn request_header_value(value: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(&format!("Bearer {value}")).ok()
}

fn insert_request_header(headers: &mut HeaderMap, value: HeaderValue) {
    headers.insert(AUTHORIZATION, value);
}

fn github_cloud_binding(_: &HashMap<String, String>) -> Option<CredentialHostBinding> {
    Some(CredentialHostBinding::HostPattern {
        exact_hosts: GITHUB_CLOUD_HOSTS,
        suffixes: GITHUB_CLOUD_HOST_SUFFIXES,
    })
}

fn github_enterprise_binding(env: &HashMap<String, String>) -> Option<CredentialHostBinding> {
    github_host_hint(env)
        .filter(|host| !github_cloud_host(host))
        .map(CredentialHostBinding::ExactHost)
}

fn github_cloud_host(host: &str) -> bool {
    GITHUB_CLOUD_HOSTS.contains(&host)
        || GITHUB_CLOUD_HOST_SUFFIXES
            .iter()
            .any(|suffix| host.ends_with(suffix))
}

fn github_token_prefix(value: &str) -> &str {
    GITHUB_TOKEN_PREFIXES
        .iter()
        .copied()
        .find(|prefix| value.starts_with(prefix))
        .unwrap_or("ghp_")
}

fn github_host_hint(env: &HashMap<String, String>) -> Option<String> {
    env.get(GH_HOST_ENV_VAR)
        .map(String::as_str)
        .map(normalize_host)
        .filter(|host| !host.is_empty())
}
