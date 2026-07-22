use super::thread_resume_has_no_rollout;
use codex_app_server_protocol::JSONRPCErrorError;

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
