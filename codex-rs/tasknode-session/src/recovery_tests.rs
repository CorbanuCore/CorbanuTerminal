use super::*;
use crate::tests::MemoryStore;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::cell::Cell;

fn session(user: &str, token: &str) -> ActiveSession {
    ActiveSession {
        origin: "https://tasknode.example".into(),
        account_id: Some(format!("acct_{user}")),
        github_username: Some(user.into()),
        terminal_token: token.into(),
        expires_at: None,
    }
}

fn pending() -> PendingLink {
    PendingLink {
        origin: "https://tasknode.example".into(),
        request_id: "pending-desk".into(),
        poll_token: "fixture-poll".into(),
        verification_url: "https://tasknode.example/choose-account".into(),
        started_at: None,
    }
}

fn response(status: u16, body: serde_json::Value) -> Response {
    Response { status, body }
}

#[test]
fn completed_relink_replaces_nonexpiring_old_token_in_only_selected_profile() {
    for scope in [
        SessionScope::default(),
        SessionScope::for_profile("research-desk"),
    ] {
        let store = MemoryStore::default();
        let other = SessionScope::for_profile("other-desk");
        let old = session("alice", "revoked-but-no-expiry");
        let unaffected = session("bob", "bob-working-token");
        promote_active_scoped_to_store(&store, &scope, &old).unwrap();
        promote_active_scoped_to_store(&store, &other, &unaffected).unwrap();
        save_pending_scoped_to_store(&store, &scope, &pending()).unwrap();
        let calls = Cell::new(0);
        let resolved = resolve_from_store(&store, &scope, None, |_, path| {
            calls.set(calls.get() + 1);
            if path.starts_with("/api/auth/terminal/session?") {
                Ok(response(200, json!({"ok":true,"accountId":"acct_alice","githubUsername":"alice","terminalToken":"replacement"})))
            } else {
                assert_eq!(path, "/api/terminal/tasknode/status");
                Ok(response(200, json!({"ok":true,"accountId":"acct_alice"})))
            }
        }).unwrap();
        assert_eq!(resolved, session("alice", "replacement"));
        assert_eq!(calls.get(), 2);
        assert_eq!(
            load_scoped_from_store(&store, &scope).unwrap(),
            LocalState {
                active: Some(resolved.clone()),
                pending: None
            }
        );
        assert_eq!(
            load_scoped_from_store(&store, &other).unwrap(),
            LocalState {
                active: Some(unaffected),
                pending: None
            }
        );
        // A fresh resolver after restart must use exactly the promoted identity.
        assert_eq!(
            resolve_from_store(&store, &scope, None, |_, _| panic!(
                "no pending link to exchange"
            ))
            .unwrap(),
            resolved
        );
    }
}

#[test]
fn unfinished_link_preserves_working_authority_and_explains_rejected_authority() {
    for valid in [true, false] {
        let store = MemoryStore::default();
        let scope = SessionScope::for_profile("desk");
        let old = session("alice", "old-token");
        promote_active_scoped_to_store(&store, &scope, &old).unwrap();
        save_pending_scoped_to_store(&store, &scope, &pending()).unwrap();
        let before = load_scoped_from_store(&store, &scope).unwrap();
        let result = resolve_from_store(&store, &scope, None, |_, path| {
            Ok(if path.starts_with("/api/auth/") {
                response(202, json!({"error":"terminal_auth_pending"}))
            } else if valid {
                response(200, json!({"ok":true,"accountId":"acct_alice"}))
            } else {
                response(401, json!({"ok":false,"error":"terminal_login_required"}))
            })
        });
        if valid {
            assert_eq!(result.unwrap(), old);
        } else {
            assert!(result.unwrap_err().contains("waiting for GitHub"));
        }
        assert_eq!(load_scoped_from_store(&store, &scope).unwrap(), before);
    }
}

#[test]
fn failed_or_mismatched_replacement_never_overwrites_active_session() {
    for validation in [
        response(401, json!({"ok":false})),
        response(200, json!({"ok":true,"accountId":"acct_intruder"})),
    ] {
        let store = MemoryStore::default();
        let scope = SessionScope::for_profile("desk");
        promote_active_scoped_to_store(&store, &scope, &session("alice", "old")).unwrap();
        save_pending_scoped_to_store(&store, &scope, &pending()).unwrap();
        let before = load_scoped_from_store(&store, &scope).unwrap();
        assert!(
            resolve_from_store(&store, &scope, None, |_, path| {
                Ok(if path.starts_with("/api/auth/") {
                    response(
                        200,
                        json!({"accountId":"acct_alice","terminalToken":"unproven"}),
                    )
                } else {
                    validation.clone()
                })
            })
            .is_err()
        );
        assert_eq!(load_scoped_from_store(&store, &scope).unwrap(), before);
    }
}

#[test]
fn transport_failure_keeps_active_and_pending_records() {
    let store = MemoryStore::default();
    let scope = SessionScope::for_profile("desk");
    promote_active_scoped_to_store(&store, &scope, &session("alice", "old")).unwrap();
    save_pending_scoped_to_store(&store, &scope, &pending()).unwrap();
    let before = load_scoped_from_store(&store, &scope).unwrap();
    assert!(
        resolve_from_store(&store, &scope, None, |_, _| Err("fixture offline".into())).is_err()
    );
    assert_eq!(load_scoped_from_store(&store, &scope).unwrap(), before);
}

#[test]
fn expired_attempt_clears_only_pending_and_cancel_preserves_existing_account() {
    let store = MemoryStore::default();
    let scope = SessionScope::for_profile("desk");
    let old = session("alice", "working");
    promote_active_scoped_to_store(&store, &scope, &old).unwrap();
    save_pending_scoped_to_store(&store, &scope, &pending()).unwrap();
    assert_eq!(
        resolve_from_store(&store, &scope, None, |_, _| Ok(response(404, json!({})))).unwrap(),
        old
    );
    assert_eq!(
        load_scoped_from_store(&store, &scope).unwrap(),
        LocalState {
            active: Some(old),
            pending: None
        }
    );
}

#[test]
fn another_origin_or_unlinked_profile_never_receives_saved_authority() {
    let store = MemoryStore::default();
    let scope = SessionScope::for_profile("desk");
    promote_active_scoped_to_store(
        &store,
        &SessionScope::default(),
        &session("alice", "global"),
    )
    .unwrap();
    assert!(
        resolve_from_store(&store, &scope, None, |_, _| panic!(
            "must not query default account"
        ))
        .is_err()
    );
    save_pending_scoped_to_store(&store, &scope, &pending()).unwrap();
    assert!(
        resolve_from_store(
            &store,
            &scope,
            Some("https://elsewhere.example"),
            |_, _| panic!("must not disclose poll authority")
        )
        .is_err()
    );
}

#[test]
fn authentication_recovery_uses_corbanu_guidance_even_with_obsolete_server_text() {
    for text in [
        "Link Task Node from PFTerminal before calling terminal routes.",
        "Another server message",
    ] {
        let message = response(
            401,
            json!({"error":"terminal_login_required","message":text}),
        )
        .message();
        assert_eq!(
            message,
            "Task Node no longer accepts this profile's session. Run /tasknode link, choose your GitHub account, then run /tasknode status."
        );
    }
}
