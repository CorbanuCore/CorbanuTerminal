use pretty_assertions::assert_eq;

use super::*;

const CHILD_ENV: &str = "CODEX_CORE_HOOK_SECRETLESS_LAUNCH_CHILD";

/// Runs `check` in a child test process, so arming secretless launch (one-way,
/// process-wide) never reaches the other tests.
fn in_isolated_child(test: &str, check: impl FnOnce()) {
    if std::env::var_os(CHILD_ENV).is_some() {
        check();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test, "--nocapture"])
        .env(CHILD_ENV, "1")
        .output()
        .expect("run child test");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "child did not run {test}"
    );
}

#[test]
fn pf_27_s02_armed_local_environment_hooks_never_use_a_login_shell() {
    in_isolated_child(
        "hook_environment_executor::tests::pf_27_s02_armed_local_environment_hooks_never_use_a_login_shell",
        || {
            codex_protocol::secretless_launch::arm();
            let shell = crate::shell::Shell {
                shell_type: ShellType::Bash,
                shell_path: "/bin/bash".into(),
            };
            let argv = |remote| {
                shell.derive_exec_args(
                    "true",
                    hook_uses_login_shell(/*allow_login_shell*/ true, remote),
                )
            };
            assert_eq!(
                (argv(/*remote*/ false), argv(/*remote*/ true)),
                (
                    vec![
                        "/bin/bash".to_string(),
                        "-c".to_string(),
                        "true".to_string()
                    ],
                    vec![
                        "/bin/bash".to_string(),
                        "-lc".to_string(),
                        "true".to_string()
                    ],
                )
            );
            assert!(!hook_uses_login_shell(
                /*allow_login_shell*/ false, /*remote*/ true
            ));
        },
    );
}
