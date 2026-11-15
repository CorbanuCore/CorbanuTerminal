//! Main #2332's filter tests, ported to upstream's capture format.
use super::*;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_shell_command::shell_detect::ShellType;
use pretty_assertions::assert_eq;

fn capture(value: &str) -> String {
    format!(
        "# Snapshot file\n\0\0#values\0SAFE_EXPORT\0ordinary\0HIDDEN_HEADER\0{value}\0\0HIDDEN_HEADER\0export HIDDEN_HEADER='serialized';\n\0SAFE_EXPORT\0export SAFE_EXPORT='ordinary';\n\0\0"
    )
}

fn filter_with_gate(raw: &str, removed: &[String], gate: &OutputGate) -> Result<String> {
    let mut capture = CapturedSnapshot::parse(ShellType::Bash, raw.as_bytes())
        .context("invalid shell snapshot capture")?;
    let filter = retain_with_gate(&mut capture, raw.len(), removed, gate)?;
    let snapshot = capture.render_script(&ShellEnvironmentPolicy::default());
    filter.ensure_clean(&snapshot)?;
    Ok(snapshot)
}

#[test]
fn snapshot_uses_registered_vault_values_before_shell_quoting() -> Result<()> {
    let gate = OutputGate::new();
    let secret = "vault-'quoted'\\and\nmultiline-2314";
    gate.register("vault:fixture", SecretClass::Operational, secret)?;
    gate.arm()?;
    for value in [secret.to_string(), format!("prefix:{secret}:suffix")] {
        let snapshot = filter_with_gate(&capture(&value), &[], &gate)?;
        assert_eq!(
            snapshot,
            "# Snapshot file\n# exports (native declarations)\nexport SAFE_EXPORT='ordinary';\n"
        );
    }
    Ok(())
}

#[test]
fn snapshot_drops_short_credentials_inside_concatenations() -> Result<()> {
    let gate = OutputGate::new();
    gate.register("vault:short-fixture", SecretClass::Operational, "s3c")?;
    gate.arm()?;
    // Preserve ordinary prose masking while applying the stricter snapshot rule.
    assert!(
        gate.scrub(output_gate::OutputSink::ToolResult, "prefixs3csuffix")
            .is_none()
    );
    let snapshot = filter_with_gate(&capture("prefixs3csuffix"), &[], &gate)?;
    assert!(!snapshot.contains("HIDDEN_HEADER"));
    assert!(snapshot.contains("SAFE_EXPORT"));
    Ok(())
}

#[test]
fn snapshot_preserves_unmatched_values() -> Result<()> {
    let snapshot = filter_with_gate(&capture("ordinary header"), &[], &OutputGate::new())?;
    assert_eq!(
        snapshot,
        "# Snapshot file\n# exports (native declarations)\nexport HIDDEN_HEADER='serialized';\nexport SAFE_EXPORT='ordinary';\n"
    );
    Ok(())
}

#[test]
fn snapshot_rejects_capture_without_variable_metadata() {
    for raw in [
        // A capture without the value section (e.g. a changed capture script).
        "# Snapshot file\n\0\0HIDDEN_HEADER\0export HIDDEN_HEADER=raw\n\0\0",
        // An empty value section.
        "# Snapshot file\n\0\0#values\0\0HIDDEN_HEADER\0export HIDDEN_HEADER=raw\n\0\0",
        // Incomplete or unframed captures do not parse.
        "# Snapshot file\nexport HIDDEN_HEADER=raw\n",
        "# Snapshot file\n\0\0#values\0GH_TOKEN",
        "# Snapshot file\n\0\0#values\0GH_TOKEN\0a-secret-value\0\0HEADER\0raw",
    ] {
        assert!(
            filter_with_gate(raw, &[], &OutputGate::new()).is_err(),
            "{raw:?}"
        );
    }
}

#[test]
fn snapshot_drops_values_of_secret_and_provider_names() -> Result<()> {
    // A captured secret-named variable, exported or not, protects its value,
    // and so does a configured provider name that does not look like one.
    let raw = "# Snapshot file\n\0\0#values\0LOCAL_TOKEN\0local-secret-2314\0CORP_LLM_CRED\0provider-secret-2314\0LOCAL_ALIAS\0x:local-secret-2314\0PROVIDER_ALIAS\0provider-secret-2314\0SAFE_EXPORT\0ordinary\0\0CORP_LLM_CRED\0export CORP_LLM_CRED=provider-secret-2314\n\0LOCAL_ALIAS\0export LOCAL_ALIAS=x:local-secret-2314\n\0PROVIDER_ALIAS\0export PROVIDER_ALIAS=provider-secret-2314\n\0SAFE_EXPORT\0export SAFE_EXPORT=ordinary\n\0\0";
    let snapshot = filter_with_gate(raw, &["CORP_LLM_CRED".to_string()], &OutputGate::new())?;
    assert_eq!(
        snapshot,
        "# Snapshot file\n# exports (native declarations)\nexport SAFE_EXPORT=ordinary\n"
    );
    Ok(())
}

#[test]
fn snapshot_refuses_credentials_left_in_functions_or_aliases() {
    let raw = "# Snapshot file\ngh () { GH_TOKEN=exported-secret-2314 command gh \"$@\"; }\n\0alias ghx='GH_TOKEN=exported-secret-2314 gh'\n\0#values\0GH_TOKEN\0exported-secret-2314\0SAFE_EXPORT\0ordinary\0\0SAFE_EXPORT\0export SAFE_EXPORT=ordinary\n\0\0";
    let error = filter_with_gate(raw, &[], &OutputGate::new()).unwrap_err();
    assert_eq!(
        error.to_string(),
        "Snapshot contains protected credential values"
    );
}
