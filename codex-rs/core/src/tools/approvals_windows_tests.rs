use super::*;

/// #1863: a `\\localhost` UNC cwd reaches the automatic reviewer's request
/// without a panic in `url` (upstream renders it with `LegacyAppPathString`).
#[test]
fn guardian_request_with_localhost_unc_cwd_does_not_panic() {
    let native_cwd = AbsolutePathBuf::from_absolute_path_checked(r"\\localhost\C$\workspace")
        .expect("UNC path should be absolute");
    let action = ApprovalAction::ExecCommand {
        id: "issue-1863".to_string(),
        environment_id: codex_exec_server::LOCAL_ENVIRONMENT_ID.to_string(),
        command: vec!["dir".to_string()],
        hook_command: "dir".to_string(),
        cwd: PathUri::from_abs_path(&native_cwd),
        sandbox_permissions: SandboxPermissions::UseDefault,
        additional_permissions: None,
        justification: None,
        tty: false,
        proposed_execpolicy_amendment: None,
    };

    // Either outcome is fine; reaching it is the point.
    let _ = action.into_guardian_request(Some(PathConvention::Windows));
}
