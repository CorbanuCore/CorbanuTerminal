use codex_http_client::Request;
use codex_http_client::TransportError;
use rand::Rng;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u64,
    pub base_delay: Duration,
    pub retry_on: RetryOn,
}

#[derive(Debug, Clone)]
pub struct RetryOn {
    pub retry_429: bool,
    pub retry_5xx: bool,
    pub retry_transport: bool,
}

impl RetryOn {
    pub fn should_retry(&self, err: &TransportError, attempt: u64, max_attempts: u64) -> bool {
        if attempt >= max_attempts {
            return false;
        }
        match err {
            TransportError::Http { status, .. } => {
                (self.retry_429 && status.as_u16() == 429)
                    || (self.retry_5xx && status.is_server_error())
            }
            TransportError::Timeout | TransportError::Network(_) => self.retry_transport,
            _ => false,
        }
    }
}

/// The longest `Retry-After` a transport retry of an HTTP 429 waits out. A
/// 429 asking for longer is returned at once, so the caller's provider-request
/// throttle records the cooldown and holds every session on the key back
/// instead of this request sleeping through it unseen (#510).
const MAX_TRANSPORT_RETRY_AFTER: Duration = Duration::from_secs(5);

enum RateLimitRetry {
    NotRateLimited,
    /// Wait at least this long before the retry.
    After(Duration),
    Stop,
}

/// How a retryable error's requested wait shapes the next transport retry.
///
/// The headers are read in the order and the way core reads them for the
/// throttle's cooldown (`codex-api` `extract_retry_after_ms`): a positive
/// integer `retry-after-ms`, then a positive integer `Retry-After` in seconds
/// (a `Retry-After` that is not an integer, such as an HTTP date, stops the
/// retries; core reads the date), then `x-ratelimit-reset-ms` and
/// `x-ratelimit-reset` (Unix time in milliseconds or seconds). A wait up to
/// `MAX_TRANSPORT_RETRY_AFTER` is honoured; a longer one stops the retries. A 429 with none of them (a
/// transient overload, as Z.AI sends) keeps the normal backoff.
fn rate_limit_retry_delay(err: &TransportError) -> RateLimitRetry {
    let TransportError::Http {
        status, headers, ..
    } = err
    else {
        return RateLimitRetry::NotRateLimited;
    };
    if status.as_u16() != 429 {
        return RateLimitRetry::NotRateLimited;
    }
    let header = |name: &str| {
        headers
            .as_ref()
            .and_then(|headers| headers.get(name))
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
    };
    let positive = |value: &str| value.parse::<i64>().ok().filter(|value| *value > 0);
    let now_ms = || {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |now| i64::try_from(now.as_millis()).unwrap_or(i64::MAX))
    };
    let reset_in_ms = |value: &str| {
        let reset = positive(value)?;
        let reset_ms = if reset > 10_000_000_000 {
            reset
        } else {
            reset.saturating_mul(1000)
        };
        Some(reset_ms.saturating_sub(now_ms())).filter(|ms| *ms > 0)
    };
    let wait_ms = match header("retry-after-ms").and_then(positive) {
        Some(ms) => ms,
        None => {
            let retry_after = header("retry-after");
            if retry_after.is_some_and(|value| value.parse::<i64>().is_err()) {
                return RateLimitRetry::Stop;
            }
            retry_after
                .and_then(positive)
                .map(|seconds| seconds.saturating_mul(1000))
                .or_else(|| header("x-ratelimit-reset-ms").and_then(reset_in_ms))
                .or_else(|| header("x-ratelimit-reset").and_then(reset_in_ms))
                .unwrap_or(0)
        }
    };
    let wait = Duration::from_millis(u64::try_from(wait_ms).unwrap_or(0));
    if wait <= MAX_TRANSPORT_RETRY_AFTER {
        RateLimitRetry::After(wait)
    } else {
        RateLimitRetry::Stop
    }
}

pub fn backoff(base: Duration, attempt: u64) -> Duration {
    if attempt == 0 {
        return base;
    }
    let exp = 2u64.saturating_pow(attempt as u32 - 1);
    let millis = base.as_millis() as u64;
    let raw = millis.saturating_mul(exp);
    let jitter: f64 = rand::rng().random_range(0.9..1.1);
    Duration::from_millis((raw as f64 * jitter) as u64)
}

pub async fn run_with_retry<T, F, Fut>(
    policy: RetryPolicy,
    mut make_req: impl FnMut() -> Request,
    op: F,
) -> Result<T, TransportError>
where
    F: Fn(Request, u64) -> Fut,
    Fut: Future<Output = Result<T, TransportError>>,
{
    for attempt in 0..=policy.max_attempts {
        let req = make_req();
        match op(req, attempt).await {
            Ok(resp) => return Ok(resp),
            Err(err)
                if policy
                    .retry_on
                    .should_retry(&err, attempt, policy.max_attempts) =>
            {
                let delay = backoff(policy.base_delay, attempt + 1);
                let delay = match rate_limit_retry_delay(&err) {
                    RateLimitRetry::NotRateLimited => delay,
                    RateLimitRetry::After(wait) => delay.max(wait),
                    // The provider-request throttle owns a longer wait (#510).
                    RateLimitRetry::Stop => return Err(err),
                };
                sleep(delay).await;
            }
            Err(err) => return Err(err),
        }
    }
    Err(TransportError::RetryLimit)
}
