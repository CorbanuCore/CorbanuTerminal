//! Callback tracing identity and originating-step lifetime coverage.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use codex_code_mode::CellId;
use codex_code_mode::CodeModeNestedToolCall;
use codex_code_mode::CodeModeSessionDelegate;
use codex_code_mode::CodeModeToolKind;
use codex_tools::ToolName;
use pretty_assertions::assert_eq;
use tokio_util::sync::CancellationToken;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::Registry;

use super::CodeModeCellDelegate;
use super::CodeModeDispatchBroker;
use super::DispatchMessage;
use crate::session::step_context::StepContext;
use crate::session::tests::make_session_and_context;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use crate::tools::registry::ToolRegistry;
use crate::tools::router::ToolRouter;
use crate::turn_diff_tracker::TurnDiffTracker;
use tokio::sync::oneshot;

#[derive(Default)]
struct DispatchFields(BTreeMap<String, String>);

impl Visit for DispatchFields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_str(field, &format!("{value:?}"));
    }
}

type DispatchRecords = Arc<Mutex<Vec<(BTreeMap<String, String>, bool)>>>;
type RouterParents = Arc<Mutex<Vec<Option<&'static str>>>>;

struct DispatchCapture {
    dispatch_records: DispatchRecords,
    router_parents: RouterParents,
}

impl Layer<Registry> for DispatchCapture {
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, Registry>) {
        if attributes.metadata().target() == "codex_core::tools::router"
            && attributes.metadata().name() == "dispatch_tool_call_with_state"
        {
            let parent = context
                .span(id)
                .and_then(|span| span.parent())
                .map(|parent| parent.metadata().name());
            self.router_parents.lock().unwrap().push(parent);
            return;
        }
        if attributes.metadata().target() != "codex_core::tools::parallel"
            || attributes.metadata().name() != "dispatch_tool_call_with_code_mode_result"
        {
            return;
        }
        let mut fields = DispatchFields::default();
        attributes.record(&mut fields);
        let has_turn_ancestor = context.span(id).is_some_and(|span| {
            span.scope().any(|ancestor| {
                ancestor.metadata().target() == "codex_core::tasks"
                    && ancestor.metadata().name() == "turn"
            })
        });
        self.dispatch_records
            .lock()
            .unwrap()
            .push((fields.0, has_turn_ancestor));
    }
}

struct TestHandler;

impl ToolExecutor<ToolInvocation> for TestHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("audit_probe").with_default_namespace()
    }

    fn spec(&self) -> codex_tools::ToolSpec {
        codex_tools::ToolSpec::Function(codex_tools::ResponsesApiTool {
            name: "audit_probe".to_string(),
            description: "Return a marker for the code-mode dispatch test.".to_string(),
            strict: false,
            defer_loading: None,
            parameters: codex_tools::JsonSchema::default(),
            output_schema: None,
        })
    }

    fn handle<'a>(&'a self, _invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async {
            Ok(Box::new(FunctionToolOutput::from_text(
                "audit-probe-ok".to_string(),
                Some(true),
            )) as Box<dyn crate::tools::context::ToolOutput>)
        })
    }
}

impl CoreToolRuntime for TestHandler {}

