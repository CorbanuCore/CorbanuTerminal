//! Issue #445 (milestone "Aggressive ships", AGG-22): with the security kill
//! switch on, a model turn is refused before anything is sent, with a message
//! naming the kill switch and how to turn it off, and the refused prompt is
//! not kept for later; turning it off in `/security` lets the next turn
//! through.

use codex_core::security_level_change::LevelBasis;
use codex_core::security_revocation::HumanRevocation;
use codex_core::security_revocation::commit_human_revocation;
use codex_features::Feature;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;

fn switch(test: &TestCodex, choice: HumanRevocation) -> anyhow::Result<()> {
    let home = test.codex_home_path();
    let thread = Some(test.session_configured.thread_id);
    let basis = LevelBasis::read(home, SecurityLevel::Aggressive, thread);
    assert!(basis.live, "the session's policy tree runs in this process");
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

/// Submit `text` and return the error shown for the turn, if any.
async fn turn(test: &TestCodex, text: &str) -> anyhow::Result<Option<String>> {
    test.codex
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
    let mut error = None;
    loop {
        match wait_for_event(&test.codex, |event| {
            matches!(event, EventMsg::Error(_) | EventMsg::TurnComplete(_))
        })
        .await
        {
            EventMsg::Error(event) => error = Some(event.message),
            _ => return Ok(error),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_445_kill_switch_refuses_model_turns_until_turned_off() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![sse(vec![
            ev_response_created("answer"),
            ev_assistant_message("msg-answer", "4"),
            ev_completed("answer"),
        ])],
    )
    .await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.security_level = SecurityLevel::Aggressive;
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
        .build_with_auto_env(&server)
        .await?;

    switch(&test, HumanRevocation::KillSwitchOn)?;
    let refused = turn(&test, "What is 2+2?").await?;
    let message = refused.expect("the turn is refused");
    assert!(message.contains("kill switch is on"), "{message}");
    assert!(message.contains("/security"), "{message}");
    assert!(
        message.contains("Turn the kill switch off"),
        "names how to turn it off: {message}"
    );
    assert_eq!(captured.requests().len(), 0, "nothing reached the model");

    switch(&test, HumanRevocation::KillSwitchOff)?;
    assert_eq!(turn(&test, "What is 3+3?").await?, None);
    let requests = captured.requests();
    assert_eq!(requests.len(), 1, "the turn after it is sent");
    assert!(requests[0].body_contains_text("What is 3+3?"));
    assert!(
        !requests[0].body_contains_text("What is 2+2?"),
        "the refused prompt was not kept for later"
    );
    Ok(())
}

/// Review HIGH 1: an agent message that asks for a turn while the switch is
/// on does not start a refused turn over and over; it waits, and reaches the
/// model with the first turn after the switch is turned off.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue_445_agent_messages_wait_while_the_kill_switch_is_on() -> anyhow::Result<()> {
    use codex_protocol::AgentPath;
    use codex_protocol::protocol::InterAgentCommunication;
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let captured = mount_sse_sequence(
        &server,
        vec![sse(vec![
            ev_response_created("answer"),
            ev_assistant_message("msg-answer", "noted"),
            ev_completed("answer"),
        ])],
    )
    .await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .with_config(|config| {
            config.security_level = SecurityLevel::Aggressive;
            config
                .features
                .enable(Feature::SourceEnvelopes)
                .expect("enable source envelopes");
        })
        .build_with_auto_env(&server)
        .await?;
    switch(&test, HumanRevocation::KillSwitchOn)?;
    test.codex
        .submit(Op::InterAgentCommunication {
            communication: InterAgentCommunication::new(
                AgentPath::try_from("/root/worker").expect("worker path"),
                AgentPath::root(),
                Vec::new(),
                "worker finished: pf445-mail".to_string(),
                /*trigger_turn*/ true,
            ),
        })
        .await?;
    // Long enough for a refusal loop to show; nothing may start or be sent.
    let mut errors = 0;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    while let Ok(Ok(event)) = tokio::time::timeout_at(deadline, test.codex.next_event()).await {
        if matches!(event.msg, EventMsg::Error(_)) {
            errors += 1;
        }
    }
    assert!(errors <= 1, "{errors} refusals: the refused turn looped");
    assert_eq!(captured.requests().len(), 0, "nothing reached the model");

    switch(&test, HumanRevocation::KillSwitchOff)?;
    assert_eq!(turn(&test, "Anything new?").await?, None);
    let requests = captured.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].body_contains_text("pf445-mail"));
    Ok(())
}
