#![allow(clippy::unwrap_used)]
use codex_login::CODEX_API_KEY_ENV_VAR;
use codex_login::OPENAI_API_KEY_ENV_FALLBACK_NOTICE;
use codex_login::OPENAI_API_KEY_ENV_VAR;
use core_test_support::responses::ev_completed;
use core_test_support::responses::mount_sse_once_match;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex_exec::test_codex_exec;
use pretty_assertions::assert_eq;
use wiremock::matchers::header;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_uses_codex_api_key_env_var() -> anyhow::Result<()> {
    let test = test_codex_exec();
    let server = start_mock_server().await;
    let repo_root = codex_utils_cargo_bin::repo_root()?;

    mount_sse_once_match(
        &server,
        header("Authorization", "Bearer dummy"),
        sse(vec![ev_completed("request_0")]),
    )
    .await;

    test.cmd_with_server(&server)
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(&repo_root)
        .arg("echo testing codex api key")
        .assert()
        .success();

    Ok(())
}

/// Runs one exec turn against the mock server and returns (stdout, stderr).
async fn run_exec_with_env(
    env: &[(&str, &str)],
    expected_bearer: &str,
    json: bool,
) -> anyhow::Result<(String, String)> {
    let test = test_codex_exec();
    let server = start_mock_server().await;
    mount_sse_once_match(
        &server,
        header("Authorization", format!("Bearer {expected_bearer}")),
        sse(vec![ev_completed("request_0")]),
    )
    .await;

    let mut cmd = test.cmd_with_server(&server);
    cmd.env_remove(CODEX_API_KEY_ENV_VAR)
        .env_remove(OPENAI_API_KEY_ENV_VAR);
    for (key, value) in env {
        cmd.env(key, value);
    }
    if json {
        cmd.arg("--json");
    }
    let output = cmd
        .arg("--skip-git-repo-check")
        .arg("-C")
        .arg(test.cwd_path())
        .arg("say ok")
        .output()?;
    assert!(output.status.success(), "{output:?}");
    Ok((
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_announces_openai_api_key_env_fallback_once_on_stderr() -> anyhow::Result<()> {
    let (stdout, stderr) = run_exec_with_env(
        &[(OPENAI_API_KEY_ENV_VAR, "sk-env-fallback")],
        "sk-env-fallback",
        /*json*/ false,
    )
    .await?;

    assert_eq!(
        stderr.matches(OPENAI_API_KEY_ENV_FALLBACK_NOTICE).count(),
        1,
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!("warning: {OPENAI_API_KEY_ENV_FALLBACK_NOTICE}")),
        "{stderr}"
    );
    assert!(
        !stdout.contains(OPENAI_API_KEY_ENV_FALLBACK_NOTICE),
        "{stdout}"
    );
    assert!(!stderr.contains("sk-env-fallback"), "{stderr}");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_json_reports_openai_api_key_env_fallback_as_a_warning_item() -> anyhow::Result<()> {
    let (stdout, _stderr) = run_exec_with_env(
        &[(OPENAI_API_KEY_ENV_VAR, "sk-env-fallback")],
        "sk-env-fallback",
        /*json*/ true,
    )
    .await?;

    // Every stdout line stays a JSON event; the notice is one warning item.
    let events = stdout
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    let notices = events
        .iter()
        .filter(|event| event["item"]["message"] == OPENAI_API_KEY_ENV_FALLBACK_NOTICE)
        .collect::<Vec<_>>();
    assert_eq!(notices.len(), 1, "{stdout}");
    assert_eq!(notices[0]["type"], "item.completed");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_does_not_announce_codex_api_key() -> anyhow::Result<()> {
    let (stdout, stderr) = run_exec_with_env(
        &[
            (CODEX_API_KEY_ENV_VAR, "sk-codex"),
            (OPENAI_API_KEY_ENV_VAR, "sk-env-fallback"),
        ],
        "sk-codex",
        /*json*/ false,
    )
    .await?;

    assert!(!stderr.contains("OPENAI_API_KEY"), "{stderr}");
    assert!(!stdout.contains("OPENAI_API_KEY"), "{stdout}");
    Ok(())
}
