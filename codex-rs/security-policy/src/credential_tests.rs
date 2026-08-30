use std::collections::BTreeMap;

use pretty_assertions::assert_eq;

use super::*;

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("valid bounded text")
}

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human:jim").expect("valid human")
}

fn actors(agent_id: &str) -> ActorChain {
    ActorChain::new(vec![
        human(),
        PolicyPrincipal::new(PrincipalKind::Agent, agent_id).expect("valid agent"),
    ])
    .expect("valid actor chain")
}

fn capability_request(
    host: &str,
    method: CredentialHttpMethod,
    purpose: &str,
    scope: &str,
    path: &str,
    revocations: &RevocationState,
) -> CredentialCapabilityRequest {
    let destination = CredentialDestination::https(host, 443).expect("valid destination");
    let credential = CredentialReference::new("provider.openai", scope).expect("valid reference");
    let authorization = AuthorizationRequest::new(
        actors("agent:root"),
        ProtectedResource::new(ResourceKind::VaultCredential, "provider.openai")
            .expect("valid resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: text("session:security-test"),
            task_id: text("task:responses"),
            purpose: text(purpose),
            operation: credential.scope.clone(),
            destination: Some(destination.authority().expect("canonical authority")),
            quantity: None,
            grant_id: None,
        },
    )
    .expect("valid authorization");
    let grant = BoundedGrant::issue(
        human(),
        actors("agent:root"),
        GrantScope::new(
            authorization.resource.clone(),
            [PolicyAction::Use],
            GrantContext::new(
                authorization.context.session_id.clone(),
                authorization.context.task_id.clone(),
                authorization.context.purpose.clone(),
                authorization.context.operation.clone(),
            ),
            authorization.context.destination.clone(),
            BTreeMap::new(),
        )
        .expect("valid scope"),
        90,
        200,
        text("credential-grant-nonce"),
    )
    .expect("valid grant");
    CredentialCapabilityRequest::new(
        authorization,
        grant,
        credential,
        method,
        destination,
        path,
        100,
        180,
        revocations,
        None,
    )
    .expect("valid capability request")
}

fn usage_limits(requests: u64, tokens: u64, bytes: u64, spend: u64) -> BTreeMap<BoundedText, u64> {
    BTreeMap::from([
        (text("requests"), requests),
        (text("tokens"), tokens),
        (text("bytes"), bytes),
        (text("spend_microunits"), spend),
    ])
}

fn metered_capability_request(
    model: &str,
    revocations: &RevocationState,
) -> CredentialCapabilityRequest {
    let destination = CredentialDestination::https("api.openai.com", 443).expect("destination");
    let credential =
        CredentialReference::new("provider.openai", "responses.create").expect("reference");
    let aggregate = usage_limits(
        /*requests*/ 2, /*tokens*/ 150, /*bytes*/ 1_500, /*spend*/ 150,
    );
    let per_request = usage_limits(
        /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
    );
    let authorization = AuthorizationRequest::new(
        actors("agent:root"),
        ProtectedResource::new(ResourceKind::VaultCredential, "provider.openai").expect("resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: text("session:security-test"),
            task_id: text("task:responses"),
            purpose: text("model-inference"),
            operation: credential.scope.clone(),
            destination: Some(destination.authority().expect("authority")),
            quantity: Some(
                QuantitativeLimit::new("credential.aggregate.requests", 2).expect("quantity"),
            ),
            grant_id: None,
        },
    )
    .expect("authorization");
    let mut grant_limits = BTreeMap::new();
    for (scope, limits) in [("per_request", &per_request), ("aggregate", &aggregate)] {
        for (dimension, limit) in limits {
            grant_limits.insert(text(&format!("credential.{scope}.{dimension}")), *limit);
        }
    }
    let grant = BoundedGrant::issue(
        human(),
        actors("agent:root"),
        GrantScope::new(
            authorization.resource.clone(),
            [PolicyAction::Use],
            GrantContext::new(
                authorization.context.session_id.clone(),
                authorization.context.task_id.clone(),
                authorization.context.purpose.clone(),
                authorization.context.operation.clone(),
            )
            .with_model(text(model)),
            authorization.context.destination.clone(),
            grant_limits,
        )
        .expect("grant scope"),
        90,
        200,
        text("metered-credential-grant-nonce"),
    )
    .expect("grant");
    CredentialCapabilityRequest::new_with_usage_limits(
        authorization,
        grant,
        credential,
        CredentialHttpMethod::Post,
        destination,
        "/v1/responses",
        100,
        180,
        revocations,
        None,
        model,
        per_request,
        aggregate,
    )
    .expect("usage limits")
}

