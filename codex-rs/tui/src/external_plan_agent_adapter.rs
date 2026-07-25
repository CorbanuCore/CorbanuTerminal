//! Compatibility decoding for plan-backed or text-only agent runtimes.
//!
//! Native agents never use these envelopes internally. This adapter preserves malformed provider
//! output visibly and emits only bounded, mechanical messages for the canonical agent mailbox.

use codex_protocol::protocol::MAX_AGENT_MESSAGE_BYTES;

pub(crate) const SEND_TASK_FENCE_OPEN: &str = "```pfterminal-send-task";
const SEND_TASK_FENCE_CLOSE: &str = "```";
pub(crate) const SEND_TASK_OPEN: &str = "<pfterminal_send_task";
const SEND_TASK_CLOSE: &str = "</pfterminal_send_task>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpawnTaskDispatch {
    pub(crate) target: String,
    pub(crate) task: String,
    pub(crate) seq: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalPlanDecodeFailure {
    MalformedEnvelope,
    MessageTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalPlanDecode {
    pub(crate) visible_text: String,
    pub(crate) dispatches: Vec<SpawnTaskDispatch>,
    pub(crate) failures: Vec<ExternalPlanDecodeFailure>,
}

pub(crate) fn decode_external_plan_messages(text: &str) -> ExternalPlanDecode {
    let mut visible = String::new();
    let mut dispatches = Vec::new();
    let mut failures = Vec::new();
    let mut rest = text;

    loop {
        let next_fence = rest.find(SEND_TASK_FENCE_OPEN);
        let next_xml = rest.find(SEND_TASK_OPEN);
        let Some((start, encoding)) = earliest_envelope(next_fence, next_xml) else {
            visible.push_str(rest);
            break;
        };

        visible.push_str(&rest[..start]);
        let block = &rest[start..];
        let decoded = match encoding {
            EnvelopeEncoding::Fenced => decode_fenced_block(block),
            EnvelopeEncoding::Xmlish => decode_xmlish_block(block),
        };
        if let Some(dispatch) = decoded.dispatch {
            dispatches.push(dispatch);
        } else {
            visible.push_str(&block[..decoded.consumed]);
        }
        if let Some(failure) = decoded.failure {
            failures.push(failure);
        }
        rest = &block[decoded.consumed..];
    }

    ExternalPlanDecode {
        visible_text: visible.trim().to_string(),
        dispatches,
        failures,
    }
}

pub(crate) fn extract_spawn_task_dispatches(text: &str) -> (String, Vec<SpawnTaskDispatch>) {
    let decoded = decode_external_plan_messages(text);
    (decoded.visible_text, decoded.dispatches)
}

pub(crate) fn mentions_unparsed_dispatch(original_text: &str, visible_text: &str) -> bool {
    (original_text.contains(SEND_TASK_OPEN) && visible_text.contains(SEND_TASK_OPEN))
        || (original_text.contains(SEND_TASK_FENCE_OPEN)
            && visible_text.contains(SEND_TASK_FENCE_OPEN))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvelopeEncoding {
    Fenced,
    Xmlish,
}

#[derive(Debug)]
struct DecodedEnvelope {
    consumed: usize,
    dispatch: Option<SpawnTaskDispatch>,
    failure: Option<ExternalPlanDecodeFailure>,
}

fn earliest_envelope(
    fence: Option<usize>,
    xml: Option<usize>,
) -> Option<(usize, EnvelopeEncoding)> {
    match (fence, xml) {
        (Some(fence), Some(xml)) if fence <= xml => Some((fence, EnvelopeEncoding::Fenced)),
        (Some(_), Some(xml)) => Some((xml, EnvelopeEncoding::Xmlish)),
        (Some(fence), None) => Some((fence, EnvelopeEncoding::Fenced)),
        (None, Some(xml)) => Some((xml, EnvelopeEncoding::Xmlish)),
        (None, None) => None,
    }
}

fn malformed_remainder(block: &str) -> DecodedEnvelope {
    DecodedEnvelope {
        consumed: block.len(),
        dispatch: None,
        failure: Some(ExternalPlanDecodeFailure::MalformedEnvelope),
    }
}

fn decode_fenced_block(block: &str) -> DecodedEnvelope {
    let Some(header_end) = block.find('\n') else {
        return malformed_remainder(block);
    };
    let header = &block[..header_end];
    let content_start = header_end + 1;
    let Some(close_index) = block[content_start..].find(SEND_TASK_FENCE_CLOSE) else {
        return malformed_remainder(block);
    };
    let content_end = content_start + close_index;
    let consumed = content_end + SEND_TASK_FENCE_CLOSE.len();
    let dispatch = fenced_dispatch_from_parts(header, &block[content_start..content_end]);
    match dispatch {
        Some(dispatch) if dispatch.task.len() <= MAX_AGENT_MESSAGE_BYTES => DecodedEnvelope {
            consumed,
            dispatch: Some(dispatch),
            failure: None,
        },
        Some(_) => DecodedEnvelope {
            consumed,
            dispatch: None,
            failure: Some(ExternalPlanDecodeFailure::MessageTooLarge),
        },
        None => DecodedEnvelope {
            consumed,
            dispatch: None,
            failure: Some(ExternalPlanDecodeFailure::MalformedEnvelope),
        },
    }
}

fn decode_xmlish_block(block: &str) -> DecodedEnvelope {
    let Some(tag_end) = block.find('>') else {
        return malformed_remainder(block);
    };
    let tag = &block[..=tag_end];
    let content_start = tag_end + 1;
    let Some(close_index) = block[content_start..].find(SEND_TASK_CLOSE) else {
        return malformed_remainder(block);
    };
    let content_end = content_start + close_index;
    let consumed = content_end + SEND_TASK_CLOSE.len();
    let content = block[content_start..content_end].trim();
    let target = xmlish_attr_value(tag, "target");

    if let Some(target) = target.filter(|target| !target.trim().is_empty())
        && !content.is_empty()
        && content.len() <= MAX_AGENT_MESSAGE_BYTES
    {
        DecodedEnvelope {
            consumed,
            dispatch: Some(SpawnTaskDispatch {
                target: target.trim().to_string(),
                task: content.to_string(),
                seq: None,
            }),
            failure: None,
        }
    } else {
        DecodedEnvelope {
            consumed,
            dispatch: None,
            failure: Some(if content.len() > MAX_AGENT_MESSAGE_BYTES {
                ExternalPlanDecodeFailure::MessageTooLarge
            } else {
                ExternalPlanDecodeFailure::MalformedEnvelope
            }),
        }
    }
}

fn fenced_dispatch_from_parts(header: &str, content: &str) -> Option<SpawnTaskDispatch> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content.trim()) {
        let target = value.get("target")?.as_str()?.trim().to_string();
        let task = value.get("task")?.as_str()?.trim().to_string();
        let seq = value.get("seq").and_then(serde_json::Value::as_u64);
        return (!target.is_empty() && !task.is_empty()).then_some(SpawnTaskDispatch {
            target,
            task,
            seq,
        });
    }

    let mut target = yamlish_field_value(header, "target");
    let mut task_lines = Vec::new();
    let mut consumed_task_marker = false;
    for line in content.lines() {
        if target.is_none()
            && let Some(value) = yamlish_field_value(line, "target")
        {
            target = Some(value);
            continue;
        }
        if !consumed_task_marker && let Some(value) = yamlish_field_value(line, "task") {
            consumed_task_marker = true;
            if !value.trim().is_empty() {
                task_lines.push(value);
            }
            continue;
        }
        if task_lines.is_empty() && line.trim().is_empty() {
            continue;
        }
        task_lines.push(line.to_string());
    }

    let target = target?.trim().to_string();
    let task = task_lines.join("\n").trim().to_string();
    (!target.is_empty() && !task.is_empty()).then_some(SpawnTaskDispatch {
        target,
        task,
        seq: None,
    })
}

