use std::collections::BTreeMap;
use std::collections::HashMap;
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

fn store(
    capacity: usize,
    clock: TestClock,
) -> CredentialCapabilityStore<TestClock, CounterEntropy> {
    CredentialCapabilityStore::with_sources(capacity, clock, CounterEntropy::default())
        .expect("store")
}

#[test]
fn issued_capability_authorizes_only_the_complete_bound_request() {
    let revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock);
    let request = standard_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let authorized = store
        .authorize(&capability, &request, &revocations)
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
    assert_eq!(store.len().expect("length"), 1);
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
            store.authorize(&capability, &variant, &revocations),
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
        store.authorize(&forged, &request, &revocations),
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
    assert!(
        store
            .authorize(&capability, &request, &revocations)
            .is_err()
    );
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
    assert!(
        store
            .authorize(&capability, &request, &revocations)
            .is_err()
    );
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
fn network_proxy_spec_resolves_authorized_openai_credential_only_at_injection() {
    let revocations = RevocationState::new();
    let clock = TestClock::new(100);
    let store = store(4, clock.clone());
    let request = standard_request(&revocations);
    let capability = store
        .issue(request.clone(), &revocations)
        .expect("issue capability");
    let authorized = store
        .authorize(&capability, &request, &revocations)
        .expect("authorize capability");

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
            secret: "sk-core-scoped-canary".to_string(),
        })
        .expect("add scoped credential");

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
            Arc::new(RwLock::new(revocations)),
            clock,
        )
        .expect("scoped network proxy state");

    let mut env = HashMap::from([(
        "OPENAI_API_KEY".to_string(),
        "sk-untrusted-environment-value".to_string(),
    )]);
    state.virtualize_child_credentials(&mut env);
    let dummy = env.get("OPENAI_API_KEY").expect("scoped dummy").to_string();
    assert_ne!(dummy, "sk-untrusted-environment-value");

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
    assert_eq!(
        headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        Some("Bearer sk-core-scoped-canary")
    );
}
