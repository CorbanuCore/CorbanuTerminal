use codex_api::AnthropicMessagesRequest;
use codex_api::ApiError;
use codex_api::TransportError;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;
use http::StatusCode;
use serde_json::Value;
use serde_json::json;

/// Anthropic documents a 32 MB Messages API request limit. Keep two decimal megabytes of
/// headroom for serialization differences at proxies and gateways.
pub(crate) const ANTHROPIC_MESSAGES_REQUEST_BUDGET_BYTES: usize = 30_000_000;

/// A 413 response can indicate a lower intermediary limit. Retry once with a substantially
/// smaller body instead of resending the request that the provider already rejected.
pub(crate) const ANTHROPIC_MESSAGES_RETRY_BUDGET_BYTES: usize = 15_000_000;

const OMITTED_IMAGE_PLACEHOLDER: &str = concat!(
    "[Earlier image omitted from this request to stay within the provider payload limit. ",
    "Use view_image again if that visual is still needed.]"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PayloadBudgetReport {
    pub(crate) original_bytes: usize,
    pub(crate) final_bytes: usize,
    pub(crate) omitted_images: usize,
}

/// Bounds a full-history Anthropic request without mutating durable conversation history.
///
/// Images are removed oldest-first, so the model retains the most recent visual evidence. Each
/// removed image becomes a text content block at the same position, preserving tool-result and
/// message structure while making the omission explicit to the model.
pub(crate) fn enforce_anthropic_payload_budget(
    request: &mut AnthropicMessagesRequest,
    max_bytes: usize,
) -> Result<PayloadBudgetReport> {
    let original_bytes = serialized_request_bytes(request)?;
    if original_bytes <= max_bytes {
        return Ok(PayloadBudgetReport {
            original_bytes,
            final_bytes: original_bytes,
            omitted_images: 0,
        });
    }

    let mut current_bytes = original_bytes;
    let mut omitted_images = 0;
    for message in &mut request.messages {
        replace_base64_images_until_within_budget(
            message,
            max_bytes,
            &mut current_bytes,
            &mut omitted_images,
        )?;
        if current_bytes <= max_bytes {
            break;
        }
    }

    let final_bytes = serialized_request_bytes(request)?;
    if final_bytes > max_bytes {
        return Err(CodexErr::InvalidRequest(format!(
            "Anthropic request is {final_bytes} bytes after omitting all inline images, above the \
             safe {max_bytes}-byte request budget; compact the conversation or start a new thread"
        )));
    }

    Ok(PayloadBudgetReport {
        original_bytes,
        final_bytes,
        omitted_images,
    })
}

pub(crate) fn is_anthropic_payload_too_large(error: &ApiError) -> bool {
    matches!(
        error,
        ApiError::Transport(TransportError::Http { status, .. })
            if *status == StatusCode::PAYLOAD_TOO_LARGE
    )
}

fn replace_base64_images_until_within_budget(
    value: &mut Value,
    max_bytes: usize,
    current_bytes: &mut usize,
    omitted_images: &mut usize,
) -> Result<()> {
    if *current_bytes <= max_bytes {
        return Ok(());
    }

    if is_base64_image_block(value) {
        let original_block_bytes = serde_json::to_vec(value)?.len();
        let replacement = image_omission_placeholder(value);
        let replacement_bytes = serde_json::to_vec(&replacement)?.len();
        *value = replacement;
        *current_bytes = current_bytes
            .saturating_sub(original_block_bytes)
            .saturating_add(replacement_bytes);
        *omitted_images += 1;
        return Ok(());
    }

    match value {
        Value::Array(values) => {
            for value in values {
                replace_base64_images_until_within_budget(
                    value,
                    max_bytes,
                    current_bytes,
                    omitted_images,
                )?;
                if *current_bytes <= max_bytes {
                    break;
                }
            }
        }
        Value::Object(object) => {
            for value in object.values_mut() {
                replace_base64_images_until_within_budget(
                    value,
                    max_bytes,
                    current_bytes,
                    omitted_images,
                )?;
                if *current_bytes <= max_bytes {
                    break;
                }
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}

fn is_base64_image_block(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("image")
        && value.pointer("/source/type").and_then(Value::as_str) == Some("base64")
        && value.pointer("/source/data").is_some_and(Value::is_string)
}

fn image_omission_placeholder(image: &Value) -> Value {
    let mut placeholder = json!({
        "type": "text",
        "text": OMITTED_IMAGE_PLACEHOLDER,
    });
    if let Some(cache_control) = image.get("cache_control").cloned()
        && let Some(object) = placeholder.as_object_mut()
    {
        object.insert("cache_control".to_string(), cache_control);
    }
    placeholder
}

fn serialized_request_bytes(request: &AnthropicMessagesRequest) -> Result<usize> {
    serde_json::to_vec(request)
        .map(|body| body.len())
        .map_err(Into::into)
}

#[cfg(test)]
#[path = "anthropic_payload_tests.rs"]
mod tests;
