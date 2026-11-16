//! Corbanu #2356: the executor's in-memory snapshot gets the same credential
//! hygiene as core's snapshot file (#310 names; #2314 values, main #2332, sync
//! #2355). Startup-file exports carrying a protected credential under any name
//! are not replayed, and functions or aliases that still contain one refuse the
//! snapshot. Raw values travel only over the capture pipe and stay in memory.
//!
//! On the credential-broker path (masking on, zsh) exports are the broker's;
//! only functions, aliases and the final replay script are checked there, after
//! restoring placeholders. Omitting exports on that path awaits the owner's
//! broker-path decision for core's snapshot (#2355, D640).
//!
//! Core-only helpers cannot be shared from here: the secret-name list mirrors
//! core's `SECRET_EXPORT_NAME_PATTERNS` and the value rules mirror
//! `security::disclosure_gate::env_values`. Keep them in step.

use std::borrow::Cow;
use std::path::Path;

use codex_exec_server_protocol::JSONRPCErrorError;
use codex_protocol::config_types::EnvironmentVariablePattern;
use codex_protocol::secretless_launch;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputGate;
use codex_secret_broker::output_gate::SecretClass;
use codex_shell_command::shell_detect::ShellType;
use codex_shell_command::shell_snapshot::CapturedSnapshot;
use zeroize::Zeroizing;

use crate::protocol::ExecEnvPolicy;
use crate::rpc::internal_error;

type FilterError = (&'static str, JSONRPCErrorError);

/// Core's #310 list: conventional credential names, matched case-insensitively.
const SECRET_EXPORT_NAME_PATTERNS: &[&str] = &[
    "*_API_KEY",
    "*_TOKEN",
    "*_SECRET",
    "*_PASSWORD",
    "*_PASSWD",
    "*_PRIVATE_KEY",
    "*_CREDENTIAL",
    "*_CREDENTIALS",
    "*_SECRET_*",
    "API_KEY",
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "PRIVATE_KEY",
    "CREDENTIAL",
    "CREDENTIALS",
];

/// The broker's marker lists brokered variables as (name, dummy placeholder)
/// pairs: no real credential (as #2355; the constant is crate-private there).
const BROKERED_CREDENTIALS_ENV_KEY: &str = "CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS";

/// Opens the value section. Not an `env -0` entry (it has no `=`), so it cannot
/// be mistaken for the environment records it follows.
const VALUES_MARKER: &[u8] = b"#values";
const VALUES_END: &[u8] = b"#end";

/// Appended after the capture script, so it sees the state startup left:
/// `#values NUL (name NUL value NUL)* #end NUL count NUL`, every shell
/// variable, exported or not. A non-identifier end record and the record count
/// reject a torn capture. Builtins a startup file may shadow (`builtin`, `exec`,
/// `unset`) are not used; a failure fails the whole capture.
pub(super) fn values_script(shell_type: ShellType) -> String {
    let body = match shell_type {
        ShellType::Bash => {
            r#"(
  set +u -f
  # List names before printing: a process substitution's exit (SIGCHLD) can
  # interrupt bash 3.2's printf mid-record and tear it.
  IFS='
'
  __codex_snapshot_value_count=0
  for __codex_snapshot_value_name in $(compgen -v); do
    case "$__codex_snapshot_value_name" in
      ""|[0-9]*|*[!A-Za-z0-9_]*|__codex_snapshot_*) continue ;;
      # This capture script's own text: never a startup credential.
      BASH_EXECUTION_STRING|BASH_COMMAND) continue ;;
    esac
    RECORD "${!__codex_snapshot_value_name}" || exit 1
    COUNT
  done
  END
) || exit 1
"#
        }
        ShellType::Zsh => {
            r#"(
  # Native options, whatever startup set or argv[0] selected (`c*` is csh).
  emulate zsh 2>/dev/null || exit 1
  __codex_snapshot_value_count=0
  for __codex_snapshot_value_name in "${(@f)$(typeset +)}"; do
    __codex_snapshot_value_name=${__codex_snapshot_value_name##* }
    case "$__codex_snapshot_value_name" in
      ""|[0-9]*|*[!A-Za-z0-9_]*|__codex_snapshot_*) continue ;;
    esac
    # zsh's default IFS includes a NUL. It is shell plumbing, not a credential.
    [[ "$__codex_snapshot_value_name" == IFS && "$IFS" == $' \t\n\0' ]] && continue
    # A zsh scalar can hold NULs, which would forge records. Fail the capture.
    [[ "${(P)__codex_snapshot_value_name}" == *$'\0'* ]] && exit 1
    RECORD "${(P)__codex_snapshot_value_name}" || exit 1
    COUNT
  done
  END
) || exit 1
"#
        }
        ShellType::Sh => {
            r#"(
  set +u
  # The count lives in the pipeline's subshell, so the end record does too.
  set | __codex_snapshot_command sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p' | {
  __codex_snapshot_value_count=0
  while IFS= read -r __codex_snapshot_value_name; do
    case "$__codex_snapshot_value_name" in
      ""|[0-9]*|*[!A-Za-z0-9_]*|__codex_snapshot_*) continue ;;
      # This capture script's own text (bash-backed sh): never a startup credential.
      BASH_EXECUTION_STRING|BASH_COMMAND) continue ;;
    esac
    # Only the validated identifier enters eval.
    eval '__codex_snapshot_value=${'"$__codex_snapshot_value_name"'-}'
    RECORD "$__codex_snapshot_value" || exit 1
    COUNT
  done
  END
  }
) || exit 1
"#
        }
        ShellType::PowerShell | ShellType::Cmd => return String::new(),
    };
    format!(
        "printf '%s\\0' '{}'\n{}",
        String::from_utf8_lossy(VALUES_MARKER),
        body.replace(
            "COUNT",
            "__codex_snapshot_value_count=$((__codex_snapshot_value_count + 1))",
        )
        .replace(
            "  END\n",
            &format!(
                "  printf '%s\\0%s\\0' '{}' \"$__codex_snapshot_value_count\"\n",
                String::from_utf8_lossy(VALUES_END)
            ),
        )
        .replace(
            "RECORD",
            "printf '%s\\0%s\\0' \"$__codex_snapshot_value_name\"",
        )
    )
}