#[tokio::test(flavor = "current_thread")]
async fn detached_code_mode_callback_keeps_thread_id_on_dispatch_span() -> anyhow::Result<()> {
    let (session, turn) = make_session_and_context().await;
    let thread_id = session.thread_id.to_string();
    let turn = Arc::new(turn);
    let registry = ToolRegistry::with_handler_for_test(Arc::new(TestHandler));
    let router = Arc::new(ToolRouter::from_registry(
        &turn,
        turn.model_info(),
        registry,
        /*hosted_specs*/ Vec::new(),
        &Default::default(),
    ));
    let step = StepContext::for_test(Arc::clone(&turn)).with_tool_router_for_test(router);
    let broker = Arc::new(CodeModeDispatchBroker::new(
        session.thread_id,
        /*executed_tool_calls*/ Default::default(),
    ));
    let cell_id = CellId::new("audit-cell".to_string());
    broker.mark_cell_ready_for_dispatch(&cell_id, /*originating_call*/ None);
    let records = DispatchRecords::default();
    let router_parents = RouterParents::default();
    let subscriber = tracing_subscriber::registry().with(DispatchCapture {
        dispatch_records: Arc::clone(&records),
        router_parents: Arc::clone(&router_parents),
    });
    let _untraced = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    let _subscriber = tracing::subscriber::set_default(subscriber);
    let delegate = CodeModeCellDelegate {
        broker: Arc::clone(&broker),
        step_context: Arc::clone(&step),
    };
    let _worker = broker.start_turn_worker(
        Arc::new(session),
        step,
        Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
    );
    // Process-owned code-mode callbacks run on a fresh task without a turn span.
    let result = tokio::spawn(async move {
        delegate
            .invoke_tool(
                CodeModeNestedToolCall {
                    cell_id,
                    runtime_tool_call_id: "runtime-audit-call".to_string(),
                    tool_name: ToolName::plain("audit_probe"),
                    tool_kind: CodeModeToolKind::Function,
                    input: Some(serde_json::json!({})),
                },
                CancellationToken::new(),
            )
            .await
    })
    .await?
    .map_err(anyhow::Error::msg)?;
    assert_eq!(result, serde_json::json!("audit-probe-ok"));
    let records = records.lock().unwrap();
    let [(fields, has_turn_ancestor)] = records.as_slice() else {
        panic!("expected one nested dispatch span, got {records:?}");
    };
    assert!(!has_turn_ancestor);
    assert_eq!(fields.get("thread.id"), Some(&thread_id));
    assert_eq!(
        fields.get("tool_name"),
        Some(&TestHandler.tool_name().to_string())
    );
    let call_id = fields
        .get("call_id")
        .expect("dispatch must identify the tool call");
    let uuid = call_id
        .strip_prefix("exec-")
        .expect("nested call ID must belong to code mode");
    uuid::Uuid::parse_str(uuid)?;
    assert_eq!(
        router_parents.lock().unwrap().as_slice(),
        [Some("dispatch_tool_call_with_code_mode_result")]
    );
    Ok(())
}

#[tokio::test]
async fn dropped_tool_callbacks_release_the_origin_before_dispatch() {
    let (session, turn) = make_session_and_context().await;
    let session = Arc::new(session);
    let turn = Arc::new(turn);
    let broker = Arc::new(CodeModeDispatchBroker::new(
        session.thread_id,
        /*executed_tool_calls*/ Default::default(),
    ));
    let delegate = CodeModeCellDelegate {
        broker: Arc::clone(&broker),
        step_context: StepContext::for_test(Arc::clone(&turn)),
    };
    let origin = Arc::downgrade(&delegate.step_context);
    let cell_id = CellId::new("queued-cell".to_string());
    let cancellation_token = CancellationToken::new();
    let mut callback = delegate.invoke_tool(
        CodeModeNestedToolCall {
            cell_id: cell_id.clone(),
            runtime_tool_call_id: "call-a".to_string(),
            tool_name: codex_tools::ToolName::plain("test-tool"),
            tool_kind: codex_code_mode::CodeModeToolKind::Function,
            input: None,
        },
        cancellation_token.clone(),
    );
    assert!(futures::poll!(callback.as_mut()).is_pending());
    assert_eq!(broker.dispatch_rx.len(), 1);
    drop(callback);
    drop(delegate);
    assert!(
        origin.upgrade().is_none(),
        "queued work must not own the step"
    );

    // Observe the queued message's result after its original receiver was dropped.
    let mut message = broker.dispatch_rx.try_recv().unwrap();
    let (response_tx, response_rx) = oneshot::channel();
    let DispatchMessage::InvokeTool {
        response_tx: queued_response,
        ..
    } = &mut message
    else {
        panic!("expected tool invocation");
    };
    *queued_response = response_tx;
    broker.dispatch_tx.send(message).await.unwrap();
    broker.mark_cell_ready_for_dispatch(&cell_id, /*originating_call*/ None);
    let _worker = broker.start_turn_worker(
        Arc::clone(&session),
        StepContext::for_test(Arc::clone(&turn)),
        Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
    );
    let response = response_rx.await;
    assert_eq!(
        response.unwrap(),
        Err("code mode nested tool call cancelled".to_string()),
    );
}

