//! Mailbox delivery preserves in-flight responses without preempting them.
//! Queued mail reaches the next request without losing completed response items.

use codex_core::TurnInputRequest;
use codex_core::TurnInputSubmission;
use codex_features::Feature;
use codex_protocol::AgentPath;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InterAgentCommunication;
use codex_protocol::protocol::Op;
use core_test_support::context_snapshot;
use core_test_support::context_snapshot::ContextSnapshotOptions;
use core_test_support::context_snapshot::SnapshotEntry;
use core_test_support::responses;
use core_test_support::streaming_sse::StreamingSseChunk;
use core_test_support::streaming_sse::start_streaming_sse_server;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use test_case::test_case;
use tokio::sync::oneshot;

#[derive(Clone, Copy)]
enum Boundary {
    Reasoning,
    Commentary,
    PartialAnswer,
}

#[allow(
    clippy::disallowed_methods,
    reason = "traced_test captures spawned tasks; test_case preserves allow but strips expect"
)]
#[tracing_test::traced_test]
#[test_case(Boundary::Reasoning, false; "reasoning_default")]
#[test_case(Boundary::Commentary, false; "commentary_default")]
#[test_case(Boundary::PartialAnswer, false; "partial_answer_default")]
#[test_case(Boundary::Reasoning, true; "reasoning_deferred")]
#[test_case(Boundary::Commentary, true; "commentary_deferred")]
#[test_case(Boundary::PartialAnswer, true; "partial_answer_deferred")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_preemption_preserves_response_when_deferred(
    boundary: Boundary,
    defer_mailbox_preemption: bool,
) -> anyhow::Result<()> {
    let (release, gate) = oneshot::channel();
    let (added, done, boundary_name) = match boundary {
        Boundary::Reasoning => (
            responses::ev_reasoning_item_added("boundary", &["Preparing the next action"]),
            responses::ev_reasoning_item("boundary", &["Preparing the next action"], &[]),
            "reasoning",
        ),
        Boundary::Commentary | Boundary::PartialAnswer => {
            let phase = match boundary {
                Boundary::Commentary => "commentary",
                Boundary::PartialAnswer => "partial_answer",
                Boundary::Reasoning => unreachable!(),
            };
            let mut done = responses::ev_assistant_message("boundary", "I will update the plan.");
            done["item"]["phase"] = json!(phase);
            let mut added = responses::ev_message_item_added("boundary", "");
            added["item"]["phase"] = json!(phase);
            (added, done, phase)
        }
    };
    let (streaming, _completions) = start_streaming_sse_server(vec![
        vec![
            StreamingSseChunk {
                gate: None,
                body: responses::sse(vec![responses::ev_response_created("first"), added]),
            },
            StreamingSseChunk {
                gate: Some(gate),
                body: responses::sse(vec![
                    done,
                    responses::ev_function_call(
                        "planned-action",
                        "update_plan",
                        r#"{"plan":[{"step":"Reply to the user","status":"in_progress"}]}"#,
                    ),
                    responses::ev_completed("first"),
                ]),
            },
        ],
        vec![StreamingSseChunk {
            gate: None,
            body: responses::sse(vec![
                responses::ev_response_created("follow-up"),
                responses::ev_assistant_message("answer", "The worker update is now included."),
                responses::ev_completed("follow-up"),
            ]),
        }],
    ])
    .await;
    let config_server = responses::start_mock_server().await;
    let base_url = format!("{}/v1", streaming.uri());
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(move |config| {
            config.model_provider.base_url = Some(base_url);
            config.update_plan_enabled = true;
            config
                .features
                .set_enabled(Feature::DeferMailboxPreemption, defer_mailbox_preemption)
                .expect("set mailbox policy");
            // The gated server captures uncompressed request bodies for assertions.
            config
                .features
                .disable(Feature::EnableRequestCompression)
                .expect("disable request compression");
        })
        .build_with_auto_env(&config_server)
        .await?;
    test.codex
        .start_or_steer_turn(TurnInputRequest::user_input(vec![super::text(
            "Plan the next action while the worker finishes.",
        )]))
        .await?;
    wait_for_event(
        &test.codex,
        |event| matches!(event, EventMsg::ItemStarted(item) if item.item.id() == "boundary"),
    )
    .await;
    for message in ["Worker found the result.", "Worker checked the result."] {
        test.codex
            .submit(Op::InterAgentCommunication {
                communication: InterAgentCommunication::new(
                    AgentPath::root().join("worker").expect("worker path"),
                    AgentPath::root(),
                    Vec::new(),
                    message.to_string(),
                    /*trigger_turn*/ false,
                ),
                start_options: Default::default(),
            })
            .await?;
    }
    // The barrier confirms that both messages were queued before finishing the item.
    test.codex
        .submit(Op::RealtimeConversationListVoices)
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::RealtimeConversationListVoicesResponse(_))
    })
    .await;
    release.send(()).expect("release response");
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;

    let logs = String::from_utf8(
        tracing_test::internal::global_buf()
            .lock()
            .expect("test log buffer is not poisoned")
            .clone(),
    )?;
    let thread_id = test.session_configured.thread_id.to_string();
    assert_eq!(
        logs.lines()
            .filter(|line| line.contains("codex.mailbox_preemption") && line.contains(&thread_id))
            .count(),
        0,
    );
    let requests = streaming
        .requests()
        .await
        .iter()
        .map(|body| serde_json::from_slice::<Value>(body))
        .collect::<serde_json::Result<Vec<_>>>()?;
    assert_eq!(requests.len(), 2);
    let input = requests[1]["input"].as_array().expect("follow-up input");
    let output = input
        .iter()
        .find(|item| item["type"] == "function_call_output" && item["call_id"] == "planned-action");
    assert_eq!(
        output.map(|item| &item["output"]),
        Some(&json!("Plan updated")),
    );
    let messages = input
        .iter()
        .filter(|item| item["type"] == "agent_message")
        .map(|item| item["content"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            json!([{"type": "input_text", "text": "Worker found the result."}]),
            json!([{"type": "input_text", "text": "Worker checked the result."}]),
        ],
    );
    let entries = requests.iter().map(SnapshotEntry::body).collect::<Vec<_>>();
    insta::assert_snapshot!(
        format!("mailbox_preemption_{boundary_name}_deferred_{defer_mailbox_preemption}"),
        context_snapshot::format_context_snapshot(
            "Worker messages arrive during a response. The planned tool completes before the next request receives both messages, with either mailbox policy setting.",
            &entries,
            &ContextSnapshotOptions::default().rewrite_known_segments(),
        )
    );
    streaming.shutdown().await;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MailboxCompletion {
    AfterBoundary,
    BufferedCompletion,
    BufferedItems,
    UserSteering,
}

