//! Issue #2314 (main #2332), ported for #2347: exercise real startup files
//! with synthetic credentials only, through both of upstream's render paths.
use super::*;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_shell_command::shell_detect::ShellType;
use codex_shell_command::shell_snapshot::PreparedSnapshot;
use codex_shell_command::shell_snapshot::SnapshotCaptureOptions;
use codex_shell_command::shell_snapshot::SnapshotCredentialEnvironment;
use codex_shell_command::shell_snapshot::SnapshotStartup;
use codex_shell_command::shell_snapshot::prepare_snapshot_credentials;
use codex_shell_command::shell_snapshot::snapshot_capture_script;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::Path;
use tempfile::tempdir;

// Include an unexported credential: examining only the child environment
// misses the exported alias made from it.
const STARTUP: &str = r#"
export GH_TOKEN='snapshot-exported-Canary-2314'
LOCAL_TOKEN='snapshot-local-Canary-2314'
CORP_LLM_CRED='snapshot-provider-Canary-2314'
export EXACT_ALIAS="$GH_TOKEN"
export CONCAT_ALIAS="prefix\"$GH_TOKEN\"suffix"
export LOCAL_ALIAS="prefix:${LOCAL_TOKEN}:suffix"
export PROVIDER_ALIAS="$CORP_LLM_CRED"
export MULTILINE_ALIAS="first
$GH_TOKEN
last"
export SAFE_EXPORT='ordinary value'
SAFE_LOCAL='not exported'
export BASH_EXECUTION_STRING="quoted\"$GH_TOKEN"
"#;

const FORBIDDEN: [&str; 9] = [
    "snapshot-exported-Canary-2314",
    "snapshot-local-Canary-2314",
    "snapshot-provider-Canary-2314",
    "EXACT_ALIAS",
    "CONCAT_ALIAS",
    "LOCAL_ALIAS",
    "PROVIDER_ALIAS",
    "MULTILINE_ALIAS",
    "SAFE_LOCAL",
];

#[derive(Clone, Copy, Debug)]
enum Render {
    /// No credential broker: `CapturedSnapshot::render_script`.
    Plain,
    /// Credential broker: `prepare_snapshot_credentials`.
    Brokered,
}

fn write_startup(home: &Path, startup: &str) -> Result<()> {
    for file in [".bashrc", ".zshrc", "env.sh"] {
        std::fs::write(home.join(file), startup)?;
    }
    Ok(())
}

