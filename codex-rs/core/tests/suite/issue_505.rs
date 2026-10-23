//! Issue #505: confirming Aggressive in `/security` requires approval for
//! every command that is not known read-only, decided by a person, from each
//! thread's next turn, before the restart that loads the Aggressive rules.
//! It changes each thread's stored settings, so steering and child-agent
//! messages during a later turn are not held back as "permissions changed",
//! and a managed requirement is never overridden.
#![cfg(unix)]

use codex_config::ConfigLayerStack;
use codex_config::ConstrainedWithSource;
use codex_config::RequirementSource;
use codex_core::CodexThread;
use codex_core::StartThreadOptions;
use codex_core::config::Config;
use codex_core::config::Constrained;
use codex_core::config::CurrentTimeReminderConfig;
use codex_core::security_level_change::LevelBasis;
use codex_core::security_level_change::LevelChangeKind;
use codex_core::security_level_change::LevelChangeReport;
use codex_core::security_level_change::Probes;
use codex_core::security_level_change::commit_human_level_change;
use codex_extension_items::ExtensionItem;
use codex_features::Feature;
use codex_protocol::AgentPath;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::items::TurnItem;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::RolloutLine;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_function_call_with_namespace;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::streaming_sse::StreamingSseChunk;
use core_test_support::streaming_sse::start_streaming_sse_server;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

/// A Permissive session with approvals off and the automatic reviewer.
fn permissive_without_approvals(config: &mut Config) {
    config.security_level = SecurityLevel::Permissive;
    config.permissions.approval_policy = Constrained::allow_any(AskForApproval::Never);
    config.approvals_reviewer = ApprovalsReviewer::AutoReview;
    config
        .permissions
        .set_permission_profile(PermissionProfile::workspace_write())
        .expect("workspace write");
    config
        .features
        .enable(Feature::SourceEnvelopes)
        .expect("enable source envelopes");
}

/// A sleep tool, so a turn waits until new input interrupts it.
fn with_sleep_tool(config: &mut Config) {
    config
        .features
        .enable(Feature::CurrentTimeReminder)
        .expect("enable current-time reminders");
    config.current_time_reminder = Some(CurrentTimeReminderConfig {
        sleep_tool: true,
        ..CurrentTimeReminderConfig::default()
    });
}

/// A person confirms Aggressive in this session's `/security`, as the TUI
/// does: off the async runtime.
async fn confirm_aggressive(test: &TestCodex) -> anyhow::Result<LevelChangeReport> {
    let home = test.codex_home_path().to_path_buf();
    let thread = Some(test.session_configured.thread_id);
    let report = tokio::task::spawn_blocking(move || {
        let basis = LevelBasis::read(&home, SecurityLevel::Permissive, thread);
        assert!(basis.live, "the session's policy tree runs in this process");
        commit_human_level_change(
            &home,
            SecurityLevel::Permissive,
            thread,
            &basis,
            SecurityLevel::Aggressive,
            Probes::Passed,
            /*now_unix_seconds*/ 1_000,
        )
    })
    .await??;
    assert_eq!(report.kind, LevelChangeKind::Stricter);
    assert!(report.live, "Aggressive applies in this session now");
    Ok(report)
}

/// #978: a person confirms Aggressive where the confirming session's policy
/// does not run in this process (no thread yet): Core's
/// stored record is committed, and the commit raises this process's sessions.
async fn confirm_aggressive_without_a_live_tree(
    test: &TestCodex,
) -> anyhow::Result<LevelChangeReport> {
    let home = test.codex_home_path().to_path_buf();
    let report = tokio::task::spawn_blocking(move || {
        let basis = LevelBasis::read(&home, SecurityLevel::Permissive, /*thread*/ None);
        assert!(!basis.live, "no live tree without a thread");
        commit_human_level_change(
            &home,
            SecurityLevel::Permissive,
            /*thread*/ None,
            &basis,
            SecurityLevel::Aggressive,
            Probes::Passed,
            /*now_unix_seconds*/ 1_000,
        )
    })
    .await??;
    assert_eq!(report.kind, LevelChangeKind::Stricter);
    assert!(!report.live);
    assert!(report.other_sessions, "the running session rose");
    Ok(report)
}

