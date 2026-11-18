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

/// True when the value carries a URL password: `scheme://user:pass@host`
/// anywhere in the value (not only in its first URL), or a scheme-less proxy
/// form such as `user:pass@proxy:3128` as one of its whitespace-, comma- or
/// semicolon-separated items (#2395).
pub fn value_has_url_password(value: &str) -> bool {
    url_passwords(value).next().is_some()
}

/// Every nonempty URL password in a value, including scheme-less proxy items.
/// Passwords are borrowed verbatim: percent escapes are preserved, not decoded.
/// Consumers registering redactions must apply their minimum-length policy to
/// the decoded password as well; launch filtering deliberately catches even
/// short passwords.
pub fn url_passwords(value: &str) -> impl Iterator<Item = &str> {
    value
        .match_indices("://")
        .filter_map(|(index, _)| authority_password(&value[index + 3..]))
        .chain(
            value
                .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .filter_map(schemeless_url_password),
        )
}

/// Extract userinfo only, never a colon or at-sign in a URL path/query.
fn authority_password(rest: &str) -> Option<&str> {
    let authority = rest
        .split(|c: char| c.is_whitespace() || matches!(c, '/' | '?' | '#'))
        .next()?;
    let (userinfo, _) = authority.rsplit_once('@')?;
    let (user, full_password) = userinfo.split_once(':')?;
    // Suppress a duplicate from an earlier public URL only when the
    // scheme-less pass really extracts this same password. Otherwise retain
    // the conservative explicit-URL match (userinfo can contain separators).
    if let Some((_, item)) = authority[..user.len()].rsplit_once([',', ';']) {
        let item_start = user.len() - item.len();
        let item = rest[item_start..]
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
            .next()?;
        if schemeless_url_password(item) == Some(full_password) {
            return None;
        }
    }
    let at = authority.find('@')?;
    // Commas/semicolons can be password characters, even after a raw @.
    // End this authority only when a following item is itself a proxy URL.
    let host_end = authority
        .match_indices([',', ';'])
        .find_map(|(index, _)| {
            let next = rest[index + 1..]
                .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .next()?;
            (index > at && schemeless_url_password(next).is_some()).then_some(index)
        })
        .unwrap_or(authority.len());
    let (userinfo, _) = authority[..host_end].rsplit_once('@')?;
    let (_, password) = userinfo.split_once(':')?;
    (!password.is_empty()).then_some(password)
}

/// Prefixes written `prefix:name@host` that are not a user name: URI schemes
/// with no `//` (`mailto:a@b.example`, `sip:alice@host`, the npm alias
/// `npm:pkg@latest`) and cloud IAM member kinds (`group:devs@example.com`,
/// `serviceAccount:sa@proj.iam.gserviceaccount.com`). `user:` is deliberately
/// absent: `user:alice@example.com` has the same shape as `user:pass@proxy`.
const OPAQUE_USER_SCHEMES: &[&str] = &[
    "mailto",
    "sip",
    "sips",
    "xmpp",
    "im",
    "pres", // codespell:ignore pres
    "tel",
    "acct",
    "news",
    "npm",
    "group",
    "serviceaccount",
    "deleted",
];

/// `user:pass@host[:port][/path]` with no scheme, as curl accepts for proxies.
/// The shape is strict so that values without a password, emails, `git@host:repo`,
/// image digests (`img:tag@sha256:...`) and package specs (`npm:pkg@1.2.3`) pass.
fn schemeless_url_password(token: &str) -> Option<&str> {
    let token = token.trim_matches(|c| matches!(c, '"' | '\'' | '(' | ')' | '<' | '>'));
    let authority = token.split(['/', '?', '#']).next()?;
    let (userinfo, host_port) = authority.rsplit_once('@')?;
    let (user, password) = userinfo.split_once(':')?;
    (!user.is_empty()
        && !password.is_empty()
        && user
            .chars()
            .all(|c| c == '@' || c == '\\' || is_userinfo_char(c))
        && password
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '\\' | '"' | '`'))
        && !OPAQUE_USER_SCHEMES
            .iter()
            .any(|scheme| user.eq_ignore_ascii_case(scheme))
        && is_host_and_port(host_port))
    .then_some(password)
}

/// RFC 3986 unreserved characters, sub-delims and `%` escapes, minus the
/// item separators `,` and `;`. The user part also accepts a raw `@` (an email
/// user name) and `\` (`DOMAIN\user`); the password accepts any printable ASCII
/// except `\`, `"` and `` ` `` so that Windows paths and shell prompts pass.
fn is_userinfo_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '-' | '.' | '_' | '~' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | '=' | '%'
        )
}

/// `host`, `host:port`, `[ipv6]` or `[ipv6]:port`, where the port is 1–5 digits
/// and a dotted all-numeric host must be a four-part IPv4 address.
fn is_host_and_port(host_port: &str) -> bool {
    let (host_ok, port) = match host_port.strip_prefix('[') {
        Some(bracketed) => {
            let Some((inside, after)) = bracketed.split_once(']') else {
                return false;
            };
            let ipv6_ok = !inside.is_empty()
                && inside
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || matches!(c, ':' | '.' | '%'));
            match after {
                "" => (ipv6_ok, None),
                _ => match after.strip_prefix(':') {
                    Some(port) => (ipv6_ok, Some(port)),
                    None => return false,
                },
            }
        }
        None => match host_port.split_once(':') {
            Some((host, port)) => (is_dns_name_or_ipv4(host), Some(port)),
            None => (is_dns_name_or_ipv4(host_port), None),
        },
    };
    host_ok
        && port.is_none_or(|port| {
            (1..=5).contains(&port.len()) && port.bytes().all(|b| b.is_ascii_digit())
        })
}

fn is_dns_name_or_ipv4(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    let labels: Vec<&str> = host.split('.').collect();
    let labels_ok = labels.iter().all(|label| {
        label.starts_with(|c: char| c.is_ascii_alphanumeric())
            && label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    });
    let numeric = |label: &&str| label.bytes().all(|b| b.is_ascii_digit());
    labels_ok
        && (!labels.last().is_some_and(numeric)
            || (labels.len() == 4 && labels.iter().all(numeric)))
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

#[cfg(test)]
#[path = "secretless_launch_tests.rs"]
mod tests;
