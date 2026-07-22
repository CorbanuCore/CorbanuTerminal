use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Context;
use futures::Stream;
use futures::StreamExt;
use teloxide::ApiError;
use teloxide::Bot;
use teloxide::RequestError;
use teloxide::backoff::exponential_backoff_strategy;
use teloxide::error_handlers::ErrorHandler;
use teloxide::stop::StopToken;
use teloxide::types::AllowedUpdate;
use teloxide::types::Update;
use teloxide::update_listeners::AsUpdateStream;
use teloxide::update_listeners::Polling;
use teloxide::update_listeners::UpdateListener;
use tracing::error;
use tracing::warn;

use crate::error::PollingBackoff;
use crate::error::TelegramError;
use crate::error::is_http_409_conflict;

type GuardedStream<'a> =
    std::pin::Pin<Box<dyn Stream<Item = Result<Update, RequestError>> + Send + 'a>>;

#[derive(Debug)]
struct PollingState {
    failures: PollingBackoff,
}

impl PollingState {
    fn new(max_consecutive_failures: u32) -> Self {
        Self {
            failures: PollingBackoff::new(
                Duration::from_secs(1),
                Duration::from_secs(64),
                max_consecutive_failures,
            ),
        }
    }

    fn record_success(&mut self) {
        self.failures.record_success();
    }

    fn record_failure(&mut self, error: &RequestError) -> bool {
        if is_polling_conflict(error) {
            error!("Telegram polling conflict: another poller is already using this bot token");
            return true;
        }

        match self.failures.record_failure() {
            Ok(delay) => {
                warn!(
                    retry_after_secs = delay.as_secs(),
                    "Telegram polling failed; teloxide will retry with backoff"
                );
                false
            }
            Err(TelegramError::PollingFailureCapExceeded) => {
                error!("Telegram polling exceeded configured consecutive failure cap");
                true
            }
            Err(err) => {
                error!("Telegram polling failed: {err}");
                true
            }
        }
    }
}

pub struct GuardedPolling {
    inner: Polling<Bot>,
    state: Arc<Mutex<PollingState>>,
    fatal: FatalPollingFlag,
}

impl GuardedPolling {
    pub fn stop_token(&mut self) -> StopToken {
        self.inner.stop_token()
    }

    pub fn fatal_flag(&self) -> FatalPollingFlag {
        self.fatal.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FatalPollingFlag {
    fatal: Arc<AtomicBool>,
}

impl FatalPollingFlag {
    fn mark_fatal(&self) {
        self.fatal.store(true, Ordering::SeqCst);
    }

    pub fn is_fatal(&self) -> bool {
        self.fatal.load(Ordering::SeqCst)
    }
}

/// HTTP client ceiling for the long-poll reader. It must exceed the
/// `getUpdates` long-poll timeout below plus network slack: teloxide's
/// default client uses a 17s overall timeout for a 10s poll, so a normal
/// long-poll response arrives well inside this window. This bound only
/// matters when the connection stalls mid-read — without it a wedged socket
/// could hang the poller silently.
const POLLING_CLIENT_TIMEOUT: Duration = Duration::from_secs(17);

/// Build the bot handle dedicated to long-polling, on an HTTP client whose
/// timeout accommodates long-poll reads. Kept separate from the action-style
/// outbound client in `lib.rs` because the two have incompatible timeout
/// requirements on a shared reqwest client.
pub fn polling_bot(token: String) -> anyhow::Result<Bot> {
    let client = teloxide::net::default_reqwest_settings()
        .build()
        .context("failed to build Telegram polling HTTP client")?;
    Ok(Bot::with_client(token, client))
}

pub async fn guarded_polling(bot: Bot, max_consecutive_failures: u32) -> GuardedPolling {
    debug_assert!(
        POLLING_CLIENT_TIMEOUT > Duration::from_secs(10),
        "polling client timeout must exceed the long-poll timeout"
    );
    let inner = Polling::builder(bot)
        .timeout(Duration::from_secs(10))
        .backoff_strategy(exponential_backoff_strategy)
        .delete_webhook()
        .await
        .build();
    GuardedPolling {
        inner,
        state: Arc::new(Mutex::new(PollingState::new(max_consecutive_failures))),
        fatal: FatalPollingFlag::default(),
    }
}

pub fn listener_error_handler(
    stop_token: StopToken,
    fatal: FatalPollingFlag,
) -> Arc<impl ErrorHandler<RequestError> + Send + Sync> {
    Arc::new(move |error: RequestError| {
        let stop_token = stop_token.clone();
        let fatal = fatal.clone();
        async move {
            if is_polling_conflict(&error) {
                error!("Telegram polling stopped because another poller is using this bot token");
                fatal.mark_fatal();
                stop_token.stop();
            } else {
                warn!("Telegram polling listener error: {error}");
            }
        }
    })
}

impl<'a> AsUpdateStream<'a> for GuardedPolling {
    type StreamErr = RequestError;
    type Stream = GuardedStream<'a>;

    fn as_stream(&'a mut self) -> Self::Stream {
        let state = Arc::clone(&self.state);
        let stop_token = self.inner.stop_token();
        let fatal = self.fatal.clone();
        Box::pin(self.inner.as_stream().map(move |result| {
            let should_stop = {
                let mut state = match state.lock() {
                    Ok(state) => state,
                    Err(poisoned) => {
                        warn!("Telegram polling state mutex was poisoned; continuing");
                        poisoned.into_inner()
                    }
                };
                match &result {
                    Ok(_) => {
                        state.record_success();
                        false
                    }
                    Err(error) => state.record_failure(error),
                }
            };
            if should_stop {
                fatal.mark_fatal();
                stop_token.stop();
            }
            result
        }))
    }
}

impl UpdateListener for GuardedPolling {
    type Err = RequestError;

    fn stop_token(&mut self) -> StopToken {
        self.inner.stop_token()
    }

    fn hint_allowed_updates(&mut self, hint: &mut dyn Iterator<Item = AllowedUpdate>) {
        self.inner.hint_allowed_updates(hint);
    }
}

fn is_polling_conflict(error: &RequestError) -> bool {
    matches!(
        error,
        RequestError::Api(ApiError::TerminatedByOtherGetUpdates)
    ) || is_http_409_conflict(error)
}
