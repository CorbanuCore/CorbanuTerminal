use std::collections::HashMap;
use std::sync::Mutex;

use pretty_assertions::assert_eq;

use super::*;

#[derive(Default)]
struct MemoryStore {
    entries: Mutex<HashMap<String, String>>,
}

impl MemoryStore {
    fn seed_raw(&self, label: &str, secret: String) {
        self.entries
            .lock()
            .expect("memory store lock")
            .insert(label.to_string(), secret);
    }
}

impl SessionStore for MemoryStore {
    fn reveal_optional(&self, label: &str) -> Result<Option<String>, SessionStoreError> {
        Ok(self
            .entries
            .lock()
            .expect("memory store lock")
            .get(label)
            .cloned())
    }

    fn upsert(
        &self,
        label: &str,
        secret: String,
        _notes: &str,
        _origin: &str,
    ) -> Result<(), SessionStoreError> {
        self.entries
            .lock()
            .expect("memory store lock")
            .insert(label.to_string(), secret);
        Ok(())
    }

    fn delete(&self, label: &str) -> Result<bool, SessionStoreError> {
        Ok(self
            .entries
            .lock()
            .expect("memory store lock")
            .remove(label)
            .is_some())
    }
}

fn active(token: &str) -> ActiveSession {
    active_for("tester", token)
}

fn active_for(username: &str, token: &str) -> ActiveSession {
    ActiveSession {
        origin: "https://tasknode.example".to_string(),
        account_id: Some("acct_1".to_string()),
        github_username: Some(username.to_string()),
        terminal_token: token.to_string(),
        expires_at: Some("2027-01-01T00:00:00Z".to_string()),
    }
}

#[test]
fn named_profiles_keep_active_and_pending_sessions_isolated() {
    let store = MemoryStore::default();
    let goodalexander = SessionScope::for_profile("goodalexander");
    let secondfoundation = SessionScope::for_profile("secondfoundation");

    promote_active_scoped_to_store(
        &store,
        &goodalexander,
        &active_for("goodalexander", "tok-goodalexander"),
    )
    .expect("seed goodalexander");
    promote_active_scoped_to_store(
        &store,
        &secondfoundation,
        &active_for("secondfoundation", "tok-secondfoundation"),
    )
    .expect("seed secondfoundation");
    save_pending_scoped_to_store(&store, &goodalexander, &pending("req-goodalexander"))
        .expect("pending goodalexander relink");

    let goodalexander_state = load_scoped_from_store(&store, &goodalexander).expect("load first");
    let secondfoundation_state =
        load_scoped_from_store(&store, &secondfoundation).expect("load second");

    assert_eq!(
        goodalexander_state
            .active
            .map(|session| session.terminal_token),
        Some("tok-goodalexander".to_string())
    );
    assert_eq!(
        goodalexander_state.pending.map(|link| link.request_id),
        Some("req-goodalexander".to_string())
    );
    assert_eq!(
        secondfoundation_state
            .active
            .map(|session| session.terminal_token),
        Some("tok-secondfoundation".to_string())
    );
    assert_eq!(secondfoundation_state.pending, None);

    clear_all_scoped_from_store(&store, &goodalexander).expect("unlink first profile");
    assert_eq!(
        load_scoped_from_store(&store, &secondfoundation)
            .expect("reload second")
            .active
            .map(|session| session.terminal_token),
        Some("tok-secondfoundation".to_string()),
        "unlinking one profile must not disturb another profile"
    );
}

#[test]
fn named_profile_never_imports_mismatched_legacy_identity() {
    let store = MemoryStore::default();
    promote_active_to_store(
        &store,
        &active_for("secondfoundation", "tok-secondfoundation"),
    )
    .expect("seed legacy global session");

    let state = load_scoped_from_store(&store, &SessionScope::for_profile("goodalexander"))
        .expect("load goodalexander");

    assert_eq!(state, LocalState::default());
    assert_eq!(
        load_from_store(&store)
            .expect("legacy session remains")
            .active
            .map(|session| session.terminal_token),
        Some("tok-secondfoundation".to_string())
    );
}

#[test]
fn matching_named_profile_imports_legacy_identity_once() {
    let store = MemoryStore::default();
    promote_active_to_store(
        &store,
        &active_for("SecondFoundation", "tok-secondfoundation"),
    )
    .expect("seed legacy global session");
    let scope = SessionScope::for_profile("secondfoundation");

    let imported = load_scoped_from_store(&store, &scope).expect("import matching session");
    assert_eq!(
        imported.active.map(|session| session.terminal_token),
        Some("tok-secondfoundation".to_string())
    );

    clear_all_from_store(&store).expect("clear legacy state");
    assert_eq!(
        load_scoped_from_store(&store, &scope)
            .expect("profile copy remains")
            .active
            .map(|session| session.terminal_token),
        Some("tok-secondfoundation".to_string())
    );
}

fn pending(request_id: &str) -> PendingLink {
    PendingLink {
        origin: "https://tasknode.example".to_string(),
        request_id: request_id.to_string(),
        poll_token: "poll-secret".to_string(),
        verification_url: format!("https://tasknode.example/auth/{request_id}"),
        started_at: Some("2026-08-07T00:00:00Z".to_string()),
    }
}

/// Regression for the incident class in the failure analysis: beginning a
/// replacement link must not destroy the currently usable credential.
#[test]
fn active_session_survives_link_start() {
    let store = MemoryStore::default();
    promote_active_to_store(&store, &active("tok-live")).expect("seed active");

    save_pending_to_store(&store, &pending("req-1")).expect("start link");

    let state = load_from_store(&store).expect("load");
    assert_eq!(
        state.active.as_ref().map(|s| s.terminal_token.as_str()),
        Some("tok-live"),
        "active token must survive a link start"
    );
    assert_eq!(
        state.pending.as_ref().map(|p| p.request_id.as_str()),
        Some("req-1")
    );
}

