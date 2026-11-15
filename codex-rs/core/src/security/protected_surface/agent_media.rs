//! Model-selected child attachments must not become unsandboxed user uploads.
use super::image_reads::check_local_reads;
use super::protect_file_tool_context;
use crate::function_tool::FunctionCallError;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use codex_protocol::user_input::UserInput;
use codex_utils_image::data_url_from_bytes;

/// Under protected levels, resolve local media now through the caller's file
/// sandbox. The child receives bytes, so later input serialization cannot read
/// the path again (or follow a link swapped after approval). Human attachments
/// and Permissive keep their existing preparation path.
pub(crate) async fn prepare_agent_media(
    session: &Session,
    step: &StepContext,
    call_id: &str,
    mut items: Vec<UserInput>,
) -> Result<Vec<UserInput>, FunctionCallError> {
    if !session
        .services
        .model_client()
        .post_taint_state()
        .is_some_and(|state| state.protected_paths_apply())
    {
        return Ok(items);
    }
    let paths = items
        .iter()
        .filter_map(|item| match item {
            UserInput::LocalImage { path, .. } | UserInput::LocalAudio { path } => Some(path),
            _ => None,
        })
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Ok(items);
    }
    let environment = step.environments.primary().ok_or_else(|| {
        FunctionCallError::RespondToModel(
            "Local media is unavailable without a file environment".into(),
        )
    })?;
    if environment.environment.is_remote() {
        return Err(FunctionCallError::RespondToModel(
            "Local agent media requires a local file environment".into(),
        ));
    }
    let paths = paths
        .into_iter()
        .map(|path| {
            // Resolve once to an absolute path, so admission and the read use
            // the same location even for a relative path.
            codex_utils_absolute_path::AbsolutePathBuf::try_from(path.as_path())
                .map(|path| codex_utils_path_uri::PathUri::from_abs_path(&path))
                .map_err(|error| FunctionCallError::RespondToModel(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    check_local_reads(
        session,
        &step.turn,
        call_id,
        environment.cwd().clone(),
        serde_json::json!(
            paths
                .iter()
                .map(codex_utils_path_uri::PathUri::inferred_native_path_string)
                .collect::<Vec<_>>()
        ),
    )
    .await?;
    let sandbox = protect_file_tool_context(
        session,
        &step.turn,
        environment,
        step.turn
            .file_system_sandbox_context(/*additional_permissions*/ None, environment),
    );
    let local_items = items.iter_mut().filter(|item| {
        matches!(
            item,
            UserInput::LocalImage { .. } | UserInput::LocalAudio { .. }
        )
    });
    for (item, uri) in local_items.zip(paths) {
        let replacement = match item {
            UserInput::LocalImage { detail, .. } => {
                let bytes = environment
                    .environment
                    .get_filesystem()
                    .read_file(&uri, Some(&sandbox))
                    .await
                    .map_err(|error| {
                        FunctionCallError::RespondToModel(format!(
                            "Unable to read local media: {error}"
                        ))
                    })?;
                UserInput::Image {
                    image_url: data_url_from_bytes("application/octet-stream", &bytes),
                    detail: *detail,
                }
            }
            UserInput::LocalAudio { path } => {
                let mime = match path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .map(str::to_ascii_lowercase)
                    .as_deref()
                {
                    Some("wav") => "audio/wav",
                    Some("mp3") => "audio/mpeg",
                    Some("m4a") => "audio/mp4",
                    Some("webm") => "audio/webm",
                    Some("ogg") => "audio/ogg",
                    _ => {
                        return Err(FunctionCallError::RespondToModel(
                            "Unsupported local audio format".into(),
                        ));
                    }
                };
                let bytes = environment
                    .environment
                    .get_filesystem()
                    .read_file(&uri, Some(&sandbox))
                    .await
                    .map_err(|error| {
                        FunctionCallError::RespondToModel(format!(
                            "Unable to read local media: {error}"
                        ))
                    })?;
                UserInput::Audio {
                    audio_url: data_url_from_bytes(mime, &bytes),
                }
            }
            _ => continue,
        };
        *item = replacement;
    }
    Ok(items)
}
