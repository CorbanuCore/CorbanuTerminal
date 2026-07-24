use super::app_server_sandbox_mode;
use super::thread_resume_has_no_rollout;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::SandboxMode;
use codex_protocol::protocol::NetworkAccess;
use codex_protocol::protocol::SandboxPolicy;

#[test]
fn only_missing_rollouts_are_replaceable_resume_failures() {
    let missing = JSONRPCErrorError {
        code: -32600,
        message: "no rollout found for thread id stale-thread".into(),
        data: None,
    };
    assert!(thread_resume_has_no_rollout("thread/resume", &missing));
    assert!(!thread_resume_has_no_rollout("thread/read", &missing));

    for (code, message) in [
        (-32600, "thread is temporarily unavailable"),
        (-32603, "no rollout found for thread id stale-thread"),
    ] {
        let error = JSONRPCErrorError {
            code,
            message: message.into(),
            data: None,
        };
        assert!(!thread_resume_has_no_rollout("thread/resume", &error));
    }
}

#[test]
fn telegram_propagates_each_configured_sandbox_mode_to_app_server_threads() {
    assert_eq!(
        app_server_sandbox_mode(&SandboxPolicy::ReadOnly {
            network_access: false,
        }),
        SandboxMode::ReadOnly
    );
    assert_eq!(
        app_server_sandbox_mode(&SandboxPolicy::WorkspaceWrite {
            writable_roots: Vec::new(),
            network_access: false,
            exclude_tmpdir_env_var: false,
            exclude_slash_tmp: false,
        }),
        SandboxMode::WorkspaceWrite
    );
    assert_eq!(
        app_server_sandbox_mode(&SandboxPolicy::DangerFullAccess),
        SandboxMode::DangerFullAccess
    );
    assert_eq!(
        app_server_sandbox_mode(&SandboxPolicy::ExternalSandbox {
            network_access: NetworkAccess::Enabled,
        }),
        SandboxMode::DangerFullAccess
    );
}
