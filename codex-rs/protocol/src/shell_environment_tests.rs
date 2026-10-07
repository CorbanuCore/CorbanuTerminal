use super::*;
use crate::secretless_launch;
use pretty_assertions::assert_eq;

const CHILD_ENV: &str = "CODEX_PROTOCOL_KEYRING_ISOLATION_CHILD";

/// Runs `test` in a child test process that has the isolation variable set,
/// so this process's environment is never changed.
fn in_isolated_child(test: &str, check: impl FnOnce()) {
    if std::env::var_os(CHILD_ENV).is_some() {
        check();
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", test, "--nocapture"])
        .env(CHILD_ENV, "1")
        .env(NO_NATIVE_KEYRING_ENV_VAR, "1")
        .output()
        .expect("run child test");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(debug_assertions)]
#[test]
fn keyring_isolation_survives_filtered_child_env() {
    in_isolated_child(
        "shell_environment::tests::keyring_isolation_survives_filtered_child_env",
        || {
            let expected = Some(&"1".to_string());
            // The default `*KEY*` exclude would drop it.
            let policy = ShellEnvironmentPolicy {
                ignore_default_excludes: false,
                ..Default::default()
            };
            let env = create_env(&policy, /*thread_id*/ None);
            assert_eq!(env.get(NO_NATIVE_KEYRING_ENV_VAR), expected);
            // So would an `include_only` list or `inherit = "none"`.
            let policy = ShellEnvironmentPolicy {
                inherit: ShellEnvironmentPolicyInherit::None,
                include_only: vec![EnvironmentVariablePattern::new_case_insensitive("PATH")],
                ..Default::default()
            };
            let env = create_env(&policy, /*thread_id*/ None);
            assert_eq!(env.get(NO_NATIVE_KEYRING_ENV_VAR), expected);
            // The secretless-launch allowlist (hooks, MCP servers) keeps it too,
            // and so does `create_env` once that contract is armed (this child
            // process only).
            assert!(secretless_launch::is_launch_env_name_allowed(
                NO_NATIVE_KEYRING_ENV_VAR
            ));
            secretless_launch::arm();
            let env = create_env(&policy, /*thread_id*/ None);
            assert_eq!(env.get(NO_NATIVE_KEYRING_ENV_VAR), expected);
        },
    );
}

#[test]
fn keyring_isolation_exemption_is_exact() {
    // Only the one known, non-secret name is exempt.
    for name in [
        "CORBANU_TEST_NO_NATIVE_KEYRING_X",
        "corbanu_test_no_native_keyring",
        "OTHER_KEYRING",
    ] {
        assert!(
            !secretless_launch::is_launch_env_name_allowed(name),
            "{name}"
        );
    }
    let vars = vec![(
        "CORBANU_TEST_NO_NATIVE_KEYRING_X".to_string(),
        "1".to_string(),
    )];
    let policy = ShellEnvironmentPolicy {
        ignore_default_excludes: false,
        ..Default::default()
    };
    assert_eq!(
        populate_env(vars, &policy, /*thread_id*/ None),
        HashMap::new()
    );
}
