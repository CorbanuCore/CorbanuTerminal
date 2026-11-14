use super::*;
use codex_apply_patch::MaybeApplyPatchVerified;
use codex_exec_server::LOCAL_FS;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::FileSystemSandboxPolicyContext;
use codex_protocol::protocol::FileChange;
use codex_utils_absolute_path::AbsolutePathBuf;
use core_test_support::PathBufExt;
use core_test_support::PathExt;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

fn local_context(cwd: &PathUri) -> FileSystemSandboxPolicyContext<'_> {
    FileSystemSandboxPolicyContext {
        cwd,
        workspace_roots: std::slice::from_ref(cwd),
        user_home_dir: None,
        temporary_directories: None,
    }
}

use crate::session::step_context::StepContext;
use crate::session::tests::make_session_and_context;
use crate::tools::context::ToolInvocation;
use crate::tools::hook_names::HookToolName;
use crate::tools::registry::PostToolUsePayload;
use crate::tools::registry::PreToolUsePayload;
use crate::turn_diff_tracker::TurnDiffTracker;

fn sample_patch() -> &'static str {
    r#"*** Begin Patch
*** Add File: hello.txt
+hello
*** End Patch"#
}

async fn invocation_for_payload(payload: ToolPayload) -> ToolInvocation {
    let (session, turn) = make_session_and_context().await;
    let turn = Arc::new(turn);
    ToolInvocation {
        session: session.into(),
        step_context: StepContext::for_test(Arc::clone(&turn)),
        turn,
        cancellation_token: tokio_util::sync::CancellationToken::new(),
        tracker: Arc::new(Mutex::new(TurnDiffTracker::new())),
        call_id: "call-apply-patch".to_string(),
        tool_name: codex_tools::ToolName::plain("apply_patch"),
        source: crate::tools::context::ToolCallSource::Direct,
        payload,
        native_call: None,
        environments: Default::default(),
    }
}

async fn intercept_for_test(
    command: &[String],
    session: Arc<Session>,
    turn: Arc<TurnContext>,
    turn_environment: TurnEnvironment,
) -> Result<Option<FunctionToolOutput>, FunctionCallError> {
    let cwd = turn_environment.cwd().clone();
    let fs = turn_environment.environment.get_filesystem();
    intercept_apply_patch(
        command,
        &cwd,
        fs.as_ref(),
        turn_environment,
        session,
        StepContext::for_test(turn),
        tokio_util::sync::CancellationToken::new(),
        /*tracker*/ None,
        "shell-apply-patch-test",
        "exec_command",
        InterceptedPatchSource::StrictShell,
    )
    .await
}

fn response_error(result: Result<Option<FunctionToolOutput>, FunctionCallError>) -> String {
    match result {
        Err(FunctionCallError::RespondToModel(message)) => message,
        Err(_) => panic!("expected RespondToModel error"),
        Ok(_) => panic!("expected tool error"),
    }
}

#[tokio::test]
async fn pre_tool_use_payload_uses_freeform_patch_input() {
    let patch = sample_patch();
    let payload = ToolPayload::Custom {
        input: patch.to_string(),
    };
    let invocation = invocation_for_payload(payload).await;
    let handler = ApplyPatchHandler::default();

    assert_eq!(
        handler.pre_tool_use_payload(&invocation),
        Some(PreToolUsePayload {
            tool_name: HookToolName::apply_patch(),
            tool_input: json!({ "command": patch }),
        })
    );
}

#[tokio::test]
async fn pre_tool_use_payload_accepts_chat_wrapped_patch_input() {
    let patch = sample_patch();
    let payload = ToolPayload::Function {
        arguments: json!({ "input": patch }).to_string(),
    };
    let invocation = invocation_for_payload(payload).await;
    let handler = ApplyPatchHandler::default();

    assert_eq!(
        handler.pre_tool_use_payload(&invocation),
        Some(PreToolUsePayload {
            tool_name: HookToolName::apply_patch(),
            tool_input: json!({ "command": patch }),
        })
    );
}

