use std::sync::Arc;

use crate::function_tool::FunctionCallError;
use crate::maybe_emit_implicit_skill_invocation;
use crate::security::refusal_notice;
use crate::tools::context::ExecCommandToolOutput;
use crate::tools::context::ToolEnvironments;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
<<<<<<< HEAD
use crate::tools::handlers::apply_granted_turn_permissions;
use crate::tools::handlers::apply_patch::InterceptedPatchSource;
||||||| 413492cd6c
use crate::tools::handlers::apply_granted_turn_permissions;
=======
use crate::tools::handlers::ToolEnvironmentArgs;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::tools::handlers::apply_patch::intercept_apply_patch;
use crate::tools::handlers::function_tool_environment;
use crate::tools::handlers::parse_arguments;
<<<<<<< HEAD
use crate::tools::handlers::parse_arguments_with_base_path;
use crate::tools::handlers::reject_source_write_heredoc_when_structured_edit_enabled;
use crate::tools::handlers::resolve_tool_environment;
||||||| 413492cd6c
use crate::tools::handlers::parse_arguments_with_base_path;
use crate::tools::handlers::resolve_tool_environment;
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use crate::tools::handlers::rewrite_function_string_argument;
use crate::tools::handlers::selected_executor;
use crate::tools::handlers::updated_hook_command;
use crate::tools::hook_names::HookToolName;
use crate::tools::orchestrator::HandlerBasis;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::PostToolUsePayload;
use crate::tools::registry::PreToolUsePayload;
use crate::tools::registry::ToolExecutor;
use crate::tools::registry::ToolTelemetryTags;
use crate::unified_exec::ExecCommandRequest;
use crate::unified_exec::UnifiedExecContext;
use crate::unified_exec::UnifiedExecError;
use crate::unified_exec::UnifiedExecProcessManager;
use crate::unified_exec::generate_chunk_id;
use crate::unified_exec::output::CapturedStream;
use codex_features::Feature;
use codex_otel::SessionTelemetry;
use codex_otel::TOOL_CALL_UNIFIED_EXEC_METRIC;
<<<<<<< HEAD
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxType;
use codex_sandboxing::SandboxablePreference;
use codex_shell_command::shell_detect::detect_shell_type;
||||||| 413492cd6c
use codex_sandboxing::SandboxManager;
use codex_sandboxing::SandboxType;
use codex_sandboxing::SandboxablePreference;
use codex_shell_command::shell_detect::detect_shell_type;
=======
use codex_tools::JsonSchema;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use codex_utils_output_truncation::approx_token_count;
use codex_utils_path_uri::PathUri;
use codex_utils_string::truncate_middle_chars;

use super::super::shell_spec::CommandToolOptions;
use super::super::shell_spec::create_exec_command_tool_with_environment_id;
use super::ExecCommandArgs;
use super::ExecCommandWorkdirArgs;
use super::post_unified_exec_tool_use_payload;
use super::preparation::ExecCommandLifetime;
use super::preparation::ParsedExecCommand;

// A byte limit is a conservative hard token bound even for byte-fallback tokenizers.
const EXEC_COMMAND_REJECTION_MAX_BYTES: usize = 900;

#[derive(Clone, Copy)]
pub(crate) struct ExecCommandHandlerOptions {
    pub(crate) allow_tty: bool,
    pub(crate) exec_permission_approvals_enabled: bool,
    pub(crate) include_environment_id: bool,
    pub(crate) include_windows_shell_guidance: bool,
}

pub struct ExecCommandHandler {
    options: ExecCommandHandlerOptions,
    lifetime: ExecCommandLifetime,
}

impl Default for ExecCommandHandler {
    fn default() -> Self {
        Self {
            lifetime: ExecCommandLifetime::Interactive,
            options: ExecCommandHandlerOptions {
                allow_tty: true,
                exec_permission_approvals_enabled: false,
                include_environment_id: false,
                include_windows_shell_guidance: cfg!(windows),
            },
        }
    }
}

impl ExecCommandHandler {
    pub(crate) fn new(options: ExecCommandHandlerOptions) -> Self {
        Self {
            options,
            lifetime: ExecCommandLifetime::Interactive,
        }
    }

