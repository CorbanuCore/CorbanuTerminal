//! #2092 (#2562, main ledger row 21): direct review prepares a recorded call on a fresh
//! speculative turn. Grants another turn received never reach it; session grants arrive only
//! through the session's stripped reader.
use super::*;
use crate::session::tests::make_session_and_context;
use crate::tools::handlers::read_granted_permissions;
use codex_exec_server::LOCAL_ENVIRONMENT_ID;
use codex_protocol::models::NetworkPermissions;
use codex_protocol::request_permissions::PermissionGrantScope;
use codex_protocol::request_permissions::RequestPermissionProfile;
use codex_protocol::request_permissions::RequestPermissionsResponse;
use pretty_assertions::assert_eq;

fn network_grant(scope: PermissionGrantScope) -> RequestPermissionsResponse {
    RequestPermissionsResponse {
        permissions: RequestPermissionProfile {
            network: Some(NetworkPermissions {
                enabled: Some(true),
            }),
            ..RequestPermissionProfile::default()
        },
        scope,
        strict_auto_review: false,
    }
}

#[tokio::test]
async fn review_step_gets_no_grant_from_another_turn() {
    let (session, earlier_turn) = make_session_and_context().await;
    let session = Arc::new(session);
    // The turn that answered `request_permissions` holds its turn-scoped grant.
    assert!(session.record_granted_request_permissions_for_turn(
        &network_grant(PermissionGrantScope::Turn),
        LOCAL_ENVIRONMENT_ID,
        &earlier_turn,
        session.services.authority_marker(),
    ));
    assert!(
        read_granted_permissions(&session, &earlier_turn, LOCAL_ENVIRONMENT_ID)
            .await
            .is_some(),
        "control: the granting turn itself sees its grant"
    );

    let step = speculative_review_step(&session, &CancellationToken::new())
        .await
        .expect("speculative review step");
    assert_eq!(
        read_granted_permissions(&session, &step.turn, LOCAL_ENVIRONMENT_ID).await,
        None
    );
    assert_eq!(step.turn.granted_permissions(LOCAL_ENVIRONMENT_ID), None);
}

#[tokio::test]
async fn review_step_reads_session_grants_through_the_session() {
    let (session, earlier_turn) = make_session_and_context().await;
    let session = Arc::new(session);
    let grant = network_grant(PermissionGrantScope::Session);
    assert!(session.record_granted_request_permissions_for_turn(
        &grant,
        LOCAL_ENVIRONMENT_ID,
        &earlier_turn,
        session.services.authority_marker(),
    ));
    let step = speculative_review_step(&session, &CancellationToken::new())
        .await
        .expect("speculative review step");
    assert_eq!(
        read_granted_permissions(&session, &step.turn, LOCAL_ENVIRONMENT_ID).await,
        session
            .usable_grant(Some(grant.permissions.clone().into()))
            .await
    );
    assert_eq!(
        read_granted_permissions(&session, &step.turn, LOCAL_ENVIRONMENT_ID).await,
        Some(grant.permissions.into())
    );
}

/// #245 on the review step: while the protected-path rules apply, a session grant that covers a
/// protected file (the Corbanu home's `auth.json`) reaches the speculative step without it.
#[tokio::test]
async fn review_step_gets_session_grants_with_protected_paths_stripped() {
    use codex_protocol::models::AdditionalPermissionProfile;
    use codex_protocol::models::FileSystemPermissions;
    use codex_protocol::permissions::FileSystemAccessMode;
    use codex_protocol::permissions::FileSystemPath;
    use codex_protocol::permissions::FileSystemSandboxEntry;
    use codex_security_policy::SecurityLevel;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use codex_utils_path_uri::PathUri;

    let (session, turn) = make_session_and_context().await;
    let session = Arc::new(session);
    let entry = |path: &std::path::Path| {
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: PathUri::from_abs_path(
                    &AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
                ),
            },
            FileSystemAccessMode::Read,
        )
    };
    let additional = |entries| AdditionalPermissionProfile {
        network: None,
        file_system: Some(FileSystemPermissions {
            entries,
            glob_scan_max_depth: None,
        }),
    };
    #[allow(deprecated)]
    let notes = turn.cwd.join("notes.txt");
    let auth = turn.config.codex_home.join("auth.json");
    let notes_only = additional(vec![entry(notes.as_path())]);
    let grant = additional(vec![entry(notes.as_path()), entry(auth.as_path())]);
    let control = session
        .services
        .agent_control
        .local()
        .expect("local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Moderate,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("policy");
    assert!(session.services.record_granted_permissions(
        LOCAL_ENVIRONMENT_ID,
        grant,
        session.services.authority_marker(),
    ));
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_policy(SecurityLevel::Moderate, control.effective_security_policy())
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);

    let step = speculative_review_step(&session, &CancellationToken::new())
        .await
        .expect("speculative review step");
    assert_eq!(
        read_granted_permissions(&session, &step.turn, LOCAL_ENVIRONMENT_ID).await,
        Some(notes_only)
    );
}
