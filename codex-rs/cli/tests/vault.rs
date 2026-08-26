use std::fs;
use std::path::Path;

use anyhow::Result;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

const LABEL_CANARY: &str = "provider.raw-export-label-canary";

fn write_security_config(codex_home: &Path, level: &str) -> Result<()> {
    fs::write(
        codex_home.join("config.toml"),
        format!("[security]\nversion = 1\nlevel = \"{level}\"\n"),
    )?;
    Ok(())
}

fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut command = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("codex")?);
    command.env("CODEX_HOME", codex_home);
    Ok(command)
}

#[test]
fn vault_auth_helper_denies_raw_export_in_protected_levels_without_label_disclosure() -> Result<()>
{
    for level in ["moderate", "aggressive"] {
        let codex_home = TempDir::new()?;
        write_security_config(codex_home.path(), level)?;

        let mut command = codex_command(codex_home.path())?;
        command
            .args(["vault", "auth-helper", LABEL_CANARY])
            .assert()
            .failure()
            .stdout("")
            .stderr(
                contains(format!("vault auth-helper is unavailable under {level}"))
                    .and(contains(LABEL_CANARY).not()),
            );
    }
    Ok(())
}

#[test]
fn vault_auth_helper_preserves_permissive_compatibility_path() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_security_config(codex_home.path(), "permissive")?;

    let mut command = codex_command(codex_home.path())?;
    command
        .args(["vault", "auth-helper", LABEL_CANARY])
        .assert()
        .failure()
        .stdout("")
        .stderr(contains("vault auth-helper is unavailable").not());

    Ok(())
}

#[test]
fn vault_auth_helper_cli_override_cannot_downgrade_persisted_posture() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_security_config(codex_home.path(), "moderate")?;

    let mut command = codex_command(codex_home.path())?;
    command
        .args([
            "-c",
            "security.level=\"permissive\"",
            "vault",
            "auth-helper",
            LABEL_CANARY,
        ])
        .assert()
        .failure()
        .stdout("")
        .stderr(
            contains("vault auth-helper is unavailable under moderate")
                .and(contains(LABEL_CANARY).not()),
        );

    Ok(())
}
