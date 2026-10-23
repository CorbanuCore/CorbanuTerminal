//! Issue #507 (owner decision 2026-10-12): turning the security kill switch
//! off restarts nothing. Each path that could start work on its own while the
//! switch was on (agent messages to a pane, a child agent's mailbox, a durable
//! sleep woken by mail, an automatic idle turn such as a goal continuation)
//! starts no turn when the switch goes off; the waiting work goes with the
//! next turn someone starts.

use std::sync::Arc;
use std::time::Duration;

use codex_core::CodexThread;
use codex_core::StartThreadOptions;
use codex_core::TryStartTurnIfIdleRejectionReason;
use codex_core::security_level_change::LevelBasis;
use codex_core::security_revocation::HumanRevocation;
use codex_core::security_revocation::commit_human_revocation;
use codex_extension_items::sleep::SleepItem;
use codex_features::Feature;
use codex_protocol::AgentPath;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ResponseMock;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::responses::user_message_item;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::TestCodexBuilder;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use wiremock::MockServer;

/// How long a turn that should not start is given to show up.
const QUIET: Duration = Duration::from_secs(2);

fn builder() -> TestCodexBuilder {
    test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.security_level = SecurityLevel::Aggressive;
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
}

async fn answers(server: &MockServer) -> ResponseMock {
    mount_sse_sequence(
        server,
        vec![sse(vec![
            ev_response_created("answer"),
            ev_assistant_message("msg-answer", "noted"),
            ev_completed("answer"),
        ])],
    )
    .await
}

/// The person turns the kill switch on or off in `/security`.
fn switch(test: &TestCodex, choice: HumanRevocation) -> anyhow::Result<()> {
    let home = test.codex_home_path();
    let thread = Some(test.session_configured.thread_id);
    let basis = LevelBasis::read(home, SecurityLevel::Aggressive, thread);
    let report = commit_human_revocation(
        home,
        SecurityLevel::Aggressive,
        thread,
        &basis,
        choice,
        /*now_unix_seconds*/ 1_000,
    )?;
    assert_eq!(
        report.kill_switch_active,
        choice == HumanRevocation::KillSwitchOn
    );
    Ok(())
}

async fn mail(codex: &CodexThread, text: &str, trigger_turn: bool) -> anyhow::Result<()> {
    codex
        .submit(Op::InterAgentCommunication {
            communication: InterAgentCommunication::new(
                AgentPath::try_from("/root/worker").map_err(anyhow::Error::msg)?,
                AgentPath::root(),
                Vec::new(),
                text.to_string(),
                trigger_turn,
            ),
        })
        .await?;
    Ok(())
}

/// No turn starts in `codex` for [`QUIET`].
async fn assert_no_turn_starts(codex: &CodexThread, what: &str) {
    let deadline = tokio::time::Instant::now() + QUIET;
    while let Ok(Ok(event)) = tokio::time::timeout_at(deadline, codex.next_event()).await {
        assert!(
            !matches!(event.msg, EventMsg::TurnStarted(_)),
            "{what}: a turn started on its own"
        );
    }
}

/// A person types `text`; returns once the turn is complete.
async fn person_types(codex: &CodexThread, text: &str) -> anyhow::Result<()> {
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
    wait_for_event(codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    Ok(())
}

/// A Codex pane: an agent message that asks for a turn while the switch is on
/// waits past the switch going off, and goes with the person's next turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_switch_off_does_not_start_a_turn_for_waiting_agent_messages()
-> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let captured = answers(&server).await;
    let test = builder().build_with_auto_env(&server).await?;

    switch(&test, HumanRevocation::KillSwitchOn)?;
    mail(
        &test.codex,
        "worker finished: pf507-pane-mail",
        /*trigger_turn*/ true,
    )
    .await?;
    assert_no_turn_starts(&test.codex, "switch on").await;
    switch(&test, HumanRevocation::KillSwitchOff)?;
    assert_no_turn_starts(&test.codex, "switch turned off").await;
    assert_eq!(captured.requests().len(), 0, "nothing reached the model");

    person_types(&test.codex, "Anything new?").await?;
    let requests = captured.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body_contains_text("pf507-pane-mail"));
    Ok(())
}

