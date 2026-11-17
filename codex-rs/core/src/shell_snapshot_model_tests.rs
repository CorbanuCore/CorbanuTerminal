use super::*;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_shell_command::shell_detect::ShellType;
use codex_shell_command::shell_snapshot::{
    SnapshotCaptureOptions, SnapshotStartup, snapshot_capture_script,
};
use pretty_assertions::assert_eq;

fn filtered(raw: &str, scan: &ModelScan) -> Result<String> {
    let mut capture =
        CapturedSnapshot::parse(ShellType::Bash, raw.as_bytes()).context("capture")?;
    let gate = OutputGate::new();
    let filter = retain_with_holders(
        &mut capture,
        raw.len(),
        &[],
        &gate,
        /*brokered*/ None,
        scan,
    )?;
    let snapshot = capture.render_script(&ShellEnvironmentPolicy::default());
    filter.ensure_clean(&snapshot)?;
    Ok(snapshot)
}

#[cfg(unix)]
#[test]
fn d690_startup_copy_then_unset_is_removed_from_snapshot() -> Result<()> {
    let home = tempfile::tempdir()?;
    std::fs::write(
        home.path().join(".bashrc"),
        "export OPENAI_API_KEY='d690-plain-model-value'\nexport HIDDEN_HEADER=\"prefix${OPENAI_API_KEY}suffix\"\nunset OPENAI_API_KEY\nexport SAFE_EXPORT=ordinary\n",
    )?;
    let script = snapshot_capture_script(
        ShellType::Bash,
        SnapshotCaptureOptions {
            startup: SnapshotStartup::Interactive,
            declarations: true,
            environment: false,
        },
    )
    .context("script")?;
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
    let safe = filtered(&raw, &|text| Ok(text.contains("d690-plain-model-value")))?;
    assert!(!safe.contains("HIDDEN_HEADER"));
    assert!(!safe.contains("d690-plain-model-value"));
    assert!(safe.contains("SAFE_EXPORT"));
    let unprotected = filtered(&raw, &|_| Ok(false))?;
    assert!(unprotected.contains("prefixd690-plain-model-valuesuffix"));
    assert!(filtered(&raw, &|_| Err(codex_network_proxy::ModelSnapshotScanError)).is_err());
    Ok(())
}

#[test]
fn d690_model_key_in_function_refuses_snapshot() {
    let raw = "# Snapshot file\nf() { echo d690-model-value; }\n\0\0#values\0SAFE\0ordinary\0#end\x000\0\0";
    let result = filtered(raw, &|text| Ok(text.contains("d690-model-value")));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Snapshot contains protected credential values"
    );
}
