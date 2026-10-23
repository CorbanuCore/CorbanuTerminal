//! Pinned-schema experiment. All bytes, operator policy and credentials are synthetic.
//! Private child of the test-only engine; requests are built, never executed.
use super::*;
use reqwest::Method;
use sha2::Digest;
use sha2::Sha256;

const STATUS: &str = "/api/terminal/tasknode/status";
const ENROLLMENT: &str = "/api/terminal/tasknode/campaign-tracker/status";
const GATEWAY: &str = "/v1/account";

#[derive(Clone)]
struct Raw {
    binding: Binding,
    method: Method,
    path: String,
    key_handle: Option<String>,
    status: u16,
    bytes: Vec<u8>,
}

struct Operator {
    binding: Binding,
    supported_statuses: Vec<String>,
    one_event: Knowledge,
    posting: Knowledge,
}

fn key_handle(key: &str) -> String {
    format!("{:x}", Sha256::digest(key.as_bytes()))
}

fn task_path(id: &str) -> String {
    // Match encodeURIComponent's segment semantics, never form-query '+' encoding.
    let encoded = url::form_urlencoded::byte_serialize(id.as_bytes())
        .collect::<String>()
        .replace('+', "%20");
    format!("/api/terminal/tasknode/tasks/{encoded}")
}

fn observation(
    raw: &[Raw],
    binding: &Binding,
    path: &str,
    key: Option<&str>,
) -> Result<Value, Hold> {
    let mut matches = raw.iter().filter(|item| item.path == path);
    let item = matches.next().ok_or(Hold::Preflight)?;
    if matches.next().is_some()
        || item.binding != *binding
        || item.method != Method::GET
        || item.key_handle.as_deref() != key
        || !(200..300).contains(&item.status)
    {
        return Err(Hold::Preflight);
    }
    let body = Client::decode_fixture(item.status, &item.bytes)
        .map_err(|_| Hold::Preflight)?
        .body;
    if key.is_none() && body.get("ok") != Some(&Value::Bool(true)) {
        return Err(Hold::Preflight);
    }
    Ok(body)
}

fn entitlement(account: &Value, now: DateTime<Utc>) -> bool {
    // Consumer schema only: no assertion that gateway and Task Node account IDs match.
    // Support RFC3339 periods and exact decimal strings / JS-safe JSON integers.
    let period = account
        .get("period")
        .filter(|v| !v.is_null())
        .or_else(|| account.pointer("/legacy/period"));
    let date = |field| {
        period
            .and_then(|p| p.get(field))
            .and_then(Value::as_str)
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    };
    let active = date("startsAt")
        .zip(date("endsAt"))
        .is_some_and(|(start, end)| start <= now && now < end);
    let funded = account
        .pointer("/corbanuApi/balanceMicrousd")
        .is_some_and(|v| {
            if let Some(decimal) = v.as_str() {
                !decimal.is_empty()
                    && decimal.bytes().all(|b| b.is_ascii_digit())
                    && decimal.bytes().any(|b| b != b'0')
            } else {
                v.as_u64()
                    .is_some_and(|n| n > 0 && n <= 9_007_199_254_740_991)
            }
        });
    active || funded
}

