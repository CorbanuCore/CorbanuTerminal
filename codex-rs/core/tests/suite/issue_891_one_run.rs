//! D27 (a) (Travis, 2026-10-19, security sign-off, #891): under Moderate on
//! Linux, a person may let ONE command create user namespaces by approving
//! its request to run outside the sandbox when asked, before untrusted
//! content; the prompt warns that this command can get past the file
//! protections. Never remembered: not for the next request (also after "for
//! the session"), not under a saved "don't ask again" rule, not for a
//! command that stays in the sandbox. Aggressive, an Aggressive launch and
//! a session with untrusted content keep the #826 rule.
#![cfg(target_os = "linux")]

use std::sync::Arc;

use codex_core::config::Constrained;
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
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

/// Prints [`NAMESPACE_MADE`] only when it runs: the command's own text
/// doesn't contain it (a refusal can quote the command).
const SCRIPT: &str = "echo marker=${CODEX_SANDBOX_USER_NAMESPACES_DENIED:-unset}; unshare -Ur true && echo NAMESPACE-MADE-D$((20+7))";
/// One plain command, which a saved rule can match (the script above is too
/// complex for exec-policy rules, so it is always asked about).
const PLAIN: &str = "unshare -Ur printf NAMESPACE-MADE-%s D27";
const NAMESPACE_MADE: &str = "NAMESPACE-MADE-D27";
/// What `unshare` prints when the sandbox refuses the namespace.
const REFUSED: &str = "unshare failed: Operation not permitted";
const JUSTIFICATION: &str = "rootless podman needs a user namespace";
/// The start of the prompt's warning.
const WARNING: &str = "Warning (Linux): approving also lets this one run create user namespaces";
/// What the warning must say (the owner's wording).
const PAST_THE_PROTECTIONS: &str = "this command can get past the file protections";

/// How a call runs its command.
#[derive(Clone, Copy, Debug)]
enum Kind {
    /// `shell_command` asking to run outside the sandbox
    /// (`require_escalated`), with a justification.
    Escalated,
    /// The same through `exec_command`.
    EscalatedExec,
    /// `shell_command` in the sandbox.
    Sandboxed,
}

/// One model step running `command`.
fn step(call_id: &str, command: &str, kind: Kind) -> Vec<Value> {
    let escalated = json!({
        "sandbox_permissions": "require_escalated",
        "justification": JUSTIFICATION,
    });
    let (tool, mut arguments) = match kind {
        Kind::Escalated | Kind::Sandboxed => ("shell_command", json!({ "command": command })),
        Kind::EscalatedExec => (
            "exec_command",
            json!({ "cmd": command, "yield_time_ms": 5_000 }),
        ),
    };
    if !matches!(kind, Kind::Sandboxed) {
        for key in ["sandbox_permissions", "justification"] {
            arguments[key] = escalated[key].clone();
        }
    }
    vec![
        ev_response_created(call_id),
        ev_function_call(call_id, tool, &arguments.to_string()),
        ev_completed(call_id),
    ]
}

#[derive(Clone, Copy, Debug)]
struct Setup {
    level: SecurityLevel,
    aggressive_launch: bool,
    /// A tool output (untrusted content) before the calls.
    untrusted_first: bool,
    /// A "don't ask again" rule saved earlier for `unshare`; the calls run
    /// [`PLAIN`] instead of [`SCRIPT`].
    saved_rule: bool,
    /// The person's settings turn the sandbox off (full access).
    full_access: bool,
    approval: AskForApproval,
}

const MODERATE: Setup = Setup {
    level: SecurityLevel::Moderate,
    aggressive_launch: false,
    untrusted_first: false,
    saved_rule: false,
    full_access: false,
    approval: AskForApproval::OnRequest,
};

