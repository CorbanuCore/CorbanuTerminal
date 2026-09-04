//! Task Node terminal-session state, shared by the TUI (`/tasknode`) and the
//! JSON helper CLI (`pfterminal tasknode …`).
//!
//! # Why this crate exists
//!
//! The link flow was previously implemented twice (TUI and CLI) around a
//! single mutable vault record, and starting a relink wrote a token-less
//! "pending" record over the active credential — destroying a working session
//! before the replacement existed. The full incident write-up lives in
//! `docs/archive/2026/TASKNODE_GITHUB_LINK_FAILURE_ANALYSIS.md`.
//!
//! This crate makes that failure unrepresentable at the storage layer:
//!
//! - **Active and pending state live under different vault labels.** Starting
//!   a link preserves usable active state and records the pending attempt under
//!   [`TASKNODE_PENDING_LINK_LABEL`]. Durable unlink intent suppresses legacy
//!   reimport; relinking clears only already-revoked credential residue. The active
//!   session under [`TASKNODE_ACTIVE_SESSION_LABEL`] is written by exactly one
//!   operation: [`promote_active`], which callers invoke only after the
//!   replacement token has been validated against the server.
//! - **Both surfaces share one resolver.** [`load`] returns the same
//!   [`LocalState`] to the TUI and the CLI, so the two can no longer disagree
//!   about whether a session exists.
//! - **Legacy single-record state migrates transparently.** Older builds may
//!   have left a pending-only record under the active label; [`load`] moves it
//!   to the pending label so a valid active session can be restored without
//!   fighting the old blob.

use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::Vault;
use codex_vault::VaultError;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

/// Vault label holding the active terminal session (bearer token). Kept at the
/// pre-split value so existing linked installations keep working unchanged.
pub const TASKNODE_ACTIVE_SESSION_LABEL: &str = "tasknode/session";
/// Vault label holding an in-flight GitHub link attempt. Never contains a
/// usable bearer token.
pub const TASKNODE_PENDING_LINK_LABEL: &str = "tasknode/link-pending";

/// Selects the local Corbanu profile that owns a Task Node session.
///
/// Named profiles must never share bearer authority. Their vault labels use a
/// stable hash of the profile name so every valid profile name fits the vault
/// label contract and cannot inject label separators. The default scope keeps
/// the legacy labels for installations that do not use profiles.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionScope {
    profile: Option<String>,
}

impl SessionScope {
    pub fn for_profile(profile: impl Into<String>) -> Self {
        Self {
            profile: Some(profile.into()),
        }
    }

    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    fn label(&self, legacy_label: &str, suffix: &str) -> String {
        let Some(profile) = self.profile.as_deref() else {
            return legacy_label.to_string();
        };
        let digest = Sha256::digest(profile.as_bytes());
        let fingerprint = format!("{digest:x}");
        format!("tasknode/profiles/{}/{suffix}", &fingerprint[..32])
    }

    fn active_label(&self) -> String {
        self.label(TASKNODE_ACTIVE_SESSION_LABEL, "session")
    }

    fn pending_label(&self) -> String {
        self.label(TASKNODE_PENDING_LINK_LABEL, "link-pending")
    }

    fn lifecycle_label(&self) -> String {
        self.label("tasknode/session-lifecycle", "session-lifecycle")
    }
}

/// Persistent local intent outlives credential deletion and legacy migration.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SessionLifecycle {
    Linked,
    Unlinked,
}

fn load_lifecycle<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
) -> Result<Option<SessionLifecycle>, SessionStoreError> {
    store
        .reveal_optional(&scope.lifecycle_label())?
        .map(|raw| {
            serde_json::from_str(&raw)
                .map_err(|err| SessionStoreError::Corrupt(format!("session lifecycle: {err}")))
        })
        .transpose()
}

fn save_lifecycle<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
    lifecycle: SessionLifecycle,
) -> Result<(), SessionStoreError> {
    let value = serde_json::to_string(&lifecycle)
        .map_err(|err| SessionStoreError::Corrupt(format!("session lifecycle: {err}")))?;
    store.upsert(
        &scope.lifecycle_label(),
        value,
        "Local Task Node link intent; contains no credential.",
        "",
    )
}

