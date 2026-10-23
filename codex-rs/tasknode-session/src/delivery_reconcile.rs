//! Offline observation of one uncertain goal; no authority or attempt mutation.
use super::*;
use reqwest::Method;
use serde::Deserialize;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use sha2::Digest;
use sha2::Sha256;

const ACTIVITY: &str = "/api/terminal/tasknode/campaign-tracker/activity";

// Never derive Debug for credential-bearing requests or injected content.
struct RawActivity {
    binding: Binding,
    method: Method,
    path: String,
    status: u16,
    bytes: Vec<u8>,
}

struct ExactObservation {
    request: reqwest::blocking::Request,
}

#[derive(Debug, PartialEq, Eq)]
enum Diagnostic {
    NotUncertain,
    Fence(Hold),
    Unrelated,
    Build,
    NotFoundUncertain,
    Http(u16),
    Decoder,
    Shape,
    Annotations,
    Conflict,
}

fn activity_path(binding: &Binding) -> String {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("accountId", &binding.account)
        .append_pair("id", &binding.event_id)
        .finish();
    format!("{ACTIVITY}?{query}")
}

fn reconcile(
    attempt: &Attempt,
    uncertain: &Outcome,
    current: &Current,
    raw: &RawActivity,
) -> Result<ExactObservation, Diagnostic> {
    if !attempt.consumed
        || uncertain.state != State::OutcomeUnknown
        || uncertain.event_id != attempt.binding.event_id
        || uncertain.payload_sha256 != attempt.binding.payload_sha256
    {
        return Err(Diagnostic::NotUncertain);
    }
    if current.cancelled {
        return Err(Diagnostic::Fence(Hold::Cancelled));
    }
    attempt.identity(current).map_err(Diagnostic::Fence)?;
    if current.now < attempt.prepared_at {
        return Err(Diagnostic::Fence(Hold::Stale));
    }
    let b = &attempt.binding;
    let tasks = b.tasks.iter().map(String::as_str).collect::<Vec<_>>();
    let refreshed = delivery_goal::prepare(
        &current.event_json,
        Selection {
            event_id: &b.event_id,
            payload_sha256: &b.payload_sha256,
            sprint_id: &b.sprint,
            workspace_id: &b.workspace,
            task_ids: &tasks,
        },
        current.now,
    )
    .map_err(|_| Diagnostic::Fence(Hold::Preparation))?;
    if refreshed
        .blockers()
        .contains(&"stale_observation_requires_review")
    {
        return Err(Diagnostic::Fence(Hold::Stale));
    }
    if refreshed != attempt.preparation {
        return Err(Diagnostic::Fence(Hold::Preparation));
    }
    // Read observations do not derive or require new write-authorization facts.
    let path = activity_path(b);
    if raw.binding != *b || raw.method != Method::GET || raw.path != path {
        return Err(Diagnostic::Unrelated);
    }
    let client = Client::for_session(&current.session, &b.origin)
        .map_err(|_| Diagnostic::Fence(Hold::Identity))?;
    let request = client
        .build_fixture_request(Method::GET, &path, /*body*/ None)
        .map_err(|_| Diagnostic::Build)?;
    match raw.status {
        // Expiry, visibility filters and tombstones prevent absence inference.
        404 => return Err(Diagnostic::NotFoundUncertain),
        200..=299 => {}
        status => return Err(Diagnostic::Http(status)),
    }
    if raw.bytes.len() > 16 * 1024 * 1024 {
        return Err(Diagnostic::Decoder);
    }
    let response =
        Client::decode_fixture(raw.status, &raw.bytes).map_err(|_| Diagnostic::Decoder)?;
    // The native Value decoder accepts duplicate keys. Reject them recursively
    // before comparing its projection, including conflicts hidden in metadata.
    serde_json::from_slice::<UniqueKeys>(&raw.bytes).map_err(|_| Diagnostic::Decoder)?;
    let body = &response.body;
    let items = body
        .get("items")
        .and_then(Value::as_array)
        .ok_or(Diagnostic::Shape)?;
    if body.as_object().map(serde_json::Map::len) != Some(4)
        || body.get("ok") != Some(&Value::Bool(true))
        || body.get("coverage").and_then(Value::as_str) != Some("observed_tui")
        || body.get("nextCursor") != Some(&Value::Null)
        || items.len() != 1
    {
        return Err(Diagnostic::Shape);
    }
    let mut event = items[0].as_object().cloned().ok_or(Diagnostic::Shape)?;
    let expected = &attempt.preparation.payload()["event"];
    let source_digest = format!(
        "{:x}",
        Sha256::digest(expected["content"].as_str().unwrap().as_bytes())
    );
    let capabilities = event
        .remove("capabilities")
        .ok_or(Diagnostic::Annotations)?;
    let mut caps = capabilities
        .as_array()
        .ok_or(Diagnostic::Annotations)?
        .clone();
    caps.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    let received = event.remove("receivedAt").ok_or(Diagnostic::Annotations)?;
    let valid_received = received.as_str().is_some_and(|s| {
        DateTime::parse_from_rfc3339(s).is_ok_and(|date| {
            date.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string() == s
                && date.timestamp_subsec_nanos() < 1_000_000_000
        })
    });
    if event.remove("accountId") != Some(Value::String(b.account.clone()))
        || !event.remove("revision").is_some_and(|v| {
            v.as_u64()
                .is_some_and(|n| (1..=9_007_199_254_740_991).contains(&n))
        })
        || event.remove("summaryState") != Some(Value::String("not_applicable".into()))
        || !valid_received
        || caps
            != serde_json::json!(["export", "prompt", "replay", "review", "summary"])
                .as_array()
                .unwrap()
                .clone()
        || !event
            .remove("handleAtExecution")
            .is_some_and(|v| v.is_string())
        || event.remove("summary") != Some(Value::Null)
        || event.remove("facts") != Some(serde_json::json!({}))
        || event.remove("model") != Some(Value::String(String::new()))
        || event.remove("sourceDigest") != Some(Value::String(source_digest))
    {
        return Err(Diagnostic::Annotations);
    }
    // Only server-added fields were removed. Missing, extra or changed original
    // fields cannot be manufactured by this equality check.
    if Value::Object(event) != *expected {
        return Err(Diagnostic::Conflict);
    }
    Ok(ExactObservation { request })
}

// Validate key uniqueness without retaining or exporting any decoded content.
struct UniqueKeys;

impl<'de> Deserialize<'de> for UniqueKeys {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeysVisitor;
        impl<'de> Visitor<'de> for KeysVisitor {
            type Value = UniqueKeys;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("JSON with unique object keys")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<UniqueKeys, A::Error> {
                let mut keys = std::collections::HashSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(serde::de::Error::custom("duplicate key"));
                    }
                    map.next_value::<UniqueKeys>()?;
                }
                Ok(UniqueKeys)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<UniqueKeys, A::Error> {
                while seq.next_element::<UniqueKeys>()?.is_some() {}
                Ok(UniqueKeys)
            }
            fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<UniqueKeys, E> {
                Ok(UniqueKeys)
            }
            fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<UniqueKeys, E> {
                Ok(UniqueKeys)
            }
            fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<UniqueKeys, E> {
                Ok(UniqueKeys)
            }
            fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<UniqueKeys, E> {
                Ok(UniqueKeys)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<UniqueKeys, E> {
                Ok(UniqueKeys)
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<UniqueKeys, E> {
                Ok(UniqueKeys)
            }
        }
        deserializer.deserialize_any(KeysVisitor)
    }
}

#[path = "delivery_reconcile_tests.rs"]
mod tests;
