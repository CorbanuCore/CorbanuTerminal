//! PF-27-S02 secretless agent launch: the environment allowlist applied to
//! every process Corbanu launches for an agent once a protected launch
//! contract is armed (feature `secretless_agent_launch`).
//!
//! Arming is process-wide and one-way: once a config that enables the
//! contract is loaded, every later launch in this process is filtered, so a
//! session can only become stricter. With the feature off nothing here runs.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static ARMED: AtomicBool = AtomicBool::new(false);

/// Arms the secretless launch environment for the rest of this process.
pub fn arm() {
    ARMED.store(true, Ordering::SeqCst);
}

/// True once [`arm`] has been called in this process.
pub fn is_armed() -> bool {
    ARMED.load(Ordering::SeqCst)
}

/// Name fragments that mark a variable as credential-bearing.
const SECRET_NAME_FRAGMENTS: &[&str] = &[
    "KEY",
    "SECRET",
    "TOKEN",
    "PASS",
    "CREDENTIAL",
    "VAULT",
    "AUTH",
    "COOKIE",
    "SESSION",
    "PRIVATE",
];

/// Variables (exact names) that run code at shell or tool start-up, or hand
/// out credentials through a helper or agent socket.
const STARTUP_AND_HELPER_NAMES: &[&str] = &[
    "BASH_ENV",
    "ENV",
    "ZDOTDIR",
    "PROMPT_COMMAND",
    "PERL5OPT",
    "PERL5DB",
    "PYTHONSTARTUP",
    "NODE_OPTIONS",
    "RUBYOPT",
    "SSH_AGENT_PID",
    "GPG_AGENT_INFO",
    "GNUPGHOME",
    "DOCKER_CONFIG",
    "KUBECONFIG",
    "NETRC",
    "CURL_HOME",
];

/// Prefixes with the same role as [`STARTUP_AND_HELPER_NAMES`].
const STARTUP_AND_HELPER_PREFIXES: &[&str] = &[
    "BASH_FUNC_",
    "LD_",
    "DYLD_",
    "GIT_CONFIG",
    "AWS_",
    "AZURE_",
    "NPM_CONFIG_",
];

/// Exact names an agent command may inherit.
const ALLOWED_NAMES: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "USERNAME",
    "SHELL",
    "TERM",
    "TERM_PROGRAM",
    "TERM_PROGRAM_VERSION",
    "COLORTERM",
    "COLUMNS",
    "LINES",
    "LANG",
    "LANGUAGE",
    "TZ",
    "TMPDIR",
    "TEMP",
    "TMP",
    "PWD",
    "EDITOR",
    "VISUAL",
    "PAGER",
    "LESS",
    "MANPAGER",
    "NO_COLOR",
    "FORCE_COLOR",
    "CLICOLOR",
    "CLICOLOR_FORCE",
    "CI",
    "SHLVL",
    "HOSTNAME",
    "__CF_USER_TEXT_ENCODING",
    "VIRTUAL_ENV",
    "GOROOT",
    "GOFLAGS",
    "GOPRIVATE",
    "GOPROXY",
    "SDKROOT",
    "DEVELOPER_DIR",
    "MACOSX_DEPLOYMENT_TARGET",
    "RUST_LOG",
    "RUST_BACKTRACE",
    "RUSTFLAGS",
    "CC",
    "CXX",
    "CFLAGS",
    "CXXFLAGS",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "FTP_PROXY",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "REQUESTS_CA_BUNDLE",
    "CURL_CA_BUNDLE",
    "NODE_EXTRA_CA_CERTS",
    "GIT_SSL_CAINFO",
    "PIP_CERT",
    "GH_HOST",
    "GITHUB_API_URL",
    "GITHUB_SERVER_URL",
    "PATHEXT",
    "SYSTEMROOT",
    "COMSPEC",
];

/// Prefixes an agent command may inherit (after the deny rules).
const ALLOWED_PREFIXES: &[&str] = &[
    "LC_",
    "XDG_",
    "CODEX_",
    "CORBANU_",
    "PFTERMINAL_",
    "CARGO_",
    "RUSTUP_",
    "CONDA_",
    "PYENV_",
    "NVM_",
    "HOMEBREW_",
];

