use super::*;
use pretty_assertions::assert_eq;

async fn owner(level: SecurityLevel) -> Arc<Session> {
    let (mut session, _) = crate::session::tests::make_session_and_context().await;
    session.services.agent_control = session.services.agent_control.clone()
        .with_effective_security_policy(level, session.thread_id, false).unwrap();
    Arc::new(session)
}

async fn client(owner: &Arc<Session>) -> Result<StageOneMemoryClient, StageOneMemoryError> {
    StageOneMemoryClient::new(Arc::downgrade(owner), futures::future::pending().boxed().shared(),
        owner.thread_id, &owner.provider().await).await
}

#[tokio::test]
async fn pf_30_s04_live_protected_floor_denies_without_a_request() {
    for level in [SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let owner = owner(level).await;
        assert!(matches!(client(&owner).await,
            Err(StageOneMemoryError::Denied(StageOneMemoryDenial::ProtectedInputUnavailable))));
    }
}

#[tokio::test]
async fn pf_30_s04_binding_is_owner_specific_and_missing_policy_denies() {
    let owner = owner(SecurityLevel::Permissive).await;
    let wrong = StageOneMemoryClient::new(Arc::downgrade(&owner), futures::future::pending().boxed().shared(),
        ThreadId::new(), &owner.provider().await).await;
    assert!(matches!(wrong, Err(StageOneMemoryError::Denied(StageOneMemoryDenial::OwnerMismatch))));
    let (session, _) = crate::session::tests::make_session_and_context().await;
    let session = Arc::new(session);
    assert!(matches!(client(&session).await,
        Err(StageOneMemoryError::Denied(StageOneMemoryDenial::PolicyUnavailable))));
}

#[tokio::test]
async fn pf_30_s04_live_strengthening_is_sticky_through_later_downgrade() {
    let owner = owner(SecurityLevel::Permissive).await;
    let client = client(&owner).await.unwrap();
    assert!(client.check_completion().await.is_ok());
    let controller = owner.services.agent_control.trusted_security_controller().unwrap();
    for level in [SecurityLevel::Moderate, SecurityLevel::Permissive] {
        let change = controller.confirm_level_change(level, codex_security_policy::RevocationState::new()).unwrap();
        controller.apply_confirmed_change(change).unwrap();
        assert!(matches!(client.check_completion().await,
            Err(StageOneMemoryError::Denied(StageOneMemoryDenial::ProtectedInputUnavailable))));
    }
}

#[tokio::test]
async fn pf_30_s04_termination_and_dropped_owner_fail_closed() {
    let owner = owner(SecurityLevel::Permissive).await;
    let completed = StageOneMemoryClient::new(Arc::downgrade(&owner), futures::future::ready(()).boxed().shared(),
        owner.thread_id, &owner.provider().await).await;
    assert!(matches!(completed, Err(StageOneMemoryError::Denied(StageOneMemoryDenial::OwnerTerminated))));
    let client = client(&owner).await.unwrap();
    drop(owner);
    assert!(matches!(client.check_completion().await,
        Err(StageOneMemoryError::Denied(StageOneMemoryDenial::OwnerTerminated))));
}

#[tokio::test]
async fn pf_30_s04_provider_change_invalidates_bound_client() {
    let owner = owner(SecurityLevel::Permissive).await;
    let client = client(&owner).await.unwrap();
    owner.update_settings(crate::session::SessionSettingsUpdate {
        model_provider: Some(codex_model_provider_info::ZAI_PROVIDER_ID.into()),
        ..Default::default()
    }).await.unwrap();
    assert!(matches!(client.check_completion().await,
        Err(StageOneMemoryError::Denied(StageOneMemoryDenial::ProviderChanged))));
}

#[test]
fn pf_30_s04_denial_messages_are_bounded_and_input_independent() {
    assert_eq!(StageOneMemoryDenial::ProtectedInputUnavailable.to_string(),
        "protected stage-one memory input is unavailable");
    assert_eq!(StageOneMemoryDenial::OwnerMismatch.to_string(), "stage-one memory owner does not match");
}
