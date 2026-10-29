use super::*;
use pretty_assertions::assert_eq;

const CALL_ID: &str = "slow-to-record-call";

/// Runs one direct tool call the way a regular turn does, but is slow to record
/// the call's output, like a loaded runtime that misses the interrupt grace
/// period (#1467).
struct SlowToRecordToolTask {
    tool_started: Arc<tokio::sync::Notify>,
    /// `test_sync_tool` arguments.
    arguments: serde_json::Value,
}

impl SessionTask for SlowToRecordToolTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.slow_to_record_tool"
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        ctx: Arc<TurnContext>,
        _input: Vec<TurnInput>,
        cancellation_token: CancellationToken,
    ) -> SessionTaskResult {
        let session = session.clone_session();
        let item = ResponseItem::FunctionCall {
            id: None,
            name: "test_sync_tool".to_string(),
            namespace: None,
            arguments: self.arguments.to_string(),
            call_id: CALL_ID.to_string(),
            encrypted_function_args: None,
            internal_chat_message_metadata_passthrough: None,
        };
        session
            .record_conversation_items(&ctx, std::slice::from_ref(&item))
            .await;
        let call = ToolRouter::build_tool_call(item)
            .expect("test_sync_tool call should parse")
            .expect("test_sync_tool item should build a tool call");
        let tool = test_tool_runtime(Arc::clone(&session), Arc::clone(&ctx))
            .handle_tool_call(call, cancellation_token.child_token());
        // A tool that sleeps is interrupted while running; one that does not is
        // left to finish first.
        let finishes = self.arguments.get("sleep_after_ms").is_none();
        if !finishes {
            self.tool_started.notify_one();
        }
        let output = tool.await.expect("tool should return an output");
        if finishes {
            self.tool_started.notify_one();
        }
        // The interrupt grace period runs out before the output is recorded.
        sleep(Duration::from_secs(60)).await;
        session
            .record_conversation_items(&ctx, &[output.into()])
            .await;
        Ok(None)
    }
}

/// Spawns the task, waits for its tool call, interrupts and returns the call's
/// text outputs in history.
async fn interrupted_call_outputs(arguments: serde_json::Value) -> Vec<Option<String>> {
    let (sess, mut tc) = make_session_and_context().await;
    tc.model_info
        .experimental_supported_tools
        .push("test_sync_tool".to_string());
    let sess = Arc::new(sess);
    let tool_started = Arc::new(tokio::sync::Notify::new());
    sess.spawn_task(
        Arc::new(tc),
        Vec::new(),
        SlowToRecordToolTask {
            tool_started: Arc::clone(&tool_started),
            arguments,
        },
    )
    .await;
    timeout(Duration::from_secs(10), tool_started.notified())
        .await
        .expect("tool call should start");

    sess.abort_all_tasks(TurnAbortReason::Interrupted).await;

    let history = sess.clone_history().await;
    history
        .raw_items()
        .iter()
        .filter_map(|item| match item {
            ResponseItem::FunctionCallOutput {
                call_id, output, ..
            } if call_id == CALL_ID => Some(output.body.to_text()),
            _ => None,
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hard_aborted_turn_records_tool_abort_output() {
    let outputs = interrupted_call_outputs(json!({ "sleep_after_ms": 60_000 })).await;
    let [Some(output)] = outputs.as_slice() else {
        panic!("expected one text abort output for the call, got {outputs:?}");
    };
    assert!(
        output.starts_with("aborted by user after ") && output.ends_with('s'),
        "expected the tool's timed abort output, got {output:?}"
    );
}

/// D244: the tool finished before the interrupt, but the turn was killed before
/// it recorded the result. The real result is recorded, not an abort output.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hard_aborted_turn_records_finished_tool_output() {
    let outputs = interrupted_call_outputs(json!({})).await;
    assert_eq!(outputs, vec![Some("ok".to_string())]);
}
