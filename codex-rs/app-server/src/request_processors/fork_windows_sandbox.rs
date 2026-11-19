//! D721: the Windows sandbox a `thread/fork` request loads its config with.
//! Upstream pins it to the server's start-up mode, which also discarded a
//! stricter mode the request asked for. A stricter request is kept; any
//! other request still gets the start-up mode. The loaded parent's current
//! Windows sandbox is applied afterwards as a floor (D711), so neither path
//! puts the fork below its parent.

use std::collections::HashMap;

use codex_config::types::WindowsSandboxModeToml;

const WINDOWS_SANDBOX_KEY: &str = "windows.sandbox";

/// Sets the request's Windows sandbox override to the server's start-up
/// mode, or keeps the request's own mode when it is stricter.
///
/// The request can name the mode as a dotted `windows.sandbox` key or as
/// `sandbox` in a `windows` table. Overrides apply in map order and a
/// `windows` table replaces the whole table, so the mode is written to
/// exactly one place: into the request's `windows` table when it sent one,
/// otherwise as the dotted key.
pub(super) fn pin_fork_windows_sandbox(
    overrides: &mut HashMap<String, serde_json::Value>,
    server_mode: WindowsSandboxModeToml,
) {
    let dotted = overrides.remove(WINDOWS_SANDBOX_KEY);
    let windows_table = match overrides.get_mut("windows") {
        Some(serde_json::Value::Object(windows)) => Some(windows),
        _ => None,
    };
    let nested = windows_table
        .as_ref()
        .and_then(|windows| windows.get("sandbox").cloned());
    let requested = [dotted, nested]
        .into_iter()
        .flatten()
        .filter_map(|value| serde_json::from_value::<WindowsSandboxModeToml>(value).ok())
        .max_by_key(|mode| strength(*mode));
    let mode = match requested {
        Some(requested) if is_stricter(requested, server_mode) => requested,
        _ => server_mode,
    };
    let mode = serde_json::json!(mode);
    match windows_table {
        Some(windows) => {
            windows.insert("sandbox".to_string(), mode);
        }
        None => {
            overrides.insert(WINDOWS_SANDBOX_KEY.to_string(), mode);
        }
    }
}

/// MXC is a different isolation technology, not a rung on the same ladder:
/// a request involving it never replaces the start-up mode.
fn is_stricter(requested: WindowsSandboxModeToml, server_mode: WindowsSandboxModeToml) -> bool {
    matches!(
        (strength(requested), strength(server_mode)),
        (Some(requested), Some(server)) if requested > server
    )
}

fn strength(mode: WindowsSandboxModeToml) -> Option<u8> {
    match mode {
        WindowsSandboxModeToml::Unelevated => Some(1),
        WindowsSandboxModeToml::Elevated => Some(2),
        WindowsSandboxModeToml::Mxc => None,
    }
}

#[cfg(test)]
#[path = "fork_windows_sandbox_tests.rs"]
mod tests;