/// Captured shell variables, exported or not, in the order the shell listed them.
pub(super) type CapturedValues<'a> = Vec<(&'a str, Cow<'a, str>)>;

/// Split the value section from a capture. Returns the capture without it, in
/// the format `CapturedSnapshot::parse` expects, and the decoded values. A
/// missing or malformed section rejects the capture: it is a failed capture,
/// not an older one. Output after the section (an EXIT trap) is ignored.
pub(super) fn split_values(
    shell_type: ShellType,
    output: &[u8],
) -> Result<(&[u8], CapturedValues<'_>), FilterError> {
    let invalid = || {
        (
            "invalid_capture",
            internal_error("invalid shell snapshot value section".to_string()),
        )
    };
    let captured = CapturedSnapshot::parse(shell_type, output).ok_or_else(invalid)?;
    let mut remaining = &output[output.len() - captured.environment.len()..];
    loop {
        let record = take_record(&mut remaining).ok_or_else(invalid)?;
        if record == VALUES_MARKER {
            break;
        }
        // Every `env -0` entry has a `=`; anything else is not the environment.
        if !record.contains(&b'=') {
            return Err(invalid());
        }
    }
    let values_start = output.len() - remaining.len() - VALUES_MARKER.len() - 1;
    let mut values = Vec::new();
    loop {
        let name = take_record(&mut remaining).ok_or_else(invalid)?;
        if name == VALUES_END {
            let count = take_record(&mut remaining).ok_or_else(invalid)?;
            let count: usize = std::str::from_utf8(count)
                .ok()
                .and_then(|count| count.parse().ok())
                .ok_or_else(invalid)?;
            if count != values.len() {
                return Err(invalid());
            }
            return Ok((&output[..values_start], values));
        }
        let name = std::str::from_utf8(name).map_err(|_| invalid())?;
        if !is_plain_env_name(name) {
            return Err(invalid());
        }
        let value = take_record(&mut remaining).ok_or_else(invalid)?;
        values.push((name, String::from_utf8_lossy(value)));
    }
}