#[tokio::test]
async fn post_tool_use_payload_uses_patch_input_and_tool_output() {
    let patch = sample_patch();
    let payload = ToolPayload::Custom {
        input: patch.to_string(),
    };
    let invocation = invocation_for_payload(payload).await;
    let output = ApplyPatchToolOutput::from_text("Success. Updated files.".to_string());
    let handler = ApplyPatchHandler::default();

    assert_eq!(
        handler.post_tool_use_payload(&invocation, &output),
        Some(PostToolUsePayload {
            tool_cwd: None,
            tool_name: HookToolName::apply_patch(),
            tool_use_id: "call-apply-patch".to_string(),
            tool_input: json!({ "command": patch }),
            tool_response: json!("Success. Updated files."),
        })
    );
}

#[test]
fn diff_consumer_streams_apply_patch_changes() {
    let mut consumer = ApplyPatchArgumentDiffConsumer::default();
    assert!(
        consumer
            .push_delta("call-1".to_string(), "*** Begin Patch\n")
            .is_none()
    );

    let event = consumer
        .push_delta("call-1".to_string(), "*** Add File: hello.txt\n+hello")
        .expect("progress event");
    assert_eq!(
        (event.call_id, event.changes),
        (
            "call-1".to_string(),
            HashMap::from([(
                PathBuf::from("hello.txt"),
                FileChange::Add {
                    content: String::new(),
                },
            )]),
        )
    );

    assert!(
        consumer
            .push_delta("call-1".to_string(), "\n+world")
            .is_none()
    );
    assert!(
        consumer
            .push_delta("call-1".to_string(), "\n*** End Patch")
            .is_none()
    );

    let event = consumer
        .finish_update_on_complete()
        .expect("finish parser")
        .expect("progress event");
    assert_eq!(
        (event.call_id, event.changes),
        (
            "call-1".to_string(),
            HashMap::from([(
                PathBuf::from("hello.txt"),
                FileChange::Add {
                    content: "hello\nworld\n".to_string(),
                },
            )]),
        )
    );
}

#[test]
fn diff_consumer_streams_apply_patch_changes_with_environment_header() {
    let mut consumer = ApplyPatchArgumentDiffConsumer::default();
    assert!(
        consumer
            .push_delta(
                "call-1".to_string(),
                "*** Begin Patch\n*** Environment ID: remote\n",
            )
            .is_none()
    );

    let event = consumer
        .push_delta("call-1".to_string(), "*** Add File: hello.txt\n+hello")
        .expect("progress event");
    assert_eq!(
        event.changes,
        HashMap::from([(
            PathBuf::from("hello.txt"),
            FileChange::Add {
                content: String::new(),
            },
        )])
    );
}

#[test]
fn diff_consumer_sends_next_update_after_buffer_interval() {
    let mut consumer = ApplyPatchArgumentDiffConsumer::default();
    consumer.push_delta("call-1".to_string(), "*** Begin Patch\n");
    let first = consumer
        .push_delta("call-1".to_string(), "*** Add File: hello.txt\n+hello")
        .expect("first progress event");
    assert_eq!(
        first.changes,
        HashMap::from([(
            PathBuf::from("hello.txt"),
            FileChange::Add {
                content: String::new(),
            },
        )])
    );

    consumer.last_sent_at =
        Some(std::time::Instant::now() - APPLY_PATCH_ARGUMENT_DIFF_BUFFER_INTERVAL);
    let second = consumer
        .push_delta("call-1".to_string(), "\n+world")
        .expect("second progress event");
    assert_eq!(
        second.changes,
        HashMap::from([(
            PathBuf::from("hello.txt"),
            FileChange::Add {
                content: "hello\n".to_string(),
            },
        )])
    );
}

#[test]
fn reconcile_environment_id_requires_selection_when_enabled() {
    assert_eq!(
        require_environment_id(Some("remote"), /*allow_environment_id*/ false),
        Err(FunctionCallError::RespondToModel(
            "apply_patch environment selection is unavailable for this turn".to_string(),
        ))
    );
    assert_eq!(
        require_environment_id(
            /*parsed_environment_id*/ None, /*allow_environment_id*/ true
        ),
        Ok(None)
    );
}