#[test]
fn credential_request_is_a_complete_secret_free_authority_object() {
    let revocations = RevocationState::new();
    let request = capability_request(
        "API.OpenAI.COM",
        CredentialHttpMethod::Post,
        "model-inference",
        "responses.create",
        "/v1/responses",
        &revocations,
    );

    assert_eq!(
        request,
        CredentialCapabilityRequest {
            schema_version: CREDENTIAL_CAPABILITY_SCHEMA_VERSION,
            authorization: request.authorization.clone(),
            grant: request.grant.clone(),
            credential: CredentialReference::new("provider.openai", "responses.create")
                .expect("reference"),
            method: CredentialHttpMethod::Post,
            destination: CredentialDestination::https("api.openai.com", 443).expect("destination"),
            path: text("/v1/responses"),
            issued_at_unix_seconds: 100,
            expires_at_unix_seconds: 180,
            revocation_generation: 0,
            triggering_receipt: None,
            usage_schema_version: None,
            model: None,
            per_request_usage_limits: BTreeMap::new(),
            aggregate_usage_limits: BTreeMap::new(),
        }
    );
    assert_eq!(
        request.decision().expect("decision").matched_grant_id,
        Some(request.grant.grant_id.clone())
    );

    let serialized = serde_json::to_string(&request).expect("serialize");
    for forbidden in ["sk-", "bearer", "secret_value", "private_key"] {
        assert!(!serialized.to_ascii_lowercase().contains(forbidden));
    }
}

#[test]
fn credential_request_digest_binds_every_authority_dimension() {
    let revocations = RevocationState::new();
    let original = capability_request(
        "api.openai.com",
        CredentialHttpMethod::Post,
        "model-inference",
        "responses.create",
        "/v1/responses",
        &revocations,
    );
    let variants = [
        capability_request(
            "api.openai.com",
            CredentialHttpMethod::Get,
            "model-inference",
            "responses.create",
            "/v1/responses",
            &revocations,
        ),
        capability_request(
            "api.openai.com",
            CredentialHttpMethod::Post,
            "portfolio-disclosure",
            "responses.create",
            "/v1/responses",
            &revocations,
        ),
        capability_request(
            "api.openai.com",
            CredentialHttpMethod::Post,
            "model-inference",
            "responses.admin",
            "/v1/responses",
            &revocations,
        ),
        capability_request(
            "api.openai.com",
            CredentialHttpMethod::Post,
            "model-inference",
            "responses.create",
            "/v1/files",
            &revocations,
        ),
        capability_request(
            "api.openai.com.evil.test",
            CredentialHttpMethod::Post,
            "model-inference",
            "responses.create",
            "/v1/responses",
            &revocations,
        ),
    ];

    let original_digest = original.digest().expect("digest");
    for variant in variants {
        assert_ne!(variant.digest().expect("variant digest"), original_digest);
    }
}

#[test]
fn credential_request_rejects_invalid_authority_and_lifecycle() {
    let revocations = RevocationState::new();
    let request = capability_request(
        "api.openai.com",
        CredentialHttpMethod::Post,
        "model-inference",
        "responses.create",
        "/v1/responses",
        &revocations,
    );

    let mut wrong_actor = request.clone();
    wrong_actor.authorization.subject = actors("agent:other");
    assert!(matches!(
        wrong_actor.validate(),
        Err(CredentialCapabilityError::GrantMismatch)
    ));

    let mut wrong_label = request.clone();
    wrong_label.credential.label = text("provider.other");
    assert!(matches!(
        wrong_label.validate(),
        Err(CredentialCapabilityError::CredentialAuthorityMismatch)
    ));

    let mut bad_time = request.clone();
    bad_time.expires_at_unix_seconds = bad_time.issued_at_unix_seconds;
    assert!(matches!(
        bad_time.validate(),
        Err(CredentialCapabilityError::InvalidExpiry)
    ));

    assert!(matches!(
        request.validate_at(99, &revocations),
        Err(CredentialCapabilityError::ExpiredOrNotYetValid)
    ));
    assert!(matches!(
        request.validate_at(180, &revocations),
        Err(CredentialCapabilityError::ExpiredOrNotYetValid)
    ));
}

