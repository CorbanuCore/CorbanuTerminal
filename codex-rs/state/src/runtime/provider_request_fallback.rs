//! The provider-request throttle, kept working when the state DB is busy or
//! read-only.
//!
//! The throttle protects a provider key from this machine's own sessions:
//! after a 429 it holds requests for the cooldown the provider asked for, and
//! it lets one autonomous sub-agent at a time send a large uncached request on
//! a metered third-party key (the lease). Both live in the shared state DB so
//! that every Corbanu process on the home sees them. Neither is a configured
//! spend cap; no budget is enforced here.
//!
//! Every Corbanu process on a home shares that DB, so another one can hold its
//! write lock for minutes, and the file can be read-only. Either used to end
//! the turn ("failed to check provider request throttle state"). Now a step
//! waits for the DB at most `PROVIDER_REQUEST_BUSY_WAIT` and then decides from
//! this process's copy of the same state, which every step keeps current:
//! - a cooldown this process recorded or read still holds its requests back;
//! - this process's sub-agents still take the lease one at a time.
//!
//! Until the DB is writable again, this process and the others no longer see
//! each other's cooldowns and leases: each one goes on enforcing what it
//! knows. A busy DB slows a turn rather than ending it, and no request is sent
//! that this process's own limits would have held back. After a failure the
//! DB is left alone for `STATE_DB_RETRY_AFTER`, so the requests of a long turn
//! don't each wait the lock out again.
use super::busy_retry::is_busy;
use super::busy_retry::is_read_only;
use super::provider_requests::RATE_LIMIT_STATUS;
use super::provider_requests::default_cooldown_ms;
use super::provider_requests::since;
use super::*;
use crate::ProviderRequestBlock;
use crate::ProviderRequestBlockReason;
use crate::ProviderRequestKey;
use crate::ProviderRequestLease;
use crate::ProviderRequestLeaseDecision;
use crate::ProviderRequestPreflight;
use crate::ProviderRequestResult;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use tracing::info;

/// How long the throttle leaves the state DB alone after it failed.
const STATE_DB_RETRY_AFTER: Duration = Duration::from_secs(60);

/// Why a throttle decision was made without the state DB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateDbFallback {
    /// Another process held the write lock past the wait.
    Busy,
    /// The database file cannot be written.
    ReadOnly,
    /// Any other failure.
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequestThrottled<T> {
    pub decision: T,
    /// Set when the decision was made from this process's memory alone.
    pub state_db_fallback: Option<StateDbFallback>,
}

impl<T> ProviderRequestThrottled<T> {
    fn new(decision: T, state_db_fallback: Option<StateDbFallback>) -> Self {
        Self {
            decision,
            state_db_fallback,
        }
    }
}

/// This process's copy of the throttle state, one row per provider key.
#[derive(Default)]
pub(crate) struct ProviderRequestMemory {
    rows: Mutex<HashMap<ProviderRequestKey, MemoryRow>>,
    paused: Mutex<Option<(Instant, StateDbFallback)>>,
}

/// The fields of a `provider_request_state` row. A cooldown or lease the
/// state DB also holds (`*_in_state_db`) gives way to the DB while the DB
/// answers; one it doesn't hold was taken without it and always applies.
#[derive(Default)]
struct MemoryRow {
    cooldown_until_ms: i64,
    cooldown_in_state_db: bool,
    lease_owner: Option<String>,
    lease_until_ms: i64,
    lease_in_state_db: bool,
    last_status: Option<i64>,
    last_request_id: Option<String>,
    last_input_tokens: i64,
    last_cached_input_tokens: i64,
    last_request_bytes: i64,
    last_provider_input_tokens: i64,
    last_provider_cached_input_tokens: i64,
    consecutive_429_count: i64,
}

impl MemoryRow {
    fn note_preflight(&mut self, preflight: &ProviderRequestPreflight) {
        self.last_input_tokens = preflight.input_tokens.max(0);
        self.last_cached_input_tokens = preflight.cached_input_tokens.max(0);
        self.last_request_bytes = preflight.request_bytes.max(0);
    }

