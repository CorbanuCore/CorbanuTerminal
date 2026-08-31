use std::collections::BTreeMap;
use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
use codex_secret_broker::BrokerAuditError;
use codex_secret_broker::BrokerAuditIntent;
use codex_secret_broker::BrokerAuditResolution;
use codex_secret_broker::BrokerBinding;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::BrokerCredentialGrant;
use codex_secret_broker::BrokerOperation;
use codex_secret_broker::BrokerRuntime;
use codex_secret_broker::BrokerRuntimeConfig;
use codex_secret_broker::DurableBrokerAudit;
use codex_secret_broker::ObservedPeer;
use codex_secret_broker::OpenAiResponsesOperation;
use codex_secret_broker::platform_contract::*;
use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::CapabilityId;
use codex_security_policy::CredentialCapabilityRequest;
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
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tracing_test::traced_test;

use super::*;
use crate::AddCredential;
use crate::CredentialType;

const LABEL: &str = "provider.openai";
const SCOPE: &str = "responses.create";
const SECRET: &str = "SCOPED-SECRET-CANARY-9d45";

fn text(value: &str) -> BoundedText {
    BoundedText::new(value).expect("bounded text")
}

fn human() -> PolicyPrincipal {
    PolicyPrincipal::new(PrincipalKind::Human, "human:jim").expect("human")
}

fn actors() -> ActorChain {
    ActorChain::new(vec![
        human(),
        PolicyPrincipal::new(PrincipalKind::Agent, "agent:root").expect("agent"),
    ])
    .expect("actor chain")
}

fn request(label: &str, scope: &str, revocations: &RevocationState) -> CredentialCapabilityRequest {
    let destination = CredentialDestination::https("api.openai.com", 443).expect("destination");
    let credential = CredentialReference::new(label, scope).expect("reference");
    let authorization = AuthorizationRequest::new(
        actors(),
        ProtectedResource::new(ResourceKind::VaultCredential, label).expect("resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: text("session:scoped-vault"),
            task_id: text("task:scoped-vault"),
            purpose: text("model-inference"),
            operation: credential.scope.clone(),
            destination: Some(destination.authority().expect("authority")),
            quantity: None,
            grant_id: None,
        },
    )
    .expect("authorization");
    let grant = BoundedGrant::issue(
        human(),
        actors(),
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
        text("scoped-vault-grant"),
    )
    .expect("grant");
    CredentialCapabilityRequest::new(
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
    )
    .expect("request")
}

fn reference(request: CredentialCapabilityRequest) -> VaultCredentialRef {
    VaultCredentialRef::from_authorized(
        CapabilityId::from_sha256_hex("a".repeat(64)).expect("capability id"),
        request,
    )
    .expect("authorized reference")
}

fn test_vault(credential_type: CredentialType) -> (tempfile::TempDir, Vault) {
    let directory = tempfile::tempdir().expect("tempdir");
    let keyring = Arc::new(MockKeyringStore::default());
    let vault = Vault::new_with_keyring_store(directory.path().to_path_buf(), keyring);
    vault
        .add(AddCredential {
            label: LABEL.to_string(),
            credential_type,
            provider: Some("openai".to_string()),
            notes: None,
            revocation_notes: None,
            secret: SECRET.to_string(),
        })
        .expect("add credential");
    (directory, vault)
}

#[test]
#[traced_test]
fn scoped_resolution_exposes_secret_only_inside_redacted_callback() {
    let (_directory, vault) = test_vault(CredentialType::BearerToken);
    let revocations = RevocationState::new();
    let credential = reference(request(LABEL, SCOPE, &revocations));
    let mut callback_ran = false;

    vault
        .with_scoped_credential(&credential, 110, &revocations, |secret| {
            assert_eq!(secret, SECRET);
            callback_ran = true;
            Ok(())
        })
        .expect("scoped resolution");

    assert!(callback_ran);
    assert_eq!(credential.label(), LABEL);
    assert_eq!(credential.scope(), SCOPE);
    assert_eq!(credential.capability_id().as_str(), "a".repeat(64));
    let debug = format!("{credential:?}");
    tracing::info!(reference = ?credential, "scoped credential reference");
    assert_eq!(debug, "VaultCredentialRef(<redacted>)");
    assert!(!debug.contains(SECRET));
    assert!(!logs_contain(SECRET));
}

