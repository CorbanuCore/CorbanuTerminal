//! Private identity-at-check pipeline; no enabled caller or durable validity.
use crate::ActiveSession;
use crate::Client;
use crate::ClientError;
use crate::SessionScope;
use crate::normalize_origin;
use chrono::DateTime;
use chrono::Utc;
use reqwest::Method;
use serde::Deserialize;
use std::future::Future;

const STATUS_PATH: &str = "/api/terminal/tasknode/status";
const RESPONSE_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservationControl {
    Active,
    Cancelled,
    SessionChanged,
}

/// The caller must expose pending relink/unlink/unavailable authority as a hold,
/// and retain cancellation/generation changes even if control later looks active.
/// These supplied fences cannot detect independent native-store writers.
pub(crate) struct ValidityCurrent {
    pub(crate) invocation: String,
    pub(crate) generation: u64,
    pub(crate) scope: SessionScope,
    pub(crate) session: ActiveSession,
    pub(crate) now: DateTime<Utc>,
    pub(crate) control: ObservationControl,
}

/// Consumed by the check, including on failure: an operation cannot retry it.
pub(crate) struct ValidityBinding {
    invocation: String,
    generation: u64,
    scope: SessionScope,
    account: String,
    origin: String,
    identity: String,
    expiry: Option<String>,
    started_at: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ValidityHold {
    #[error("Identity observation cancelled.")]
    Cancelled,
    #[error("Current session authority changed or is unavailable.")]
    SessionChanged,
    #[error("Identity observation binding mismatch.")]
    BindingMismatch,
    #[error("Known session expiry reached.")]
    KnownExpiry,
    #[error("Invalid session expiry metadata.")]
    InvalidExpiry,
    #[error("Identity observation clock moved backward.")]
    BackwardTime,
    #[error("Identity rejected at check time (HTTP {0}).")]
    Http(u16),
    #[error("Identity observation transport failed.")]
    Transport,
    #[error("Identity response unreadable.")]
    Unreadable,
    #[error("Invalid server identity.")]
    InvalidIdentity,
}

impl ValidityBinding {
    pub(crate) fn new(current: &ValidityCurrent) -> Result<Self, ValidityHold> {
        let client = current_client(current)?;
        Ok(Self {
            invocation: current.invocation.clone(),
            generation: current.generation,
            scope: current.scope.clone(),
            account: current
                .session
                .account_id
                .clone()
                .ok_or(ValidityHold::BindingMismatch)?,
            origin: client.origin().to_owned(),
            identity: client.identity(),
            expiry: current.session.expires_at.clone(),
            started_at: current.now,
        })
    }