fn capture_raw(kind: ShellType, executable: &str, home: &Path, render: Render) -> Result<String> {
    let script = snapshot_capture_script(
        kind,
        SnapshotCaptureOptions {
            startup: SnapshotStartup::Interactive,
            declarations: true,
            environment: matches!(render, Render::Brokered),
        },
    )
    .context("supported shell")?;
    let output = std::process::Command::new(executable)
        .args(["-c", &script])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", home)
        .env("ZDOTDIR", home)
        .env("ENV", home.join("env.sh"))
        .output()?;
    if !output.status.success() {
        bail!("capture failed for {kind:?}");
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// Mirrors `capture_snapshot`: filter right after parsing, then render and
/// check the final text on the selected path.
fn snapshot(kind: ShellType, raw: &str, removed: &[String], render: Render) -> Result<String> {
    let mut capture =
        CapturedSnapshot::parse(kind, raw.as_bytes()).context("invalid shell snapshot capture")?;
    let filter = retain_safe_exports(&mut capture, raw.len(), removed, /*brokered*/ None)?;
    let snapshot = match render {
        Render::Plain => capture.render_script(&ShellEnvironmentPolicy::default()),
        Render::Brokered => {
            let original = std::str::from_utf8(capture.environment)?
                .split('\0')
                .filter_map(|entry| entry.split_once('='))
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<HashMap<_, _>>();
            let empty = HashMap::new();
            let Some(PreparedSnapshot { script, .. }) = prepare_snapshot_credentials(
                &capture,
                SnapshotCredentialEnvironment {
                    original: &original,
                    restored: &original,
                    configured: &empty,
                    discovered: &original,
                    allowed: &original,
                    is_allowed_unset: &|_| true,
                    brokered_keys: &[],
                    brokered_alias_keys: &[],
                    allowed_brokered_keys: &[],
                },
                |_| true,
            ) else {
                bail!("brokered preparation refused");
            };
            script
        }
    };
    filter.ensure_clean(&snapshot)?;
    Ok(snapshot)
}

#[test_case::test_case(ShellType::Bash, "/bin/bash"; "bash")]
#[test_case::test_case(ShellType::Sh, "/bin/sh"; "sh")]
#[test_case::test_case(ShellType::Zsh, "/bin/zsh"; "zsh")]
fn snapshots_drop_startup_credential_aliases_before_persistence(
    kind: ShellType,
    executable: &str,
) -> Result<()> {
    if !Path::new(executable).exists() {
        return Ok(());
    }
    let home = tempdir()?;
    let removed = vec!["CORP_LLM_CRED".to_string()];
    write_startup(home.path(), STARTUP)?;
    for render in [Render::Plain, Render::Brokered] {
        let raw = capture_raw(kind, executable, home.path(), render)?;
        let snapshot = snapshot(kind, &raw, &removed, render)?;
        let path = home.path().join("snapshot.sh");
        std::fs::write(&path, &snapshot)?;
        let persisted = std::fs::read_to_string(&path)?;
        for forbidden in FORBIDDEN {
            assert!(
                !persisted.contains(forbidden),
                "{kind:?} {render:?} persisted {forbidden}"
            );
        }
        let replay = std::process::Command::new(executable)
            .args(["-c", ". \"$1\"; printf '%s' \"$SAFE_EXPORT\"", "snapshot"])
            .arg(&path)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()?;
        assert!(
            replay.status.success(),
            "replay failed for {kind:?} {render:?}"
        );
        assert_eq!(String::from_utf8(replay.stdout)?, "ordinary value");
    }

    if kind == ShellType::Zsh {
        // A zsh scalar can hold NULs, including apparently valid records.
        // Reject it in the shell rather than trusting the Rust parser to
        // distinguish data from protocol delimiters.
        write_startup(
            home.path(),
            &format!("{STARTUP}\nBINARY_VALUE=$'x\\0GH_TOKEN\\0ordinary\\0'\n"),
        )?;
        assert!(
            capture_raw(kind, executable, home.path(), Render::Plain).is_err(),
            "NUL-bearing capture must fail"
        );
    }
    Ok(())
}

#[test_case::test_case(ShellType::Bash, "/bin/bash"; "bash")]
#[test_case::test_case(ShellType::Sh, "/bin/sh"; "sh")]
#[test_case::test_case(ShellType::Zsh, "/bin/zsh"; "zsh")]
fn snapshots_refuse_startup_aliases_that_embed_credentials(
    kind: ShellType,
    executable: &str,
) -> Result<()> {
    if !Path::new(executable).exists() {
        return Ok(());
    }
    let home = tempdir()?;
    write_startup(
        home.path(),
        &format!("{STARTUP}alias ghx='GH_TOKEN=snapshot-exported-Canary-2314 gh'\n"),
    )?;
    for render in [Render::Plain, Render::Brokered] {
        let raw = capture_raw(kind, executable, home.path(), render)?;
        let result = snapshot(kind, &raw, &[], render);
        if kind == ShellType::Sh {
            // POSIX sh capture runs `unalias -a` and has no function printer,
            // so its snapshot carries neither; it must still omit the value.
            let snapshot = result?;
            assert!(!snapshot.contains("ghx"), "{render:?}: {snapshot}");
            assert!(!snapshot.contains("snapshot-exported-Canary-2314"));
            continue;
        }
        let Err(error) = result else {
            panic!("{kind:?} {render:?}: snapshot with a credential alias was written");
        };
        assert_eq!(
            error.to_string(),
            "Snapshot contains protected credential values",
            "{kind:?} {render:?}"
        );
    }
    Ok(())
}