/// Keeps a regular turn running until it is aborted.
struct PendingTurnTask;

impl crate::tasks::SessionTask for PendingTurnTask {
    fn kind(&self) -> crate::state::TaskKind {
        crate::state::TaskKind::Regular
    }

    fn span_name(&self) -> &'static str {
        "session_task.pending_turn"
    }

    async fn run(
        self: Arc<Self>,
        _session: Arc<crate::session::session::Session>,
        _ctx: Arc<crate::session::turn_context::TurnContext>,
        _input: Vec<crate::session::TurnInput>,
        cancellation_token: CancellationToken,
    ) -> crate::tasks::SessionTaskResult {
        cancellation_token.cancelled().await;
        Ok(None)
    }
}

/// Main's `authorization_changed_turn_receives_code_mode_notify_and_user_shell_output`, code-mode
/// notify half (PF-83; the user-shell half is in `session/tests/steering_authority_tests.rs`):
/// after the running turn's permissions changed, a notification from the turn's own code-mode
/// cell still reaches that turn. It is output of work the turn already started, not new input
/// steered into it, so the changed authorization does not defer it to the next turn.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorization_changed_turn_receives_code_mode_notify() {
    let (session, turn, _rx) = crate::session::tests::make_session_and_context_with_rx().await;
    session
        .spawn_task(Arc::clone(&turn), Vec::new(), PendingTurnTask)
        .await;
    session
        .update_settings(crate::session::SessionSettingsUpdate {
            step_settings: crate::session::step_settings::StepSettingsUpdate {
                approval_policy: Some(codex_protocol::protocol::AskForApproval::UnlessTrusted),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .expect("update approval policy");
    assert_ne!(
        turn.approval_policy(),
        codex_protocol::protocol::AskForApproval::UnlessTrusted,
        "the running turn must keep the permissions it was admitted under"
    );

    let broker = Arc::new(CodeModeDispatchBroker::new(
        session.thread_id,
        /*executed_tool_calls*/ Default::default(),
    ));
    let delegate = CodeModeCellDelegate {
        broker: Arc::clone(&broker),
        step_context: StepContext::for_test(Arc::clone(&turn)),
    };
    let cell_id = CellId::new("notify-cell".to_string());
    broker.mark_cell_ready_for_dispatch(&cell_id, /*originating_call*/ None);
    let _worker = broker.start_turn_worker(
        Arc::clone(&session),
        StepContext::for_test(Arc::clone(&turn)),
        Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
    );
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        delegate.notify(
            "notify-call".to_string(),
            cell_id,
            "pf83-notify-output".to_string(),
            CancellationToken::new(),
        ),
    )
    .await
    .expect("notify should finish")
    .expect("notify should reach the running turn");

    let (pending, _) = session
        .input_queue
        .get_pending_input_for_running_task(&session, &turn.sub_id)
        .await;
    let [crate::session::TurnInput::ResponseItem(notify_output)] = pending.as_slice() else {
        panic!("expected the notify output as the turn's input, got {pending:?}");
    };
    assert_eq!(
        notify_output.clone().into_item(),
        codex_protocol::models::ResponseItem::CustomToolCallOutput {
            id: None,
            call_id: "notify-call".to_string(),
            name: Some("exec".to_string()),
            output: codex_protocol::models::FunctionCallOutputPayload::from_text(
                "pf83-notify-output".to_string()
            ),
            internal_chat_message_metadata_passthrough: None,
        }
    );
    session
        .abort_all_tasks(codex_protocol::protocol::TurnAbortReason::Interrupted)
        .await;
}
