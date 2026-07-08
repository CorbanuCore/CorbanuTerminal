use std::fmt;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::Vault;
use codex_vault::VaultError;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

pub const TASKNODE_SESSION_LABEL: &str = "tasknode/session";
pub const DEFAULT_TASKNODE_ORIGIN: &str = "https://tasknode.postfiat.org";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskNodeLocalSession {
    #[serde(default = "default_tasknode_origin")]
    pub origin: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub github_username: Option<String>,
    #[serde(default)]
    pub terminal_token: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub pending_request_id: Option<String>,
    #[serde(default)]
    pub pending_poll_token: Option<String>,
    #[serde(default)]
    pub pending_verification_url: Option<String>,
}

impl fmt::Debug for TaskNodeLocalSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskNodeLocalSession")
            .field("origin", &self.origin)
            .field("account_id", &self.account_id)
            .field("github_username", &self.github_username)
            .field(
                "terminal_token",
                &self.terminal_token.as_ref().map(|_| "<redacted>"),
            )
            .field("expires_at", &self.expires_at)
            .field("pending_request_id", &self.pending_request_id)
            .field(
                "pending_poll_token",
                &self.pending_poll_token.as_ref().map(|_| "<redacted>"),
            )
            .field("pending_verification_url", &self.pending_verification_url)
            .finish()
    }
}

impl TaskNodeLocalSession {
    pub fn load(codex_home: &Path) -> Result<Self, TaskNodeLocalError> {
        let vault = Vault::new(codex_home.to_path_buf());
        load_from_vault(&vault)
    }

