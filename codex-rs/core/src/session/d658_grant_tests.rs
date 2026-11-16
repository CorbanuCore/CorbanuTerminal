use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::models::FileSystemPermissions;
use codex_protocol::models::NetworkPermissions;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::RevocationState;
use codex_security_policy::SecurityLevel;
use codex_security_policy::SecuritySettings;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;

use crate::session::tests::make_session_and_context;
use crate::session::tests::test_environment_id;
use crate::tools::handlers::read_granted_permissions;

#[tokio::test]
async fn captured_turn_grants_survive_a_new_turn_but_not_security_revocation() {
    let (session, old_turn) = make_session_and_context().await;
    let controller = crate::security::TrustedSecurityController::initialize(
        &session
            .services
            .agent_control
            .local()
            .unwrap()
            .effective_security_policy(),
        crate::security::PersistedHumanSecurityState::new(
            SecuritySettings::new(SecurityLevel::Permissive),
            PolicyPrincipal::new(PrincipalKind::Human, "human:d658").unwrap(),
            RevocationState::new(),
        )
        .unwrap(),
        session.thread_id,
        codex_protocol::SessionId::from(session.thread_id),
        crate::security::EffectivePolicyInitialization::Root,
    )
    .unwrap();
    let environment_id = test_environment_id(&old_turn).to_string();
    let grant = AdditionalPermissionProfile {
        network: Some(NetworkPermissions {
            enabled: Some(true),
        }),
        ..Default::default()
    };
    assert!(old_turn.record_granted_permissions(
        &environment_id,
        grant.clone(),
        /*strict_auto_review*/ false,
        session.services.authority_marker(),
    ));
    let new_turn = session
        .new_turn_with_default_settings("d658-next-turn".to_string(), Default::default())
        .await;
    assert_eq!(
        read_granted_permissions(&session, &old_turn, &environment_id).await,
        Some(grant.clone()),
    );
    assert_eq!(
        read_granted_permissions(&session, &new_turn, &environment_id).await,
        None
    );
    let new_grant = AdditionalPermissionProfile {
        network: None,
        file_system: Some(FileSystemPermissions {
            entries: vec![FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: PathUri::from_abs_path(&old_turn.config.cwd)
                        .join("d658-new-only.txt")
                        .unwrap(),
                },
                FileSystemAccessMode::Write,
            )],
            glob_scan_max_depth: None,
        }),
    };
    assert!(new_turn.record_granted_permissions(
        &environment_id,
        new_grant.clone(),
        /*strict_auto_review*/ false,
        session.services.authority_marker(),
    ));
    assert_eq!(
        read_granted_permissions(&session, &new_turn, &environment_id).await,
        Some(new_grant)
    );
    assert_eq!(
        read_granted_permissions(&session, &old_turn, &environment_id).await,
        Some(grant),
        "the old cell must not acquire the new turn's different grant",
    );

    let change = controller
        .confirm_level_change(SecurityLevel::Moderate, RevocationState::new())
        .unwrap();
    controller.apply_confirmed_change(change).unwrap();
    for turn in [&old_turn, new_turn.as_ref()] {
        assert_eq!(
            read_granted_permissions(&session, turn, &environment_id).await,
            None
        );
    }
}
