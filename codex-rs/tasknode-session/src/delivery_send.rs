//! Synthetic one-event experiment, excluded from normal builds by lib.rs.
//! No production constructor, authority source, credential loader or network adapter.

use crate::ActiveSession;
use crate::SessionScope;
use crate::client::Client;
use crate::client::Response;
use crate::client::normalize_origin;
use crate::delivery_goal;
use crate::delivery_goal::Preparation;
use crate::delivery_goal::Selection;
use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;

const ENDPOINT: &str = "/api/terminal/tasknode/campaign-tracker/events";

// Explicit default scope is distinct from any named profile. This binding does
// not prove where a credential came from: every constructor lives in fixtures.
#[derive(Clone, PartialEq, Eq)]
struct Binding {
    invocation: String,
    profile: SessionScope,
    account: String,
    origin: String,
    client_identity: String,
    event_id: String,
    payload_sha256: String,
    sprint: String,
    workspace: String,
    tasks: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fact {
    Identity,
    Enrollment,
    Ownership,
    SupportedLifecycle,
    Entitlement,
    OneEventAuthorization,
    Posting,
}

const REQUIRED: [Fact; 7] = [
    Fact::Identity,
    Fact::Enrollment,
    Fact::Ownership,
    Fact::SupportedLifecycle,
    Fact::Entitlement,
    Fact::OneEventAuthorization,
    Fact::Posting,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Knowledge {
    Verified,
    Unknown,
    Denied,
}

struct FixtureFact {
    kind: Fact,
    binding: Binding,
    knowledge: Knowledge,
}

// No Debug on anything that can contain a session or fixture request/response.
struct Current {
    binding: Binding,
    session: ActiveSession,
    now: DateTime<Utc>,
    cancelled: bool,
    event_json: String,
    facts: Vec<FixtureFact>,
}

#[derive(Debug, PartialEq, Eq)]
enum Hold {
    Cancelled,
    Identity,
    Expired,
    ExpiryUnknown,
    Preparation,
    Stale,
    Preflight,
    Receipt,
    Http(u16),
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    NotAttempted(Hold),
    AlreadyConsumed,
    OutcomeUnknown,
    Held(Hold),
    RelinkCurrentProfile,
    GoalRecorded,
}

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    event_id: String,
    payload_sha256: String,
    state: State,
}

enum ExchangeFailure {
    Timeout,
    Transport,
}

/// A single in-memory exchange. Only sibling fixtures implement this seam;
/// they attach a synthetic API key and may change the injected state in flight.
/// There is intentionally no implementation that calls Client networking.
trait FixtureExchange {
    fn exchange(
        &mut self,
        endpoint: &'static str,
        approved_payload: &Value,
        current: &mut Current,
    ) -> Result<Response, ExchangeFailure>;
}

// Deliberately neither Clone nor Copy. Even a rejected call consumes the local
// attempt. This is not durable exactly-once delivery or permission to retry.
struct Attempt {
    binding: Binding,
    preparation: Preparation,
    prepared_at: DateTime<Utc>,
    consumed: bool,
}

impl Attempt {
    fn identity(&self, current: &Current) -> Result<(), Hold> {
        let bound = &current.binding;
        if bound != &self.binding
            || bound.invocation.trim().is_empty()
            || bound.account.trim().is_empty()
            || bound.profile.profile().is_some_and(|p| p.trim().is_empty())
            || current.session.account_id.as_deref() != Some(bound.account.as_str())
            || current.session.terminal_token.trim().is_empty()
            || !bound.origin.starts_with("https://")
            || normalize_origin(&bound.origin).ok().as_deref() != Some(bound.origin.as_str())
        {
            return Err(Hold::Identity);
        }
        let client =
            Client::for_session(&current.session, &bound.origin).map_err(|_| Hold::Identity)?;
        if client.identity() != bound.client_identity {
            return Err(Hold::Identity);
        }
        if current.session.is_expired_at(current.now) {
            return Err(Hold::Expired);
        }
        // Native is_expired_at deliberately tolerates absent/invalid metadata.
        // This experiment holds it; "not expired" is not validity evidence.
        if current
            .session
            .expires_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .is_none()
        {
            return Err(Hold::ExpiryUnknown);
        }
        Ok(())
    }

    fn ready(&self, current: &Current) -> Result<(), Hold> {
        if current.cancelled {
            return Err(Hold::Cancelled);
        }
        self.identity(current)?;
        if current.now < self.prepared_at {
            return Err(Hold::Stale);
        }
        let b = &current.binding;
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
        .map_err(|_| Hold::Preparation)?;
        if refreshed
            .blockers()
            .contains(&"stale_observation_requires_review")
        {
            return Err(Hold::Stale);
        }
        // Retain the entire advisory snapshot, including unverified live authority.
        if refreshed != self.preparation {
            return Err(Hold::Preparation);
        }
        if current.facts.len() != REQUIRED.len()
            || REQUIRED.iter().any(|kind| {
                current
                    .facts
                    .iter()
                    .filter(|fact| {
                        fact.kind == *kind
                            && fact.knowledge == Knowledge::Verified
                            && fact.binding == self.binding
                    })
                    .count()
                    != 1
            })
        {
            return Err(Hold::Preflight);
        }
        Ok(())
    }

    fn run(&mut self, current: &mut Current, fixture: &mut impl FixtureExchange) -> Outcome {
        let state = if std::mem::replace(&mut self.consumed, true) {
            State::AlreadyConsumed
        } else if let Err(reason) = self.ready(current) {
            State::NotAttempted(reason)
        } else {
            // No callback or ambient read between the final gates and exchange.
            let response = fixture.exchange(ENDPOINT, self.preparation.payload(), current);
            if self.ready(current).is_err() {
                State::OutcomeUnknown
            } else {
                match response {
                    Err(ExchangeFailure::Timeout | ExchangeFailure::Transport) => {
                        State::OutcomeUnknown
                    }
                    Ok(response) => self.receipt(response),
                }
            }
        };
        Outcome {
            event_id: self.binding.event_id.clone(),
            payload_sha256: self.binding.payload_sha256.clone(),
            state,
        }
    }

    fn receipt(&self, response: Response) -> State {
        match response.status {
            401 => State::RelinkCurrentProfile,
            200..=299 => {
                // Do not use Response::is_ok: it permits a missing `ok` field.
                let body = &response.body;
                if body.get("ok") == Some(&Value::Bool(true))
                    && body.get("id").and_then(Value::as_str) == Some(&self.binding.event_id)
                    && body.get("summaryState").and_then(Value::as_str) == Some("not_applicable")
                {
                    State::GoalRecorded
                } else {
                    State::Held(Hold::Receipt)
                }
            }
            status => State::Held(Hold::Http(status)),
        }
    }
}

#[path = "delivery_send_tests.rs"]
mod tests;

#[path = "delivery_adapter.rs"]
mod delivery_adapter;

#[path = "delivery_reconcile.rs"]
mod delivery_reconcile;