fn take_record<'a>(remaining: &mut &'a [u8]) -> Option<&'a [u8]> {
    let boundary = remaining.iter().position(|byte| *byte == 0)?;
    let record = &remaining[..boundary];
    *remaining = &remaining[boundary + 1..];
    Some(record)
}

/// The protected values of one capture, plus the process registry when armed.
pub(super) struct SnapshotCredentialFilter<'g> {
    local: OutputGate,
    process: &'g OutputGate,
    secret_names: Vec<EnvironmentVariablePattern>,
}

impl SnapshotCredentialFilter<'static> {
    /// `variables` are the capture shell's launch environment and captured
    /// values, with any broker placeholders already restored to real values.
    pub(super) fn new<'a>(
        variables: impl IntoIterator<Item = (&'a str, &'a str)>,
        env_policy: Option<&ExecEnvPolicy>,
    ) -> Result<Self, FilterError> {
        Self::with_gate(variables, env_policy, output_gate::global())
    }
}

impl<'g> SnapshotCredentialFilter<'g> {
    /// Never arm or add startup values to the process gate: snapshot hygiene
    /// must not alter what commands receive or what other sinks mask.
    pub(super) fn with_gate<'a>(
        variables: impl IntoIterator<Item = (&'a str, &'a str)>,
        env_policy: Option<&ExecEnvPolicy>,
        process: &'g OutputGate,
    ) -> Result<Self, FilterError> {
        let refuse = || {
            (
                "credential_filter",
                internal_error("cannot protect shell snapshot credential values".to_string()),
            )
        };
        let secret_names = SECRET_EXPORT_NAME_PATTERNS
            .iter()
            .map(|pattern| EnvironmentVariablePattern::new_case_insensitive(pattern))
            .collect::<Vec<_>>();
        // #310's provider credential names reach the executor as plain-name
        // excludes, mixed with runtime and user excludes. Their values are
        // protected when they look like credentials, so a runtime flag such as
        // `1` neither refuses every snapshot nor drops unrelated exports.
        let excluded = env_policy
            .map(|policy| {
                policy
                    .exclude
                    .iter()
                    .filter(|name| is_plain_env_name(name))
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut managed = Vec::new();
        let mut protect = |name: &str, value: &str| {
            // The marker's credential-looking name must not register its
            // placeholders: every brokered replay script would then be refused.
            if name.eq_ignore_ascii_case(BROKERED_CREDENTIALS_ENV_KEY) {
                return;
            }
            push_env_values(name, value, &mut managed);
            if !value.is_empty()
                && (secret_names.iter().any(|pattern| pattern.matches(name))
                    || excluded
                        .iter()
                        .any(|excluded| excluded.eq_ignore_ascii_case(name))
                        && is_plausible_secret(value.trim()))
            {
                managed.push((name.to_string(), Zeroizing::new(value.to_string())));
            }
        };
        // The executor's own environment holds the credentials #310 keeps from
        // commands; startup files could still copy them by other means.
        for (name, value) in std::env::vars_os() {
            if let (Some(name), Some(value)) = (name.to_str(), value.to_str()) {
                protect(name, value);
            }
        }
        for (name, value) in variables {
            protect(name, value);
        }
        let local = OutputGate::new();
        let entries = managed
            .iter()
            .map(|(name, value)| (name.as_str(), SecretClass::Operational, value.as_str()))
            .collect::<Vec<_>>();
        local.register_all(&entries).map_err(|_| refuse())?;
        local.arm().map_err(|_| refuse())?;
        Ok(Self {
            local,
            process,
            secret_names,
        })
    }

    pub(super) fn sensitive(&self, value: &str) -> bool {
        self.local.contains_snapshot_secret(value) || self.process.contains_snapshot_secret(value)
    }

    /// Whether a captured export may be replayed: not a capture helper, not a
    /// #310 secret name, and no protected credential in its value.
    pub(super) fn keep(&self, name: &str, value: &str) -> bool {
        !name.starts_with("__codex_snapshot_")
            && !matches!(name, "BASH_EXECUTION_STRING" | "BASH_COMMAND")
            && !self
                .secret_names
                .iter()
                .any(|pattern| pattern.matches(name))
            && !self.sensitive(value)
    }

    /// Functions and aliases can embed the same values. Refuse the snapshot
    /// rather than replaying a partial or syntactically altered shell program.
    pub(super) fn ensure_clean(&self, state: &str) -> Result<(), FilterError> {
        if self.sensitive(state) {
            return Err((
                "credential_value",
                internal_error("shell snapshot contains protected credential values".to_string()),
            ));
        }
        Ok(())
    }
}

fn is_plain_env_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().enumerate().all(|(index, byte)| {
            byte == b'_' || byte.is_ascii_alphabetic() || index > 0 && byte.is_ascii_digit()
        })
}

