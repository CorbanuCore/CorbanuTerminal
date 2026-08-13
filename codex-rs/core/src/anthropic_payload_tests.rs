use super::ANTHROPIC_MESSAGES_REQUEST_BUDGET_BYTES;
use super::PayloadBudgetReport;
use super::enforce_anthropic_payload_budget;
use codex_api::AnthropicMessagesRequest;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

fn request(messages: Vec<Value>) -> AnthropicMessagesRequest {
    AnthropicMessagesRequest {
        model: "claude-opus-5".to_string(),
        system: Vec::new(),
        messages,
        tools: Vec::new(),
        tool_choice: None,
        stream: true,
        max_tokens: 1_024,
        thinking: None,
        output_config: None,
        provider_options: None,
    }
}

fn image(data: &str) -> Value {
    json!({
        "type": "image",
        "source": {
            "type": "base64",
            "media_type": "image/png",
            "data": data,
        }
    })
}

fn count_base64_images(value: &Value) -> usize {
    let current = usize::from(
        value.get("type").and_then(Value::as_str) == Some("image")
            && value.pointer("/source/type").and_then(Value::as_str) == Some("base64"),
    );
    current
        + match value {
            Value::Array(values) => values.iter().map(count_base64_images).sum(),
            Value::Object(object) => object.values().map(count_base64_images).sum(),
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 0,
        }
}

#[test]
fn request_under_budget_is_unchanged() {
    let original_messages = vec![json!({
        "role": "user",
        "content": [
            {"type": "text", "text": "inspect"},
            image("cG5n"),
        ],
    })];
    let mut request = request(original_messages.clone());

    let report =
        enforce_anthropic_payload_budget(&mut request, ANTHROPIC_MESSAGES_REQUEST_BUDGET_BYTES)
            .expect("request should fit");

    assert_eq!(
        report,
        PayloadBudgetReport {
            original_bytes: report.original_bytes,
            final_bytes: report.original_bytes,
            omitted_images: 0,
        }
    );
    assert_eq!(request.messages, original_messages);
}

#[test]
fn oversized_history_omits_oldest_images_and_retains_newest() {
    let old_data = "a".repeat(1_000);
    let middle_data = "b".repeat(1_000);
    let newest_data = "c".repeat(1_000);
    let mut request = request(vec![
        json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "old"},
                image(&old_data),
            ],
        }),
        json!({
            "role": "assistant",
            "content": [{"type": "text", "text": "checked"}],
        }),
        json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": "toolu_view",
                "content": [
                    {"type": "text", "text": "middle"},
                    image(&middle_data),
                ],
            }],
        }),
        json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "newest"},
                image(&newest_data),
            ],
        }),
    ]);
    let original_bytes = serde_json::to_vec(&request).expect("serialize").len();
    let max_bytes = original_bytes - old_data.len() - middle_data.len() + 500;

    let report = enforce_anthropic_payload_budget(&mut request, max_bytes)
        .expect("request should be pruned");
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(report.original_bytes, original_bytes);
    assert_eq!(report.omitted_images, 2);
    assert!(report.final_bytes <= max_bytes);
    assert_eq!(count_base64_images(&body), 1);
    assert_eq!(
        body.pointer("/messages/0/content/1/type"),
        Some(&json!("text"))
    );
    assert_eq!(
        body.pointer("/messages/2/content/0/content/1/type"),
        Some(&json!("text"))
    );
    assert_eq!(
        body.pointer("/messages/3/content/1/source/data"),
        Some(&json!(newest_data))
    );
}

#[test]
fn omission_preserves_cache_control_marker() {
    let mut cached_image = image(&"a".repeat(2_000));
    cached_image.as_object_mut().expect("image object").insert(
        "cache_control".to_string(),
        json!({"type": "ephemeral", "ttl": "1h"}),
    );
    let mut request = request(vec![json!({
        "role": "user",
        "content": [cached_image],
    })]);

    let report = enforce_anthropic_payload_budget(&mut request, /*max_bytes*/ 1_000)
        .expect("request should be pruned");
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(report.omitted_images, 1);
    assert_eq!(
        body.pointer("/messages/0/content/0/cache_control"),
        Some(&json!({"type": "ephemeral", "ttl": "1h"}))
    );
}

#[test]
fn request_that_cannot_fit_without_images_returns_actionable_error() {
    let mut request = request(vec![json!({
        "role": "user",
        "content": [{"type": "text", "text": "x".repeat(2_000)}],
    })]);

    let error = enforce_anthropic_payload_budget(&mut request, /*max_bytes*/ 500)
        .expect_err("text-only request cannot be reduced");

    assert!(error.to_string().contains("compact the conversation"));
}

#[test]
fn fifteen_visual_turns_stay_below_anthropic_messages_limit() {
    let image_data = "v".repeat(2_400_000);
    let messages = (0..15)
        .map(|index| {
            json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": format!("visual inspection {index}")},
                    image(&image_data),
                ],
            })
        })
        .collect();
    let mut request = request(messages);

    let report =
        enforce_anthropic_payload_budget(&mut request, ANTHROPIC_MESSAGES_REQUEST_BUDGET_BYTES)
            .expect("accumulated visual history should be pruned");
    let body = serde_json::to_value(&request).expect("serialize request");

    assert!(report.original_bytes > 32_000_000);
    assert!(report.final_bytes <= ANTHROPIC_MESSAGES_REQUEST_BUDGET_BYTES);
    assert!(report.omitted_images > 0);
    assert!(count_base64_images(&body) > 0);
    assert_eq!(
        body.pointer("/messages/14/content/1/source/data"),
        Some(&json!(image_data)),
        "the newest visual evidence must survive request-local pruning"
    );
}
