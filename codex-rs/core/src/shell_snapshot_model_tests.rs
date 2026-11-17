use super::*;
use pretty_assertions::assert_eq;

#[cfg(unix)]
#[test]
fn d690_startup_copy_then_unset_is_removed_from_snapshot() -> Result<()> {
    let home = tempfile::tempdir()?;
    std::fs::write(
        home.path().join(".bashrc"),
        "export OPENAI_API_KEY='d690-plain-model-value'\nexport HIDDEN_HEADER=\"prefix${OPENAI_API_KEY}suffix\"\nunset OPENAI_API_KEY\nexport SAFE_EXPORT=ordinary\n",
    )?;
    let script = capture_script(super::super::bash_snapshot_script(&[]), ShellType::Bash);
    let output = std::process::Command::new("/bin/bash")
        .args(["--noprofile", "--norc", "-c", &script])
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .output()?;
    assert!(output.status.success());
    let raw = String::from_utf8(output.stdout)?;
    assert!(raw.contains("prefixd690-plain-model-valuesuffix"));
    assert!(!raw.contains("\0OPENAI_API_KEY\0"));
    let gate = OutputGate::new();
    let scan = |text: &str| Ok(text.contains("d690-plain-model-value"));
    let safe = filter_with_holders(&raw, &[], &gate, &scan)?;
    assert!(!safe.contains("HIDDEN_HEADER"));
    assert!(!safe.contains("d690-plain-model-value"));
    assert!(safe.contains("SAFE_EXPORT"));
    // Negative control: the same capture preserves copies unknown to holders.
    let unprotected = filter_with_holders(&raw, &[], &gate, &|_| Ok(false))?;
    assert!(unprotected.contains("prefixd690-plain-model-valuesuffix"));
    // Scoped/isolated routes with no inspection API must refuse even safe text.
    assert!(
        filter_with_holders(&raw, &[], &gate, &|_| Err(
            codex_network_proxy::ModelSnapshotScanError
        ))
        .is_err()
    );
    Ok(())
}

#[test]
fn d690_model_key_in_function_refuses_snapshot() {
    let raw = "\0value\0SAFE\0ordinary\0# Snapshot file\nf() { echo d690-model-value; }\n";
    let result = filter_with_holders(raw, &[], &OutputGate::new(), &|text| {
        Ok(text.contains("d690-model-value"))
    });
    assert_eq!(
        result.unwrap_err().to_string(),
        "Snapshot contains protected credential values"
    );
}
