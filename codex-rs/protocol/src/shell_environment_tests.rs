use std::collections::HashMap;
use std::process::Command;

use pretty_assertions::assert_eq;

use super::*;
use crate::secretless_launch;

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

const CHILD_MODE_ENV_VAR: &str = "CODEX_SHELL_ENVIRONMENT_SCRUBBER_TEST_MODE";
const TEST_NAME: &str =
    "shell_environment::tests::command_scrubber_removes_names_from_real_child_environment";

#[test]
fn tool_call_id_replaces_platform_equivalent_names() {
    let representable = format!("exec-雪-{}", "x".repeat(512));
    for (call_id, expected_call) in [
        (None, None),
        (Some(""), Some("")),
        (Some("invalid\0call"), None),
        (Some("current-call"), Some("current-call")),
        (Some(representable.as_str()), Some(representable.as_str())),
    ] {
        let mut env = HashMap::from([
            ("CODEX_TOOL_CALL_ID".to_string(), "stale-call".to_string()),
            (
                "Codex_Tool_Call_Id".to_string(),
                "mixed-case-call".to_string(),
            ),
            (
                "codex_tool_call_id".to_string(),
                "lowercase-call".to_string(),
            ),
            ("OTHER".to_string(), "unchanged".to_string()),
        ]);

        set_tool_call_id_env_var(&mut env, call_id);

        let mut expected = HashMap::from([("OTHER".to_string(), "unchanged".to_string())]);
        #[cfg(not(windows))]
        expected.extend([
            (
                "Codex_Tool_Call_Id".to_string(),
                "mixed-case-call".to_string(),
            ),
            (
                "codex_tool_call_id".to_string(),
                "lowercase-call".to_string(),
            ),
        ]);
        if let Some(expected_call) = expected_call {
            expected.insert("CODEX_TOOL_CALL_ID".to_string(), expected_call.to_string());
        }
        assert_eq!(env, expected, "call_id: {call_id:?}");
    }
}

#[test]
fn non_inheritable_environment_is_removed_after_policy_overrides() {
    let vars = [
        ("SAFE".to_string(), "inherited".to_string()),
        (
            "codex_guardian_decisions_api_key".into(),
            "synthetic-inherited-key".into(),
        ),
        ("node_repl_auth_token".into(), "inherited-token".into()),
        (
            "openai_federation_rule_id".to_string(),
            "inherited-rule".to_string(),
        ),
        (
            "OPENAI_WORKLOAD_IDENTITY_CONTEXT".to_string(),
            r#"{"instance_id":"box-one"}"#.to_string(),
        ),
        (
            "codex_exec_server_noise_auth_token".to_string(),
            "inherited-noise-token".to_string(),
        ),
    ];
    let policy = ShellEnvironmentPolicy {
        inherit: ShellEnvironmentPolicyInherit::All,
        ignore_default_excludes: true,
        r#set: HashMap::from([
            ("SAFE".to_string(), "override".to_string()),
            (
                "CODEX_GUARDIAN_DECISIONS_API_KEY".into(),
                "synthetic-override-key".into(),
            ),
            ("Node_Repl_Auth_Token".into(), "configured-token".into()),
            (
                "OpenAI_Identity_Token_File".to_string(),
                "/run/identity-token".to_string(),
            ),
            (
                "Codex_Exec_Server_Noise_Auth_Token".to_string(),
                "configured-noise-token".to_string(),
            ),
        ]),
        ..Default::default()
    };

    assert_eq!(
        populate_env(vars, &policy, /*thread_id*/ None),
        HashMap::from([("SAFE".to_string(), "override".to_string())])
    );
}

#[test]
fn command_scrubber_removes_names_from_real_child_environment() {
    if std::env::var_os(CHILD_MODE_ENV_VAR).is_none() {
        let output = Command::new(std::env::current_exe().expect("locate current test binary"))
            .args([TEST_NAME, "--exact", "--nocapture"])
            .env(CHILD_MODE_ENV_VAR, "1")
            .env(
                "Codex_Guardian_Decisions_Api_Key",
                "synthetic-inherited-key",
            )
            .env("Node_Repl_Auth_Token", "inherited-token")
            .env("OpenAI_Federation_Rule_Id", "inherited-rule")
            .env(
                "OpenAI_Workload_Identity_Context",
                r#"{"instance_id":"box-one"}"#,
            )
            .env(
                "Codex_Exec_Server_Noise_Auth_Token",
                "inherited-noise-token",
            )
            .output()
            .expect("run inherited-environment test process");
        assert!(
            output.status.success(),
            "child failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return;
    }

    let mut command = environment_command();
    command
        .env("NODE_REPL_AUTH_TOKEN", "configured-token")
        .env("openai_identity_token_file", "/run/identity-token")
        .env(
            CODEX_EXEC_SERVER_NOISE_AUTH_TOKEN_ENV_VAR,
            "configured-noise-token",
        )
        .env("CODEX_GUARDIAN_DECISIONS_API_KEY", "synthetic-override-key")
        .env("SAFE", "value");
    scrub_non_inheritable_env_vars(&mut command);
    let output = command.output().expect("read child environment");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let restricted_names = stdout
        .lines()
        .filter_map(|line| line.split_once('=').map(|(name, _)| name))
        .filter(|name| is_non_inheritable_env_var(name))
        .collect::<Vec<_>>();
    assert_eq!(restricted_names, Vec::<&str>::new());
}

#[cfg(windows)]
fn environment_command() -> Command {
    let mut command = Command::new("cmd.exe");
    command.args(["/D", "/C", "set"]);
    command
}

#[cfg(not(windows))]
fn environment_command() -> Command {
    Command::new("env")
}