    pub fn load_optional(codex_home: &Path) -> Result<Option<Self>, TaskNodeLocalError> {
        match Self::load(codex_home) {
            Ok(session) => Ok(Some(session)),
            Err(TaskNodeLocalError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn save(&self, codex_home: &Path) -> Result<(), TaskNodeLocalError> {
        let vault = Vault::new(codex_home.to_path_buf());
        save_to_vault(self, &vault)
    }

    pub fn state(&self) -> SessionState {
        if self.terminal_token.is_some() {
            return SessionState::Linked {
                github_username: self.github_username.clone(),
                expires_at: self.expires_at.clone(),
            };
        }

        if let (Some(verification_url), Some(request_id)) = (
            self.pending_verification_url.clone(),
            self.pending_request_id.clone(),
        ) {
            return SessionState::Pending {
                verification_url,
                request_id,
            };
        }

        SessionState::Unlinked
    }

    pub fn apply_terminal_session(&mut self, poll: TerminalSessionResponse) {
        self.account_id = Some(poll.account_id);
        self.github_username = poll.github_username;
        self.terminal_token = Some(poll.terminal_token);
        self.expires_at = poll.expires_at;
        self.pending_request_id = None;
        self.pending_poll_token = None;
        self.pending_verification_url = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Linked {
        github_username: Option<String>,
        expires_at: Option<String>,
    },
    Pending {
        verification_url: String,
        request_id: String,
    },
    Unlinked,
}

pub fn load_session_state(codex_home: &Path) -> Result<SessionState, TaskNodeLocalError> {
    match TaskNodeLocalSession::load(codex_home) {
        Ok(session) => Ok(session.state()),
        Err(TaskNodeLocalError::NotFound) => Ok(SessionState::Unlinked),
        Err(err) => Err(err),
    }
}

pub fn delete_session(codex_home: &Path) -> Result<bool, TaskNodeLocalError> {
    let vault = Vault::new(codex_home.to_path_buf());
    vault
        .delete(TASKNODE_SESSION_LABEL)
        .map_err(|err| TaskNodeLocalError::VaultUnavailable(err.to_string()))
}

#[cfg(test)]
fn load_from_vault(vault: &Vault) -> Result<TaskNodeLocalSession, TaskNodeLocalError> {
    let secret = vault
        .reveal(TASKNODE_SESSION_LABEL)
        .map_err(TaskNodeLocalError::from_vault)?;
    serde_json::from_str(&secret).map_err(|err| TaskNodeLocalError::Corrupt(err.to_string()))
}

#[cfg(not(test))]
fn load_from_vault(vault: &Vault) -> Result<TaskNodeLocalSession, TaskNodeLocalError> {
    let secret = vault
        .reveal(TASKNODE_SESSION_LABEL)
        .map_err(TaskNodeLocalError::from_vault)?;
    serde_json::from_str(&secret).map_err(|err| TaskNodeLocalError::Corrupt(err.to_string()))
}

#[cfg(test)]
fn save_to_vault(session: &TaskNodeLocalSession, vault: &Vault) -> Result<(), TaskNodeLocalError> {
    let secret = serde_json::to_string(session)
        .map_err(|err| TaskNodeLocalError::Corrupt(err.to_string()))?;
    upsert_session_secret(session, vault, secret)
}

#[cfg(not(test))]
fn save_to_vault(session: &TaskNodeLocalSession, vault: &Vault) -> Result<(), TaskNodeLocalError> {
    let secret = serde_json::to_string(session)
        .map_err(|err| TaskNodeLocalError::Corrupt(err.to_string()))?;
    upsert_session_secret(session, vault, secret)
}

fn upsert_session_secret(
    session: &TaskNodeLocalSession,
    vault: &Vault,
    secret: String,
) -> Result<(), TaskNodeLocalError> {
    match vault.add(AddCredential {
        label: TASKNODE_SESSION_LABEL.to_string(),
        credential_type: CredentialType::BearerToken,
        provider: Some("tasknode".to_string()),
        notes: Some("Task Node terminal session; token is not printed to chat.".to_string()),
        revocation_notes: Some(format!("{}/settings/accounts", session.origin)),
        secret: secret.clone(),
    }) {
        Ok(()) => Ok(()),
        Err(VaultError::CredentialExists { .. }) => vault
            .update(
                TASKNODE_SESSION_LABEL,
                Some(secret),
                Some(Some("tasknode".to_string())),
                None,
                None,
            )
            .map(|_| ())
            .map_err(|err| TaskNodeLocalError::VaultUnavailable(err.to_string())),
        Err(err) => Err(TaskNodeLocalError::VaultUnavailable(err.to_string())),
    }
}

#[derive(Debug, Error)]
pub enum TaskNodeLocalError {
    #[error("Task Node credential vault is unavailable: {0}")]
    VaultUnavailable(String),
    #[error("Task Node is not linked")]
    NotFound,
    #[error("invalid local Task Node session: {0}")]
    Corrupt(String),
}

impl TaskNodeLocalError {
    fn from_vault(err: VaultError) -> Self {
        match err {
            VaultError::NotFound { .. } => Self::NotFound,
            err => Self::VaultUnavailable(err.to_string()),
        }
    }
}

pub fn resolve_origin(origin_override: Option<String>, saved_origin: Option<&str>) -> String {
    origin_override
        .or_else(|| std::env::var("PFT_TASKNODE_ORIGIN").ok())
        .or_else(|| std::env::var("TASKNODE_ORIGIN").ok())
        .or_else(|| saved_origin.map(ToString::to_string))
        .unwrap_or_else(|| DEFAULT_TASKNODE_ORIGIN.to_string())
        .trim_end_matches('/')
        .to_string()
}

#[derive(Debug, Clone)]
pub struct TaskNodeClient {
    origin: String,
    token: Option<String>,
}

impl TaskNodeClient {
    pub fn new(token: String) -> Self {
        Self {
            origin: resolve_origin(None, None),
            token: Some(token),
        }
    }

    pub fn new_with_origin(origin: String, token: String) -> Self {
        Self {
            origin: origin.trim_end_matches('/').to_string(),
            token: Some(token),
        }
    }

    pub fn new_without_token() -> Self {
        Self {
            origin: resolve_origin(None, None),
            token: None,
        }
    }

    pub fn new_without_token_for_origin(origin: String) -> Self {
        Self {
            origin: origin.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn bearer_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn url_for_path(&self, path: &str) -> String {
        self.url(path)
    }

    pub fn start_github_link(&self) -> Result<TerminalAuthStartResponse, TaskNodeClientError> {
        self.post_json("/api/auth/terminal/start/github", &serde_json::json!({}))
    }

    pub fn poll_session(
        &self,
        request_id: &str,
        poll_token: &str,
    ) -> Result<TerminalSessionResponse, TaskNodeClientError> {
        let response = self.get_raw_json(&format!(
            "/api/auth/terminal/session?requestId={}&pollToken={}",
            urlencoding::encode(request_id),
            urlencoding::encode(poll_token)
        ))?;
        if response.status == 202 {
            return Err(TaskNodeClientError::Pending);
        }
        if !(200..300).contains(&response.status) {
            let message = tasknode_error_message(response.body);
            return match response.status {
                400 | 401 | 403 | 404 | 410 => Err(TaskNodeClientError::Rejected(message)),
                _ => Err(TaskNodeClientError::Http(message)),
            };
        }
        serde_json::from_value(response.body)
            .map_err(|err| TaskNodeClientError::Http(err.to_string()))
    }

    pub fn get_json<T: DeserializeOwned + Send + 'static>(
        &self,
        path: &str,
    ) -> Result<T, TaskNodeClientError> {
        let response = self.get_raw_json(path)?;
        parse_tasknode_json_response(response)
    }

    pub fn post_json<T: DeserializeOwned + Send + 'static>(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<T, TaskNodeClientError> {
        let response = self.post_raw_json(path, body)?;
        parse_tasknode_json_response(response)
    }

    pub fn get_raw_json(&self, path: &str) -> Result<TaskNodeRawResponse, TaskNodeClientError> {
        let url = self.url(path);
        let token = self.token.clone();
        tasknode_blocking_http(move || {
            let http = tasknode_http_client()?;
            let mut request = http.get(url);
            if let Some(token) = &token {
                request = request.bearer_auth(token);
            }
            parse_raw_response(request.send())
        })
    }

    pub fn post_raw_json(
        &self,
        path: &str,
        body: &Value,
    ) -> Result<TaskNodeRawResponse, TaskNodeClientError> {
        let url = self.url(path);
        let token = self.token.clone();
        let body = body.clone();
        tasknode_blocking_http(move || {
            let http = tasknode_http_client()?;
            let mut request = http.post(url).json(&body);
            if let Some(token) = &token {
                request = request.bearer_auth(token);
            }
            parse_raw_response(request.send())
        })
    }

    pub fn post_sse(
        &self,
        path: &str,
        body: &Value,
        mut on_event: impl FnMut(&str, &Value, &mut String),
    ) -> Result<Value, TaskNodeClientError> {
        let url = self.url(path);
        let token = self.token.clone();
        let body = body.clone();
        let http = tasknode_streaming_http_client()?;
        let mut request = http.post(url).json(&body);
        if let Some(token) = &token {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .map_err(|err| TaskNodeClientError::Http(tasknode_reqwest_error(err)))?;
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();
        if !(200..300).contains(&status) || !content_type.contains("text/event-stream") {
            return parse_tasknode_json_response(parse_raw_response(Ok(response))?);
        }

        let mut buffer = String::new();
        let mut accumulated = String::new();
        let mut done: Option<Value> = None;
        let mut response = response;
        let mut chunk = [0u8; 8192];
        loop {
            let read = response
                .read(&mut chunk)
                .map_err(|err| TaskNodeClientError::Http(err.to_string()))?;
            if read == 0 {
                break;
            }
            buffer.push_str(&String::from_utf8_lossy(&chunk[..read]));
            for block in tasknode_sse_drain_blocks(&mut buffer) {
                let Some((event, value)) = tasknode_parse_sse_block(&block)? else {
                    continue;
                };
                match event.as_str() {
                    "delta" => on_event(&event, &value, &mut accumulated),
                    "done" => done = Some(value),
                    "error" => {
                        return Err(TaskNodeClientError::Http(
                            value
                                .get("message")
                                .or_else(|| value.get("error"))
                                .and_then(Value::as_str)
                                .unwrap_or("Task Node chat stream failed.")
                                .to_string(),
                        ));
                    }
                    _ => {}
                }
            }
        }
        for block in tasknode_sse_drain_remainder(&mut buffer) {
            let Some((event, value)) = tasknode_parse_sse_block(&block)? else {
                continue;
            };
            if event == "done" {
                done = Some(value);
            } else if event == "error" {
                return Err(TaskNodeClientError::Http(
                    value
                        .get("message")
                        .or_else(|| value.get("error"))
                        .and_then(Value::as_str)
                        .unwrap_or("Task Node chat stream failed.")
                        .to_string(),
                ));
            }
        }
        done.ok_or_else(|| {
            TaskNodeClientError::Http(
                "Task Node chat stream ended without a final response.".to_string(),
            )
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.origin.trim_end_matches('/'), path)
    }
}

#[derive(Debug)]
pub struct TaskNodeRawResponse {
    pub status: u16,
    pub body: Value,
}

#[derive(Debug, Error)]
pub enum TaskNodeClientError {
    #[error("pending")]
    Pending,
    #[error("{0}")]
    Rejected(String),
    #[error("{0}")]
    Http(String),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TerminalAuthStartResponse {
    pub request_id: String,
    pub poll_token: String,
    pub verification_url: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionResponse {
    pub account_id: String,
    pub github_username: Option<String>,
    pub terminal_token: String,
    pub expires_at: Option<String>,
}

pub fn tasknode_sse_drain_blocks(buffer: &mut String) -> Vec<String> {
    let mut blocks = Vec::new();
    while let Some((index, separator_len)) = tasknode_sse_separator(buffer) {
        let drained: String = buffer.drain(..index + separator_len).collect();
        blocks.push(drained[..index].to_string());
    }
    blocks
}

pub fn tasknode_sse_drain_remainder(buffer: &mut String) -> Vec<String> {
    let remainder = std::mem::take(buffer);
    if remainder.trim().is_empty() {
        Vec::new()
    } else {
        vec![remainder]
    }
}

pub fn tasknode_parse_sse_block(
    block: &str,
) -> Result<Option<(String, Value)>, TaskNodeClientError> {
    let normalized = block.replace("\r\n", "\n");
    let mut event = "message".to_string();
    let mut data = Vec::new();
    for line in normalized.lines() {
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("event:") {
            event = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data.push(rest.trim_start().to_string());
        }
    }
    if data.is_empty() {
        return Ok(None);
    }
    let data = data.join("\n");
    if data.trim() == "[DONE]" {
        return Ok(None);
    }
    let value = serde_json::from_str(&data).map_err(|err| {
        TaskNodeClientError::Http(format!("invalid Task Node chat stream event: {err}"))
    })?;
    Ok(Some((event, value)))
}

fn default_tasknode_origin() -> String {
    DEFAULT_TASKNODE_ORIGIN.to_string()
}

fn tasknode_http_client() -> Result<reqwest::blocking::Client, TaskNodeClientError> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(45))
        .build()
        .map_err(|err| TaskNodeClientError::Http(tasknode_reqwest_error(err)))
}

fn tasknode_streaming_http_client() -> Result<reqwest::blocking::Client, TaskNodeClientError> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|err| TaskNodeClientError::Http(tasknode_reqwest_error(err)))
}

fn tasknode_blocking_http<T: Send + 'static>(
    request: impl FnOnce() -> Result<T, TaskNodeClientError> + Send + 'static,
) -> Result<T, TaskNodeClientError> {
    let handle = std::thread::Builder::new()
        .name("tasknode-http".to_string())
        .spawn(request)
        .map_err(|err| TaskNodeClientError::Http(err.to_string()))?;
    handle
        .join()
        .map_err(|_| TaskNodeClientError::Http("Task Node HTTP worker panicked".to_string()))?
}

fn parse_raw_response(
    response: Result<reqwest::blocking::Response, reqwest::Error>,
) -> Result<TaskNodeRawResponse, TaskNodeClientError> {
    let response =
        response.map_err(|err| TaskNodeClientError::Http(tasknode_reqwest_error(err)))?;
    let status = response.status().as_u16();
    let text = response
        .text()
        .map_err(|err| TaskNodeClientError::Http(tasknode_reqwest_error(err)))?;
    let body = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| {
        serde_json::json!({
            "ok": false,
            "error": "tasknode_non_json_response",
            "message": text,
            "httpStatus": status,
        })
    });
    Ok(TaskNodeRawResponse { status, body })
}

fn parse_tasknode_json_response<T: DeserializeOwned>(
    response: TaskNodeRawResponse,
) -> Result<T, TaskNodeClientError> {
    if response.status == 202 {
        return Err(TaskNodeClientError::Pending);
    }
    if !(200..300).contains(&response.status) {
        return Err(TaskNodeClientError::Http(tasknode_error_message(
            response.body,
        )));
    }
    serde_json::from_value(response.body).map_err(|err| TaskNodeClientError::Http(err.to_string()))
}

fn tasknode_error_message(body: Value) -> String {
    body.get("message")
        .or_else(|| body.get("error"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| body.to_string())
}

fn tasknode_sse_separator(buffer: &str) -> Option<(usize, usize)> {
    match (buffer.find("\n\n"), buffer.find("\r\n\r\n")) {
        (Some(lf), Some(crlf)) if crlf < lf => Some((crlf, 4)),
        (Some(lf), _) => Some((lf, 2)),
        (None, Some(crlf)) => Some((crlf, 4)),
        (None, None) => None,
    }
}

fn tasknode_reqwest_error(mut err: reqwest::Error) -> String {
    if let Some(url) = err.url_mut() {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_query(None);
    }
    let mut message = err.to_string();
    let mut source = std::error::Error::source(&err);
    while let Some(err) = source {
        let part = err.to_string();
        if !part.is_empty() && !message.contains(&part) {
            message.push_str(": ");
            message.push_str(&part);
        }
        source = std::error::Error::source(err);
    }
    message
}