/// A proven, usable terminal session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveSession {
    pub origin: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub github_username: Option<String>,
    pub terminal_token: String,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// An in-flight link attempt: authority to poll, not authority to act.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingLink {
    pub origin: String,
    pub request_id: String,
    pub poll_token: String,
    pub verification_url: String,
    #[serde(default)]
    pub started_at: Option<String>,
}

/// Combined local state as both surfaces must see it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalState {
    pub active: Option<ActiveSession>,
    pub pending: Option<PendingLink>,
}

/// Pre-split record shape: one blob that was either an active session or a
/// pending attempt depending on which fields were populated.
#[derive(Debug, Deserialize)]
struct LegacyRecord {
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    github_username: Option<String>,
    #[serde(default)]
    terminal_token: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    pending_request_id: Option<String>,
    #[serde(default)]
    pending_poll_token: Option<String>,
    #[serde(default)]
    pending_verification_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("vault error: {0}")]
    Vault(String),
    #[error("invalid local Task Node session state: {0}")]
    Corrupt(String),
}

impl From<VaultError> for SessionStoreError {
    fn from(err: VaultError) -> Self {
        Self::Vault(err.to_string())
    }
}

/// Load combined state, transparently migrating legacy single-record blobs.
///
/// Migration is deliberately conservative: a legacy pending-only blob under
/// the active label is moved to the pending label (unless a newer pending
/// attempt already exists) and the active label is cleared, because that blob
/// never contained usable authority in the first place.
pub fn load(vault: &Vault) -> Result<LocalState, SessionStoreError> {
    load_from_store(vault)
}

/// Load only the Task Node state owned by `scope`.
///
/// A named profile may import the legacy global active session once, but only
/// when the session's GitHub username matches the profile name. A mismatched
/// global session is never returned to the named profile.
pub fn load_scoped(vault: &Vault, scope: &SessionScope) -> Result<LocalState, SessionStoreError> {
    load_scoped_from_store(vault, scope)
}

fn load_from_store<S: SessionStore + ?Sized>(store: &S) -> Result<LocalState, SessionStoreError> {
    load_scoped_from_store(store, &SessionScope::default())
}

fn load_scoped_from_store<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
) -> Result<LocalState, SessionStoreError> {
    let mut state = LocalState::default();
    let lifecycle = load_lifecycle(store, scope)?;
    if lifecycle == Some(SessionLifecycle::Unlinked) {
        return Ok(state);
    }
    let pending_label = scope.pending_label();
    let active_label = scope.active_label();

    if let Some(raw) = store.reveal_optional(&pending_label)? {
        let pending: PendingLink = serde_json::from_str(&raw)
            .map_err(|err| SessionStoreError::Corrupt(format!("pending link: {err}")))?;
        state.pending = Some(pending);
    }

    if let Some(raw) = store.reveal_optional(&active_label)? {
        let record: LegacyRecord = serde_json::from_str(&raw)
            .map_err(|err| SessionStoreError::Corrupt(format!("active session: {err}")))?;
        let origin = record.origin.unwrap_or_default();
        match record.terminal_token {
            Some(token) if !token.trim().is_empty() => {
                state.active = Some(ActiveSession {
                    origin,
                    account_id: record.account_id,
                    github_username: record.github_username,
                    terminal_token: token,
                    expires_at: record.expires_at,
                });
            }
            _ => {
                // Legacy pending-only blob written by the pre-split link flow.
                if state.pending.is_none()
                    && let (Some(request_id), Some(poll_token)) =
                        (record.pending_request_id, record.pending_poll_token)
                {
                    let pending = PendingLink {
                        origin,
                        request_id,
                        poll_token,
                        verification_url: record.pending_verification_url.unwrap_or_default(),
                        started_at: None,
                    };
                    save_pending_scoped_to_store(store, scope, &pending)?;
                    state.pending = Some(pending);
                }
                // Either way the active label holds no authority; clear it so
                // a real session can be written cleanly later.
                let _ = store.delete(&active_label);
            }
        }
    }

    if state.active.is_none()
        && state.pending.is_none()
        && lifecycle.is_none()
        && let Some(profile) = scope.profile()
    {
        let legacy = load_from_store(store)?;
        if let Some(active) = legacy.active
            && active
                .github_username
                .as_deref()
                .is_some_and(|username| username.eq_ignore_ascii_case(profile))
        {
            promote_active_scoped_to_store(store, scope, &active)?;
            state.active = Some(active);
        }
    }

    Ok(state)
}