#[test_case(MailboxCompletion::AfterBoundary; "after_boundary")]
#[test_case(MailboxCompletion::BufferedCompletion; "completion_already_buffered")]
#[test_case(MailboxCompletion::BufferedItems; "buffered_items")]
#[test_case(MailboxCompletion::UserSteering; "user_steering_during_response")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_delivery_preserves_websocket_response(
    scenario: MailboxCompletion,
) -> anyhow::Result<()> {
    core_test_support::skip_if_no_network!(Ok(()));

    let boundary = responses::ev_reasoning_item("boundary", &["Preparing the next action"], &[]);
    let mut exchanges = mailbox_initial_responses(&boundary);
    let completed = responses::ev_completed_with_tokens("first", /*total_tokens*/ 42);
    let has_tool = matches!(
        scenario,
        MailboxCompletion::BufferedItems | MailboxCompletion::UserSteering
    );
    if has_tool {
        exchanges[1].extend([
            responses::ev_reasoning_item_added("buffered", &[]),
            responses::ev_reasoning_item("buffered", &["Already completed"], &[]),
            responses::ev_function_call(
                "buffered-tool",
                "update_plan",
                r#"{"plan":[{"step":"Finish the planned action before reading mail","status":"in_progress"}]}"#,
            ),
        ]);
    }
    // Complete the original response without waiting for a client interrupt.
    exchanges[1].push(completed.clone());
    exchanges.push(vec![
        responses::ev_response_created("follow-up"),
        responses::ev_assistant_message("answer", "The worker updates are included."),
        responses::ev_completed("follow-up"),
    ]);
    let server = responses::start_websocket_server(vec![exchanges]).await;
    let release =
        server.gate_response_event(if scenario == MailboxCompletion::BufferedCompletion {
            &completed
        } else {
            &boundary
        });
    let release_completion = (scenario == MailboxCompletion::UserSteering)
        .then(|| server.gate_response_event(&completed));
    let test = start_mailbox_preemption(&server, release).await?;
    if let Some(release_completion) = release_completion {
        wait_for_event(&test.codex, |event| {
            assert!(
                !matches!(event, EventMsg::Error(_) | EventMsg::StreamError(_)),
                "{event:?}"
            );
            matches!(event, EventMsg::ItemCompleted(item) if item.item.id() == "buffered")
        })
        .await;
        let submission = test
            .codex
            .start_or_steer_turn(TurnInputRequest::user_input(vec![super::text(
                "Use both worker updates.",
            )]))
            .await?;
        assert!(matches!(submission, TurnInputSubmission::Steered { .. }));
        release_completion
            .send(())
            .expect("release response completion");
    }
    wait_for_event(&test.codex, |event| {
        assert!(
            !matches!(event, EventMsg::Error(_) | EventMsg::StreamError(_)),
            "{event:?}"
        );
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    test.codex.shutdown_and_wait().await?;

    let requests = server.single_connection();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].body_json()["generate"], false);
    assert!(
        requests
            .iter()
            .all(|request| request.body_json()["type"] == "response.create")
    );
    let follow_up = requests[2].body_json();
    assert_eq!(follow_up["previous_response_id"], "first");
    let input = responses::strip_response_item_ids_from_json(responses::strip_metadata_from_json(
        follow_up["input"].clone(),
    ));
    let input = input.as_array().expect("follow-up input");
    let mail = input
        .iter()
        .filter(|item| item["type"] == "agent_message")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(mail, expected_mailbox_input());
    let outputs = input
        .iter()
        .filter(|item| item["type"] == "function_call_output")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        outputs,
        if has_tool {
            vec![
                json!({"type": "function_call_output", "call_id": "buffered-tool", "output": "Plan updated"}),
            ]
        } else {
            Vec::new()
        }
    );
    if scenario == MailboxCompletion::UserSteering {
        assert_eq!(
            input
                .iter()
                .filter(|item| item["role"] == "user"
                    && item["content"]
                        == json!([{"type": "input_text", "text": "Use both worker updates."}]))
                .count(),
            1,
        );
    }
    server.shutdown().await;
    Ok(())
}

