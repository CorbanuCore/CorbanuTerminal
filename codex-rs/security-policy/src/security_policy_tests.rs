use std::collections::BTreeMap;

use pretty_assertions::assert_eq;

use super::*;

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("valid bounded text")
}

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human:jim").expect("valid human")
}

fn agent(id: &str) -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Agent, id).expect("valid agent")
}

fn chain() -> ActorChain {
    ActorChain::new(vec![human(), agent("agent:root")]).expect("valid actor chain")
}

fn resource() -> ProtectedResource {
    ProtectedResource::new(ResourceKind::FinancialAction, "account:paper").expect("valid resource")
}

fn request(now_unix_seconds: i64) -> AuthorizationRequest {
    AuthorizationRequest::new(
        chain(),
        resource(),
        PolicyAction::Execute,
        AuthorizationContext {
            now_unix_seconds,
            session_id: text("session:security-test"),
            task_id: text("task:execute-paper-order"),
            purpose: text("paper-trading-regression"),
            operation: text("order.execute"),
            destination: Some(text("venue:paper")),
            quantity: Some(QuantitativeLimit::new("USD", 500).expect("valid quantity")),
            grant_id: None,
        },
    )
    .expect("valid request")
}

fn scope(max_units: u64) -> GrantScope {
    GrantScope::new(
        resource(),
        [PolicyAction::Execute, PolicyAction::Export],
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), max_units)]),
    )
    .expect("valid scope")
}

fn grant() -> BoundedGrant {
    BoundedGrant::issue(
        human(),
        chain(),
        scope(1_000),
        100,
        200,
        text("grant-nonce-1"),
    )
    .expect("valid grant")
}

fn preview() -> ProtectedActionPreview {
    ProtectedActionPreview::new(request(100), 180, text("preview-nonce-1")).expect("valid preview")
}

#[test]
fn security_level_serialization_is_stable() {
    assert_eq!(
        serde_json::to_string(&SecurityLevel::Permissive).expect("serialize"),
        "\"permissive\""
    );
    assert_eq!(
        serde_json::to_string(&SecurityLevel::Moderate).expect("serialize"),
        "\"moderate\""
    );
    assert_eq!(
        serde_json::to_string(&SecurityLevel::Aggressive).expect("serialize"),
        "\"aggressive\""
    );
    assert!(serde_json::from_str::<SecurityLevel>("\"unknown\"").is_err());
}

#[test]
fn settings_are_versioned_and_default_to_permissive() {
    assert_eq!(
        SecuritySettings::default(),
        SecuritySettings::new(SecurityLevel::Permissive)
    );

    let mut unsupported = SecuritySettings::new(SecurityLevel::Moderate);
    unsupported.version += 1;
    assert!(matches!(
        unsupported.validate(),
        Err(SecuritySettingsError::UnsupportedVersion { .. })
    ));
}

#[test]
fn policy_text_and_actor_chains_are_bounded() {
    assert!(BoundedText::new("").is_err());
    assert!(BoundedText::new(" padded").is_err());
    assert!(BoundedText::new("line\nbreak").is_err());
    assert!(BoundedText::new("x".repeat(MAX_POLICY_TEXT_BYTES + 1)).is_err());

    assert!(ActorChain::new(vec![agent("agent:root")]).is_err());
    assert!(ActorChain::new(vec![human(), human()]).is_err());
}

#[test]
fn authorization_request_digest_is_deterministic_and_mutation_sensitive() {
    let original = request(100);
    assert_eq!(
        original.digest().expect("digest"),
        original.digest().expect("digest")
    );

    let mut mutated = original.clone();
    mutated.context.destination = Some(text("venue:other"));
    assert_ne!(
        original.digest().expect("digest"),
        mutated.digest().expect("digest")
    );
}

#[test]
fn authorization_digest_binds_session_task_purpose_and_operation() {
    let original = request(100);
    let mut variants = Vec::new();

    let mut wrong_session = original.clone();
    wrong_session.context.session_id = text("session:other");
    variants.push(wrong_session);

    let mut wrong_task = original.clone();
    wrong_task.context.task_id = text("task:other");
    variants.push(wrong_task);

    let mut wrong_purpose = original.clone();
    wrong_purpose.context.purpose = text("portfolio-disclosure");
    variants.push(wrong_purpose);

    let mut wrong_operation = original.clone();
    wrong_operation.context.operation = text("credential.reveal");
    variants.push(wrong_operation);

    let original_digest = original.digest().expect("original digest");
    for variant in variants {
        assert_ne!(variant.digest().expect("variant digest"), original_digest);
    }
}

#[test]
fn authorization_requests_fail_closed_without_echoing_protected_values() {
    let mut serialized = serde_json::to_value(request(100)).expect("serialize request");
    serialized["resource"]["id"] = serde_json::Value::String("credential-canary".to_string());

    for required_field in ["session_id", "task_id", "purpose", "operation"] {
        let mut incomplete = serialized.clone();
        incomplete["context"]
            .as_object_mut()
            .expect("context object")
            .remove(required_field);
        let error = serde_json::from_value::<AuthorizationRequest>(incomplete)
            .expect_err("incomplete request must fail")
            .to_string();
        assert!(error.contains(required_field));
        assert!(!error.contains("credential-canary"));
    }

    let mut invalid_time = request(100);
    invalid_time.context.now_unix_seconds = -1;
    assert!(matches!(
        invalid_time.validate(),
        Err(AuthorizationError::NegativeTimestamp)
    ));
}

