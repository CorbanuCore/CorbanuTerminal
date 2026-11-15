use super::*;
use pretty_assertions::assert_eq;

fn capture(value: &str) -> String {
    format!(
        "\0value\0SAFE_EXPORT\0ordinary\0# Snapshot file\n# exports\n\0export\0HIDDEN_HEADER\0{value}\0export HIDDEN_HEADER='serialized';\n\0\0export\0SAFE_EXPORT\0ordinary\0export SAFE_EXPORT='ordinary';\n\0"
    )
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
            "# Snapshot file\n# exports\nexport SAFE_EXPORT='ordinary';\n"
        );
    }
    Ok(())
}

#[test]
fn snapshot_drops_short_credentials_inside_concatenations() -> Result<()> {
    let gate = OutputGate::new();
    gate.register("vault:short-fixture", SecretClass::Operational, "s3c")?;
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
        "# Snapshot file\n# exports\nexport HIDDEN_HEADER='serialized';\nexport SAFE_EXPORT='ordinary';\n"
    );
    Ok(())
}

#[test]
fn snapshot_rejects_incomplete_or_unframed_capture() {
    for raw in [
        "# Snapshot file\nexport HIDDEN_HEADER=raw\n",
        "\0value\0GH_TOKEN",
        "\0value\0GH_TOKEN\0a-secret-value\0\0export\0HEADER\0raw",
    ] {
        assert!(filter_with_gate(raw, &[], &OutputGate::new()).is_err());
    }
}
