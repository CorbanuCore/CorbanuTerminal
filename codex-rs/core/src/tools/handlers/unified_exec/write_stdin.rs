use crate::function_tool::FunctionCallError;
use crate::security::protected_surface::Admission;
use crate::security::protected_surface::Route;
use crate::security::protected_surface::TypedWindow;
use crate::security::protected_surface::admit;
use crate::security::protected_surface::ask_human;
use crate::security::protected_surface::classify_typed_input;
use crate::security::tainted_action::ProtectedActionKind;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
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
            call_id,
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
        if !args.chars.is_empty() {
            post_taint_check(&session, &turn, &call_id, &args).await?;
        }
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

/// PF-23-S01: typing into a running process after untrusted content is
/// judged like the command it amounts to, and like the process it goes to.
async fn post_taint_check(
    session: &Session,
    turn: &TurnContext,
    call_id: &str,
    args: &WriteStdinArgs,
) -> Result<(), FunctionCallError> {
    let process_id = args.session_id.to_string();
    let Some(process) = session
        .services
        .unified_exec_manager
        .list_processes()
        .await
        .into_iter()
        .find(|process| process.process_id == process_id)
    else {
        // No such running process: the write fails on its own.
        return Ok(());
    };
    let tainted = session
        .services
        .model_client()
        .post_taint_state()
        .is_some_and(|state| state.taint_generation > 0);
    // Judged with what was typed since untrusted content arrived, so a
    // command split across calls is seen whole.
    let window = TypedWindow::open(session.thread_id(), args.session_id, &args.chars);
    let codex_home = turn.config.codex_home.to_path_buf();
    let (text, overflows) = (window.text.clone(), window.overflows());
    let command = process.command.clone();
    let admission = admit(
        session,
        turn.approval_policy.value(),
        Route::WriteStdin,
        call_id,
        move || {
            if overflows {
                return Some(ProtectedActionKind::UnseenCode);
            }
            classify_typed_input(&command, &text, &process.cwd, &codex_home)
        },
    )
    .await;
    let check = match admission {
        Admission::Clear => {
            if tainted {
                window.keep();
            }
            return Ok(());
        }
        Admission::Refused(refusal) => return Err(FunctionCallError::RespondToModel(refusal)),
        Admission::AskHuman(check) => check,
    };
    let shown = |text: &str| {
        let count = text.chars().count();
        let tail: String = text
            .chars()
            .skip(count.saturating_sub(TYPED_TEXT_SHOWN))
            .collect();
        if count > TYPED_TEXT_SHOWN {
            format!("…{tail:?}")
        } else {
            format!("{tail:?}")
        }
    };
    let earlier = if window.text == args.chars {
        String::new()
    } else {
        format!(
            " (with what was typed before, it reads {})",
            shown(&window.text)
        )
    };
    let question = format!(
        "Type {}{earlier} into the running `{}` (session {process_id})? {}",
        shown(&args.chars),
        process.command,
        check.reason()
    );
    let approved = ask_human(session, turn, call_id, question).await;
    check
        .resolve(session, approved)
        .map_err(FunctionCallError::RespondToModel)?;
    window.clear();
    Ok(())
}

/// Characters of typed text shown in the approval question.
const TYPED_TEXT_SHOWN: usize = 400;

impl CoreToolRuntime for WriteStdinHandler {
    fn repeated_identical_calls_are_polling(&self) -> bool {
        true
    }
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