/// What one call did.
#[derive(Debug, PartialEq, Eq)]
struct Call {
    /// Prompts shown for it, with their reasons.
    asked: Vec<String>,
    made: bool,
    refused: bool,
    /// The sandbox marked the command as denying user namespaces.
    marked: bool,
    /// The command ran and was not marked.
    unmarked: bool,
    output: String,
}

/// Runs one call per entry of `calls`, answering every prompt of call `n`
/// with `calls[n].1`.
async fn run(
    setup: Setup,
    calls: &[(Kind, ReviewDecision)],
) -> anyhow::Result<(Vec<Call>, Vec<String>)> {
    let home = Arc::new(TempDir::new()?);
    let origin = home.path().canonicalize()?.to_string_lossy().into_owned();
    if setup.saved_rule {
        std::fs::create_dir_all(home.path().join("rules"))?;
        std::fs::write(
            home.path().join("rules/default.rules"),
            "prefix_rule(pattern=[\"unshare\"], decision=\"allow\")\n",
        )?;
    }
    let mut steps = Vec::new();
    if setup.untrusted_first {
        steps.push(vec![
            ev_response_created("call-first"),
            ev_function_call(
                "call-first",
                "shell_command",
                &json!({ "command": "echo first" }).to_string(),
            ),
            ev_completed("call-first"),
        ]);
    }
    let ids: Vec<String> = (0..calls.len()).map(|n| format!("call-d27-{n}")).collect();
    for (id, (kind, _)) in ids.iter().zip(calls) {
        let command = if setup.saved_rule { PLAIN } else { SCRIPT };
        steps.push(step(id, command, *kind));
    }
    steps.push(vec![
        ev_response_created("done"),
        ev_assistant_message("msg-done", "done"),
        ev_completed("done"),
    ]);
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(&server, steps.into_iter().map(sse).collect()).await;
    let test = test_codex()
        .with_home(home)
        .with_model("test-gpt-5.1-codex")
        .with_config(move |config| {
            config.security_level = setup.level;
            config.permissions.approval_policy = Constrained::allow_any(setup.approval);
            for feature in [Feature::SourceEnvelopes, Feature::UnifiedExec] {
                config
                    .features
                    .enable(feature)
                    .expect("test config should allow feature update");
            }
            let profile = if setup.full_access {
                PermissionProfile::Disabled
            } else {
                PermissionProfile::workspace_write_with(
                    &[],
                    NetworkSandboxPolicy::Restricted,
                    /*exclude_tmpdir_env_var*/ true,
                    /*exclude_slash_tmp*/ true,
                )
            };
            config
                .permissions
                .set_permission_profile(profile)
                .expect("permission profile");
            if setup.aggressive_launch {
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
    let mut asked: Vec<Vec<String>> = vec![Vec::new(); calls.len()];
    let mut warnings = Vec::new();
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
                let n = ids
                    .iter()
                    .position(|id| *id == request.call_id)
                    .unwrap_or_else(|| panic!("prompt for an unknown call: {request:?}"));
                asked[n].push(request.reason.clone().unwrap_or_default());
                test.codex
                    .submit(Op::ExecApproval {
                        id: request.effective_approval_id(),
                        turn_id: None,
                        decision: calls[n].1.clone(),
                    })
                    .await?;
            }
            _ => break,
        }
    }
    let requests = captured.requests();
    let calls = ids
        .iter()
        .zip(asked)
        .map(|(id, asked)| {
            let output = requests
                .iter()
                .find_map(|request| request.function_call_output_text(id))
                .unwrap_or_default();
            Call {
                asked,
                made: output.contains(NAMESPACE_MADE),
                refused: output.contains(REFUSED),
                marked: output.contains("marker=1"),
                unmarked: output.contains("marker=unset"),
                output,
            }
        })
        .collect();
    Ok((calls, warnings))
}

fn warned(call: &Call) -> bool {
    call.asked.iter().any(|reason| {
        reason.contains(WARNING)
            && reason.contains(PAST_THE_PROTECTIONS)
            && reason
                .find(JUSTIFICATION)
                .is_some_and(|at| reason.find(WARNING).is_some_and(|warning| at < warning))
    })
}

