//! The provider-request throttle's key, and the cooldown check and result
//! record that every model request a session sends goes through (PF-84).
//!
//! The turn's own sampling requests also take the cross-process lease (see
//! `session::turn`). Compaction, the turn-completion check and memory
//! extraction only respect an active cooldown and record how they ended
//! (#509): a 429 on any of them holds every request on the key back, and a
//! cooldown holds them back too.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;
use std::time::Instant;

use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use codex_state::ProviderRequestBlock;
use codex_state::ProviderRequestBlockReason;
use codex_state::ProviderRequestKey;
use codex_state::ProviderRequestPreflight;
use codex_state::ProviderRequestResult;
use codex_state::StateDbFallback;
use sha2::Digest;
use sha2::Sha256;
use tracing::trace;
use tracing::warn;

use crate::Prompt;
use crate::StateDbHandle;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::turn_timing::now_unix_timestamp_ms;

/// The start of every message a throttle block ends a request with.
const BLOCK_MESSAGE_PREFIX: &str = "provider request blocked by local ";

/// How long reading a named account's fingerprint from the vault may hold a
/// request up. Past it the last fingerprint read in this process is used, or
/// the account is keyed by name.
const FINGERPRINT_BUDGET: Duration = Duration::from_secs(2);

/// How long a named account's fingerprint is reused before it is read again,
/// so a key replaced under the same name moves to its own throttle state.
const FINGERPRINT_FRESH_FOR: Duration = Duration::from_secs(60);

/// The throttle key for a request on `provider` (configured as `provider_id`)
/// for `model`.
///
/// Providers limit by credential, so the key is a salted fingerprint of the
/// credential (#482, and for named accounts the owner decision of
/// 2026-10-11): two named accounts holding the same key share one cooldown and
/// lease, and two keys of one provider never do. Nothing here creates or
/// writes the vault (#474), and no raw key is stored or logged.
pub(crate) async fn provider_request_key<A: std::fmt::Debug>(
    state_db: &StateDbHandle,
    codex_home: &Path,
    provider_id: &str,
    provider: &ModelProviderInfo,
    model: &str,
    auth_mode: impl FnOnce() -> A,
) -> ProviderRequestKey {
    let key_fingerprint = match provider_request_key_identity(provider, auth_mode) {
        ProviderKeyIdentity::Label(label) => label,
        ProviderKeyIdentity::NamedAccount(account) => {
            let by_name = named_account_label(&account.name);
            match named_account_fingerprint(codex_home, &account).await {
                Some(fingerprint) => {
                    let by_key = named_account_key(&fingerprint);
                    state_db
                        .adopt_named_account_provider_request_key(
                            provider_id,
                            &by_name,
                            &by_key,
                            now_unix_timestamp_ms(),
                        )
                        .await;
                    by_key
                }
                // An account added before fingerprints existed (S05), or a vault
                // that cannot be read in time: the account is keyed by name.
                None => by_name,
            }
        }
        ProviderKeyIdentity::EnvSecret { env_key, secret } => {
            match state_db.provider_request_key_salt().await {
                Ok(salt) => {
                    let salted = salted_env_key_fingerprint(&salt, &env_key, &secret);
                    state_db
                        .adopt_legacy_provider_request_key(
                            &legacy_env_key_fingerprint(&env_key, &secret),
                            &salted,
                            now_unix_timestamp_ms(),
                        )
                        .await;
                    salted
                }
                Err(error) => {
                    warn!(error = %format!("{error:#}"), "provider request key salt unavailable");
                    format!("env-salted:{env_key}:unavailable")
                }
            }
        }
    };
    ProviderRequestKey {
        provider_id: provider_id.to_string(),
        model: model.to_string(),
        key_fingerprint,
    }
}

