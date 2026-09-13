//! Native sampling ownership glue. Replay, exact money and retention stay in state.
use crate::config::AccountingMode;
use codex_api::AnthropicTokenPresence;
use codex_api::AnthropicUsagePatch;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_state::StateRuntime;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::Attempt;
use codex_state::accounting::Dialect;
use codex_state::accounting::Observation;
use codex_state::accounting::Patch;
use codex_state::accounting::Presence;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use uuid::Uuid;

#[cfg(test)]
#[path = "accounting_policy_tests.rs"]
mod policy_tests;
#[path = "accounting_prices.rs"]
mod prices;
#[path = "accounting_transport.rs"]
pub(crate) mod transport;

pub(crate) const FAILURE: &str =
    "Native Anthropic accounting failed; request stopped without a repair send";
pub(crate) type Slot = Arc<Mutex<Option<Arc<Sampling>>>>;

// The facade accepts explicit as-of values. Serialize this process's collector
// transactions before sampling time so concurrent native children cannot commit
// a later checkpoint ahead of an older local operation. Never hold across HTTP;
// external writers still use the store's fail-visible validation contract.
static WRITES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

pub(crate) fn read_slot(slot: &Slot) -> Result<Option<Arc<Sampling>>, CodexErr> {
    match slot.lock() {
        Ok(value) => Ok(value.clone()),
        Err(poison) => {
            if let Some(stale) = poison.into_inner().take() {
                stale.reject();
            }
            Err(CodexErr::Fatal(FAILURE.into()))
        }
    }
}

pub(crate) struct SamplingScope(Slot);

impl SamplingScope {
    pub(crate) fn attach(slot: Slot, sampling: Option<Arc<Sampling>>) -> Result<Self, CodexErr> {
        {
            match slot.lock() {
                Ok(mut value) => *value = sampling,
                Err(poison) => {
                    if let Some(stale) = poison.into_inner().take() {
                        stale.reject();
                    }
                    if let Some(incoming) = sampling {
                        incoming.reject();
                    }
                    return Err(CodexErr::Fatal(FAILURE.into()));
                }
            }
        }
        Ok(Self(slot))
    }
}

impl Drop for SamplingScope {
    fn drop(&mut self) {
        match self.0.lock() {
            Ok(mut value) => *value = None,
            Err(poison) => {
                if let Some(stale) = poison.into_inner().take() {
                    stale.reject();
                }
            }
        }
    }
}

pub(crate) struct Sampling {
    runtime: Arc<StateRuntime>,
    owner: ThreadId,
    turn: String,
    request: Uuid,
    scope: Uuid,
    endpoint: String,
    previous: Mutex<Option<Uuid>>,
    failed: AtomicBool,
}

// A cancelled SQL future may already have committed. Never reuse that sampling
// with a guessed predecessor; release the permit normally and fail it closed.
struct Completion<'a> {
    sampling: &'a Sampling,
    complete: bool,
}

impl Drop for Completion<'_> {
    fn drop(&mut self) {
        if !self.complete {
            self.sampling.reject();
        }
    }
}

impl Sampling {
    pub(crate) async fn start(
        runtime: Arc<StateRuntime>,
        owner: ThreadId,
        turn: String,
        mode: &AccountingMode,
    ) -> Result<Arc<Self>, CodexErr> {
        let AccountingMode::DirectAnthropic {
            scope,
            approved_endpoint,
        } = mode
        else {
            return Err(CodexErr::Fatal(FAILURE.into()));
        };
        let endpoint =
            url::Url::parse(approved_endpoint).map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        if !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !matches!(endpoint.scheme(), "http" | "https")
        {
            return Err(CodexErr::Fatal(FAILURE.into()));
        }
        let _write = WRITES
            .acquire()
            .await
            .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        AccountingStore::open(&runtime, now())
            .await
            .map_err(|_| CodexErr::Fatal(FAILURE.into()))?;
        Ok(Arc::new(Self {
            runtime,
            owner,
            turn,
            request: Uuid::new_v4(),
            scope: *scope,
            endpoint: format!("{}/messages", approved_endpoint.trim_end_matches('/')),
            previous: Mutex::new(None),
            failed: AtomicBool::new(false),
        }))
    }

    pub(crate) fn check(&self) -> Result<(), CodexErr> {
        if self.failed.load(Ordering::Acquire) {
            Err(CodexErr::Fatal(FAILURE.into()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn reject(&self) {
        self.failed.store(true, Ordering::Release);
    }

    async fn admit(&self, model: &str, endpoint: &str) -> anyhow::Result<Attempt> {
        self.check()?;
        anyhow::ensure!(endpoint == self.endpoint, "accounting route mismatch");
        let _write = WRITES.acquire().await?;
        self.check()?;
        let mut completion = Completion {
            sampling: self,
            complete: false,
        };
        let previous = *self.previous.lock().map_err(|_| {
            self.reject();
            anyhow::anyhow!(FAILURE)
        })?;
        // The facade borrows its runtime. Reopening validates/maintains through
        // its public contract; never fabricate an attached or Active handle.
        let store = AccountingStore::open(&self.runtime, now()).await?;
        let dispatched_at = now();
        let attempt = Attempt {
            attempt_id: Uuid::new_v4(),
            request_id: self.request,
            thread_id: self.owner,
            turn: self.turn.clone(),
            retry_of: previous,
            provider: "anthropic".into(),
            model: model.into(),
            scope: self.scope,
            dialect: Dialect::NativeAnthropic,
            dispatched_at_ms: dispatched_at.try_into()?,
        };
        let prices = prices::original(model, self.scope, dispatched_at)?;
        store.admit(self.owner, &attempt, &prices, now()).await?;
        *self.previous.lock().map_err(|_| {
            self.reject();
            anyhow::anyhow!(FAILURE)
        })? = Some(attempt.attempt_id);
        completion.complete = true;
        Ok(attempt)
    }

    async fn observe(
        &self,
        attempt: &Attempt,
        source: Uuid,
        position: i64,
        usage: AnthropicUsagePatch,
    ) -> anyhow::Result<()> {
        self.check()?;
        let patch = Patch {
            input: presence(usage.input_tokens)?,
            read: presence(usage.cache_read_input_tokens)?,
            write: presence(usage.cache_creation_input_tokens)?,
            output: presence(usage.output_tokens)?,
            ..Patch::default()
        };
        let position = position.try_into()?;
        let observation = Observation {
            source,
            revision: position,
            sequence: position,
            patch,
        };
        let _write = WRITES.acquire().await?;
        self.check()?;
        let mut completion = Completion {
            sampling: self,
            complete: false,
        };
        let store = AccountingStore::open(&self.runtime, now()).await?;
        store
            .observe(self.owner, attempt, &[observation], now())
            .await?;
        completion.complete = true;
        Ok(())
    }
}

fn presence(value: AnthropicTokenPresence) -> anyhow::Result<Presence> {
    Ok(match value {
        AnthropicTokenPresence::Missing => Presence::Missing,
        AnthropicTokenPresence::Null => Presence::Null,
        AnthropicTokenPresence::Number(value) => Presence::Number(value.try_into()?),
    })
}

fn now() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