#[test]
fn permissive_composition_preserves_every_frozen_surface_decision() {
    let decision = permissive_decision(&request(100)).expect("decision");
    let baseline = [
        ("permission-profile", true),
        ("approval-policy", true),
        ("sandbox", true),
        ("network", false),
        ("vault-programmatic-api-key", true),
        ("vault-programmatic-private-key", false),
        ("agent-spawn-within-depth", true),
        ("agent-spawn-over-depth", false),
    ];

    for (surface, existing_allow) in baseline {
        assert_eq!(
            compose_existing_decision(existing_allow, &decision),
            existing_allow,
            "Permissive changed {surface}"
        );
    }
}

#[test]
fn grant_integrity_expiry_and_exact_scope_are_enforced() {
    let grant = grant();
    assert!(grant.matches_request(&request(150)).expect("valid grant"));
    assert!(!grant.matches_request(&request(200)).expect("expired grant"));

    let mut wrong_destination = request(150);
    wrong_destination.context.destination = Some(text("venue:other"));
    assert!(
        !grant
            .matches_request(&wrong_destination)
            .expect("scope mismatch")
    );

    let mut excessive_quantity = request(150);
    excessive_quantity.context.quantity =
        Some(QuantitativeLimit::new("USD", 1_001).expect("valid quantity"));
    assert!(
        !grant
            .matches_request(&excessive_quantity)
            .expect("limit mismatch")
    );

    let mut wrong_asset = request(150);
    wrong_asset.context.quantity = Some(QuantitativeLimit::new("BTC", 1).expect("valid quantity"));
    assert!(!grant.matches_request(&wrong_asset).expect("asset mismatch"));

    let mut missing_quantity = request(150);
    missing_quantity.context.quantity = None;
    assert!(
        !grant
            .matches_request(&missing_quantity)
            .expect("missing bounded quantity")
    );

    let mut mutated = grant;
    mutated.expires_at_unix_seconds += 1;
    assert!(matches!(
        mutated.validate(),
        Err(GrantValidationError::IntegrityMismatch)
    ));
}

#[test]
fn derived_grants_can_only_narrow_scope_chain_and_expiry() {
    let parent = grant();
    let child_chain = ActorChain::new(vec![human(), agent("agent:root"), agent("agent:child")])
        .expect("valid child chain");
    let child_scope = GrantScope::new(
        resource(),
        [PolicyAction::Execute],
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), 250)]),
    )
    .expect("valid child scope");
    let child = BoundedGrant::derive_child(
        &parent,
        child_chain.clone(),
        child_scope,
        120,
        180,
        text("child-nonce"),
    )
    .expect("narrow child grant");
    child.validate().expect("child integrity");
    assert_eq!(child.parent_grant_id.as_ref(), Some(&parent.grant_id));

    let broader_scope = GrantScope::new(
        resource(),
        [PolicyAction::Execute],
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), 1_001)]),
    )
    .expect("structurally valid scope");
    assert!(matches!(
        BoundedGrant::derive_child(
            &parent,
            child_chain,
            broader_scope,
            120,
            180,
            text("broad-child-nonce"),
        ),
        Err(GrantValidationError::ScopeNotNarrower)
    ));
}

#[test]
fn mandate_binds_exact_preview_and_rejects_replay() {
    let preview = preview();
    let mandate = ProtectedActionMandate::approve(&preview, human(), 110).expect("approve");
    assert!(mandate.matches_preview(&preview, 120).expect("match"));

    let mut mutated = preview.clone();
    mutated.request.context.destination = Some(text("venue:other"));
    assert!(
        !mandate
            .matches_preview(&mutated, 120)
            .expect("mutation rejected")
    );

    let mut ledger = ReplayLedger::new();
    ledger.consume(&mandate, &preview, 120).expect("first use");
    assert!(ledger.contains(&mandate.mandate_id));
    assert!(matches!(
        ledger.consume(&mandate, &preview, 121),
        Err(MandateError::Replay)
    ));

    let receipt = ActionReceipt::complete(&mandate, &preview, MandateOutcome::Executed, 120)
        .expect("receipt");
    receipt.validate().expect("receipt integrity");
}

#[test]
fn revocation_and_kill_switch_are_idempotent_and_restart_safe() {
    let grant = grant();
    let event = RevocationEvent::new(
        human(),
        RevocationTarget::Grant {
            grant_id: grant.grant_id.clone(),
        },
        RevocationReason::HumanRequest,
        150,
    )
    .expect("revocation");
    let mut state = RevocationState::new();
    assert!(state.apply(&event).expect("first apply"));
    assert!(!state.apply(&event).expect("duplicate apply"));
    assert!(state.grant_is_revoked(&grant));

    let kill_switch = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::KillSwitch,
        160,
    )
    .expect("kill switch");
    state.apply(&kill_switch).expect("activate kill switch");

    let serialized = serde_json::to_string(&state).expect("serialize state");
    let restored: RevocationState = serde_json::from_str(&serialized).expect("restore state");
    assert!(restored.kill_switch_active);
    assert!(restored.grant_is_revoked(&grant));
}

#[test]
fn policy_contracts_have_no_arbitrary_payload_or_secret_fields() {
    let preview = preview();
    let mandate = ProtectedActionMandate::approve(&preview, human(), 110).expect("approve");
    let serialized = serde_json::to_string(&(grant(), preview, mandate)).expect("serialize");
    for forbidden in ["payload", "credential_value", "private_key", "secret_value"] {
        assert!(
            !serialized.contains(forbidden),
            "found forbidden field {forbidden}"
        );
    }
}
