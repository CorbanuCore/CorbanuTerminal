use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::config::NetworkProxySpec;
use codex_keyring_store::tests::MockKeyringStore;
use codex_network_proxy::NetworkProxyAuditMetadata;
use codex_network_proxy::NetworkProxyConfig;
use codex_network_proxy::ScopedCredentialInjectionError;
use codex_protocol::models::PermissionProfile;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::CredentialDestination;
use codex_security_policy::CredentialHttpMethod;
use codex_security_policy::CredentialReference;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedResource;
use codex_security_policy::QuantitativeLimit;
use codex_security_policy::ResourceKind;
use codex_security_policy::RevocationEvent;
use codex_security_policy::RevocationReason;
use codex_security_policy::RevocationState;
use codex_security_policy::RevocationTarget;
use codex_vault::AddCredential;
use codex_vault::CredentialType;
use codex_vault::Vault;
use http::HeaderMap;
use http::HeaderValue;
use http::header::AUTHORIZATION;
use pretty_assertions::assert_eq;
use sha2::Digest as _;
use sha2::Sha256;
use tracing_test::traced_test;
use uuid::Uuid;
use walkdir::WalkDir;

use super::*;

#[derive(Clone)]
struct TestClock {
    now: Arc<AtomicI64>,
    fail: Arc<AtomicBool>,
}

impl TestClock {
    fn new(now: i64) -> Self {
        Self {
            now: Arc::new(AtomicI64::new(now)),
            fail: Arc::new(AtomicBool::new(false)),
        }
    }

    fn set(&self, now: i64) {
        self.now.store(now, Ordering::SeqCst);
    }
}

impl CredentialClock for TestClock {
    fn now_unix_seconds(&self) -> Result<i64, CredentialCapabilityStoreError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(CredentialCapabilityStoreError::ClockOverflow);
        }
        Ok(self.now.load(Ordering::SeqCst))
    }
}

#[derive(Default)]
struct CounterEntropy {
    next: AtomicU64,
    fail: AtomicBool,
    constant: AtomicBool,
}

impl CredentialEntropy for CounterEntropy {
    fn fill_token(
        &self,
        token: &mut [u8; CAPABILITY_TOKEN_BYTES],
    ) -> Result<(), CredentialCapabilityStoreError> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(CredentialCapabilityStoreError::EntropyUnavailable);
        }
        let value = if self.constant.load(Ordering::SeqCst) {
            7
        } else {
            self.next.fetch_add(1, Ordering::SeqCst) + 1
        };
        token.fill(0);
        token[..8].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

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

#[allow(clippy::too_many_arguments)]
fn request(
    agent_id: &str,
    purpose: &str,
    scope: &str,
    method: CredentialHttpMethod,
    host: &str,
    path: &str,
    label: &str,
    revocations: &RevocationState,
) -> CredentialCapabilityRequest {
    let destination = CredentialDestination::https(host, 443).expect("destination");
    let credential = CredentialReference::new(label, scope).expect("reference");
    let authorization = AuthorizationRequest::new(
        actors(agent_id),
        ProtectedResource::new(ResourceKind::VaultCredential, label).expect("resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: text("session:credential-test"),
            task_id: text("task:credential-test"),
            purpose: text(purpose),
            operation: credential.scope.clone(),
            destination: Some(destination.authority().expect("authority")),
            quantity: None,
            grant_id: None,
        },
    )
    .expect("authorization");
    let grant = BoundedGrant::issue(
        human(),
        actors(agent_id),
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
        .expect("grant scope"),
        90,
        200,
        text("credential-grant-nonce"),
    )
    .expect("grant");
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
    .expect("capability request")
}

fn standard_request(revocations: &RevocationState) -> CredentialCapabilityRequest {
    request(
        "agent:root",
        "model-inference",
        "responses.create",
        CredentialHttpMethod::Post,
        "api.openai.com",
        "/v1/responses",
        "provider.openai",
        revocations,
    )
}

fn usage(requests: u64, tokens: u64, bytes: u64, spend_microunits: u64) -> CredentialUsage {
    CredentialUsage::new(requests, tokens, bytes, spend_microunits)
}

