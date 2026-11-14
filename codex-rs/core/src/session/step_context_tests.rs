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
