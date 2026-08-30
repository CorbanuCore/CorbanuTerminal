use std::collections::BTreeMap;

use pretty_assertions::assert_eq;

use super::revocation::DISPATCH_FENCE_SCHEMA_VERSION;
use super::revocation::DispatchFence;
use super::revocation::DispatchPhase;
use super::revocation::ProtectedDispatchStep;
use super::revocation::RestrictionAuditStatus;
use super::*;

const INITIAL_GENERATION: u64 = 0;
const VALID_NOW: i64 = 150;
const POST_EVENT_NOW: i64 = 151;
const BEFORE_GRANT: i64 = 99;
const MANDATE_EXPIRY: i64 = 180;
const GRANT_EXPIRY: i64 = 200;

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
        GrantContext::new(
            text("session:security-test"),
            text("task:execute-paper-order"),
            text("paper-trading-regression"),
            text("order.execute"),
        ),
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

fn grant_with_nonce(nonce: &str) -> BoundedGrant {
    BoundedGrant::issue(human(), chain(), scope(1_000), 100, 200, text(nonce)).expect("valid grant")
}

fn preview() -> ProtectedActionPreview {
    ProtectedActionPreview::new(request(100), 180, text("preview-nonce-1")).expect("valid preview")
}

fn queued_grant(
    run_id: BoundedText,
    expected_generation: u64,
    grant: &BoundedGrant,
    revocations: &RevocationState,
) -> Result<DispatchFence, RevocationError> {
    DispatchFence::queued_for_grant(run_id, expected_generation, VALID_NOW, grant, revocations)
}

fn authorize_grant(
    fence: &mut DispatchFence,
    run_id: &BoundedText,
    grant: &BoundedGrant,
    revocations: &RevocationState,
    step: ProtectedDispatchStep,
) -> Result<(), RevocationError> {
    fence.authorize_grant(run_id, VALID_NOW, grant, revocations, step)
}

fn refresh_grant(
    fence: &mut DispatchFence,
    run_id: &BoundedText,
    grant: &BoundedGrant,
    revocations: &RevocationState,
) -> Result<(), RevocationError> {
    fence.refresh_grant(run_id, VALID_NOW, grant, revocations)
}

fn queued_mandate(
    run_id: BoundedText,
    expected_generation: u64,
    mandate: &ProtectedActionMandate,
    revocations: &RevocationState,
) -> Result<DispatchFence, RevocationError> {
    DispatchFence::queued_for_mandate(run_id, expected_generation, VALID_NOW, mandate, revocations)
}

fn authorize_mandate(
    fence: &mut DispatchFence,
    run_id: &BoundedText,
    mandate: &ProtectedActionMandate,
    revocations: &RevocationState,
    step: ProtectedDispatchStep,
) -> Result<(), RevocationError> {
    fence.authorize_mandate(run_id, VALID_NOW, mandate, revocations, step)
}

