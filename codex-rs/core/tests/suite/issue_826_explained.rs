//! #826 follow-up (decided with #826): where the Linux rule of Moderate and
//! Aggressive (also where only the launch enforces Aggressive) refuses a
//! command's user namespace, the command result, the person's transcript
//! and the inspector's recent denials name the rule in plain language,
//! through both shell tools, instead of a bare "Operation not permitted";
//! a person asked to approve such a command is told first. Permissive is
//! unchanged, and the command still fails (nothing is lifted).
#![cfg(target_os = "linux")]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_core::security_inspection;
use codex_features::Feature;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::skip_if_sandbox;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::TempDir;

const CALL_ID: &str = "call-826-explained";
const NAMESPACE_MADE: &str = "NAMESPACE-MADE-826";
const SCRIPT: &str = "unshare -Ur true && echo NAMESPACE-MADE-826";
/// What `unshare` prints when the sandbox refuses the namespace.
const REFUSED: &str = "unshare failed: Operation not permitted";
/// The start of the explanation, after the level's name.
const EXPLAINED: &str = "blocks creating user namespaces for agent commands on Linux, and this command's output reports";
/// The start of the approval prompt's note.
const NOTE: &str = "On Linux, security level";
/// The inspector's recent denial.
const INSPECTED: &str = "creating a user namespace";

#[derive(Clone, Copy, Debug)]
enum Tool {
    ShellCommand,
    ExecCommand,
    /// `shell_command` with the model's justification.
    Justified,
}

/// The model's justification in [`Tool::Justified`].
const JUSTIFICATION: &str = "a container needs a namespace";

#[derive(Debug)]
struct Outcome {
    /// The output the model got.
    output: String,
    warnings: Vec<String>,
    /// Reasons of the approval prompts the person saw.
    asked: Vec<String>,
    denials: Vec<&'static str>,
}

async fn run(
    level: SecurityLevel,
    aggressive_launch: bool,
    tool: Tool,
    approval: AskForApproval,
) -> anyhow::Result<Outcome> {
    let home = Arc::new(TempDir::new()?);
    let origin = home.path().canonicalize()?.to_string_lossy().into_owned();
    let (name, arguments) = match tool {
        Tool::ShellCommand => ("shell_command", json!({ "command": SCRIPT })),
        Tool::ExecCommand => (
            "exec_command",
            json!({ "cmd": SCRIPT, "yield_time_ms": 5_000 }),
        ),
        Tool::Justified => (
            "shell_command",
            json!({ "command": SCRIPT, "justification": JUSTIFICATION }),
        ),
    };
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created(CALL_ID),
                ev_function_call(CALL_ID, name, &arguments.to_string()),
                ev_completed(CALL_ID),
            ]),
            sse(vec![
                ev_response_created("done"),
                ev_assistant_message("msg-done", "done"),
                ev_completed("done"),
            ]),
        ],
    )
    .await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = level;
            config.permissions.approval_policy = Constrained::allow_any(approval);
            for feature in [Feature::SourceEnvelopes, Feature::UnifiedExec] {
                config
                    .features
                    .enable(feature)
                    .expect("test config should allow feature update");
            }
            config
                .permissions
                .set_permission_profile(PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                ))
                .expect("workspace write");
            if aggressive_launch {
                // What an Aggressive launch puts into every agent command's
                // environment (`launch_enforces_aggressive`).
                config
                    .permissions
                    .shell_environment_policy
                    .r#set
                    .insert(codex_security_level::nested::ORIGIN_ENV.to_string(), origin);
            }
        })
        // Host-sandbox security test: local executor on purpose.
        .build(&server)
        .await?;
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "make a user namespace".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let mut warnings = Vec::new();
    let mut asked = Vec::new();
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(
                event,
                EventMsg::Warning(_) | EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
            )
        })
        .await
        {
            EventMsg::Warning(warning) => warnings.push(warning.message),
            EventMsg::ExecApprovalRequest(request) => {
                asked.push(request.reason.clone().unwrap_or_default());
                test.codex
                    .submit(Op::ExecApproval {
                        id: request.effective_approval_id(),
                        turn_id: None,
                        decision: ReviewDecision::Approved,
                    })
                    .await?;
            }
            _ => break,
        }
    }
    let output = captured
        .requests()
        .iter()
        .find_map(|request| request.function_call_output_text(CALL_ID))
        .unwrap_or_default();
    let denials = security_inspection::observe(
        test.codex_home_path(),
        level,
        Some(test.session_configured.thread_id),
        i64::MAX / 2,
    )
    .denials
    .into_iter()
    .map(|denial| denial.reason)
    .collect();
    Ok(Outcome {
        output,
        warnings,
        asked,
        denials,
    })
}

