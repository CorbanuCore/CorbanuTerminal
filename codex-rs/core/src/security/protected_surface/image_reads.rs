//! #2222: model-selected local media uses the same admission as view_image.
use super::Admission;
use super::McpCall;
use super::Route;
use super::admit;
use super::ask_human;
use super::classify_mcp;
use crate::function_tool::FunctionCallError;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use codex_utils_path_uri::PathUri;
use serde_json::Value;

/// Runs after hooks and before any reference is read. Admission never lifts
/// the extension's filesystem sandbox or read denials.
pub(crate) async fn check_image_generation(
    invocation: &ToolInvocation,
) -> Result<(), FunctionCallError> {
    if invocation.tool_name.namespace.as_deref() != Some("image_gen")
        || invocation.tool_name.name != "imagegen"
    {
        return Ok(());
    }
    let ToolPayload::Function { arguments } = &invocation.payload else {
        return Ok(());
    };
    // Match to_extension_call's first usable environment, which imagegen reads.
    let Some(environment) = invocation
        .step_context
        .environments
        .turn_environments()
        .find(|environment| environment.cwd().to_abs_path().is_ok())
    else {
        return Ok(()); // imagegen refuses local references without an environment.
    };
    // Malformed arguments are rejected by imagegen before any read.
    let Ok(args) = serde_json::from_str::<Value>(arguments) else {
        return Ok(());
    };
    let Some(paths) = args.get("referenced_image_paths") else {
        return Ok(());
    };
    // Prompt prose and images already in history are not new local reads.
    check_local_reads(
        &invocation.session,
        &invocation.turn,
        &invocation.call_id,
        environment.cwd().clone(),
        paths.clone(),
    )
    .await
}

/// Classify all paths together before reading any, with one exact-call approval.
pub(crate) async fn check_local_reads(
    session: &Session,
    turn: &TurnContext,
    call_id: &str,
    cwd: PathUri,
    paths: Value,
) -> Result<(), FunctionCallError> {
    let home = turn.config.codex_home.clone();
    let count = paths.as_array().map_or(0, Vec::len);
    let shown = paths
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .take(5)
        .map(crate::security::protected_reads::shown_path)
        .collect::<Vec<_>>();
    let admission = admit(
        session,
        turn.approval_policy.value(),
        Route::FileRead,
        call_id,
        move || {
            classify_mcp(
                &McpCall {
                    server: "builtin",
                    tool: "read_file",
                    title: None,
                    acts_outside: false,
                    arguments: Some(&paths),
                    cwd: &cwd,
                },
                &home,
            )
        },
    )
    .await;
    match admission {
        Admission::Clear => Ok(()),
        Admission::Refused(refusal) => Err(FunctionCallError::RespondToModel(refusal)),
        Admission::AskHuman(check) => {
            let question = format!(
                "Allow reading {count} local media file(s) (up to five paths shown): {shown:?}? {}",
                check.reason()
            );
            let approved = ask_human(session, turn, call_id, question).await;
            check
                .resolve(session, approved)
                .map_err(FunctionCallError::RespondToModel)
        }
    }
}
