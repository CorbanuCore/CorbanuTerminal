use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn merge(value: Value, current: Option<TokenUsage>) -> TokenUsage {
    serde_json::from_value::<AnthropicUsage>(value)
        .expect("usage")
        .merge_into(current)
}

#[test]
fn compatible_fable_fixture_preserves_existing_totals() {
    // Synthetic versioned dialect fixture from goodalexander #28,
    // e5ddf72293301386d8859093d809a505856801b7. Not live provider evidence.
    let usage = merge(
        json!({
            "input_tokens": 2, "cache_creation_input_tokens": 75,
            "cache_read_input_tokens": 137191, "output_tokens": 30,
            "output_tokens_details": {"thinking_tokens": 9},
        }),
        /*current*/ None,
    );
    assert_eq!(
        usage,
        TokenUsage {
            input_tokens: 137268,
            cached_input_tokens: 137191,
            cache_write_input_tokens: 75,
            output_tokens: 30,
            reasoning_output_tokens: 9,
            total_tokens: 137298,
            codex_rollout_budget_units: None,
        }
    );
}

#[test]
fn aliases_are_breakdowns_not_additional_output() {
    for (details, expected) in [
        (json!({"thinking_tokens": 9}), 9),
        (json!({"reasoning_tokens": 9}), 9),
        (json!({"thinking_tokens": 9, "reasoning_tokens": 9}), 9),
        (json!({"thinking_tokens": 9, "reasoning_tokens": 11}), 11),
        (json!({"thinking_tokens": 0}), 0),
        (json!({"thinking_tokens": 30}), 30),
    ] {
        let mut actual = merge(
            json!({"output_tokens": 30, "output_tokens_details": details}),
            /*current*/ None,
        );
        assert_eq!(actual.reasoning_output_tokens, expected);
        actual.reasoning_output_tokens = 0;
        assert_eq!(
            actual,
            merge(json!({"output_tokens": 30}), /*current*/ None)
        );
    }
}

#[test]
fn missing_null_and_invalid_detail_preserve_prior_valid_breakdown() {
    let initial = merge(
        json!({"output_tokens": 30, "output_tokens_details": {"thinking_tokens": 9}}),
        /*current*/ None,
    );
    for details in [
        Value::Null,
        json!({}),
        json!({"thinking_tokens": null}),
        json!("unsupported"),
        json!({"thinking_tokens": -1}),
        json!({"thinking_tokens": 1.5}),
        json!({"thinking_tokens": "12"}),
        json!({"thinking_tokens": true}),
        json!({"thinking_tokens": 18446744073709551615_u64}),
        json!({"thinking_tokens": 31}),
    ] {
        assert_eq!(
            merge(
                json!({"output_tokens_details": details}),
                Some(initial.clone())
            ),
            initial
        );
    }
    assert_eq!(merge(json!({}), Some(initial.clone())), initial);
}

#[test]
fn cumulative_repetitions_corrections_and_request_isolation() {
    let mut current = None;
    for (output, reasoning) in [(40, 10), (80, 25), (100, 30), (100, 30), (100, 0)] {
        let usage = merge(
            json!({"output_tokens": output, "output_tokens_details": {"reasoning_tokens": reasoning}}),
            current,
        );
        assert_eq!(usage.output_tokens, output);
        assert_eq!(usage.reasoning_output_tokens, reasoning);
        assert_eq!(usage.total_tokens, output);
        current = Some(usage);
    }
    assert_eq!(
        merge(json!({"output_tokens": 100}), /*current*/ None).reasoning_output_tokens,
        0
    );
}

#[test]
fn invalid_subset_never_increases_output_or_survives_a_smaller_total() {
    let initial = merge(
        json!({"output_tokens": 100, "output_tokens_details": {"thinking_tokens": 30}}),
        /*current*/ None,
    );
    let smaller = merge(json!({"output_tokens": 20}), Some(initial));
    assert_eq!(
        smaller,
        TokenUsage {
            output_tokens: 20,
            total_tokens: 20,
            ..Default::default()
        }
    );
    let later = merge(
        json!({"output_tokens_details": {"thinking_tokens": 10}}),
        Some(smaller),
    );
    assert_eq!(later.reasoning_output_tokens, 10);
}

#[test]
fn cumulative_input_and_accounting_observer_are_unchanged() {
    let initial = merge(
        json!({"input_tokens": 9, "cache_creation_input_tokens": 5,
        "cache_read_input_tokens": 7, "output_tokens": 30,
        "output_tokens_details": {"thinking_tokens": 9}}),
        /*current*/ None,
    );
    let actual = merge(
        json!({"input_tokens": 21, "output_tokens": 40}),
        Some(initial),
    );
    assert_eq!(
        actual,
        TokenUsage {
            input_tokens: 21,
            cached_input_tokens: 7,
            cache_write_input_tokens: 5,
            output_tokens: 40,
            reasoning_output_tokens: 9,
            total_tokens: 61,
            codex_rollout_budget_units: None,
        }
    );
    let plain = json!({"usage": {"input_tokens": 9, "output_tokens": 30}});
    let with_details = json!({"usage": {"input_tokens": 9, "output_tokens": 30,
        "output_tokens_details": {"thinking_tokens": 9}}});
    assert_eq!(
        accounting::body_usage(&serde_json::to_vec(&plain).expect("json")),
        accounting::body_usage(&serde_json::to_vec(&with_details).expect("json")),
    );
}

#[tokio::test]
async fn stream_completion_preserves_reasoning_when_final_delta_omits_it() {
    let events = super::tests::collect_events(&[
        br#"event: message_start
data: {"type":"message_start","message":{"id":"reasoning-fixture","usage":{"input_tokens":2,"output_tokens":30,"output_tokens_details":{"thinking_tokens":9}}}}

"#,
        br#"event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

"#,
        br#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"OK"}}

"#,
        br#"event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":40}}

"#,
        br#"event: message_stop
data: {"type":"message_stop"}

"#,
    ]).await;
    let completed = events
        .iter()
        .find_map(|event| match event {
            Ok(ResponseEvent::Completed { token_usage, .. }) => token_usage.as_ref(),
            _ => None,
        })
        .expect("completed usage");
    assert_eq!(
        completed,
        &TokenUsage {
            input_tokens: 2,
            output_tokens: 40,
            reasoning_output_tokens: 9,
            total_tokens: 42,
            ..Default::default()
        }
    );
}
