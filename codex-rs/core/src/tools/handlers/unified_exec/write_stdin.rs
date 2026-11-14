use crate::function_tool::FunctionCallError;
use crate::security::protected_surface::Admission;
use crate::security::protected_surface::Route;
use crate::security::protected_surface::TypedWindow;
use crate::security::protected_surface::admit;
use crate::security::protected_surface::ask_human;
use crate::security::protected_surface::classify_typed_input;
use crate::security::protected_surface::confined;
use crate::security::protected_surface::is_interrupt;
use crate::security::protected_surface::lock_typed_input;
use crate::security::protected_surface::note_interrupt;
use crate::security::tainted_action::ProtectedActionKind;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::tools::context::ToolEnvironments;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::parse_arguments_for_tool;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::LiveEnvironmentResolver;
use crate::tools::registry::PostToolUsePayload;
use crate::tools::registry::PreToolUsePayload;
use crate::tools::registry::ToolExecutor;
use crate::tools::sandboxing::ToolError;
use crate::unified_exec::UnifiedExecContext;
use crate::unified_exec::UnifiedExecError;
use crate::unified_exec::WriteStdinInteractionEvent;
use crate::unified_exec::WriteStdinRequest;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde::Deserialize;

use super::super::shell_spec::create_write_stdin_tool;
use super::post_unified_exec_tool_use_payload;

#[derive(Debug, Deserialize)]
struct WriteStdinArgs {
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

impl WriteStdinArgs {
    fn parse(payload: &ToolPayload) -> Result<Self, FunctionCallError> {
        let ToolPayload::Function { arguments } = payload else {
            return Err(FunctionCallError::RespondToModel(
                "write_stdin handler received unsupported payload".to_string(),
            ));
        };
        parse_arguments_for_tool("write_stdin", arguments)
    }
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

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
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
            step_context,
            cancellation_token,
            call_id,
            payload,
            environments,
            ..
        } = invocation;