async fn submit_text(codex: &CodexThread, text: &str) -> anyhow::Result<()> {
    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: text.into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    Ok(())
}

async fn assert_held(codex: &CodexThread) {
    let snapshot = codex.config_snapshot().await;
    assert_eq!(
        (snapshot.approval_policy, snapshot.approvals_reviewer),
        (AskForApproval::UnlessTrusted, ApprovalsReviewer::User),
        "the thread's stored settings require a person's approval"
    );
}

/// What happened to a command that writes a file; with `confirm`, the person
/// confirms Aggressive before the turn.
struct Run {
    /// Whether a person was asked (they decline).
    asked: bool,
    /// Whether the command ran.
    wrote: bool,
}

/// Where Aggressive is confirmed before the turn, if at all.
enum Confirm {
    No,
    /// In this session's `/security`.
    InThisSession,
    /// #978: where no live tree runs (no thread yet).
    WithoutALiveTree,
}

async fn run(confirm: Confirm) -> anyhow::Result<Run> {
    let server = start_mock_server().await;
    let command = "printf x > issue505.txt";
    mount_sse_sequence(
        &server,
        vec![
            sse(vec![
                ev_response_created("call-0"),
                ev_function_call(
                    "call-0",
                    "shell_command",
                    &json!({ "command": command }).to_string(),
                ),
                ev_completed("call-0"),
            ]),
            sse(vec![
                ev_response_created("done"),
                ev_assistant_message("msg-done", "done"),
                ev_completed("done"),
            ]),
        ],
    )
    .await;
    // A local environment: `shell_command` runs commands on the Codex host
    // only and refuses a remote one (#721), so under a remote test
    // environment the write would be refused regardless and neither run's
    // write check would test anything.
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(permissive_without_approvals)
        .build(&server)
        .await?;
    let report = match confirm {
        Confirm::No => None,
        Confirm::InThisSession => Some(confirm_aggressive(&test).await?),
        Confirm::WithoutALiveTree => Some(confirm_aggressive_without_a_live_tree(&test).await?),
    };
    if let Some(report) = report {
        assert_eq!(
            report
                .approvals
                .map(|approvals| (approvals.held, approvals.kept)),
            Some((1, Vec::new()))
        );
        assert_held(&test.codex).await;
    }
    submit_text(&test.codex, "write the file").await?;
    let mut asked = false;
    while let EventMsg::ExecApprovalRequest(approval) = wait_for_event(&test.codex, |event| {
        matches!(
            event,
            EventMsg::ExecApprovalRequest(_) | EventMsg::TurnComplete(_)
        )
    })
    .await
    {
        asked = true;
        test.codex
            .submit(Op::ExecApproval {
                id: approval.effective_approval_id(),
                turn_id: None,
                decision: ReviewDecision::denied("not now"),
            })
            .await?;
    }
    Ok(Run {
        asked,
        wrote: test.cwd_path().join("issue505.txt").exists(),
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_505_confirming_aggressive_requires_a_persons_approval_at_once() -> anyhow::Result<()>
{
    skip_if_no_network!(Ok(()));
    let confirmed = run(Confirm::InThisSession).await?;
    assert!(confirmed.asked, "a person is asked before the command runs");
    assert!(!confirmed.wrote, "declined: the command did not run");
    Ok(())
}

/// #978: the same when the person confirmed where no live tree runs (no
/// thread yet): the commit raises this process's running session, which then
/// takes approval for every command.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_978_aggressive_confirmed_without_a_live_tree_requires_approval_here()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let confirmed = run(Confirm::WithoutALiveTree).await?;
    assert!(confirmed.asked, "a person is asked before the command runs");
    assert!(!confirmed.wrote, "declined: the command did not run");
    Ok(())
}

/// Control: Permissive with approvals off, unchanged.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_505_permissive_sessions_keep_their_approval_policy() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let permissive = run(Confirm::No).await?;
    assert!(!permissive.asked);
    assert!(permissive.wrote);
    Ok(())
}

fn chunks(events: Vec<Value>) -> Vec<StreamingSseChunk> {
    events
        .into_iter()
        .map(|event| StreamingSseChunk {
            gate: None,
            body: responses::sse(vec![event]),
        })
        .collect()
}

fn sleeps(call_id: &str) -> Vec<StreamingSseChunk> {
    chunks(vec![
        ev_response_created("resp-sleep"),
        ev_function_call_with_namespace(
            call_id,
            "clock",
            "sleep",
            &json!({ "duration_ms": 3_600_000 }).to_string(),
        ),
        ev_completed("resp-sleep"),
    ])
}

fn sleep_item(event: &EventMsg, started: bool) -> bool {
    match event {
        EventMsg::ItemStarted(item) if started => {
            matches!(&item.item, TurnItem::Extension(ExtensionItem::Sleep(_)))
        }
        EventMsg::ItemCompleted(item) if !started => {
            matches!(&item.item, TurnItem::Extension(ExtensionItem::Sleep(_)))
        }
        _ => false,
    }
}

/// How new input reaches a turn that started after the confirmation.
enum NewInput {
    /// The person steers the turn.
    Steer,
    /// A child agent's message arrives.
    ChildMessage,
}

/// A turn that started after Aggressive was confirmed sleeps; the new input
/// must wake it within the same turn, not wait for a later one.
async fn new_input_reaches_a_later_turn(input: NewInput) -> anyhow::Result<()> {
    const PROMPT: &str = "sleep until something happens";
    const NEW_INPUT: &str = "issue 505 new input";
    let (server, _completions) = start_streaming_sse_server(vec![
        sleeps("sleep-1"),
        chunks(vec![
            ev_response_created("resp-done"),
            ev_completed("resp-done"),
        ]),
    ])
    .await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(|config| {
            permissive_without_approvals(config);
            with_sleep_tool(config);
        })
        .build_with_streaming_server(&server)
        .await?;
    confirm_aggressive(&test).await?;
    let codex = &test.codex;

    submit_text(codex, PROMPT).await?;
    wait_for_event(codex, |event| sleep_item(event, /*started*/ true)).await;
    match input {
        NewInput::Steer => {
            let steered = codex
                .steer_input(
                    vec![UserInput::Text {
                        text: NEW_INPUT.into(),
                        text_elements: Vec::new(),
                    }],
                    /*additional_context*/ Default::default(),
                    /*expected_turn_id*/ None,
                    /*client_user_message_id*/ None,
                    /*responsesapi_client_metadata*/ None,
                )
                .await;
            assert!(
                steered.is_ok(),
                "steering is accepted in a turn started after the confirmation: {steered:?}"
            );
        }
        NewInput::ChildMessage => {
            codex
                .submit(Op::InterAgentCommunication {
                    communication: InterAgentCommunication::new(
                        AgentPath::try_from("/root/worker").expect("worker path"),
                        AgentPath::root(),
                        Vec::new(),
                        NEW_INPUT.into(),
                        /*trigger_turn*/ false,
                    ),
                })
                .await?;
        }
    }
    // The sleep ends because the input woke it, in the same turn.
    wait_for_event(codex, |event| sleep_item(event, /*started*/ false)).await;
    wait_for_event(codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    let requests = server.requests().await;
    assert_eq!(requests.len(), 2, "one turn, two model requests");
    let second = String::from_utf8(requests[1].clone())?;
    assert!(
        second.contains(NEW_INPUT),
        "the input reached the model in that turn"
    );
    server.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_505_steering_reaches_a_turn_started_after_the_confirmation() -> anyhow::Result<()> {
    new_input_reaches_a_later_turn(NewInput::Steer).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_505_child_agent_messages_reach_a_turn_started_after_the_confirmation()
-> anyhow::Result<()> {
    new_input_reaches_a_later_turn(NewInput::ChildMessage).await
}

/// Child agents already running and threads started later in this process
/// (`/new`, new panes) require approval too.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_505_running_children_and_later_threads_are_covered() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(permissive_without_approvals)
        .build_with_auto_env(&server)
        .await?;
    let mut child = StartThreadOptions::new(test.config.clone());
    child.session_source = Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: test.session_configured.thread_id,
        depth: 1,
        agent_path: Some(AgentPath::try_from("/root/worker").expect("worker path")),
        agent_nickname: Some("worker".to_string()),
        agent_role: Some("worker".to_string()),
        agent_class: None,
    }));
    let child = test.thread_manager.start_thread(child).await?;

    let report = confirm_aggressive(&test).await?;
    assert_eq!(
        report
            .approvals
            .map(|approvals| (approvals.held, approvals.kept)),
        Some((2, Vec::new()))
    );
    assert_held(&child.thread).await;

    let later = test
        .thread_manager
        .start_thread(test.config.clone())
        .await?;
    assert_held(&later.thread).await;
    assert_eq!(
        (
            later.session_configured.approval_policy,
            later.session_configured.approvals_reviewer
        ),
        (AskForApproval::UnlessTrusted, ApprovalsReviewer::User),
        "clients are told at start"
    );
    // A client replaying its earlier values cannot lower it.
    later
        .thread
        .submit(Op::ThreadSettings {
            thread_settings: codex_protocol::protocol::ThreadSettingsOverrides {
                approval_policy: Some(AskForApproval::Never),
                approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
                ..Default::default()
            },
        })
        .await?;
    wait_for_event(&later.thread, |event| {
        matches!(event, EventMsg::ThreadSettingsApplied(_))
    })
    .await;
    assert_held(&later.thread).await;
    Ok(())
}