    fn fence(
        &self,
        current: &ValidityCurrent,
        since: DateTime<Utc>,
    ) -> Result<Client, ValidityHold> {
        let client = current_client(current)?;
        if current.invocation != self.invocation
            || current.generation != self.generation
            || current.scope != self.scope
            || current.session.account_id.as_deref() != Some(self.account.as_str())
            || client.origin() != self.origin
            || client.identity() != self.identity
            || current.session.expires_at != self.expiry
        {
            return Err(ValidityHold::BindingMismatch);
        }
        if current.now < since {
            return Err(ValidityHold::BackwardTime);
        }
        Ok(client)
    }
}

fn current_client(current: &ValidityCurrent) -> Result<Client, ValidityHold> {
    match current.control {
        ObservationControl::Cancelled => return Err(ValidityHold::Cancelled),
        ObservationControl::SessionChanged => return Err(ValidityHold::SessionChanged),
        ObservationControl::Active => {}
    }
    let session = &current.session;
    let origin = normalize_origin(&session.origin).map_err(|_| ValidityHold::BindingMismatch)?;
    if current.invocation.trim().is_empty()
        || current
            .scope
            .profile()
            .is_some_and(|profile| profile.trim().is_empty())
        || session
            .account_id
            .as_deref()
            .is_none_or(|account| account.trim().is_empty())
        || session.terminal_token.trim().is_empty()
        || !origin.starts_with("https://")
        || session.origin != origin
    {
        return Err(ValidityHold::BindingMismatch);
    }
    if let Some(raw) = &session.expires_at {
        let expiry = DateTime::parse_from_rfc3339(raw).map_err(|_| ValidityHold::InvalidExpiry)?;
        if expiry <= current.now {
            return Err(ValidityHold::KnownExpiry);
        }
    }
    Client::for_session(session, &origin).map_err(|_| ValidityHold::BindingMismatch)
}

pub(crate) struct CheckedIdentity {
    binding: ValidityBinding,
    observed_at: DateTime<Utc>,
}

/// Acceptance during the status request only, never permission for a later GET
/// or POST. No TTL, credential, server timestamp, or posting fact is returned.
pub(crate) struct IdentityAtCheck {
    _binding: ValidityBinding,
    pub(crate) observed_at: DateTime<Utc>,
}

impl CheckedIdentity {
    pub(crate) fn consume_for_observation(
        self,
        mut current: impl FnMut() -> Result<ValidityCurrent, ValidityHold>,
    ) -> Result<IdentityAtCheck, ValidityHold> {
        self.binding.fence(&current()?, self.observed_at)?;
        Ok(IdentityAtCheck {
            _binding: self.binding,
            observed_at: self.observed_at,
        })
    }
}

pub(crate) async fn check_identity(
    expected: ValidityBinding,
    current: impl FnMut() -> Result<ValidityCurrent, ValidityHold>,
) -> Result<CheckedIdentity, ValidityHold> {
    check_identity_with(expected, current, |client| async move {
        client
            .request_bytes(Method::GET, STATUS_PATH, /*body*/ None)
            .await
    })
    .await
}

/// One foreground exchange; dropping its future cannot deliver a proof and
/// cannot undo a request that already reached the server. No retry or spawning.
async fn check_identity_with<F: Future<Output = Result<(u16, Vec<u8>), ClientError>>>(
    expected: ValidityBinding,
    mut current: impl FnMut() -> Result<ValidityCurrent, ValidityHold>,
    exchange: impl FnOnce(Client) -> F,
) -> Result<CheckedIdentity, ValidityHold> {
    let before = current()?;
    let client = expected.fence(&before, expected.started_at)?;
    let response = exchange(client).await;
    let after = current()?;
    expected.fence(&after, before.now)?;
    let (status, bytes) = response.map_err(|error| match error {
        ClientError::InvalidOrigin | ClientError::OriginMismatch { .. } => {
            ValidityHold::BindingMismatch
        }
        ClientError::Transport(_) => ValidityHold::Transport,
        ClientError::InvalidResponse(_) => ValidityHold::Unreadable,
    })?;
    let identity = decode_identity_status(status, &bytes)?;
    if identity.account_id != expected.account {
        return Err(ValidityHold::InvalidIdentity);
    }
    Ok(CheckedIdentity {
        binding: expected,
        observed_at: after.now,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentityStatus {
    ok: bool,
    account_id: String,
    #[serde(deserialize_with = "object")]
    github: Github,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Github {
    linked: bool,
    terminal_bridge_eligible: bool,
    #[serde(rename = "username")]
    _username: String,
}

// Derived struct decoding otherwise accepts sequence representations too.
fn object<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(de: D) -> Result<T, D::Error> {
    struct Object<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Object<T> {
        type Value = T;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an identity object")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    de.deserialize_map(Object(std::marker::PhantomData))
}

fn decode_identity_status(status: u16, bytes: &[u8]) -> Result<IdentityStatus, ValidityHold> {
    if status != 200 {
        return Err(ValidityHold::Http(status));
    }
    if bytes.len() > RESPONSE_LIMIT {
        return Err(ValidityHold::Unreadable);
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let identity: IdentityStatus =
        object(&mut decoder).map_err(|_| ValidityHold::InvalidIdentity)?;
    decoder.end().map_err(|_| ValidityHold::InvalidIdentity)?;
    if !identity.ok
        || identity.account_id.trim().is_empty()
        || !identity.github.linked
        || !identity.github.terminal_bridge_eligible
    {
        return Err(ValidityHold::InvalidIdentity);
    }
    // Wallet/counts/sync/server and other unknown fields are deliberately ignored.
    // This checks duplicates only in the identity projection, not all JSON data.
    Ok(identity)
}

#[cfg(test)]
#[path = "session_validity_tests.rs"]
mod tests;
