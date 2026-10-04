use anyhow::Result;
use anyhow::ensure;

// Exact wire fixtures, not a general Responses schema or an allowed live model.
// Preserve these bytes end-to-end. Even equivalent JSON is outside this corpus.
pub(crate) const PACKETS: [&str; 4] = [
    r#"{"model":"synthetic-no-inference","input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"Describe this synthetic image; https://example.invalid is inert text."},{"type":"input_image","image_url":"data:image/png;base64,iVBORw0KGgo=","detail":"low"}]}],"instructions":"Fixture only","tools":[],"tool_choice":"auto","parallel_tool_calls":false,"store":false,"stream":true,"include":[]}"#,
    r#"{"model":"synthetic-no-inference","input":[{"type":"function_call","name":"noop","arguments":"{}","call_id":"fixture-call-1"},{"type":"function_call_output","call_id":"fixture-call-1","output":"fixture result"}],"tools":[{"type":"function","name":"noop","description":"Synthetic no-op","strict":true,"parameters":{"type":"object","properties":{},"additionalProperties":false}},{"type":"namespace","name":"fixture","description":"Local fixture","tools":[{"type":"function","name":"noop","description":"Synthetic no-op","strict":true,"parameters":{"type":"object","properties":{},"additionalProperties":false}}]}],"tool_choice":"auto","parallel_tool_calls":false,"store":false,"stream":true,"include":[]}"#,
    r#"{"model":"synthetic-no-inference","input":[{"type":"custom_tool_call","call_id":"fixture-call-2","name":"fixture_echo","input":"hello"},{"type":"custom_tool_call_output","call_id":"fixture-call-2","output":"hello"}],"tools":[{"type":"custom","name":"fixture_echo","description":"Synthetic local echo","format":{"type":"grammar","syntax":"lark","definition":"start: \"hello\""}}],"tool_choice":"auto","parallel_tool_calls":false,"store":false,"stream":false,"include":[]}"#,
    r#"{"model":"synthetic-no-inference","input":[{"type":"function_call","name":"noop","arguments":"{}","call_id":"fixture-call-3"},{"type":"function_call_output","call_id":"fixture-call-3","output":[{"type":"input_text","text":"fixture result"},{"type":"input_image","image_url":"data:image/png;base64,iVBORw0KGgo=","detail":"low"}]}],"tools":[{"type":"function","name":"noop","description":"Synthetic no-op","strict":true,"parameters":{"type":"object","properties":{},"additionalProperties":false}}],"tool_choice":"auto","parallel_tool_calls":false,"store":false,"stream":true,"include":[],"text":{"verbosity":"low"}}"#,
];

pub(crate) fn admit(bytes: &[u8]) -> Result<()> {
    ensure!(
        PACKETS.iter().any(|packet| packet.as_bytes() == bytes),
        "outside synthetic corpus"
    );
    Ok(())
}

#[cfg(test)]
#[path = "synthetic_policy_tests.rs"]
mod tests;
