use std::collections::BTreeMap;
use std::sync::Arc;

use codex_keyring_store::tests::MockKeyringStore;
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
use tracing_test::traced_test;

use super::*;
use crate::AddCredential;
use crate::CredentialType;
use crate::MANAGED_CLAUDE_TOKEN_LABEL;

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
fn managed_claude_token_is_denied_before_the_scoped_callback_runs() {
    let directory = tempfile::tempdir().expect("tempdir");
    let vault = Vault::new_with_keyring_store(
        directory.path().to_path_buf(),
        Arc::new(MockKeyringStore::default()),
    );
    vault
        .store_managed_claude_subscription_token("managed-token-canary".to_string())
        .expect("store managed token");
    let revocations = RevocationState::new();
    let credential = reference(request(MANAGED_CLAUDE_TOKEN_LABEL, SCOPE, &revocations));
    let mut callback_ran = false;

    let error = vault
        .with_scoped_credential(&credential, 110, &revocations, |_| {
            callback_ran = true;
            Ok(())
        })
        .expect_err("managed token must be denied to generic capabilities");

    assert_eq!(error, ScopedCredentialError::CredentialTypeDenied);
    assert!(!callback_ran);
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

const _: fn() = || {
    trait AmbiguousIfDisplay<A> {
        fn marker() {}
    }
    impl<T: ?Sized> AmbiguousIfDisplay<()> for T {}
    struct ImplementsDisplay;
    impl<T: ?Sized + std::fmt::Display> AmbiguousIfDisplay<ImplementsDisplay> for T {}
    let _ = <VaultCredentialRef as AmbiguousIfDisplay<_>>::marker;
};
