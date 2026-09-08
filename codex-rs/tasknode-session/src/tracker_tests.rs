use super::*;
use crate::tests::MemoryStore;
use pretty_assertions::assert_eq;

fn fixture() -> (tempfile::TempDir, TrackerStore) {
    let home = tempfile::tempdir().unwrap();
    let store = TrackerStore {
        store: Arc::new(MemoryStore::default()),
        lock_path: home.path().join("tracker.lock"),
        origin: "https://tracker.example".to_string(),
        key: Arc::new(OnceLock::new()),
    };
    (home, store)
}
fn event(content: &str) -> Value {
    json!({"workspaceId":"repo", "sessionId":"thread", "turnId":"turn", "kind":"human_prompt", "content":content})
}
#[test]
fn enrollment_recovery_and_sequence_preserve_identical_prompts_as_distinct_actions() {
    let (_home, store) = fixture();
    store.capture(event("Investigate the worker")).unwrap();
    assert!(store.state().unwrap().pending.is_empty());
    store.enroll("repo", true).unwrap();
    store
        .capture(event("Investigate the worker\nThen reproduce it."))
        .unwrap();
    store
        .capture(event("Investigate the worker\nThen reproduce it."))
        .unwrap();
    let reopened = TrackerStore {
        store: store.store.clone(),
        lock_path: store.lock_path.clone(),
        origin: store.origin.clone(),
        key: Arc::new(OnceLock::new()),
    };
    let ciphertext = std::fs::read(reopened.lock_path.with_file_name("outbox.bin")).unwrap();
    assert!(!String::from_utf8_lossy(&ciphertext).contains("Investigate"));
    let state = reopened.state().unwrap();
    assert_eq!(state.pending.len(), 2);
    assert_eq!(
        state.pending[0]["content"],
        json!("Investigate the worker\nThen reproduce it.")
    );
    assert_ne!(state.pending[0]["id"], state.pending[1]["id"]);
    assert_eq!(state.pending[1]["sequence"], json!(1));
    reopened
        .acknowledge(state.pending[0]["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        reopened.state().unwrap().pending,
        vec![state.pending[1].clone()]
    );
    reopened.enroll("repo", false).unwrap();
    reopened.capture(event("This workspace is paused")).unwrap();
    assert_eq!(reopened.state().unwrap().pending.len(), 1);
}
#[test]
fn tampered_outbox_fails_closed_and_preserves_ciphertext() {
    let (_home, store) = fixture();
    store.enroll("repo", true).unwrap();
    store.capture(event("Preserve my evidence")).unwrap();
    let path = store.lock_path.with_file_name("outbox.bin");
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    std::fs::write(&path, &bytes).unwrap();
    assert!(store.state().unwrap_err().contains("authentication failed"));
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[test]
fn unicode_chunking_and_secret_redaction_are_lossless_except_known_credentials() {
    let output = "界🙂\n".repeat(20_000);
    let chunks = output_chunks(&output);
    assert!(chunks.iter().all(|chunk| chunk.len() <= OUTPUT_LIMIT));
    assert_eq!(chunks.concat(), output);
    assert_eq!(
        redact("use key_abcd1234 safely", &["key_abcd1234".to_string()]),
        "use [credential removed] safely"
    );
}
#[test]
fn expired_source_is_removed_without_changing_its_digest_or_event_identity() {
    let (_home, store) = fixture();
    store.enroll("repo", true).unwrap();
    let mut output = event("Temporary output");
    output["kind"] = json!("agent_output");
    store.capture(output).unwrap();
    let mut state = store.state().unwrap();
    state.pending[0]["occurredAt"] = json!("2020-01-01T00:00:00Z");
    let original = state.pending[0].clone();
    store.write(&state).unwrap();
    let expired = store.state().unwrap().pending.remove(0);
    assert_eq!(expired["content"], json!(""));
    assert_eq!(expired["id"], original["id"]);
    assert_eq!(expired["sourceDigest"], original["sourceDigest"]);
}
#[test]
fn account_profile_and_origin_select_different_encrypted_outboxes() {
    let home = tempfile::tempdir().unwrap();
    let mut session = ActiveSession {
        origin: "https://tracker.example".to_string(),
        account_id: Some("alice".to_string()),
        github_username: None,
        terminal_token: "fixture".to_string(),
        expires_at: None,
    };
    let a = TrackerStore::new(home.path(), &SessionScope::default(), &session)
        .unwrap()
        .lock_path;
    let b = TrackerStore::new(home.path(), &SessionScope::for_profile("other"), &session)
        .unwrap()
        .lock_path;
    session.account_id = Some("bob".to_string());
    let c = TrackerStore::new(home.path(), &SessionScope::default(), &session)
        .unwrap()
        .lock_path;
    session.origin = "https://other.example".to_string();
    let d = TrackerStore::new(home.path(), &SessionScope::default(), &session)
        .unwrap()
        .lock_path;
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(c, d);
}