fn refresh_mandate(
    fence: &mut DispatchFence,
    run_id: &BoundedText,
    mandate: &ProtectedActionMandate,
    revocations: &RevocationState,
) -> Result<(), RevocationError> {
    fence.refresh_mandate(run_id, VALID_NOW, mandate, revocations)
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
        ("tool-existing-policy", false),
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
    assert!(!grant.matches_request(&request(99)).expect("not issued"));
    assert!(!grant.matches_request(&request(200)).expect("expired grant"));

    let mut wrong_actor = request(150);
    wrong_actor.subject =
        ActorChain::new(vec![human(), agent("agent:other")]).expect("valid wrong actor");
    assert!(!grant.matches_request(&wrong_actor).expect("actor mismatch"));

    let mut wrong_action = request(150);
    wrong_action.action = PolicyAction::Sign;
    assert!(
        !grant
            .matches_request(&wrong_action)
            .expect("action mismatch")
    );

    let mut wrong_resource = request(150);
    wrong_resource.resource =
        ProtectedResource::new(ResourceKind::FinancialAction, "account:other")
            .expect("valid adjacent resource");
    assert!(
        !grant
            .matches_request(&wrong_resource)
            .expect("resource mismatch")
    );

    for (field, value) in [
        ("session_id", "session:other"),
        ("task_id", "task:other"),
        ("purpose", "portfolio-disclosure"),
        ("operation", "order.cancel"),
    ] {
        let mut wrong_context = request(150);
        match field {
            "session_id" => wrong_context.context.session_id = text(value),
            "task_id" => wrong_context.context.task_id = text(value),
            "purpose" => wrong_context.context.purpose = text(value),
            "operation" => wrong_context.context.operation = text(value),
            _ => unreachable!(),
        }
        assert!(
            !grant
                .matches_request(&wrong_context)
                .expect("context mismatch"),
            "grant matched the wrong {field}"
        );
    }

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
        parent.scope.context.clone(),
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
        parent.scope.context.clone(),
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), 1_001)]),
    )
    .expect("structurally valid scope");
    assert!(matches!(
        BoundedGrant::derive_child(
            &parent,
            child_chain.clone(),
            broader_scope,
            120,
            180,
            text("broad-child-nonce"),
        ),
        Err(GrantValidationError::ScopeNotNarrower)
    ));

    let extra_asset_scope = GrantScope::new(
        resource(),
        [PolicyAction::Execute],
        parent.scope.context.clone(),
        Some(text("venue:paper")),
        BTreeMap::from([(text("BTC"), 1), (text("USD"), 250)]),
    )
    .expect("structurally valid scope");
    assert!(matches!(
        BoundedGrant::derive_child(
            &parent,
            child_chain.clone(),
            extra_asset_scope,
            120,
            180,
            text("extra-asset-child-nonce"),
        ),
        Err(GrantValidationError::ScopeNotNarrower)
    ));

    let extra_action_scope = GrantScope::new(
        resource(),
        [PolicyAction::Execute, PolicyAction::Sign],
        parent.scope.context.clone(),
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), 250)]),
    )
    .expect("structurally valid scope");
    assert!(matches!(
        BoundedGrant::derive_child(
            &parent,
            child_chain.clone(),
            extra_action_scope,
            120,
            180,
            text("extra-action-child-nonce"),
        ),
        Err(GrantValidationError::ScopeNotNarrower)
    ));

    let narrow_scope = GrantScope::new(
        resource(),
        [PolicyAction::Execute],
        parent.scope.context.clone(),
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), 250)]),
    )
    .expect("valid narrow scope");
    assert!(matches!(
        BoundedGrant::derive_child(
            &parent,
            child_chain.clone(),
            narrow_scope,
            99,
            180,
            text("pre-parent-child-nonce"),
        ),
        Err(GrantValidationError::IssuedBeforeParent)
    ));

    let mut adjacent_context = parent.scope.context.clone();
    adjacent_context.operation = text("order.cancel");
    let adjacent_context_scope = GrantScope::new(
        resource(),
        [PolicyAction::Execute],
        adjacent_context,
        Some(text("venue:paper")),
        BTreeMap::from([(text("USD"), 250)]),
    )
    .expect("structurally valid adjacent context");
    assert!(matches!(
        BoundedGrant::derive_child(
            &parent,
            child_chain,
            adjacent_context_scope,
            120,
            180,
            text("adjacent-context-child-nonce"),
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

    assert!(
        !mandate
            .matches_preview(&preview, 109)
            .expect("pre-approval use rejected")
    );
    assert!(
        !mandate
            .matches_preview(&preview, 180)
            .expect("stale use rejected")
    );
    assert!(matches!(
        ActionReceipt::complete(&mandate, &preview, MandateOutcome::Executed, 109),
        Err(MandateError::PreviewMismatchOrExpired)
    ));

    let receipt = ActionReceipt::complete(&mandate, &preview, MandateOutcome::Executed, 120)
        .expect("receipt");
    receipt.validate().expect("receipt integrity");
    assert_eq!(
        serde_json::to_value(&receipt).expect("serialize receipt"),
        serde_json::json!({
            "schema_version": receipt.schema_version,
            "receipt_id": receipt.receipt_id,
            "mandate_id": receipt.mandate_id,
            "preview_digest": receipt.preview_digest,
            "outcome": "executed",
            "completed_at_unix_seconds": 120
        })
    );

    let mut mutated_receipt = receipt;
    mutated_receipt.completed_at_unix_seconds += 1;
    assert!(matches!(
        mutated_receipt.validate(),
        Err(MandateError::ReceiptIntegrityMismatch)
    ));
}

#[test]
fn credential_use_receipt_is_bound_and_contains_only_secret_free_metadata() {
    let capability_id = CapabilityId::from_sha256_hex("a".repeat(64)).expect("capability id");
    let receipt = ActionReceipt::complete_credential_use(
        capability_id.clone(),
        DecisionReason::MatchingGrant,
        text("responses.create"),
        text("https://api.openai.com:443"),
        text(&"b".repeat(64)),
        MandateOutcome::Executed,
        120,
    )
    .expect("credential receipt");
    receipt.validate().expect("receipt integrity");

    assert_eq!(
        serde_json::to_value(&receipt).expect("serialize receipt"),
        serde_json::json!({
            "schema_version": receipt.schema_version,
            "receipt_id": receipt.receipt_id,
            "mandate_id": capability_id.as_str(),
            "preview_digest": "b".repeat(64),
            "outcome": "executed",
            "completed_at_unix_seconds": 120,
            "credential_use": {
                "capability_id": capability_id.as_str(),
                "policy_reason": "matching_grant",
                "operation": "responses.create",
                "destination": "https://api.openai.com:443"
            }
        })
    );
    let serialized = serde_json::to_string(&receipt).expect("serialize");
    for forbidden in [
        "provider.openai",
        "credential_value",
        "secret_value",
        "sk-canary",
        "\"label\"",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "found forbidden text {forbidden}"
        );
    }

    let mut mismatched = receipt;
    mismatched
        .credential_use
        .as_mut()
        .expect("credential metadata")
        .capability_id =
        CapabilityId::from_sha256_hex("c".repeat(64)).expect("adjacent capability");
    assert!(matches!(
        mismatched.validate(),
        Err(MandateError::CredentialReceiptMismatch)
    ));
}

#[test]
fn mandate_rejects_mutation_of_every_bound_preview_dimension() {
    let original = preview();
    let mandate = ProtectedActionMandate::approve(&original, human(), 110).expect("approve");
    let mut variants = Vec::new();

    let mut changed_subject = original.clone();
    changed_subject.request.subject =
        ActorChain::new(vec![human(), agent("agent:other")]).expect("changed subject");
    variants.push(changed_subject);

    let mut changed_resource = original.clone();
    changed_resource.request.resource =
        ProtectedResource::new(ResourceKind::FinancialAction, "account:other")
            .expect("changed resource");
    variants.push(changed_resource);

    let mut changed_action = original.clone();
    changed_action.request.action = PolicyAction::Sign;
    variants.push(changed_action);

    let mut changed_request_time = original.clone();
    changed_request_time.request.context.now_unix_seconds += 1;
    variants.push(changed_request_time);

    let mut changed_session = original.clone();
    changed_session.request.context.session_id = text("session:other");
    variants.push(changed_session);

    let mut changed_task = original.clone();
    changed_task.request.context.task_id = text("task:other");
    variants.push(changed_task);

    let mut changed_purpose = original.clone();
    changed_purpose.request.context.purpose = text("portfolio-disclosure");
    variants.push(changed_purpose);

    let mut changed_operation = original.clone();
    changed_operation.request.context.operation = text("credential.reveal");
    variants.push(changed_operation);

    let mut changed_destination = original.clone();
    changed_destination.request.context.destination = Some(text("venue:other"));
    variants.push(changed_destination);

    let mut changed_quantity = original.clone();
    changed_quantity.request.context.quantity =
        Some(QuantitativeLimit::new("USD", 501).expect("changed quantity"));
    variants.push(changed_quantity);

    let mut changed_grant = original.clone();
    changed_grant.request.context.grant_id = Some(text("grant:other"));
    variants.push(changed_grant);

    let mut changed_expiry = original.clone();
    changed_expiry.expires_at_unix_seconds -= 1;
    variants.push(changed_expiry);

    let mut changed_nonce = original.clone();
    changed_nonce.nonce = text("preview-nonce-other");
    variants.push(changed_nonce);

    for variant in variants {
        assert!(
            !mandate
                .matches_preview(&variant, 120)
                .expect("mutated preview rejected")
        );
    }

    let mut malformed = serde_json::to_value(&mandate).expect("serialize mandate");
    malformed
        .as_object_mut()
        .expect("mandate object")
        .insert("unexpected".to_string(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<ProtectedActionMandate>(malformed).is_err());
    assert!(
        !mandate
            .matches_preview(&original, -1)
            .expect("clock failure")
    );
}

#[test]
fn revocation_and_kill_switch_are_idempotent_and_restart_safe() {
    let grant = grant();
    let grant_event = RevocationEvent::new(
        human(),
        RevocationTarget::Grant {
            grant_id: grant.grant_id.clone(),
        },
        RevocationReason::HumanRequest,
        150,
    )
    .expect("revocation");
    let mut state = RevocationState::new();
    assert!(state.apply(&grant_event).expect("first apply"));
    assert_eq!(state.generation, 1);
    assert!(!state.apply(&grant_event).expect("duplicate apply"));
    assert_eq!(state.generation, 1);
    assert!(state.grant_is_revoked(&grant));

    let approved_preview = preview();
    let mandate =
        ProtectedActionMandate::approve(&approved_preview, human(), 110).expect("mandate");
    let actor_event = RevocationEvent::new(
        human(),
        RevocationTarget::Actor {
            actor_id: text("agent:root"),
        },
        RevocationReason::RiskSignal,
        155,
    )
    .expect("actor revocation");
    assert!(state.apply(&actor_event).expect("revoke actor"));
    assert_eq!(state.generation, 2);
    assert!(state.mandate_is_revoked(&mandate));

    let kill_switch = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::KillSwitch,
        160,
    )
    .expect("kill switch");
    assert!(state.apply(&kill_switch).expect("activate kill switch"));
    assert_eq!(state.generation, 3);

    let serialized = serde_json::to_string(&state).expect("serialize state");
    let restored: RevocationState = serde_json::from_str(&serialized).expect("restore state");
    restored.validate().expect("valid restored state");
    assert_eq!(restored, state);
    assert!(restored.kill_switch_active);
    assert!(restored.grant_is_revoked(&grant));
    assert!(restored.mandate_is_revoked(&mandate));
}

#[test]
fn revocation_kill_switch_order_converges_and_corrupt_state_fails_closed() {
    let enable = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::KillSwitch,
        200,
    )
    .expect("enable event");
    let disable = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: false },
        RevocationReason::HumanRequest,
        200,
    )
    .expect("disable event");
    let (earlier, later) = if enable.event_id < disable.event_id {
        (&enable, &disable)
    } else {
        (&disable, &enable)
    };

    let mut ordered = RevocationState::new();
    assert!(ordered.apply(earlier).expect("earlier event"));
    assert!(ordered.apply(later).expect("later event"));

    let mut raced = RevocationState::new();
    assert!(raced.apply(later).expect("later event first"));
    assert!(!raced.apply(earlier).expect("stale event is a no-op"));
    assert_eq!(raced, ordered);
    assert_eq!(raced.generation, 2);

    let mut unknown_target = serde_json::to_value(&enable).expect("serialize event");
    unknown_target["target"]["kind"] = serde_json::Value::String("unknown".to_string());
    assert!(serde_json::from_value::<RevocationEvent>(unknown_target).is_err());

    let mut corrupt = serde_json::to_value(&raced).expect("serialize state");
    corrupt["generation"] = serde_json::Value::from(0);
    let corrupt_state =
        serde_json::from_value::<RevocationState>(corrupt).expect("typed corrupt state");
    assert!(matches!(
        corrupt_state.validate(),
        Err(RevocationError::GenerationMismatch { .. })
    ));
}