/// The credential identity a provider's throttle state is kept under.
#[derive(PartialEq, Eq)]
pub(crate) enum ProviderKeyIdentity {
    /// A label that holds no credential material.
    Label(String),
    /// A named account: keyed by the vault's salted fingerprint of what it
    /// holds, or by its name when there is none.
    NamedAccount(NamedProviderAccount),
    /// An API key from the environment, fingerprinted with the home's salt.
    EnvSecret {
        env_key: String,
        secret: zeroize::Zeroizing<String>,
    },
}

/// Never shows the key.
impl std::fmt::Debug for ProviderKeyIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label(label) => f.debug_tuple("Label").field(label).finish(),
            Self::NamedAccount(account) => f.debug_tuple("NamedAccount").field(account).finish(),
            Self::EnvSecret { env_key, .. } => f
                .debug_struct("EnvSecret")
                .field("env_key", env_key)
                .finish_non_exhaustive(),
        }
    }
}

pub(crate) fn provider_request_key_identity<A: std::fmt::Debug>(
    provider: &ModelProviderInfo,
    auth_mode: impl FnOnce() -> A,
) -> ProviderKeyIdentity {
    // A named account never uses the provider's environment key, so that key
    // is not part of its identity.
    if let Some(account) = provider.account.as_ref() {
        return ProviderKeyIdentity::NamedAccount(account.clone());
    }
    let label = if let Some(env_key) = provider.env_key.as_deref() {
        match std::env::var(env_key)
            .ok()
            .filter(|value| !value.trim().is_empty())
        {
            Some(secret) => {
                return ProviderKeyIdentity::EnvSecret {
                    env_key: env_key.to_string(),
                    secret: zeroize::Zeroizing::new(secret),
                };
            }
            // Do not hit the vault on the hot path just to fingerprint throttle state.
            // The actual request path still resolves the stored key and will fail
            // normally if it is missing or invalid.
            None => format!("stored:{env_key}"),
        }
    } else if provider.experimental_bearer_token.is_some() {
        "config:bearer-token".to_string()
    } else if provider.auth.is_some() {
        "config:auth-command".to_string()
    } else if provider.aws.is_some() {
        "config:aws".to_string()
    } else {
        format!("auth-mode:{:?}", auth_mode())
    };
    ProviderKeyIdentity::Label(label)
}

/// The key a named account was kept under before throttling went by
/// credential, and still is while it has no fingerprint.
pub(crate) fn named_account_label(name: &str) -> String {
    format!("account:{name}")
}

/// A named account's key: the vault's salted fingerprint of its credential.
/// It names no account, so every name holding the credential shares it.
pub(crate) fn named_account_key(fingerprint: &str) -> String {
    format!("account-key:{fingerprint}")
}

type FingerprintKey = (PathBuf, String, String);

/// What this process knows of one named account's fingerprint.
#[derive(Default)]
struct FingerprintEntry {
    /// The last read that finished: when, and what it found (`None`: the
    /// account has no fingerprint). A read that failed keeps what was found
    /// before.
    read: Option<(Instant, Option<String>)>,
    /// Set while a read runs: when it started, and a receiver that changes
    /// when it finishes.
    reading: Option<(Instant, tokio::sync::watch::Receiver<()>)>,
    /// The fallback to the name has been logged.
    warned: bool,
}

/// Fingerprints read in this process. A request reads the vault at most once
/// a minute per account, and never twice at the same time.
static NAMED_ACCOUNT_FINGERPRINTS: LazyLock<Mutex<HashMap<FingerprintKey, FingerprintEntry>>> =
    LazyLock::new(Mutex::default);

