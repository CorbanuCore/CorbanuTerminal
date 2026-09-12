//! Offline PF-80 preparation only. No production caller or transport capability.
#![allow(dead_code)] // Deliberately private until a separately allocated consumer exists.

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use std::fmt::Write;

/// Caller-selected identity, full payload digest and expected destination.
/// These are comparison inputs, not evidence of account ownership or approval.
pub(crate) struct Selection<'a> {
    pub event_id: &'a str,
    /// SHA-256 of Python json.dumps({"event": event}, sort_keys=True), UTF-8.
    pub payload_sha256: &'a str,
    pub sprint_id: &'a str,
    pub workspace_id: &'a str,
    pub task_ids: &'a [&'a str],
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PreparationError {
    Schema,
    Identity,
    PayloadDigest,
    Mapping,
    HistoricalSource,
    UnsafeText,
    Timestamp,
}

/// Immutable advisory snapshot. This type grants no ability to send anything.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Preparation {
    payload: Value,
    payload_sha256: String,
    blockers: Vec<&'static str>,
}

impl Preparation {
    pub(crate) fn payload(&self) -> &Value {
        &self.payload
    }

    pub(crate) fn payload_sha256(&self) -> &str {
        &self.payload_sha256
    }

    pub(crate) fn blockers(&self) -> &[&'static str] {
        &self.blockers
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Event {
    id: String,
    instance_id: String,
    session_id: String,
    turn_id: String,
    sequence: u64,
    workspace_id: String,
    kind: String,
    occurred_at: String,
    content: String,
    task_ids: Vec<String>,
    coverage: String,
    goal: Goal,
    repository: Repository,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Goal {
    id: String,
    status: String,
    active: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Repository {
    label: String,
    branch: String,
    commit: String,
}

/// Validate exactly one supplied event without reading any ambient state.
/// Accepts the canonical millisecond UTC timestamps emitted by Python event_for.
/// Queue state, posting, enrollment and live authority require a later allocation.
pub(crate) fn prepare(
    event_json: &str,
    selection: Selection<'_>,
    now: DateTime<Utc>,
) -> Result<Preparation, PreparationError> {
    use PreparationError as Error;
    if event_json.len() > 1024 * 1024 {
        return Err(Error::Schema);
    }
    // Direct typed decoding rejects duplicate keys as well as unknown fields.
    let event: Event = serde_json::from_str(event_json).map_err(|_| Error::Schema)?;
    if !selection.event_id.starts_with("cc-")
        || !hex(&selection.event_id[3..], 64)
        || event.id != selection.event_id
    {
        return Err(Error::Identity);
    }
    let mut value = serde_json::to_value(&event).map_err(|_| Error::Schema)?;
    value.as_object_mut().ok_or(Error::Schema)?.remove("id");
    if format!("cc-{}", digest(&value)) != selection.event_id {
        return Err(Error::Identity);
    }
    value["id"] = Value::String(event.id.clone());
    let payload = serde_json::json!({"event": value});
    let payload_sha256 = digest(&payload);
    if !hex(selection.payload_sha256, 64) || payload_sha256 != selection.payload_sha256 {
        return Err(Error::PayloadDigest);
    }
    let sprint = event.turn_id.as_bytes();
    if event.instance_id != "corbanu-control"
        || event.kind != "goal"
        || event.sequence > (1_u64 << 53) - 1
        || sprint.len() != 9
        || &sprint[..3] != b"PF-"
        || &sprint[5..7] != b"-S"
        || !sprint[3..5].iter().all(u8::is_ascii_digit)
        || !sprint[7..9].iter().all(u8::is_ascii_digit)
        || event.goal.id != event.turn_id
        || !matches!(
            event.goal.status.as_str(),
            "reported_working"
                | "reported_blocked"
                | "reported_awaiting_review"
                | "reported_finished"
                | "reported_failed"
                | "reported_cancelled"
        )
        || event.coverage != "manager_observed_worker_report_not_independent_acceptance"
        || event.repository.label != "Corbanu Terminal"
        || !hex(&event.repository.commit, 40)
    {
        return Err(Error::Schema);
    }
    if event.turn_id == "PF-76-S01" {
        return Err(Error::HistoricalSource);
    }
    for id in [&event.session_id, &event.workspace_id] {
        if !identifier(id) {
            return Err(Error::Schema);
        }
    }
    if event.task_ids.is_empty()
        || event.task_ids.len() > 32
        || !event.task_ids.iter().all(|id| identifier(id))
        || !event.task_ids.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(Error::Schema);
    }
    let mut expected_tasks = selection.task_ids.to_vec();
    expected_tasks.sort_unstable();
    expected_tasks.dedup();
    if event.turn_id != selection.sprint_id
        || event.workspace_id != selection.workspace_id
        || event.task_ids != expected_tasks
    {
        return Err(Error::Mapping);
    }
    safe_text(&event.content, 2000, 2048)?;
    safe_text(&event.repository.branch, 300, 500)?;
    safe_text(&python_json(&payload["event"]), 12000, 12000)?;
    // No normalization of identity-bearing timestamps, including leap seconds.
    let occurred =
        DateTime::parse_from_rfc3339(&event.occurred_at).map_err(|_| Error::Timestamp)?;
    if event.occurred_at.len() != 24
        || occurred.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string() != event.occurred_at
        || occurred.timestamp_subsec_nanos() >= 1_000_000_000
        || occurred.signed_duration_since(now) > chrono::Duration::minutes(5)
    {
        return Err(Error::Timestamp);
    }
    let mut blockers = vec!["live_authority_entitlement_owner_and_target_lifecycle_unverified"];
    if now.signed_duration_since(occurred) > chrono::Duration::minutes(45) {
        blockers.push("stale_observation_requires_review");
    }
    Ok(Preparation {
        payload,
        payload_sha256,
        blockers,
    })
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c))
}

fn hex(value: &str, size: usize) -> bool {
    value.len() == size
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn safe_text(value: &str, characters: usize, bytes: usize) -> Result<(), PreparationError> {
    if value.len() > bytes
        || value.chars().count() > characters
        || value.chars().any(|c| c < ' ' && c != '\n' && c != '\t')
    {
        return Err(PreparationError::UnsafeText);
    }
    // Conservative subset of control.SECRET, without adding a regex dependency.
    // Hold even embedded token prefixes, empty credential assignments and PRIVATE KEY
    // markers. These extra rejections are intentional; this is not full regex parity.
    let lower = value
        .replace('İ', "i")
        .to_lowercase()
        .replace('ſ', "s")
        .replace('ı', "i");
    if lower.contains("private key") {
        return Err(PreparationError::UnsafeText);
    }
    for (offset, _) in lower.char_indices() {
        let tail = &lower[offset..];
        if ["sk-", "ghp_", "github_pat_"].iter().any(|prefix| {
            tail.strip_prefix(prefix).is_some_and(|s| {
                s.bytes()
                    .take_while(|c| c.is_ascii_alphanumeric() || b"_-".contains(c))
                    .count()
                    >= 12
            })
        }) {
            return Err(PreparationError::UnsafeText);
        }
        if tail
            .strip_prefix("bearer")
            .is_some_and(|s| s.starts_with(char::is_whitespace))
        {
            return Err(PreparationError::UnsafeText);
        }
        for name in ["password", "api", "access", "refresh", "secret"] {
            if let Some(mut suffix) = tail.strip_prefix(name) {
                if matches!(name, "api" | "access" | "refresh") {
                    suffix = suffix.strip_prefix(['_', ' ', '-']).unwrap_or(suffix);
                    let Some(rest) =
                        suffix.strip_prefix(if name == "api" { "key" } else { "token" })
                    else {
                        continue;
                    };
                    suffix = rest;
                }
                if suffix.trim_start().starts_with([':', '=']) {
                    return Err(PreparationError::UnsafeText);
                }
            }
        }
    }
    Ok(())
}

fn digest(value: &Value) -> String {
    format!("{:x}", Sha256::digest(python_json(value).as_bytes()))
}

/// Python's default ensure_ascii=True and separators=(', ', ': '), sorted keys.
/// Only called after typed decoding: no floats, non-string keys or null fields.
fn python_json(value: &Value) -> String {
    match value {
        Value::String(value) => {
            let mut output = String::from("\"");
            for unit in value.encode_utf16() {
                match unit {
                    8 => output.push_str("\\b"),
                    9 => output.push_str("\\t"),
                    10 => output.push_str("\\n"),
                    12 => output.push_str("\\f"),
                    13 => output.push_str("\\r"),
                    34 => output.push_str("\\\""),
                    92 => output.push_str("\\\\"),
                    32..=126 => output.push(char::from(unit as u8)),
                    _ => {
                        let _ = write!(output, "\\u{unit:04x}");
                    }
                }
            }
            output.push('"');
            output
        }
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(python_json)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            let pairs = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}: {}",
                        python_json(&Value::String(key.clone())),
                        python_json(&values[key])
                    )
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", pairs.join(", "))
        }
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Null => "null".into(),
    }
}

#[cfg(test)]
#[path = "delivery_goal_tests.rs"]
mod tests;
