use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_prompts::ResolvedModelMessages;
use codex_tools::IndirectNamespacePrefixes;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use std::borrow::Cow;
use std::sync::Arc;

use super::ExecContext;
use super::PUBLIC_TOOL_NAME;
use super::handle_runtime_response;
use super::is_exec_tool_name;
use super::output::CodeModeToolOutput;
use super::prepare_code_mode_tool_definitions;
use super::telemetry::CodeModeToolCallGuard;
use super::telemetry::trace_id;

type CodeModeNestedTool = (Arc<ToolSpec>, Option<Arc<dyn CoreToolRuntime>>);

pub struct CodeModeExecuteHandler {
    spec: ToolSpec,
    nested_tool_specs: Vec<CodeModeNestedTool>,
}

impl CodeModeExecuteHandler {
    pub(crate) fn new(spec: ToolSpec, nested_tool_specs: Vec<CodeModeNestedTool>) -> Self {
        Self {
            spec,
            nested_tool_specs,
        }
    }

    async fn execute(
        &self,
        session: std::sync::Arc<crate::session::session::Session>,
        step_context: Arc<crate::session::step_context::StepContext>,
        call_id: String,
        originating_call: Option<crate::tools::context::ToolCallOrigin>,
        code: String,
        telemetry: &mut CodeModeToolCallGuard,
    ) -> Result<CodeModeToolOutput, FunctionCallError> {
        let args =
            codex_code_mode::parse_exec_source(&code).map_err(FunctionCallError::RespondToModel)?;
        let exec = ExecContext {
            session,
            turn: Arc::clone(&step_context.turn),
        };
        let code_mode_input_schema_max_bytes = step_context
            .turn
            .config
            .code_mode
            .tool_input_schema_max_bytes;
        let mut enabled_tools = Vec::with_capacity(self.nested_tool_specs.len());
        for (spec, cached_runtime) in &self.nested_tool_specs {
            let definitions = prepare_code_mode_tool_definitions(
                cached_runtime.as_deref(),
                || Cow::Borrowed(spec.as_ref()),
                code_mode_input_schema_max_bytes,
            );
            match definitions {
                Cow::Borrowed(definitions) => enabled_tools.extend_from_slice(definitions),
                Cow::Owned(definitions) => enabled_tools.extend(definitions),
            }
        }
        enabled_tools.sort_by(|left, right| left.name.cmp(&right.name));
        enabled_tools.dedup_by(|left, right| left.name == right.name);
        let model_messages = ResolvedModelMessages::from_model(&step_context.settings.model_info);
        IndirectNamespacePrefixes::new(
            model_messages.indirect_description_prefixes(),
            step_context.tool_router.mcp_namespaces(),
        )
        .map_err(|error| FunctionCallError::Fatal(error.to_string()))?
        .apply_code_mode(&mut enabled_tools);
        let started_at = std::time::Instant::now();
        let started_cell = exec
            .session
            .services
            .code_mode_service
            .execute(
                codex_code_mode::ExecuteRequest {
                    tool_call_id: call_id.clone(),
                    enabled_tools,
                    source: args.code.clone(),
                    yield_time_ms: args.yield_time_ms,
                    max_output_tokens: args.max_output_tokens,
                },
                Arc::clone(&step_context),
            )
            .await
            .map_err(FunctionCallError::RespondToModel)?;
        let cell_id = started_cell.cell_id.clone();
        tracing::Span::current().record("cell.id", trace_id(cell_id.as_str()));
        telemetry.cell_id = Some(cell_id.to_string());
        exec.session
            .services
            .analytics_events_client
            .track_code_mode_tool_call(codex_analytics::CodeModeToolCallFact::CellStarted {
                thread_id: exec.session.thread_id.to_string(),
                turn_id: exec.turn.sub_id.clone(),
                call_id: call_id.clone(),
                cell_id: cell_id.to_string(),
            });
        exec.session
            .services
            .executed_tool_calls
            .start_cell(&cell_id, &call_id);
        let runtime_cell_id = cell_id.to_string();
        let code_cell_trace = exec
            .session
            .services
            .rollout_thread_trace
            .start_code_cell_trace(
                exec.turn.sub_id.as_str(),
                runtime_cell_id.as_str(),
                call_id.as_str(),
                args.code.as_str(),
            );
        exec.session
            .services
            .code_mode_service
            .mark_cell_ready_for_dispatch(&cell_id, originating_call);
        let response = started_cell
            .initial_response()
            .await
            .map_err(FunctionCallError::RespondToModel)?;
        if let Some(code_mode_host_duration) = response.code_mode_host_duration() {
            telemetry.record_code_mode_host_duration(code_mode_host_duration);
        }
        // Record the raw runtime boundary. The model-visible custom-tool output
        // is produced by `handle_runtime_response` and later linked through
        // `CodeCell.output_item_ids` in the reduced trace.
        code_cell_trace.record_initial_response(&response);
        // Yielded cells keep running, so terminal lifecycle is only emitted
        // here when the first response also ended the runtime.
        if matches!(response, codex_code_mode::RuntimeResponse::Yielded { .. }) {
            telemetry.record_cell_timing(
                &cell_id,
                codex_extension_api::CodeModeCellTiming::ExecYielded,
            );
        } else {
            code_cell_trace.record_ended(&response);
            exec.session
                .services
                .code_mode_service
                .finish_cell_dispatch(&cell_id);
            exec.session
                .services
                .analytics_events_client
                .track_code_mode_tool_call(codex_analytics::CodeModeToolCallFact::CellClosed {
                    thread_id: exec.session.thread_id.to_string(),
                    turn_id: exec.turn.sub_id.clone(),
                    cell_id: cell_id.to_string(),
                });
        }
        exec.session.services.elicitations.wait_until_clear().await;
        let wall_time = response
            .code_mode_host_duration()
            .unwrap_or_else(|| started_at.elapsed());
        Ok(handle_runtime_response(
            &step_context.settings.model_info,
            response,
            args.max_output_tokens,
            exec.session.services.code_mode_service.output_truncation(),
            wall_time,
            exec.turn.config.code_mode.experimental_show_cell_overhead,
        ))
    }
}

