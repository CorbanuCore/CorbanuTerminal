//! End-to-end smoke test for `corbanu exec -o FILE PROMPT`: a scripted,
//! non-interactive run writes the agent's final message to FILE and prints it
//! on stdout. It drives the real `corbanu` binary against a local mock model
//! server; every path it touches is a temporary directory removed on drop.
// Same scope as the exec suite (`codex-rs/exec/tests/suite`), which drives
// exec against this mock server only off Windows.
#![cfg(not(target_os = "windows"))]

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use app_test_support::MockResponsesConfig;
use app_test_support::create_final_assistant_message_sse_response;
use app_test_support::create_mock_responses_server_sequence;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

const PROMPT: &str = "Reply with the smoke-check summary";
// Two lines, so a file holding only the first line fails the check.
const FINAL_MESSAGE: &str = "Smoke check complete.\nBoth lines reach the output file.";

/// `corbanu` isolated from the caller's profile: its own home, no inherited
/// home aliases or API keys, and no native keychain.
fn corbanu(home: &Path, cwd: &Path) -> Result<assert_cmd::Command> {
    let mut command = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("corbanu")?);
    for name in [
        "CORBANU_HOME",
        "PFTERMINAL_HOME",
        "CORBANU_SECURITY_ORIGIN",
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
    ] {
        command.env_remove(name);
    }
    command
        .current_dir(cwd)
        .env("CODEX_HOME", home)
        .env("CODEX_SQLITE_HOME", home)
        .env("CORBANU_TEST_ACCOUNT_HOME", home.join("account"))
        .env("CORBANU_TEST_NO_NATIVE_KEYRING", "1")
        // A hung run fails the test instead of stalling the suite.
        .timeout(Duration::from_secs(90));
    Ok(command)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exec_output_last_message_writes_final_message_to_file() -> Result<()> {
    let home = TempDir::new()?;
    let cwd = TempDir::new()?;
    let response = create_final_assistant_message_sse_response(FINAL_MESSAGE)?;
    // Expects exactly one model request; the server verifies that when dropped.
    let server = create_mock_responses_server_sequence(vec![response]).await;
    MockResponsesConfig::new(&server.uri()).write(home.path())?;
    // A leftover file from an earlier run must be replaced, not kept.
    let output_file = cwd.path().join("last-message.md");
    std::fs::write(&output_file, "stale content from an earlier run")?;

    let assert = corbanu(home.path(), cwd.path())?
        .args(["exec", "--skip-git-repo-check", "-o"])
        .arg(&output_file)
        .arg(PROMPT)
        .assert()
        .success();

    assert_eq!(std::fs::read_to_string(&output_file)?, FINAL_MESSAGE);
    // Piped stdout carries only the final message, so scripts can read it too.
    assert_eq!(
        String::from_utf8(assert.get_output().stdout.clone())?,
        format!("{FINAL_MESSAGE}\n")
    );
    let requests = server
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|request| request.url.path().ends_with("/responses"))
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 1, "expected one model request");
    assert!(
        String::from_utf8_lossy(&requests[0].body).contains(PROMPT),
        "the prompt never reached the model"
    );
    Ok(())
}