#[test_case(false; "disconnect_after_boundary")]
#[test_case(true; "disconnect_after_buffered_item")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_delivery_recovers_from_disconnect(buffered_item: bool) -> anyhow::Result<()> {
    core_test_support::skip_if_no_network!(Ok(()));

    let boundary = responses::ev_reasoning_item("boundary", &["Preparing the next action"], &[]);
    let mut first_connection = mailbox_initial_responses(&boundary);
    if buffered_item {
        first_connection[1].extend([
            responses::ev_reasoning_item_added("buffered", &[]),
            responses::ev_reasoning_item("buffered", &["Already completed"], &[]),
        ]);
    }
    // Close the original stream without completing it. Ordinary stream recovery
    // retries the sampling step; the following step must still receive both messages.
    let server = responses::start_websocket_server(vec![
        first_connection,
        vec![
            vec![
                responses::ev_response_created("recovery"),
                responses::ev_function_call(
                    "planned-action",
                    "update_plan",
                    r#"{"plan":[{"step":"Reply after reading mail","status":"in_progress"}]}"#,
                ),
                responses::ev_completed("recovery"),
            ],
            vec![
                responses::ev_response_created("follow-up"),
                responses::ev_assistant_message("answer", "The worker updates are included."),
                responses::ev_completed("follow-up"),
            ],
        ],
    ])
    .await;
    let release = server.gate_response_event(&boundary);
    let test = start_mailbox_preemption(&server, release).await?;
    let mut stream_errors = 0;
    let mut plan_updates = 0;
    wait_for_event(&test.codex, |event| {
        assert!(!matches!(event, EventMsg::Error(_)), "{event:?}");
        if let EventMsg::StreamError(error) = event {
            assert_eq!(error.additional_details.as_deref(), Some(
                "stream disconnected before completion: websocket closed by server before response.completed"
            ));
            stream_errors += 1;
        }
        if matches!(event, EventMsg::PlanUpdate(_)) {
            plan_updates += 1;
        }
        matches!(event, EventMsg::TurnComplete(_))
    }).await;
    assert_eq!((stream_errors, plan_updates), (1, 1));
    test.codex.shutdown_and_wait().await?;

    let connections = server.connections();
    assert_eq!(connections.len(), 2);
    assert_eq!(connections[0].len(), 2);
    assert_eq!(connections[1].len(), 2);
    assert!(
        connections
            .iter()
            .flatten()
            .all(|request| request.body_json()["type"] == "response.create")
    );
    let recovery = connections[1][0].body_json();
    assert_eq!(recovery["previous_response_id"], Value::Null);
    let follow_up = connections[1][1].body_json();
    assert_eq!(follow_up["previous_response_id"], "recovery");
    let input = responses::strip_response_item_ids_from_json(responses::strip_metadata_from_json(
        follow_up["input"].clone(),
    ));
    let input = input.as_array().expect("follow-up input");
    let mail = input
        .iter()
        .filter(|item| item["type"] == "agent_message")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(mail, expected_mailbox_input());
    assert_eq!(
        input
            .iter()
            .filter(|item| item["type"] == "function_call_output")
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            json!({"type": "function_call_output", "call_id": "planned-action", "output": "Plan updated"})
        ],
    );
    server.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_preemption_preserves_mail_on_content_filter_retry() -> anyhow::Result<()> {
    core_test_support::skip_if_no_network!(Ok(()));

    let boundary = responses::ev_reasoning_item("boundary", &["Preparing the next action"], &[]);
    let mut first_connection = mailbox_initial_responses(&boundary);
    first_connection[1].push(json!({
        "type": "response.incomplete",
        "response": {"id": "first", "incomplete_details": {"reason": "content_filter"}},
    }));
    let server = responses::start_websocket_server(vec![
        first_connection,
        vec![
            vec![
                responses::ev_response_created("recovery"),
                responses::ev_function_call(
                    "planned-action",
                    "update_plan",
                    r#"{"plan":[{"step":"Reply after reading mail","status":"in_progress"}]}"#,
                ),
                responses::ev_completed("recovery"),
            ],
            vec![
                responses::ev_response_created("follow-up"),
                responses::ev_assistant_message("answer", "The worker updates are included."),
                responses::ev_completed("follow-up"),
            ],
        ],
    ])
    .await;
    let release = server.gate_response_event(&boundary);
    let test = start_mailbox_preemption(&server, release).await?;
    let mut plan_updates = 0;
    wait_for_event(&test.codex, |event| {
        assert!(!matches!(event, EventMsg::Error(_)), "{event:?}");
        if let EventMsg::StreamError(error) = event {
            assert_eq!(error.additional_details.as_deref(), Some(
                "stream disconnected before completion: Incomplete response returned, reason: content_filter"
            ));
        }
        if matches!(event, EventMsg::PlanUpdate(_)) {
            plan_updates += 1;
        }
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    assert_eq!(plan_updates, 1);
    test.codex.shutdown_and_wait().await?;

    let connections = server.connections();
    assert_eq!(connections.len(), 2);
    assert_eq!(connections[0].len(), 2);
    assert_eq!(connections[1].len(), 2);
    assert!(
        connections
            .iter()
            .flatten()
            .all(|request| request.body_json()["type"] == "response.create")
    );
    // Filter guidance belongs to the retry; queued mail reaches the next step.
    let recovery = connections[1][0].body_json();
    let input = responses::strip_response_item_ids_from_json(responses::strip_metadata_from_json(
        recovery["input"].clone(),
    ));
    let input = input.as_array().expect("recovery input");
    let follow_up = connections[1][1].body_json();
    let follow_up_input = responses::strip_response_item_ids_from_json(
        responses::strip_metadata_from_json(follow_up["input"].clone()),
    );
    let follow_up_input = follow_up_input.as_array().expect("follow-up input");
    let mail = follow_up_input
        .iter()
        .filter(|item| item["type"] == "agent_message")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(mail, expected_mailbox_input());
    assert_eq!(
        follow_up_input
            .iter()
            .filter(|item| item["type"] == "function_call_output")
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            json!({"type": "function_call_output", "call_id": "planned-action", "output": "Plan updated"})
        ],
    );
    assert_eq!(
        input
            .iter()
            .filter(|item| item["role"] == "developer")
            .filter(|item| item["content"][0]["text"]
                .as_str()
                .is_some_and(|text| text.contains("<content_filter_guidance>")))
            .count(),
        1,
    );
    server.shutdown().await;
    Ok(())
}

fn mailbox_initial_responses(boundary: &Value) -> Vec<Vec<Value>> {
    vec![
        vec![
            responses::ev_response_created("warmup"),
            responses::ev_completed("warmup"),
        ],
        vec![
            responses::ev_response_created("first"),
            responses::ev_reasoning_item_added("boundary", &["Preparing the next action"]),
            boundary.clone(),
        ],
    ]
}

fn expected_mailbox_input() -> Vec<Value> {
    ["Worker found the result.", "Worker checked the result."]
        .into_iter()
        .map(|message| {
            json!({
                "type": "agent_message", "author": "/root/worker", "recipient": "/root",
                "content": [{"type": "input_text", "text": message}],
            })
        })
        .collect()
}

async fn start_mailbox_preemption(
    server: &responses::WebSocketTestServer,
    release: oneshot::Sender<()>,
) -> anyhow::Result<TestCodex> {
    let base_url = format!("{}/v1", server.uri());
    let config_server = responses::start_mock_server().await;
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_model_info_override("gpt-5.4", |model_info| {
            model_info.use_responses_lite = true;
        })
        .with_config(move |config| {
            config
                .features
                .enable(Feature::InstantInterrupt)
                .expect("enable InstantInterrupt feature");
            config.update_plan_enabled = true;
            config.model_provider.base_url = Some(base_url);
            config.model_provider.supports_websockets = true;
        })
        .build_with_auto_env(&config_server)
        .await?;
    test.codex
        .start_or_steer_turn(TurnInputRequest::user_input(vec![super::text(
            "Plan the next action while the worker finishes.",
        )]))
        .await?;
    wait_for_event(&test.codex, |event| {
        assert!(!matches!(event, EventMsg::Error(_)), "{event:?}");
        matches!(event, EventMsg::ItemStarted(item) if item.item.id() == "boundary")
    })
    .await;
    for message in ["Worker found the result.", "Worker checked the result."] {
        test.codex
            .submit(Op::InterAgentCommunication {
                communication: InterAgentCommunication::new(
                    AgentPath::root().join("worker").expect("worker path"),
                    AgentPath::root(),
                    Vec::new(),
                    message.to_string(),
                    /*trigger_turn*/ false,
                ),
                start_options: Default::default(),
            })
            .await?;
    }
    // The barrier confirms that both messages were queued before finishing the item.
    test.codex
        .submit(Op::RealtimeConversationListVoices)
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::RealtimeConversationListVoicesResponse(_))
    })
    .await;
    release.send(()).expect("release response");
    Ok(test)
}