// Mirrors core's `security::disclosure_gate::env_values` and its helpers.

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
/// Fragments that make a secret-looking name ordinary.
const NON_CREDENTIAL_ENV_FRAGMENTS: &[&str] = &["AUTHOR", "KEYBOARD", "KEYMAP", "KEYCHAIN"];
/// Key fragments that mark a URL query parameter as secret.
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

type Labeled = Vec<(String, Zeroizing<String>)>;

/// Secret-looking environment values and passwords embedded in URLs.
fn push_env_values(name: &str, value: &str, values: &mut Labeled) {
    let value = value.trim();
    if is_credential_env_name(name)
        && !value.contains(char::is_whitespace)
        && is_plausible_secret(value)
    {
        values.push((format!("env:{name}"), Zeroizing::new(value.to_string())));
    }
    if let Some(password) = url_password(value) {
        values.push((format!("env:{name}:password"), password));
    }
    for (param, secret) in url_query_secrets(value) {
        values.push((format!("env:{name}:{param}"), secret));
    }
}

fn is_credential_env_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    secretless_launch::is_secret_looking_name(&upper)
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

/// Long enough, shaped like a credential rather than a word, not an existing
/// path, and not a variable name.
fn is_plausible_secret(value: &str) -> bool {
    let names_a_variable = value.contains('_')
        && value.starts_with(|ch: char| ch.is_ascii_uppercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_');
    value.len() >= MIN_ENV_VALUE_BYTES
        && (value.len() >= LONG_SECRET_BYTES || character_classes(value) >= 2)
        && !(value.starts_with('/') && Path::new(value).exists())
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

fn url_password(value: &str) -> Option<Zeroizing<String>> {
    if !secretless_launch::value_has_url_password(value) {
        return None;
    }
    let (_, rest) = value.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let (userinfo, _) = authority.rsplit_once('@')?;
    let (_, password) = userinfo.split_once(':')?;
    let password = password.trim();
    is_plausible_secret(password).then(|| Zeroizing::new(password.to_string()))
}

/// Secret-named query parameters of a URL (`?token=`, `?api_key=`, `?sig=`).
fn url_query_secrets(value: &str) -> Vec<(String, Zeroizing<String>)> {
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
        .map(|(name, secret)| (name.to_string(), Zeroizing::new(secret.to_string())))
        .collect()
}

fn is_secret_key(key: &str) -> bool {
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
#[path = "shell_snapshot_filter_tests.rs"]
mod tests;
