//! Raw values travel only over the capture pipe, never through a snapshot file.
//! Frame each export separately so quoting, embedded newlines and concatenation
//! cannot hide a credential or leave half a declaration behind when it is dropped.

use std::collections::HashMap;
use std::ffi::OsString;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::OutputGate;
use codex_secret_broker::output_gate::SecretClass;
use zeroize::Zeroizing;

use crate::shell::ShellType;

pub(super) fn capture_script(script: String, shell_type: ShellType) -> String {
    let (names, value, exports, declaration) = match shell_type {
        ShellType::Bash => (
            "compgen -v",
            "${!__corbanu_snapshot_name}",
            "compgen -e",
            "declare -xp \"$__corbanu_snapshot_name\"",
        ),
        ShellType::Zsh => (
            "printf '%s\\n' ${(k)parameters}",
            "${(P)__corbanu_snapshot_name}",
            "printf '%s\\n' ${(k)parameters}",
            "export -p \"$__corbanu_snapshot_name\"",
        ),
        ShellType::Sh => (
            "set | sed -n 's/^\\([A-Za-z_][A-Za-z0-9_]*\\)=.*/\\1/p'",
            "${__corbanu_snapshot_value}",
            "export -p | sed -nE 's/^(export|declare -x) ([A-Za-z_][A-Za-z0-9_]*).*/\\2/p'",
            // The raw value is quoted by Rust; POSIX exports have no tied arrays.
            ":",
        ),
        ShellType::PowerShell | ShellType::Cmd => return script,
    };
    let lookup = if shell_type == ShellType::Sh {
        "eval '__corbanu_snapshot_value=${'\"$__corbanu_snapshot_name\"'-}'"
    } else {
        ":"
    };
    // zsh includes a NUL in its default IFS. It is shell plumbing, not a
    // credential; every other embedded NUL fails the framing check closed.
    let skip_default_ifs = if shell_type == ShellType::Zsh {
        r#"[[ "$__corbanu_snapshot_name" == IFS && "$IFS" == $' \t\n\0' ]] && continue"#
    } else {
        ":"
    };
    let values = format!(
        r#"for __corbanu_snapshot_name in $({names}); do
  case "$__corbanu_snapshot_name" in
    ""|[0-9]*|*[!A-Za-z0-9_]*) continue ;;
  esac
  {skip_default_ifs}
  {lookup}
  printf '\0value\0%s\0%s\0' "$__corbanu_snapshot_name" "{value}"
done
"#
    );
    // Keep the existing startup-file loading, functions, aliases and options.
    // Only replace export serialization, retaining the #310 name filter below.
    let marker = if shell_type == ShellType::Zsh {
        "print '# Snapshot file'"
    } else {
        "echo '# Snapshot file'"
    };
    let script = script.replacen(marker, &format!("{values}{marker}"), /*count*/ 1);
    let exports_start = if shell_type == ShellType::Sh {
        "if export -p >/dev/null 2>&1; then"
    } else {
        "export_lines="
    };
    let Some((prefix, _)) = script.split_once(exports_start) else {
        return String::new(); // A changed script format must fail capture closed.
    };
    if shell_type == ShellType::Sh {
        // NUL-delimited environment entries preserve literal newlines and "=".
        // Parsing export -p line by line can mistake a value for another export.
        return format!(
            "{prefix}\nprintf '# exports\\n'\nprintf '\\0environment\\0'\ncommand env -0 || exit 1\nprintf '\\0'\n"
        );
    }
    let skip_readonly = if shell_type == ShellType::Zsh {
        "[[ \"${parameters[$__corbanu_snapshot_name]}\" != *export* || \"${parameters[$__corbanu_snapshot_name]}\" == *readonly* ]] && continue"
    } else {
        ":"
    };
    format!(
        r#"{prefix}
printf '# exports\n'
for __corbanu_snapshot_name in $({exports}); do
  case "$__corbanu_snapshot_name" in
    ""|[0-9]*|*[!A-Za-z0-9_]*) continue ;;
  esac
  {lookup}
  {skip_readonly}
  printf '\0export\0%s\0%s\0' "$__corbanu_snapshot_name" "{value}"
  {declaration}
  printf '\0'