#[tokio::test]
async fn approval_keys_include_move_destination() {
    let tmp = TempDir::new().expect("tmp");
    let cwd_path = tmp.path();
    let cwd = cwd_path.abs();
    std::fs::create_dir_all(cwd_path.join("old")).expect("create old dir");
    std::fs::create_dir_all(cwd_path.join("renamed/dir")).expect("create dest dir");
    std::fs::write(cwd_path.join("old/name.txt"), "old content\n").expect("write old file");
    let patch = r#"*** Begin Patch
*** Update File: old/name.txt
*** Move to: renamed/dir/name.txt
@@
-old content
+new content
*** End Patch"#;
    let argv = vec!["apply_patch".to_string(), patch.to_string()];
    // TODO(anp): Keep apply_patch handler test cwd values as PathUri.
    let cwd = PathUri::from_abs_path(&cwd);
    let action = match codex_apply_patch::maybe_parse_apply_patch_verified(
        &argv,
        &cwd,
        LOCAL_FS.as_ref(),
        /*sandbox*/ None,
    )
    .await
    {
        MaybeApplyPatchVerified::Body(action) => action,
        other => panic!("expected patch body, got: {other:?}"),
    };

    let keys = file_paths_for_action(&action);
    assert_eq!(keys.len(), 2);
}

/// #713: a patch whose paths aren't native to this host skips path matching,
/// but a restricted turn must still ask for it instead of auto-approving it.
#[tokio::test]
async fn foreign_patch_paths_are_not_auto_approved_under_a_restricted_turn() {
    use crate::session::tests::make_session_and_context_with_auth_and_config_and_rx;
    use crate::tools::sandboxing::ExecApprovalRequirement;
    use codex_protocol::models::PermissionProfile;
    use codex_protocol::protocol::AskForApproval;

    let (session, turn, _events) = make_session_and_context_with_auth_and_config_and_rx(
        codex_login::CodexAuth::from_api_key("Test API Key"),
        Vec::new(),
        |config| {
            config.permissions.approval_policy =
                crate::config::Constrained::allow_any(AskForApproval::OnRequest);
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write())
                .expect("test setup should allow permission profile");
        },
    )
    .await;
    let foreign = if cfg!(windows) {
        "file:///tmp/remote/new.txt"
    } else {
        "file:///C:/remote/new.txt"
    };
    let path = PathUri::parse(foreign).expect("foreign path URI");
    assert!(path.to_abs_path().is_err(), "{foreign} must not be native");
    let action = ApplyPatchAction::new_add_for_test(&path, "content".to_string());
    let turn_environment = turn
        .initial_environments
        .primary()
        .cloned()
        .expect("test turn should have a primary environment");
    let step_context = StepContext::for_test(Arc::clone(&turn));

    let requirement =
        match prepare_verified_patch(action, turn_environment, &session, &step_context).await {
            Ok((request, _handler_basis)) => request.exec_approval_requirement,
            // Refusing the foreign patch outright (the model is told why) is
            // also not auto-approval.
            Err(FunctionCallError::RespondToModel(_)) => return,
            Err(error) => panic!("unexpected error: {error:?}"),
        };
    assert!(
        matches!(
            requirement,
            ExecApprovalRequirement::NeedsApproval { .. }
                | ExecApprovalRequirement::NeedsHumanApproval { .. }
                | ExecApprovalRequirement::Forbidden { .. }
        ),
        "{requirement:?}"
    );
}

