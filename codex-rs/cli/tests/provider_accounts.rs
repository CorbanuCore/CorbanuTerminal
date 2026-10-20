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
        .args([
            "internal-claude-oauth-token",
            "--account",
            "faketok",
            "--disable",
            "named_accounts",
        ])
        .assert()
        .failure()
        .stdout("")
        .stderr(contains("named accounts are off").and(contains(TOKEN_CANARY).not()));
    corbanu(home.path())?
        .args(["internal-claude-oauth-token", "--disable", "named_accounts"])
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

/// PF-84-S06 slice A: an Amazon Bedrock account holds an AWS profile name,
/// validated and shown in clear; it can be renamed and removed. Other
/// providers refuse the kind.
#[test]
fn bedrock_accounts_hold_a_validated_aws_profile_name() -> Result<()> {
    let home = TempDir::new()?;
    let account = |args: &[&str]| -> Result<assert_cmd::assert::Assert> {
        Ok(corbanu(home.path())?
            .arg("account")
            .args(args)
            .args(["--enable", "named_accounts"])
            .assert())
    };
    let add = ["add", "amazon-bedrock", "work", "--kind", "aws-profile"];
    account(&[&add[..], &["--value", "bad profile"]].concat())?
        .failure()
        .stderr(contains("invalid AWS profile name"));
    account(&add)?
        .failure()
        .stderr(contains("this kind needs --value"));
    account(&[
        "add",
        "zai",
        "work",
        "--kind",
        "aws-profile",
        "--value",
        "dev",
    ])?
    .failure()
    .stderr(contains(
        "provider `zai` does not take `--kind aws-profile` accounts",
    ));
    account(&[&add[..], &["--value", "corp-dev"]].concat())?
        .success()
        .stdout("Saved amazon-bedrock account `work` (aws_profile).\n");
    account(&["list", "amazon-bedrock"])?
        .success()
        .stdout("amazon-bedrock\twork\taws_profile\tprofile corp-dev\n");
    account(&["rename", "amazon-bedrock", "work", "prod"])?
        .success()
        .stdout("Renamed amazon-bedrock account `work` to `prod`.\n");
    account(&["list"])?
        .success()
        .stdout("amazon-bedrock\tprod\taws_profile\tprofile corp-dev\n");
    account(&["remove", "amazon-bedrock", "prod"])?.success();
    account(&["list"])?
        .success()
        .stdout("No named accounts. Every provider uses its default account.\n");
    Ok(())
}

/// PF-84-S06 slice B: `corbanu login --account` signs a named OpenAI account
/// in (here with an API key) without touching the default login; list,
/// status and logout see only that account and never print the key.
#[test]
fn named_openai_account_login_list_status_logout() -> Result<()> {
    let home = TempDir::new()?;
    let canary = "sk-pf84-s06-cli-canary-0001";
    corbanu(home.path())?
        .args([
            "login",
            "--account",
            "work",
            "--with-api-key",
            "--disable",
            "named_accounts",
        ])
        .write_stdin(canary)
        .assert()
        .failure()
        .stderr(contains("named accounts are off"));
    corbanu(home.path())?
        .args([
            "login",
            "--account",
            "work",
            "--with-api-key",
            "--enable",
            "named_accounts",
        ])
        .write_stdin(canary)
        .assert()
        .success()
        .stderr(
            contains("Signed in OpenAI account `work` (OpenAI API key).")
                .and(contains(canary).not()),
        );
    assert!(
        !home.path().join("auth.json").exists(),
        "the default login is untouched"
    );
    corbanu(home.path())?
        .args(["account", "list", "openai", "--enable", "named_accounts"])
        .assert()
        .success()
        .stdout("openai\twork\tchatgpt_auth\tOpenAI API key\n");
    corbanu(home.path())?
        .args([
            "login",
            "--account",
            "work",
            "status",
            "--enable",
            "named_accounts",
        ])
        .assert()
        .success()
        .stderr(contains("OpenAI account `work`: OpenAI API key").and(contains(canary).not()));
    corbanu(home.path())?
        .args(["logout", "--account", "work", "--enable", "named_accounts"])
        .assert()
        .success()
        .stderr(contains("Signed out OpenAI account `work`"));
    corbanu(home.path())?
        .args([
            "login",
            "--account",
            "work",
            "status",
            "--enable",
            "named_accounts",
        ])
        .assert()
        .failure()
        .stderr(contains("is not signed in"));
    Ok(())
}

/// #573: a named OpenAI account can sign in with ChatGPT (device code) when
/// the configured session provider is not OpenAI; only a configured
/// `forced_login_method = "api"` refuses it, naming the API-key route for
/// that account. The issuer is a closed local port, so the attempt itself
/// fails after the method check.
#[test]
fn named_chatgpt_sign_in_is_not_refused_for_a_third_party_session_provider() -> Result<()> {
    let home = TempDir::new()?;
    std::fs::write(
        home.path().join("config.toml"),
        "model_provider = \"zai\"\nmodel = \"glm-5.3-flash\"\n[features]\nnamed_accounts = true\n",
    )?;
    let device_login = |extra: &[&str]| -> Result<assert_cmd::assert::Assert> {
        let mut command = corbanu(home.path())?;
        command
            .args(extra)
            .args([
                "login",
                "--account",
                "work",
                "--device-auth",
                "--experimental_issuer",
                "http://127.0.0.1:9",
            ])
            .timeout(std::time::Duration::from_secs(120));
        Ok(command.assert())
    };
    device_login(&[])?.failure().stderr(
        contains("Error signing in account `work`")
            .and(contains("disabled").not())
            .and(contains("Use API key login instead").not()),
    );
    device_login(&["-c", "forced_login_method=\"api\""])?
        .failure()
        .stderr(contains(
            "ChatGPT sign-in is disabled by `forced_login_method = \"api\"`; sign account `work` \
             in with an OpenAI API key: `corbanu login --account work --with-api-key`.",
        ));
    assert!(!home.path().join("auth.json").exists());
    Ok(())
}
