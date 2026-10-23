//! D48 (c) (Travis, 2026-10-20, security sign-off, #891): after untrusted
//! content under Moderate on Linux, the one-run user-namespace allowance
//! (D27 (a)) is offered only for a request to run outside the sandbox whose
//! program is a known tool (unshare, bwrap, podman, buildah, flatpak, nix),
//! through a fresh prompt with the same warning, every time. Any other
//! command, and wrapper, path, alias or shell tricks around a known tool,
//! are not offered and keep the #826 rule; so do a saved "don't ask again"
//! rule, a command that stays in the sandbox and Aggressive.
#![cfg(target_os = "linux")]

use std::sync::Arc;

use codex_core::config::Constrained;
use codex_features::Feature;
use codex_protocol::config_types::ApprovalsReviewer;
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

/// Prints [`NAMESPACE_MADE`] only inside a namespace it made: the command's
/// own text doesn't contain it (a refusal can quote the command). Its inline
/// script makes it a protected action after untrusted content (PF-30-S03),
/// so it is asked about in that fresh prompt.
const UNSHARE: &str = "unshare -Ur sh -c 'echo NAMESPACE-MADE-D$((40+8))'";
/// The same as a plain command, which isn't a protected action: asked about
/// in D48's own fresh prompt.
const PLAIN: &str = "unshare -Ur printf NAMESPACE-MADE-%s D48";
const NAMESPACE_MADE: &str = "NAMESPACE-MADE-D48";
/// What `unshare` prints when a namespace is refused.
const REFUSED: &str = "unshare failed: Operation not permitted";
/// The #826 explanation added where the rule refused it.
const EXPLAINED: &str = "blocks creating user namespaces";
const JUSTIFICATION: &str = "rootless podman needs a user namespace";
/// The start of the prompt's warning (D27's, unchanged).
const WARNING: &str = "Warning (Linux): approving also lets this one run create user namespaces";
const PAST_THE_PROTECTIONS: &str = "this command can get past the file protections";
/// Before untrusted content, D27's prompt says this; after it, approving
/// lifts nothing but the user-namespace rule.
const LIFTS_FILE_RULES: &str = "lifts Moderate's protected-file rules";
/// The fresh prompt for a protected action after untrusted content.
const PROTECTED_ACTION: &str = "after untrusted content entered the session";

/// How a call runs its command.
#[derive(Clone, Copy, Debug)]
enum Kind {
    /// `shell_command` asking to run outside the sandbox, with a
    /// justification.
    Escalated,
    /// The same through `exec_command`.
    EscalatedExec,
    /// The same through `exec_command` with a shell the model chose: a
    /// copy of bash outside the system folders.
    EscalatedExecOwnShell,
    /// `shell_command` in the sandbox.
    Sandboxed,
}