    fn block(
        &self,
        reason: ProviderRequestBlockReason,
        until_ms: i64,
        now_ms: i64,
    ) -> ProviderRequestBlock {
        ProviderRequestBlock {
            reason,
            until_ms,
            remaining_ms: until_ms.saturating_sub(now_ms),
            lease_owner: self.lease_owner.clone(),
            last_status: self.last_status,
            last_request_id: self.last_request_id.clone(),
            last_input_tokens: self.last_input_tokens,
            last_cached_input_tokens: self.last_cached_input_tokens,
            last_request_bytes: self.last_request_bytes,
            last_provider_input_tokens: self.last_provider_input_tokens,
            last_provider_cached_input_tokens: self.last_provider_cached_input_tokens,
        }
    }

    /// `state_db_answered`: the state DB decided this step, so what it holds
    /// is already accounted for.
    fn cooldown_block(&self, now_ms: i64, state_db_answered: bool) -> Option<ProviderRequestBlock> {
        (self.cooldown_until_ms > now_ms && !(state_db_answered && self.cooldown_in_state_db)).then(
            || {
                self.block(
                    ProviderRequestBlockReason::Cooldown,
                    self.cooldown_until_ms,
                    now_ms,
                )
            },
        )
    }

    fn lease_block(&self, now_ms: i64, state_db_answered: bool) -> Option<ProviderRequestBlock> {
        self.cooldown_block(now_ms, state_db_answered).or_else(|| {
            (self.lease_until_ms > now_ms && !(state_db_answered && self.lease_in_state_db)).then(
                || {
                    self.block(
                        ProviderRequestBlockReason::Lease,
                        self.lease_until_ms,
                        now_ms,
                    )
                },
            )
        })
    }

    fn take_lease(&mut self, owner: &str, lease_until_ms: i64, in_state_db: bool) {
        self.lease_owner = Some(owner.to_string());
        self.lease_until_ms = lease_until_ms;
        self.lease_in_state_db = in_state_db;
        self.last_status = None;
        self.last_request_id = None;
        self.last_provider_input_tokens = 0;
        self.last_provider_cached_input_tokens = 0;
    }

    /// Copy the cooldown the state DB just answered with, so it still applies
    /// if the DB stops answering. `None`: the DB holds no cooldown now.
    fn note_state_db(&mut self, block: Option<&ProviderRequestBlock>) {
        let Some(block) = block else {
            if self.cooldown_in_state_db {
                self.cooldown_until_ms = 0;
            }
            return;
        };
        self.last_status = block.last_status;
        self.last_request_id.clone_from(&block.last_request_id);
        self.last_provider_input_tokens = block.last_provider_input_tokens;
        self.last_provider_cached_input_tokens = block.last_provider_cached_input_tokens;
        // Another process's lease is not copied: this process could not see it
        // end, and would hold its own requests back until the lease expired.
        if block.reason == ProviderRequestBlockReason::Cooldown
            && block.until_ms >= self.cooldown_until_ms
        {
            self.cooldown_until_ms = block.until_ms;
            self.cooldown_in_state_db = true;
        }
    }

    /// The in-memory `record_provider_request_result_once`.
    fn record(
        &mut self,
        owner: &str,
        result: &ProviderRequestResult,
        now_ms: i64,
        in_state_db: bool,
    ) -> bool {
        if self.lease_owner.as_deref() != Some(owner) {
            return false;
        }
        self.lease_owner = None;
        self.lease_until_ms = 0;
        match result {
            ProviderRequestResult::Success {
                input_tokens,
                cached_input_tokens,
            } => {
                self.cooldown_until_ms = 0;
                self.last_status = Some(200);
                self.last_request_id = None;
                if let Some(tokens) = input_tokens {
                    self.last_provider_input_tokens = (*tokens).max(0);
                }
                if let Some(tokens) = cached_input_tokens {
                    self.last_provider_cached_input_tokens = (*tokens).max(0);
                }
                self.consecutive_429_count = 0;
            }
            ProviderRequestResult::Failed {
                status,
                request_id,
                retry_after_ms,
            } => {
                let status = status.map(i64::from);
                let is_rate_limit = status == Some(RATE_LIMIT_STATUS);
                self.consecutive_429_count = if is_rate_limit {
                    self.consecutive_429_count.saturating_add(1)
                } else {
                    0
                };
                self.cooldown_until_ms = if is_rate_limit {
                    now_ms.saturating_add(
                        retry_after_ms
                            .filter(|ms| *ms > 0)
                            .unwrap_or_else(|| default_cooldown_ms(self.consecutive_429_count)),
                    )
                } else {
                    0
                };
                self.cooldown_in_state_db = in_state_db;
                self.last_status = status;
                self.last_request_id.clone_from(request_id);
            }
        }
        true
    }