#[test]
fn callback_error_cancellation_and_panic_are_contained_and_secret_free() {
    let (_directory, vault) = test_vault(CredentialType::ApiKey);
    let revocations = RevocationState::new();
    let credential = reference(request(LABEL, SCOPE, &revocations));

    for (callback_result, expected) in [
        (
            ScopedCredentialCallbackError::Failed,
            ScopedCredentialError::CallbackFailed,
        ),
        (
            ScopedCredentialCallbackError::Cancelled,
            ScopedCredentialError::CallbackCancelled,
        ),
    ] {
        let error = vault
            .with_scoped_credential(&credential, 110, &revocations, |secret| {
                assert_eq!(secret, SECRET);
                Err(callback_result)
            })
            .expect_err("callback must fail");
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains(SECRET));
    }

    let panic_error = vault
        .with_scoped_credential(&credential, 110, &revocations, |secret| {
            assert_eq!(secret, SECRET);
            panic!("callback panic containing {secret}")
        })
        .expect_err("panic must be contained");
    assert_eq!(panic_error, ScopedCredentialError::CallbackPanicked);
    assert!(!format!("{panic_error:?} {panic_error}").contains(SECRET));

    vault
        .update(
            LABEL,
            Some("replacement-secret".to_string()),
            /*provider*/ None,
            /*notes*/ None,
            /*revocation_notes*/ None,
        )
        .expect("vault lock released after every callback outcome");
}

