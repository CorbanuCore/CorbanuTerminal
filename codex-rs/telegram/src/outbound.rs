//! Outbound Telegram API call policy: explicit timeouts and bounded retries
//! for safe/idempotent requests.
//!
//! WHY (2026-07-22): the connector issued Bot API calls with teloxide's
//! defaults — no per-call timeout on most paths, and no retry policy at all.
//! A stalled `getFile`/download could wedge a chat's turn pipeline forever,
//! and a transient 429/5xx surfaced as a hard failure (or, worse, a silently
//! dropped user message) instead of being absorbed. This module is the single
//! place that policy lives.
//!
//! Retry safety: only *idempotent* Bot API calls are retried. Telegram's
//! `sendMessage` family is NOT idempotent (a retried send can post the
//! message twice), so mutating sends are given a timeout but never retried
//! automatically — the duplicate-protection boundary for those is
//! `crate::dedup`, not retries.

use std::time::Duration;

use teloxide::RequestError;
use tokio::time::sleep;
use tokio::time::timeout;
use tracing::warn;

/// Default ceiling for any single outbound Bot API call.
pub const DEFAULT_API_TIMEOUT: Duration = Duration::from_secs(15);

/// Media downloads get a longer ceiling: they move up to
/// the configured attachment ceiling over whatever link the host has.
pub const MEDIA_API_TIMEOUT: Duration = Duration::from_secs(60);

/// Hard cap on automatic retries for one idempotent call, not counting the
/// initial attempt.
pub const MAX_RETRIES: u32 = 3;

/// Backoff schedule base; attempt `n` waits `base * 2^n` plus any server hint.
const RETRY_BASE_DELAY: Duration = Duration::from_millis(500);

/// Never sleep longer than this even if `retry_after` says so; Telegram's
/// flood waits are normally seconds, and an unbounded sleep wedges the chat.
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

/// Per-attempt timeout applied *inside* `call_with_policy`, a few seconds
/// below the action-style reqwest client's overall ceiling
/// (`OUTBOUND_CLIENT_TIMEOUT` in `lib.rs`). Without a distinct inner budget a
/// slow-but-not-hung attempt would trip the client ceiling mid-retry and
/// abort the whole policy loop instead of backing off and trying again.
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(25);

/// How an outbound call should be treated for retry purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallSafety {
    /// Safe to retry: reading state or fetching a file produces no duplicate
    /// side effect on Telegram's side.
    Idempotent,
    /// Never auto-retry: the call may create a user-visible artifact (a sent
    /// message, an edited message, a callback answer) and a retry could
    /// double it.
    Mutating,
}

/// Classification of a request failure for retry decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureClass {
    /// HTTP 429 with Telegram's `retry_after` hint (seconds).
    RateLimited { retry_after_secs: u64 },
    /// HTTP 5xx from Telegram or its edge.
    ServerError,
    /// Transport-level failure (connect reset, DNS, TLS, timeout reading).
    Transport,
    /// Anything else — 4xx other than 429, decode errors, etc. Not retryable.
    Other,
}

/// Classify a teloxide `RequestError` for the retry policy. Exposed for tests.
pub fn classify_request_error(error: &RequestError) -> FailureClass {
    match error {
        RequestError::RetryAfter(seconds) => FailureClass::RateLimited {
            retry_after_secs: u64::from(seconds.seconds()),
        },
        // teloxide's `ApiError` is message-parsed, not status-typed: known
        // variants map to specific (non-retryable) Telegram error strings,
        // and anything unrecognized — including transient 500/502/503/504
        // responses that reach the Bot API client as generic error bodies —
        // lands in `Unknown`. Retrying `Unknown` on idempotent calls is the
        // conservative choice: the worst case is a few wasted idempotent
        // reads, while known 4xx variants stay non-retryable.
        RequestError::Api(teloxide::ApiError::Unknown(_)) => FailureClass::ServerError,
        RequestError::Api(_) => FailureClass::Other,
        RequestError::Network(_) | RequestError::Io(_) => FailureClass::Transport,
        _ => FailureClass::Other,
    }
}

/// Decide whether an idempotent call that failed with `error` should be
/// retried on attempt `attempt` (0-based), and if so how long to wait first.
/// `None` means give up.
pub fn retry_decision(safety: CallSafety, error: &RequestError, attempt: u32) -> Option<Duration> {
    if safety == CallSafety::Mutating || attempt >= MAX_RETRIES {
        return None;
    }
    let backoff = RETRY_BASE_DELAY * 2u32.saturating_pow(attempt);
    let delay = match classify_request_error(error) {
        FailureClass::RateLimited { retry_after_secs } => {
            let hint = Duration::from_secs(retry_after_secs);
            backoff.max(hint)
        }
        FailureClass::ServerError | FailureClass::Transport => backoff,
        FailureClass::Other => return None,
    };
    Some(delay.min(MAX_RETRY_DELAY))
}