/// Suffixes for tool locations (`JAVA_HOME`, `PYTHONPATH`, `ANDROID_SDK_ROOT`).
const ALLOWED_SUFFIXES: &[&str] = &["PATH", "_HOME", "_ROOT", "_DIR", "_DIRS"];

/// True when the name looks credential-bearing (`*KEY*`, `*TOKEN*`, ...).
pub fn is_secret_looking_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    SECRET_NAME_FRAGMENTS
        .iter()
        .any(|fragment| upper.contains(fragment))
        || upper.ends_with("_PAT")
}

/// True when the variable runs start-up code or reaches a credential helper.
pub fn is_startup_or_helper_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    STARTUP_AND_HELPER_NAMES.contains(&upper.as_str())
        || STARTUP_AND_HELPER_PREFIXES
            .iter()
            .any(|prefix| upper.starts_with(prefix))
}

/// True when an agent command may inherit a variable with this name.
pub fn is_launch_env_name_allowed(name: &str) -> bool {
    if cfg!(debug_assertions) && name == crate::shell_environment::NO_NATIVE_KEYRING_ENV_VAR {
        return true;
    }
    if is_secret_looking_name(name) || is_startup_or_helper_name(name) {
        return false;
    }
    let upper = name.to_ascii_uppercase();
    ALLOWED_NAMES.contains(&upper.as_str())
        || ALLOWED_PREFIXES
            .iter()
            .any(|prefix| upper.starts_with(prefix))
        || ALLOWED_SUFFIXES
            .iter()
            .any(|suffix| upper.ends_with(suffix))
}

/// True when the value is a URL that carries a password (`scheme://user:pass@host`).
pub fn value_has_url_password(value: &str) -> bool {
    let Some((_, rest)) = value.split_once("://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    authority
        .rsplit_once('@')
        .is_some_and(|(userinfo, _)| userinfo.contains(':'))
}

/// Applies the launch allowlist to an environment map. `exempt` names (for
/// example values the user set explicitly in `shell_environment_policy.set`)
/// are kept by name; every kept value is still dropped when it embeds a URL
/// password.
pub fn retain_launch_env(env: &mut HashMap<String, String>, exempt: impl Fn(&str) -> bool) {
    env.retain(|name, value| {
        (exempt(name) || is_launch_env_name_allowed(name)) && !value_has_url_password(value)
    });
}

// Credential-value heuristics shared by Core's disclosure gate and both shell
// snapshot writers (Core's snapshot file and the executor's in-memory
// snapshot, #2392). Moved here unchanged from Core's `security::disclosure_gate`.

/// Environment values shorter than this are flags, not credentials.
const MIN_ENV_VALUE_BYTES: usize = 8;
/// Values at least this long need no mix of character classes.
const LONG_SECRET_BYTES: usize = 16;

/// Secret-looking names whose values are session plumbing, not credentials.
const NON_CREDENTIAL_ENV_PREFIXES: &[&str] = &["XDG_", "GNOME_KEYRING_", "DBUS_"];
const NON_CREDENTIAL_ENV_NAMES: &[&str] = &[
    "DESKTOP_SESSION",
    "SESSION_MANAGER",
    "XAUTHORITY",
    "KEYTIMEOUT",
];
const NON_CREDENTIAL_ENV_SUFFIXES: &[&str] = &[
    "_SOCK",
    "_PID",
    "_ADDR",
    "_PATH",
    "_DIR",
    "_FILE",
    "_URL",
    "_URI",
    "_HOST",
    "_USER",
    "_USERNAME",
    "_NAME",
    "_EMAIL",
    "_REGION",
    "_ENDPOINT",
];
/// Fragments that make a secret-looking name ordinary (`GIT_AUTHOR_NAME`
/// contains `AUTH`; `KEYBOARD` contains `KEY`).
const NON_CREDENTIAL_ENV_FRAGMENTS: &[&str] = &["AUTHOR", "KEYBOARD", "KEYMAP", "KEYCHAIN"];

/// Key fragments that mark a value in a sign-in or config file as secret.
const SECRET_KEY_FRAGMENTS: &[&str] = &[
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "API_KEY",
    "APIKEY",
    "AUTHORIZATION",
    "CREDENTIAL",
    "PRIVATE_KEY",
    "ACCESS_KEY",
    "COOKIE",
];

/// The credentials one environment variable carries, labelled for a secret
/// registry: its whole (trimmed) value when the name is credential-bearing and
/// the value looks like a credential, a password embedded in a URL, and
/// secret-named URL query parameters. Every secret is a slice of `value`, so
/// callers decide how to hold copies (for example in `Zeroizing`).
pub fn env_credential_values<'a>(name: &str, value: &'a str) -> Vec<(String, &'a str)> {
    let mut values = Vec::new();
    let value = value.trim();
    if is_credential_env_name(name)
        && !value.contains(char::is_whitespace)
        && is_plausible_secret(value)
    {
        values.push((format!("env:{name}"), value));
    }
    if let Some(password) = url_password(value) {
        values.push((format!("env:{name}:password"), password));
    }
    for (param, secret) in url_query_secrets(value) {
        values.push((format!("env:{name}:{param}"), secret));
    }
    values
}

