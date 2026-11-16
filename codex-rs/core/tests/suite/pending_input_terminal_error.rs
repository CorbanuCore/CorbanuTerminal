//! D659: terminal failures preserve pending input without retrying it forever.
use super::*;
use codex_protocol::models::AgentMessageInputContent;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::CodexErrorInfo;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompactionFailurePoint {
    PreTurn,
    MidTurn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingInputAfterFailure {
    Steer,
    QueuedMail,
    TriggeringMail,
}

#[test_case(CompactionFailurePoint::PreTurn, PendingInputAfterFailure::Steer; "pre_turn_steer")]
#[test_case(CompactionFailurePoint::PreTurn, PendingInputAfterFailure::QueuedMail; "pre_turn_mail")]
#[test_case(CompactionFailurePoint::PreTurn, PendingInputAfterFailure::TriggeringMail; "pre_turn_triggering_mail")]
#[test_case(CompactionFailurePoint::MidTurn, PendingInputAfterFailure::Steer; "mid_turn_steer")]
#[test_case(CompactionFailurePoint::MidTurn, PendingInputAfterFailure::QueuedMail; "mid_turn_mail")]
#[test_case(CompactionFailurePoint::MidTurn, PendingInputAfterFailure::TriggeringMail; "mid_turn_triggering_mail")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn terminal_compaction_error_does_not_retry_pending_input(
    failure_point: CompactionFailurePoint,
    pending_input: PendingInputAfterFailure,
) -> anyhow::Result<()> {
    const PENDING_MESSAGE: &str = "pending input must survive the failed compaction";
    const COMPACT_PROMPT: &str = "Summarize the conversation for the next turn.";
    let (release_failure, failure_gate) = oneshot::channel();
    let initial_output = match failure_point {
        CompactionFailurePoint::PreTurn => ev_message_item_done("initial", "first answer"),
        CompactionFailurePoint::MidTurn => ev_function_call("call-1", "test_tool", "{}"),
    };
    let failure = responses::sse_failed("failed-compact", "insufficient_quota", "quota exhausted");
    let mut streams = vec![
        vec![
            chunk(ev_response_created("initial")),
            chunk(initial_output),
            chunk(ev_completed_with_tokens(
                "initial", /*total_tokens*/ 500_000,
            )),
        ],
        vec![StreamingSseChunk {
            gate: Some(failure_gate),
            body: failure.clone(),
        }],
    ];
    // Mail arriving during a failed turn may start one fresh turn. That turn must also
    // stop on the terminal error, persist its mail, and not start another turn for it.
    let failed_turns = if pending_input == PendingInputAfterFailure::TriggeringMail {
        streams.push(vec![StreamingSseChunk {
            gate: None,
            body: failure,
        }]);
        2
    } else {
        1
    };
    streams.extend([
        vec![
            chunk(ev_message_item_done(
                "recovered-summary",
                "RECOVERED_COMPACTION",
            )),
            chunk(ev_completed_with_tokens(
                "recovered-compact",
                /*total_tokens*/ 50,
            )),
        ],
        vec![
            chunk(ev_message_item_done("recovered", "recovered answer")),
            chunk(ev_completed_with_tokens(
                "recovered",
                /*total_tokens*/ 60,
            )),
        ],
    ]);
    let (server, _completions) = start_streaming_sse_server(streams).await;
    let config_server = responses::start_mock_server().await;
    let base_url = format!("{}/v1", server.uri());
    let test = test_codex()
        .with_model("gpt-5.4")
        .with_config(move |config| {
            // Main's OpenAI remote compact endpoint is JSON, not SSE. Exercise the local
            // Responses compaction path, as the other pending-input compaction tests do.
            config.model_provider.name = "OpenAI (test)".to_string();
            config.model_provider.base_url = Some(base_url);
            config.model_provider.supports_websockets = false;
            config.model_provider.stream_max_retries = Some(0);
            config.compact_prompt = Some(COMPACT_PROMPT.to_string());
            config.model_auto_compact_token_limit = Some(100_000);
            // The streaming fixture records raw request bodies for JSON assertions.
            let _ = config.features.disable(Feature::EnableRequestCompression);
        })
        .build_with_auto_env(&config_server)
        .await?;
    let codex = &test.codex;

    if failure_point == CompactionFailurePoint::PreTurn {
        submit_user_input(codex, "initial prompt").await;
        wait_for_turn_complete(codex).await;
    }
    if pending_input == PendingInputAfterFailure::QueuedMail {
        submit_queue_only_agent_mail(codex, PENDING_MESSAGE).await;
    }
    submit_user_input(codex, "prompt that needs compaction").await;
    tokio::time::timeout(
        std::time::Duration::from_secs(/*secs*/ 10),
        server.wait_for_request_count(/*count*/ 2),
    )
    .await?;
    match pending_input {
        PendingInputAfterFailure::Steer => steer_user_input(codex, PENDING_MESSAGE).await,
        PendingInputAfterFailure::QueuedMail => {}
        PendingInputAfterFailure::TriggeringMail => {
            codex
                .submit(Op::InterAgentCommunication {
                    communication: InterAgentCommunication::new(
                        AgentPath::root().join("worker").expect("valid worker path"),
                        AgentPath::root(),
                        Vec::new(),
                        PENDING_MESSAGE.to_string(),
                        /*trigger_turn*/ true,
                    ),
                })
                .await?;
            codex.submit(Op::RealtimeConversationListVoices).await?;
            wait_for_event(codex, |event| {
                matches!(event, EventMsg::RealtimeConversationListVoicesResponse(_))
            })
            .await;
        }
    }
    release_failure.send(()).expect("release compact failure");

    let mut errors = Vec::new();
    let mut completed_turns = 0;
    wait_for_event(codex, |event| {
        match event {
            EventMsg::Error(error) => {
                assert_eq!(
                    error.codex_error_info,
                    Some(CodexErrorInfo::UsageLimitExceeded)
                );
                errors.push(error.clone());
            }
            EventMsg::TurnComplete(completed) => {
                assert_eq!(completed.error.as_ref(), errors.last());
                assert_eq!(completed.last_agent_message, None);
                completed_turns += 1;
            }
            _ => {}
        }
        completed_turns == failed_turns
    })
    .await;
    assert_eq!(errors.len(), failed_turns);
    let requests = server.requests().await;
    assert_eq!(requests.len(), 1 + failed_turns);
    for request in &requests[1..] {
        let body: Value = from_slice(request)?;
        assert!(
            message_input_texts(&body, "user")
                .iter()
                .any(|text| text == COMPACT_PROMPT),
            "failed requests must be compaction attempts"
        );
    }

    codex.flush_rollout().await?;
    let history = codex.load_history(/*include_archived*/ false).await?;
    let saved_pending_messages = history
        .items
        .iter()
        .filter_map(|item| {
            // Main's #1545 contract persists delivered mail as the communication itself.
            if let RolloutItem::InterAgentCommunication(communication) = item {
                return (communication.content == PENDING_MESSAGE)
                    .then_some(communication.content.as_str());
            }
            let RolloutItem::ResponseItem(response_item) = item else {
                return None;
            };
            match response_item {
                ResponseItem::Message { role, content, .. } if role == "user" => {
                    content.iter().find_map(|item| match item {
                        ContentItem::InputText { text } if text == PENDING_MESSAGE => {
                            Some(text.as_str())
                        }
                        _ => None,
                    })
                }
                ResponseItem::AgentMessage { content, .. } => {
                    content.iter().find_map(|item| match item {
                        AgentMessageInputContent::InputText { text } if text == PENDING_MESSAGE => {
                            Some(text.as_str())
                        }
                        _ => None,
                    })
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(saved_pending_messages, vec![PENDING_MESSAGE]);

    // The failed turn must not poison a later explicit retry once compaction can succeed.
    submit_user_input(codex, "retry after quota resets").await;
    wait_for_agent_message(codex, "recovered answer").await;
    let completed = wait_for_event(codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    let EventMsg::TurnComplete(completed) = completed else {
        unreachable!("expected turn completion");
    };
    assert_eq!(completed.error, None);
    assert_eq!(server.requests().await.len(), 3 + failed_turns);
    server.shutdown().await;
    Ok(())
}
