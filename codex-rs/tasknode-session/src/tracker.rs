//! Bounded encrypted Campaign Tracker outbox, isolated by profile/account/origin.
use crate::ActiveSession;
use crate::Client;
use crate::SessionScope;
use crate::SessionStore;
use base64::Engine;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::XNonce;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::AeadCore;
use chacha20poly1305::aead::KeyInit;
use chacha20poly1305::aead::OsRng;
use chacha20poly1305::aead::Payload;
use codex_vault::Vault;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use uuid::Uuid;
use zeroize::Zeroizing;

pub const OUTBOX_LIMIT: usize = 256 * 1024 * 1024;
const OUTPUT_LIMIT: usize = 32 * 1024;
const PROMPT_LIMIT: usize = 1024 * 1024;
const LABEL: &str = "campaign-tracker/outbox-key-v1";
pub const API: &str = "/api/terminal/tasknode/campaign-tracker";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrackerState {
    pub instance_id: String,
    pub next_sequence: u64,
    pub workspaces: Vec<String>,
    pub pending: Vec<Value>,
    pub last_error: Option<String>,
    pub gap_count: u64,
}
impl Default for TrackerState {
    fn default() -> Self {
        Self {
            instance_id: Uuid::new_v4().to_string(),
            next_sequence: 0,
            workspaces: Vec::new(),
            pending: Vec::new(),
            last_error: None,
            gap_count: 0,
        }
    }
}
#[derive(Clone)]
pub struct TrackerStore {
    store: Arc<dyn SessionStore + Send + Sync>,
    lock_path: PathBuf,
    origin: String,
    key: Arc<OnceLock<Zeroizing<[u8; 32]>>>,
}
impl TrackerStore {
    pub fn new(home: &Path, scope: &SessionScope, session: &ActiveSession) -> Result<Self, String> {
        let account = session
            .account_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("Relink Task Node before recording activity.")?;
        let origin = crate::normalize_origin(&session.origin).map_err(|e| e.to_string())?;
        let identity =
            serde_json::to_vec(&(scope.profile(), account, &origin)).map_err(|e| e.to_string())?;
        let root = home
            .join("campaign-tracker")
            .join(format!("{:x}", Sha256::digest(identity)));
        Ok(Self {
            store: Arc::new(Vault::new(root.clone())),
            lock_path: root.join("outbox.lock"),
            origin,
            key: Arc::new(OnceLock::new()),
        })
    }
    fn locked<T>(&self, operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        std::fs::create_dir_all(self.lock_path.parent().ok_or("Missing tracker directory")?)
            .map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.lock_path)
            .map_err(|e| e.to_string())?;
        lock.lock().map_err(|e| e.to_string())?;
        operation()
    }
    fn cipher(&self) -> Result<XChaCha20Poly1305, String> {
        if self.key.get().is_none() {
            let engine = base64::engine::general_purpose::STANDARD;
            let key: [u8; 32] = match self
                .store
                .reveal_optional(LABEL)
                .map_err(|e| e.to_string())?
            {
                Some(saved) => engine
                    .decode(saved)
                    .map_err(|_| "Invalid tracker key")?
                    .try_into()
                    .map_err(|_| "Invalid tracker key")?,
                None => {
                    if self.lock_path.with_file_name("outbox.bin").exists() {
                        return Err("Tracker key missing; existing outbox preserved.".to_string());
                    }
                    let key: [u8; 32] = XChaCha20Poly1305::generate_key(&mut OsRng).into();
                    self.store
                        .upsert(
                            LABEL,
                            engine.encode(key),
                            "Campaign Tracker encrypted outbox key",
                            &self.origin,
                        )
                        .map_err(|e| e.to_string())?;
                    key
                }
            };
            let _ = self.key.set(Zeroizing::new(key));
        }
        XChaCha20Poly1305::new_from_slice(self.key.get().ok_or("Tracker key unavailable")?.as_ref())
            .map_err(|_| "Invalid tracker key".to_string())
    }
    fn read(&self) -> Result<TrackerState, String> {
        let path = self.lock_path.with_file_name("outbox.bin");
        let size = match std::fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(TrackerState::default());
            }
            Err(error) => return Err(error.to_string()),
        };
        if size > (OUTBOX_LIMIT + 1024 * 1024) as u64 {
            return Err("Tracker outbox exceeds its bound; existing data preserved.".to_string());
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        if bytes.len() < 44 || &bytes[..4] != b"CT01" {
            return Err("Invalid tracker outbox; existing data preserved.".to_string());
        }
        let plaintext = Zeroizing::new(
            self.cipher()?
                .decrypt(
                    XNonce::from_slice(&bytes[4..28]),
                    Payload {
                        msg: &bytes[28..],
                        aad: self.origin.as_bytes(),
                    },
                )
                .map_err(|_| "Tracker outbox authentication failed; existing data preserved.")?,
        );
        serde_json::from_slice(&plaintext)
            .map_err(|_| "Unreadable tracker outbox; existing data preserved.".to_string())
    }
    fn write(&self, state: &TrackerState) -> Result<(), String> {
        let plaintext = Zeroizing::new(serde_json::to_vec(state).map_err(|e| e.to_string())?);
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let encrypted = self
            .cipher()?
            .encrypt(
                &nonce,
                Payload {
                    msg: &plaintext,
                    aad: self.origin.as_bytes(),
                },
            )
            .map_err(|_| "Tracker encryption failed")?;
        let directory = self.lock_path.parent().ok_or("Missing tracker directory")?;
        let mut file = tempfile::NamedTempFile::new_in(directory).map_err(|e| e.to_string())?;
        file.write_all(b"CT01")
            .and_then(|_| file.write_all(&nonce))
            .and_then(|_| file.write_all(&encrypted))
            .and_then(|_| file.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        file.persist(self.lock_path.with_file_name("outbox.bin"))
            .map_err(|e| e.to_string())?;
        std::fs::File::open(directory)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
    pub fn state(&self) -> Result<TrackerState, String> {
        self.locked(|| {
            let mut state = self.read()?;
            let mut changed = false;
            for event in &mut state.pending {
                if event["kind"] == "agent_output"
                    && event["content"].as_str().is_some_and(|s| !s.is_empty())
                    && event["occurredAt"]
                        .as_str()
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .is_some_and(|t| {
                            chrono::Utc::now().signed_duration_since(t).num_hours() >= 72
                        })
                {
                    event["content"] = json!("");
                    changed = true;
                }
            }
            if changed {
                self.write(&state)?;
            }
            Ok(state)
        })
    }
    pub fn enroll(&self, workspace: &str, enabled: bool) -> Result<(), String> {
        self.locked(|| {
            let mut state = self.read()?;
            if state.workspaces.iter().any(|s| s == workspace) == enabled {
                return Ok(());
            }
            state.workspaces.retain(|s| s != workspace);
            if enabled {
                state.workspaces.push(workspace.to_string());
            }
            self.write(&state)
        })
    }
    /// Persist before dispatch. At capacity, preserve every unsynced event and expose the gap.
    pub fn capture(&self, mut event: Value) -> Result<(), String> {
        self.locked(|| {
            let mut state=self.read()?;
            let workspace=event["workspaceId"].as_str().ok_or("Tracker workspace is missing")?;
            if !state.workspaces.iter().any(|s| s==workspace) { return Ok(()); }
            event["id"]=json!(Uuid::new_v4().to_string());
            event["instanceId"]=json!(state.instance_id);
            event["sequence"]=json!(state.next_sequence);
            event["occurredAt"]=json!(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
            let content=event["content"].as_str().unwrap_or("");
            if content.len()>PROMPT_LIMIT {
                event["kind"]=json!("gap"); event["content"]=json!(""); event["coverage"]=json!("oversize_input"); state.gap_count+=1;
            }
            event["sourceDigest"] = json!(format!("{:x}", Sha256::digest(event["content"].as_str().unwrap_or("").as_bytes())));
            let size=serde_json::to_vec(&state).map_err(|e|e.to_string())?.len()+serde_json::to_vec(&event).map_err(|e|e.to_string())?.len();
            if size>OUTBOX_LIMIT {
                state.last_error=Some("Recording degraded: encrypted outbox capacity reached; unsynced events preserved.".to_string()); state.gap_count+=1; self.write(&state)?;
                return Err(state.last_error.unwrap_or_default());
            }
            state.next_sequence+=1; state.pending.push(event); self.write(&state)
        })
    }
    pub fn acknowledge(&self, id: &str) -> Result<(), String> {
        self.locked(|| {
            let mut state = self.read()?;
            state.pending.retain(|e| e["id"].as_str() != Some(id));
            if state.pending.is_empty() {
                state.last_error = None;
            }
            self.write(&state)
        })
    }
    pub fn failure(&self, message: &str) -> Result<(), String> {
        self.locked(|| {
            let mut state = self.read()?;
            state.last_error = Some(message.chars().take(300).collect());
            self.write(&state)
        })
    }
    pub fn sync(&self, client: &Client, api_key: &str) -> Result<usize, String> {
        let mut count = 0;
        for mut event in self.state()?.pending.into_iter().take(20) {
            let id = event["id"]
                .as_str()
                .ok_or("Invalid saved activity")?
                .to_string();
            // Never mutate a submitted event's idempotency payload. Expired output needs
            // a server disposition while its original ID/digest remain stable.
            let expired = event["kind"] == "agent_output"
                && event["occurredAt"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .is_some_and(|t| chrono::Utc::now().signed_duration_since(t).num_hours() >= 72);
            if expired {
                event["content"] = json!("");
            }
            let body = json!({"event":event,"apiKey":api_key});
            let response = client
                .request_blocking(reqwest::Method::POST, &format!("{API}/events"), Some(&body))
                .map_err(|e| {
                    let message = e.to_string();
                    let _ = self.failure(
                        "Tracker service unavailable; encrypted activity retained for retry.",
                    );
                    message
                })?;
            if !response.is_ok() {
                let error = response.message();
                self.failure(&error)?;
                return Err(error);
            }
            if response.body["summaryState"] == "pending" {
                self.failure("Output summary pending; encrypted source retained for retry.")?;
                continue;
            }
            self.acknowledge(&id)?;
            count += 1;
        }
        Ok(count)
    }
}

pub fn workspace_id(path: &Path) -> String {
    format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes()))
}
/// Partition UTF-8 without dropping text; each bounded output window gets its own summary.
pub fn output_chunks(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let mut end = rest.len().min(OUTPUT_LIMIT);
        while !rest.is_char_boundary(end) {
            end -= 1;
        }
        result.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    result
}
/// Only known credential values are removed; no semantic regex filtering on an LLM path.
pub fn redact_value(value: &mut Value, secrets: &[String]) {
    match value {
        Value::String(text) => *text = redact(text, secrets),
        Value::Array(values) => values.iter_mut().for_each(|v| redact_value(v, secrets)),
        Value::Object(values) => values.values_mut().for_each(|v| redact_value(v, secrets)),
        _ => {}
    }
}
pub fn redact(text: &str, secrets: &[String]) -> String {
    let mut result = text.to_string();
    for secret in secrets.iter().filter(|s| s.len() >= 8) {
        result = result.replace(secret, "[credential removed]");
    }
    result
}

#[cfg(test)]
#[path = "tracker_tests.rs"]
mod tests;
