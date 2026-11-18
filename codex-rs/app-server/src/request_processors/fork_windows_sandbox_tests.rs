use std::collections::HashMap;

use codex_config::types::WindowsSandboxModeToml;
use pretty_assertions::assert_eq;
use serde_json::json;

use super::pin_fork_windows_sandbox;

fn pinned(
    request: serde_json::Value,
    server_mode: WindowsSandboxModeToml,
) -> HashMap<String, serde_json::Value> {
    let mut overrides: HashMap<String, serde_json::Value> =
        serde_json::from_value(request).expect("request overrides");
    pin_fork_windows_sandbox(&mut overrides, server_mode);
    overrides
}

fn dotted(mode: &str) -> HashMap<String, serde_json::Value> {
    HashMap::from([("windows.sandbox".to_string(), json!(mode))])
}

/// D721: a fork request for a stricter Windows sandbox than the server's
/// start-up mode keeps it, in either form the request can name it.
#[test]
fn d721_a_stricter_requested_windows_sandbox_is_kept() {
    assert_eq!(
        pinned(
            json!({ "windows.sandbox": "elevated" }),
            WindowsSandboxModeToml::Unelevated
        ),
        dotted("elevated")
    );
    assert_eq!(
        pinned(
            json!({ "windows": { "sandbox": "elevated", "allow_mxc": false } }),
            WindowsSandboxModeToml::Unelevated
        ),
        HashMap::from([(
            "windows".to_string(),
            json!({ "sandbox": "elevated", "allow_mxc": false })
        )])
    );
}

/// An equal request is unchanged.
#[test]
fn d721_an_equal_requested_windows_sandbox_is_kept() {
    for mode in ["elevated", "unelevated"] {
        let server_mode = serde_json::from_value(json!(mode)).expect("mode");
        assert_eq!(
            pinned(json!({ "windows.sandbox": mode }), server_mode),
            dotted(mode)
        );
    }
}

/// D711: a looser request still gets the start-up mode (the parent's floor
/// applies after the config loads).
#[test]
fn d721_a_looser_requested_windows_sandbox_gets_the_start_up_mode() {
    assert_eq!(
        pinned(
            json!({ "windows.sandbox": "unelevated" }),
            WindowsSandboxModeToml::Elevated
        ),
        dotted("elevated")
    );
    assert_eq!(
        pinned(
            json!({ "windows": { "sandbox": "unelevated" } }),
            WindowsSandboxModeToml::Elevated
        ),
        HashMap::from([("windows".to_string(), json!({ "sandbox": "elevated" }))])
    );
    // Conflicting forms: the stricter one counts, and only one carrier stays.
    assert_eq!(
        pinned(
            json!({ "windows.sandbox": "unelevated", "windows": { "sandbox": "elevated" } }),
            WindowsSandboxModeToml::Unelevated
        ),
        HashMap::from([("windows".to_string(), json!({ "sandbox": "elevated" }))])
    );
}

/// No request, an unreadable one, or one involving MXC (a different
/// technology, not a stricter rung) gets the start-up mode.
#[test]
fn d721_other_requests_get_the_start_up_mode() {
    let cases = [
        (json!({}), WindowsSandboxModeToml::Unelevated, "unelevated"),
        (
            json!({ "windows.sandbox": "strongest" }),
            WindowsSandboxModeToml::Unelevated,
            "unelevated",
        ),
        (
            json!({ "windows.sandbox": "mxc" }),
            WindowsSandboxModeToml::Unelevated,
            "unelevated",
        ),
        (
            json!({ "windows.sandbox": "elevated" }),
            WindowsSandboxModeToml::Mxc,
            "mxc",
        ),
    ];
    for (request, server_mode, expected) in cases {
        assert_eq!(
            pinned(request.clone(), server_mode),
            dotted(expected),
            "{request}"
        );
    }
    // A `windows` table without a mode gets it there, so the table cannot
    // replace a separate dotted key.
    assert_eq!(
        pinned(
            json!({ "windows": { "allow_mxc": false } }),
            WindowsSandboxModeToml::Unelevated
        ),
        HashMap::from([(
            "windows".to_string(),
            json!({ "allow_mxc": false, "sandbox": "unelevated" })
        )])
    );
}