impl ToolExecutor<ToolInvocation> for CodeModeExecuteHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(PUBLIC_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        self.spec.clone()
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(self.handle_call(invocation))
    }
}

impl CodeModeExecuteHandler {
    // Default to interrupted if this future is dropped; telemetry::CodeModeToolCallGuard::finish
    // overwrites this handler's captured span on explicit success or failure.
    #[tracing::instrument(
        name = "code_mode.handler.execute",
        level = "info",
        skip_all,
        fields(
            conversation.id = %invocation.session.thread_id,
            turn_id = invocation.turn.sub_id.as_str(),
            call_id = trace_id(&invocation.call_id),
            cell.id = tracing::field::Empty,
            outcome = "interrupted",
        )
    )]
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let handler_span = tracing::Span::current();
        let originating_call = invocation.originating_call().await;
        let ToolInvocation {
            session,
            turn,
            step_context,
            call_id,
            tool_name,
            payload,
            ..
        } = invocation;

<<<<<<< HEAD
        match exec_source(&tool_name, &payload) {
            Some(source) => self
                .execute(session, turn, call_id, source)
||||||| 413492cd6c
        match payload {
            ToolPayload::Custom { input } if is_exec_tool_name(&tool_name) => self
                .execute(session, turn, call_id, input)
=======
        let mut telemetry = CodeModeToolCallGuard::new(
            &session,
            turn.sub_id.clone(),
            turn.turn_metadata_state.clone(),
            call_id.clone(),
            PUBLIC_TOOL_NAME,
            handler_span,
        );
        let result = match payload {
            ToolPayload::Custom { input } if is_exec_tool_name(&tool_name) => self
                .execute(
                    session,
                    step_context,
                    call_id,
                    originating_call,
                    input,
                    &mut telemetry,
                )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                .await
                .map(boxed_tool_output),
<<<<<<< HEAD
            None => Err(FunctionCallError::RespondToModel(
                invalid_exec_payload_message(&payload),
            )),
        }
||||||| 413492cd6c
            _ => Err(FunctionCallError::RespondToModel(format!(
                "{PUBLIC_TOOL_NAME} expects raw JavaScript source text"
            ))),
        }
=======
            _ => Err(FunctionCallError::RespondToModel(format!(
                "{PUBLIC_TOOL_NAME} expects raw JavaScript source text"
            ))),
        };
        telemetry.finish(
            result
                .as_ref()
                .is_ok_and(|output| output.success_for_logging()),
        );
        result
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
}

