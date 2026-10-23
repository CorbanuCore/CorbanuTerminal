use super::*;
use crate::runtime::busy_retry::retry_busy;
use crate::runtime::busy_retry::retry_busy_within;
use std::time::Duration;

pub(super) const RATE_LIMIT_STATUS: i64 = 429;
const DEFAULT_COOLDOWN_CAP_MS: i64 = 5 * 60 * 1000;
/// About how long a throttle step on the turn's path waits for a busy state DB
/// before the turn goes on with this process's in-memory throttle. These steps
/// run on the short-wait pool (`StateRuntime::quick`): an attempt waits at most
/// `QUICK_BUSY_TIMEOUT`, so this allows a second one and no more.
///
/// This is a budget for starting tries, not a hard limit on the wait: a second
/// try that starts inside it runs its full `QUICK_BUSY_TIMEOUT`. With the lock
/// held throughout, the step is two 500 ms tries and a 50 ms pause, about
/// 1.05 s on paper. Measured (#905, #916): about 1.2 s on macOS and about
/// 1.3 s on Windows, where SQLite's busy sleeps round up to the 15.6 ms system
/// timer (each 500 ms try takes about 625 ms). A loaded machine takes longer:
/// 1.4-1.8 s on a loaded Mac, and one short wait took 2 s on a Windows CI
/// runner (#914). Kept as is (owner decision D30, recorded on #905): a strict
/// 1 s would allow one try only and send brief lock holds to the 60 s
/// in-memory fallback (`STATE_DB_RETRY_AFTER`) sooner.
pub(super) const PROVIDER_REQUEST_BUSY_WAIT: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProviderRequestKey {
    pub provider_id: String,
    pub model: String,
    pub key_fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequestPreflight {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub request_bytes: i64,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequestLease {
    pub key: ProviderRequestKey,
    pub owner: String,
    pub lease_until_ms: i64,
    /// Whether the lease was (or may have been) written to the state DB, so
    /// that releasing it must clear it there too. A lease taken while the
    /// state DB was busy or read-only lives only in this process.
    pub in_state_db: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRequestBlock {
    pub reason: ProviderRequestBlockReason,
    pub until_ms: i64,
    pub remaining_ms: i64,
    pub lease_owner: Option<String>,
    pub last_status: Option<i64>,
    pub last_request_id: Option<String>,
    pub last_input_tokens: i64,
    pub last_cached_input_tokens: i64,
    pub last_request_bytes: i64,
    pub last_provider_input_tokens: i64,
    pub last_provider_cached_input_tokens: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderRequestBlockReason {
    Cooldown,
    Lease,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderRequestLeaseDecision {
    Acquired(ProviderRequestLease),
    Blocked(ProviderRequestBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderRequestResult {
    Success {
        input_tokens: Option<i64>,
        cached_input_tokens: Option<i64>,
    },
    Failed {
        status: Option<u16>,
        request_id: Option<String>,
        retry_after_ms: Option<i64>,
    },
}

impl StateRuntime {
    async fn check_provider_request_cooldown_once(
        &self,
        key: &ProviderRequestKey,
        preflight: &ProviderRequestPreflight,
        now_ms: i64,
    ) -> anyhow::Result<Option<ProviderRequestBlock>> {
        let mut tx = crate::runtime::db_lock::begin_immediate(&self.pool).await?;

        sqlx::query(
            r#"
INSERT INTO provider_request_state (
    provider_id,
    model,
    key_fingerprint,
    last_input_tokens,
    last_cached_input_tokens,
    last_request_bytes,
    last_thread_id,
    last_turn_id,
    updated_at_ms
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(provider_id, model, key_fingerprint) DO UPDATE SET
    last_input_tokens = excluded.last_input_tokens,
    last_cached_input_tokens = excluded.last_cached_input_tokens,
    last_request_bytes = excluded.last_request_bytes,
    last_thread_id = excluded.last_thread_id,
    last_turn_id = excluded.last_turn_id,
    updated_at_ms = excluded.updated_at_ms
            "#,
        )
        .bind(key.provider_id.as_str())
        .bind(key.model.as_str())
        .bind(key.key_fingerprint.as_str())
        .bind(preflight.input_tokens.max(0))
        .bind(preflight.cached_input_tokens.max(0))
        .bind(preflight.request_bytes.max(0))
        .bind(preflight.thread_id.as_deref())
        .bind(preflight.turn_id.as_deref())
        .bind(now_ms)
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query(
            r#"
SELECT
    cooldown_until_ms,
    lease_owner,
    lease_until_ms,
    last_status,
    last_request_id,
    last_input_tokens,
    last_cached_input_tokens,
    last_request_bytes,
    last_provider_input_tokens,
    last_provider_cached_input_tokens
FROM provider_request_state
WHERE provider_id = ? AND model = ? AND key_fingerprint = ?
            "#,
        )
        .bind(key.provider_id.as_str())
        .bind(key.model.as_str())
        .bind(key.key_fingerprint.as_str())
        .fetch_one(&mut *tx)
        .await?;

        let cooldown_until_ms: i64 = row.try_get("cooldown_until_ms")?;
        let block = if cooldown_until_ms > now_ms {
            Some(block_from_row(
                &row,
                ProviderRequestBlockReason::Cooldown,
                cooldown_until_ms,
                now_ms,
            )?)
        } else {
            None
        };

        tx.commit().await?;
        Ok(block)
    }

    async fn try_acquire_provider_request_lease_once(
        &self,
        key: &ProviderRequestKey,
        preflight: &ProviderRequestPreflight,
        owner: &str,
        lease_ttl_ms: i64,
        now_ms: i64,
    ) -> anyhow::Result<ProviderRequestLeaseDecision> {
        let lease_until_ms = now_ms.saturating_add(lease_ttl_ms.max(1));
        let mut tx = crate::runtime::db_lock::begin_immediate(&self.pool).await?;

        sqlx::query(
            r#"
INSERT INTO provider_request_state (
    provider_id,
    model,
    key_fingerprint,
    last_input_tokens,
    last_cached_input_tokens,
    last_request_bytes,
    last_thread_id,
    last_turn_id,
    updated_at_ms
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(provider_id, model, key_fingerprint) DO UPDATE SET
    last_input_tokens = excluded.last_input_tokens,
    last_cached_input_tokens = excluded.last_cached_input_tokens,
    last_request_bytes = excluded.last_request_bytes,
    last_thread_id = excluded.last_thread_id,
    last_turn_id = excluded.last_turn_id,
    updated_at_ms = excluded.updated_at_ms
            "#,
        )
        .bind(key.provider_id.as_str())
        .bind(key.model.as_str())
        .bind(key.key_fingerprint.as_str())
        .bind(preflight.input_tokens.max(0))
        .bind(preflight.cached_input_tokens.max(0))
        .bind(preflight.request_bytes.max(0))
        .bind(preflight.thread_id.as_deref())
        .bind(preflight.turn_id.as_deref())
        .bind(now_ms)
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query(
            r#"
SELECT
    cooldown_until_ms,
    lease_owner,
    lease_until_ms,
    last_status,
    last_request_id,
    last_input_tokens,
    last_cached_input_tokens,
    last_request_bytes,
    last_provider_input_tokens,
    last_provider_cached_input_tokens
FROM provider_request_state
WHERE provider_id = ? AND model = ? AND key_fingerprint = ?
            "#,
        )
        .bind(key.provider_id.as_str())
        .bind(key.model.as_str())
        .bind(key.key_fingerprint.as_str())
        .fetch_one(&mut *tx)
        .await?;

        let cooldown_until_ms: i64 = row.try_get("cooldown_until_ms")?;
        if cooldown_until_ms > now_ms {
            let block = block_from_row(
                &row,
                ProviderRequestBlockReason::Cooldown,
                cooldown_until_ms,
                now_ms,
            )?;
            tx.commit().await?;
            return Ok(ProviderRequestLeaseDecision::Blocked(block));
        }

        let lease_until_existing_ms: i64 = row.try_get("lease_until_ms")?;
        if lease_until_existing_ms > now_ms {
            let block = block_from_row(
                &row,
                ProviderRequestBlockReason::Lease,
                lease_until_existing_ms,
                now_ms,
            )?;
            tx.commit().await?;
            return Ok(ProviderRequestLeaseDecision::Blocked(block));
        }

        sqlx::query(
            r#"
UPDATE provider_request_state
SET lease_owner = ?,
    lease_until_ms = ?,
    last_status = NULL,
    last_request_id = NULL,
    last_provider_input_tokens = 0,
    last_provider_cached_input_tokens = 0,
    updated_at_ms = ?
WHERE provider_id = ? AND model = ? AND key_fingerprint = ?
            "#,
        )
        .bind(owner)
        .bind(lease_until_ms)
        .bind(now_ms)
        .bind(key.provider_id.as_str())
        .bind(key.model.as_str())
        .bind(key.key_fingerprint.as_str())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(ProviderRequestLeaseDecision::Acquired(
            ProviderRequestLease {
                key: key.clone(),
                owner: owner.to_string(),
                lease_until_ms,
                in_state_db: true,
            },
        ))
    }

    /// End `lease` and record how its request ended. Since unleased requests
    /// record results too (#476), a cooldown can start while a lease is held,
    /// so a leased result follows the same rules as an unleased one: it never
    /// shortens an active cooldown (`UnleasedRow::apply`).
    async fn record_provider_request_result_once(
        &self,
        lease: &ProviderRequestLease,
        result: ProviderRequestResult,
        now_ms: i64,
    ) -> anyhow::Result<u64> {
        let mut tx = crate::runtime::db_lock::begin_immediate(&self.pool).await?;
        let released = sqlx::query(
            r#"
UPDATE provider_request_state
SET lease_owner = NULL,
    lease_until_ms = 0,
    updated_at_ms = ?
WHERE provider_id = ? AND model = ? AND key_fingerprint = ? AND lease_owner = ?
            "#,
        )
        .bind(now_ms)
        .bind(lease.key.provider_id.as_str())
        .bind(lease.key.model.as_str())
        .bind(lease.key.key_fingerprint.as_str())
        .bind(lease.owner.as_str())
        .execute(&mut *tx)
        .await?
        .rows_affected();
        // Not this lease's row any more: record nothing under its name.
        if released > 0 {
            record_result_on_connection(&mut tx, &lease.key, &result, now_ms).await?;
        }
        tx.commit().await?;
        Ok(released)
    }

    /// Record how a request sent without a lease ended (#476). Unlike a leased
    /// result it never touches the lease, and since other requests on the key
    /// may have ended meanwhile, it never shortens a cooldown: a 429 extends
    /// it, and any other result leaves an active one alone.
    async fn record_provider_request_unleased_result_once(
        &self,
        key: &ProviderRequestKey,
        result: &ProviderRequestResult,
        now_ms: i64,
    ) -> anyhow::Result<()> {
        let mut tx = crate::runtime::db_lock::begin_immediate(&self.pool).await?;
        record_result_on_connection(&mut tx, key, result, now_ms).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn release_provider_request_lease_once(
        &self,
        lease: &ProviderRequestLease,
        now_ms: i64,
    ) -> anyhow::Result<u64> {
        let rows_affected = sqlx::query(
            r#"
UPDATE provider_request_state
SET lease_owner = NULL,
    lease_until_ms = 0,
    updated_at_ms = ?
WHERE provider_id = ? AND model = ? AND key_fingerprint = ? AND lease_owner = ?
            "#,
        )
        .bind(now_ms)
        .bind(lease.key.provider_id.as_str())
        .bind(lease.key.model.as_str())
        .bind(lease.key.key_fingerprint.as_str())
        .bind(lease.owner.as_str())
        .execute(self.pool.as_ref())
        .await?
        .rows_affected();
        Ok(rows_affected)
    }
}

impl StateRuntime {
    // The state DB side of the provider-request throttle. Callers outside this
    // crate go through `provider_request_fallback`, which falls back to this
    // process's in-memory throttle when these fail.
    //
    // Each write below is one short transaction that rolls back whole when the
    // state DB is busy, so it is retried; see `retry_busy`. The steps on the
    // turn's path wait about `PROVIDER_REQUEST_BUSY_WAIT`; releasing a lease
    // off the turn's path waits the full busy deadline, so another process is
    // not kept waiting on a lease nobody holds. A retry's clock
    // reading moves on by the time spent waiting, so cooldowns and leases count
    // from when the write lands.

    pub(crate) async fn check_provider_request_cooldown(
        &self,
        key: &ProviderRequestKey,
        preflight: &ProviderRequestPreflight,
        now_ms: i64,
    ) -> anyhow::Result<Option<ProviderRequestBlock>> {
        let started = Instant::now();
        retry_busy_within(
            "check provider request cooldown",
            PROVIDER_REQUEST_BUSY_WAIT,
            || {
                self.quick().check_provider_request_cooldown_once(
                    key,
                    preflight,
                    since(now_ms, started),
                )
            },
        )
        .await
    }

    pub(crate) async fn try_acquire_provider_request_lease(
        &self,
        key: &ProviderRequestKey,
        preflight: &ProviderRequestPreflight,
        owner: &str,
        lease_ttl_ms: i64,
        now_ms: i64,
    ) -> anyhow::Result<ProviderRequestLeaseDecision> {
        let started = Instant::now();
        retry_busy_within(
            "acquire provider request lease",
            PROVIDER_REQUEST_BUSY_WAIT,
            || {
                self.quick().try_acquire_provider_request_lease_once(
                    key,
                    preflight,
                    owner,
                    lease_ttl_ms,
                    since(now_ms, started),
                )
            },
        )
        .await
    }

    pub(crate) async fn record_provider_request_result(
        &self,
        lease: &ProviderRequestLease,
        result: ProviderRequestResult,
        now_ms: i64,
    ) -> anyhow::Result<u64> {
        let started = Instant::now();
        retry_busy_within(
            "record provider request result",
            PROVIDER_REQUEST_BUSY_WAIT,
            || {
                self.quick().record_provider_request_result_once(
                    lease,
                    result.clone(),
                    since(now_ms, started),
                )
            },
        )
        .await
    }

    pub(crate) async fn release_provider_request_lease(
        &self,
        lease: &ProviderRequestLease,
        now_ms: i64,
    ) -> anyhow::Result<u64> {
        let started = Instant::now();
        retry_busy("release provider request lease", || {
            self.release_provider_request_lease_once(lease, since(now_ms, started))
        })
        .await
    }

    pub(crate) async fn record_provider_request_unleased_result(
        &self,
        key: &ProviderRequestKey,
        result: &ProviderRequestResult,
        now_ms: i64,
    ) -> anyhow::Result<()> {
        let started = Instant::now();
        retry_busy_within(
            "record provider request result",
            PROVIDER_REQUEST_BUSY_WAIT,
            || {
                self.quick().record_provider_request_unleased_result_once(
                    key,
                    result,
                    since(now_ms, started),
                )
            },
        )
        .await
    }

    /// The per-home salt provider keys are fingerprinted with (#482), so a
    /// throttle row's key differs on every home holding the same API key.
    /// Migration 0053 creates it; it is read once per process.
    pub async fn provider_request_key_salt(&self) -> anyhow::Result<Arc<[u8]>> {
        self.provider_request_memory
            .key_salt
            .get_or_try_init(|| async {
                let read = || async {
                    sqlx::query_scalar::<_, Vec<u8>>(
                        "SELECT salt FROM provider_request_key_salt WHERE id = 1",
                    )
                    .fetch_optional(self.pool.as_ref())
                    .await
                };
                let salt = match read().await? {
                    Some(salt) => salt,
                    None => {
                        sqlx::query(
                            "INSERT OR IGNORE INTO provider_request_key_salt (id, salt) VALUES (1, randomblob(32))",
                        )
                        .execute(self.pool.as_ref())
                        .await?;
                        read()
                            .await?
                            .ok_or_else(|| anyhow::anyhow!("provider request key salt missing"))?
                    }
                };
                anyhow::ensure!(salt.len() >= 16, "provider request key salt too short");
                Ok(Arc::from(salt))
            })
            .await
            .cloned()
    }

    /// Move the throttle rows an older build kept under `legacy` (an unsalted
    /// key fingerprint) to `salted`, keeping their cooldown and lease, and
    /// delete every idle unsalted row, such as one an older build sharing the
    /// state DB wrote since migration 0053 (#482). A salted row that already
    /// exists wins.
    ///
    /// Tried once per key and process, and never while the throttle leaves the
    /// DB alone: a busy DB costs one wait, then pauses the DB as the other
    /// throttle steps do, and the rows are adopted by a later process.
    pub async fn adopt_legacy_provider_request_key(&self, legacy: &str, salted: &str, now_ms: i64) {
        let memory = &self.provider_request_memory;
        if !memory
            .adopted_legacy_keys
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(salted.to_string())
        {
            return;
        }
        if self
            .provider_request_state_db_unready(now_ms)
            .await
            .is_some()
        {
            return;
        }
        let started = Instant::now();
        let adopt = || async {
            let now_ms = since(now_ms, started);
            let mut tx = crate::runtime::db_lock::begin_immediate(&self.quick().pool).await?;
            sqlx::query(
                "UPDATE OR IGNORE provider_request_state SET key_fingerprint = ? WHERE key_fingerprint = ?",
            )
            .bind(salted)
            .bind(legacy)
            .execute(&mut *tx)
            .await?;
            sqlx::query("DELETE FROM provider_request_state WHERE key_fingerprint = ?")
                .bind(legacy)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                "DELETE FROM provider_request_state WHERE key_fingerprint LIKE 'env:%' AND cooldown_until_ms <= ? AND lease_until_ms <= ?",
            )
            .bind(now_ms)
            .bind(now_ms)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            anyhow::Ok(())
        };
        // One short try: the rows are adopted by a later process if not now,
        // and a busy DB then pauses the other steps rather than adding to them.
        if let Err(error) = retry_busy_within(
            "adopt legacy provider request key",
            super::QUICK_BUSY_TIMEOUT,
            adopt,
        )
        .await
        {
            self.pause_state_db("adopt legacy provider request key", &error);
        }
    }

    /// Carry a named account's active cooldown and lease from the row it was
    /// kept under by name (`by_name`) to the one for its credential
    /// (`by_key`), and drop the name's rows for `provider_id`; idle ones carry
    /// nothing worth keeping. Several names can hold one credential, so an
    /// existing credential row keeps the later cooldown and its own lease
    /// when that is still active.
    ///
    /// Done once per name, key and process. Like the other steps it waits one
    /// short try for a busy DB, then pauses it, and unless the DB is read-only
    /// it is tried again on a later request once the pause ends. A name row's
    /// lease is dropped when the credential row holds an active lease of its
    /// own: a request still running under it is then not waited for, once.
    pub async fn adopt_named_account_provider_request_key(
        &self,
        provider_id: &str,
        by_name: &str,
        by_key: &str,
        now_ms: i64,
    ) {
        let memory = &self.provider_request_memory;
        let adopted = format!("{provider_id}\0{by_name}\0{by_key}");
        if !memory
            .adopted_legacy_keys
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(adopted.clone())
        {
            return;
        }
        // Not done: try again on a later request, once the throttle uses the DB
        // again (a paused DB answers without waiting). A read-only DB is not
        // retried.
        let not_adopted = || {
            memory
                .adopted_legacy_keys
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&adopted);
        };
        if let Some(unready) = self.provider_request_state_db_unready(now_ms).await {
            if unready != StateDbFallback::ReadOnly {
                not_adopted();
            }
            return;
        }
        let started = Instant::now();
        let adopt = || async {
            let now_ms = since(now_ms, started);
            let mut tx = crate::runtime::db_lock::begin_immediate(&self.quick().pool).await?;
            // SET expressions read the row as it was before the update.
            sqlx::query(
                r#"
UPDATE provider_request_state AS current SET
    cooldown_until_ms = MAX(current.cooldown_until_ms, named.cooldown_until_ms),
    consecutive_429_count = MAX(current.consecutive_429_count, named.consecutive_429_count),
    last_status = CASE WHEN named.cooldown_until_ms > current.cooldown_until_ms
        THEN named.last_status ELSE current.last_status END,
    last_request_id = CASE WHEN named.cooldown_until_ms > current.cooldown_until_ms
        THEN named.last_request_id ELSE current.last_request_id END,
    lease_owner = CASE WHEN current.lease_until_ms <= ?1 AND named.lease_until_ms > ?1
        THEN named.lease_owner ELSE current.lease_owner END,
    lease_until_ms = CASE WHEN current.lease_until_ms <= ?1 AND named.lease_until_ms > ?1
        THEN named.lease_until_ms ELSE current.lease_until_ms END
FROM provider_request_state AS named
WHERE current.provider_id = ?2 AND current.key_fingerprint = ?4
    AND named.provider_id = current.provider_id AND named.model = current.model
    AND named.key_fingerprint = ?3
    AND (named.cooldown_until_ms > ?1 OR named.lease_until_ms > ?1)
                "#,
            )
            .bind(now_ms)
            .bind(provider_id)
            .bind(by_name)
            .bind(by_key)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                r#"
UPDATE OR IGNORE provider_request_state SET key_fingerprint = ?4
WHERE provider_id = ?2 AND key_fingerprint = ?3
    AND (cooldown_until_ms > ?1 OR lease_until_ms > ?1)
                "#,
            )
            .bind(now_ms)
            .bind(provider_id)
            .bind(by_name)
            .bind(by_key)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "DELETE FROM provider_request_state WHERE provider_id = ? AND key_fingerprint = ?",
            )
            .bind(provider_id)
            .bind(by_name)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            anyhow::Ok(())
        };
        if let Err(error) = retry_busy_within(
            "adopt named account provider request key",
            super::QUICK_BUSY_TIMEOUT,
            adopt,
        )
        .await
            && self.pause_state_db("adopt named account provider request key", &error)
                != StateDbFallback::ReadOnly
        {
            not_adopted();
        }
    }

    /// `release_provider_request_lease`, waiting only as long as a step on the
    /// turn's path does.
    pub(crate) async fn release_provider_request_lease_briefly(
        &self,
        lease: &ProviderRequestLease,
        now_ms: i64,
    ) -> anyhow::Result<u64> {
        let started = Instant::now();
        retry_busy_within(
            "release provider request lease",
            PROVIDER_REQUEST_BUSY_WAIT,
            || {
                self.quick()
                    .release_provider_request_lease_once(lease, since(now_ms, started))
            },
        )
        .await
    }
}

/// `now_ms`, read at `started`, moved on by the time elapsed since.
pub(super) fn since(now_ms: i64, started: Instant) -> i64 {
    now_ms.saturating_add(i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX))
}

fn block_from_row(
    row: &sqlx::sqlite::SqliteRow,
    reason: ProviderRequestBlockReason,
    until_ms: i64,
    now_ms: i64,
) -> anyhow::Result<ProviderRequestBlock> {
    Ok(ProviderRequestBlock {
        reason,
        until_ms,
        remaining_ms: until_ms.saturating_sub(now_ms),
        lease_owner: row.try_get("lease_owner")?,
        last_status: row.try_get("last_status")?,
        last_request_id: row.try_get("last_request_id")?,
        last_input_tokens: row.try_get("last_input_tokens")?,
        last_cached_input_tokens: row.try_get("last_cached_input_tokens")?,
        last_request_bytes: row.try_get("last_request_bytes")?,
        last_provider_input_tokens: row.try_get("last_provider_input_tokens")?,
        last_provider_cached_input_tokens: row.try_get("last_provider_cached_input_tokens")?,
    })
}

/// Record how a request on `key` ended (`UnleasedRow::apply`), leaving any
/// lease alone. Caller owns the write transaction.
async fn record_result_on_connection(
    conn: &mut sqlx::SqliteConnection,
    key: &ProviderRequestKey,
    result: &ProviderRequestResult,
    now_ms: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
INSERT INTO provider_request_state (provider_id, model, key_fingerprint, updated_at_ms)
VALUES (?, ?, ?, ?)
ON CONFLICT(provider_id, model, key_fingerprint) DO NOTHING
        "#,
    )
    .bind(key.provider_id.as_str())
    .bind(key.model.as_str())
    .bind(key.key_fingerprint.as_str())
    .bind(now_ms)
    .execute(&mut *conn)
    .await?;
    let row = sqlx::query(
        r#"
SELECT consecutive_429_count, cooldown_until_ms
FROM provider_request_state
WHERE provider_id = ? AND model = ? AND key_fingerprint = ?
        "#,
    )
    .bind(key.provider_id.as_str())
    .bind(key.model.as_str())
    .bind(key.key_fingerprint.as_str())
    .fetch_one(&mut *conn)
    .await?;
    let existing = UnleasedRow {
        consecutive_429_count: row.try_get("consecutive_429_count")?,
        cooldown_until_ms: row.try_get("cooldown_until_ms")?,
    };
    let Some(update) = existing.apply(result, now_ms) else {
        return Ok(());
    };
    sqlx::query(
        r#"
UPDATE provider_request_state
SET cooldown_until_ms = ?,
    last_status = ?,
    last_request_id = ?,
    last_provider_input_tokens = COALESCE(?, last_provider_input_tokens),
    last_provider_cached_input_tokens = COALESCE(?, last_provider_cached_input_tokens),
    consecutive_429_count = ?,
    updated_at_ms = ?
WHERE provider_id = ? AND model = ? AND key_fingerprint = ?
        "#,
    )
    .bind(update.cooldown_until_ms)
    .bind(update.last_status)
    .bind(update.last_request_id)
    .bind(update.provider_input_tokens)
    .bind(update.provider_cached_input_tokens)
    .bind(update.consecutive_429_count)
    .bind(now_ms)
    .bind(key.provider_id.as_str())
    .bind(key.model.as_str())
    .bind(key.key_fingerprint.as_str())
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// The fields a request's result reads, in the state DB or this process's copy.
pub(super) struct UnleasedRow {
    pub(super) consecutive_429_count: i64,
    pub(super) cooldown_until_ms: i64,
}

/// What an unleased result writes.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct UnleasedUpdate {
    pub(super) cooldown_until_ms: i64,
    pub(super) last_status: Option<i64>,
    pub(super) last_request_id: Option<String>,
    pub(super) provider_input_tokens: Option<i64>,
    pub(super) provider_cached_input_tokens: Option<i64>,
    pub(super) consecutive_429_count: i64,
}

impl UnleasedRow {
    /// `None` when the result changes nothing: it carries no status, or it is
    /// not a 429 and a cooldown is active (its status would then read as the
    /// cooldown's cause).
    pub(super) fn apply(
        &self,
        result: &ProviderRequestResult,
        now_ms: i64,
    ) -> Option<UnleasedUpdate> {
        let cooling = self.cooldown_until_ms > now_ms;
        match result {
            ProviderRequestResult::Failed {
                status: Some(status),
                request_id,
                retry_after_ms,
            } if i64::from(*status) == RATE_LIMIT_STATUS => {
                // 429s from requests already in flight when a cooldown began
                // are one burst, not consecutive refusals: they extend the
                // cooldown without escalating it.
                let consecutive_429_count = if cooling {
                    self.consecutive_429_count.max(1)
                } else {
                    self.consecutive_429_count.saturating_add(1)
                };
                let cooldown = retry_after_ms
                    .filter(|ms| *ms > 0)
                    .unwrap_or_else(|| default_cooldown_ms(consecutive_429_count));
                Some(UnleasedUpdate {
                    cooldown_until_ms: self.cooldown_until_ms.max(now_ms.saturating_add(cooldown)),
                    last_status: Some(RATE_LIMIT_STATUS),
                    last_request_id: request_id.clone(),
                    provider_input_tokens: None,
                    provider_cached_input_tokens: None,
                    consecutive_429_count,
                })
            }
            _ if cooling => None,
            ProviderRequestResult::Failed {
                status: Some(status),
                request_id,
                ..
            } => Some(UnleasedUpdate {
                cooldown_until_ms: self.cooldown_until_ms,
                last_status: Some(i64::from(*status)),
                last_request_id: request_id.clone(),
                provider_input_tokens: None,
                provider_cached_input_tokens: None,
                consecutive_429_count: 0,
            }),
            ProviderRequestResult::Failed { status: None, .. } => None,
            ProviderRequestResult::Success {
                input_tokens,
                cached_input_tokens,
            } => Some(UnleasedUpdate {
                cooldown_until_ms: self.cooldown_until_ms,
                last_status: Some(200),
                last_request_id: None,
                provider_input_tokens: input_tokens.map(|tokens| tokens.max(0)),
                provider_cached_input_tokens: cached_input_tokens.map(|tokens| tokens.max(0)),
                consecutive_429_count: 0,
            }),
        }
    }
}

pub(super) fn default_cooldown_ms(consecutive_429_count: i64) -> i64 {
    match consecutive_429_count {
        count if count <= 1 => 30_000,
        2 => 60_000,
        3 => 120_000,
        _ => DEFAULT_COOLDOWN_CAP_MS,
    }
}

#[cfg(test)]
mod tests {
    use super::ProviderRequestBlockReason;
    use super::ProviderRequestKey;
    use super::ProviderRequestLeaseDecision;
    use super::ProviderRequestPreflight;
    use super::ProviderRequestResult;
    use super::StateRuntime;
    use super::test_support::unique_temp_dir;

    fn key() -> ProviderRequestKey {
        ProviderRequestKey {
            provider_id: "ambient".to_string(),
            model: "z-ai/glm-5.2".to_string(),
            key_fingerprint: "env:AMBIENT_API_KEY:test".to_string(),
        }
    }

    fn preflight() -> ProviderRequestPreflight {
        ProviderRequestPreflight {
            input_tokens: 37_492,
            cached_input_tokens: 20_000,
            request_bytes: 160_000,
            thread_id: Some("thread-a".to_string()),
            turn_id: Some("turn-a".to_string()),
        }
    }

    fn small_preflight() -> ProviderRequestPreflight {
        ProviderRequestPreflight {
            input_tokens: 2_426,
            cached_input_tokens: 576,
            request_bytes: 29_441,
            thread_id: Some("thread-b".to_string()),
            turn_id: Some("turn-b".to_string()),
        }
    }

    #[tokio::test]
    async fn lease_blocks_second_process_until_ttl() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let first = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire first lease");
        assert!(matches!(first, ProviderRequestLeaseDecision::Acquired(_)));

        let second = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_500,
            )
            .await
            .expect("acquire second lease");
        match second {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Lease);
                assert_eq!(block.lease_owner.as_deref(), Some("worker-a"));
                assert_eq!(block.last_input_tokens, 37_492);
                assert_eq!(block.last_request_bytes, 160_000);
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected lease block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn releasing_lease_allows_next_process() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(lease) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire first lease")
        else {
            panic!("expected acquired lease");
        };
        let rows_affected = runtime
            .release_provider_request_lease(&lease, /*now_ms*/ 2_000)
            .await
            .expect("release lease");
        assert_eq!(rows_affected, 1);

        let second = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_100,
            )
            .await
            .expect("acquire second lease");
        assert!(matches!(second, ProviderRequestLeaseDecision::Acquired(_)));

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn cooldown_only_check_ignores_active_lease() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let first = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire first lease");
        assert!(matches!(first, ProviderRequestLeaseDecision::Acquired(_)));

        let block = runtime
            .check_provider_request_cooldown(&key(), &small_preflight(), /*now_ms*/ 1_500)
            .await
            .expect("check cooldown");
        assert_eq!(block, None);

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn failed_non_rate_limit_result_clears_lease_without_cooldown() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(lease) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire lease")
        else {
            panic!("expected acquired lease");
        };
        let rows_affected = runtime
            .record_provider_request_result(
                &lease,
                ProviderRequestResult::Failed {
                    status: Some(500),
                    request_id: Some("req-500".to_string()),
                    retry_after_ms: Some(60_000),
                },
                /*now_ms*/ 2_000,
            )
            .await
            .expect("record non-rate-limit failure");
        assert_eq!(rows_affected, 1);

        let retry = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_500,
            )
            .await
            .expect("retry lease");
        assert!(matches!(retry, ProviderRequestLeaseDecision::Acquired(_)));

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn stale_owner_result_or_release_does_not_clear_active_owner() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(first) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 1_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire first lease")
        else {
            panic!("expected first lease");
        };
        let ProviderRequestLeaseDecision::Acquired(_second) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_500,
            )
            .await
            .expect("acquire second lease")
        else {
            panic!("expected second lease after first expires");
        };

        let result_rows = runtime
            .record_provider_request_result(
                &first,
                ProviderRequestResult::Success {
                    input_tokens: Some(1),
                    cached_input_tokens: Some(1),
                },
                /*now_ms*/ 3_000,
            )
            .await
            .expect("record stale result");
        assert_eq!(result_rows, 0);
        let release_rows = runtime
            .release_provider_request_lease(&first, /*now_ms*/ 3_100)
            .await
            .expect("release stale lease");
        assert_eq!(release_rows, 0);

        let blocked = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-c",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 3_200,
            )
            .await
            .expect("blocked by active second lease");
        match blocked {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Lease);
                assert_eq!(block.lease_owner.as_deref(), Some("worker-b"));
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected active lease block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn rate_limit_result_sets_cooldown_and_blocks_retry() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(lease) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire lease")
        else {
            panic!("expected acquired lease");
        };
        let rows_affected = runtime
            .record_provider_request_result(
                &lease,
                ProviderRequestResult::Failed {
                    status: Some(429),
                    request_id: Some("req-1".to_string()),
                    retry_after_ms: None,
                },
                /*now_ms*/ 2_000,
            )
            .await
            .expect("record 429");
        assert_eq!(rows_affected, 1);

        let retry = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_500,
            )
            .await
            .expect("retry lease");
        match retry {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Cooldown);
                assert_eq!(block.remaining_ms, 29_500);
                assert_eq!(block.last_status, Some(429));
                assert_eq!(block.last_request_id.as_deref(), Some("req-1"));
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected cooldown block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn success_result_records_provider_reported_cache_usage() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(lease) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire lease")
        else {
            panic!("expected acquired lease");
        };
        let rows_affected = runtime
            .record_provider_request_result(
                &lease,
                ProviderRequestResult::Success {
                    input_tokens: Some(17_136),
                    cached_input_tokens: Some(17_088),
                },
                /*now_ms*/ 2_000,
            )
            .await
            .expect("record success");
        assert_eq!(rows_affected, 1);

        let ProviderRequestLeaseDecision::Acquired(second) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_500,
            )
            .await
            .expect("acquire second lease")
        else {
            panic!("expected second lease");
        };
        runtime
            .record_provider_request_result(
                &second,
                ProviderRequestResult::Failed {
                    status: Some(429),
                    request_id: None,
                    retry_after_ms: None,
                },
                /*now_ms*/ 3_000,
            )
            .await
            .expect("record 429");

        let blocked = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-c",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 3_500,
            )
            .await
            .expect("blocked after cooldown");
        match blocked {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Cooldown);
                assert_eq!(block.last_input_tokens, 37_492);
                assert_eq!(block.last_cached_input_tokens, 20_000);
                assert_eq!(block.last_provider_input_tokens, 0);
                assert_eq!(block.last_provider_cached_input_tokens, 0);
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected cooldown block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn acquiring_new_lease_clears_stale_result_fields() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(first) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire first")
        else {
            panic!("expected first lease");
        };
        runtime
            .record_provider_request_result(
                &first,
                ProviderRequestResult::Success {
                    input_tokens: Some(88_303),
                    cached_input_tokens: Some(0),
                },
                /*now_ms*/ 2_000,
            )
            .await
            .expect("record cold-cache success");

        let ProviderRequestLeaseDecision::Acquired(_second) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_500,
            )
            .await
            .expect("acquire second")
        else {
            panic!("expected second lease");
        };

        let blocked = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-c",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 3_000,
            )
            .await
            .expect("blocked by active second lease");
        match blocked {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Lease);
                assert_eq!(block.last_status, None);
                assert_eq!(block.last_request_id, None);
                assert_eq!(block.last_provider_input_tokens, 0);
                assert_eq!(block.last_provider_cached_input_tokens, 0);
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected active lease block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn repeated_rate_limits_back_off_to_sixty_seconds() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(first) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire first")
        else {
            panic!("expected first lease");
        };
        runtime
            .record_provider_request_result(
                &first,
                ProviderRequestResult::Failed {
                    status: Some(429),
                    request_id: None,
                    retry_after_ms: None,
                },
                /*now_ms*/ 2_000,
            )
            .await
            .expect("record first 429");

        let ProviderRequestLeaseDecision::Acquired(second) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 33_000,
            )
            .await
            .expect("acquire after first cooldown")
        else {
            panic!("expected second lease");
        };
        runtime
            .record_provider_request_result(
                &second,
                ProviderRequestResult::Failed {
                    status: Some(429),
                    request_id: None,
                    retry_after_ms: None,
                },
                /*now_ms*/ 34_000,
            )
            .await
            .expect("record second 429");

        let blocked = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-c",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 35_000,
            )
            .await
            .expect("blocked after second cooldown");
        match blocked {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Cooldown);
                assert_eq!(block.remaining_ms, 59_000);
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected cooldown block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    #[tokio::test]
    async fn rate_limit_result_uses_retry_after_when_available() {
        let codex_home = unique_temp_dir();
        let runtime =
            StateRuntime::init_for_testing(codex_home.clone(), "test-provider".to_string())
                .await
                .expect("initialize runtime");

        let ProviderRequestLeaseDecision::Acquired(lease) = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-a",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 1_000,
            )
            .await
            .expect("acquire lease")
        else {
            panic!("expected acquired lease");
        };
        runtime
            .record_provider_request_result(
                &lease,
                ProviderRequestResult::Failed {
                    status: Some(429),
                    request_id: None,
                    retry_after_ms: Some(12_000),
                },
                /*now_ms*/ 2_000,
            )
            .await
            .expect("record 429");

        let retry = runtime
            .try_acquire_provider_request_lease(
                &key(),
                &preflight(),
                "worker-b",
                /*lease_ttl_ms*/ 10_000,
                /*now_ms*/ 2_500,
            )
            .await
            .expect("retry lease");
        match retry {
            ProviderRequestLeaseDecision::Blocked(block) => {
                assert_eq!(block.reason, ProviderRequestBlockReason::Cooldown);
                assert_eq!(block.remaining_ms, 11_500);
            }
            ProviderRequestLeaseDecision::Acquired(_) => panic!("expected cooldown block"),
        }

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }

    /// Owner decision 2026-10-11: a named account's rows move from its name to
    /// its credential. An active cooldown or lease carries over (the later
    /// cooldown wins, a credential row's own active lease stays), idle rows
    /// and the name's rows are dropped, and another provider's rows are kept.
    #[tokio::test]
    async fn named_account_rows_merge_into_the_credential_row() {
        let codex_home = unique_temp_dir();
        let runtime = StateRuntime::init_for_testing(codex_home.clone(), "p".to_string())
            .await
            .expect("initialize runtime");
        let insert = |provider: &'static str,
                      model: &'static str,
                      key: &'static str,
                      cooldown: i64,
                      status: i64,
                      lease: Option<(&'static str, i64)>| {
            let runtime = runtime.clone();
            async move {
                sqlx::query(
                    "INSERT INTO provider_request_state (provider_id, model, key_fingerprint, cooldown_until_ms, last_status, consecutive_429_count, lease_owner, lease_until_ms, updated_at_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)",
                )
                .bind(provider)
                .bind(model)
                .bind(key)
                .bind(cooldown)
                .bind(status)
                .bind(i64::from(status == 429))
                .bind(lease.map(|(owner, _)| owner))
                .bind(lease.map_or(0, |(_, until)| until))
                .execute(runtime.pool.as_ref())
                .await
                .expect("insert row");
            }
        };
        let now = 10_000;
        // `alpha`: cooling down longer than the credential row, with a lease
        // while the credential row holds none.
        insert(
            "p",
            "m",
            "account:alpha",
            50_000,
            429,
            Some(("w-alpha", 40_000)),
        )
        .await;
        insert("p", "m", "account-key:f", 20_000, 503, None).await;
        // `alpha` on a model with no credential row yet: moved as it is.
        insert("p", "m2", "account:alpha", 60_000, 429, None).await;
        // `beta`: a shorter cooldown and a lease that loses to the credential
        // row's own active one by then.
        insert(
            "p",
            "m",
            "account:beta",
            30_000,
            429,
            Some(("w-beta", 45_000)),
        )
        .await;
        // `gamma`: idle; dropped, not moved.
        insert("p", "m3", "account:gamma", 1_000, 200, None).await;
        // Another provider's `alpha` is not this account.
        insert("q", "m", "account:alpha", 70_000, 429, None).await;

        for name in ["account:alpha", "account:beta", "account:gamma"] {
            runtime
                .adopt_named_account_provider_request_key("p", name, "account-key:f", now)
                .await;
        }

        type Row = (String, String, String, i64, i64, Option<String>, i64);
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT provider_id, model, key_fingerprint, cooldown_until_ms, last_status, lease_owner, lease_until_ms FROM provider_request_state ORDER BY provider_id, model, key_fingerprint",
        )
        .fetch_all(runtime.pool.as_ref())
        .await
        .expect("rows");
        let row = |provider: &str,
                   model: &str,
                   key: &str,
                   cooldown,
                   status,
                   lease: Option<&str>,
                   until| {
            (
                provider.to_string(),
                model.to_string(),
                key.to_string(),
                cooldown,
                status,
                lease.map(str::to_string),
                until,
            )
        };
        assert_eq!(
            rows,
            vec![
                row(
                    "p",
                    "m",
                    "account-key:f",
                    50_000,
                    429,
                    Some("w-alpha"),
                    40_000
                ),
                row("p", "m2", "account-key:f", 60_000, 429, None, 0),
                row("q", "m", "account:alpha", 70_000, 429, None, 0),
            ]
        );

        let _ = tokio::fs::remove_dir_all(codex_home).await;
    }
}