#[allow(clippy::too_many_arguments)]
fn metered_request(
    agent_id: &str,
    scope: &str,
    model: &str,
    host: &str,
    path: &str,
    label: &str,
    per_request: CredentialUsage,
    aggregate: CredentialUsage,
    revocations: &RevocationState,
) -> CredentialCapabilityRequest {
    let destination = CredentialDestination::https(host, 443).expect("destination");
    let credential = CredentialReference::new(label, scope).expect("reference");
    let authorization = AuthorizationRequest::new(
        actors(agent_id),
        ProtectedResource::new(ResourceKind::VaultCredential, label).expect("resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: text("session:credential-test"),
            task_id: text("task:credential-test"),
            purpose: text("model-inference"),
            operation: credential.scope.clone(),
            destination: Some(destination.authority().expect("authority")),
            quantity: Some(
                QuantitativeLimit::new("credential.aggregate.requests", aggregate.requests)
                    .expect("quantity"),
            ),
            grant_id: None,
        },
    )
    .expect("authorization");
    let limits = |value: CredentialUsage| {
        BTreeMap::from([
            (text("requests"), value.requests),
            (text("tokens"), value.tokens),
            (text("bytes"), value.bytes),
            (text("spend_microunits"), value.spend_microunits),
        ])
    };
    let per_request_limits = limits(per_request);
    let aggregate_limits = limits(aggregate);
    let mut grant_limits = BTreeMap::new();
    for (scope, limits) in [
        ("per_request", &per_request_limits),
        ("aggregate", &aggregate_limits),
    ] {
        for (dimension, limit) in limits {
            grant_limits.insert(text(&format!("credential.{scope}.{dimension}")), *limit);
        }
    }
    let grant = BoundedGrant::issue(
        human(),
        actors(agent_id),
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
        path,
        100,
        180,
        revocations,
        None,
        model,
        per_request_limits,
        aggregate_limits,
    )
    .expect("usage policy")
}

fn standard_metered_request(revocations: &RevocationState) -> CredentialCapabilityRequest {
    metered_request(
        "agent:root",
        "responses.create",
        "gpt-5.5",
        "api.openai.com",
        "/v1/responses",
        "provider.openai",
        usage(
            /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
        ),
        usage(
            /*requests*/ 2, /*tokens*/ 150, /*bytes*/ 1_500, /*spend*/ 150,
        ),
        revocations,
    )
}

fn store(
    capacity: usize,
    clock: TestClock,
) -> CredentialCapabilityStore<TestClock, CounterEntropy> {
    CredentialCapabilityStore::with_sources(capacity, clock, CounterEntropy::default())
        .expect("store")
}

fn sha256_hex(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("encode digest");
    }
    encoded
}

fn assert_canary_absent(surface: &str, value: &[u8], canary: &str) {
    assert!(
        !value
            .windows(canary.len())
            .any(|candidate| candidate == canary.as_bytes()),
        "credential canary escaped into {surface}"
    );
}

#[test]
fn issued_capability_is_consumed_only_for_the_complete_bound_request() {
    let revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock);
    let request = standard_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let authorized = store
        .consume(&capability, &request, &revocations)
        .expect("authorize exact request");

    assert_eq!(
        authorized,
        AuthorizedCredentialCapability {
            capability_id: capability.capability_id().clone(),
            request: request.clone(),
        }
    );
    let vault_reference = authorized.into_vault_ref().expect("vault reference");
    assert_eq!(vault_reference.label(), "provider.openai");
    assert_eq!(vault_reference.scope(), "responses.create");
    assert_eq!(
        capability.decision(),
        &request.decision().expect("matching decision")
    );
    let debug = format!("{capability:?}");
    assert!(debug.contains("CapabilityToken(<redacted>)"));
    assert!(!debug.contains("model-inference"));
    assert_eq!(store.len().expect("length"), 0);
    assert!(matches!(
        store.consume(&capability, &request, &revocations),
        Err(CredentialCapabilityStoreError::UnknownCapability)
    ));
}

