//! Issue #2314: exercise real startup files with synthetic credentials only.
use super::*;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

#[test_case::test_case(ShellType::Bash, "/bin/bash"; "bash")]
#[test_case::test_case(ShellType::Sh, "/bin/sh"; "sh")]
#[test_case::test_case(ShellType::Zsh, "/bin/zsh"; "zsh")]
fn snapshots_drop_startup_credential_aliases_before_persistence(
    kind: ShellType,
    executable: &str,
) -> Result<()> {
    let home = tempdir()?;
    let removed = vec!["CORP_LLM_CRED".to_string()];
    // Include an unexported credential: examining only the child environment
    // misses the exported alias made from it.
    let startup = r#"
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
"#;
    std::fs::write(home.path().join(".bashrc"), startup)?;
    std::fs::write(home.path().join(".zshrc"), startup)?;
    std::fs::write(home.path().join("env.sh"), startup)?;

    if !Path::new(executable).exists() {
        return Ok(());
    }
    {
        let script = match kind {
            ShellType::Bash => bash_snapshot_script(&removed),
            ShellType::Sh => sh_snapshot_script(&removed),
            ShellType::Zsh => zsh_snapshot_script(&removed),
            _ => unreachable!(),
        };
        let script = filter::capture_script(script, kind);
        let output = std::process::Command::new(executable)
            .args(["-c", &script])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", home.path())
            .env("ZDOTDIR", home.path())
            .env("ENV", home.path().join("env.sh"))
            .output()?;
        assert!(output.status.success(), "capture failed for {kind:?}");
        let raw = String::from_utf8(output.stdout)?;
        let snapshot = filter::filter_snapshot(&raw, &removed)?;
        let path = home.path().join("snapshot.sh");
        std::fs::write(&path, &snapshot)?;
        let persisted = std::fs::read_to_string(&path)?;
        for forbidden in [
            "snapshot-exported-Canary-2314",
            "snapshot-local-Canary-2314",
            "snapshot-provider-Canary-2314",
            "EXACT_ALIAS",
            "CONCAT_ALIAS",
            "LOCAL_ALIAS",
            "PROVIDER_ALIAS",
            "MULTILINE_ALIAS",
            "SAFE_LOCAL",
        ] {
            assert!(
                !persisted.contains(forbidden),
                "{kind:?} persisted {forbidden}"
            );
        }
        let replay = std::process::Command::new(executable)
            .args(["-c", ". \"$1\"; printf '%s' \"$SAFE_EXPORT\"", "snapshot"])
            .arg(&path)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()?;
        assert!(replay.status.success(), "replay failed for {kind:?}");
        assert_eq!(String::from_utf8(replay.stdout)?, "ordinary value");
    }
    Ok(())
}