#[test]
fn abandoned_link_is_non_destructive() {
    let store = MemoryStore::default();
    promote_active_to_store(&store, &active("tok-live")).expect("seed active");
    save_pending_to_store(&store, &pending("req-1")).expect("start link");

    assert!(clear_pending_from_store(&store).expect("clear pending"));

    let state = load_from_store(&store).expect("load");
    assert_eq!(
        state.active.map(|s| s.terminal_token),
        Some("tok-live".to_string())
    );
    assert_eq!(state.pending, None);
}

#[test]
fn promotion_replaces_active_and_clears_pending() {
    let store = MemoryStore::default();
    promote_active_to_store(&store, &active("tok-old")).expect("seed active");
    save_pending_to_store(&store, &pending("req-2")).expect("start link");

    promote_active_to_store(&store, &active("tok-new")).expect("promote");

    let state = load_from_store(&store).expect("load");
    assert_eq!(
        state.active.map(|s| s.terminal_token),
        Some("tok-new".to_string())
    );
    assert_eq!(
        state.pending, None,
        "promotion consumes the pending attempt"
    );
}

/// Pre-split blobs with a token load as an active session unchanged.
#[test]
fn legacy_active_record_loads() {
    let store = MemoryStore::default();
    let legacy = serde_json::json!({
        "origin": "https://tasknode.example",
        "account_id": "acct_9",
        "github_username": "legacy-user",
        "terminal_token": "tok-legacy",
        "expires_at": null,
        "pending_request_id": null,
        "pending_poll_token": null,
        "pending_verification_url": null,
    });
    store.seed_raw(TASKNODE_ACTIVE_SESSION_LABEL, legacy.to_string());

    let state = load_from_store(&store).expect("load");
    assert_eq!(
        state.active.map(|s| (s.terminal_token, s.account_id)),
        Some(("tok-legacy".to_string(), Some("acct_9".to_string())))
    );
    assert_eq!(state.pending, None);
}

/// A pre-split pending-only blob (the corrupted state the old flow produced)
/// migrates to the pending label and stops occupying the active label.
#[test]
fn legacy_pending_only_record_migrates() {
    let store = MemoryStore::default();
    let legacy = serde_json::json!({
        "origin": "https://tasknode.example",
        "terminal_token": null,
        "pending_request_id": "req-legacy",
        "pending_poll_token": "poll-legacy",
        "pending_verification_url": "https://tasknode.example/auth/req-legacy",
    });
    store.seed_raw(TASKNODE_ACTIVE_SESSION_LABEL, legacy.to_string());

    let state = load_from_store(&store).expect("load");
    assert_eq!(state.active, None);
    assert_eq!(
        state.pending.map(|p| (p.request_id, p.poll_token)),
        Some(("req-legacy".to_string(), "poll-legacy".to_string()))
    );

    // A new active session can now be installed cleanly.
    promote_active_to_store(&store, &active("tok-fresh")).expect("promote");
    let state = load_from_store(&store).expect("reload");
    assert_eq!(
        state.active.map(|s| s.terminal_token),
        Some("tok-fresh".to_string())
    );
    assert_eq!(state.pending, None);
}

#[test]
fn clear_all_unlinks_everything() {
    let store = MemoryStore::default();
    promote_active_to_store(&store, &active("tok")).expect("seed");
    save_pending_to_store(&store, &pending("req")).expect("pending");

    clear_all_from_store(&store).expect("clear");

    assert_eq!(
        load_from_store(&store).expect("load"),
        LocalState::default()
    );
}

#[test]
fn state_summary_never_contains_secrets() {
    let store = MemoryStore::default();
    promote_active_to_store(&store, &active("tok-secret-value")).expect("seed");
    save_pending_to_store(&store, &pending("req-1")).expect("pending");

    let summary = state_summary(&load_from_store(&store).expect("load")).to_string();
    assert!(!summary.contains("tok-secret-value"));
    assert!(!summary.contains("poll-secret"));
    assert!(summary.contains("req-1"));
}

/// The field scenario from 2026-08-07: a daily-TTL-expired active session must
/// not be treated as usable, and must not block completing a pending link.
#[test]
fn expired_active_session_is_detected() {
    let mut session = active("tok-expired");
    session.expires_at = Some("2026-08-07T13:07:07.100Z".to_string());
    let after = chrono::DateTime::parse_from_rfc3339("2026-08-07T17:00:00Z")
        .expect("parse")
        .with_timezone(&chrono::Utc);
    let before = chrono::DateTime::parse_from_rfc3339("2026-08-07T10:00:00Z")
        .expect("parse")
        .with_timezone(&chrono::Utc);
    assert!(session.is_expired_at(after));
    assert!(!session.is_expired_at(before));
}

/// Missing or malformed expiry metadata must never lock a user out.
#[test]
fn absent_or_invalid_expiry_counts_as_fresh() {
    let now = chrono::Utc::now();
    let mut session = active("tok");
    session.expires_at = None;
    assert!(!session.is_expired_at(now));
    session.expires_at = Some("not-a-date".to_string());
    assert!(!session.is_expired_at(now));
}

#[test]
fn state_summary_reports_expiry() {
    let store = MemoryStore::default();
    let mut session = active("tok");
    session.expires_at = Some("2000-01-01T00:00:00Z".to_string());
    promote_active_to_store(&store, &session).expect("seed");
    let summary = state_summary(&load_from_store(&store).expect("load"));
    assert_eq!(
        summary
            .get("activeSession")
            .and_then(|active| active.get("expired"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}