/// Record a new link attempt. Preserves usable active state; removes residual
/// credentials suppressed by a previous unlink before reopening the scope.
pub fn save_pending(vault: &Vault, pending: &PendingLink) -> Result<(), SessionStoreError> {
    save_pending_to_store(vault, pending)
}

pub fn save_pending_scoped(
    vault: &Vault,
    scope: &SessionScope,
    pending: &PendingLink,
) -> Result<(), SessionStoreError> {
    save_pending_scoped_to_store(vault, scope, pending)
}

fn save_pending_to_store<S: SessionStore + ?Sized>(
    store: &S,
    pending: &PendingLink,
) -> Result<(), SessionStoreError> {
    save_pending_scoped_to_store(store, &SessionScope::default(), pending)
}

fn save_pending_scoped_to_store<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
    pending: &PendingLink,
) -> Result<(), SessionStoreError> {
    if load_lifecycle(store, scope)? == Some(SessionLifecycle::Unlinked) {
        // A prior failed unlink cleanup must not regain authority on relink.
        store.delete(&scope.active_label())?;
    }
    let secret = serde_json::to_string(pending)
        .map_err(|err| SessionStoreError::Corrupt(format!("serialize pending link: {err}")))?;
    store.upsert(
        &scope.pending_label(),
        secret,
        "Task Node link attempt in progress; holds no usable token.",
        &pending.origin,
    )?;
    save_lifecycle(store, scope, SessionLifecycle::Linked)
}

/// Install a validated replacement session, then clear the pending
/// attempt. Callers must have proven the token against the server first (a
/// successful authenticated `status` call); this function is the only writer
/// of the active label.
pub fn promote_active(vault: &Vault, session: &ActiveSession) -> Result<(), SessionStoreError> {
    promote_active_to_store(vault, session)
}

pub fn promote_active_scoped(
    vault: &Vault,
    scope: &SessionScope,
    session: &ActiveSession,
) -> Result<(), SessionStoreError> {
    promote_active_scoped_to_store(vault, scope, session)
}

fn promote_active_to_store<S: SessionStore + ?Sized>(
    store: &S,
    session: &ActiveSession,
) -> Result<(), SessionStoreError> {
    promote_active_scoped_to_store(store, &SessionScope::default(), session)
}

fn promote_active_scoped_to_store<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
    session: &ActiveSession,
) -> Result<(), SessionStoreError> {
    let secret = serde_json::to_string(session)
        .map_err(|err| SessionStoreError::Corrupt(format!("serialize session: {err}")))?;
    store.upsert(
        &scope.active_label(),
        secret,
        "Task Node terminal session; token is not printed to chat.",
        &session.origin,
    )?;
    store.delete(&scope.pending_label())?;
    save_lifecycle(store, scope, SessionLifecycle::Linked)
}

/// Abandon an in-flight link attempt. The active session is untouched.
pub fn clear_pending(vault: &Vault) -> Result<bool, SessionStoreError> {
    clear_pending_from_store(vault)
}

pub fn clear_pending_scoped(
    vault: &Vault,
    scope: &SessionScope,
) -> Result<bool, SessionStoreError> {
    clear_pending_scoped_from_store(vault, scope)
}

fn clear_pending_from_store<S: SessionStore + ?Sized>(
    store: &S,
) -> Result<bool, SessionStoreError> {
    clear_pending_scoped_from_store(store, &SessionScope::default())
}

fn clear_pending_scoped_from_store<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
) -> Result<bool, SessionStoreError> {
    store.delete(&scope.pending_label())
}

/// Unlink Task Node, retaining a durable marker that prevents legacy reimport.
pub fn clear_all(vault: &Vault) -> Result<(), SessionStoreError> {
    clear_all_from_store(vault)
}

pub fn clear_all_scoped(vault: &Vault, scope: &SessionScope) -> Result<(), SessionStoreError> {
    clear_all_scoped_from_store(vault, scope)
}

fn clear_all_from_store<S: SessionStore + ?Sized>(store: &S) -> Result<(), SessionStoreError> {
    clear_all_scoped_from_store(store, &SessionScope::default())
}