fn yamlish_field_value(line: &str, field: &str) -> Option<String> {
    let (key, value) = line.split_once(':')?;
    key.trim()
        .eq_ignore_ascii_case(field)
        .then(|| value.trim().to_string())
}

fn xmlish_attr_value(tag: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=");
    let start = tag.find(&needle)? + needle.len();
    let quote = tag[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = tag[value_start..].find(quote)? + value_start;
    Some(tag[value_start..value_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_text_envelopes_normalize_without_leaking_transport_markup() {
        let decoded = decode_external_plan_messages(
            "Before <pfterminal_send_task target=\"worker-a\">Inspect it.</pfterminal_send_task> after\n\
             ```pfterminal-send-task\n{\"target\":\"worker-b\",\"task\":\"Verify it.\",\"seq\":7}\n```",
        );
        assert_eq!(decoded.dispatches.len(), 2);
        assert_eq!(decoded.dispatches[0].target, "worker-a");
        assert_eq!(decoded.dispatches[1].seq, Some(7));
        assert!(!decoded.visible_text.contains("pfterminal_send_task"));
        assert!(decoded.failures.is_empty());
    }

    #[test]
    fn malformed_and_oversized_envelopes_stay_visible() {
        let malformed = decode_external_plan_messages(
            "<pfterminal_send_task>missing target</pfterminal_send_task>",
        );
        assert!(malformed.dispatches.is_empty());
        assert!(malformed.visible_text.contains(SEND_TASK_OPEN));
        assert_eq!(
            malformed.failures,
            vec![ExternalPlanDecodeFailure::MalformedEnvelope]
        );

        let oversized_body = "x".repeat(MAX_AGENT_MESSAGE_BYTES + 1);
        let oversized = decode_external_plan_messages(&format!(
            "<pfterminal_send_task target=\"worker\">{oversized_body}</pfterminal_send_task>"
        ));
        assert!(oversized.dispatches.is_empty());
        assert!(oversized.visible_text.contains(&oversized_body));
        assert_eq!(
            oversized.failures,
            vec![ExternalPlanDecodeFailure::MessageTooLarge]
        );
    }
}