#[test]
fn concurrent_duplicate_consumption_allows_exactly_one_use() {
    let revocations = Arc::new(RevocationState::new());
    let request = Arc::new(standard_request(&revocations));
    let store = Arc::new(store(4, TestClock::new(100)));
    let capability = Arc::new(
        store
            .issue(request.as_ref().clone(), &revocations)
            .expect("issue capability"),
    );
    let barrier = Arc::new(std::sync::Barrier::new(8));
    let mut workers = Vec::new();

    for _ in 0..8 {
        let revocations = Arc::clone(&revocations);
        let request = Arc::clone(&request);
        let store = Arc::clone(&store);
        let capability = Arc::clone(&capability);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            store.consume(&capability, &request, &revocations).is_ok()
        }));
    }

    assert_eq!(
        workers
            .into_iter()
            .map(|worker| worker.join().expect("worker"))
            .filter(|success| *success)
            .count(),
        1
    );
    assert_eq!(store.len().expect("length"), 0);
}

#[test]
fn capability_authority_does_not_survive_runtime_restart() {
    let revocations = RevocationState::new();
    let request = standard_request(&revocations);
    let original_store = store(4, TestClock::new(100));
    let capability = original_store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let restarted_store = store(4, TestClock::new(100));

    assert!(matches!(
        restarted_store.consume(&capability, &request, &revocations),
        Err(CredentialCapabilityStoreError::UnknownCapability)
    ));
    assert_eq!(original_store.len().expect("original length"), 1);
    assert_eq!(restarted_store.len().expect("restarted length"), 0);
}

#[test]
fn adjacent_actor_purpose_operation_method_host_path_and_scope_fail() {
    let revocations = RevocationState::new();
    let store = store(8, TestClock::new(100));
    let original = standard_request(&revocations);
    let capability = store
        .issue(original, &revocations)
        .expect("issue capability");
    let variants = [
        request(
            "agent:other",
            "model-inference",
            "responses.create",
            CredentialHttpMethod::Post,
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            &revocations,
        ),
        request(
            "agent:root",
            "portfolio-disclosure",
            "responses.create",
            CredentialHttpMethod::Post,
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            &revocations,
        ),
        request(
            "agent:root",
            "model-inference",
            "responses.admin",
            CredentialHttpMethod::Post,
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            &revocations,
        ),
        request(
            "agent:root",
            "model-inference",
            "responses.create",
            CredentialHttpMethod::Get,
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            &revocations,
        ),
        request(
            "agent:root",
            "model-inference",
            "responses.create",
            CredentialHttpMethod::Post,
            "api.openai.com.evil.test",
            "/v1/responses",
            "provider.openai",
            &revocations,
        ),
        request(
            "agent:root",
            "model-inference",
            "responses.create",
            CredentialHttpMethod::Post,
            "api.openai.com",
            "/v1/files",
            "provider.openai",
            &revocations,
        ),
        request(
            "agent:root",
            "model-inference",
            "responses.create/admin",
            CredentialHttpMethod::Post,
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            &revocations,
        ),
    ];

    for variant in variants {
        assert!(matches!(
            store.consume(&capability, &variant, &revocations),
            Err(CredentialCapabilityStoreError::AuthorityMismatch)
        ));
    }
}

#[test]
fn forged_bearer_and_public_id_alone_cannot_authorize() {
    let revocations = RevocationState::new();
    let store = store(4, TestClock::new(100));
    let request = standard_request(&revocations);
    let issued = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let forged = IssuedCredentialCapability {
        capability_id: issued.capability_id().clone(),
        token: CapabilityToken([0x55; CAPABILITY_TOKEN_BYTES]),
        decision: issued.decision().clone(),
    };

    assert!(matches!(
        store.consume(&forged, &request, &revocations),
        Err(CredentialCapabilityStoreError::ForgedCapability)
    ));
    assert!(issued.capability_id().as_str().len() == 64);
}