fn fingerprints() -> std::sync::MutexGuard<'static, HashMap<FingerprintKey, FingerprintEntry>> {
    NAMED_ACCOUNT_FINGERPRINTS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Ends a background fingerprint read, however its task ends (even if it
/// never runs), so the account can be read again.
struct FinishRead {
    key: FingerprintKey,
    /// `Some(found)` when the vault answered, `None` when it could not be read.
    found: Option<Option<String>>,
    done: tokio::sync::watch::Sender<()>,
}

impl FinishRead {
    /// End the read with what it found.
    fn with(mut self, found: Option<Option<String>>) {
        self.found = found;
    }
}

impl Drop for FinishRead {
    fn drop(&mut self) {
        let mut entries = fingerprints();
        let entry = entries.entry(self.key.clone()).or_default();
        entry.reading = None;
        let found = match self.found.take() {
            Some(found) => found,
            // A failed read never replaces a fingerprint read before.
            None => entry.read.take().and_then(|(_, fingerprint)| fingerprint),
        };
        entry.read = Some((Instant::now(), found));
        drop(entries);
        self.done.send_replace(());
    }
}

/// The vault's per-home salted fingerprint of a named account's credential
/// (the one S05 records in the ledger). The read never creates a vault, key
/// or salt and never writes (#474).
///
/// A fingerprint read in the last minute is used as it is. An older one is
/// used at once while a background read refreshes it. With none, a request
/// waits for the read, but no longer than `FINGERPRINT_BUDGET` after the read
/// started, so a hung vault costs that once, not on every request. Only one
/// read runs per account. `None` when the account holds nothing, its vault has
/// no fingerprint salt yet (every account in it was added before S05), or the
/// vault has not answered: the account is then keyed by name.
async fn named_account_fingerprint(
    codex_home: &Path,
    account: &NamedProviderAccount,
) -> Option<String> {
    let key: FingerprintKey = (
        codex_home.to_path_buf(),
        account.provider_id.clone(),
        account.name.clone(),
    );
    let mut start_read = None;
    let wait = {
        let mut entries = fingerprints();
        let entry = entries.entry(key.clone()).or_default();
        let known = entry.read.clone();
        if let Some((read_at, fingerprint)) = &known
            && read_at.elapsed() < FINGERPRINT_FRESH_FOR
        {
            return fingerprint.clone();
        }
        let (started, reading) = match &entry.reading {
            Some(reading) => reading.clone(),
            None => {
                let (done, reading) = tokio::sync::watch::channel(());
                entry.reading = Some((Instant::now(), reading.clone()));
                start_read = Some(FinishRead {
                    key: key.clone(),
                    found: None,
                    done,
                });
                (Instant::now(), reading)
            }
        };
        match known {
            // An older fingerprint serves while the read refreshes it.
            Some((_, Some(fingerprint))) => Err(fingerprint),
            _ => Ok(FINGERPRINT_BUDGET
                .checked_sub(started.elapsed())
                .map(|left| (left, reading))),
        }
    };
    // Started with the lock released: `FinishRead` takes it when dropped.
    if let Some(finish) = start_read {
        let (home, selected) = (codex_home.to_path_buf(), account.clone());
        drop(tokio::task::spawn_blocking(move || {
            finish.with(read_named_account_fingerprint(home, &selected).ok());
        }));
    }
    let wait = match wait {
        Ok(wait) => wait,
        Err(older) => return Some(older),
    };
    let finished = match wait {
        Some((left, mut reading)) => tokio::time::timeout(left, reading.changed()).await.is_ok(),
        None => false,
    };
    let mut entries = fingerprints();
    let entry = entries.entry(key).or_default();
    let fingerprint = entry
        .read
        .as_ref()
        .and_then(|(_, fingerprint)| fingerprint.clone());
    if fingerprint.is_none() && !entry.warned {
        entry.warned = true;
        warn!(
            provider = %account.provider_id,
            read_in_time = finished,
            "a named account has no credential fingerprint yet; throttling it by name"
        );
    }
    fingerprint
}

/// `Ok(None)`: the account has no fingerprint. `Err`: the vault could not be
/// read.
fn read_named_account_fingerprint(
    codex_home: PathBuf,
    account: &NamedProviderAccount,
) -> Result<Option<String>, ()> {
    let Ok(name) = codex_vault::ProviderAccountName::parse(&account.name) else {
        return Ok(None);
    };
    let vault = codex_vault::Vault::new(codex_home);
    // Checked first, so the read below never sets up a vault's storage.
    if vault.key_storage() == codex_vault::VaultKeyStorage::NotInitialized {
        return Ok(None);
    }
    vault
        .provider_account_fingerprint(&account.provider_id, &name)
        .map_err(|_| ())
}

/// #482: an environment key's throttle identity, salted per home so the same
/// key gives a different value on every machine.
pub(crate) fn salted_env_key_fingerprint(salt: &[u8], env_key: &str, secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"corbanu-provider-request-key-v1\0");
    hasher.update(salt);
    hasher.update(b"\0");
    hasher.update(env_key.as_bytes());
    hasher.update(b"\0");
    hasher.update(secret.as_bytes());
    format!("env-salted:{env_key}:{}", hex_prefix(&hasher.finalize()))
}

