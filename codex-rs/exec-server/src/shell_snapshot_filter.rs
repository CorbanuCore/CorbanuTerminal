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
//! The secret-name list, the value-capture script and its record format, and
//! the credential-value heuristics are shared with core's snapshot file (#2392):
//! `codex_shell_command::shell_snapshot` and
//! `codex_protocol::secretless_launch::env_credential_values`.

use std::borrow::Cow;

use codex_exec_server_protocol::JSONRPCErrorError;
use codex_network_proxy::BROKERED_CREDENTIALS_ENV_KEY;
use codex_protocol::config_types::EnvironmentVariablePattern;
use codex_protocol::secretless_launch;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputGate;
use codex_secret_broker::output_gate::SecretClass;
use codex_shell_command::shell_detect::ShellType;
use codex_shell_command::shell_snapshot::CapturedSnapshot;
use codex_shell_command::shell_snapshot::SECRET_EXPORT_NAME_PATTERNS;
use codex_shell_command::shell_snapshot::SNAPSHOT_VALUES_MARKER;
use codex_shell_command::shell_snapshot::parse_snapshot_values;
use zeroize::Zeroizing;

use crate::protocol::ExecEnvPolicy;
use crate::rpc::internal_error;

type FilterError = (&'static str, JSONRPCErrorError);

/// Captured shell variables, exported or not, in the order the shell listed them.
pub(super) type CapturedValues<'a> = Vec<(&'a str, Cow<'a, str>)>;

/// Split the value section from a capture. Returns the capture without it, in
/// the format `CapturedSnapshot::parse` expects, and the decoded values
/// followed by the exported environment (a second source for the filter). A
/// missing, empty or malformed section rejects the capture: it is a failed
/// capture (every real shell has variables), not an older one. Output after
/// the section (an EXIT trap) is ignored.
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
    let mut environment = Vec::new();
    loop {
        let record = take_record(&mut remaining).ok_or_else(invalid)?;
        if record == SNAPSHOT_VALUES_MARKER {
            break;
        }
        // Every `env -0` entry has a `=`; anything else is not the environment.
        let equals = record
            .iter()
            .position(|byte| *byte == b'=')
            .ok_or_else(invalid)?;
        if let Ok(name) = std::str::from_utf8(&record[..equals]) {
            environment.push((name, String::from_utf8_lossy(&record[equals + 1..])));
        }
    }
    let values_start = output.len() - remaining.len() - SNAPSHOT_VALUES_MARKER.len() - 1;
    let mut values = parse_snapshot_values(&mut remaining).ok_or_else(invalid)?;
    if values.is_empty() {
        return Err(invalid());
    }
    values.extend(environment);
    Ok((&output[..values_start], values))
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
                        && secretless_launch::is_plausible_secret(value.trim()))
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

/// Secret-looking environment values and passwords embedded in URLs, as for
/// core's disclosure gate (`secretless_launch::env_credential_values`).
fn push_env_values(name: &str, value: &str, values: &mut Vec<(String, Zeroizing<String>)>) {
    values.extend(
        secretless_launch::env_credential_values(name, value)
            .into_iter()
            .map(|(label, secret)| (label, Zeroizing::new(secret.to_string()))),
    );
}

#[cfg(test)]
#[path = "shell_snapshot_filter_tests.rs"]
mod tests;