#[test]
fn expiry_and_revocation_remove_authority_before_reuse() {
    let mut revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock.clone());
    let request = standard_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");

    clock.set(180);
    assert!(store.consume(&capability, &request, &revocations).is_err());
    assert_eq!(store.purge(&revocations).expect("purge expired"), 1);
    assert_eq!(store.len().expect("length"), 0);

    clock.set(100);
    let request = standard_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue replacement");
    let event = RevocationEvent::new(
        human(),
        RevocationTarget::Grant {
            grant_id: request.grant.grant_id.clone(),
        },
        RevocationReason::HumanRequest,
        110,
    )
    .expect("revocation");
    revocations.apply(&event).expect("apply");
    clock.set(111);
    assert!(store.consume(&capability, &request, &revocations).is_err());
    assert_eq!(store.purge(&revocations).expect("purge revoked"), 1);
}

#[test]
fn capacity_is_hard_bounded_and_cleanup_reclaims_space() {
    let revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(1, clock.clone());
    let request = standard_request(&revocations);
    let _capability = store
        .issue(request.clone(), &revocations)
        .expect("first issue");
    assert!(matches!(
        store.issue(request.clone(), &revocations),
        Err(CredentialCapabilityStoreError::CapacityReached { capacity: 1 })
    ));

    clock.set(180);
    assert_eq!(store.purge(&revocations).expect("purge"), 1);
    clock.set(100);
    store
        .issue(request, &revocations)
        .expect("capacity reclaimed");
    assert!(CredentialCapabilityStore::new(0).is_err());
    assert!(CredentialCapabilityStore::new(MAX_CREDENTIAL_CAPABILITIES + 1).is_err());
}

#[test]
fn concurrent_issuance_never_aliases_capability_ids() {
    let revocations = Arc::new(RevocationState::new());
    let store = Arc::new(store(32, TestClock::new(100)));
    let mut workers = Vec::new();
    for _ in 0..32 {
        let store = Arc::clone(&store);
        let revocations = Arc::clone(&revocations);
        workers.push(std::thread::spawn(move || {
            let request = standard_request(&revocations);
            store
                .issue(request, &revocations)
                .expect("concurrent issue")
                .capability_id()
                .clone()
        }));
    }

    let ids = workers
        .into_iter()
        .map(|worker| worker.join().expect("worker"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 32);
    assert_eq!(store.len().expect("length"), 32);
}

#[test]
fn clock_entropy_collision_and_lock_failures_are_fail_closed() {
    let revocations = RevocationState::new();
    let clock = TestClock::new(100);
    clock.fail.store(true, Ordering::SeqCst);
    let failing_clock_store = store(1, clock);
    assert!(matches!(
        failing_clock_store.issue(standard_request(&revocations), &revocations),
        Err(CredentialCapabilityStoreError::ClockOverflow)
    ));

    let entropy = CounterEntropy::default();
    entropy.fail.store(true, Ordering::SeqCst);
    let failing_entropy_store =
        CredentialCapabilityStore::with_sources(1, TestClock::new(100), entropy).expect("store");
    assert!(matches!(
        failing_entropy_store.issue(standard_request(&revocations), &revocations),
        Err(CredentialCapabilityStoreError::EntropyUnavailable)
    ));

    let collision_entropy = CounterEntropy::default();
    collision_entropy.constant.store(true, Ordering::SeqCst);
    let collision_store =
        CredentialCapabilityStore::with_sources(2, TestClock::new(100), collision_entropy)
            .expect("store");
    collision_store
        .issue(standard_request(&revocations), &revocations)
        .expect("first issue");
    assert!(matches!(
        collision_store.issue(standard_request(&revocations), &revocations),
        Err(CredentialCapabilityStoreError::TokenCollision)
    ));

    let poisoned = store(1, TestClock::new(100));
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = poisoned.entries.write().expect("lock");
        panic!("poison credential store");
    }));
    assert!(matches!(
        poisoned.len(),
        Err(CredentialCapabilityStoreError::StorePoisoned)
    ));
}

