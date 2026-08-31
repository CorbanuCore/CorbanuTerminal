#![allow(clippy::expect_used)]

use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::permissive_decision;
use pretty_assertions::assert_eq;

use crate::AuthorityIdentity;
use crate::EventContext;
use crate::SecurityEvent;
use crate::SecurityEventError;

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("bounded test text")
}

fn principal(kind: PrincipalKind, id: &str) -> PolicyPrincipal {
    PolicyPrincipal::new(kind, id).expect("test principal")
}

fn request(secret_marker: &str) -> AuthorizationRequest {
    AuthorizationRequest::new(
        ActorChain::new(vec![
            principal(PrincipalKind::Human, "human-1"),
            principal(PrincipalKind::Agent, "agent-1"),
        ])
        .expect("actor chain"),
        ProtectedResource::new(ResourceKind::FinancialAction, "account-1").expect("resource"),
        PolicyAction::Sign,
        AuthorizationContext {
            now_unix_seconds: 10,
            session_id: text("session-1"),
            task_id: text("task-1"),
            purpose: text(secret_marker),
            operation: text("sign"),
            destination: Some(text(secret_marker)),
            quantity: None,
            grant_id: None,
        },
    )
    .expect("request")
}

fn context(policy_generation: u64, run_generation: u64) -> EventContext {
    EventContext::new(
        principal(PrincipalKind::Service, "audit-producer"),
        policy_generation,
        run_generation,
    )
    .expect("event context")
}

#[test]
fn request_event_is_secret_free_and_digest_bound() {
    let request = request("SECRET-canary-not-for-journal");
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(context(3, 7), None, &request, decision, 11)
        .expect("security event");

    let encoded = serde_json::to_string(&event).expect("serialize event");
    assert!(!encoded.contains("SECRET-canary-not-for-journal"));
    assert!(!encoded.contains("destination"));
    assert!(!encoded.contains("purpose"));
    assert!(!encoded.contains("operation"));
    let decoded: SecurityEvent = serde_json::from_str(&encoded).expect("deserialize event");
    assert_eq!(decoded, event);
}

#[test]
fn tampered_event_identity_fails_closed() {
    let request = request("research");
    let decision = permissive_decision(&request).expect("decision");
    let event = SecurityEvent::decision(context(1, 1), None, &request, decision, 11)
        .expect("security event");
    let mut value = serde_json::to_value(event).expect("event value");
    value["occurred_at_unix_seconds"] = serde_json::json!(12);

    let error = serde_json::from_value::<SecurityEvent>(value).expect_err("tampering must fail");
    assert!(error.to_string().contains("identity does not match"));
}

#[test]
fn reservation_identity_is_stable_and_generation_bound() {
    let request = request("research");
    let secret_deduplication_key = "SECRET-deduplication-canary";
    let authority = AuthorityIdentity::Grant {
        grant_id: text("grant-1"),
    };
    let first = SecurityEvent::dispatch_intent(
        context(2, 9),
        None,
        &request,
        authority.clone(),
        text(secret_deduplication_key),
        12,
    )
    .expect("first intent");
    let duplicate = SecurityEvent::dispatch_intent(
        context(2, 9),
        None,
        &request,
        authority.clone(),
        text(secret_deduplication_key),
        12,
    )
    .expect("duplicate intent");
    let next_run = SecurityEvent::dispatch_intent(
        context(2, 10),
        None,
        &request,
        authority,
        text(secret_deduplication_key),
        12,
    )
    .expect("next-run intent");

    assert_eq!(duplicate, first);
    assert_ne!(next_run.event_id, first.event_id);
    let encoded = serde_json::to_string(&first).expect("serialize intent");
    assert!(!encoded.contains(secret_deduplication_key));
}

#[test]
fn zero_run_generation_is_rejected() {
    let error = EventContext::new(principal(PrincipalKind::Service, "audit-producer"), 0, 0)
        .expect_err("zero run generation must fail");
    assert!(matches!(error, SecurityEventError::InvalidRunGeneration));
}