fn invalid_exec_payload_message(payload: &ToolPayload) -> String {
    match payload {
        ToolPayload::Function { .. } => format!(
            "{PUBLIC_TOOL_NAME} arguments must be a JSON object with one string field `input` \
             holding the JavaScript source. The call was not run: its arguments were not valid \
             JSON of that shape. Escape every `\"` and `\\` inside `input`, then retry."
        ),
        _ => format!("{PUBLIC_TOOL_NAME} expects raw JavaScript source text"),
    }
}

/// Resolves the JavaScript source of an `exec` call. Chat Completions and
/// Anthropic adapters expose the freeform `exec` tool as a function with a
/// single `input` string, so both payload shapes carry the same source.
fn exec_source(tool_name: &ToolName, payload: &ToolPayload) -> Option<String> {
    if !is_exec_tool_name(tool_name) {
        return None;
    }
    payload.freeform_input()
}

impl CoreToolRuntime for CodeModeExecuteHandler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(
            payload,
            ToolPayload::Custom { .. } | ToolPayload::Function { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_tools::FreeformTool;
    use codex_tools::FreeformToolFormat;
    use pretty_assertions::assert_eq;

    fn handler() -> CodeModeExecuteHandler {
        CodeModeExecuteHandler::new(
            ToolSpec::Freeform(FreeformTool {
                name: PUBLIC_TOOL_NAME.to_string(),
                description: String::new(),
                format: FreeformToolFormat {
                    r#type: "grammar".to_string(),
                    syntax: "lark".to_string(),
                    definition: "start: /.+/".to_string(),
                },
            }),
            Vec::new(),
        )
    }

    #[test]
    fn exec_accepts_custom_and_function_wrapped_payloads() {
        let handler = handler();
        let custom = ToolPayload::Custom {
            input: "text(1)".to_string(),
        };
        let function = ToolPayload::Function {
            arguments: r#"{"input":"text(1)"}"#.to_string(),
        };
        let exec = ToolName::plain(PUBLIC_TOOL_NAME);

        assert!(handler.matches_kind(&custom));
        assert!(handler.matches_kind(&function));
        assert_eq!(exec_source(&exec, &custom).as_deref(), Some("text(1)"));
        assert_eq!(exec_source(&exec, &function).as_deref(), Some("text(1)"));
    }

    #[test]
    fn exec_rejects_malformed_function_payloads_and_other_tool_names() {
        let exec = ToolName::plain(PUBLIC_TOOL_NAME);
        for arguments in ["", "{}", r#"{"code":"text(1)"}"#, r#"{"input":1}"#] {
            let payload = ToolPayload::Function {
                arguments: arguments.to_string(),
            };
            assert_eq!(exec_source(&exec, &payload), None, "{arguments}");
        }

        let other = ToolName::plain("wait");
        let payload = ToolPayload::Custom {
            input: "text(1)".to_string(),
        };
        assert_eq!(exec_source(&other, &payload), None);
    }
}