#[test]
fn usage_reservation_enforces_limits_and_retry_cannot_reset_spent_authority() {
    let revocations = RevocationState::new();
    let store = store(4, TestClock::new(100));
    let request = standard_metered_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue metered capability");

    assert!(matches!(
        store.consume(&capability, &request, &revocations),
        Err(CredentialCapabilityStoreError::UsageReservationRequired)
    ));
    assert!(matches!(
        store.reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 101, /*bytes*/ 1_000, /*spend*/ 100
            ),
            &revocations,
        ),
        Err(CredentialCapabilityStoreError::PerRequestUsageExceeded)
    ));

    let first = store
        .reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
            ),
            &revocations,
        )
        .expect("reserve first request");
    let first_settlement = store
        .settle(
            &first,
            TrustedCredentialMetering::partial(usage(
                /*requests*/ 1, /*tokens*/ 60, /*bytes*/ 600, /*spend*/ 60,
            )),
            &revocations,
        )
        .expect("settle partial response");
    assert_eq!(first_settlement.outcome, CredentialUsageOutcome::Partial);
    assert_eq!(
        first_settlement.charged,
        usage(
            /*requests*/ 1, /*tokens*/ 60, /*bytes*/ 600, /*spend*/ 60
        )
    );

    let retry = store
        .reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 90, /*bytes*/ 900, /*spend*/ 90,
            ),
            &revocations,
        )
        .expect("reserve retry against remaining aggregate budget");
    assert!(matches!(
        store.reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 1, /*bytes*/ 1, /*spend*/ 1
            ),
            &revocations,
        ),
        Err(CredentialCapabilityStoreError::AggregateUsageExceeded)
    ));
    let cancellation = store
        .settle(&retry, TrustedCredentialMetering::cancelled(), &revocations)
        .expect("settle cancelled retry");
    assert_eq!(
        cancellation.charged,
        usage(
            /*requests*/ 1, /*tokens*/ 0, /*bytes*/ 0, /*spend*/ 0
        )
    );
    assert_eq!(
        store
            .settle(&retry, TrustedCredentialMetering::unknown(), &revocations)
            .expect("duplicate settlement is idempotent"),
        cancellation
    );
    assert!(matches!(
        store.reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 1, /*bytes*/ 1, /*spend*/ 1
            ),
            &revocations,
        ),
        Err(CredentialCapabilityStoreError::AggregateUsageExceeded)
    ));
}

#[test]
fn concurrent_usage_reservations_cannot_overcommit() {
    let revocations = Arc::new(RevocationState::new());
    let request = Arc::new(metered_request(
        "agent:root",
        "responses.create",
        "gpt-5.5",
        "api.openai.com",
        "/v1/responses",
        "provider.openai",
        usage(
            /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
        ),
        usage(
            /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
        ),
        &revocations,
    ));
    let store = Arc::new(store(4, TestClock::new(100)));
    let capability = Arc::new(
        store
            .issue(request.as_ref().clone(), &revocations)
            .expect("issue capability"),
    );
    let barrier = Arc::new(std::sync::Barrier::new(8));
    let mut workers = Vec::new();
    for _ in 0..8 {
        let barrier = Arc::clone(&barrier);
        let capability = Arc::clone(&capability);
        let request = Arc::clone(&request);
        let revocations = Arc::clone(&revocations);
        let store = Arc::clone(&store);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            store
                .reserve(
                    &capability,
                    &request,
                    usage(
                        /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000,
                        /*spend*/ 100,
                    ),
                    &revocations,
                )
                .is_ok()
        }));
    }
    assert_eq!(
        workers
            .into_iter()
            .map(|worker| worker.join().expect("worker"))
            .filter(|reserved| *reserved)
            .count(),
        1
    );
}