done
"#
    )
}

/// Apply both the process registry (including vault-resolved values) and a
/// capture-local registry. Never arm or add startup values to the process gate:
/// snapshot hygiene must not alter what commands receive or other sinks mask.
pub(super) fn filter_snapshot(raw: &str, removed: &[String]) -> Result<String> {
    filter_with_gate(raw, removed, output_gate::global())
}

fn filter_with_gate(raw: &str, removed: &[String], process_gate: &OutputGate) -> Result<String> {
    if raw.len() > output_gate::MAX_SCAN_BYTES {
        bail!("Snapshot capture exceeds credential scan limit");
    }
    let mut fields = raw.split('\0');
    let mut text = fields.next().unwrap_or_default().to_string();
    let mut variables = HashMap::new();
    let mut exports = Vec::new();
    while let Some(tag) = fields.next() {
        if tag == "environment" {
            loop {
                let entry = fields.next().context("Incomplete snapshot environment")?;
                if entry.is_empty() {
                    break;
                }
                let (name, value) = entry
                    .split_once('=')
                    .context("Invalid snapshot environment")?;
                if super::is_plain_env_name(name) {
                    exports.push((name, value, ""));
                }
            }
            text.push_str(fields.next().context("Incomplete snapshot capture frame")?);
            continue;
        }
        let name = fields.next().context("Missing snapshot variable name")?;
        let value = fields.next().context("Missing snapshot variable value")?;
        if !super::is_plain_env_name(name) {
            bail!("Invalid snapshot variable name");
        }
        match tag {
            "value" => {
                variables.insert(name.to_string(), Zeroizing::new(value.to_string()));
            }
            "export" => {
                let declaration = fields.next().context("Missing snapshot export")?;
                exports.push((name, value, declaration));
            }
            _ => bail!("Invalid snapshot capture frame"),
        }
        text.push_str(fields.next().context("Incomplete snapshot capture frame")?);
    }
    // Missing metadata is not a legacy snapshot: it is a failed capture.
    if variables.is_empty() {
        bail!("Snapshot capture has no variable metadata");
    }
    let inherited: Vec<_> = std::env::vars_os().collect();
    let mut managed = crate::security::disclosure_gate::env_values(
        inherited.iter().cloned().chain(
            variables
                .iter()
                .map(|(name, value)| (OsString::from(name), OsString::from(value.as_str()))),
        ),
    );
    let secret_name =
        regex_lite::Regex::new(&format!("(?i)^({})$", super::SECRET_EXPORT_NAME_REGEX))?;
    // #310's configured provider names may not look like credentials. Include
    // their inherited values even though launch removes them before startup.
    for (name, value) in inherited
        .iter()
        .filter_map(|(name, value)| Some((name.to_str()?, value.to_str()?)))
        .chain(
            variables
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str())),
        )
    {
        if (secret_name.is_match(name)
            || removed
                .iter()
                .any(|removed| removed.eq_ignore_ascii_case(name)))
            && !value.is_empty()
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
    let sensitive = |value: &str| {
        local.contains_snapshot_secret(value) || process_gate.contains_snapshot_secret(value)
    };
    let excluded = regex_lite::Regex::new(&format!(
        "(?i)^({})$",
        super::excluded_exports_regex(removed)
    ))?;
    let mut snapshot = super::strip_snapshot_preamble(&text)?;
    for (name, value, declaration) in exports {
        if name.starts_with("__corbanu_snapshot_")
            || excluded.is_match(name)
            || sensitive(value)
            || sensitive(declaration)
        {
            continue;
        }
        if declaration.is_empty() {
            // POSIX sh has no portable per-variable declaration printer.
            let value = value.replace('\'', "'\\''");
            snapshot.push_str(&format!("export {name}='{value}'\n"));
        } else {
            snapshot.push_str(declaration);
        }
    }
    // Functions/aliases can embed the same values too. Refuse the snapshot
    // rather than writing a partial or syntactically altered shell program.
    if sensitive(&snapshot) {
        bail!("Snapshot contains protected credential values");
    }
    Ok(snapshot)
}

#[cfg(test)]
#[path = "shell_snapshot_filter_tests.rs"]
mod tests;