fn derive_facts(
    current: &Current,
    raw: &[Raw],
    operator: &Operator,
    key: &str,
) -> Result<Vec<FixtureFact>, Hold> {
    let b = &current.binding;
    let client = Client::for_session(&current.session, &b.origin).map_err(|_| Hold::Identity)?;
    if b.account.trim().is_empty()
        || client.identity() != b.client_identity
        || current.session.account_id.as_deref() != Some(&b.account)
        || normalize_origin(&b.origin).ok().as_deref() != Some(&b.origin)
        || !b.origin.starts_with("https://")
        || operator.binding != *b
        || key.trim().is_empty()
        || key == current.session.terminal_token
        || raw.len() != b.tasks.len() + 3
    {
        return Err(Hold::Preflight);
    }
    for path in [STATUS, ENROLLMENT] {
        let body = observation(raw, b, path, /*key*/ None)?;
        if body.get("accountId").and_then(Value::as_str) != Some(&b.account) {
            return Err(Hold::Preflight);
        }
        if path == ENROLLMENT {
            let rows = body
                .get("enrollments")
                .and_then(Value::as_array)
                .ok_or(Hold::Preflight)?;
            let mut seen = std::collections::HashSet::new();
            for row in rows {
                let workspace = row
                    .get("workspace_id")
                    .and_then(Value::as_str)
                    .ok_or(Hold::Preflight)?;
                if !seen.insert(workspace) || row.get("enabled").and_then(Value::as_bool).is_none()
                {
                    return Err(Hold::Preflight);
                }
            }
            if !rows
                .iter()
                .any(|row| row["workspace_id"] == b.workspace && row["enabled"] == true)
            {
                return Err(Hold::Preflight);
            }
        }
    }
    let mut supported = true;
    for id in &b.tasks {
        let body = observation(raw, b, &task_path(id), /*key*/ None)?;
        let task = &body["task"];
        if task.get("fullId").and_then(Value::as_str) != Some(id)
            || task.get("taskId").and_then(Value::as_str) != Some(id)
        {
            return Err(Hold::Preflight);
        }
        // Ownership follows the pinned authenticated account + wallet query, not task.id.
        let status = task
            .get("statusKey")
            .and_then(Value::as_str)
            .ok_or(Hold::Preflight)?;
        if !matches!(
            status,
            "accepted"
                | "cancelled"
                | "expired"
                | "proposed"
                | "refused"
                | "rejected"
                | "reward_decided"
                | "rewarded"
                | "submitted"
                | "verification_requested"
                | "verification_response_submitted"
        ) {
            return Err(Hold::Preflight);
        }
        supported &= operator.supported_statuses.iter().any(|s| s == status);
    }
    let gateway = observation(raw, b, GATEWAY, Some(&key_handle(key)))?;
    if !entitlement(&gateway, current.now) {
        return Err(Hold::Preflight);
    }
    Ok([
        (Fact::Identity, Knowledge::Verified),
        (Fact::Enrollment, Knowledge::Verified),
        (Fact::Ownership, Knowledge::Verified),
        (Fact::Entitlement, Knowledge::Verified),
        (
            Fact::SupportedLifecycle,
            if supported {
                Knowledge::Verified
            } else {
                Knowledge::Unknown
            },
        ),
        (Fact::OneEventAuthorization, operator.one_event),
        (Fact::Posting, operator.posting),
    ]
    .into_iter()
    .map(|(kind, knowledge)| FixtureFact {
        kind,
        knowledge,
        binding: b.clone(),
    })
    .collect())
}

struct BuildOnly {
    binding: Binding,
    key: String,
    receipt: (u16, Vec<u8>),
    request: Option<reqwest::blocking::Request>,
    during: fn(&mut Current),
}

impl BuildOnly {
    fn prepare(
        current: &mut Current,
        raw: &[Raw],
        operator: &Operator,
        key: String,
        receipt: (u16, Vec<u8>),
    ) -> Result<Self, Hold> {
        current.facts = derive_facts(current, raw, operator, &key)?;
        Ok(Self {
            binding: current.binding.clone(),
            key,
            receipt,
            request: None,
            during: |_| {},
        })
    }
}

impl FixtureExchange for BuildOnly {
    fn exchange(
        &mut self,
        endpoint: &'static str,
        approved_payload: &Value,
        current: &mut Current,
    ) -> Result<Response, ExchangeFailure> {
        if endpoint != ENDPOINT || current.binding != self.binding || self.request.is_some() {
            return Err(ExchangeFailure::Transport);
        }
        let client = Client::for_session(&current.session, &self.binding.origin)
            .map_err(|_| ExchangeFailure::Transport)?;
        let mut wire = approved_payload.clone();
        wire["apiKey"] = Value::String(self.key.clone());
        self.request = Some(
            client
                .build_fixture_request(Method::POST, endpoint, Some(&wire))
                .map_err(|_| ExchangeFailure::Transport)?,
        );
        (self.during)(current);
        Client::decode_fixture(self.receipt.0, &self.receipt.1)
            .map_err(|_| ExchangeFailure::Transport)
    }
}

#[path = "delivery_adapter_tests.rs"]
mod tests;
