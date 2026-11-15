//! Shell parity (S3b): main tested these protections through the
//! `shell_command` tool, which upstream removed (D275). `exec_command` (unified
//! exec) now runs every command, so the same checks run a real command through
//! it: `core/src/tools/handlers/shell_tests.rs` (#310) and
//! `core/tests/suite/shell_command.rs` (keyring isolation, Task Node scope) on
//! main.

use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::ExecCommandHandler;
use crate::turn_diff_tracker::TurnDiffTracker;
use codex_protocol::config_types::ShellEnvironmentPolicy;
use codex_protocol::config_types::ShellEnvironmentPolicyInherit;
use codex_protocol::models::PermissionProfile;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::protocol::AskForApproval;
use codex_tools::ToolExecutor;
use pretty_assertions::assert_eq;
use std::sync::Arc;

/// A session whose commands run unsandboxed without prompts, with `policy`
/// applied to the turn's environment (upstream reads the policy from the
/// environment) and the config.
async fn session_with(
    policy: impl Fn(&mut ShellEnvironmentPolicy),
    config_edit: impl FnOnce(&mut crate::config::Config),
) -> (Arc<Session>, Arc<TurnContext>) {
    let (session, mut turn) = crate::session::tests::make_session_and_context().await;
    {
        let config = Arc::make_mut(&mut turn.config);
        config
            .permissions
            .approval_policy
            .set(AskForApproval::Never)
            .expect("approval policy");
        config
            .permissions
            .set_permission_profile(PermissionProfile::Disabled)
            .expect("permission profile");
        policy(&mut config.permissions.shell_environment_policy);
        config_edit(config);
    }
    for environment in &mut turn.initial_environments.environments {
        if let crate::environment_selection::TurnEnvironmentState::Ready(environment) = environment
        {
            let environment = environment.config_mut();
            environment.permission_profile =
                crate::config::PermissionProfileSnapshot::legacy(PermissionProfile::Disabled);
            policy(&mut environment.shell_environment_policy);
        }
    }
    (Arc::new(session), Arc::new(turn))
}

/// Runs `cmd` through `exec_command` and returns the command's output.
async fn exec_command_output(session: &Arc<Session>, turn: &Arc<TurnContext>, cmd: &str) -> String {
    let invocation = ToolInvocation {
        environments: Default::default(),
        native_call: None,
        session: Arc::clone(session),
        step_context: StepContext::for_test(Arc::clone(turn)),
        turn: Arc::clone(turn),
        cancellation_token: tokio_util::sync::CancellationToken::new(),
        tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
        call_id: "call-parity".to_string(),
        tool_name: codex_tools::ToolName::plain("exec_command"),
        source: crate::tools::context::ToolCallSource::Direct,
        payload: ToolPayload::Function {
            arguments: serde_json::json!({
                "cmd": cmd,
                "login": false,
                "yield_time_ms": 30_000,
            })
            .to_string(),
        },
    };
    let response = ExecCommandHandler::default()
        .handle(invocation)
        .await
        .unwrap_or_else(|err| panic!("exec_command failed: {err:?}"));
    match response.to_response_item(
        "call-parity",
        &ToolPayload::Function {
            arguments: "{}".to_string(),
        },
    ) {
        ResponseInputItem::FunctionCallOutput { output, .. }
        | ResponseInputItem::CustomToolCallOutput { output, .. } => {
            output.body.to_text().unwrap_or_default()
        }
        other => panic!("expected a function call output, got {other:?}"),
    }
}

/// The command's own output lines (the tool output adds a header).
fn output_line_after<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|line| line.trim().strip_prefix(marker))
        .map(str::trim)
}

/// Issue #310: provider keys are removed from inherited variables, but a policy
/// `set` entry is an explicit opt-in and still reaches the command.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_310_exec_command_keeps_provider_auth_env_set_explicitly_by_policy() {
    // An inherited credential of a configured provider, set by nothing else.
    let inherited = format!("S3B_PARITY_PROVIDER_TOKEN_{}", std::process::id());
    // SAFETY: a variable unique to this test; nothing else reads it.
    unsafe { std::env::set_var(&inherited, "inherited-provider-secret") };
    let inherited_key = inherited.clone();
    let (session, turn) = session_with(
        |policy| {
            for (name, value) in [
                ("OPENAI_API_KEY", "openai-provider-secret"),
                ("CORP_MODEL_TOKEN", "custom-provider-secret"),
                ("GENERIC_API_KEY", "workflow-secret"),
            ] {
                policy.r#set.insert(name.to_string(), value.to_string());
            }
        },
        move |config| {
            let mut custom_provider = config.model_provider.clone();
            custom_provider.env_key = Some("CORP_MODEL_TOKEN".to_string());
            config
                .model_providers
                .insert("corp".to_string(), custom_provider.clone());
            let mut inherited_provider = custom_provider;
            inherited_provider.env_key = Some(inherited_key);
            config
                .model_providers
                .insert("inherited".to_string(), inherited_provider);
        },
    )
    .await;
    let text = exec_command_output(
        &session,
        &turn,
        &format!(
            r#"printf 'keys=%s|%s|%s|%s\n' "$OPENAI_API_KEY" "$CORP_MODEL_TOKEN" "$GENERIC_API_KEY" "${inherited}""#
        ),
    )
    .await;
    assert_eq!(
        output_line_after(&text, "keys="),
        Some("openai-provider-secret|custom-provider-secret|workflow-secret|"),
        "{text}"
    );
}

