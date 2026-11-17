//! Corbanu #2314 (main #2332) on upstream's snapshot pipeline: omit every
//! export whose raw value carries a protected credential, whatever its name,
//! and refuse a snapshot whose remaining text still contains one. Raw values
//! come from the capture's value section and never reach the snapshot file.
//!
//! D685: with a credential broker, protected values also include every
//! credential its hardened engine already brokers, so a copy such as
//! `HIDDEN_HEADER="$GH_TOKEN"` is omitted even after startup unsets the
//! source. The engine matches in place (isolated records by fingerprint); no
//! raw value is retrieved, and a check it cannot complete refuses the snapshot.

use std::ffi::OsString;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_network_proxy::BROKERED_CREDENTIALS_ENV_KEY;
use codex_network_proxy::NetworkProxy;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputGate;
use codex_secret_broker::output_gate::SecretClass;
use codex_shell_command::shell_snapshot::CapturedSnapshot;
use zeroize::Zeroizing;

/// The protected values of one capture; checks the final snapshot text.
pub(super) struct SnapshotCredentialFilter<'g> {
    local: OutputGate,
    process: &'g OutputGate,
    /// The session's credential broker, when snapshots are brokered (D685).
    brokered: Option<&'g NetworkProxy>,
}

impl SnapshotCredentialFilter<'_> {
    fn sensitive(&self, value: &str) -> Result<bool> {
        if self.local.contains_snapshot_secret(value)
            || self.process.contains_snapshot_secret(value)
        {
            return Ok(true);
        }
        let Some(proxy) = self.brokered else {
            return Ok(false);
        };
        proxy
            .brokered_credential_occurs_in(value)
            .context("Cannot check snapshot against brokered credentials")
    }

    /// Functions and aliases can embed the same values. Refuse the snapshot
    /// rather than writing a partial or syntactically altered shell program.
    pub(super) fn ensure_clean(&self, snapshot: &str) -> Result<()> {
        if self.sensitive(snapshot)? {
            bail!("Snapshot contains protected credential values");
        }
        Ok(())
    }
}

/// Apply #310's name filter and drop credential-bearing exports. Uses a
/// capture-local registry and the process registry when armed. The latter
/// includes registered vault values; enabling secret_output_gate normally
/// disables snapshots, but another session in the process may have armed it.
/// Never arm or add startup values to the process gate: snapshot hygiene must
/// not alter what commands receive or other sinks mask. `brokered` adds the
/// credentials the session's broker already holds (D685).
pub(super) fn retain_safe_exports<'g>(
    capture: &mut CapturedSnapshot<'_>,
    raw_len: usize,
    removed: &[String],
    brokered: Option<&'g NetworkProxy>,
) -> Result<SnapshotCredentialFilter<'g>> {
    retain_with_gate(capture, raw_len, removed, output_gate::global(), brokered)
}

fn retain_with_gate<'g>(
    capture: &mut CapturedSnapshot<'_>,
    raw_len: usize,
    removed: &[String],
    process_gate: &'g OutputGate,
    brokered: Option<&'g NetworkProxy>,
) -> Result<SnapshotCredentialFilter<'g>> {
    if raw_len > output_gate::MAX_SCAN_BYTES {
        bail!("Snapshot capture exceeds credential scan limit");
    }
    // Missing metadata is not a legacy snapshot: it is a failed capture.
    let variables = capture
        .values
        .as_ref()
        .filter(|values| !values.is_empty())
        .context("Snapshot capture has no variable metadata")?;
    let secret_exports = super::secret_export_patterns(removed);
    let is_secret_name = |name: &str| secret_exports.iter().any(|pattern| pattern.matches(name));
    let inherited: Vec<_> = std::env::vars_os().collect();
    let mut managed = crate::security::disclosure_gate::env_values(
        inherited.iter().cloned().chain(
            variables
                .iter()
                .map(|(name, value)| (OsString::from(*name), OsString::from(value.as_ref()))),
        ),
    );
    // #310's configured provider names may not look like credentials. Include
    // their inherited values even though launch removes them before startup.
    for (name, value) in inherited
        .iter()
        .filter_map(|(name, value)| Some((name.to_str()?, value.to_str()?)))
        .chain(
            variables
                .iter()
                .map(|(name, value)| (*name, value.as_ref())),
        )
    {
        // The broker's own marker lists brokered variables as (name, dummy
        // placeholder) pairs, "[]" when none: no real credential, and too short
        // to register. The placeholders stay protected through their variables.
        if is_secret_name(name)
            && !value.is_empty()
            && !name.eq_ignore_ascii_case(BROKERED_CREDENTIALS_ENV_KEY)
        {
            managed.push((name.to_string(), Zeroizing::new(value.to_string())));
        }
    }
    let local = OutputGate::new();
    let entries: Vec<_> = managed
        .iter()
        .map(|(name, value)| (name.as_str(), SecretClass::Operational, value.as_str()))
        .collect();
    local
        .register_all(&entries)
        .context("Cannot protect snapshot credential values")?;
    local
        .arm()
        .context("Cannot arm snapshot credential filter")?;
    let filter = SnapshotCredentialFilter {
        local,
        process: process_gate,
        brokered,
    };
    let mut check_failure = None;
    capture.retain_exports_by_value(|name, declaration, value| {
        // The capture's own helper names, and bash's own script text, are
        // skipped by the value section, so they could only be checked by their
        // shell-quoted declaration: drop them (main drops __corbanu_snapshot_*).
        if name.starts_with("__codex_snapshot_")
            || matches!(name, "BASH_EXECUTION_STRING" | "BASH_COMMAND")
            || is_secret_name(name)
        {
            return false;
        }
        let sensitive = value
            .map_or(Ok(false), |value| filter.sensitive(value))
            .and_then(|sensitive| Ok(sensitive || filter.sensitive(declaration)?));
        match sensitive {
            Ok(sensitive) => !sensitive,
            Err(error) => {
                check_failure.get_or_insert(error);
                false
            }
        }
    });
    if let Some(error) = check_failure {
        return Err(error);
    }
    Ok(filter)
}

#[cfg(test)]
#[path = "shell_snapshot_filter_tests.rs"]
mod tests;

#[cfg(all(test, unix))]
#[path = "shell_snapshot_alias_tests.rs"]
mod alias_tests;
