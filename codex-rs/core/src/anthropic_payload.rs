use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use codex_api::AnthropicMessagesRequest;
use codex_api::ApiError;
use codex_api::TransportError;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;
use codex_utils_image::PromptImageMode;
use codex_utils_image::PromptImageResizeLimits;
use codex_utils_image::base64_image_dimensions;
use codex_utils_image::load_data_url_for_prompt;
use http::StatusCode;
use serde_json::Value;
use serde_json::json;
use tracing::warn;

/// Anthropic documents a 32 MB Messages API request limit. Keep two decimal megabytes of
/// headroom for serialization differences at proxies and gateways.
#[cfg(test)]
pub(crate) const ANTHROPIC_MESSAGES_REQUEST_BUDGET_BYTES: usize = 30_000_000;

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

/// Anthropic's per-image limit, and the stricter limit that applies to every
/// image once a request carries more than `ANTHROPIC_MANY_IMAGES_THRESHOLD`.
const ANTHROPIC_MAX_IMAGE_DIMENSION: u32 = 8_000;
const ANTHROPIC_MANY_IMAGES_MAX_DIMENSION: u32 = 2_000;
const ANTHROPIC_MANY_IMAGES_THRESHOLD: usize = 20;

const UNRESIZABLE_IMAGE_PLACEHOLDER: &str = concat!(
    "[Earlier image omitted from this request because it exceeded the provider's image size ",
    "limit and could not be resized. Use view_image again if that visual is still needed.]"
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ImageDimensionReport {
    pub(crate) resized_images: usize,
    pub(crate) omitted_images: usize,
}

/// Brings every inline image within Anthropic's documented dimension limits,
/// without mutating durable conversation history. A request with more than 20
/// images is rejected outright if any image exceeds 2000 px on a side, so one
/// oversized screenshot in a long session made every later turn fail.
pub(crate) fn fit_anthropic_image_dimensions(
    request: &mut AnthropicMessagesRequest,
) -> ImageDimensionReport {
    let image_count = request
        .messages
        .iter()
        .map(count_image_blocks)
        .sum::<usize>();
    let max_dimension = if image_count > ANTHROPIC_MANY_IMAGES_THRESHOLD {
        ANTHROPIC_MANY_IMAGES_MAX_DIMENSION
    } else {
        ANTHROPIC_MAX_IMAGE_DIMENSION
    };
    let mut report = ImageDimensionReport::default();
    for message in &mut request.messages {
        fit_image_blocks(message, max_dimension, &mut report);
    }
    report
}

fn count_image_blocks(value: &Value) -> usize {
    if value.get("type").and_then(Value::as_str) == Some("image") {
        return 1;
    }
    match value {
        Value::Array(values) => values.iter().map(count_image_blocks).sum(),
        Value::Object(object) => object.values().map(count_image_blocks).sum(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
    }
}

fn fit_image_blocks(value: &mut Value, max_dimension: u32, report: &mut ImageDimensionReport) {
    if is_base64_image_block(value) {
        fit_image_block(value, max_dimension, report);
        return;
    }
    match value {
        Value::Array(values) => {
            for value in values {
                fit_image_blocks(value, max_dimension, report);
            }
        }
        Value::Object(object) => {
            for value in object.values_mut() {
                fit_image_blocks(value, max_dimension, report);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn fit_image_block(image: &mut Value, max_dimension: u32, report: &mut ImageDimensionReport) {
    let Some(data) = image.pointer("/source/data").and_then(Value::as_str) else {
        return;
    };
    let Some((width, height)) = base64_image_dimensions(data) else {
        return;
    };
    if width.max(height) <= max_dimension {
        return;
    }
    let media_type = image
        .pointer("/source/media_type")
        .and_then(Value::as_str)
        .unwrap_or("image/png");
    let limits = PromptImageMode::ResizeWithLimits(PromptImageResizeLimits {
        max_dimension,
        max_patches: usize::MAX,
    });
    match load_data_url_for_prompt(&format!("data:{media_type};base64,{data}"), limits) {
        Ok(resized) => {
            if let Some(source) = image.get_mut("source").and_then(Value::as_object_mut) {
                source.insert("media_type".to_string(), Value::String(resized.mime));
                source.insert(
                    "data".to_string(),
                    Value::String(BASE64_STANDARD.encode(&resized.bytes)),
                );
            }
            report.resized_images += 1;
        }
        Err(error) => {
            warn!(%error, width, height, "omitting an Anthropic image that could not be resized");
            let mut placeholder = json!({
                "type": "text",
                "text": UNRESIZABLE_IMAGE_PLACEHOLDER,
            });
            if let Some(cache_control) = image.get("cache_control").cloned()
                && let Some(object) = placeholder.as_object_mut()
            {
                object.insert("cache_control".to_string(), cache_control);
            }
            *image = placeholder;
            report.omitted_images += 1;
        }
    }
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
