//! #2324: a launch asking the executor to drop inherited URL passwords is
//! refused before it reaches an executor that would ignore the request.
use std::time::Duration;

use codex_exec_server_protocol::JSONRPCMessage;
use codex_exec_server_protocol::JSONRPCResponse;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use codex_protocol::config_types::ShellEnvironmentPolicyInherit;
use codex_utils_path_uri::PathUri;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use super::super::LazyRemoteExecServerClient;
use super::accept_websocket;
use super::complete_websocket_initialize;
use super::write_jsonrpc_websocket;
use crate::ProcessId;
use crate::client_api::ExecServerTransportParams;
use crate::protocol::ENVIRONMENT_INFO_METHOD;
use crate::protocol::EXEC_ENV_URL_PASSWORD_FILTER_UNSUPPORTED;
use crate::protocol::EXEC_METHOD;
use crate::protocol::EnvironmentCapabilities;
use crate::protocol::EnvironmentInfo;
use crate::protocol::ExecEnvPolicy;
use crate::protocol::ExecParams;

fn params(drop_url_password_values: bool) -> ExecParams {
    ExecParams {
        process_id: ProcessId::from("url-password-filter"),
        argv: vec!["true".to_string()],
        cwd: PathUri::from_host_native_path(std::env::current_dir().expect("cwd"))
            .expect("cwd URI"),
        env_policy: Some(ExecEnvPolicy {
            inherit: ShellEnvironmentPolicyInherit::Core,
            ignore_default_excludes: true,
            exclude: Vec::new(),
            r#set: Default::default(),
            include_only: vec!["PATH".to_string()],
            drop_url_password_values,
        }),
        env: Default::default(),
        tty: false,
        pipe_stdin: false,
        arg0: None,
        sandbox: None,
        enforce_managed_network: false,
        managed_network: None,
        network_proxy: None,
    }
}

/// Runs one `start_process` against a scripted executor reporting
/// `env_url_password_filter = supported`; returns the `exec` request the
/// executor received, if any.
async fn start_against(
    supported: bool,
    params: ExecParams,
) -> (
    Result<(), crate::client::ExecServerError>,
    Option<serde_json::Value>,
) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let websocket_url = format!("ws://{}", listener.local_addr().expect("listener address"));
    let server = tokio::spawn(async move {
        let mut websocket = accept_websocket(&listener).await;
        complete_websocket_initialize(&mut websocket, "p", Default::default()).await;
        let mut exec = None;
        // Serve until the client hangs up, recording any `exec` request.
        while let Ok(Some(Ok(frame))) = timeout(Duration::from_secs(5), websocket.next()).await {
            let Message::Text(text) = frame else {
                continue;
            };
            let Ok(JSONRPCMessage::Request(request)) = serde_json::from_str(text.as_ref()) else {
                continue;
            };
            let result = match request.method.as_str() {
                ENVIRONMENT_INFO_METHOD => {
                    let mut info = EnvironmentInfo::local();
                    info.capabilities = EnvironmentCapabilities {
                        env_url_password_filter: supported,
                        ..EnvironmentCapabilities::default()
                    };
                    serde_json::to_value(info).expect("info serializes")
                }
                EXEC_METHOD => {
                    exec = request.params;
                    serde_json::json!({"processId": "url-password-filter"})
                }
                _ => serde_json::json!({"running": false}),
            };
            write_jsonrpc_websocket(
                &mut websocket,
                JSONRPCMessage::Response(JSONRPCResponse {
                    id: request.id,
                    result,
                }),
            )
            .await;
        }
        exec
    });
    let client = LazyRemoteExecServerClient::new(
        ExecServerTransportParams::websocket_url(websocket_url, Duration::from_secs(1)),
        HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
    )
    .get()
    .await
    .expect("client should connect");
    let result = client
        .start_process(
            params, /*network_policy_decider*/ None, /*network_denial_observer*/ None,
        )
        .await
        .map(drop);
    drop(client);
    (result, server.await.expect("server task should finish"))
}

#[tokio::test]
async fn url_password_filter_is_refused_by_executors_without_it() {
    let (result, exec) = start_against(
        /*supported*/ false,
        params(/*drop_url_password_values*/ true),
    )
    .await;
    let error = result.expect_err("an executor that would ignore the filter is refused");
    assert!(
        error
            .to_string()
            .contains(EXEC_ENV_URL_PASSWORD_FILTER_UNSUPPORTED),
        "{error}"
    );
    assert_eq!(exec, None, "the launch reached an executor that ignores it");
}

#[tokio::test]
async fn url_password_filter_reaches_executors_with_it() {
    let (result, exec) = start_against(
        /*supported*/ true,
        params(/*drop_url_password_values*/ true),
    )
    .await;
    result.expect("supported executor starts the process");
    let exec = exec.expect("exec request sent");
    assert_eq!(exec["envPolicy"]["dropUrlPasswordValues"], true);
}

#[tokio::test]
async fn unfiltered_launches_keep_working_with_older_executors() {
    let (result, exec) = start_against(
        /*supported*/ false,
        params(/*drop_url_password_values*/ false),
    )
    .await;
    result.expect("a launch without the filter needs no capability");
    // Older executors never see the new field.
    let exec = exec.expect("exec request sent");
    assert_eq!(exec["envPolicy"].get("dropUrlPasswordValues"), None);
}