#[test]
fn trusted_settlement_rejects_forgery_and_unknown_usage_charges_the_full_reservation() {
    let revocations = RevocationState::new();
    let store = store(4, TestClock::new(100));
    let request = standard_metered_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let reservation = store
        .reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
            ),
            &revocations,
        )
        .expect("reserve usage");
    let forged = IssuedCredentialReservation {
        capability_id: capability.capability_id().clone(),
        reservation_id: reservation.reservation_id().clone(),
        token: ReservationToken([0x55; CAPABILITY_TOKEN_BYTES]),
        request: request,
        reserved: reservation.reserved(),
    };
    assert!(matches!(
        store.settle(&forged, TrustedCredentialMetering::unknown(), &revocations),
        Err(CredentialCapabilityStoreError::ForgedReservation)
    ));
    assert!(matches!(
        store.settle(
            &reservation,
            TrustedCredentialMetering::completed(usage(
                /*requests*/ 1, /*tokens*/ 101, /*bytes*/ 1_000, /*spend*/ 100,
            )),
            &revocations,
        ),
        Err(CredentialCapabilityStoreError::MeteringExceedsReservation)
    ));
    let settlement = store
        .settle(
            &reservation,
            TrustedCredentialMetering::unknown(),
            &revocations,
        )
        .expect("settle unknown usage");
    assert_eq!(settlement.charged, reservation.reserved());
    assert_eq!(
        store
            .settle(
                &reservation,
                TrustedCredentialMetering::cancelled(),
                &revocations,
            )
            .expect("duplicate settlement"),
        settlement
    );
    assert!(format!("{reservation:?}").contains("ReservationToken(<redacted>)"));
    assert_eq!(
        reservation
            .vault_ref()
            .expect("opaque broker handoff")
            .label(),
        "provider.openai"
    );
}

#[test]
fn metered_authority_binds_operation_model_resource_and_settles_after_revocation() {
    let mut revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock.clone());
    let request = standard_metered_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let variants = [
        metered_request(
            "agent:root",
            "responses.admin",
            "gpt-5.5",
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            usage(
                /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
            ),
            usage(
                /*requests*/ 2, /*tokens*/ 150, /*bytes*/ 1_500, /*spend*/ 150,
            ),
            &revocations,
        ),
        metered_request(
            "agent:root",
            "responses.create",
            "claude-opus-5",
            "api.openai.com",
            "/v1/responses",
            "provider.openai",
            usage(
                /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
            ),
            usage(
                /*requests*/ 2, /*tokens*/ 150, /*bytes*/ 1_500, /*spend*/ 150,
            ),
            &revocations,
        ),
        metered_request(
            "agent:root",
            "responses.create",
            "gpt-5.5",
            "api.openai.com",
            "/v1/responses",
            "provider.backup",
            usage(
                /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
            ),
            usage(
                /*requests*/ 2, /*tokens*/ 150, /*bytes*/ 1_500, /*spend*/ 150,
            ),
            &revocations,
        ),
    ];
    for variant in variants {
        assert!(matches!(
            store.reserve(
                &capability,
                &variant,
                usage(
                    /*requests*/ 1, /*tokens*/ 1, /*bytes*/ 1, /*spend*/ 1
                ),
                &revocations,
            ),
            Err(CredentialCapabilityStoreError::AuthorityMismatch)
        ));
    }

    let reservation = store
        .reserve(
            &capability,
            &request,
            usage(
                /*requests*/ 1, /*tokens*/ 100, /*bytes*/ 1_000, /*spend*/ 100,
            ),
            &revocations,
        )
        .expect("reserve before revocation");
    revocations
        .apply(
            &RevocationEvent::new(
                human(),
                RevocationTarget::Grant {
                    grant_id: request.grant.grant_id.clone(),
                },
                RevocationReason::HumanRequest,
                110,
            )
            .expect("revocation event"),
        )
        .expect("apply revocation");
    clock.set(181);
    assert!(
        store
            .reserve(
                &capability,
                &request,
                usage(
                    /*requests*/ 1, /*tokens*/ 1, /*bytes*/ 1, /*spend*/ 1
                ),
                &revocations,
            )
            .is_err()
    );
    assert_eq!(store.purge(&revocations).expect("retain pending"), 0);
    assert_eq!(
        store
            .settle(
                &reservation,
                TrustedCredentialMetering::unknown(),
                &revocations,
            )
            .expect("settle pending after revocation")
            .charged,
        reservation.reserved()
    );
    assert_eq!(store.purge(&revocations).expect("purge settled"), 1);
}

