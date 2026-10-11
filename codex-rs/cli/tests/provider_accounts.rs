//! PF-84 named-account regressions from the independent acceptance run.

use std::path::Path;

use anyhow::Result;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

const TOKEN_CANARY: &str = "synthetic-claude-account-canary-0415";

fn corbanu(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut command = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("corbanu")?);
    command
        .env_remove("CORBANU_HOME")
        .env_remove("PFTERMINAL_HOME")
        .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
        .env_remove("CORBANU_PROVIDER_ACCOUNT")
        .env("CODEX_HOME", codex_home)
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1");
    Ok(command)
}

fn home_with_claude_account() -> Result<TempDir> {
    let home = TempDir::new()?;
    corbanu(home.path())?
        .args([
            "account",
            "add",
            "claude-plan",
            "faketok",
            "--kind",
            "claude-token",
            "--enable",
            "named_accounts",
        ])
        .write_stdin(TOKEN_CANARY)
        .assert()
        .success();
    Ok(home)
}

/// #415: with `named_accounts` off, the Claude token helper refuses a named
/// account (from `--account` or `CORBANU_PROVIDER_ACCOUNT`) and prints nothing.
#[test]
fn claude_token_helper_refuses_named_accounts_when_the_feature_is_off() -> Result<()> {
    let home = home_with_claude_account()?;
    corbanu(home.path())?
        .args(["internal-claude-oauth-token", "--account", "faketok"])
        .assert()
        .failure()
        .stdout("")
        .stderr(contains("named accounts are off").and(contains(TOKEN_CANARY).not()));
    corbanu(home.path())?
        .args(["internal-claude-oauth-token"])
        .env("CORBANU_PROVIDER_ACCOUNT", "faketok")
        .assert()
        .failure()
        .stdout("")
        .stderr(contains("named accounts are off"));
    // What a session with the feature on runs (#414 argv).
    corbanu(home.path())?
        .args([
            "internal-claude-oauth-token",
            "--account",
            "faketok",
            "--enable",
            "named_accounts",
        ])
        .assert()
        .success()
        .stdout(TOKEN_CANARY);
    Ok(())
}

/// #419: a Claude Code config dir must be an existing absolute directory, and
/// a kind mismatch reads correctly.
#[test]
fn account_add_validates_claude_config_dirs_and_names_the_kind() -> Result<()> {
    let home = TempDir::new()?;
    let config_dir = TempDir::new()?;
    let add_config_dir = |value: &str| -> Result<assert_cmd::assert::Assert> {
        Ok(corbanu(home.path())?
            .args([
                "account",
                "add",
                "claude-plan",
                "cfgacct",
                "--kind",
                "claude-config-dir",
                "--value",
                value,
                "--enable",
                "named_accounts",
            ])
            .assert())
    };
    add_config_dir("relative/dir")?.failure().stderr(contains(
        "absolute path of an existing Claude Code config directory",
    ));
    let missing = config_dir.path().join("missing");
    add_config_dir(missing.to_str().expect("utf-8 path"))?
        .failure()
        .stderr(contains(
            "absolute path of an existing Claude Code config directory",
        ));
    add_config_dir(config_dir.path().to_str().expect("utf-8 path"))?.success();

    corbanu(home.path())?
        .args([
            "account",
            "add",
            "zai",
            "work",
            "--kind",
            "claude-token",
            "--enable",
            "named_accounts",
        ])
        .write_stdin("unused")
        .assert()
        .failure()
        .stderr(contains(
            "provider `zai` does not take `--kind claude-token` accounts",
        ));
    Ok(())
}