    pub(crate) fn one_shot(options: ExecCommandHandlerOptions) -> Self {
        Self {
            options,
            lifetime: ExecCommandLifetime::OneShot,
        }
    }
}

impl ToolExecutor<ToolInvocation> for ExecCommandHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("exec_command")
    }

    fn spec(&self) -> ToolSpec {
        let spec = create_exec_command_tool_with_environment_id(
            CommandToolOptions {
                exec_permission_approvals_enabled: self.options.exec_permission_approvals_enabled,
            },
            self.options.include_environment_id,
            self.options.include_windows_shell_guidance,
        );
        let mut spec = match self.lifetime {
            ExecCommandLifetime::Interactive => spec,
            ExecCommandLifetime::OneShot => one_shot_exec_command_spec(spec),
        };
        if !self.options.allow_tty
            && let ToolSpec::Function(spec) = &mut spec
        {
            spec.parameters
                .properties
                .get_or_insert_default()
                .remove("tty");
        }
        spec
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

impl ExecCommandHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            environments,
            session,
            step_context,
            cancellation_token,
            tracker,
            call_id,
            payload,
            ..
        } = invocation;

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "exec_command handler received unsupported payload".to_string(),
                ));
            }
        };

        let manager: &UnifiedExecProcessManager = &session.services.unified_exec_manager;
        let context = UnifiedExecContext::new(
            session.clone(),
            step_context.clone(),
            cancellation_token,
            call_id.clone(),
        );
