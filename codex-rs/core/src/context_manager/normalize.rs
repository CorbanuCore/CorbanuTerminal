use codex_context_fragments::set_annotated_content;
use codex_context_fragments::to_annotated_content;
use codex_history::ResponseItemEnvelope;
use codex_protocol::ResponseItemId;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::InputModality;
use std::collections::HashSet;
use uuid::Uuid;

use crate::context::ContextualUserFragment;
use crate::context::UnsupportedMedia;
use crate::util::error_or_panic;
use tracing::info;
use tracing::warn;

<<<<<<< HEAD
pub(crate) const IMAGE_CONTENT_OMITTED_PLACEHOLDER: &str =
    "image content omitted because you do not support image input";
pub(crate) const AUDIO_CONTENT_OMITTED_PLACEHOLDER: &str =
    "audio content omitted because you do not support audio input";
||||||| 413492cd6c
const IMAGE_CONTENT_OMITTED_PLACEHOLDER: &str =
    "image content omitted because you do not support image input";
const AUDIO_CONTENT_OMITTED_PLACEHOLDER: &str =
    "audio content omitted because you do not support audio input";
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
// Changing this value would change model-visible IDs and invalidate prompt caches.
const SYNTHETIC_OUTPUT_ID_NAMESPACE: Uuid = Uuid::from_u128(0x90d38d3e_6a5b_4d52_bfe2_2f1e634bfac4);

