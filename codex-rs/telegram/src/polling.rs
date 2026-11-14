use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Context;
use futures::Stream;
use teloxide::ApiError;
use teloxide::Bot;
use teloxide::RequestError;
use teloxide::error_handlers::ErrorHandler;
use teloxide::errors::AsResponseParameters;
use teloxide::payloads::GetUpdatesSetters;
use teloxide::requests::Requester;
use teloxide::stop::StopFlag;
use teloxide::stop::StopToken;
use teloxide::stop::mk_stop_token;
use teloxide::types::AllowedUpdate;
use teloxide::types::Update;
use teloxide::update_listeners::AsUpdateStream;
use teloxide::update_listeners::UpdateListener;
use tracing::error;
use tracing::warn;

use crate::error::PollingBackoff;
use crate::error::TelegramError;
use crate::error::is_http_409_conflict;

type GuardedStream<'a> =
    std::pin::Pin<Box<dyn Stream<Item = Result<Update, RequestError>> + Send + 'a>>;

pub struct GuardedPolling {
    bot: Bot,
    replay: Vec<Update>,
    failures: PollingBackoff,
    fatal: FatalPollingFlag,
    stop_token: StopToken,
    stop_flag: StopFlag,
    allowed_updates: Option<Vec<AllowedUpdate>>,
}

impl GuardedPolling {
    pub fn stop_token(&mut self) -> StopToken {
        self.stop_token.clone()
    }

    pub fn fatal_flag(&self) -> FatalPollingFlag {
        self.fatal.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FatalPollingFlag {
    // 0 = running, 1 = conflict, 2 = failure cap.
    fatal: Arc<AtomicU8>,
}

impl FatalPollingFlag {
    fn mark_conflict(&self) {
        self.fatal.store(1, Ordering::SeqCst);
    }

    pub fn error(&self) -> Option<TelegramError> {
        match self.fatal.load(Ordering::SeqCst) {
            1 => Some(TelegramError::PollingConflict),
            2 => Some(TelegramError::PollingFailureCapExceeded),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn is_fatal(&self) -> bool {
        self.error().is_some()
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

pub async fn guarded_polling(
    bot: Bot,
    max_consecutive_failures: u32,
    replay: Vec<Update>,
) -> GuardedPolling {
    debug_assert!(
        POLLING_CLIENT_TIMEOUT > Duration::from_secs(10),
        "polling client timeout must exceed the long-poll timeout"
    );
    match bot.get_webhook_info().await {
        Ok(info) if info.url.is_some() => {
            if let Err(err) = bot.delete_webhook().await {
                warn!("failed to delete Telegram webhook: {err}");
            }
        }
        Ok(_) => {}
        Err(err) => warn!("failed to get Telegram webhook info: {err}"),
    }
    let (stop_token, stop_flag) = mk_stop_token();
    GuardedPolling {
        bot,
        replay,
        failures: PollingBackoff::new(
            Duration::from_secs(1),
            Duration::from_secs(64),
            max_consecutive_failures,
        ),
        fatal: FatalPollingFlag::default(),
        stop_token,
        stop_flag,
        allowed_updates: None,
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
                fatal.mark_conflict();
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
        // Observe whole responses: teloxide's update-only stream hides empty
        // successful polls, which must reset the consecutive-failure counter.
        let replay = std::mem::take(&mut self.replay).into_iter();
        Box::pin(futures::stream::unfold(
            (self, replay, 0, Duration::ZERO),
            |(polling, mut buffered, mut offset, mut delay)| async move {
                loop {
                    if let Some(update) = buffered.next() {
                        return Some((Ok(update), (polling, buffered, offset, delay)));
                    }
                    if polling.stop_flag.is_stopped() {
                        // Acknowledge only the fetched batch, never replay ids.
                        // Failure here must not replace the original stop reason.
                        if let Err(err) = polling
                            .bot
                            .get_updates()
                            .offset(offset)
                            .timeout(0)
                            .limit(1)
                            .await
                        {
                            warn!("failed to acknowledge Telegram polling shutdown: {err}");
                            if polling.fatal.error().is_none() && is_polling_conflict(&err) {
                                polling.fatal.mark_conflict();
                            }
                        }
                        return None;
                    }
                    if !delay.is_zero() {
                        tokio::select! {
                            _ = polling.stop_flag.clone() => continue,
                            _ = tokio::time::sleep(delay) => {}
                        }
                        delay = Duration::ZERO;
                    }
                    let mut request = polling.bot.get_updates().offset(offset).timeout(10);
                    if let Some(allowed) = polling.allowed_updates.take() {
                        request = request.allowed_updates(allowed);
                    }
                    let result = tokio::select! {
                        _ = polling.stop_flag.clone() => continue,
                        result = request => result,
                    };
                    match result {
                        Ok(updates) => {
                            polling.failures.record_success();
                            if let Some(update) = updates.last() {
                                offset = update.id.as_offset();
                            }
                            buffered = updates.into_iter();
                        }
                        Err(err) => {
                            if is_polling_conflict(&err) {
                                polling.fatal.mark_conflict();
                                polling.stop_token.stop();
                            } else {
                                match polling.failures.record_failure() {
                                    Ok(backoff) => {
                                        delay = err
                                            .retry_after()
                                            .map_or(backoff, teloxide::types::Seconds::duration);
                                    }
                                    Err(_) => {
                                        polling.fatal.fatal.store(2, Ordering::SeqCst);
                                        polling.stop_token.stop();
                                    }
                                }
                            }
                            return Some((Err(err), (polling, buffered, offset, delay)));
                        }
                    }
                }
            },
        ))
    }
}

impl UpdateListener for GuardedPolling {
    type Err = RequestError;

    fn stop_token(&mut self) -> StopToken {
        self.stop_token.clone()
    }

    fn hint_allowed_updates(&mut self, hint: &mut dyn Iterator<Item = AllowedUpdate>) {
        self.allowed_updates = Some(hint.collect());
    }
}

fn is_polling_conflict(error: &RequestError) -> bool {
    matches!(
        error,
        RequestError::Api(ApiError::TerminatedByOtherGetUpdates)
    ) || is_http_409_conflict(error)
}

#[cfg(test)]
#[path = "polling_tests.rs"]
mod tests;