<<<<<<< HEAD
        // `to_abs_path()` alone cannot identify foreign drive paths: `file:///C:/repo` is
        // representable as `/C:/repo` on POSIX. Require the inferred convention to match too.
        let cwd_uses_native_convention =
            cwd.infer_path_convention() == Some(PathConvention::native());
        // TODO(anp): Remove this parsing split once sandboxing supports foreign paths.
        let native_cwd = match cwd.to_abs_path() {
            Ok(cwd) if cwd_uses_native_convention => Some(cwd),
            _ if sandbox == SandboxType::None => None,
            Err(err) => return Err(FunctionCallError::RespondToModel(err.to_string())),
            Ok(_) => {
                return Err(FunctionCallError::RespondToModel(format!(
                    "path URI `{cwd}` does not use the host's native {} path convention",
                    PathConvention::native()
                )));
            }
        };
        let mut args: ExecCommandArgs = match native_cwd.as_ref() {
            Some(native_cwd) => {
                // The base path only resolves paths nested in the permissions config types.
                parse_arguments_with_base_path(&arguments, native_cwd)?
            }
            None => {
                // Parsing without a base only skips relative-path resolution inside the
                // permissions config. That is safe only for a truly unsandboxed attempt;
                // sandboxed attempts fall through and return the conversion error below.
                parse_arguments(&arguments)?
            }
        };
        reject_source_write_heredoc_when_structured_edit_enabled(turn.as_ref(), &args.cmd)?;
        let hook_command = args.cmd.clone();
        // TODO(anp) wire PathUri through implicit skills instead of skipping on foreign paths
        if let Some(native_cwd) = native_cwd.as_ref() {
            maybe_emit_implicit_skill_invocation(
                session.as_ref(),
                context.turn.as_ref(),
                &hook_command,
                native_cwd,
            )
            .await;
        }
        let shell_mode =
            shell_mode_for_environment(&turn.unified_exec_shell_mode, environment.as_ref());
        // Remote environments may use a different OS and must build commands with their native
        // shell; fall back to the session shell when the environment did not report one.
        let shell = turn_environment
            .shell
            .clone()
            .map(Arc::new)
            .unwrap_or_else(|| session.user_shell());
        // TODO(anp): Resolve requested shells in remote environments instead of restricting
        // commands to the reported default shell.
        if environment.is_remote()
            && let Some(requested_shell) = args.shell.take()
        {
            let Some(remote_shell) = turn_environment.shell.as_ref() else {
                return Err(FunctionCallError::RespondToModel(format!(
                    "environment `{}` does not report a shell",
                    turn_environment.environment_id
                )));
            };
            if detect_shell_type(Path::new(&requested_shell)) != Some(remote_shell.shell_type) {
                return Err(FunctionCallError::RespondToModel(format!(
                    "environment `{}` only supports `{}`",
                    turn_environment.environment_id,
                    remote_shell.name()
                )));
            }
        }
        let process_id = manager.allocate_process_id().await;
        let resolved_command = get_command(
            &args,
            shell,
            &shell_mode,
            turn.config.permissions.allow_login_shell,
        )
        .map_err(FunctionCallError::RespondToModel)?;
        let command = resolved_command.command;
        let shell_type = resolved_command.shell_type;
        let command_for_display = codex_shell_command::parse_command::shlex_join(&command);

        let ExecCommandArgs {
            tty,
            yield_time_ms,
            max_output_tokens,
            sandbox_permissions,
            additional_permissions,
            justification,
            prefix_rule,
            ..
        } = args;

        let exec_permission_approvals_enabled =
            session.features().enabled(Feature::ExecPermissionApprovals);
        let requested_additional_permissions = additional_permissions.clone();
        // TODO(anp): Make permission matching operate on PathUri for remote environments.
        let permission_cwd = native_cwd.as_ref().unwrap_or(&turn.config.cwd);
        // Issue #245: under Aggressive a protected path is never opened.
        if let Some(refusal) = crate::security::protected_reads::additional_permission_refusal(
            &session,
            &turn,
            additional_permissions.as_ref(),
            permission_cwd.as_path(),
        )
        .await
        {
            crate::security::protected_reads::notify(&session, &turn, refusal.clone()).await;
            crate::security::protected_reads::record_permission_refusal(&session);
            manager.release_process_id(process_id).await;
            return Err(FunctionCallError::RespondToModel(refusal));
        }
        // #1057: taken before the grants are read.
        let handler_basis = HandlerBasis::now(context.session.as_ref());
        let effective_additional_permissions = apply_granted_turn_permissions(
            context.session.as_ref(),
            &turn_environment.environment_id,
            permission_cwd.as_path(),
            sandbox_permissions,
            additional_permissions,
||||||| 413492cd6c
        // `to_abs_path()` alone cannot identify foreign drive paths: `file:///C:/repo` is
        // representable as `/C:/repo` on POSIX. Require the inferred convention to match too.
        let cwd_uses_native_convention =
            cwd.infer_path_convention() == Some(PathConvention::native());
        // TODO(anp): Remove this parsing split once sandboxing supports foreign paths.
        let native_cwd = match cwd.to_abs_path() {
            Ok(cwd) if cwd_uses_native_convention => Some(cwd),
            _ if sandbox == SandboxType::None => None,
            Err(err) => return Err(FunctionCallError::RespondToModel(err.to_string())),
            Ok(_) => {
                return Err(FunctionCallError::RespondToModel(format!(
                    "path URI `{cwd}` does not use the host's native {} path convention",
                    PathConvention::native()
                )));
            }
        };
        let mut args: ExecCommandArgs = match native_cwd.as_ref() {
            Some(native_cwd) => {
                // The base path only resolves paths nested in the permissions config types.
                parse_arguments_with_base_path(&arguments, native_cwd)?
            }
            None => {
                // Parsing without a base only skips relative-path resolution inside the
                // permissions config. That is safe only for a truly unsandboxed attempt;
                // sandboxed attempts fall through and return the conversion error below.
                parse_arguments(&arguments)?
            }
        };
        let hook_command = args.cmd.clone();
        // TODO(anp) wire PathUri through implicit skills instead of skipping on foreign paths
        if let Some(native_cwd) = native_cwd.as_ref() {
            maybe_emit_implicit_skill_invocation(
                session.as_ref(),
                context.turn.as_ref(),
                &hook_command,
                native_cwd,
            )
            .await;
        }
        let shell_mode =
            shell_mode_for_environment(&turn.unified_exec_shell_mode, environment.as_ref());
        // Remote environments may use a different OS and must build commands with their native
        // shell; fall back to the session shell when the environment did not report one.
        let shell = turn_environment
            .shell
            .clone()
            .map(Arc::new)
            .unwrap_or_else(|| session.user_shell());
        // TODO(anp): Resolve requested shells in remote environments instead of restricting
        // commands to the reported default shell.
        if environment.is_remote()
            && let Some(requested_shell) = args.shell.take()
        {
            let Some(remote_shell) = turn_environment.shell.as_ref() else {
                return Err(FunctionCallError::RespondToModel(format!(
                    "environment `{}` does not report a shell",
                    turn_environment.environment_id
                )));
            };
            if detect_shell_type(Path::new(&requested_shell)) != Some(remote_shell.shell_type) {
                return Err(FunctionCallError::RespondToModel(format!(
                    "environment `{}` only supports `{}`",
                    turn_environment.environment_id,
                    remote_shell.name()
                )));
            }
        }
        let process_id = manager.allocate_process_id().await;
        let resolved_command = get_command(
            &args,
            shell,
            &shell_mode,
            turn.config.permissions.allow_login_shell,
        )
        .map_err(FunctionCallError::RespondToModel)?;
        let command = resolved_command.command;
        let shell_type = resolved_command.shell_type;
        let command_for_display = codex_shell_command::parse_command::shlex_join(&command);

        let ExecCommandArgs {
            tty,
            yield_time_ms,
            max_output_tokens,
            sandbox_permissions,
            additional_permissions,
            justification,
            prefix_rule,
            ..
        } = args;

        let exec_permission_approvals_enabled =
            session.features().enabled(Feature::ExecPermissionApprovals);
        let requested_additional_permissions = additional_permissions.clone();
        // TODO(anp): Make permission matching operate on PathUri for remote environments.
        let permission_cwd = native_cwd.as_ref().unwrap_or(&turn.config.cwd);
        let effective_additional_permissions = apply_granted_turn_permissions(
            context.session.as_ref(),
            &turn_environment.environment_id,
            permission_cwd.as_path(),
            sandbox_permissions,
            additional_permissions,
=======
        let turn_environment = selected_executor(&environments, &step_context.environments)?;
        let parsed =
            ParsedExecCommand::parse(&arguments, &session, &step_context, turn_environment)?;
        maybe_emit_implicit_skill_invocation(
            &session,
            &step_context,
            &parsed.args.cmd,
            &parsed.cwd,
            parsed.native_cwd.as_ref(),
            &turn_environment.selection.environment_id,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        )
        .await;
        let prepared = parsed.prepare(self.lifetime)?;
        let max_output_tokens = prepared.max_output_tokens;
        let fs = prepared.turn_environment.environment.get_filesystem();
        let intercepted_patch = intercept_apply_patch(
            &prepared.resolved.command,
            &prepared.cwd,
            fs.as_ref(),
            prepared.turn_environment.clone(),
            context.session.clone(),
            Arc::clone(&context.step_context),
            context.cancellation_token.clone(),
            Some(&tracker),
            &context.call_id,
            "exec_command",
            InterceptedPatchSource::StrictShell,
        )
        .await;
        if let Some(output) = intercepted_patch? {
            return Ok(boxed_tool_output(ExecCommandToolOutput {
                event_call_id: String::new(),
                chunk_id: String::new(),
                wall_time: std::time::Duration::ZERO,
                raw_output: CapturedStream {
                    text: output.into_text(),
                    omitted_bytes: 0,
                },
                truncation_policy: step_context.settings.model_info.truncation_policy.into(),
                max_output_tokens,
                process_id: None,
                exit_code: None,
                original_token_count: None,
                hook_command: None,
                tool_cwd: None,
            }));
        }

<<<<<<< HEAD
        emit_unified_exec_tty_metric(&turn.session_telemetry, tty);
        let explained_command = command.clone();
        let result = match manager
            .exec_command(
                ExecCommandRequest {
                    command,
                    shell_type,
                    hook_command: hook_command.clone(),
                    process_id,
                    yield_time_ms,
                    max_output_tokens,
                    cwd,
                    sandbox_cwd: native_environment_cwd,
                    turn_environment: turn_environment.clone(),
                    shell_mode,
                    network: context.turn.network.clone(),
                    tty,
                    sandbox_permissions: effective_additional_permissions.sandbox_permissions,
                    additional_permissions: normalized_additional_permissions,
                    additional_permissions_preapproved: effective_additional_permissions
                        .permissions_preapproved,
                    justification,
                    prefix_rule,
                    handler_basis: Some(handler_basis),
                },
                &context,
            )
            .await
        {
            Ok(response) => Ok((response, /*sandbox_denied*/ false)),
||||||| 413492cd6c
        emit_unified_exec_tty_metric(&turn.session_telemetry, tty);
        match manager
            .exec_command(
                ExecCommandRequest {
                    command,
                    shell_type,
                    hook_command: hook_command.clone(),
                    process_id,
                    yield_time_ms,
                    max_output_tokens,
                    cwd,
                    sandbox_cwd: native_environment_cwd,
                    turn_environment: turn_environment.clone(),
                    shell_mode,
                    network: context.turn.network.clone(),
                    tty,
                    sandbox_permissions: effective_additional_permissions.sandbox_permissions,
                    additional_permissions: normalized_additional_permissions,
                    additional_permissions_preapproved: effective_additional_permissions
                        .permissions_preapproved,
                    justification,
                    prefix_rule,
                },
                &context,
            )
            .await
        {
            Ok(response) => Ok(boxed_tool_output(response)),
=======
        emit_unified_exec_tty_metric(&step_context.session_telemetry, prepared.tty);
        crate::tools::lifecycle::notify_command_start(
            context.session.as_ref(),
            context.step_context.turn.as_ref(),
            &context.call_id,
            &prepared.resolved.command,
            &prepared.cwd,
            fs.as_ref(),
        )
        .await;
        // Preparation can be cancelled. Reserve a process only once it is done.
        let process_id = manager.allocate_process_id().await;
        let hook_command = prepared.hook_command.clone();
        let tool_cwd =
            (!prepared.turn_environment.environment.is_remote()).then(|| prepared.cwd.clone());
        let request = ExecCommandRequest {
            prepared,
            process_id,
            network: context.step_context.turn.network.clone(),
        };
        let result = match request.prepared.completion_timeout {
            Some(timeout) => {
                UnifiedExecProcessManager::exec_command_to_completion(request, &context, timeout)
                    .await
            }
            None => manager.exec_command(request, &context).await,
        };
        match result {
            Ok(response) => Ok(boxed_tool_output(response)),
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            Err(UnifiedExecError::SandboxDenied {
                output,
                original_token_count,
                output_omitted_bytes,
                ..
            }) => {
                let output_text = output.aggregated_output.text;
                let original_token_count =
                    original_token_count.unwrap_or_else(|| approx_token_count(&output_text));
<<<<<<< HEAD
                Ok((
                    ExecCommandToolOutput {
                        event_call_id: context.call_id.clone(),
                        chunk_id: generate_chunk_id(),
                        wall_time: output.duration,
                        raw_output: output_text.into_bytes(),
                        truncation_policy: turn.model_info.truncation_policy.into(),
                        max_output_tokens,
                        // Sandbox denial is terminal, so there is no live
                        // process for write_stdin to resume.
                        process_id: None,
                        exit_code: Some(output.exit_code),
                        original_token_count: Some(original_token_count),
                        output_omitted_bytes,
                        hook_command: Some(hook_command),
                    },
                    /*sandbox_denied*/ true,
                ))
||||||| 413492cd6c
                Ok(boxed_tool_output(ExecCommandToolOutput {
                    event_call_id: context.call_id.clone(),
                    chunk_id: generate_chunk_id(),
                    wall_time: output.duration,
                    raw_output: output_text.into_bytes(),
                    truncation_policy: turn.model_info.truncation_policy.into(),
                    max_output_tokens,
                    // Sandbox denial is terminal, so there is no live
                    // process for write_stdin to resume.
                    process_id: None,
                    exit_code: Some(output.exit_code),
                    original_token_count: Some(original_token_count),
                    output_omitted_bytes,
                    hook_command: Some(hook_command),
                }))
=======
                Ok(boxed_tool_output(ExecCommandToolOutput {
                    event_call_id: context.call_id.clone(),
                    chunk_id: generate_chunk_id(),
                    wall_time: output.duration,
                    raw_output: CapturedStream {
                        text: output_text,
                        omitted_bytes: output_omitted_bytes.map_or(0, std::num::NonZeroUsize::get),
                    },
                    truncation_policy: step_context.settings.model_info.truncation_policy.into(),
                    max_output_tokens,
                    // Sandbox denial is terminal, so there is no live
                    // process for write_stdin to resume.
                    process_id: None,
                    exit_code: Some(output.exit_code),
                    original_token_count: Some(original_token_count),
                    hook_command: Some(hook_command),
                    tool_cwd,
                }))
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
<<<<<<< HEAD
            // #439: a refusal reads as its own message, not as a debug dump.
            Err(UnifiedExecError::Rejected { message }) => Err(FunctionCallError::RespondToModel(
                format!("exec_command failed for `{command_for_display}`: {message}"),
            )),
            Err(err) => Err(FunctionCallError::RespondToModel(format!(
                "exec_command failed for `{command_for_display}`: {err}"
            ))),
        };
        let (mut output, sandbox_denied) = result?;
        let output_text = String::from_utf8_lossy(&output.raw_output).into_owned();
        // #442: a protected read the Aggressive rules denied is explained
        // where it failed, to the person and to the model (it is never
        // granted). Otherwise #439: say when Aggressive stopped the command.
        let protected_read = match native_cwd.as_ref().filter(|_| !environment.is_remote()) {
            Some(cwd) => {
                crate::security::protected_reads::denial_explanation(
                    session.as_ref(),
                    turn.as_ref(),
                    &explained_command,
                    cwd.as_path(),
                    output.exit_code,
                    &output_text,
                )
                .await
            }
            None => None,
        };
        // #826 follow-up: and a user namespace the Linux rule refused, named
        // instead of a bare "Operation not permitted".
        let user_namespace = crate::security::user_namespaces::denial_explanation(
            session.as_ref(),
            turn.as_ref(),
            &context.call_id,
            output.exit_code,
            &output_text,
        );
        let explanation = match (protected_read, user_namespace) {
            (Some(read), Some(namespace)) => Some(format!("{read}\n{namespace}")),
            (read, namespace) => read.or(namespace),
        };
        if let Some(explanation) = explanation {
            crate::security::protected_reads::notify(
                session.as_ref(),
                turn.as_ref(),
                explanation.clone(),
            )
            .await;
            output
                .raw_output
                .extend_from_slice(format!("\n{explanation}\n").as_bytes());
        } else if let Some(blocked) =
            refusal_notice::blocked(&turn.config, sandbox_denied, output.exit_code, &output_text)
        {
            notify_blocked(&context, blocked).await;
            output
                .raw_output
                .extend_from_slice(blocked.model_note().as_bytes());
||||||| 413492cd6c
            Err(err) => Err(FunctionCallError::RespondToModel(format!(
                "exec_command failed for `{command_for_display}`: {err:?}"
            ))),
=======
            Err(err) => {
                let message = format!("exec_command failed: {err:?}");
                Err(FunctionCallError::RespondToModel(truncate_middle_chars(
                    &message,
                    EXEC_COMMAND_REJECTION_MAX_BYTES,
                )))
            }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }
        Ok(boxed_tool_output(output))
    }
}

<<<<<<< HEAD
/// #439: tells the person in the transcript why the command failed.
async fn notify_blocked(context: &UnifiedExecContext, blocked: refusal_notice::Blocked) {
    // #622: listed under the inspector's recent denials too.
    crate::security::inspection::record_refusal(
        context.session.thread_id(),
        blocked.inspected(),
        blocked.inspected_outcome(),
    );
    context
        .session
        .send_event(
            context.turn.as_ref(),
            EventMsg::Warning(WarningEvent {
                message: blocked.notice().to_string(),
            }),
        )
        .await;
||||||| 413492cd6c
=======
fn one_shot_exec_command_spec(spec: ToolSpec) -> ToolSpec {
    let ToolSpec::Function(mut spec) = spec else {
        unreachable!("exec_command has a function schema");
    };
    spec.description = spec.description.replacen(
        "Runs a command in a PTY, returning output or a session ID for ongoing interaction.",
        "Runs a command to completion and returns its output. The process is terminated on timeout or cancellation and cannot be resumed.",
        1,
    );
    let properties = spec.parameters.properties.get_or_insert_default();
    properties.remove("tty");
    properties.remove("yield_time_ms");
    properties.insert(
        "timeout_ms".to_string(),
        JsonSchema::number(Some(
            "Maximum command runtime. Defaults to 10000 ms.".to_string(),
        )),
    );
    spec.output_schema = spec.output_schema.map(|schema| {
        let mut schema = schema.into_value();
        if let Some(output_properties) = schema
            .get_mut("properties")
            .and_then(serde_json::Value::as_object_mut)
        {
            output_properties.remove("session_id");
        }
        schema.into()
    });
    ToolSpec::Function(spec)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
}

impl CoreToolRuntime for ExecCommandHandler {
    fn telemetry_tags(&self, invocation: &ToolInvocation) -> ToolTelemetryTags {
        // Record the configured mode so fallback calls remain in their rollout cohort.
        // This does not indicate whether the command actually used a snapshot.
        let mode = if invocation.session.enabled(Feature::ShellSnapshotV2) {
            "v2"
        } else if invocation.session.enabled(Feature::ShellSnapshot) {
            "v1"
        } else {
            "disabled"
        };
        vec![("shell_snapshot_mode", mode.to_string())]
    }

    fn environments_for_input(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<ToolEnvironments, FunctionCallError> {
        function_tool_environment(invocation, |args: ToolEnvironmentArgs| args.environment_id)
    }

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }

    fn pre_tool_use_payload(&self, invocation: &ToolInvocation) -> Option<PreToolUsePayload> {
        let ToolPayload::Function { arguments } = &invocation.payload else {
            return None;
        };

        parse_arguments::<ExecCommandArgs>(arguments)
            .ok()
            .map(|args| PreToolUsePayload {
                tool_name: HookToolName::bash(),
                tool_input: serde_json::json!({ "command": args.cmd }),
            })
    }

    fn pre_tool_use_cwd(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<Option<PathUri>, FunctionCallError> {
        let ToolPayload::Function { arguments } = &invocation.payload else {
            return Ok(None);
        };
        let args: ExecCommandWorkdirArgs = parse_arguments(arguments)?;
        let environment = selected_executor(
            &invocation.environments,
            &invocation.step_context.environments,
        )?;
        let cwd = args.resolve_cwd(environment)?;
        // Hook scripts execute on the host. Remote command paths cannot replace
        // the host cwd in hook input, even when both machines use the same OS.
        Ok((!environment.environment.is_remote()).then_some(cwd))
    }

    fn with_updated_hook_input(
        &self,
        mut invocation: ToolInvocation,
        updated_input: serde_json::Value,
    ) -> Result<ToolInvocation, FunctionCallError> {
        let ToolPayload::Function { arguments } = invocation.payload else {
            return Err(FunctionCallError::RespondToModel(
                "hook input rewrite received unsupported exec_command payload".to_string(),
            ));
        };
        invocation.payload = ToolPayload::Function {
            arguments: rewrite_function_string_argument(
                &arguments,
                "exec_command",
                "cmd",
                updated_hook_command(&updated_input)?,
            )?,
        };
        Ok(invocation)
    }

    fn post_tool_use_payload(
        &self,
        invocation: &ToolInvocation,
        result: &dyn crate::tools::context::ToolOutput,
    ) -> Option<PostToolUsePayload> {
        post_unified_exec_tool_use_payload(invocation, result)
    }
}

fn emit_unified_exec_tty_metric(session_telemetry: &SessionTelemetry, tty: bool) {
    session_telemetry.counter(
        TOOL_CALL_UNIFIED_EXEC_METRIC,
        /*inc*/ 1,
        &[("tty", if tty { "true" } else { "false" })],
    );
}