#[test]
#[traced_test]
fn credential_authority_unique_canary_is_confined_to_one_outgoing_request() {
    let canary = format!("sk-pf13-{}", Uuid::new_v4());
    let canary_sha256 = sha256_hex(canary.as_bytes());
    let revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock.clone());
    let request = standard_request(&revocations);
    let model_context = serde_json::to_string(&request).expect("serialize authority");
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let capability_debug = format!("{capability:?}");
    let authorized = store
        .consume(&capability, &request, &revocations)
        .expect("authorize capability");
    let authorized_debug = format!("{authorized:?}");
    let tool_payload = serde_json::json!({
        "capability_id": capability.capability_id().as_str(),
        "authority": request,
    })
    .to_string();
    let audit_payload =
        serde_json::to_string(&NetworkProxyAuditMetadata::default()).expect("serialize audit");

    let directory = tempfile::tempdir().expect("vault directory");
    let vault = Arc::new(Vault::new_with_keyring_store(
        directory.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    ));
    vault
        .add(AddCredential {
            label: "provider.openai".to_string(),
            credential_type: CredentialType::BearerToken,
            provider: Some("openai".to_string()),
            notes: None,
            revocation_notes: None,
            secret: canary.clone(),
        })
        .expect("add scoped credential");

    let crash_capability = store
        .issue(standard_request(&revocations), &revocations)
        .expect("issue crash capability");
    let crash_reference = store
        .consume(
            &crash_capability,
            &standard_request(&revocations),
            &revocations,
        )
        .expect("authorize crash capability")
        .into_vault_ref()
        .expect("crash vault reference");
    let crash_error = vault
        .with_scoped_credential(&crash_reference, 100, &revocations, |secret| {
            panic!("credential canary callback crash: {secret}")
        })
        .expect_err("callback panic must be contained");
    assert_eq!(
        crash_error,
        codex_vault::ScopedCredentialError::CallbackPanicked
    );
    let crash_output = format!("{crash_error:?}: {crash_error}");

    let mut config = NetworkProxyConfig {
        enabled: true,
        mitm: true,
        ..NetworkProxyConfig::default()
    };
    config.set_credential_broker_enabled(/*enabled*/ true);
    let spec = NetworkProxySpec::from_config_and_constraints(
        config,
        None,
        &PermissionProfile::workspace_write(),
    )
    .expect("network proxy spec");
    let state = spec
        .build_state_with_scoped_openai_credential_and_clock(
            NetworkProxyAuditMetadata::default(),
            authorized,
            Arc::clone(&vault),
            Arc::new(RwLock::new(revocations)),
            clock,
        )
        .expect("scoped network proxy state");

    let mut child_env = HashMap::from([(
        "OPENAI_API_KEY".to_string(),
        "sk-untrusted-environment-value".to_string(),
    )]);
    state.virtualize_child_credentials(&mut child_env);
    let dummy = child_env
        .get("OPENAI_API_KEY")
        .expect("scoped dummy")
        .to_string();
    assert_ne!(dummy, "sk-untrusted-environment-value");
    assert_ne!(dummy, canary);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {dummy}")).expect("dummy header"),
    );
    state
        .inject_request_credentials(
            "https",
            "api.openai.com",
            443,
            "POST",
            "/v1/responses",
            &mut headers,
        )
        .expect("scoped credential injection");
    let outgoing_authorization = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .expect("outgoing authorization");
    assert_eq!(outgoing_authorization, format!("Bearer {canary}"));
    let provider_request_count = 1;
    headers.remove(AUTHORIZATION);

    let mut replay_headers = HeaderMap::new();
    replay_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {dummy}")).expect("dummy header"),
    );
    let replay_error = state
        .inject_request_credentials(
            "https",
            "api.openai.com",
            443,
            "POST",
            "/v1/responses",
            &mut replay_headers,
        )
        .expect_err("replay must fail before another provider request");
    assert_eq!(replay_error, ScopedCredentialInjectionError::AlreadyUsed);

    for (surface, value) in [
        ("model_context", model_context),
        ("tool_payload", tool_payload),
        (
            "child_environment",
            serde_json::to_string(&child_env).expect("serialize child environment"),
        ),
        ("audit", audit_payload),
        ("errors", format!("{replay_error:?}: {replay_error}")),
        ("crash_output", crash_output),
        ("capability_debug", capability_debug),
        ("authorized_debug", authorized_debug),
    ] {
        assert_canary_absent(surface, value.as_bytes(), &canary);
    }

    for entry in WalkDir::new(directory.path()) {
        let entry = entry.expect("walk vault artifact");
        if entry.file_type().is_file() {
            let bytes = fs::read(entry.path()).expect("read vault artifact");
            assert_canary_absent("vault_artifact", &bytes, &canary);
        }
    }

    assert!(logs_contain("scoped credential action receipt"));
    assert!(logs_contain(capability.capability_id().as_str()));
    assert!(logs_contain("responses.create"));
    assert!(logs_contain("https://api.openai.com:443"));
    assert!(!logs_contain("provider.openai"));
    logs_assert(|lines: &[&str]| {
        if lines.iter().any(|line| line.contains(&canary)) {
            Err("credential canary escaped into tracing logs".to_string())
        } else {
            Ok(())
        }
    });

    let result = serde_json::json!({
        "canary_sha256": canary_sha256,
        "outgoing_request_count": provider_request_count,
        "raw_secret_observations": 1,
        "scanned_surfaces": [
            "exact_outgoing_request_capture",
            "model_context",
            "tool_payloads",
            "child_environment",
            "logs",
            "audit",
            "errors",
            "receipts",
            "crash_output",
            "vault_artifacts"
        ]
    });
    writeln!(
        std::io::stdout().lock(),
        "CORBANU_SECURITY_CREDENTIAL_CANARY {result}"
    )
    .expect("write canary result");
}