#[test]
fn revocation_dispatch_fence_requires_bound_run_authority_and_generation() {
    let grant = grant();
    let run_id = text("run:one");
    let revocations = RevocationState::new();
    let mut fence = queued_grant(run_id.clone(), INITIAL_GENERATION, &grant, &revocations).unwrap();
    assert_eq!(fence.schema_version(), DISPATCH_FENCE_SCHEMA_VERSION);
    assert_eq!(fence.phase(), DispatchPhase::Queued);

    assert!(matches!(
        authorize_grant(
            &mut fence,
            &run_id,
            &grant,
            &revocations,
            ProtectedDispatchStep::ChannelWrite
        ),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    authorize_grant(
        &mut fence,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::Admit,
    )
    .unwrap();
    assert!(matches!(
        authorize_grant(
            &mut fence,
            &run_id,
            &grant,
            &revocations,
            ProtectedDispatchStep::BeginUpload,
        ),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    authorize_grant(
        &mut fence,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::EstablishChannel,
    )
    .unwrap();
    assert_eq!(fence.phase(), DispatchPhase::EstablishedChannel);

    assert!(matches!(
        authorize_grant(
            &mut fence,
            &text("run:other"),
            &grant,
            &revocations,
            ProtectedDispatchStep::ChannelWrite
        ),
        Err(RevocationError::DispatchBindingMismatch)
    ));
    assert!(matches!(
        queued_grant(run_id, /*expected_generation*/ 1, &grant, &revocations),
        Err(RevocationError::StaleDispatchGeneration {
            expected: 1,
            current: 0
        })
    ));
}

#[test]
fn revocation_dispatch_fence_rechecks_authority_validity_at_every_boundary() {
    let grant = grant();
    let mandate = ProtectedActionMandate::approve(&preview(), human(), 110).unwrap();
    let run_id = text("run:validity-window");
    let revocations = RevocationState::new();

    assert!(matches!(
        DispatchFence::queued_for_grant(
            run_id.clone(),
            INITIAL_GENERATION,
            BEFORE_GRANT,
            &grant,
            &revocations,
        ),
        Err(RevocationError::AuthorityOutsideValidityWindow)
    ));
    assert!(matches!(
        DispatchFence::queued_for_mandate(
            run_id.clone(),
            INITIAL_GENERATION,
            MANDATE_EXPIRY,
            &mandate,
            &revocations,
        ),
        Err(RevocationError::AuthorityOutsideValidityWindow)
    ));

    let mut grant_fence = DispatchFence::queued_for_grant(
        run_id.clone(),
        INITIAL_GENERATION,
        VALID_NOW,
        &grant,
        &revocations,
    )
    .unwrap();
    assert!(matches!(
        grant_fence.refresh_grant(&run_id, GRANT_EXPIRY, &grant, &revocations,),
        Err(RevocationError::AuthorityOutsideValidityWindow)
    ));
    assert_eq!(grant_fence.phase(), DispatchPhase::Fenced);

    let mut mandate_fence = DispatchFence::queued_for_mandate(
        run_id.clone(),
        INITIAL_GENERATION,
        VALID_NOW,
        &mandate,
        &revocations,
    )
    .unwrap();
    assert!(matches!(
        mandate_fence.authorize_mandate(
            &run_id,
            MANDATE_EXPIRY,
            &mandate,
            &revocations,
            ProtectedDispatchStep::Admit,
        ),
        Err(RevocationError::AuthorityOutsideValidityWindow)
    ));
    assert_eq!(mandate_fence.phase(), DispatchPhase::Fenced);
}

#[test]
fn revocation_kill_linearizes_before_open_channel_and_upload_writes() {
    let grant = grant();
    let run_id = text("run:kill-race");
    let mut revocations = RevocationState::new();
    let mut channel =
        queued_grant(run_id.clone(), INITIAL_GENERATION, &grant, &revocations).unwrap();
    authorize_grant(
        &mut channel,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::Admit,
    )
    .unwrap();
    authorize_grant(
        &mut channel,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::EstablishChannel,
    )
    .unwrap();
    let mut upload = channel.clone();
    authorize_grant(
        &mut upload,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::BeginUpload,
    )
    .unwrap();

    let kill = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::KillSwitch,
        150,
    )
    .unwrap();
    revocations.apply(&kill).unwrap();

    for (fence, step) in [
        (&mut channel, ProtectedDispatchStep::ChannelWrite),
        (&mut upload, ProtectedDispatchStep::UploadWrite),
    ] {
        assert!(matches!(
            fence.authorize_grant(&run_id, POST_EVENT_NOW, &grant, &revocations, step),
            Err(RevocationError::AuthorityRevoked)
        ));
        assert_eq!(fence.phase(), DispatchPhase::Fenced);
    }
}

#[test]
fn revocation_targeted_event_fences_victim_without_revoking_sibling() {
    let victim = grant_with_nonce("grant-nonce-victim");
    let sibling = grant_with_nonce("grant-nonce-sibling");
    let victim_run = text("run:victim");
    let sibling_run = text("run:sibling");
    let mut revocations = RevocationState::new();
    let mut victim_fence = queued_grant(
        victim_run.clone(),
        INITIAL_GENERATION,
        &victim,
        &revocations,
    )
    .unwrap();
    let mut sibling_fence = queued_grant(
        sibling_run.clone(),
        INITIAL_GENERATION,
        &sibling,
        &revocations,
    )
    .unwrap();

    let event = RevocationEvent::new(
        human(),
        RevocationTarget::Grant {
            grant_id: victim.grant_id.clone(),
        },
        RevocationReason::HumanRequest,
        150,
    )
    .unwrap();
    revocations.apply(&event).unwrap();

    assert!(matches!(
        authorize_grant(
            &mut victim_fence,
            &victim_run,
            &victim,
            &revocations,
            ProtectedDispatchStep::Admit
        ),
        Err(RevocationError::AuthorityRevoked)
    ));
    assert!(matches!(
        refresh_grant(&mut victim_fence, &victim_run, &victim, &revocations,),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    assert_eq!(victim_fence.phase(), DispatchPhase::Fenced);
    assert!(matches!(
        authorize_grant(
            &mut sibling_fence,
            &sibling_run,
            &sibling,
            &revocations,
            ProtectedDispatchStep::Admit
        ),
        Err(RevocationError::StaleDispatchGeneration {
            expected: 0,
            current: 1
        })
    ));
    refresh_grant(&mut sibling_fence, &sibling_run, &sibling, &revocations).unwrap();
    assert_eq!(sibling_fence.generation(), 1);
    assert!(matches!(
        refresh_grant(
            &mut sibling_fence,
            &sibling_run,
            &sibling,
            &RevocationState::new(),
        ),
        Err(RevocationError::StaleDispatchGeneration {
            expected: 1,
            current: 0
        })
    ));
    authorize_grant(
        &mut sibling_fence,
        &sibling_run,
        &sibling,
        &revocations,
        ProtectedDispatchStep::Admit,
    )
    .unwrap();
    assert_eq!(sibling_fence.phase(), DispatchPhase::Admitted);
}

#[test]
fn revocation_audit_unavailable_does_not_delay_emergency_restriction() {
    let grant = grant();
    let run_id = text("run:audit-gap");
    let mut revocations = RevocationState::new();
    let mut fence = queued_grant(run_id.clone(), INITIAL_GENERATION, &grant, &revocations).unwrap();
    let kill = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::KillSwitch,
        150,
    )
    .unwrap();

    let first = revocations
        .apply_restriction(&kill, || RestrictionAuditStatus::Unavailable)
        .unwrap();
    assert_eq!(
        first,
        super::revocation::RestrictionApplication {
            event_was_effective: true,
            generation: 1,
            audit_status: RestrictionAuditStatus::Unavailable,
        }
    );
    assert!(matches!(
        authorize_grant(
            &mut fence,
            &run_id,
            &grant,
            &revocations,
            ProtectedDispatchStep::Admit,
        ),
        Err(RevocationError::AuthorityRevoked)
    ));
    assert!(matches!(
        fence.record_completed(),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    assert!(matches!(
        fence.record_unknown_financial_outcome(),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    let repeated = revocations
        .apply_restriction(&kill, || RestrictionAuditStatus::Recorded)
        .unwrap();
    assert!(!repeated.event_was_effective);
    assert_eq!(repeated.generation, 1);

    let deactivate = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: false },
        RevocationReason::HumanRequest,
        151,
    )
    .unwrap();
    assert!(matches!(
        revocations.apply_restriction(&deactivate, || panic!("audit must not run")),
        Err(RevocationError::NotARestriction)
    ));
    assert!(revocations.kill_switch_active);

    assert!(revocations.apply(&deactivate).unwrap());
    assert!(!revocations.kill_switch_active);
    let skewed_enable = RevocationEvent::new(
        human(),
        RevocationTarget::KillSwitch { active: true },
        RevocationReason::RiskSignal,
        149,
    )
    .unwrap();
    assert!(matches!(
        revocations.apply_restriction(&skewed_enable, || panic!("audit must not run")),
        Err(RevocationError::RestrictionSuperseded)
    ));
    assert!(!revocations.kill_switch_active);
    assert_eq!(revocations.generation, 2);
}

#[test]
fn revocation_preserves_completed_and_unknown_financial_outcomes() {
    let grant = grant();
    let run_id = text("run:financial");
    let revocations = RevocationState::new();
    let mut completed =
        queued_grant(run_id.clone(), INITIAL_GENERATION, &grant, &revocations).unwrap();
    authorize_grant(
        &mut completed,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::Admit,
    )
    .unwrap();
    completed.record_completed().unwrap();
    completed.record_completed().unwrap();
    assert_eq!(completed.phase(), DispatchPhase::Completed);
    assert!(matches!(
        completed.authorize_grant(
            &run_id,
            GRANT_EXPIRY,
            &grant,
            &revocations,
            ProtectedDispatchStep::ChannelWrite,
        ),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    assert_eq!(completed.phase(), DispatchPhase::Completed);

    let mut unknown =
        queued_grant(run_id.clone(), INITIAL_GENERATION, &grant, &revocations).unwrap();
    authorize_grant(
        &mut unknown,
        &run_id,
        &grant,
        &revocations,
        ProtectedDispatchStep::Admit,
    )
    .unwrap();
    unknown.record_unknown_financial_outcome().unwrap();
    unknown.record_unknown_financial_outcome().unwrap();
    assert_eq!(unknown.phase(), DispatchPhase::UnknownFinancialOutcome);
    assert!(matches!(
        unknown.record_completed(),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    assert!(matches!(
        unknown.refresh_grant(&run_id, GRANT_EXPIRY, &grant, &revocations),
        Err(RevocationError::InvalidDispatchTransition)
    ));
    assert_eq!(unknown.phase(), DispatchPhase::UnknownFinancialOutcome);
}

#[test]
fn revocation_mandate_dispatch_uses_the_same_generation_fence() {
    let preview = preview();
    let mandate = ProtectedActionMandate::approve(&preview, human(), 110).unwrap();
    let sibling_mandate = ProtectedActionMandate::approve(&preview, human(), 111).unwrap();
    let run_id = text("run:mandate");
    let sibling_run_id = text("run:mandate-sibling");
    let mut revocations = RevocationState::new();
    let mut fence =
        queued_mandate(run_id.clone(), INITIAL_GENERATION, &mandate, &revocations).unwrap();
    let mut sibling_fence = queued_mandate(
        sibling_run_id.clone(),
        INITIAL_GENERATION,
        &sibling_mandate,
        &revocations,
    )
    .unwrap();
    let event = RevocationEvent::new(
        human(),
        RevocationTarget::Mandate {
            mandate_id: mandate.mandate_id.clone(),
        },
        RevocationReason::HumanRequest,
        150,
    )
    .unwrap();
    revocations.apply(&event).unwrap();
    assert!(matches!(
        authorize_mandate(
            &mut fence,
            &run_id,
            &mandate,
            &revocations,
            ProtectedDispatchStep::Admit
        ),
        Err(RevocationError::AuthorityRevoked)
    ));
    refresh_mandate(
        &mut sibling_fence,
        &sibling_run_id,
        &sibling_mandate,
        &revocations,
    )
    .unwrap();
    authorize_mandate(
        &mut sibling_fence,
        &sibling_run_id,
        &sibling_mandate,
        &revocations,
        ProtectedDispatchStep::Admit,
    )
    .unwrap();
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