/// The unsalted identity older builds kept the key's throttle row under; it
/// is only computed to move that row to the salted key.
pub(crate) fn legacy_env_key_fingerprint(env_key: &str, secret: &str) -> String {
    format!(
        "env:{env_key}:{}",
        hex_prefix(&Sha256::digest(secret.as_bytes()))
    )
}

fn hex_prefix(digest: &[u8]) -> String {
    digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn format_provider_request_block(
    key: &ProviderRequestKey,
    block: &ProviderRequestBlock,
) -> String {
    let reason = match block.reason {
        ProviderRequestBlockReason::Cooldown => "cooldown",
        ProviderRequestBlockReason::Lease => "active request lease",
    };
    let wait_seconds = (block.remaining_ms.max(0) + 999) / 1000;
    let status = block
        .last_status
        .map(|status| status.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let request_id = block
        .last_request_id
        .as_deref()
        .map(|id| format!(", request id: {id}"))
        .unwrap_or_default();
    let provider_cache = if block.last_provider_input_tokens > 0 {
        format!(
            " Last provider usage: input={} cached_input={}.",
            block.last_provider_input_tokens, block.last_provider_cached_input_tokens
        )
    } else {
        String::new()
    };
    format!(
        "{BLOCK_MESSAGE_PREFIX}{reason} for {}/{} ({}) for about {wait_seconds}s. Last status: {status}{request_id}. Last preflight: input={} cached_input={} request_bytes={}.{provider_cache} Wait, compact, switch provider/model, or start a fresh thread before retrying.",
        key.provider_id,
        key.model,
        key.key_fingerprint,
        block.last_input_tokens,
        block.last_cached_input_tokens,
        block.last_request_bytes
    )
}

/// Whether `err` is a request the throttle held back, which no retry helps.
pub(crate) fn is_provider_request_block(err: &CodexErr) -> bool {
    matches!(err.details(), CodexErrorDetails::InvalidRequest(message) if message.starts_with(BLOCK_MESSAGE_PREFIX))
}

pub(crate) fn provider_request_result_kind(result: &ProviderRequestResult) -> &'static str {
    match result {
        ProviderRequestResult::Success { .. } => "success",
        ProviderRequestResult::Failed { .. } => "failed",
    }
}

pub(crate) fn provider_request_result_from_error(err: &CodexErr) -> ProviderRequestResult {
    ProviderRequestResult::Failed {
        status: err.http_status_code_value(),
        request_id: provider_request_error_request_id(err),
        retry_after_ms: provider_request_error_retry_after_ms(err),
    }
}

fn provider_request_error_request_id(err: &CodexErr) -> Option<String> {
    match err.details() {
        CodexErrorDetails::RetryLimit(err) => err.request_id.clone(),
        CodexErrorDetails::UnexpectedStatus(err) => err.request_id.clone(),
        CodexErrorDetails::ResponseStreamFailed(err) => err.request_id.clone(),
        _ => None,
    }
}

fn provider_request_error_retry_after_ms(err: &CodexErr) -> Option<i64> {
    match err.details() {
        CodexErrorDetails::RetryLimit(err) => err.retry_after_ms,
        _ => None,
    }
}

/// #476: a request sent without a lease still records how it ended, so a 429
/// holds every later request on the key back for the cooldown. A failure
/// without an HTTP status (an interrupt, a network error) says nothing about
/// the key and is not recorded.
pub(crate) async fn record_unleased_provider_request_result(
    state_db: &StateDbHandle,
    key: &ProviderRequestKey,
    result: ProviderRequestResult,
) {
    if matches!(result, ProviderRequestResult::Failed { status: None, .. }) {
        return;
    }
    let result_kind = provider_request_result_kind(&result);
    let recorded = state_db
        .record_provider_request_throttle_outcome(key, result, now_unix_timestamp_ms())
        .await;
    trace!(
        provider = %key.provider_id,
        model = %key.model,
        key_fingerprint = %key.key_fingerprint,
        result = result_kind,
        state_db_fallback = ?recorded.state_db_fallback,
        "recorded unleased provider request result"
    );
}

/// Say once per session that the shared state DB could not be written, so the
/// provider-request throttle went on from this process's own state (#351).
pub(crate) async fn notify_state_db_fallback(
    sess: &Session,
    turn_context: &TurnContext,
    fallback: Option<StateDbFallback>,
) {
    let Some(fallback) = fallback else {
        return;
    };
    if sess
        .services
        .state_db_fallback_noticed
        .swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let cause = match fallback {
        StateDbFallback::Busy => "is busy (another session or process is writing to it)",
        StateDbFallback::ReadOnly => "is read-only",
        StateDbFallback::Unavailable => "can't be written",
    };
    let message = format!(
        "The shared state database {cause}. Requests continue, and the provider request limits this Corbanu process knows about still apply; limits shared with other Corbanu processes resume once the database can be written."
    );
    sess.send_event(turn_context, EventMsg::Warning(WarningEvent { message }))
        .await;
}

/// The home a turn's credentials are read from and its auth mode: the turn's
/// own auth manager (it follows a mid-session account switch), else the
/// session's.
pub(crate) fn turn_auth(
    sess: &Session,
    turn_context: &TurnContext,
) -> (PathBuf, Option<codex_protocol::auth::AuthMode>) {
    let auth_manager = turn_context
        .auth_manager
        .clone()
        .unwrap_or_else(|| sess.services.auth_manager());
    (
        auth_manager.codex_home().to_path_buf(),
        auth_manager.auth_mode(),
    )
}

/// Which model request outside the turn's sampling loop is being admitted.
#[derive(Clone, Copy, Debug)]
pub(crate) enum AuxiliaryRequest {
    Compaction,
    CompletionCheck,
    MemoryExtraction,
}

impl AuxiliaryRequest {
    fn as_str(self) -> &'static str {
        match self {
            Self::Compaction => "compaction",
            Self::CompletionCheck => "turn completion check",
            Self::MemoryExtraction => "memory extraction",
        }
    }
}

