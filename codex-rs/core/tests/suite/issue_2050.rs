//! #2050 (D510): #1976's rule (D491) applies where the work runs. A session
//! whose environment is a remote Windows machine without the Windows sandbox
//! refuses agent commands and file-tool access under Moderate and Aggressive,
//! whatever this host is, naming the level, the environment and the steps
//! for that machine; nothing reaches the executor. Permissive is unchanged,
//! and so is a remote Linux machine.

use std::sync::Arc;
use std::sync::Mutex;

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use futures::SinkExt;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

const CALL_ID: &str = "call-2050";

#[derive(Clone, Copy, Debug)]
enum Call {
    Command,
    ViewImage,
}

#[derive(Clone, Copy, Debug)]
enum Host {
    /// Windows with no Windows sandbox set up.
    Windows,
    /// An older executor on Windows that reports no sandbox.
    OlderWindows,
    Linux,
}

fn info(host: Host) -> Value {
    let windows_shell = json!({ "name": "powershell", "path": "powershell.exe" });
    match host {
        Host::Windows => json!({
            "shell": windows_shell,
            "cwd": "file:///C:/work",
            "hostSandbox": { "os": "windows", "windowsSandboxSetUp": false },
        }),
        Host::OlderWindows => json!({ "shell": windows_shell, "cwd": "file:///C:/work" }),
        Host::Linux => json!({
            "shell": { "name": "bash", "path": "/bin/bash" },
            "cwd": "file:///home/me",
            "hostSandbox": { "os": "linux" },
        }),
    }
}

/// A fake executor: answers `initialize` and `environment/info`, records
/// every other request and answers it with an error.
async fn fake_executor(host: Host) -> anyhow::Result<(String, Arc<Mutex<Vec<String>>>)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("ws://{}", listener.local_addr()?);
    let methods = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&methods);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let seen = Arc::clone(&seen);
            tokio::spawn(async move {
                let Ok(mut websocket) = tokio_tungstenite::accept_async(stream).await else {
                    return;
                };
                while let Some(Ok(message)) = websocket.next().await {
                    let request: Value = match message {
                        Message::Text(text) => match serde_json::from_str(text.as_ref()) {
                            Ok(request) => request,
                            Err(_) => continue,
                        },
                        _ => continue,
                    };
                    let method = request["method"].as_str().unwrap_or_default().to_string();
                    let Some(id) = request.get("id").cloned() else {
                        continue;
                    };
                    let reply = match method.as_str() {
                        "initialize" => json!({ "id": id, "result": { "sessionId": "fake" } }),
                        "environment/info" => json!({ "id": id, "result": info(host) }),
                        _ => {
                            seen.lock().expect("methods").push(method);
                            json!({
                                "id": id,
                                "error": { "code": -32601, "message": "not served by the fake" },
                            })
                        }
                    };
                    if websocket
                        .send(Message::Text(reply.to_string().into()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }
    });
    Ok((url, methods))
}

struct Outcome {
    output: String,
    /// The requests other than `initialize` and `environment/info` the
    /// executor got.
    reached: Vec<String>,
}

async fn run(level: SecurityLevel, host: Host, call: Call) -> anyhow::Result<Outcome> {
    let server = start_mock_server().await;
    let (url, methods) = fake_executor(host).await?;
    let test = test_codex()
        .with_exec_server_url(url)
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("test config should allow feature update");
            config.use_experimental_unified_exec_tool = true;
            config
                .features
                .enable(Feature::UnifiedExec)
                .expect("test config should allow feature update");
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
                .expect("profile");
        })
        .build(&server)
        .await?;
    // Only what the tool call sends counts.
    methods.lock().expect("methods").clear();
    let (tool, arguments) = match call {
        Call::Command => ("exec_command", json!({ "cmd": "echo issue2050", "yield_time_ms": 500 })),
        Call::ViewImage => ("view_image", json!({ "path": "C:\\work\\picture.png" })),
    };
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created(CALL_ID),
                ev_function_call(CALL_ID, tool, &arguments.to_string()),
                ev_completed(CALL_ID),
            ]),
            sse(vec![
                ev_response_created("done"),
                ev_assistant_message("msg-done", "done"),
                ev_completed("done"),
            ]),
        ],
    )
    .await;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "go".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    wait_for_event(&test.codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    let request = captured
        .last_request()
        .ok_or_else(|| anyhow::anyhow!("no request after the tool call"))?;
    let reached = methods.lock().expect("methods").clone();
    Ok(Outcome {
        output: request
            .function_call_output_text(CALL_ID)
            .unwrap_or_default(),
        reached,
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_2050_remote_windows_without_its_sandbox_refuses_commands_and_file_tools()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let mut wrong = Vec::new();
    for (level, name) in [
        (SecurityLevel::Permissive, "Permissive"),
        (SecurityLevel::Moderate, "Moderate"),
        (SecurityLevel::Aggressive, "Aggressive"),
    ] {
        for host in [Host::Windows, Host::OlderWindows, Host::Linux] {
            let refused = level != SecurityLevel::Permissive && !matches!(host, Host::Linux);
            // A Windows client keeps refusing every command under these
            // levels without its own sandbox (#1976, #2049); not this test's
            // concern.
            let client_refuses = cfg!(windows) && level != SecurityLevel::Permissive;
            let command = run(level, host, Call::Command).await?;
            let expected = format!("Security level {name} refused this command");
            let ok = if refused {
                command.output.contains(&expected)
                    && (client_refuses
                        || (command.output.contains("environment `remote`")
                            && command.output.contains("on that machine")))
                    && !command.reached.iter().any(|method| method == "process/start")
            } else if client_refuses {
                !command.reached.iter().any(|method| method == "process/start")
            } else {
                command.reached.iter().any(|method| method == "process/start")
            };
            if !ok {
                wrong.push(format!(
                    "{name} {host:?} command: output {}; reached {:?}",
                    command.output, command.reached
                ));
            }
            let image = run(level, host, Call::ViewImage).await?;
            let expected = format!("Security level {name} refused this file access");
            let ok = if refused {
                image.output.contains(&expected)
                    && image.output.contains("environment `remote`")
                    && !image.reached.iter().any(|method| method.starts_with("fs/"))
            } else {
                image.reached.iter().any(|method| method.starts_with("fs/"))
            };
            if !ok {
                wrong.push(format!(
                    "{name} {host:?} image: output {}; reached {:?}",
                    image.output, image.reached
                ));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