/// Approved when asked, through both shell tools: that run may create a
/// user namespace (no deny marker; the namespace is made where the host
/// allows a nested one), after the warning, behind the model's
/// justification. The next request is asked afresh, also after "for the
/// session" (declined here, so it doesn't run; it comes after the first
/// command's output, untrusted content, so it couldn't be allowed anyway),
/// and a command that stays in the sandbox keeps the rule.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d27_moderate_one_approved_command_may_create_user_namespaces() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (kind, first) in [
        (Kind::Escalated, ReviewDecision::Approved),
        (Kind::Escalated, ReviewDecision::ApprovedForSession),
        (Kind::EscalatedExec, ReviewDecision::Approved),
    ] {
        // Boxed: the session future is large for the test thread's stack.
        let (calls, warnings) = Box::pin(run(
            MODERATE,
            &[
                (kind, first.clone()),
                (kind, ReviewDecision::denied("no")),
                (Kind::Sandboxed, ReviewDecision::Approved),
            ],
        ))
        .await?;
        let allowed = &calls[0];
        eprintln!("{kind:?} {first:?}: namespace made: {}", allowed.made);
        let ok = warned(allowed)
            && allowed.asked.len() == 1
            && allowed.unmarked
            && (allowed.made || allowed.refused)
            // Asked again, and declined: nothing ran.
            && calls[1].asked.len() == 1
            && !calls[1].made
            && !calls[1].refused
            // In the sandbox: refused, not asked.
            && calls[2].asked.is_empty()
            && calls[2].refused
            && calls[2].marked
            && !calls[2].made
            // Only that refusal is put down to the rule.
            && warnings
                .iter()
                .filter(|warning| warning.contains("blocks creating user namespaces"))
                .count()
                == 1;
        if !ok {
            wrong.push(format!(
                "{kind:?} {first:?}: {calls:#?}\nwarnings: {warnings:#?}"
            ));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// No warned, fresh approval, no namespace: a saved "don't ask again" rule
/// skips the prompt and keeps Moderate's rules (#840); Aggressive, an
/// Aggressive launch, and Moderate after untrusted content keep the #826
/// rule however the person answers, and their prompts carry no warning.
/// With the sandbox turned off in the person's settings no warning is shown,
/// so the run, which Moderate's rules put in a sandbox, keeps the rule.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d27_no_fresh_moderate_approval_keeps_the_rule() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (name, setup) in [
        (
            "saved rule",
            Setup {
                saved_rule: true,
                ..MODERATE
            },
        ),
        (
            "Aggressive",
            Setup {
                level: SecurityLevel::Aggressive,
                ..MODERATE
            },
        ),
        (
            "Aggressive launch",
            Setup {
                aggressive_launch: true,
                ..MODERATE
            },
        ),
        (
            "untrusted content",
            Setup {
                untrusted_first: true,
                ..MODERATE
            },
        ),
        (
            "full access",
            Setup {
                full_access: true,
                ..MODERATE
            },
        ),
    ] {
        let (calls, _) =
            Box::pin(run(setup, &[(Kind::Escalated, ReviewDecision::Approved)])).await?;
        let call = &calls[0];
        eprintln!("{name}: {call:#?}");
        let unwarned = !call.asked.iter().any(|reason| reason.contains(WARNING));
        let ok = if setup.full_access {
            // Asked, without the warning; Moderate's rules put the run in a
            // sandbox, which keeps the rule.
            !call.asked.is_empty() && unwarned && call.marked && call.refused && !call.made
        } else {
            unwarned
                && call.refused
                && (call.marked || setup.saved_rule)
                && !call.made
                && (!setup.saved_rule || call.asked.is_empty())
        };
        if !ok {
            wrong.push(format!("{name}: {call:#?}"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