fn clear_all_scoped_from_store<S: SessionStore + ?Sized>(
    store: &S,
    scope: &SessionScope,
) -> Result<(), SessionStoreError> {
    // Commit revocation before cleanup. A failed delete must neither silently
    // succeed nor let a saved token or pending link become usable on reload.
    save_lifecycle(store, scope, SessionLifecycle::Unlinked)?;
    let pending = store.delete(&scope.pending_label());
    let active = store.delete(&scope.active_label());
    pending?;
    active?;
    Ok(())
}

/// Non-secret diagnostic view of local state; safe to print anywhere.
pub fn state_summary(state: &LocalState) -> serde_json::Value {
    serde_json::json!({
        "activeSession": state.active.as_ref().map(|active| serde_json::json!({
            "origin": active.origin,
            "accountId": active.account_id,
            "githubUsername": active.github_username,
            "expiresAt": active.expires_at,
            "expired": active.is_expired(),
        })),
        "pendingLink": state.pending.as_ref().map(|pending| serde_json::json!({
            "origin": pending.origin,
            "requestId": pending.request_id,
            "verificationUrl": pending.verification_url,
            "startedAt": pending.started_at,
        })),
    })
}

/// `POST /api/auth/terminal/start/github` response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalAuthStart {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "pollToken")]
    pub poll_token: String,
    #[serde(rename = "verificationUrl")]
    pub verification_url: String,
    #[serde(rename = "expiresAt", default)]
    pub expires_at: Option<String>,
}

/// `GET /api/auth/terminal/session` success response.
#[derive(Clone, Debug, Deserialize)]
pub struct TerminalSessionIssued {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "githubUsername", default)]
    pub github_username: Option<String>,
    #[serde(rename = "terminalToken")]
    pub terminal_token: String,
    #[serde(rename = "expiresAt", default)]
    pub expires_at: Option<String>,
}

impl ActiveSession {
    /// Whether the server-provided expiry has passed at `now`.
    ///
    /// A missing or unparsable expiry counts as "not expired": the server is
    /// the authority, and guessing a session dead when the metadata is absent
    /// would lock users out of a working session.
    pub fn is_expired_at(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        self.expires_at
            .as_deref()
            .and_then(|raw| chrono::DateTime::parse_from_rfc3339(raw).ok())
            .is_some_and(|expires_at| expires_at <= now)
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_at(chrono::Utc::now())
    }

    pub fn from_issued(origin: String, issued: TerminalSessionIssued) -> Self {
        Self {
            origin,
            account_id: Some(issued.account_id),
            github_username: issued.github_username,
            terminal_token: issued.terminal_token,
            expires_at: issued.expires_at,
        }
    }
}

trait SessionStore {
    fn reveal_optional(&self, label: &str) -> Result<Option<String>, SessionStoreError>;
    fn upsert(
        &self,
        label: &str,
        secret: String,
        notes: &str,
        origin: &str,
    ) -> Result<(), SessionStoreError>;
    fn delete(&self, label: &str) -> Result<bool, SessionStoreError>;
}

impl SessionStore for Vault {
    fn reveal_optional(&self, label: &str) -> Result<Option<String>, SessionStoreError> {
        match self.reveal(label) {
            Ok(secret) => Ok(Some(secret)),
            Err(VaultError::NotFound { .. }) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn upsert(
        &self,
        label: &str,
        secret: String,
        notes: &str,
        origin: &str,
    ) -> Result<(), SessionStoreError> {
        match self.add(AddCredential {
            label: label.to_string(),
            credential_type: CredentialType::BearerToken,
            provider: Some("tasknode".to_string()),
            notes: Some(notes.to_string()),
            revocation_notes: Some(format!("{origin}/settings/accounts")),
            secret: secret.clone(),
        }) {
            Ok(()) => Ok(()),
            Err(VaultError::CredentialExists { .. }) => self
                .update(
                    label,
                    Some(secret),
                    Some(Some("tasknode".to_string())),
                    /*notes*/ None,
                    /*revocation_notes*/ None,
                )
                .map(|_| ())
                .map_err(Into::into),
            Err(err) => Err(err.into()),
        }
    }

    fn delete(&self, label: &str) -> Result<bool, SessionStoreError> {
        Vault::delete(self, label).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests;