pub(crate) fn ensure_call_outputs_present(items: &mut Vec<ResponseItemEnvelope>) {
    let mut function_output_ids = HashSet::new();
    let mut tool_search_output_ids = HashSet::new();
    let mut custom_tool_output_ids = HashSet::new();
    for envelope in items.iter() {
        match &envelope.item {
            ResponseItem::FunctionCallOutput {
                call_id: Some(call_id),
                ..
            } => {
                function_output_ids.insert(call_id.as_str());
            }
            ResponseItem::ToolSearchOutput {
                call_id: Some(call_id),
                ..
            } => {
                tool_search_output_ids.insert(call_id.as_str());
            }
            ResponseItem::CustomToolCallOutput { call_id, .. } => {
                custom_tool_output_ids.insert(call_id.as_str());
            }
            _ => {}
        }
    }

    // Collect synthetic outputs to insert immediately after their calls.
    // Store the insertion position (index of call) alongside the item so
    // we can insert in reverse order and avoid index shifting.
<<<<<<< HEAD
    let mut missing_outputs_to_insert: Vec<(usize, ResponseItem)> = Vec::new();
    let mut missing_function_outputs = 0usize;
    let mut missing_function_sample: Option<String> = None;
    let mut missing_tool_search_outputs = 0usize;
    let mut missing_tool_search_sample: Option<String> = None;
||||||| 413492cd6c
    let mut missing_outputs_to_insert: Vec<(usize, ResponseItem)> = Vec::new();
=======
    let mut missing_outputs_to_insert: Vec<(usize, ResponseItemEnvelope)> = Vec::new();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

    for (idx, envelope) in items.iter().enumerate() {
        match &envelope.item {
            ResponseItem::FunctionCall { id, call_id, .. }
                if !function_output_ids.contains(call_id.as_str()) =>
            {
                missing_function_outputs += 1;
                if missing_function_sample.is_none() {
                    missing_function_sample = Some(call_id.clone());
                }
                missing_outputs_to_insert.push((
                    idx,
                    ResponseItemEnvelope::new(ResponseItem::FunctionCallOutput {
                        id: synthetic_output_id("fco", id.as_deref()),
                        call_id: Some(call_id.clone()),
                        name: None,
                        namespace: None,
                        output: FunctionCallOutputPayload::from_text("aborted".to_string()),
                        internal_chat_message_metadata_passthrough: None,
                    }),
                ));
            }
            ResponseItem::ToolSearchCall {
                id,
                call_id: Some(call_id),
                ..
            } if !tool_search_output_ids.contains(call_id.as_str()) => {
                missing_tool_search_outputs += 1;
                if missing_tool_search_sample.is_none() {
                    missing_tool_search_sample = Some(call_id.clone());
                }
                missing_outputs_to_insert.push((
                    idx,
                    ResponseItemEnvelope::new(ResponseItem::ToolSearchOutput {
                        id: synthetic_output_id("tso", id.as_deref()),
                        call_id: Some(call_id.clone()),
                        status: "completed".to_string(),
                        execution: "client".to_string(),
                        tools: Vec::new(),
                        internal_chat_message_metadata_passthrough: None,
                    }),
                ));
            }
            ResponseItem::CustomToolCall { id, call_id, .. }
                if !custom_tool_output_ids.contains(call_id.as_str()) =>
            {
                // Preserved-prefix forks mark copied calls as inherited. A call still
                // running in the parent has no output in the child's snapshot.
                if !envelope
                    .metadata
                    .as_ref()
                    .is_some_and(|metadata| metadata.inherited_user_message)
                {
                    error_or_panic(format!(
                        "Custom tool call output is missing for call id: {call_id}"
                    ));
                }
                missing_outputs_to_insert.push((
                    idx,
                    ResponseItemEnvelope::new(ResponseItem::CustomToolCallOutput {
                        id: synthetic_output_id("ctco", id.as_deref()),
                        call_id: call_id.clone(),
                        name: None,
                        output: FunctionCallOutputPayload::from_text("aborted".to_string()),
                        internal_chat_message_metadata_passthrough: None,
                    }),
                ));
            }
            // LocalShellCall is represented in upstream streams by a FunctionCallOutput
            ResponseItem::LocalShellCall {
                id,
                call_id: Some(call_id),
                ..
            } if !function_output_ids.contains(call_id.as_str()) => {
                // Preserved-prefix forks mark copied calls as inherited. A call still
                // running in the parent has no output in the child's snapshot.
                if !envelope
                    .metadata
                    .as_ref()
                    .is_some_and(|metadata| metadata.inherited_user_message)
                {
                    error_or_panic(format!(
                        "Local shell call output is missing for call id: {call_id}"
                    ));
                }
                missing_outputs_to_insert.push((
                    idx,
                    ResponseItemEnvelope::new(ResponseItem::FunctionCallOutput {
                        id: synthetic_output_id("fco", id.as_deref()),
                        call_id: Some(call_id.clone()),
                        name: None,
                        namespace: None,
                        output: FunctionCallOutputPayload::from_text("aborted".to_string()),
                        internal_chat_message_metadata_passthrough: None,
                    }),
                ));
            }
            _ => {}
        }
    }
    drop((
        function_output_ids,
        tool_search_output_ids,
        custom_tool_output_ids,
    ));

    // Insert synthetic outputs in reverse index order to avoid re-indexing.
    for (idx, output_item) in missing_outputs_to_insert.into_iter().rev() {
        items.insert(idx + 1, output_item);
    }

    if missing_function_outputs > 0 {
        info!(
            count = missing_function_outputs,
            sample_call_id = missing_function_sample.as_deref(),
            "function call outputs are missing; inserted aborted outputs"
        );
    }
    if missing_tool_search_outputs > 0 {
        info!(
            count = missing_tool_search_outputs,
            sample_call_id = missing_tool_search_sample.as_deref(),
            "tool search outputs are missing; inserted completed outputs"
        );
    }
}

/// Derives a stable ID for a prompt-only output from its source call's item ID.
///
/// Prompt normalization can run repeatedly without persisting its synthetic
/// outputs, so the namespace and name format must remain stable across retries
/// and resumes to preserve prompt-cache reuse. Returning `None` when the source
/// call has no ID preserves the legacy behavior for older history items.
fn synthetic_output_id(prefix: &str, item_id: Option<&str>) -> Option<ResponseItemId> {
    let source_id = item_id.filter(|id| !id.is_empty())?;
    let name = format!("{prefix}:{source_id}");
    Some(ResponseItemId::with_suffix(
        prefix,
        Uuid::new_v5(&SYNTHETIC_OUTPUT_ID_NAMESPACE, name.as_bytes()),
    ))
}