#[tokio::test]
async fn shell_intercept_uses_consecutive_grammar_failures_and_rejects_after_fallback() {
    let (session, turn) = make_session_and_context().await;
    let session = Arc::new(session);
    let turn = Arc::new(turn);
    let turn_environment = turn
        .initial_environments
        .primary()
        .cloned()
        .expect("test turn should have a primary environment");
    let malformed = vec![
        "apply_patch".to_string(),
        "*** Begin Patch\n*** Update File: broken.txt\n*** End Patch".to_string(),
    ];

    response_error(
        intercept_for_test(
            &malformed,
            Arc::clone(&session),
            Arc::clone(&turn),
            turn_environment.clone(),
        )
        .await,
    );
    assert!(!turn.structured_edit_fallback_enabled());

    let valid_but_implicit = vec![sample_patch().to_string()];
    response_error(
        intercept_for_test(
            &valid_but_implicit,
            Arc::clone(&session),
            Arc::clone(&turn),
            turn_environment.clone(),
        )
        .await,
    );
    assert!(!turn.structured_edit_fallback_enabled());

    response_error(
        intercept_for_test(
            &malformed,
            Arc::clone(&session),
            Arc::clone(&turn),
            turn_environment.clone(),
        )
        .await,
    );
    assert!(!turn.structured_edit_fallback_enabled());
    let activation = response_error(
        intercept_for_test(
            &malformed,
            Arc::clone(&session),
            Arc::clone(&turn),
            turn_environment.clone(),
        )
        .await,
    );
    assert!(activation.contains(STRUCTURED_EDIT_FALLBACK_GUIDANCE));
    assert!(turn.structured_edit_fallback_enabled());

    let rejected = vec!["apply_patch".to_string(), sample_patch().to_string()];
    let rejection = response_error(
        intercept_for_test(
            &rejected,
            Arc::clone(&session),
            Arc::clone(&turn),
            turn_environment.clone(),
        )
        .await,
    );
    assert!(rejection.contains(STRUCTURED_EDIT_FALLBACK_GUIDANCE));
    assert!(
        !turn_environment
            .cwd()
            .join("hello.txt")
            .expect("valid child path")
            .to_path_buf()
            .exists(),
        "fallback rejection must occur before filesystem mutation"
    );
}

#[test]
fn write_permissions_for_paths_skip_dirs_already_writable_under_workspace_root() {
    let tmp = TempDir::new().expect("tmp");
    let cwd_path = tmp.path();
    let cwd = cwd_path.abs();
    let nested = cwd_path.join("nested");
    std::fs::create_dir_all(&nested).expect("create nested dir");
    let file_path = AbsolutePathBuf::try_from(nested.join("file.txt"))
        .expect("nested file path should be absolute");
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ false,
    );

    let permissions = write_permissions_for_paths(
        &[file_path.into()],
        &PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled)
            .prepare_matching(&sandbox_policy, &local_context(&cwd.into()))
            .expect("prepare patch matching"),
    )
    .expect("derive write permissions");

    assert_eq!(permissions, None);
}

#[test]
fn write_permissions_for_paths_keep_dirs_outside_workspace_root() {
    let tmp = TempDir::new().expect("tmp");
    let cwd = tmp.path().join("workspace");
    let outside = tmp.path().join("outside");
    std::fs::create_dir_all(&cwd).expect("create cwd");
    std::fs::create_dir_all(&outside).expect("create outside dir");
    let file_path = AbsolutePathBuf::try_from(outside.join("file.txt"))
        .expect("outside file path should be absolute");
    let cwd_abs = cwd.abs();
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );

    let permissions = write_permissions_for_paths(
        &[file_path.into()],
        &PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled)
            .prepare_matching(&sandbox_policy, &local_context(&cwd_abs.into()))
            .expect("prepare patch matching"),
    )
    .expect("derive write permissions");
    let expected_outside = outside.abs();

    assert_eq!(
        permissions
            .and_then(|profile| profile.file_system)
            .and_then(|fs| fs.legacy_read_write_roots())
            .and_then(|roots| roots.write),
        Some(vec![expected_outside])
    );
}

