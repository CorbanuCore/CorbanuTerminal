//! The standalone `codex-exec` binary runs the same nested-launch check as
//! `corbanu exec`: an agent command under Aggressive cannot start it to get
//! around that check. The origin home is made read-only with an unreadable
//! vault store, as the Aggressive profile makes it for agent commands.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const ORIGIN_ENV: &str = "CORBANU_SECURITY_ORIGIN";

/// An Aggressive origin home as an agent command sees it. `None` when this
/// user can write a read-only directory (root).
struct Origin(TempDir);

impl Origin {
    fn new(nested: &str) -> anyhow::Result<Option<Self>> {
        let home = TempDir::new()?;
        fs::write(
            home.path().join("security_level.toml"),
            format!("version = 1\nlevel = \"aggressive\"\nnested_agents = \"{nested}\"\n"),
        )?;
        fs::create_dir_all(home.path().join("secrets"))?;
        fs::set_permissions(
            home.path().join("secrets"),
            fs::Permissions::from_mode(0o000),
        )?;
        fs::set_permissions(home.path(), fs::Permissions::from_mode(0o555))?;
        let origin = Self(home);
        if fs::write(origin.0.path().join("probe"), "").is_ok() {
            return Ok(None);
        }
        Ok(Some(origin))
    }
}

impl Drop for Origin {
    fn drop(&mut self) {
        let _ = fs::set_permissions(self.0.path(), fs::Permissions::from_mode(0o755));
        let _ = fs::set_permissions(
            self.0.path().join("secrets"),
            fs::Permissions::from_mode(0o755),
        );
    }
}

fn codex_exec(home: &Path, origin: Option<&Path>) -> anyhow::Result<std::process::Output> {
    let mut command = Command::new(codex_utils_cargo_bin::cargo_bin("codex-exec")?);
    for name in ["CORBANU_HOME", "PFTERMINAL_HOME", ORIGIN_ENV] {
        command.env_remove(name);
    }
    if let Some(origin) = origin {
        command.env(ORIGIN_ENV, origin);
    }
    Ok(command
        .args(["--skip-git-repo-check", "hi"])
        .current_dir(home)
        .env("CODEX_HOME", home)
        .env("CODEX_SQLITE_HOME", home)
        .env("CORBANU_TEST_ACCOUNT_HOME", home.join("no-account"))
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        .env_remove("OPENAI_API_KEY")
        .env_remove("CODEX_API_KEY")
        .output()?)
}

#[test]
fn standalone_exec_refuses_nested_launches_in_both_modes() -> anyhow::Result<()> {
    for (nested, expected) in [
        ("refuse", "Nested agent launches are set to refuse"),
        (
            "pass",
            "Only `corbanu exec` and `corbanu review` can run there",
        ),
    ] {
        let Some(origin) = Origin::new(nested)? else {
            return Ok(());
        };
        let home = TempDir::new()?;
        let output = codex_exec(home.path(), Some(origin.0.path()))?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{nested}: {stderr}");
        assert!(output.stdout.is_empty(), "{nested}");
        assert!(
            stderr.contains(
                "`codex-exec` was started by an agent command while security level Aggressive"
            ) && stderr.contains(expected),
            "{nested}: {stderr}"
        );
    }
    Ok(())
}

#[test]
fn standalone_exec_without_an_origin_is_not_refused() -> anyhow::Result<()> {
    let home = TempDir::new()?;
    let output = codex_exec(home.path(), /*origin*/ None)?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("was started by an agent command"),
        "{stderr}"
    );
    Ok(())
}
