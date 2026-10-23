//! #616: when a provider's auth command keeps failing (as when macOS revoked
//! access to the drive holding it), `exec` retries it a bounded number of
//! times, reports the failure and exits non-zero instead of hanging.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use core_test_support::responses::start_mock_server;
use core_test_support::test_codex_exec::test_codex_exec;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_failing_provider_auth_command_ends_exec_with_an_error() -> anyhow::Result<()> {
    let server = start_mock_server().await;
    let test = test_codex_exec();
    let helper = test.home_path().join("failing-auth-helper.sh");
    let attempts = test.home_path().join("auth-attempts.log");
    std::fs::write(
        &helper,
        format!(
            "#!/bin/sh\necho attempt >> '{}'\necho 'Operation not permitted' >&2\nexit 1\n",
            attempts.display()
        ),
    )?;
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755))?;

    let output = test
        .cmd()
        .timeout(Duration::from_secs(120))
        .arg("--skip-git-repo-check")
        .arg("-c")
        .arg(format!(
            "model_providers.cmd_auth={{name=\"cmd auth\",base_url={:?},wire_api=\"responses\",\
             supports_websockets=false,stream_max_retries=2,request_max_retries=0,\
             auth={{command={:?},timeout_ms=5000,refresh_interval_ms=0}}}}",
            format!("{}/v1", server.uri()),
            helper.display().to_string(),
        ))
        .arg("-c")
        .arg("model_provider=\"cmd_auth\"")
        .arg("say hello")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.contains("provider auth command"),
        "the failure is reported:\n{stderr}"
    );
    // Retried, but boundedly: never sent without a credential.
    let attempts = std::fs::read_to_string(&attempts).unwrap_or_default();
    let attempts = attempts.lines().count();
    assert!(
        (2..=30).contains(&attempts),
        "auth command ran {attempts} times"
    );
    // The model catalogue may still be listed (GET /models); no turn is sent.
    let requests = server.received_requests().await.unwrap_or_default();
    let summary: Vec<String> = requests
        .iter()
        .filter(|request| request.method.as_str() != "GET")
        .map(|request| {
            format!(
                "{} {} auth={:?}",
                request.method,
                request.url.path(),
                request.headers.get("authorization")
            )
        })
        .collect();
    assert!(
        summary.is_empty(),
        "no model turn without a credential: {summary:?}"
    );
    Ok(())
}