    fn release(&mut self, owner: &str) -> bool {
        if self.lease_owner.as_deref() != Some(owner) {
            return false;
        }
        self.lease_owner = None;
        self.lease_until_ms = 0;
        true
    }
}

impl ProviderRequestMemory {
    fn with_row<T>(&self, key: &ProviderRequestKey, f: impl FnOnce(&mut MemoryRow) -> T) -> T {
        let mut rows = self.rows.lock().unwrap_or_else(PoisonError::into_inner);
        f(rows.entry(key.clone()).or_default())
    }

    /// Why the state DB is being left alone, if it is.
    fn paused(&self) -> Option<StateDbFallback> {
        let mut paused = self.paused.lock().unwrap_or_else(PoisonError::into_inner);
        let (until, reason) = (*paused)?;
        if Instant::now() < until {
            return Some(reason);
        }
        *paused = None;
        info!("retrying the state DB for the provider request throttle");
        None
    }

    fn pause(&self, operation: &'static str, error: &anyhow::Error) -> StateDbFallback {
        let reason = if is_busy(error) {
            StateDbFallback::Busy
        } else if is_read_only(error) {
            StateDbFallback::ReadOnly
        } else {
            StateDbFallback::Unavailable
        };
        warn!(
            operation,
            ?reason,
            error = %format!("{error:#}"),
            retry_after_s = STATE_DB_RETRY_AFTER.as_secs(),
            "state DB unavailable; the provider request throttle uses this process's state"
        );
        *self.paused.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((Instant::now() + STATE_DB_RETRY_AFTER, reason));
        reason
    }
}

impl StateRuntime {
    /// Whether a request may be sent now, or a cooldown holds it back. Never
    /// fails: see the module docs.
    pub async fn check_provider_request_throttle(
        &self,
        key: &ProviderRequestKey,
        preflight: &ProviderRequestPreflight,
        now_ms: i64,
    ) -> ProviderRequestThrottled<Option<ProviderRequestBlock>> {
        let memory = &self.provider_request_memory;
        let started = Instant::now();
        let mut fallback = memory.paused();
        if fallback.is_none() {
            match self
                .check_provider_request_cooldown(key, preflight, now_ms)
                .await
            {
                Ok(block) => {
                    let now_ms = since(now_ms, started);
                    let decision = memory.with_row(key, |row| {
                        row.note_preflight(preflight);
                        row.note_state_db(block.as_ref());
                        block
                            .or_else(|| row.cooldown_block(now_ms, /*state_db_answered*/ true))
                    });
                    return ProviderRequestThrottled::new(decision, None);
                }
                Err(error) => {
                    fallback = Some(memory.pause("check provider request cooldown", &error));
                }
            }
        }
        let now_ms = since(now_ms, started);
        let decision = memory.with_row(key, |row| {
            row.note_preflight(preflight);
            row.cooldown_block(now_ms, /*state_db_answered*/ false)
        });
        ProviderRequestThrottled::new(decision, fallback)
    }