/// Debug builds: the keyring-isolation flag reaches agent commands even though
/// its name matches the default `*KEY*` exclude, and with `inherit = "none"`,
/// so a `corbanu` an agent starts in a test or demo profile stays off the real
/// OS keyring.
#[cfg(debug_assertions)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn keyring_isolation_reaches_exec_command() {
    const NAME: &str = codex_protocol::shell_environment::NO_NATIVE_KEYRING_ENV_VAR;
    // `just test` and CI set it; a runner that does not is not keyring-isolated,
    // so there is nothing to check.
    let Ok(expected) = std::env::var(NAME) else {
        return;
    };
    for inherit_none in [false, true] {
        let (session, turn) = session_with(
            move |policy| {
                policy.ignore_default_excludes = false;
                if inherit_none {
                    policy.inherit = ShellEnvironmentPolicyInherit::None;
                }
            },
            |_| {},
        )
        .await;
        let text =
            exec_command_output(&session, &turn, &format!("printf 'flag=%s\\n' \"${NAME}\"")).await;
        assert_eq!(
            output_line_after(&text, "flag="),
            Some(expected.as_str()),
            "inherit none: {inherit_none}: {text}"
        );
    }
}

/// The Task Node profile and `CODEX_HOME` a command sees come from the session's
/// own profile and home, not from stale policy `set` values.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tasknode_profile_scope_survives_exec_command_overrides() {
    for profile in [Some("alice"), Some("bob"), None] {
        let (session, turn) = session_with(
            |policy| {
                policy.r#set.insert(
                    "CORBANU_TASKNODE_PROFILE".to_string(),
                    "\"stale\"".to_string(),
                );
                policy
                    .r#set
                    .insert("CODEX_HOME".to_string(), "stale-home".to_string());
            },
            move |config| {
                config.config_layer_stack = codex_config::ConfigLayerStack::new(
                    vec![codex_config::ConfigLayerEntry::new(
                        codex_config::ConfigLayerSource::User {
                            file: config.codex_home.join("config.toml"),
                            profile: profile.map(|name| name.parse().unwrap()),
                        },
                        toml::Value::Table(Default::default()),
                    )],
                    Default::default(),
                    Default::default(),
                )
                .unwrap();
            },
        )
        .await;
        let text = exec_command_output(
            &session,
            &turn,
            r#"printf 'profile=%s\nhome=%s\n' "$CORBANU_TASKNODE_PROFILE" "$CODEX_HOME""#,
        )
        .await;
        let expected_profile = serde_json::to_string(&profile).unwrap();
        let expected_home = turn.config.codex_home.display().to_string();
        assert_eq!(
            (
                output_line_after(&text, "profile="),
                output_line_after(&text, "home=")
            ),
            (
                Some(expected_profile.as_str()),
                Some(expected_home.as_str())
            ),
            "{profile:?}: {text}"
        );
    }
}

/// A session at `level` (the live level the model client reports), with the
/// Linux sandbox helper so protected levels keep the command sandboxed.
#[cfg(target_os = "linux")]
async fn session_at_level(
    level: codex_security_policy::SecurityLevel,
) -> Option<(Arc<Session>, Arc<TurnContext>)> {
    let (session, mut turn) = crate::session::tests::make_session_and_context().await;
    let config = Arc::make_mut(&mut turn.config);
    config
        .permissions
        .approval_policy
        .set(AskForApproval::Never)
        .expect("approval policy");
    config.codex_linux_sandbox_exe = Some(core_test_support::find_codex_linux_sandbox_exe().ok()?);
    let client = (*session.services.model_client())
        .clone()
        .with_ingress_level(level)
        .with_source_envelopes(/*enabled*/ true);
    session.services.replace_model_client(client);
    Some((Arc::new(session), Arc::new(turn)))
}

/// #442 (AGG-09) on the unified-exec path: under Aggressive, a command whose
/// read of a protected file the sandbox denied is explained in its output,
/// naming the level and /security.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_442_exec_command_explains_a_denied_protected_read() {
    let Some((session, turn)) =
        session_at_level(codex_security_policy::SecurityLevel::Aggressive).await
    else {
        eprintln!("skipping: no codex-linux-sandbox helper");
        return;
    };
    let home = turn.config.codex_home.to_path_buf();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("auth.json"), "issue-442-secret").unwrap();
    let text = exec_command_output(
        &session,
        &turn,
        &format!("cat '{}'", home.join("auth.json").display()),
    )
    .await;
    assert!(!text.contains("issue-442-secret"), "{text}");
    assert!(
        text.contains("Security level Aggressive blocked access to")
            && text.contains("switch to Permissive in /security"),
        "{text}"
    );
}

/// #826 follow-up on the unified-exec path: under Moderate on Linux, a
/// command reporting a refused user namespace is explained in its output,
/// naming the level, instead of a bare "Operation not permitted".
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_exec_command_explains_a_refused_user_namespace() {
    let Some((session, turn)) =
        session_at_level(codex_security_policy::SecurityLevel::Moderate).await
    else {
        eprintln!("skipping: no codex-linux-sandbox helper");
        return;
    };
    let text = exec_command_output(
        &session,
        &turn,
        "unshare --user true || { echo 'unshare: unshare failed: Operation not permitted' >&2; exit 1; }",
    )
    .await;
    assert!(
        text.contains("Security level Moderate blocks creating user namespaces"),
        "{text}"
    );
}
