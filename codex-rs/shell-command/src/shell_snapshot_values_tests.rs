//! Corbanu #2314: synthetic credentials only.
use super::super::CapturedSnapshot;
use super::super::SnapshotCaptureOptions;
use super::super::SnapshotStartup;
use super::super::snapshot_capture_script;
use super::super::snapshot_source_capture_script;
use crate::shell_detect::ShellType;
use anyhow::Result;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::process::Command;
use tempfile::tempdir;

const SHELLS: [(&str, ShellType); 3] = [
    ("/bin/bash", ShellType::Bash),
    ("/bin/sh", ShellType::Sh),
    ("/bin/zsh", ShellType::Zsh),
];

const STARTUP: &str = r#"
export GH_TOKEN='values-exported-Canary-2314'
LOCAL_TOKEN='values-local-Canary-2314'
export MULTILINE_ALIAS="first
$GH_TOKEN
last"
EMPTY_LOCAL=
"#;

fn options(environment: bool) -> SnapshotCaptureOptions {
    SnapshotCaptureOptions {
        startup: SnapshotStartup::Interactive,
        declarations: true,
        environment,
    }
}

fn run(shell: &str, script: &str, home: &std::path::Path) -> Result<std::process::Output> {
    Ok(Command::new(shell)
        .args(["-c", script])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", home)
        .env("ZDOTDIR", home)
        .env("ENV", home.join("env.sh"))
        .output()?)
}

fn write_startup(home: &std::path::Path, startup: &str) -> Result<()> {
    for file in [".bashrc", ".zshrc", "env.sh"] {
        std::fs::write(home.join(file), startup)?;
    }
    Ok(())
}

#[test]
fn snapshot_capture_records_raw_values_of_every_shell_variable() -> Result<()> {
    let home = tempdir()?;
    // nounset in startup must not abort the value section on unset parameters.
    write_startup(home.path(), &format!("{STARTUP}set -u\n"))?;
    for (shell, shell_type) in SHELLS {
        if !std::path::Path::new(shell).exists() {
            continue;
        }
        // Both the plain and the broker capture carry the value section.
        for environment in [false, true] {
            let script = snapshot_capture_script(shell_type, options(environment)).unwrap();
            let output = run(shell, &script, home.path())?;
            assert!(output.status.success(), "{shell}: {output:?}");
            let captured = CapturedSnapshot::parse(shell_type, &output.stdout).unwrap();
            let values: HashMap<_, _> = captured
                .values
                .as_ref()
                .expect("value section")
                .iter()
                .map(|(name, value)| (*name, value.as_ref()))
                .collect();
            assert_eq!(values.get("GH_TOKEN"), Some(&"values-exported-Canary-2314"));
            // Unexported variables are captured too: an alias built from one
            // would otherwise go unnoticed.
            assert_eq!(values.get("LOCAL_TOKEN"), Some(&"values-local-Canary-2314"));
            assert_eq!(values.get("EMPTY_LOCAL"), Some(&""), "{shell}");
            assert_eq!(
                values.get("MULTILINE_ALIAS"),
                Some(&"first\nvalues-exported-Canary-2314\nlast"),
                "{shell}"
            );
            assert!(
                !values
                    .keys()
                    .any(|name| name.starts_with("__codex_snapshot_")),
                "{shell}"
            );
            // The section must not shift the export records that follow it.
            assert!(
                captured
                    .exports
                    .iter()
                    .any(|export| export.key == "MULTILINE_ALIAS"),
                "{shell}"
            );
            assert!(
                !captured
                    .exports
                    .iter()
                    .any(|export| export.key.starts_with("__codex_snapshot_")),
                "{shell}"
            );
        }
    }
    Ok(())
}

#[test]
fn zsh_value_with_nul_fails_capture() -> Result<()> {
    if !std::path::Path::new("/bin/zsh").exists() {
        return Ok(());
    }
    let home = tempdir()?;
    // NULs could forge value records; the shell must refuse, not the parser.
    write_startup(
        home.path(),
        &format!("{STARTUP}BINARY_VALUE=$'x\\0GH_TOKEN\\0ordinary\\0'\n"),
    )?;
    let script = snapshot_capture_script(ShellType::Zsh, options(false)).unwrap();
    let output = run("/bin/zsh", &script, home.path())?;
    assert!(!output.status.success(), "NUL-bearing capture must fail");
    Ok(())
}

#[test]
fn source_capture_has_no_value_section() -> Result<()> {
    let home = tempdir()?;
    write_startup(home.path(), STARTUP)?;
    for (shell, shell_type) in SHELLS {
        if !std::path::Path::new(shell).exists() {
            continue;
        }
        let script = snapshot_source_capture_script(shell_type, options(true)).unwrap();
        assert!(!script.contains("#values"), "{shell}");
        let output = run(shell, &script, home.path())?;
        assert!(output.status.success(), "{shell}: {output:?}");
        if shell_type == ShellType::Sh {
            let captured = CapturedSnapshot::parse(shell_type, &output.stdout).unwrap();
            assert!(captured.values.is_none());
        }
    }
    Ok(())
}

#[test]
fn malformed_value_section_rejects_capture() {
    for captured in [
        // Unterminated section.
        "# Snapshot file\n\0\0#values\0GH_TOKEN\0value\0",
        // A record name that is not an identifier.
        "# Snapshot file\n\0\0#values\0BAD-NAME\0value\0\0\0",
        // A name without its value.
        "# Snapshot file\n\0\0#values\0GH_TOKEN",
    ] {
        assert!(
            CapturedSnapshot::parse(ShellType::Bash, captured.as_bytes()).is_none(),
            "{captured:?}"
        );
    }
    let captured = CapturedSnapshot::parse(
        ShellType::Bash,
        b"# Snapshot file\n\0\0#values\0GH_TOKEN\0a\nb\0EMPTY\0\0\0SAFE\0declare -x SAFE=\"1\"\n\0\0",
    )
    .unwrap();
    let values = captured.values.unwrap();
    assert_eq!(
        values
            .iter()
            .map(|(name, value)| (*name, value.as_ref()))
            .collect::<Vec<_>>(),
        vec![("GH_TOKEN", "a\nb"), ("EMPTY", "")]
    );
    assert_eq!(captured.exports.len(), 1);
}