pub(crate) fn remove_orphan_outputs(items: &mut Vec<ResponseItemEnvelope>) {
    let mut function_call_ids = HashSet::new();
    let mut tool_search_call_ids = HashSet::new();
    let mut custom_tool_call_ids = HashSet::new();
    for envelope in items.iter() {
        match &envelope.item {
            ResponseItem::FunctionCall { call_id, .. }
            | ResponseItem::LocalShellCall {
                call_id: Some(call_id),
                ..
            } => {
                function_call_ids.insert(call_id.as_str());
            }
            ResponseItem::ToolSearchCall {
                call_id: Some(call_id),
                ..
            } => {
                tool_search_call_ids.insert(call_id.as_str());
            }
            ResponseItem::CustomToolCall { call_id, .. } => {
                custom_tool_call_ids.insert(call_id.as_str());
            }
            _ => {}
        }
    }

    let mut orphan_positions = Vec::new();
    for (position, envelope) in items.iter().enumerate() {
        match &envelope.item {
            ResponseItem::FunctionCallOutput {
                call_id: Some(call_id),
                ..
<<<<<<< HEAD
            } => Some(call_id.clone()),
            _ => None,
        })
        .collect();

    let custom_tool_call_ids: HashSet<String> = items
        .iter()
        .filter_map(|i| match i {
            ResponseItem::CustomToolCall { call_id, .. } => Some(call_id.clone()),
            _ => None,
        })
        .collect();

    let mut orphan_function_outputs = 0usize;
    let mut orphan_function_empty_call_ids = 0usize;
    let mut orphan_function_sample: Option<String> = None;
    let mut orphan_custom_tool_outputs = 0usize;
    let mut orphan_custom_tool_sample: Option<String> = None;
    let mut orphan_tool_search_outputs = 0usize;
    let mut orphan_tool_search_sample: Option<String> = None;

    items.retain(|item| match item {
        ResponseItem::FunctionCallOutput { call_id, .. } => {
            let has_match =
                function_call_ids.contains(call_id) || local_shell_call_ids.contains(call_id);
            if !has_match {
                orphan_function_outputs += 1;
                if call_id.trim().is_empty() {
                    orphan_function_empty_call_ids += 1;
                } else if orphan_function_sample.is_none() {
                    orphan_function_sample = Some(call_id.clone());
                }
||||||| 413492cd6c
            } => Some(call_id.clone()),
            _ => None,
        })
        .collect();

    let custom_tool_call_ids: HashSet<String> = items
        .iter()
        .filter_map(|i| match i {
            ResponseItem::CustomToolCall { call_id, .. } => Some(call_id.clone()),
            _ => None,
        })
        .collect();

    items.retain(|item| match item {
        ResponseItem::FunctionCallOutput { call_id, .. } => {
            let has_match =
                function_call_ids.contains(call_id) || local_shell_call_ids.contains(call_id);
            if !has_match {
                error_or_panic(format!(
                    "Orphan function call output for call id: {call_id}"
                ));
=======
            } if !function_call_ids.contains(call_id.as_str()) => {
                error_or_panic(format!(
                    "Orphan function call output for call id: {call_id}"
                ));
                orphan_positions.push(position);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
<<<<<<< HEAD
            has_match
        }
        ResponseItem::CustomToolCallOutput { call_id, .. } => {
            let has_match = custom_tool_call_ids.contains(call_id);
            if !has_match {
                orphan_custom_tool_outputs += 1;
                if orphan_custom_tool_sample.is_none() {
                    orphan_custom_tool_sample = Some(call_id.clone());
                }
||||||| 413492cd6c
            has_match
        }
        ResponseItem::CustomToolCallOutput { call_id, .. } => {
            let has_match = custom_tool_call_ids.contains(call_id);
            if !has_match {
                error_or_panic(format!(
                    "Orphan custom tool call output for call id: {call_id}"
                ));
=======
            ResponseItem::CustomToolCallOutput { call_id, .. }
                if !custom_tool_call_ids.contains(call_id.as_str()) =>
            {
                error_or_panic(format!(
                    "Orphan custom tool call output for call id: {call_id}"
                ));
                orphan_positions.push(position);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
<<<<<<< HEAD
            has_match
        }
        ResponseItem::ToolSearchOutput { execution, .. } if execution == "server" => true,
        ResponseItem::ToolSearchOutput {
            call_id: Some(call_id),
            ..
        } => {
            let has_match = tool_search_call_ids.contains(call_id);
            if !has_match {
                orphan_tool_search_outputs += 1;
                if orphan_tool_search_sample.is_none() {
                    orphan_tool_search_sample = Some(call_id.clone());
                }
||||||| 413492cd6c
            has_match
        }
        ResponseItem::ToolSearchOutput { execution, .. } if execution == "server" => true,
        ResponseItem::ToolSearchOutput {
            call_id: Some(call_id),
            ..
        } => {
            let has_match = tool_search_call_ids.contains(call_id);
            if !has_match {
                error_or_panic(format!("Orphan tool search output for call id: {call_id}"));
=======
            ResponseItem::ToolSearchOutput {
                call_id: Some(call_id),
                execution,
                ..
            } if execution != "server" && !tool_search_call_ids.contains(call_id.as_str()) => {
                error_or_panic(format!("Orphan tool search output for call id: {call_id}"));
                orphan_positions.push(position);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            }
            _ => {}
        }
<<<<<<< HEAD
        ResponseItem::ToolSearchOutput { call_id: None, .. } => true,
        _ => true,
    });

    if orphan_function_outputs > 0 {
        warn!(
            count = orphan_function_outputs,
            empty_call_ids = orphan_function_empty_call_ids,
            sample_call_id = orphan_function_sample.as_deref(),
            "orphan function call outputs removed"
        );
    }
    if orphan_custom_tool_outputs > 0 {
        warn!(
            count = orphan_custom_tool_outputs,
            sample_call_id = orphan_custom_tool_sample.as_deref(),
            "orphan custom tool call outputs removed"
        );
    }
    if orphan_tool_search_outputs > 0 {
        warn!(
            count = orphan_tool_search_outputs,
            sample_call_id = orphan_tool_search_sample.as_deref(),
            "orphan tool search outputs removed"
        );
||||||| 413492cd6c
        ResponseItem::ToolSearchOutput { call_id: None, .. } => true,
        _ => true,
    });
=======
    }

    if !orphan_positions.is_empty() {
        let mut orphan_positions = orphan_positions.into_iter().peekable();
        let mut position = 0;
        items.retain(|_| {
            let retain = orphan_positions.peek() != Some(&position);
            if !retain {
                orphan_positions.next();
            }
            position += 1;
            retain
        });
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
}

pub(crate) fn remove_corresponding_for(items: &mut Vec<ResponseItemEnvelope>, item: &ResponseItem) {
    match item {
        ResponseItem::FunctionCall { call_id, .. } => {
            remove_first_matching(items, |i| {
                matches!(
                    i,
                    ResponseItem::FunctionCallOutput {
                        call_id: Some(existing),
                        ..
                    } if existing == call_id
                )
            });
        }
        ResponseItem::FunctionCallOutput {
            call_id: Some(call_id),
            ..
        } => {
            if let Some(pos) = items.iter().position(|envelope| {
                matches!(&envelope.item, ResponseItem::FunctionCall { call_id: existing, .. } if existing == call_id)
            }) {
                items.remove(pos);
            } else if let Some(pos) = items.iter().position(|envelope| {
                matches!(&envelope.item, ResponseItem::LocalShellCall { call_id: Some(existing), .. } if existing == call_id)
            }) {
                items.remove(pos);
            }
        }
        ResponseItem::ToolSearchCall {
            call_id: Some(call_id),
            ..
        } => {
            remove_first_matching(items, |i| {
                matches!(
                    i,
                    ResponseItem::ToolSearchOutput {
                        call_id: Some(existing),
                        ..
                    } if existing == call_id
                )
            });
        }
        ResponseItem::ToolSearchOutput {
            call_id: Some(call_id),
            ..
        } => {
            remove_first_matching(
                items,
                |i| {
                    matches!(
                        i,
                        ResponseItem::ToolSearchCall {
                            call_id: Some(existing),
                            ..
                        } if existing == call_id
                    )
                },
            );
        }
        ResponseItem::CustomToolCall { call_id, .. } => {
            remove_first_matching(items, |i| {
                matches!(
                    i,
                    ResponseItem::CustomToolCallOutput {
                        call_id: existing, ..
                    } if existing == call_id
                )
            });
        }
        ResponseItem::CustomToolCallOutput { call_id, .. } => {
            remove_first_matching(
                items,
                |i| matches!(i, ResponseItem::CustomToolCall { call_id: existing, .. } if existing == call_id),
            );
        }
        ResponseItem::LocalShellCall {
            call_id: Some(call_id),
            ..
        } => {
            remove_first_matching(items, |i| {
                matches!(
                    i,
                    ResponseItem::FunctionCallOutput {
                        call_id: Some(existing),
                        ..
                    } if existing == call_id
                )
            });
        }
        _ => {}
    }
}

fn remove_first_matching<F>(items: &mut Vec<ResponseItemEnvelope>, predicate: F)
where
    F: Fn(&ResponseItem) -> bool,
{
    if let Some(pos) = items.iter().position(|envelope| predicate(&envelope.item)) {
        items.remove(pos);
    }
}

/// Strip image content from messages and tool outputs when the model does not support images.
/// When `input_modalities` contains `InputModality::Image`, no stripping is performed.
pub(crate) fn strip_images_when_unsupported(
    input_modalities: &[InputModality],
    items: &mut [ResponseItemEnvelope],
) {
    let supports_images = input_modalities.contains(&InputModality::Image);
    if supports_images {
        return;
    }

    for envelope in items.iter_mut() {
        match &mut envelope.item {
            ResponseItem::Message { .. } => {
                let Some(mut content) = to_annotated_content(&mut envelope.item) else {
                    continue;
                };
                for content_item in &mut content {
                    if matches!(content_item.content(), ContentItem::InputImage { .. }) {
                        *content_item = UnsupportedMedia::IMAGE.render_fragment().into_parts().1;
                    }
                }
                let _ = set_annotated_content(&mut envelope.item, content);
            }
            ResponseItem::FunctionCallOutput { output, .. }
            | ResponseItem::CustomToolCallOutput { output, .. } => {
                if let Some(content_items) = output.content_items_mut() {
                    let mut normalized_content_items = Vec::with_capacity(content_items.len());
                    for content_item in content_items.iter() {
                        match content_item {
                            FunctionCallOutputContentItem::InputImage { .. } => {
                                normalized_content_items
                                    .push(UnsupportedMedia::IMAGE.render().into());
                            }
                            _ => normalized_content_items.push(content_item.clone()),
                        }
                    }
                    *content_items = normalized_content_items;
                }
            }
            ResponseItem::ImageGenerationCall { result, .. } => {
                result.clear();
            }
            _ => {}
        }
    }
}

/// Strip audio content from messages and tool outputs when the model does not support audio.
/// When `input_modalities` contains `InputModality::Audio`, no stripping is performed.
pub(crate) fn strip_audio_when_unsupported(
    input_modalities: &[InputModality],
    items: &mut [ResponseItemEnvelope],
) {
    if input_modalities.contains(&InputModality::Audio) {
        return;
    }

    for envelope in items.iter_mut() {
        match &mut envelope.item {
            ResponseItem::Message { .. } => {
                let Some(mut content) = to_annotated_content(&mut envelope.item) else {
                    continue;
                };
                for content_item in &mut content {
                    if matches!(content_item.content(), ContentItem::InputAudio { .. }) {
                        *content_item = UnsupportedMedia::AUDIO.render_fragment().into_parts().1;
                    }
                }
                let _ = set_annotated_content(&mut envelope.item, content);
            }
            ResponseItem::FunctionCallOutput { output, .. }
            | ResponseItem::CustomToolCallOutput { output, .. } => {
                if let Some(content_items) = output.content_items_mut() {
                    for content_item in content_items.iter_mut() {
                        if matches!(
                            content_item,
                            FunctionCallOutputContentItem::InputAudio { .. }
                        ) {
                            *content_item = UnsupportedMedia::AUDIO.render().into();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
