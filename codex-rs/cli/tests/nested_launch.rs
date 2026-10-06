//! Agent launches started by agent commands under Aggressive (nested
//! launches). The marker is set here as an Aggressive session sets it in
//! every agent command's environment.
#![cfg(not(target_os = "windows"))]

use std::fs;
use std::path::Path;

use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_sequence_unchecked;
use app_test_support::create_shell_command_sse_response;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

const ORIGIN_ENV: &str = "CORBANU_SECURITY_ORIGIN";
const VAULT_CANARY: &str = "nested-vault-canary-7f3a";

fn write_level(home: &Path, nested: Option<&str>) -> Result<()> {
    let nested = nested
        .map(|value| format!("nested_agents = \"{value}\"\n"))
        .unwrap_or_default();
    fs::write(
        home.join("security_level.toml"),
        format!("version = 1\nlevel = \"aggressive\"\n{nested}"),
    )?;
    Ok(())
}

fn corbanu(home: &Path, cwd: &Path) -> Result<assert_cmd::Command> {
    let mut command = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("codex")?);
    for name in ["CORBANU_HOME", "PFTERMINAL_HOME", ORIGIN_ENV] {
        command.env_remove(name);
    }
    command
        .current_dir(cwd)
        .env("CODEX_HOME", home)
        .env("CODEX_SQLITE_HOME", home)
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1");
    Ok(command)
}

#[test]
fn refuse_mode_refuses_every_agent_launch_and_credential_helper() -> Result<()> {
    let home = TempDir::new()?;
    let cwd = TempDir::new()?;
    write_level(home.path(), /*nested*/ None)?;
    for args in [
        vec!["exec", "hi"],
        vec!["review", "--uncommitted"],
        vec!["resume", "--last"],
        vec!["fork", "--last"],
        vec!["hi"],
        vec!["app-server"],
        vec!["mcp-server"],
        vec!["vault", "auth-helper", "provider/zai_api_key"],
        vec!["internal-claude-oauth-token"],
    ] {
        corbanu(home.path(), cwd.path())?
            .env(ORIGIN_ENV, home.path())
            .args(&args)
            .assert()
            .failure()
            .stdout("")
            .stderr(contains(
                "was started by an agent command while security level Aggressive",
            ));
    }
    // Commands that start no agent are unaffected.
    corbanu(home.path(), cwd.path())?
        .env(ORIGIN_ENV, home.path())
        .args(["features", "list"])
        .assert()
        .success();
    Ok(())
}

#[test]
fn pass_mode_still_refuses_hosts_and_credentials() -> Result<()> {
    let home = TempDir::new()?;
    let cwd = TempDir::new()?;
    write_level(home.path(), Some("pass"))?;
    for (args, reason) in [
        (vec!["app-server"], "never allowed"),
        (vec!["mcp-server"], "never allowed"),
        (
            vec!["vault", "auth-helper", "provider/zai_api_key"],
            "credentials",
        ),
    ] {
        corbanu(home.path(), cwd.path())?
            .env(ORIGIN_ENV, home.path())
            .args(&args)
            .assert()
            .failure()
            .stderr(contains(reason));
    }
    Ok(())
}

/// With pass, a nested `corbanu exec` runs with Aggressive: its weakening
/// flags are ignored, the vault store stays unreadable to its commands, and
/// commands that need approval (network, writes outside the folder) never
/// run because nobody can approve them.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pass_mode_runs_exec_with_aggressive_enforced() -> Result<()> {
    let home = TempDir::new()?;
    let root = TempDir::new()?;
    let cwd = root.path().join("workspace");
    let outside = root.path().join("outside");
    fs::create_dir_all(&cwd)?;
    fs::create_dir_all(&outside)?;
    fs::create_dir_all(home.path().join("secrets"))?;
    fs::write(home.path().join("secrets").join("probe.txt"), VAULT_CANARY)?;
    write_level(home.path(), Some("pass"))?;

    let outside_file = outside.join("written.txt");
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_shell_command_sse_response(
            vec![
                "cat".to_string(),
                home.path()
                    .join("secrets")
                    .join("probe.txt")
                    .display()
                    .to_string(),
            ],
            /*workdir*/ None,
            Some(10_000),
            "vault",
        )?,
        create_shell_command_sse_response(
            vec![
                "bash".to_string(),
                "-c".to_string(),
                format!("echo hi > {}", outside_file.display()),
            ],
            /*workdir*/ None,
            Some(10_000),
            "outside",
        )?,
        create_shell_command_sse_response(
            vec![
                "curl".to_string(),
                "-sS".to_string(),
                "https://example.com".to_string(),
            ],
            /*workdir*/ None,
            Some(10_000),
            "network",
        )?,
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    MockResponsesConfig::new(&server.uri())
        .with_extra_config(&format!(
            "[projects.\"{}\"]\ntrust_level = \"trusted\"\n",
            cwd.display()
        ))
        .write(home.path())?;

    let output = corbanu(home.path(), &cwd)?
        .env(ORIGIN_ENV, home.path())
        .args([
            "exec",
            "--skip-git-repo-check",
            "--sandbox",
            "danger-full-access",
            "probe",
        ])
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Security level Aggressive is enforced because an agent command")
            && stderr.contains("Ignored --sandbox.")
            && stderr.contains("approval: untrusted"),
        "{stderr}"
    );

    let requests = server.received_requests().await.unwrap_or_default();
    let bodies = requests
        .iter()
        .map(|request| String::from_utf8_lossy(&request.body).into_owned())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!bodies.is_empty(), "the nested run never reached the model");
    assert!(
        !bodies.contains(VAULT_CANARY),
        "vault value reached the model"
    );
    assert!(!outside_file.exists(), "a write outside the folder ran");
    assert!(!stderr.contains("Example Domain"), "{stderr}");
    // The vault read ran inside the sandbox and failed (Seatbelt's message;
    // Linux runners may lack a working sandbox); the other two needed an
    // approval nobody can give.
    if cfg!(target_os = "macos") {
        assert!(stderr.contains("Operation not permitted"), "{stderr}");
    }
    assert_eq!(stderr.matches("declined in").count(), 2, "{stderr}");
    Ok(())
}

/// Without the marker and with a writable home, a person's own launch is not
/// nested, whatever the stored setting.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn own_launch_with_aggressive_stored_is_not_refused() -> Result<()> {
    let home = TempDir::new()?;
    let cwd = TempDir::new()?;
    write_level(home.path(), /*nested*/ None)?;
    let server = create_mock_responses_server_sequence_unchecked(vec![
        create_final_assistant_message_sse_response("done")?,
    ])
    .await;
    MockResponsesConfig::new(&server.uri()).write(home.path())?;
    corbanu(home.path(), cwd.path())?
        .args(["exec", "--skip-git-repo-check", "hi"])
        .assert()
        .success()
        .stderr(predicates::str::contains("was started by an agent command").not());
    Ok(())
}