/// Under the rule, through both shell tools: still refused, and named to the
/// model, to the person and in the inspector, with the level.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_refused_user_namespace_is_explained() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (level, aggressive_launch, named) in [
        (SecurityLevel::Moderate, false, "Moderate"),
        (SecurityLevel::Aggressive, false, "Aggressive"),
        // Only the launch enforces Aggressive (Core's level stays lower).
        (SecurityLevel::Permissive, true, "Aggressive"),
    ] {
        for tool in [Tool::ShellCommand, Tool::ExecCommand] {
            // Boxed: the session future is large for the test thread's stack.
            let outcome =
                Box::pin(run(level, aggressive_launch, tool, AskForApproval::Never)).await?;
            let explanation = format!("Security level {named} {EXPLAINED}");
            let ok = outcome.output.contains(REFUSED)
                && !outcome.output.contains(NAMESPACE_MADE)
                && outcome.output.contains(&explanation)
                && outcome
                    .warnings
                    .iter()
                    .any(|warning| warning.starts_with(&explanation))
                && outcome.denials.contains(&INSPECTED);
            if !ok {
                wrong.push(format!(
                    "{level:?} aggressive_launch={aggressive_launch} {tool:?}: {outcome:?}"
                ));
            }
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// A person asked to approve `unshare` under the rule is told it will fail
/// and why, after the model's justification, in the ordinary prompt;
/// approving it does not lift the rule. (Moderate's prompt for a request to
/// run outside the sandbox lifts it for that run since D27: see
/// `issue_891_one_run`.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_approval_prompt_names_the_rule() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let note = format!("{NOTE} Moderate blocks creating user namespaces");
    let mut wrong = Vec::new();
    for (tool, approval, justified) in [
        (Tool::ShellCommand, AskForApproval::UnlessTrusted, false),
        (Tool::Justified, AskForApproval::UnlessTrusted, true),
    ] {
        let outcome = Box::pin(run(
            SecurityLevel::Moderate,
            /*aggressive_launch*/ false,
            tool,
            approval,
        ))
        .await?;
        let asked_ok = outcome.asked.iter().any(|reason| {
            reason.contains(&note)
                && (!justified
                    || reason
                        .find(JUSTIFICATION)
                        .is_some_and(|at| reason.find(&note).is_some_and(|note| at < note)))
        });
        if !asked_ok || !outcome.output.contains(REFUSED) || outcome.output.contains(NAMESPACE_MADE)
        {
            wrong.push(format!("{tool:?} {approval:?}: {outcome:?}"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// Permissive: no rule, so nothing names it (the namespace is made where
/// the host allows a nested one, as before), and no approval note.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_826_permissive_names_no_rule() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    for (tool, approval) in [
        (Tool::ShellCommand, AskForApproval::Never),
        (Tool::ExecCommand, AskForApproval::Never),
        (Tool::ShellCommand, AskForApproval::UnlessTrusted),
    ] {
        let outcome = Box::pin(run(
            SecurityLevel::Permissive,
            /*aggressive_launch*/ false,
            tool,
            approval,
        ))
        .await?;
        eprintln!("Permissive {tool:?} {approval:?}: {outcome:?}");
        assert!(
            !outcome.output.contains(EXPLAINED)
                && !outcome.warnings.iter().any(|w| w.contains(EXPLAINED))
                && !outcome.asked.iter().any(|reason| reason.contains(NOTE))
                && !outcome.denials.contains(&INSPECTED),
            "{tool:?}: {outcome:?}"
        );
    }
    Ok(())
}
