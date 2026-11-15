use std::sync::Arc;

use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::models::FileSystemPermissions;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_security_policy::SecurityLevel;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_path_uri::PathUri;
use pretty_assertions::assert_eq;

use super::StepContext;
use crate::session::tests::make_session_and_context;
use crate::session::tests::test_environment_id;

/// #245: a step's file access reads the originating turn's grants through the
/// session, so a protected path a grant names is stripped while the
/// protected-path rules apply, even when the grant was recorded before they did.
#[tokio::test]
async fn issue_245_step_file_access_strips_protected_paths_from_turn_grants() {
    let (session, turn) = make_session_and_context().await;
    let entry = |path: &std::path::Path| {
        FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: PathUri::from_abs_path(
                    &AbsolutePathBuf::from_absolute_path(path).expect("absolute path"),
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
    // The session's policy tree is in place before the grant (D51, #1001: grants end at a
    // security transition); the protected-path rules apply later.
    let control = session
        .services
        .agent_control
        .local()
        .expect("the local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Moderate,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("policy");
    let environment_id = test_environment_id(&turn).to_string();
    assert!(turn.record_granted_permissions(
        &environment_id,
        grant.clone(),
        /*strict_auto_review*/ false,
        session.services.authority_marker(),
    ));
    let step = StepContext::for_test(Arc::new(turn));
    // No rules apply yet: as recorded.
    assert_eq!(
        step.usable_grants(&session, &environment_id).await,
        Some(grant)
    );

    let client = (*session.services.model_client())
        .clone()
        .with_ingress_policy(SecurityLevel::Moderate, control.effective_security_policy())
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);

    assert_eq!(
        step.usable_grants(&session, &environment_id).await,
        Some(notes_only)
    );
}

/// PF-23-S01: a step's file access (extension tools' fs) gets the protected-path read denials
/// once they apply, as in-process file tools do (here Moderate, which applies them before
/// taint too, #239).
#[tokio::test]
async fn pf_23_s01_step_file_access_gets_the_read_denials() {
    let (session, turn) = make_session_and_context().await;
    let auth = turn.config.codex_home.join("auth.json");
    #[allow(deprecated)]
    let cwd = turn.cwd.clone();
    let step = StepContext::for_test(Arc::new(turn));
    let environment = step
        .environments
        .turn_environments()
        .next()
        .expect("an environment");
    let readable = |context: codex_file_system::FileSystemSandboxContext| {
        codex_protocol::models::PermissionProfile::try_from(context.permissions)
            .expect("a permission profile")
            .file_system_sandbox_policy()
            .can_read_path_with_cwd(auth.as_path(), cwd.as_path())
    };
    assert!(readable(
        step.file_access_context(&session, environment).await
    ));

    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(SecurityLevel::Moderate)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    assert!(!readable(
        step.file_access_context(&session, environment).await
    ));
}

async fn metadata(
    access: &dyn codex_file_system::EnvironmentAccess,
    path: &PathUri,
) -> Result<(), (std::io::ErrorKind, String)> {
    access
        .get_metadata(path, codex_file_system::GetMetadataOptions::default())
        .await
        .map(drop)
        .map_err(|error| (error.kind(), error.to_string()))
}

/// #2183 review: a step's file access is built once per step, so a security
/// transition committed during the step (here a revocation, which moves the
/// policy marker and keeps the level) refuses every later operation through
/// it; the next step's access is decided under the new policy and works.
#[tokio::test]
async fn file_access_built_before_a_transition_is_refused_after_it() {
    let (session, _events) = crate::session::tests::make_session_with_config_and_rx(|_| {})
        .await
        .expect("session");
    let control = session
        .services
        .agent_control
        .local()
        .expect("the local agent backend")
        .clone()
        .with_effective_security_policy(
            SecurityLevel::Permissive,
            session.thread_id,
            /*inherits_from_spawn_parent*/ false,
        )
        .expect("policy");
    let controller = control.trusted_security_controller().expect("controller");
    let turn = session.new_default_turn().await;
    let step = StepContext::for_test(Arc::clone(&turn));
    let before = step.environments(&session).await;
    let (environment, access) = before.first().expect("an environment");
    let cwd = environment.cwd().clone();
    assert_eq!(metadata(access, &cwd).await, Ok(()));

    let mut revocations = codex_security_policy::RevocationState::new();
    revocations
        .apply(
            &codex_security_policy::RevocationEvent::new(
                codex_security_policy::PolicyPrincipal::new(
                    codex_security_policy::PrincipalKind::Human,
                    "human:step-file-access",
                )
                .expect("principal"),
                codex_security_policy::RevocationTarget::Grant {
                    grant_id: codex_security_policy::BoundedText::new("step-file-access-grant")
                        .expect("grant id"),
                },
                codex_security_policy::RevocationReason::HumanRequest,
                /*created_at_unix_seconds*/ 1,
            )
            .expect("event"),
        )
        .expect("apply");
    controller
        .apply_confirmed_change(
            controller
                .confirm_level_change(SecurityLevel::Permissive, revocations)
                .expect("confirm"),
        )
        .expect("transition");

    assert_eq!(
        metadata(access, &cwd).await,
        Err((
            std::io::ErrorKind::PermissionDenied,
            super::STEP_FILE_ACCESS_STALE.to_string()
        ))
    );
    let after = step.environments(&session).await;
    assert_eq!(metadata(&after[0].1, &cwd).await, Ok(()));
}