fn is_credential_env_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    is_secret_looking_name(&upper)
        && !NON_CREDENTIAL_ENV_NAMES.contains(&upper.as_str())
        && !NON_CREDENTIAL_ENV_FRAGMENTS
            .iter()
            .any(|fragment| upper.contains(fragment))
        && !NON_CREDENTIAL_ENV_PREFIXES
            .iter()
            .any(|prefix| upper.starts_with(prefix))
        && !NON_CREDENTIAL_ENV_SUFFIXES
            .iter()
            .any(|suffix| upper.ends_with(suffix))
}

/// Long enough, shaped like a credential rather than a word, not an
/// existing path, and not a variable name (`GITHUB_TOKEN` as a config value
/// names where the secret lives). Redacting ordinary text would corrupt what
/// the model reads and edits, so seeding errs towards real credentials.
pub fn is_plausible_secret(value: &str) -> bool {
    let names_a_variable = value.contains('_')
        && value.starts_with(|ch: char| ch.is_ascii_uppercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_');
    value.len() >= MIN_ENV_VALUE_BYTES
        && (value.len() >= LONG_SECRET_BYTES || character_classes(value) >= 2)
        && !(value.starts_with('/') && std::path::Path::new(value).exists())
        && !names_a_variable
}

/// Lowercase, uppercase, digit and other: how many appear.
fn character_classes(value: &str) -> usize {
    let has = |test: fn(&u8) -> bool| usize::from(value.bytes().any(|byte| test(&byte)));
    has(u8::is_ascii_lowercase)
        + has(u8::is_ascii_uppercase)
        + has(u8::is_ascii_digit)
        + has(|byte| !byte.is_ascii_alphanumeric())
}

fn url_password(value: &str) -> Option<&str> {
    if !value_has_url_password(value) {
        return None;
    }
    let (_, rest) = value.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let (userinfo, _) = authority.rsplit_once('@')?;
    let (_, password) = userinfo.split_once(':')?;
    let password = password.trim();
    is_plausible_secret(password).then_some(password)
}

/// Secret-named query parameters of a URL (`?token=`, `?api_key=`, `?sig=`).
fn url_query_secrets(value: &str) -> Vec<(&str, &str)> {
    if !value.contains("://") {
        return Vec::new();
    }
    let Some((_, query)) = value.split_once('?') else {
        return Vec::new();
    };
    let query = query.split('#').next().unwrap_or_default();
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .filter(|(name, secret)| {
            let lower = name.to_ascii_lowercase();
            (is_secret_key(name) || matches!(lower.as_str(), "key" | "sig" | "signature"))
                && is_plausible_secret(secret)
        })
        .collect()
}

/// True when a config or sign-in key names a secret (`api_key`, `token`, ...).
pub fn is_secret_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase().replace('-', "_");
    // `bearer_token_env_var = "GITHUB_TOKEN"` names a variable; it is not one.
    if upper.ends_with("_ENV_VAR") || upper.ends_with("ENV_KEY") || upper.ends_with("_ENV") {
        return false;
    }
    SECRET_KEY_FRAGMENTS
        .iter()
        .any(|fragment| upper.contains(fragment))
}

#[cfg(test)]
#[path = "secretless_launch_tests.rs"]
mod tests;