#[test]
fn missing_deleted_and_ineligible_credentials_fail_closed() {
    let revocations = RevocationState::new();
    let credential = reference(request(LABEL, SCOPE, &revocations));

    let directory = tempfile::tempdir().expect("tempdir");
    let vault = Vault::new_with_keyring_store(
        directory.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    assert_eq!(
        vault
            .with_scoped_credential(&credential, 110, &revocations, |_| Ok(()),)
            .expect_err("missing credential"),
        ScopedCredentialError::NotFound
    );

    vault
        .add(AddCredential {
            label: LABEL.to_string(),
            credential_type: CredentialType::SeedPhrase,
            provider: None,
            notes: None,
            revocation_notes: None,
            secret: SECRET.to_string(),
        })
        .expect("add ineligible credential");
    assert_eq!(
        vault
            .with_scoped_credential(&credential, 110, &revocations, |_| Ok(()),)
            .expect_err("ineligible type"),
        ScopedCredentialError::CredentialTypeDenied
    );

    vault.delete(LABEL).expect("delete credential");
    assert_eq!(
        vault
            .with_scoped_credential(&credential, 110, &revocations, |_| Ok(()),)
            .expect_err("deleted credential"),
        ScopedCredentialError::NotFound
    );
}

#[test]
fn expired_and_revoked_authority_is_revalidated_before_decryption() {
    let (_directory, vault) = test_vault(CredentialType::BearerToken);
    let initial_revocations = RevocationState::new();
    let request = request(LABEL, SCOPE, &initial_revocations);
    let grant_id = request.grant.grant_id.clone();
    let credential = reference(request);

    assert_eq!(
        vault
            .with_scoped_credential(&credential, 180, &initial_revocations, |_| Ok(()),)
            .expect_err("expired capability"),
        ScopedCredentialError::Expired
    );

    let mut revoked = initial_revocations;
    revoked
        .apply(
            &RevocationEvent::new(
                human(),
                RevocationTarget::Grant { grant_id },
                RevocationReason::HumanRequest,
                110,
            )
            .expect("revocation event"),
        )
        .expect("apply revocation");
    assert_eq!(
        vault
            .with_scoped_credential(&credential, 111, &revoked, |_| Ok(()),)
            .expect_err("revoked capability"),
        ScopedCredentialError::Revoked
    );
}

#[test]
fn mismatched_label_and_scope_are_rejected_as_stable_errors() {
    let revocations = RevocationState::new();

    let mut wrong_label = request(LABEL, SCOPE, &revocations);
    wrong_label.authorization.resource.id = text("provider.adjacent");
    assert_eq!(
        VaultCredentialRef::from_authorized(
            CapabilityId::from_sha256_hex("b".repeat(64)).expect("id"),
            wrong_label,
        )
        .expect_err("wrong label"),
        ScopedCredentialError::LabelMismatch
    );

    let mut wrong_scope = request(LABEL, SCOPE, &revocations);
    wrong_scope.authorization.context.operation = text("responses.read");
    assert_eq!(
        VaultCredentialRef::from_authorized(
            CapabilityId::from_sha256_hex("c".repeat(64)).expect("id"),
            wrong_scope,
        )
        .expect_err("wrong scope"),
        ScopedCredentialError::ScopeMismatch
    );
}

const _: fn() = || {
    trait AmbiguousIfSerialize<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfSerialize<()> for T {}
    struct ImplementsSerialize;
    impl<T: ?Sized + serde::Serialize> AmbiguousIfSerialize<ImplementsSerialize> for T {}
    let _ = <VaultCredentialRef as AmbiguousIfSerialize<_>>::marker;
};

struct FixedBrokerClock;

impl VaultBrokerClock for FixedBrokerClock {
    fn now_unix_seconds(&self) -> Result<i64, BackendDispatchError> {
        Ok(110)
    }
}

#[derive(Default)]
struct TestBrokerTransport {
    calls: AtomicUsize,
}

impl VaultBrokerTransport for Arc<TestBrokerTransport> {
    fn execute_openai_responses(
        &self,
        raw_credential: &str,
        operation: &OpenAiResponsesOperation,
        cancellation: &CancellationFence,
    ) -> Result<TypedOperationReceipt, BackendDispatchError> {
        assert_eq!(raw_credential, SECRET);
        assert_eq!(operation.path(), "/v1/responses");
        cancellation.ensure_active()?;
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(TypedOperationReceipt {
            response_status: 200,
            uploaded_bytes: 12,
            downloaded_bytes: 24,
        })
    }
}

struct TestAudit;

impl DurableBrokerAudit for TestAudit {
    type Permit = ();

    fn reserve(&self, _intent: &BrokerAuditIntent) -> Result<Self::Permit, BrokerAuditError> {
        Ok(())
    }

    fn resolve(
        &self,
        _permit: Self::Permit,
        _resolution: BrokerAuditResolution,
    ) -> Result<(), BrokerAuditError> {
        Ok(())
    }
}

fn broker_authorization() -> ProtectedModeAuthorization {
    let capabilities = REQUIRED_CAPABILITIES
        .iter()
        .copied()
        .map(|capability| CapabilityResult {
            capability,
            status: CapabilityStatus::Supported,
            observation: Observation::Denied,
            mechanism: "synthetic-vault-test",
            detail_code: "denied",
        })
        .collect::<Vec<_>>();
    validate_protected_mode_report(
        &PlatformReport {
            contract_version: CONTRACT_VERSION,
            fixture_protocol: FIXTURE_PROTOCOL_VERSION,
            probe_sha256: "b".repeat(64).as_str(),
            target_id: "c".repeat(64).as_str(),
            measured_at_unix_seconds: 100,
            expires_at_unix_seconds: 200,
            capabilities: &capabilities,
            protected_mode_eligible: true,
        },
        "c".repeat(64).as_str(),
        "b".repeat(64).as_str(),
        110,
    )
    .expect("authorization")
}

#[test]
fn pf_27_s04_vault_backend_resolves_only_inside_typed_broker_dispatch() {
    let (_directory, vault) = test_vault(CredentialType::BearerToken);
    let revocations = Arc::new(RwLock::new(RevocationState::new()));
    let request = request(LABEL, SCOPE, &revocations.read().expect("revocations"));
    let credential = reference(request);
    let broker_reference =
        BrokerCredentialReference::from_sha256_hex("a".repeat(64)).expect("reference");
    let transport = Arc::new(TestBrokerTransport::default());
    let backend = VaultBrokerBackend::with_clock(
        Arc::new(vault),
        vec![(broker_reference.clone(), credential)],
        revocations,
        transport.clone(),
        FixedBrokerClock,
    )
    .expect("backend");
    let runtime = BrokerRuntime::new(
        "d".repeat(64),
        BrokerRuntimeConfig::bounded(1, 1).expect("config"),
        broker_authorization(),
        backend,
        TestAudit,
    )
    .expect("runtime");
    let binding = BrokerBinding {
        controller_instance: "controller-1".to_string(),
        worker_instance: "worker-1".to_string(),
        session_id: "session-1".to_string(),
        task_id: "task-1".to_string(),
        run_id: "run-1".to_string(),
        run_generation: 1,
    };
    let peer = ObservedPeer::from_os("worker-uid-501", 42).expect("peer");
    let handle = runtime
        .register_session(
            binding.clone(),
            peer.clone(),
            BrokerChannelMac::from_secret([9; 32]),
            vec![
                BrokerCredentialGrant::expiring(broker_reference.clone(), i64::MAX).expect("grant"),
            ],
        )
        .expect("session");
    let frame = BrokerChannelMac::from_secret([9; 32])
        .sign(
            binding,
            1,
            BrokerOperation::OpenAiResponses {
                credential: broker_reference,
                request: OpenAiResponsesOperation::new("/v1/responses").expect("operation"),
            },
        )
        .expect("frame");
    assert_eq!(
        runtime
            .dispatch(&handle, &peer, &frame)
            .expect("receipt")
            .response_status,
        200
    );
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
}

const _: fn() = || {
    trait AmbiguousIfDisplay<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfDisplay<()> for T {}
    struct ImplementsDisplay;
    impl<T: ?Sized + std::fmt::Display> AmbiguousIfDisplay<ImplementsDisplay> for T {}
    let _ = <VaultCredentialRef as AmbiguousIfDisplay<_>>::marker;
};
