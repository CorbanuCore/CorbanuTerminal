//! Issue #310: provider credential variables in Corbanu's environment must not
//! reach commands the model runs, while the provider itself still reads them.
#![cfg(not(target_os = "windows"))]

use anyhow::Context;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex_exec::test_codex_exec;
use pretty_assertions::assert_eq;
use serde_json::json;

/// The mock provider's own `env_key`. Unlike the built-in names it doesn't
/// look like a secret, so only the provider-key filter can catch it.
const PROVIDER_ENV_KEY: &str = "CORP_LLM_CRED";
const PROVIDER_KEY: &str = "corp-test-key-310";
const CALL_ID: &str = "env-probe";
const PROBE: &str = "for v in CORP_LLM_CRED ZAI_API_KEY OPENROUTER_API_KEY CODEX_API_KEY CORBANU_310_CONTROL; do \
if printenv \"$v\" >/dev/null; then echo \"$v=PRESENT\"; else echo \"$v=ABSENT\"; fi; done";

/// Runs one `corbanu exec` turn whose model calls `tool_name` with the env
/// probe, using a configured provider authenticated by [`PROVIDER_ENV_KEY`].
/// Returns the probe output and the `Authorization` header of every model
/// request.
async fn probe_tool_env(
    tool_name: &str,
    arguments: serde_json::Value,
    extra_config: &[&str],
) -> anyhow::Result<(String, Vec<Option<String>>)> {
    let server = start_mock_server().await;
    let mock = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("resp-1"),
                ev_function_call(CALL_ID, tool_name, &arguments.to_string()),
                ev_completed("resp-1"),
            ]),
            sse(vec![
                ev_response_created("resp-2"),
                ev_assistant_message("msg-1", "done"),
                ev_completed("resp-2"),
            ]),
        ],
    )
    .await;

    let test = test_codex_exec();
    let mut cmd = test.cmd();
    cmd.env(PROVIDER_ENV_KEY, PROVIDER_KEY)
        .env("ZAI_API_KEY", "zai-test-key-310")
        .env("OPENROUTER_API_KEY", "openrouter-test-key-310")
        .env("CORBANU_310_CONTROL", "1")
        .arg("--skip-git-repo-check")
        .arg("-s")
        .arg("danger-full-access")
        .arg("-c")
        .arg(format!(
            "model_providers.corp_mock={{name=\"corp mock\",base_url={:?},wire_api=\"responses\",env_key=\"{PROVIDER_ENV_KEY}\",supports_websockets=false}}",
            format!("{}/v1", server.uri())
        ))
        .arg("-c")
        .arg("model_provider=\"corp_mock\"");
    for config in extra_config {
        cmd.arg("-c").arg(config);
    }
    cmd.arg("probe the environment").assert().success();

    let requests = mock.requests();
    let output = requests
        .last()
        .context("model received the tool output")?
        .function_call_output(CALL_ID)["output"]
        .as_str()
        .context("tool output is text")?
        .to_string();
    let auth = requests
        .iter()
        .map(|request| request.header("authorization"))
        .collect();
    Ok((output, auth))
}

fn assert_provider_authenticated(auth: Vec<Option<String>>) {
    let expected = Some(format!("Bearer {PROVIDER_KEY}"));
    assert_eq!(auth, vec![expected.clone(), expected]);
}

fn assert_probe(output: &str, zai: &str) {
    for line in [
        "CORP_LLM_CRED=ABSENT".to_string(),
        format!("ZAI_API_KEY={zai}"),
        "OPENROUTER_API_KEY=ABSENT".to_string(),
        "CODEX_API_KEY=ABSENT".to_string(),
        "CORBANU_310_CONTROL=PRESENT".to_string(),
    ] {
        assert!(output.contains(&line), "expected {line} in: {output}");
    }
}

async fn assert_tool_hides_provider_keys(tool_name: &str, login: bool) -> anyhow::Result<()> {
    let (arguments, unified_exec) = match tool_name {
        "shell_command" => (json!({ "command": PROBE, "login": login }), false),
        _ => (
            json!({ "cmd": PROBE, "login": login, "yield_time_ms": 5_000 }),
            true,
        ),
    };
    let feature = format!("features.unified_exec={unified_exec}");
    let (output, auth) = probe_tool_env(tool_name, arguments, &[feature.as_str()]).await?;
    assert_probe(&output, "ABSENT");
    assert_provider_authenticated(auth);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shell_command_does_not_inherit_provider_keys() -> anyhow::Result<()> {
    assert_tool_hides_provider_keys("shell_command", /*login*/ false).await
}

/// Login shells source the session's shell snapshot, which must not
/// re-export the keys either.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn login_shell_command_does_not_inherit_provider_keys() -> anyhow::Result<()> {
    assert_tool_hides_provider_keys("shell_command", /*login*/ true).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unified_exec_does_not_inherit_provider_keys() -> anyhow::Result<()> {
    assert_tool_hides_provider_keys("exec_command", /*login*/ false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn login_unified_exec_does_not_inherit_provider_keys() -> anyhow::Result<()> {
    assert_tool_hides_provider_keys("exec_command", /*login*/ true).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exact_include_only_entry_passes_a_provider_key_through() -> anyhow::Result<()> {
    let (output, auth) = probe_tool_env(
        "exec_command",
        json!({ "cmd": PROBE, "login": true, "yield_time_ms": 5_000 }),
        &[
            "features.unified_exec=true",
            "shell_environment_policy.include_only=[\"*\",\"ZAI_API_KEY\"]",
        ],
    )
    .await?;
    assert_probe(&output, "PRESENT");
    assert_provider_authenticated(auth);
    Ok(())
}
