//! Encrypted, profile/account/origin-scoped command receipts across restarts.
use crate::ActiveSession;
use crate::SessionScope;
use crate::SessionStore;
use crate::normalize_origin;
use codex_vault::Vault;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskCommand {
    pub key: String,
    pub detail: String,
    pub kind: String,
    pub created_at: String,
    pub request_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::MemoryStore;

    #[test]
    fn uncertain_retry_survives_reopen_and_concurrent_saves_without_crossing_accounts() {
        let home = tempfile::tempdir().unwrap();
        let memory = Arc::new(MemoryStore::default());
        let session = ActiveSession {
            origin: "https://tasknode.example".to_string(),
            account_id: Some("alice".to_string()),
            github_username: None,
            terminal_token: "fixture".to_string(),
            expires_at: None,
        };
        let make = |scope: SessionScope, session: &ActiveSession| {
            let mut store = CommandStore::new(home.path(), &scope, session).unwrap();
            store.store = memory.clone();
            store
        };
        let store = make(SessionScope::default(), &session);
        let first = store
            .begin("Investigate the task queue", "personal")
            .unwrap();
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let store = store.clone();
                std::thread::spawn(move || {
                    store
                        .begin("Investigate the task queue", "personal")
                        .unwrap()
                })
            })
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap().key, first.key);
        }
        let reopened = make(SessionScope::default(), &session);
        assert_eq!(reopened.list().unwrap(), vec![first.clone()]);
        assert!(
            make(SessionScope::for_profile("other"), &session)
                .list()
                .unwrap()
                .is_empty()
        );
        let mut other = session.clone();
        other.account_id = Some("bob".to_string());
        assert!(
            make(SessionScope::default(), &other)
                .list()
                .unwrap()
                .is_empty()
        );
        other = session;
        other.origin = "https://other.example".to_string();
        assert!(
            make(SessionScope::default(), &other)
                .list()
                .unwrap()
                .is_empty()
        );
        reopened
            .acknowledge(&first.key, &serde_json::json!({"requestId":"req_original"}))
            .unwrap();
        assert!(
            reopened
                .acknowledge(&first.key, &serde_json::json!({"requestId":"req_wrong"}))
                .is_err()
        );
        assert_ne!(
            reopened.begin(&first.detail, "personal").unwrap().key,
            first.key
        );
    }
}

impl TaskCommand {
    pub fn body(&self) -> Value {
        serde_json::json!({
            "userDetailText": self.detail, "requestedTaskKind": self.kind,
            "source": "pfterminal", "sourceConversationTitle": "Corbanu Terminal",
            "idempotencyKey": self.key,
        })
    }
}

#[derive(Clone)]
pub struct CommandStore {
    store: Arc<dyn SessionStore + Send + Sync>,
    label: String,
    origin: String,
    lock_path: PathBuf,
}

impl CommandStore {
    pub fn new(
        codex_home: &Path,
        scope: &SessionScope,
        session: &ActiveSession,
    ) -> Result<Self, String> {
        let account = session
            .account_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or("Relink Task Node to establish the account for saved requests.")?;
        let origin = normalize_origin(&session.origin).map_err(|error| error.to_string())?;
        let identity =
            serde_json::to_vec(&(account, &origin)).map_err(|error| error.to_string())?;
        let suffix = format!("commands/{:x}", Sha256::digest(identity));
        let label = scope.label(&format!("tasknode/{suffix}"), &suffix);
        let lock_name = format!(
            "tasknode-commands-{:x}.lock",
            Sha256::digest(label.as_bytes())
        );
        Ok(Self {
            store: Arc::new(Vault::new(codex_home.to_path_buf())),
            label,
            origin,
            lock_path: codex_home.join("secrets").join(lock_name),
        })
    }

    fn with_lock<T>(&self, operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
        let directory = self
            .lock_path
            .parent()
            .ok_or("Missing request storage directory")?;
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.lock_path)
            .map_err(|error| error.to_string())?;
        lock.lock().map_err(|error| error.to_string())?;
        operation()
    }

    fn read(&self) -> Result<Vec<TaskCommand>, String> {
        let raw = self
            .store
            .reveal_optional(&self.label)
            .map_err(|error| error.to_string())?;
        raw.map(|text| {
            serde_json::from_str(&text)
                .map_err(|error| format!("Saved Task Node requests are unreadable: {error}"))
        })
        .unwrap_or_else(|| Ok(Vec::new()))
    }

    fn save(&self, commands: &[TaskCommand]) -> Result<(), String> {
        self.store
            .upsert(
                &self.label,
                serde_json::to_string(commands).map_err(|error| error.to_string())?,
                "Task Node request recovery; encrypted input and immutable retry keys.",
                &self.origin,
            )
            .map_err(|error| error.to_string())
    }

    pub fn list(&self) -> Result<Vec<TaskCommand>, String> {
        self.with_lock(|| self.read())
    }

    /// Reuses an uncertain submission; a known receipt permits a new command.
    /// The separate process lock makes simultaneous CLI/TUI saves atomic.
    pub fn begin(&self, detail: &str, kind: &str) -> Result<TaskCommand, String> {
        let detail = detail.trim();
        if detail.is_empty() || detail.chars().count() > 8000 {
            return Err("Describe the task in at most 8,000 characters.".to_string());
        }
        self.with_lock(|| {
            let mut commands = self.read()?;
            if let Some(existing) = commands.iter().find(|command| {
                command.request_id.is_none() && command.detail == detail && command.kind == kind
            }) {
                return Ok(existing.clone());
            }
            if commands
                .iter()
                .filter(|command| command.request_id.is_none())
                .count()
                >= 50
            {
                return Err(
                    "Recover pending Task Node submissions before creating more requests."
                        .to_string(),
                );
            }
            let command = TaskCommand {
                key: format!("corbanu-request:{}", Uuid::new_v4()),
                detail: detail.to_string(),
                kind: kind.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                request_id: None,
            };
            while commands.len() >= 100 {
                let Some(index) = commands
                    .iter()
                    .position(|command| command.request_id.is_some())
                else {
                    break;
                };
                commands.remove(index);
            }
            commands.push(command.clone());
            self.save(&commands)?;
            Ok(command)
        })
    }

    pub fn acknowledge(&self, key: &str, response: &Value) -> Result<(), String> {
        let request_id = response.get("requestId").and_then(Value::as_str).filter(|value| !value.is_empty())
            .ok_or("Task Node returned no durable receipt. Recover this submission with its saved key.")?;
        self.with_lock(|| {
            let mut commands = self.read()?;
            let command = commands
                .iter_mut()
                .find(|command| command.key == key)
                .ok_or("Saved request key is missing")?;
            if let Some(existing) = &command.request_id
                && existing != request_id
            {
                return Err(
                    "Task Node returned conflicting receipts for one request key.".to_string(),
                );
            }
            command.request_id = Some(request_id.to_string());
            self.save(&commands)
        })
    }
}
