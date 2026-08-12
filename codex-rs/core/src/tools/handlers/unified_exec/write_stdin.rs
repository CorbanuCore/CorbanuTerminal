use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::parse_arguments_for_tool;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::PostToolUsePayload;
use crate::tools::registry::PreToolUsePayload;
use crate::tools::registry::ToolExecutor;
use crate::unified_exec::WriteStdinInteractionEvent;
use crate::unified_exec::WriteStdinRequest;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde::Deserialize;

use super::super::shell_spec::create_write_stdin_tool;
use super::post_unified_exec_tool_use_payload;

#[derive(Debug, Deserialize)]
struct WriteStdinArgs {
    // The model is trained on `session_id`.
    #[serde(deserialize_with = "super::deserialize_integral_i32")]
    session_id: i32,
    #[serde(default)]
    chars: String,
    #[serde(
        default = "super::default_write_stdin_yield_time_ms",
        deserialize_with = "super::deserialize_integral_u64"
    )]
    yield_time_ms: u64,
    #[serde(
        default,
        deserialize_with = "super::deserialize_optional_integral_usize"
    )]
    max_output_tokens: Option<usize>,
}

pub struct WriteStdinHandler;

impl ToolExecutor<ToolInvocation> for WriteStdinHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("write_stdin")
    }

    fn spec(&self) -> ToolSpec {
        create_write_stdin_tool()
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(self.handle_call(invocation))
    }
}

impl WriteStdinHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            payload,
            ..
        } = invocation;

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "write_stdin handler received unsupported payload".to_string(),
                ));
            }
        };

        let args: WriteStdinArgs = parse_arguments_for_tool("write_stdin", &arguments)?;
        let response = session
            .services
            .unified_exec_manager
            .write_stdin(WriteStdinRequest {
                process_id: args.session_id,
                input: &args.chars,
                yield_time_ms: args.yield_time_ms,
                max_output_tokens: args.max_output_tokens,
                truncation_policy: turn.model_info.truncation_policy.into(),
                interaction_event: Some(WriteStdinInteractionEvent {
                    session: &session,
                    turn: &turn,
                }),
            })
            .await
            .map_err(|err| {
                FunctionCallError::RespondToModel(format!("write_stdin failed: {err}"))
            })?;

        Ok(boxed_tool_output(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_integral_float_arguments_from_model_tool_calls() {
        let args: WriteStdinArgs = parse_arguments_for_tool(
            "write_stdin",
            r#"{"session_id":29757.0,"chars":"","yield_time_ms":120000.0,"max_output_tokens":3000.0}"#,
        )
        .expect("integral JSON floats should be compatible with integer tool fields");

        assert_eq!(args.session_id, 29_757);
        assert_eq!(args.yield_time_ms, 120_000);
        assert_eq!(args.max_output_tokens, Some(3_000));
    }

    #[test]
    fn retains_integer_arguments_and_defaults() {
        let args: WriteStdinArgs =
            parse_arguments_for_tool("write_stdin", r#"{"session_id":42,"chars":"continue"}"#)
                .expect("ordinary integer arguments should remain valid");

        assert_eq!(args.session_id, 42);
        assert_eq!(
            args.yield_time_ms,
            super::super::default_write_stdin_yield_time_ms()
        );
        assert_eq!(args.max_output_tokens, None);
    }

    #[test]
    fn rejects_fractional_integer_fields() {
        let error = parse_arguments_for_tool::<WriteStdinArgs>(
            "write_stdin",
            r#"{"session_id":42.5,"yield_time_ms":1000}"#,
        )
        .expect_err("fractional process ids must not be rounded");

        assert!(error.to_string().contains("exactly represented integer"));
    }

    #[test]
    fn rejects_negative_unsigned_fields() {
        let error = parse_arguments_for_tool::<WriteStdinArgs>(
            "write_stdin",
            r#"{"session_id":42,"yield_time_ms":-1.0}"#,
        )
        .expect_err("negative yield times must remain invalid");

        assert!(error.to_string().contains("non-negative integer"));
    }
}

impl CoreToolRuntime for WriteStdinHandler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }

    fn pre_tool_use_payload(&self, _invocation: &ToolInvocation) -> Option<PreToolUsePayload> {
        // `write_stdin` is transport for an existing exec session. Empty writes
        // are background polls, and non-empty writes continue a command that
        // already ran PreToolUse as Bash, so do not emit a second pre hook here.
        None
    }

    fn post_tool_use_payload(
        &self,
        invocation: &ToolInvocation,
        result: &dyn crate::tools::context::ToolOutput,
    ) -> Option<PostToolUsePayload> {
        // A `write_stdin` poll can observe final completion for the original
        // `exec_command`; emit that command's matching Bash PostToolUse.
        post_unified_exec_tool_use_payload(invocation, result)
    }
}