/// #1057: the grants that widen the patch's policy (and so may let it
/// through unasked) are read once and carried by the request, so a
/// transition after that read is noticed by the orchestrator (it refuses a
/// request carrying permissions read before a transition).
#[tokio::test]
async fn issue_1057_grants_that_widen_the_patch_policy_are_carried() {
    let (session, turn) = make_session_and_context().await;
    let turn = Arc::new(turn);
    let outside = TempDir::new().expect("tempdir");
    let outside_root = outside.path().abs();
    assert!(
        session.record_granted_request_permissions_for_turn(
            &codex_protocol::request_permissions::RequestPermissionsResponse {
                permissions: codex_protocol::request_permissions::RequestPermissionProfile {
                    file_system: Some(
                        codex_protocol::models::FileSystemPermissions::from_read_write_roots(
                            /*read*/ None,
                            Some(vec![outside_root.clone()]),
                        ),
                    ),
                    ..Default::default()
                },
                scope: codex_protocol::request_permissions::PermissionGrantScope::Session,
                strict_auto_review: false,
            },
            codex_exec_server::LOCAL_ENVIRONMENT_ID,
            &turn,
            session.services.authority_marker(),
        ),
        "the grant is recorded"
    );
    let turn_environment = turn
        .initial_environments
        .primary()
        .cloned()
        .expect("test turn should have a primary environment");
    let path = PathUri::from_abs_path(&outside_root.join("new.txt"));
    let action = ApplyPatchAction::new_add_for_test(&path, "content".to_string());
    let step_context = StepContext::for_test(Arc::clone(&turn));

    let (request, _handler_basis) =
        prepare_verified_patch(action, turn_environment, &session, &step_context)
            .await
            .expect("native paths");

    assert!(
        request.additional_permissions.is_some(),
        "the grant widens the policy and is carried"
    );
    assert!(request.permissions_preapproved);
}

#[test]
fn write_permissions_for_paths_do_not_widen_workspace_root_target() {
    let tmp = TempDir::new().expect("tmp");
    let cwd = tmp.path().join("workspace").abs();
    std::fs::create_dir_all(&cwd).expect("create workspace");
    let sandbox_policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );
    let permissions = write_permissions_for_paths(
        &[cwd.clone().into()],
        &PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled)
            .prepare_matching(&sandbox_policy, &local_context(&cwd.into()))
            .expect("prepare patch matching"),
    )
    .expect("derive write permissions");

    assert_eq!(permissions, None);
}

#[test]
fn write_permissions_for_paths_do_not_regrant_an_already_writable_parent() {
    let tmp = TempDir::new().expect("tmp");
    let cwd = tmp.path().abs();
    let file_path = cwd.join("protected.txt");
    let sandbox_policy = FileSystemSandboxPolicy::restricted(vec![
        FileSystemSandboxEntry::new(cwd.clone().into(), FileSystemAccessMode::Write),
        FileSystemSandboxEntry::new(file_path.clone().into(), FileSystemAccessMode::Read),
    ]);

    let permissions = write_permissions_for_paths(
        &[file_path.into()],
        &PatchSandboxRoute::Platform(WindowsSandboxLevel::Disabled)
            .prepare_matching(&sandbox_policy, &local_context(&cwd.into()))
            .expect("prepare patch matching"),
    )
    .expect("derive write permissions");

    assert_eq!(permissions, None);
}

#[test]
fn write_permissions_for_windows_paths_uses_executor_uris() {
    let cwd = PathUri::parse("file:///C:/workspace").expect("Windows cwd");
    let context = local_context(&cwd);
    let policy = FileSystemSandboxPolicy::workspace_write(
        &[],
        /*exclude_tmpdir_env_var*/ true,
        /*exclude_slash_tmp*/ true,
    );
    let outside = PathUri::parse("file:///C:/outside/out.txt").expect("outside");

    assert_eq!(
        write_permissions_for_paths(
            &[outside],
            &PatchSandboxRoute::ExecutorManaged
                .prepare_matching(&policy, &context)
                .expect("prepare patch matching"),
        )
        .expect("derive write permissions")
        .and_then(|profile| profile.file_system)
        .map(|permissions| permissions.entries),
        Some(vec![FileSystemSandboxEntry::new(
            PathUri::parse("file:///C:/outside")
                .expect("outside parent")
                .into(),
            FileSystemAccessMode::Write,
        )]),
    );
}