fn step(call_id: &str, command: &str, kind: Kind, own_shell: &str) -> Vec<Value> {
    let (tool, mut arguments) = match kind {
        Kind::Escalated | Kind::Sandboxed => ("shell_command", json!({ "command": command })),
        Kind::EscalatedExec => (
            "exec_command",
            json!({ "cmd": command, "yield_time_ms": 5_000 }),
        ),
        Kind::EscalatedExecOwnShell => (
            "exec_command",
            json!({ "cmd": command, "shell": own_shell, "yield_time_ms": 5_000 }),
        ),
    };
    if !matches!(kind, Kind::Sandboxed) {
        arguments["sandbox_permissions"] = json!("require_escalated");
        arguments["justification"] = json!(JUSTIFICATION);
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
    /// A "don't ask again" rule saved earlier for `unshare`.
    saved_rule: bool,
    approval: AskForApproval,
    /// The automatic reviewer answers ordinary prompts.
    auto_review: bool,
}

#[derive(Debug)]
struct Call {
    command: &'static str,
    /// Prompts shown for it, with their reasons.
    asked: Vec<String>,
    made: bool,
    refused: bool,
    explained: bool,
    output: String,
}

impl Call {
    /// Asked with the warning, without D27's before-untrusted wording: in
    /// D48's own prompt behind the model's justification, or in the fresh
    /// prompt for a protected action after untrusted content behind its
    /// reason.
    fn warned(&self) -> bool {
        self.asked.iter().any(|reason| {
            let before_the_warning = |text: &str| {
                reason
                    .find(text)
                    .is_some_and(|at| reason.find(WARNING).is_some_and(|warning| at < warning))
            };
            reason.contains(PAST_THE_PROTECTIONS)
                && !reason.contains(LIFTS_FILE_RULES)
                && (before_the_warning(JUSTIFICATION) || before_the_warning(PROTECTED_ACTION))
        })
    }

    fn unwarned(&self) -> bool {
        !self.asked.iter().any(|reason| reason.contains(WARNING))
    }

    /// Allowed: no rule explanation, and the namespace made where the host
    /// allows a nested one (else refused by the host, not the rule).
    fn allowed(&self) -> bool {
        !self.explained && (self.made || self.refused)
    }

    /// The #826 rule refused it (and explained so).
    fn kept_the_rule(&self) -> bool {
        self.explained && self.refused && !self.made
    }
}

/// Runs, after a first command whose output is untrusted content, one call
/// per entry of `calls`, answering every prompt of call `n` with
/// `calls[n].2`. A link `./unshare` to the real one is in the working folder.
async fn run(
    setup: Setup,
    calls: &[(Kind, &'static str, ReviewDecision)],
) -> anyhow::Result<Vec<Call>> {
    let home = Arc::new(TempDir::new()?);
    if setup.saved_rule {
        std::fs::create_dir_all(home.path().join("rules"))?;
        std::fs::write(
            home.path().join("rules/default.rules"),
            "prefix_rule(pattern=[\"unshare\"], decision=\"allow\")\n",
        )?;
    }
    let mut steps = vec![vec![
        ev_response_created("call-first"),
        ev_function_call(
            "call-first",
            "shell_command",
            &json!({ "command": "echo first" }).to_string(),
        ),
        ev_completed("call-first"),
    ]];
    // A shell the model may name for `exec_command`, outside the system
    // folders.
    let own_shell_dir = TempDir::new()?;
    let own_shell = own_shell_dir.path().join("bash");
    std::os::unix::fs::symlink("/bin/bash", &own_shell)?;
    let own_shell = own_shell.to_string_lossy().into_owned();
    let ids: Vec<String> = (0..calls.len()).map(|n| format!("call-d48-{n}")).collect();
    for (id, (kind, command, _)) in ids.iter().zip(calls) {
        steps.push(step(id, command, *kind, &own_shell));
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
            if setup.auto_review {
                config.approvals_reviewer = ApprovalsReviewer::AutoReview;
            }
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
                .expect("permission profile");
        })
        // Host-sandbox security test: local executor on purpose.
        .build(&server)
        .await?;
    // The "copy" of a known tool in a folder the agent can write.
    std::os::unix::fs::symlink("/usr/bin/unshare", test.cwd_path().join("unshare"))?;
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
    while let EventMsg::ExecApprovalRequest(request) = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    {
        let decision = match ids.iter().position(|id| *id == request.call_id) {
            Some(n) => {
                asked[n].push(request.reason.clone().unwrap_or_default());
                calls[n].2.clone()
            }
            None => ReviewDecision::Approved,
        };
        test.codex
            .submit(Op::ExecApproval {
                id: request.effective_approval_id(),
                turn_id: None,
                decision,
            })
            .await?;
    }
    let requests = captured.requests();
    Ok(ids
        .iter()
        .zip(asked)
        .zip(calls)
        .map(|((id, asked), (_, command, _))| {
            let output = requests
                .iter()
                .find_map(|request| request.function_call_output_text(id))
                .unwrap_or_default();
            Call {
                command,
                asked,
                made: output.contains(NAMESPACE_MADE),
                refused: output.contains(REFUSED),
                explained: output.contains(EXPLAINED),
                output,
            }
        })
        .collect())
}

const MODERATE: Setup = Setup {
    level: SecurityLevel::Moderate,
    saved_rule: false,
    approval: AskForApproval::OnRequest,
    auto_review: false,
};

/// In a session with untrusted content: a known tool's request to run
/// outside the sandbox is offered with the warning, every time (also after
/// "for the session"), through both shell tools, and the approved run may
/// create user namespaces; podman and bwrap are offered too (declined here). Any other
/// command, and the tricks around a known tool, get no warning and keep
/// the rule, approved or not; so does a known tool in the sandbox.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d48_after_untrusted_content_only_known_tools_are_offered() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    // By name where it is installed (found through PATH), else by its
    // system path.
    let bwrap = if std::path::Path::new("/usr/bin/bwrap").exists() {
        "bwrap --ro-bind / / --unshare-user true"
    } else {
        "/usr/bin/bwrap --ro-bind / / --unshare-user true"
    };
    let offered: &[(Kind, &str, ReviewDecision)] = &[
        (Kind::Escalated, PLAIN, ReviewDecision::ApprovedForSession),
        (Kind::Escalated, PLAIN, ReviewDecision::Approved),
        (Kind::EscalatedExec, PLAIN, ReviewDecision::Approved),
        (Kind::Escalated, UNSHARE, ReviewDecision::Approved),
        (Kind::EscalatedExec, UNSHARE, ReviewDecision::Approved),
        // Podman by its system path (offered where it isn't installed too)
        // and bwrap: declined.
        (
            Kind::Escalated,
            "/usr/bin/podman run --rm alpine true",
            ReviewDecision::denied("no"),
        ),
        (Kind::EscalatedExec, bwrap, ReviewDecision::denied("no")),
    ];
    let not_offered: &[(Kind, &str, ReviewDecision)] = &[
        // An arbitrary command.
        (
            Kind::Escalated,
            "unshare -Ur true && echo NAMESPACE-MADE-D$((40+8))",
            ReviewDecision::Approved,
        ),
        // A copy of the tool where the agent can write.
        (
            Kind::Escalated,
            "./unshare -Ur sh -c 'echo NAMESPACE-MADE-D$((40+8))'",
            ReviewDecision::Approved,
        ),
        // Wrappers with an assignment, an unknown wrapper, a nested shell.
        (
            Kind::Escalated,
            "env PATH=/usr/bin unshare -Ur sh -c 'echo NAMESPACE-MADE-D$((40+8))'",
            ReviewDecision::Approved,
        ),
        (
            Kind::Escalated,
            "timeout 20 unshare -Ur sh -c 'echo NAMESPACE-MADE-D$((40+8))'",
            ReviewDecision::Approved,
        ),
        (
            Kind::Escalated,
            "bash -c \"unshare -Ur sh -c 'echo NAMESPACE-MADE-D\\$((40+8))'\"",
            ReviewDecision::Approved,
        ),
        // An alias, and a second command after the tool.
        (
            Kind::Escalated,
            "shopt -s expand_aliases\nalias podman='unshare -Ur'\npodman sh -c 'echo NAMESPACE-MADE-D$((40+8))'",
            ReviewDecision::Approved,
        ),
        (
            Kind::EscalatedExec,
            "unshare -Ur true; unshare -Ur sh -c 'echo NAMESPACE-MADE-D$((40+8))'",
            ReviewDecision::Approved,
        ),
        // A shell the model chose (the prompt shows only the script).
        (Kind::EscalatedExecOwnShell, PLAIN, ReviewDecision::Approved),
        // A known tool that stays in the sandbox.
        (Kind::Sandboxed, UNSHARE, ReviewDecision::Approved),
    ];
    let calls: Vec<_> = offered.iter().chain(not_offered).cloned().collect();
    // Boxed: the session future is large for the test thread's stack.
    let calls = Box::pin(run(MODERATE, &calls)).await?;
    let mut wrong = Vec::new();
    for (n, call) in calls.iter().enumerate() {
        eprintln!(
            "{n} {}: asked {}, warned {}, made {}, refused {}, explained {}",
            call.command,
            call.asked.len(),
            call.warned(),
            call.made,
            call.refused,
            call.explained
        );
        let ok = match n {
            // Unshare approved in D48's prompt (also "for the session", then
            // asked again) and in a protected action's.
            0..=2 => {
                call.asked.len() == 1
                    && call.warned()
                    && call.allowed()
                    && call.asked[0].contains(JUSTIFICATION)
            }
            3 | 4 => {
                call.asked.len() == 1
                    && call.warned()
                    && call.allowed()
                    && call.asked[0].contains(PROTECTED_ACTION)
            }
            // Podman and bwrap offered, declined: nothing ran.
            5 | 6 => call.asked.len() == 1 && call.warned() && !call.made,
            // Whether the shell expands the alias decides whether it runs.
            _ if call.command.contains("alias") => {
                call.unwarned() && !call.made && (call.explained || !call.refused)
            }
            _ => call.unwarned() && call.kept_the_rule(),
        };
        if !ok {
            wrong.push(format!("{n}: {call:#?}\noutput: {}", call.output));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

/// After untrusted content, for a known tool: a saved "don't ask again"
/// rule (no prompt), Aggressive and approvals off keep the rule, with no
/// warning; with the automatic reviewer configured the person is still asked,
/// with the warning, and the approved run may create user namespaces.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn d48_saved_rule_aggressive_approvals_off_and_auto_review() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    skip_if_sandbox!(Ok(()));
    let mut wrong = Vec::new();
    for (name, setup, command) in [
        (
            "saved rule",
            Setup {
                saved_rule: true,
                ..MODERATE
            },
            PLAIN,
        ),
        (
            "Aggressive",
            Setup {
                level: SecurityLevel::Aggressive,
                ..MODERATE
            },
            PLAIN,
        ),
        (
            "approvals off",
            Setup {
                approval: AskForApproval::Never,
                ..MODERATE
            },
            PLAIN,
        ),
        (
            "automatic reviewer",
            Setup {
                auto_review: true,
                ..MODERATE
            },
            PLAIN,
        ),
    ] {
        let calls = Box::pin(run(
            setup,
            &[(Kind::Escalated, command, ReviewDecision::Approved)],
        ))
        .await?;
        let call = &calls[0];
        eprintln!("{name}: {call:#?}");
        let ok = match name {
            "automatic reviewer" => call.asked.len() == 1 && call.warned() && call.allowed(),
            "approvals off" => call.asked.is_empty() && !call.made,
            _ => {
                call.unwarned()
                    && call.refused
                    && !call.made
                    && (!setup.saved_rule || call.asked.is_empty())
            }
        };
        if !ok {
            wrong.push(format!("{name}: {call:#?}\noutput: {}", call.output));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}