        let ToolEnvironments::Terminal(terminal) = environments else {
            return Err(FunctionCallError::Fatal(
                "write_stdin requires a terminal session".to_string(),
            ));
        };
        let args = WriteStdinArgs::parse(&payload)?;
        // The process the write goes to (the model's `session_id`).
        let process_id = terminal.process_id;
        // #1976 (D491): nothing is typed into a process without the Windows
        // sandbox under Moderate or Aggressive (one started before the level
        // rose); polling and interrupts still work. #2000: a process started
        // before the session's Windows sandbox was set up still runs without
        // it, so typing into it is judged by the sandbox it started with.
        // D522 (a): the terminal's environment launching with MXC counts as
        // no sandbox, as for commands.
        let typing = !args.chars.is_empty() && !is_interrupt(&args.chars);
        if typing {
            let windows_sandbox_level = step_context
                .environments
                .turn_environments()
                .find(|environment| environment.selection.environment_id == terminal.environment_id)
                .map_or(turn.windows_sandbox_level, |environment| {
                    crate::tools::orchestrator::unsandboxed_check_level(
                        environment.config(),
                        turn.windows_sandbox_level,
                    )
                });
            if let Some(refusal) = crate::security::windows_unsandboxed::session_typing_refusal(
                &session,
                windows_sandbox_level,
                session
                    .services
                    .unified_exec_manager
                    .process_windows_sandbox_level(process_id)
                    .await,
            ) {
                crate::security::inspection::record_refusal(
                    session.thread_id(),
                    crate::security::windows_unsandboxed::INSPECTED_TYPING,
                    crate::security::windows_unsandboxed::inspected_outcome(
                        session.services.model_client().live_level(),
                    ),
                );
                return Err(FunctionCallError::RespondToModel(refusal));
            }
            // #2050 (D510): the same for a process on a remote machine without a
            // sandbox for the level it started with (or one that cannot say).
            if let Some((environment_id, environment, started_with)) = session
                .services
                .unified_exec_manager
                .process_remote_environment(process_id)
                .await
                && let Some(refusal) =
                    crate::security::windows_unsandboxed::session_remote_typing_refusal(
                        &session,
                        &environment_id,
                        &environment,
                        started_with,
                    )
                    .await
            {
                crate::security::inspection::record_refusal(
                    session.thread_id(),
                    crate::security::windows_unsandboxed::INSPECTED_TYPING,
                    crate::security::windows_unsandboxed::inspected_outcome(
                        session.services.model_client().live_level(),
                    ),
                );
                return Err(FunctionCallError::RespondToModel(refusal));
            }
            typed_link_check(&session, &turn, process_id, &args).await?;
        }
        // PF-23-S01: writes to one process, interrupts included, are judged
        // and sent one at a time.
        let post_taint = session.services.model_client().post_taint_state();
        let _typing = if args.chars.is_empty() || post_taint.is_none() {
            None
        } else {
            let guard = lock_typed_input(session.thread_id(), process_id).await;
            if is_interrupt(&args.chars) {
                if post_taint.is_some_and(|state| state.taint_generation > 0) {
                    note_interrupt(session.thread_id(), process_id);
                }
            } else {
                post_taint_check(&session, &turn, &call_id, process_id, &args).await?;
            }
            Some(guard)
        };
        let context =
            UnifiedExecContext::new(session.clone(), step_context, cancellation_token, call_id);
        let response = session
            .services
            .unified_exec_manager
            .write_stdin(
                &context,
                WriteStdinRequest {
                    terminal: &terminal,
                    input: &args.chars,
                    yield_time_ms: args.yield_time_ms,
                    max_output_tokens: args.max_output_tokens,
                    truncation_policy: context
                        .step_context
                        .settings
                        .model_info
                        .truncation_policy
                        .into(),
                    interaction_event: Some(WriteStdinInteractionEvent {
                        session: &session,
                        turn: &turn,
                    }),
                },
            )
            .await
            .map_err(|err| {
                let message = match err {
                    UnifiedExecError::StdinApproval(ToolError::Rejected(reason)) => {
                        format!("write_stdin rejected: {reason}")
                    }
                    UnifiedExecError::StdinApproval(ToolError::Codex(err)) => {
                        format!("write_stdin approval failed: {err}")
                    }
                    err => format!("write_stdin failed: {err}"),
                };
                FunctionCallError::RespondToModel(message)
            })?;

        Ok(boxed_tool_output(response))
    }
}

/// #623: under Aggressive, typing a command (or code) into a running process
/// that creates a link to the project's `.codex` or `.agents` is refused like
/// the command itself. One write at a time: a command typed across several
/// writes is not put together here.
async fn typed_link_check(
    session: &Session,
    turn: &TurnContext,
    session_id: i32,
    args: &WriteStdinArgs,
) -> Result<(), FunctionCallError> {
    use crate::security::protected_links;
    if !protected_links::applies(session) {
        return Ok(());
    }
    // A shell's command line, or code typed into an interpreter.
    let typed = [
        vec!["sh".to_string(), "-c".to_string(), args.chars.clone()],
        vec!["python3".to_string(), "-c".to_string(), args.chars.clone()],
    ];
    if !typed
        .iter()
        .any(|command| protected_links::may_link_command(command))
    {
        return Ok(());
    }
    let process_id = session_id.to_string();
    let Some(cwd) = session
        .services
        .unified_exec_manager
        .list_processes()
        .await
        .into_iter()
        .find(|process| process.process_id == process_id)
        .and_then(|process| process.cwd.to_abs_path().ok())
    else {
        // No such running process: the write fails on its own.
        return Ok(());
    };
    for command in &typed {
        if let Some(refusal) = protected_links::refusal(session, turn, command, cwd.as_path()).await
        {
            return Err(FunctionCallError::RespondToModel(refusal));
        }
    }
    Ok(())
}