    /// Take the request lease for `owner`, or say what holds it back. Never
    /// fails: see the module docs.
    pub async fn acquire_provider_request_throttle_lease(
        &self,
        key: &ProviderRequestKey,
        preflight: &ProviderRequestPreflight,
        owner: &str,
        lease_ttl_ms: i64,
        now_ms: i64,
    ) -> ProviderRequestThrottled<ProviderRequestLeaseDecision> {
        let memory = &self.provider_request_memory;
        let started = Instant::now();
        let mut fallback = memory.paused();
        if fallback.is_none() {
            match self
                .try_acquire_provider_request_lease(key, preflight, owner, lease_ttl_ms, now_ms)
                .await
            {
                Ok(ProviderRequestLeaseDecision::Acquired(lease)) => {
                    let now_ms = since(now_ms, started);
                    let block = memory.with_row(key, |row| {
                        row.note_preflight(preflight);
                        row.note_state_db(None);
                        let block = row.lease_block(now_ms, /*state_db_answered*/ true);
                        if block.is_none() {
                            row.take_lease(owner, lease.lease_until_ms, /*in_state_db*/ true);
                        }
                        block
                    });
                    let Some(block) = block else {
                        return ProviderRequestThrottled::new(
                            ProviderRequestLeaseDecision::Acquired(lease),
                            None,
                        );
                    };
                    // A cooldown or lease this process took while the state DB
                    // was unavailable is still running; give the DB lease back.
                    if let Err(error) = self.release_provider_request_lease(&lease, now_ms).await {
                        warn!(
                            error = %format!("{error:#}"),
                            "failed to release a provider request lease held back in memory"
                        );
                    }
                    return ProviderRequestThrottled::new(
                        ProviderRequestLeaseDecision::Blocked(block),
                        None,
                    );
                }
                Ok(ProviderRequestLeaseDecision::Blocked(block)) => {
                    memory.with_row(key, |row| {
                        row.note_preflight(preflight);
                        row.note_state_db(Some(&block));
                    });
                    return ProviderRequestThrottled::new(
                        ProviderRequestLeaseDecision::Blocked(block),
                        None,
                    );
                }
                Err(error) => {
                    fallback = Some(memory.pause("acquire provider request lease", &error));
                }
            }
        }
        let now_ms = since(now_ms, started);
        let decision = memory.with_row(key, |row| {
            row.note_preflight(preflight);
            if let Some(block) = row.lease_block(now_ms, /*state_db_answered*/ false) {
                return ProviderRequestLeaseDecision::Blocked(block);
            }
            let lease_until_ms = now_ms.saturating_add(lease_ttl_ms.max(1));
            row.take_lease(owner, lease_until_ms, /*in_state_db*/ false);
            ProviderRequestLeaseDecision::Acquired(ProviderRequestLease {
                key: key.clone(),
                owner: owner.to_string(),
                lease_until_ms,
                in_state_db: false,
            })
        });
        ProviderRequestThrottled::new(decision, fallback)
    }

    /// Record how the leased request ended and end the lease. The decision is
    /// whether the state DB no longer holds the lease; when it is `false` the
    /// caller releases it later with `release_provider_request_throttle_lease`.
    pub async fn record_provider_request_throttle_result(
        &self,
        lease: &ProviderRequestLease,
        result: ProviderRequestResult,
        now_ms: i64,
    ) -> ProviderRequestThrottled<bool> {
        let memory = &self.provider_request_memory;
        let started = Instant::now();
        let mut fallback = None;
        let mut state_db_recorded = false;
        if lease.in_state_db {
            fallback = memory.paused();
            if fallback.is_none() {
                match self
                    .record_provider_request_result(lease, result.clone(), now_ms)
                    .await
                {
                    Ok(rows_affected) => {
                        state_db_recorded = true;
                        if rows_affected == 0 {
                            warn!(
                                provider = %lease.key.provider_id,
                                model = %lease.key.model,
                                owner = %lease.owner,
                                "provider request result did not match active lease owner"
                            );
                        }
                    }
                    Err(error) => {
                        fallback = Some(memory.pause("record provider request result", &error));
                    }
                }
            }
        }
        let now_ms = since(now_ms, started);
        memory.with_row(&lease.key, |row| {
            row.record(&lease.owner, &result, now_ms, state_db_recorded)
        });
        ProviderRequestThrottled::new(state_db_recorded || !lease.in_state_db, fallback)
    }

    /// End a lease without a result (an interrupted or abandoned request).
    /// This waits out a busy state DB for longer than the turn's steps do, so
    /// run it off the turn's path.
    pub async fn release_provider_request_throttle_lease(
        &self,
        lease: &ProviderRequestLease,
        now_ms: i64,
    ) -> anyhow::Result<u64> {
        let released = self
            .provider_request_memory
            .with_row(&lease.key, |row| row.release(&lease.owner));
        if lease.in_state_db {
            self.release_provider_request_lease(lease, now_ms).await
        } else {
            Ok(u64::from(released))
        }
    }
}

#[cfg(test)]
#[path = "provider_request_fallback_tests.rs"]
mod tests;