/// A managed requirement that allows only the automatic reviewer is never
/// overridden: the thread keeps its own approval settings and is told why.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_505_a_managed_reviewer_requirement_is_not_overridden() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    mount_sse_sequence(
        &server,
        vec![sse(vec![
            ev_response_created("done"),
            ev_assistant_message("msg-done", "done"),
            ev_completed("done"),
        ])],
    )
    .await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            permissive_without_approvals(config);
            config.permissions.approval_policy = Constrained::allow_any(AskForApproval::OnRequest);
            let layers = config
                .config_layer_stack
                .get_layers(
                    codex_config::ConfigLayerStackOrdering::LowestPrecedenceFirst,
                    /*include_disabled*/ true,
                )
                .into_iter()
                .cloned()
                .collect();
            let mut requirements = config.config_layer_stack.requirements().clone();
            requirements.approvals_reviewer = ConstrainedWithSource::new(
                Constrained::allow_only(ApprovalsReviewer::AutoReview),
                Some(RequirementSource::LegacyManagedConfigTomlFromMdm),
            );
            let requirements_toml = config.config_layer_stack.requirements_toml().clone();
            config.config_layer_stack =
                ConfigLayerStack::new(layers, requirements, requirements_toml)
                    .expect("managed reviewer requirement");
        })
        .build_with_auto_env(&server)
        .await?;

    let report = confirm_aggressive(&test).await?;
    let approvals = report.approvals.expect("confirmed in this process");
    assert_eq!(approvals.held, 0);
    assert_eq!(approvals.kept.len(), 1, "{approvals:?}");
    let snapshot = test.codex.config_snapshot().await;
    assert_eq!(
        (snapshot.approval_policy, snapshot.approvals_reviewer),
        (AskForApproval::OnRequest, ApprovalsReviewer::AutoReview),
        "neither setting changed"
    );
    let warning = wait_for_event(&test.codex, |event| matches!(event, EventMsg::Warning(_))).await;
    let EventMsg::Warning(warning) = warning else {
        unreachable!("waited for a warning");
    };
    assert!(
        warning
            .message
            .contains("approval for every command was not applied"),
        "{}",
        warning.message
    );

    // The next turn runs with the managed reviewer, not a person.
    submit_text(&test.codex, "hello").await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    test.codex.flush_rollout().await?;
    let rollout = std::fs::read_to_string(test.codex.rollout_path().expect("rollout path"))?;
    let turns: Vec<_> = rollout
        .lines()
        .filter_map(|line| serde_json::from_str::<RolloutLine>(line).ok())
        .filter_map(|line| match line.item {
            RolloutItem::TurnContext(turn) => Some((turn.approval_policy, turn.approvals_reviewer)),
            _ => None,
        })
        .collect();
    assert_eq!(
        turns.last(),
        Some(&(
            AskForApproval::OnRequest,
            Some(ApprovalsReviewer::AutoReview)
        ))
    );
    Ok(())
}