/// Run one outbound Bot API call under an explicit timeout, with bounded
/// retries for idempotent calls that fail transiently (429/5xx/transport).
///
/// `requested_timeout` declares the caller's latency ceiling (15s default,
/// 60s media); the actual attempt timeout is `min(requested_timeout,
/// ATTEMPT_TIMEOUT)` so retrying calls never exceed the HTTP client ceiling.
///
/// `make_request` is invoked once per attempt; it must build a fresh future
/// each time because teloxide request futures are not `Unpin`/reusable.
pub async fn call_with_policy<F, Fut, T>(
    safety: CallSafety,
    requested_timeout: Duration,
    what: &'static str,
    mut make_request: F,
) -> Result<T, RequestError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, RequestError>>,
{
    let call_timeout = requested_timeout.min(ATTEMPT_TIMEOUT);
    let mut attempt = 0u32;
    loop {
        let result = timeout(call_timeout, make_request()).await;
        let error = match result {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(error)) => error,
            Err(_elapsed) => {
                warn!(
                    what,
                    timeout_secs = call_timeout.as_secs(),
                    "Telegram API call timed out"
                );
                // A timeout has no server response to classify; treat as
                // transient for idempotent calls, fatal for mutating ones.
                if safety == CallSafety::Mutating || attempt >= MAX_RETRIES {
                    return Err(RequestError::Io(std::sync::Arc::new(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("{what} timed out after {}s", call_timeout.as_secs()),
                    ))));
                }
                let delay = (RETRY_BASE_DELAY * 2u32.saturating_pow(attempt)).min(MAX_RETRY_DELAY);
                attempt += 1;
                warn!(
                    what,
                    attempt,
                    delay_ms = delay.as_millis() as u64,
                    "retrying Telegram API call after timeout"
                );
                sleep(delay).await;
                continue;
            }
        };
        match retry_decision(safety, &error, attempt) {
            Some(delay) => {
                let class = classify_request_error(&error);
                warn!(
                    what,
                    attempt,
                    delay_ms = delay.as_millis() as u64,
                    class = ?class,
                    "retrying Telegram API call after transient failure"
                );
                attempt += 1;
                sleep(delay).await;
            }
            None => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;

    fn rate_limit_error(secs: u64) -> RequestError {
        RequestError::RetryAfter(teloxide::types::Seconds::from_seconds(secs as u32))
    }

    fn transport_error() -> RequestError {
        RequestError::Io(std::sync::Arc::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "reset",
        )))
    }

    #[test]
    fn classifies_retry_after_with_hint() {
        assert_eq!(
            classify_request_error(&rate_limit_error(/*secs*/ 7)),
            FailureClass::RateLimited {
                retry_after_secs: 7
            }
        );
    }

    #[test]
    fn classifies_transport_failures() {
        assert_eq!(
            classify_request_error(&transport_error()),
            FailureClass::Transport
        );
    }

    #[test]
    fn mutating_calls_are_never_retried() {
        assert_eq!(
            retry_decision(
                CallSafety::Mutating,
                &rate_limit_error(/*secs*/ 1),
                /*attempt*/ 0
            ),
            None
        );
        assert_eq!(
            retry_decision(CallSafety::Mutating, &transport_error(), /*attempt*/ 0),
            None
        );
    }

    #[test]
    fn idempotent_429_honors_retry_after_floor() {
        let delay = retry_decision(
            CallSafety::Idempotent,
            &rate_limit_error(/*secs*/ 5),
            /*attempt*/ 0,
        )
        .expect("429 on idempotent call must be retried");
        assert!(delay >= Duration::from_secs(5));
        assert!(delay <= MAX_RETRY_DELAY);
    }

    #[test]
    fn idempotent_transport_uses_bounded_backoff() {
        let first = retry_decision(
            CallSafety::Idempotent,
            &transport_error(),
            /*attempt*/ 0,
        )
        .unwrap();
        let second = retry_decision(
            CallSafety::Idempotent,
            &transport_error(),
            /*attempt*/ 1,
        )
        .unwrap();
        assert!(second > first);
        assert!(second <= MAX_RETRY_DELAY);
    }

    #[test]
    fn retries_are_capped() {
        assert_eq!(
            retry_decision(CallSafety::Idempotent, &transport_error(), MAX_RETRIES),
            None
        );
    }

    #[tokio::test]
    async fn call_with_policy_retries_transient_then_succeeds() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);
        let value = call_with_policy(
            CallSafety::Idempotent,
            Duration::from_secs(5),
            "test call",
            move || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                        Err(transport_error())
                    } else {
                        Ok(42)
                    }
                }
            },
        )
        .await
        .expect("second attempt should succeed");
        assert_eq!(value, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn call_with_policy_does_not_retry_mutating_calls() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);
        let result: Result<(), RequestError> = call_with_policy(
            CallSafety::Mutating,
            Duration::from_secs(5),
            "test mutating call",
            move || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err(rate_limit_error(/*secs*/ 1))
                }
            },
        )
        .await;
        assert!(result.is_err());
        assert_eq!(
            attempts.load(Ordering::SeqCst),
            1,
            "mutating call must not be retried"
        );
    }

    #[tokio::test]
    async fn call_with_policy_times_out_hanging_calls() {
        let result: Result<(), RequestError> = call_with_policy(
            CallSafety::Mutating,
            Duration::from_millis(50),
            "hanging call",
            || async {
                sleep(Duration::from_secs(60)).await;
                Ok(())
            },
        )
        .await;
        let error = result.expect_err("hanging call must fail");
        assert!(error.to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn call_with_policy_gives_up_after_cap() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);
        let result: Result<(), RequestError> = call_with_policy(
            CallSafety::Idempotent,
            Duration::from_secs(5),
            "always failing call",
            move || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err(transport_error())
                }
            },
        )
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), MAX_RETRIES + 1);
    }
}