/// A model request outside the turn's sampling loop that the throttle let
/// through (#509). It never takes the lease; it records how it ended, so its
/// 429 holds every request on the key back.
pub(crate) struct AdmittedProviderRequest {
    state_db: StateDbHandle,
    key: ProviderRequestKey,
}

/// What to admit a request outside the turn's sampling loop on.
pub(crate) struct AuxiliaryRoute<'a> {
    pub(crate) state_db: Option<StateDbHandle>,
    pub(crate) codex_home: &'a Path,
    pub(crate) provider_id: &'a str,
    pub(crate) provider: &'a ModelProviderInfo,
    pub(crate) model: &'a str,
    pub(crate) auth_mode: Option<codex_protocol::auth::AuthMode>,
    pub(crate) thread_id: Option<String>,
    pub(crate) turn_id: Option<String>,
}

impl AdmittedProviderRequest {
    /// Check the key's cooldown before sending `prompt`. `Ok(None)` without a
    /// state DB; an `InvalidRequest` error while a cooldown holds the key back.
    pub(crate) async fn admit(
        request: AuxiliaryRequest,
        route: AuxiliaryRoute<'_>,
        prompt: &Prompt,
    ) -> CodexResult<(Option<Self>, Option<StateDbFallback>)> {
        let Some(state_db) = route.state_db else {
            return Ok((None, None));
        };
        let key = provider_request_key(
            &state_db,
            route.codex_home,
            route.provider_id,
            route.provider,
            route.model,
            || route.auth_mode,
        )
        .await;
        // A rough estimate. The check stores it as the key's last preflight, so a
        // later block message may show this request's figures, not the turn's.
        let input_bytes = serde_json::to_vec(&prompt.input).map_or(0, |bytes| bytes.len());
        let preflight = ProviderRequestPreflight {
            input_tokens: i64::try_from(input_bytes / 4).unwrap_or(i64::MAX),
            cached_input_tokens: 0,
            request_bytes: i64::try_from(input_bytes).unwrap_or(i64::MAX),
            thread_id: route.thread_id,
            turn_id: route.turn_id,
        };
        let checked = state_db
            .check_provider_request_throttle(&key, &preflight, now_unix_timestamp_ms())
            .await;
        if let Some(block) = checked.decision {
            warn!(
                request = request.as_str(),
                provider = %key.provider_id,
                model = %key.model,
                key_fingerprint = %key.key_fingerprint,
                remaining_ms = block.remaining_ms,
                last_status = ?block.last_status,
                "provider request blocked by local hammer-reduction state"
            );
            return Err(CodexErr::InvalidRequest(format_provider_request_block(
                &key, &block,
            )));
        }
        Ok((Some(Self { state_db, key }), checked.state_db_fallback))
    }

