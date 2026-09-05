//! Synthetic provider, evidence and identity support for the owned manual fixture.
use super::{CANARY, FOREGROUND, Case};
use crate::support::tmux::TmuxSession;
use anyhow::{Context, Result};
use core_test_support::responses;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Duration};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub(super) struct Artifacts {
    pub(super) binary: std::path::PathBuf,
    pub(super) identity: Value,
}

impl Artifacts {
    pub(super) fn load() -> Result<Self> {
        let binary = codex_utils_cargo_bin::cargo_bin("codex")?;
        let runner = std::env::current_exe()?;
        let identity = json!({"candidate":binary, "candidate_sha256":digest(&binary)?,
            "runner":runner, "runner_sha256":digest(&runner)?});
        Ok(Self { binary, identity })
    }
}

pub(super) async fn fake_provider(label: &'static str, case: Case, delay: Duration) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST")).respond_with(move |request: &wiremock::Request| {
        let memory = String::from_utf8_lossy(&request.body).contains(CANARY);
        let text = if memory { r#"{"raw_memory":"synthetic result","rollout_summary":"synthetic summary","rollout_slug":"fixture"}"#.to_owned() } else { format!("{label} foreground complete") };
        let response = ResponseTemplate::new(200).insert_header("content-type", "text/event-stream").set_body_string(responses::sse(vec![responses::ev_response_created("fixture"), responses::ev_assistant_message("fixture-message", &text), responses::ev_completed("fixture")]));
        if memory && case != Case::Startup { response.set_delay(delay) } else { response }
    }).mount(&server).await;
    server
}

pub(super) async fn routing(a: &MockServer, b: &MockServer) -> Vec<Value> {
    let mut routes = Vec::new();
    for (label, server) in [("A", a), ("B", b)] {
        for request in server.received_requests().await.unwrap_or_default() {
            let text = String::from_utf8_lossy(&request.body);
            let body: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
            routes.push(json!({"endpoint":label, "model":body["model"], "kind":if text.contains(CANARY) {"memory"} else if text.contains(FOREGROUND) {"foreground"} else {"other"}}));
        }
    }
    routes
}

pub(super) fn publish_attachment(
    session: &TmuxSession<'_>,
    root: &Path,
    home: &Path,
    artifacts: &Artifacts,
) -> Result<()> {
    let command = session.attachment_command();
    let socket = command
        .get_envs()
        .find(|(key, _)| *key == "TMUX_TMPDIR")
        .and_then(|(_, value)| value)
        .context("owned socket env")?;
    let quote =
        |value: &std::ffi::OsStr| format!("'{}'", value.to_string_lossy().replace('\'', "'\\''"));
    let args = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(quote)
        .collect::<Vec<_>>()
        .join(" ");
    fs::write(
        root.join("attach.sh"),
        format!(
            "#!/bin/sh\nexport TMUX_TMPDIR={}\nexec {args}\n",
            quote(socket)
        ),
    )?;
    let mut ready = artifacts.identity.clone();
    ready["home"] = json!(home);
    ready["socket_dir"] = json!(socket.to_string_lossy());
    ready["attach"] = json!("Run: bash <evidence>/attach.sh");
    ready["restart"] = json!("After exit: touch <evidence>/restart");
    ready["cancel"] = json!("touch <evidence>/cancel");
    write_json(root, "ready.json", &ready)?;
    eprintln!(
        "Human memory fixture ready: {} (bash {}/attach.sh)",
        root.display(),
        root.display()
    );
    Ok(())
}

fn digest(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub(super) fn write_json(root: &Path, name: &str, value: &Value) -> Result<()> {
    let temporary = root.join(format!("{name}.tmp"));
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    fs::rename(temporary, root.join(name))?;
    Ok(())
}