/// A child agent: a task its parent sent while the switch was on waits in
/// the child's mailbox; turning the switch off starts no turn in the child.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_switch_off_does_not_start_a_child_agent() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let captured = answers(&server).await;
    let test = builder().build_with_auto_env(&server).await?;
    let mut options = StartThreadOptions::new(test.config.clone());
    options.session_source = Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: test.session_configured.thread_id,
        depth: 1,
        agent_path: Some(AgentPath::try_from("/root/worker").map_err(anyhow::Error::msg)?),
        agent_nickname: Some("worker".to_string()),
        agent_role: Some("worker".to_string()),
        agent_class: None,
    }));
    let child = test.thread_manager.start_thread(options).await?.thread;

    switch(&test, HumanRevocation::KillSwitchOn)?;
    child
        .submit(Op::InterAgentCommunication {
            communication: InterAgentCommunication::new(
                AgentPath::root(),
                AgentPath::try_from("/root/worker").map_err(anyhow::Error::msg)?,
                Vec::new(),
                "parent task: pf507-child-task".to_string(),
                /*trigger_turn*/ true,
            ),
        })
        .await?;
    assert_no_turn_starts(&child, "switch on").await;
    switch(&test, HumanRevocation::KillSwitchOff)?;
    assert_no_turn_starts(&child, "switch turned off").await;
    assert_eq!(captured.requests().len(), 0, "nothing reached the model");

    person_types(&child, "Go on.").await?;
    let requests = captured.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body_contains_text("pf507-child-task"));
    Ok(())
}

struct SleepingThread;

impl codex_extension_api::ThreadLifecycleContributor<codex_core::config::Config>
    for SleepingThread
{
    fn on_thread_start<'a>(
        &'a self,
        input: codex_extension_api::ThreadStartInput<'a, codex_core::config::Config>,
    ) -> codex_extension_api::ExtensionFuture<'a, ()> {
        Box::pin(async move {
            input.thread_store.insert(SleepItem {
                id: "clock-wait-507".to_string(),
                duration_ms: 60_000,
            });
        })
    }
}

/// A durable sleep (a timer the agent set) is woken by any mail; mail that
/// arrives while the switch is on does not wake it, before or after the
/// switch goes off.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_switch_off_does_not_wake_a_durable_sleep() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let captured = answers(&server).await;
    let mut extensions =
        codex_extension_api::ExtensionRegistryBuilder::<codex_core::config::Config>::new();
    extensions.thread_lifecycle_contributor(Arc::new(SleepingThread));
    let test = builder()
        .with_extensions(Arc::new(extensions.build()))
        .build_with_auto_env(&server)
        .await?;

    switch(&test, HumanRevocation::KillSwitchOn)?;
    mail(
        &test.codex,
        "worker update: pf507-sleep-mail",
        /*trigger_turn*/ false,
    )
    .await?;
    assert_no_turn_starts(&test.codex, "switch on").await;
    switch(&test, HumanRevocation::KillSwitchOff)?;
    assert_no_turn_starts(&test.codex, "switch turned off").await;
    assert_eq!(captured.requests().len(), 0, "nothing reached the model");

    person_types(&test.codex, "Anything new?").await?;
    let requests = captured.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body_contains_text("pf507-sleep-mail"));
    Ok(())
}

/// Automatic idle work (a goal continuation) is refused while the switch is
/// on instead of starting a turn; after it is off, nothing starts until it is
/// asked for again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_507_no_automatic_idle_turn_while_the_switch_is_on() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let captured = answers(&server).await;
    let test = builder().build_with_auto_env(&server).await?;

    switch(&test, HumanRevocation::KillSwitchOn)?;
    let refused = test
        .codex
        .try_start_turn_if_idle(vec![user_message_item("continue the goal: pf507-goal")])
        .await
        .expect_err("no automatic turn while the switch is on");
    assert_eq!(
        refused.reason(),
        TryStartTurnIfIdleRejectionReason::KillSwitch
    );
    assert_no_turn_starts(&test.codex, "switch on").await;
    switch(&test, HumanRevocation::KillSwitchOff)?;
    assert_no_turn_starts(&test.codex, "switch turned off").await;
    assert_eq!(captured.requests().len(), 0, "nothing reached the model");

    test.codex
        .try_start_turn_if_idle(vec![user_message_item("continue the goal: pf507-goal")])
        .await
        .map_err(|err| anyhow::anyhow!("{:?}", err.reason()))?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    assert_eq!(captured.requests().len(), 1, "asked for again: it runs");
    Ok(())
}