    /// Admit a request the turn sends besides its sampling requests
    /// (compaction, the completion check).
    pub(crate) async fn admit_for_turn(
        request: AuxiliaryRequest,
        sess: &Session,
        turn_context: &TurnContext,
        prompt: &Prompt,
    ) -> CodexResult<Option<Self>> {
        let (codex_home, auth_mode) = turn_auth(sess, turn_context);
        let (admitted, fallback) = Self::admit(
            request,
            AuxiliaryRoute {
                state_db: sess.state_db(),
                codex_home: &codex_home,
                provider_id: turn_context.config.model_provider_id.as_str(),
                provider: turn_context.provider.info(),
                model: turn_context.model_info.slug.as_str(),
                auth_mode,
                thread_id: Some(sess.thread_id.to_string()),
                turn_id: Some(turn_context.sub_id.clone()),
            },
            prompt,
        )
        .await?;
        notify_state_db_fallback(sess, turn_context, fallback).await;
        Ok(admitted)
    }

    pub(crate) async fn record(&self, result: ProviderRequestResult) {
        record_unleased_provider_request_result(&self.state_db, &self.key, result).await;
    }

    /// Record how a request ended: `None` for a success, which carries no
    /// usage here.
    pub(crate) async fn record_error(&self, error: Option<&CodexErr>) {
        let result = match error {
            None => ProviderRequestResult::Success {
                input_tokens: None,
                cached_input_tokens: None,
            },
            Some(error) => provider_request_result_from_error(error),
        };
        self.record(result).await;
    }
}

/// Record `outcome` when a request was admitted.
pub(crate) async fn record_auxiliary_outcome<T>(
    admitted: Option<&AdmittedProviderRequest>,
    outcome: &CodexResult<T>,
) {
    record_auxiliary_error(admitted, outcome.as_ref().err()).await;
}

/// Record how a request ended (`None`: a success) when it was admitted.
pub(crate) async fn record_auxiliary_error(
    admitted: Option<&AdmittedProviderRequest>,
    error: Option<&CodexErr>,
) {
    if let Some(admitted) = admitted {
        admitted.record_error(error).await;
    }
}

#[cfg(test)]
#[path = "provider_throttle_tests.rs"]
pub(crate) mod tests;