/// PF-23-S01: typing into a running process after untrusted content is
/// judged like the command it amounts to, and like the process it goes to.
async fn post_taint_check(
    session: &Session,
    turn: &TurnContext,
    call_id: &str,
    session_id: i32,
    args: &WriteStdinArgs,
) -> Result<(), FunctionCallError> {
    let process_id = session_id.to_string();
    let processes = session.services.unified_exec_manager.list_processes().await;
    let live: Vec<i32> = processes
        .iter()
        .filter_map(|process| process.process_id.parse().ok())
        .collect();
    let Some(process) = processes
        .into_iter()
        .find(|process| process.process_id == process_id)
    else {
        // No such running process: the write fails on its own.
        return Ok(());
    };
    let state = session.services.model_client().post_taint_state();
    let tainted = state
        .as_ref()
        .is_some_and(|state| state.taint_generation > 0);
    // PF-23-S02: a process keeps the sandbox it started with. One started
    // without the protected-path rules (before untrusted content, or with
    // them lifted) can still read credentials: typing into it needs the
    // human once under Moderate, and a grant under Aggressive.
    let thread = session.thread_id();
    let ask_unconfined = match state.as_ref() {
        Some(state) => match confined::typing(thread, session_id, state) {
            confined::Typing::Clear => false,
            confined::Typing::AskOnce => true,
            confined::Typing::Refused(refusal) => {
                return Err(FunctionCallError::RespondToModel(refusal));
            }
        },
        None => false,
    };
    // Judged with what was typed since untrusted content arrived, so a
    // command split across calls is seen whole.
    let window = TypedWindow::open(session.thread_id(), session_id, &args.chars, &live);
    let codex_home = turn.config.codex_home.to_path_buf();
    let (texts, unreadable) = (window.texts(), window.unreadable());
    let command = process.command.clone();
    let admission = admit(
        session,
        turn.approval_policy(),
        Route::WriteStdin,
        call_id,
        move || {
            if unreadable {
                return Some(ProtectedActionKind::UnseenCode);
            }
            // The whole text, and after an interrupt the text from it on.
            texts
                .iter()
                .filter_map(|text| classify_typed_input(&command, text, &process.cwd, &codex_home))
                .chain(ask_unconfined.then_some(ProtectedActionKind::UnconfinedProcess))
                .reduce(ProtectedActionKind::strongest)
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
        if count <= TYPED_TEXT_SHOWN * 2 {
            return format!("{text:?}");
        }
        let head: String = text.chars().take(TYPED_TEXT_SHOWN).collect();
        let tail: String = text.chars().skip(count - TYPED_TEXT_SHOWN).collect();
        format!("{head:?} … {tail:?} ({count} characters)")
    };
    let earlier = if window.text() == args.chars {
        String::new()
    } else {
        format!(
            " (with what was typed before, it reads {})",
            shown(window.text())
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
    if ask_unconfined && let Some(state) = &state {
        confined::note_allowed(thread, session_id, state);
    }
    window.clear();
    Ok(())
}

/// Characters of typed text shown from each end in the approval question.
const TYPED_TEXT_SHOWN: usize = 300;

impl CoreToolRuntime for WriteStdinHandler {
    fn repeated_identical_calls_are_polling(&self) -> bool {
        true
    }

    fn live_environment_resolver(&self) -> Option<LiveEnvironmentResolver> {
        Some(super::terminal_session_for_input)
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
        let terminal: super::super::TerminalSessionArgs =
            parse_arguments_for_tool("write_stdin", r#"{"session_id":29757.0,"chars":""}"#)
                .expect("an integral JSON float session id should be accepted");

        assert_eq!(terminal.session_id, 29_757);
        assert_eq!(args.yield_time_ms, 120_000);
        assert_eq!(args.max_output_tokens, Some(3_000));
    }

    #[test]
    fn retains_integer_arguments_and_defaults() {
        let args: WriteStdinArgs =
            parse_arguments_for_tool("write_stdin", r#"{"session_id":42,"chars":"continue"}"#)
                .expect("ordinary integer arguments should remain valid");

        assert_eq!(
            args.yield_time_ms,
            super::super::default_write_stdin_yield_time_ms()
        );
        assert_eq!(args.max_output_tokens, None);
    }

    #[test]
    fn rejects_fractional_integer_fields() {
        let error = parse_arguments_for_tool::<super::super::TerminalSessionArgs>(
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
