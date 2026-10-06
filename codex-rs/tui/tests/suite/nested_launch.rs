//! The standalone `codex-tui` binary runs the same nested-launch check as
//! `corbanu`: an agent command under Aggressive cannot start an interactive
//! session through it. The origin home is made read-only with an unreadable
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

fn codex_tui(home: &Path, origin: &Path) -> anyhow::Result<std::process::Output> {
    let mut command = Command::new(codex_utils_cargo_bin::cargo_bin("codex-tui")?);
    for name in ["CORBANU_HOME", "PFTERMINAL_HOME", ORIGIN_ENV] {
        command.env_remove(name);
    }
    Ok(command
        .arg("hi")
        .env(ORIGIN_ENV, origin)
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
fn standalone_tui_refuses_nested_sessions_in_both_modes() -> anyhow::Result<()> {
    for nested in ["refuse", "pass"] {
        let Some(origin) = Origin::new(nested)? else {
            return Ok(());
        };
        let home = TempDir::new()?;
        let output = codex_tui(home.path(), origin.0.path())?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{nested}: {stderr}");
        assert!(
            stderr.contains(
                "`codex-tui` was started by an agent command while security level Aggressive"
            ) && stderr.contains("Interactive sessions are never allowed there"),
            "{nested}: {stderr}"
        );
    }
    Ok(())
}
