//! A busy or read-only state DB doesn't end a turn through the
//! provider-request throttle (#351).
use anyhow::Result;
use codex_core::config::AccountingMode;
use codex_features::Feature;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ResponseMock;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::TestCodexBuilder;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use std::time::Duration;
use wiremock::MockServer;

const NOTE: &str = "The shared state database";

/// Developer accounting is off: its own busy and read-only handling (#299)
/// would add its wait to every request and is tested on its own.
fn builder() -> TestCodexBuilder {
    test_codex().with_config(|config| {
        config
            .features
            .enable(Feature::Sqlite)
            .expect("test config should allow feature update");
        config.accounting = AccountingMode::Disabled;
    })
}

/// A turn of three model requests: two tool calls, then the answer.
fn three_requests() -> Vec<String> {
    let plan = r#"{"plan":[{"step":"check","status":"in_progress"}]}"#;
    vec![
        sse(vec![
            ev_response_created("resp-1"),
            ev_function_call("call-1", "update_plan", plan),
            ev_completed("resp-1"),
        ]),
        sse(vec![
            ev_response_created("resp-2"),
            ev_function_call("call-2", "update_plan", plan),
            ev_completed("resp-2"),
        ]),
        sse(vec![
            ev_response_created("resp-3"),
            ev_assistant_message("msg-1", "done"),
            ev_completed("resp-3"),
        ]),
    ]
}

/// Run one turn and return its events through `TurnComplete` (or an abort).
async fn turn(test: &TestCodex) -> Result<Vec<EventMsg>> {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "state DB fixture".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    tokio::time::timeout(Duration::from_secs(120), async {
        let mut events = Vec::new();
        loop {
            let event = test.codex.next_event().await?.msg;
            let finished = matches!(event, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_));
            events.push(event);
            if finished {
                return anyhow::Ok(events);
            }
        }
    })
    .await?
}

/// The turn finished, sent every request, and said once why the DB was skipped.
fn assert_completed_with_one_note(
    events: &[EventMsg],
    mock: &ResponseMock,
    requests: usize,
    cause: &str,
) {
    assert!(
        matches!(events.last(), Some(EventMsg::TurnComplete(_))),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_))),
        "{events:?}"
    );
    let notes: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            EventMsg::Warning(warning) if warning.message.starts_with(NOTE) => {
                Some(warning.message.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(notes.len(), 1, "{events:?}");
    assert!(notes[0].contains(cause), "{}", notes[0]);
    assert_eq!(mock.requests().len(), requests);
}

/// Another process holds the state DB's write lock for the whole turn, past
/// the throttle's wait. This takes about a minute: every thread-metadata write
/// on the way waits out SQLite's 5 s busy timeout before it is logged and
/// skipped, as it did before #351.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_turn_completes_while_another_process_holds_the_state_db() -> Result<()> {
    let server: MockServer = start_mock_server().await;
    let mock = mount_sse_sequence(&server, three_requests()).await;
    let test = builder().build(&server).await?;
    let db = test.codex.state_db().expect("state db enabled");

    let holder = db
        .sqlite()
        .open_read_write_pool(&db.sqlite().state_db_path())
        .await?;
    let held = holder.begin_with("BEGIN IMMEDIATE").await?;
    let events = turn(&test).await?;
    held.rollback().await?;

    assert_completed_with_one_note(&events, &mock, /*requests*/ 3, "busy");
    Ok(())
}

/// The state DB file is read-only when the session starts.
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_turn_completes_on_a_read_only_state_db() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let server = start_mock_server().await;
    let mut responses = vec![sse(vec![
        ev_response_created("resp-0"),
        ev_assistant_message("msg-0", "ready"),
        ev_completed("resp-0"),
    ])];
    responses.extend(three_requests());
    let mock = mount_sse_sequence(&server, responses).await;
    let mut builder = builder();
    let test = builder.build(&server).await?;
    let first = turn(&test).await?;
    assert!(
        matches!(first.last(), Some(EventMsg::TurnComplete(_))),
        "{first:?}"
    );
    let home = test.home.clone();
    let rollout = test.codex.rollout_path().expect("rollout path");
    let db = test.codex.state_db().expect("state db enabled");
    let db_path = db.sqlite().state_db_path();
    test.thread_manager
        .shutdown_all_threads_bounded(Duration::from_secs(3))
        .await;
    drop(test);
    db.close().await;
    drop(db);

    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o444))?;
    let resumed = builder.resume(&server, home, rollout).await?;
    let events = turn(&resumed).await;
    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o644))?;

    assert_completed_with_one_note(&events?, &mock, /*requests*/ 4, "read-only");
    Ok(())
}
