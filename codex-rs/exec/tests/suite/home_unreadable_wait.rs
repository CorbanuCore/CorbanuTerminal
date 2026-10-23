//! #616: when Corbanu's auth helper reports an unreadable home (exit 75),
//! `exec` waits with backoff for access to return and then finishes the turn;
//! when the wait window is spent it still ends with an error.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use core_test_support::responses::ev_completed;
use core_test_support::responses::mount_sse_once_match;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex_exec::TestCodexExecBuilder;
use core_test_support::test_codex_exec::test_codex_exec;
use wiremock::MockServer;
use wiremock::matchers::header;

/// A stand-in for `corbanu internal-claude-oauth-token`: while `blocked`
/// exists it fails as an unreadable home does, then prints a token.
fn write_helper(test: &TestCodexExecBuilder) -> anyhow::Result<std::path::PathBuf> {
    let home = test.home_path();
    let helper = home.join("home-helper.sh");
    std::fs::write(
        &helper,
        format!(
            "#!/bin/sh\necho attempt >> '{attempts}'\nif [ -e '{blocked}' ]; then\n  \
             echo 'Corbanu could not read its home (Operation not permitted)' >&2\n  exit 75\nfi\n\
             printf home-token\n",
            attempts = home.join("attempts.log").display(),
            blocked = home.join("blocked").display(),
        ),
    )?;
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755))?;
    std::fs::write(home.join("blocked"), "")?;
    Ok(helper)
}

fn exec_command(
    test: &TestCodexExecBuilder,
    server: &MockServer,
    helper: &Path,
    home_wait_secs: Option<u64>,
) -> std::process::Command {
    let mut cmd = std::process::Command::new(
        codex_utils_cargo_bin::cargo_bin("codex-exec").expect("codex-exec binary"),
    );
    cmd.current_dir(test.cwd_path())
        .env("CODEX_HOME", test.home_path())
        .env("CODEX_SQLITE_HOME", test.home_path())
        .arg("--skip-git-repo-check");
    if let Some(secs) = home_wait_secs {
        cmd.arg("--home-wait-secs").arg(secs.to_string());
    }
    cmd.arg("-c")
        .arg(format!(
            "model_providers.home_auth={{name=\"home auth\",base_url={:?},wire_api=\"responses\",\
             supports_websockets=false,stream_max_retries=0,request_max_retries=0,\
             auth={{command={:?},args=[\"internal-claude-oauth-token\"],timeout_ms=5000,\
             refresh_interval_ms=0}}}}",
            format!("{}/v1", server.uri()),
            helper.display().to_string(),
        ))
        .arg("-c")
        .arg("model_provider=\"home_auth\"")
        .arg("say hello")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

fn attempts(test: &TestCodexExecBuilder) -> usize {
    std::fs::read_to_string(test.home_path().join("attempts.log"))
        .map(|log| log.lines().count())
        .unwrap_or(0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_waits_for_an_unreadable_home_and_then_finishes_the_turn() -> anyhow::Result<()> {
    let server = start_mock_server().await;
    mount_sse_once_match(
        &server,
        header("Authorization", "Bearer home-token"),
        sse(vec![ev_completed("request_0")]),
    )
    .await;
    let test = test_codex_exec();
    let helper = write_helper(&test)?;
    let mut child = exec_command(
        &test, &server, &helper, /*home_wait_secs*/ None, // the default window
    )
    .spawn()?;

    // Keep the home "blocked" for a few seconds and several helper runs,
    // longer than exec's ordinary retries last; then let access return.
    let started = Instant::now();
    let mut first_failure = None;
    loop {
        if let Some(status) = child.try_wait()? {
            let output = child.wait_with_output()?;
            panic!(
                "exec ended ({status}) while the home was unreadable:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        if attempts(&test) > 0 {
            let first = *first_failure.get_or_insert_with(Instant::now);
            if attempts(&test) >= 4 && first.elapsed() >= Duration::from_secs(3) {
                break;
            }
        }
        assert!(
            started.elapsed() < Duration::from_secs(120),
            "no helper run"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    std::fs::remove_file(test.home_path().join("blocked"))?;

    let output = tokio::task::spawn_blocking(move || child.wait_with_output()).await??;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr:\n{stderr}");
    assert!(
        stderr.contains("Corbanu could not read its home (Operation not permitted); waiting"),
        "each wait is reported:\n{stderr}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_ends_with_an_error_when_the_home_stays_unreadable() -> anyhow::Result<()> {
    let server = start_mock_server().await;
    let test = test_codex_exec();
    let helper = write_helper(&test)?;

    let output = tokio::task::spawn_blocking({
        let mut cmd = exec_command(&test, &server, &helper, /*home_wait_secs*/ Some(3));
        move || cmd.output()
    })
    .await??;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.contains("Corbanu could not read its home"),
        "the failure is reported:\n{stderr}"
    );
    let requests = server.received_requests().await.unwrap_or_default();
    assert!(
        requests
            .iter()
            .all(|request| request.method.as_str() == "GET"),
        "no model turn without a credential"
    );
    Ok(())
}