#[test]
fn credential_request_fails_on_revocation_and_generation_change() {
    let mut revocations = RevocationState::new();
    let request = capability_request(
        "api.openai.com",
        CredentialHttpMethod::Post,
        "model-inference",
        "responses.create",
        "/v1/responses",
        &revocations,
    );
    let event = RevocationEvent::new(
        human(),
        RevocationTarget::Grant {
            grant_id: request.grant.grant_id.clone(),
        },
        RevocationReason::HumanRequest,
        120,
    )
    .expect("valid event");
    revocations.apply(&event).expect("apply revocation");

    assert!(matches!(
        request.validate_at(121, &revocations),
        Err(CredentialCapabilityError::StaleRevocationGeneration)
    ));

    let mut same_generation = request;
    same_generation.revocation_generation = revocations.generation;
    assert!(matches!(
        same_generation.validate_at(121, &revocations),
        Err(CredentialCapabilityError::Revoked)
    ));
}

#[test]
fn malformed_or_ambiguous_credential_metadata_fails_closed() {
    assert!(CredentialDestination::https("127.0.0.1", 443).is_err());
    assert!(CredentialDestination::https("api.openai.com.", 443).is_err());
    assert!(CredentialDestination::https("user@api.openai.com", 443).is_err());
    assert!(CredentialReference::new("provider openai", "responses.create").is_err());

    let revocations = RevocationState::new();
    let request = capability_request(
        "api.openai.com",
        CredentialHttpMethod::Post,
        "model-inference",
        "responses.create",
        "/v1/responses",
        &revocations,
    );
    let mut value = serde_json::to_value(&request).expect("serialize");
    value["unexpected"] = serde_json::json!("credential-canary");
    let error = serde_json::from_value::<CredentialCapabilityRequest>(value)
        .expect_err("unknown fields fail")
        .to_string();
    assert!(!error.contains("credential-canary"));

    let mut noncanonical = serde_json::to_value(&request).expect("serialize");
    noncanonical["destination"]["host"] = serde_json::json!("API.OpenAI.COM");
    let decoded: CredentialCapabilityRequest =
        serde_json::from_value(noncanonical).expect("typed decode");
    assert!(matches!(
        decoded.validate(),
        Err(CredentialCapabilityError::NonCanonicalDestinationHost)
    ));
}

#[test]
fn capability_id_is_a_digest_identifier_not_a_bearer_value() {
    let digest = "a".repeat(CAPABILITY_ID_HEX_LENGTH);
    let id = CapabilityId::from_sha256_hex(digest.clone()).expect("valid digest");
    assert_eq!(id.as_str(), digest);
    assert_eq!(
        serde_json::to_string(&id).expect("serialize"),
        format!("\"{digest}\"")
    );

    for invalid in [
        "abc",
        &"A".repeat(CAPABILITY_ID_HEX_LENGTH),
        &"g".repeat(CAPABILITY_ID_HEX_LENGTH),
    ] {
        assert!(CapabilityId::from_sha256_hex(invalid).is_err());
    }
}

#[test]
fn credential_usage_policy_is_grant_bound_and_digest_sensitive() {
    let revocations = RevocationState::new();
    let request = metered_capability_request("gpt-5.5", &revocations);
    assert!(request.has_usage_limits());
    assert_eq!(request.per_request_usage_limit("requests"), Some(1));
    assert_eq!(request.aggregate_usage_limit("tokens"), Some(150));

    let changed_model = metered_capability_request("claude-opus-5", &revocations);
    assert_ne!(
        request.digest().expect("request digest"),
        changed_model.digest().expect("changed digest")
    );

    let mut forged_model = request.clone();
    forged_model.model = Some(text("claude-opus-5"));
    assert!(matches!(
        forged_model.validate(),
        Err(CredentialCapabilityError::GrantMismatch)
    ));

    let mut forged_limit = request;
    forged_limit
        .aggregate_usage_limits
        .insert(text("tokens"), 151);
    assert!(matches!(
        forged_limit.validate(),
        Err(CredentialCapabilityError::GrantMismatch)
    ));

    let mut forged_extra_limit = changed_model;
    let mut extra_scope = forged_extra_limit.grant.scope.clone();
    extra_scope
        .quantitative_limits
        .insert(text("credential.aggregate.requests.backdoor"), 1);
    forged_extra_limit.grant = BoundedGrant::issue(
        forged_extra_limit.grant.issuer.clone(),
        forged_extra_limit.grant.actor_chain.clone(),
        extra_scope,
        forged_extra_limit.grant.issued_at_unix_seconds,
        forged_extra_limit.grant.expires_at_unix_seconds,
        text("metered-extra-limit-nonce"),
    )
    .expect("internally valid grant with extra usage dimension");
    assert!(matches!(
        forged_extra_limit.validate(),
        Err(CredentialCapabilityError::GrantMismatch)
    ));
}