#[test]
#[traced_test]
fn credential_authority_revocation_before_resolve_denies_without_vault_access() {
    let initial_revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock.clone());
    let request = standard_request(&initial_revocations);
    let capability = store
        .issue(request.clone(), &initial_revocations)
        .expect("issue capability");
    let authorized = store
        .consume(&capability, &request, &initial_revocations)
        .expect("consume capability");

    let revocations = Arc::new(RwLock::new(initial_revocations));
    revocations
        .write()
        .expect("revocation state")
        .apply(
            &RevocationEvent::new(
                human(),
                RevocationTarget::Grant {
                    grant_id: request.grant.grant_id,
                },
                RevocationReason::HumanRequest,
                101,
            )
            .expect("revocation event"),
        )
        .expect("apply revocation");
    clock.set(102);

    let directory = tempfile::tempdir().expect("vault directory");
    let vault = Arc::new(Vault::new_with_keyring_store(
        directory.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    ));
    let mut config = NetworkProxyConfig {
        enabled: true,
        mitm: true,
        ..NetworkProxyConfig::default()
    };
    config.set_credential_broker_enabled(/*enabled*/ true);
    let spec = NetworkProxySpec::from_config_and_constraints(
        config,
        None,
        &PermissionProfile::workspace_write(),
    )
    .expect("network proxy spec");
    let state = spec
        .build_state_with_scoped_openai_credential_and_clock(
            NetworkProxyAuditMetadata::default(),
            authorized,
            vault,
            revocations,
            clock,
        )
        .expect("scoped network proxy state");

    let mut env = HashMap::new();
    state.virtualize_child_credentials(&mut env);
    let dummy = env.get("OPENAI_API_KEY").expect("scoped dummy");
    let expected_authorization = format!("Bearer {dummy}");
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&expected_authorization).expect("dummy header"),
    );
    assert_eq!(
        state
            .inject_request_credentials(
                "https",
                "api.openai.com",
                443,
                "POST",
                "/v1/responses",
                &mut headers,
            )
            .expect_err("revoked capability"),
        ScopedCredentialInjectionError::ResolutionFailed
    );
    assert_eq!(
        headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        Some(expected_authorization.as_str())
    );
    assert!(logs_contain("scoped credential action receipt"));
    assert!(logs_contain("Revoked"));
    assert!(!logs_contain("provider.openai"));
    assert!(!logs_contain("sk-revoked-canary"));
}
