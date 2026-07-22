use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelegramError {
    #[error("telegram connector is disabled in config.toml")]
    Disabled,

    #[error("telegram bot token was not found in `{env_var}` or vault label `telegram/bot_token`")]
    MissingToken { env_var: String },

    #[error(
        "telegram webhook mode is configured but only polling is implemented in this connector"
    )]
    WebhookUnsupported,

    #[error("another Telegram poller is already running for this bot token")]
    PollingConflict,

    #[error("Telegram polling exceeded the configured consecutive failure cap")]
    PollingFailureCapExceeded,
}

#[derive(Debug, Clone)]
pub struct PollingBackoff {
    initial: Duration,
    max: Duration,
    max_consecutive_failures: u32,
    consecutive_failures: u32,
}

impl PollingBackoff {
    pub fn new(initial: Duration, max: Duration, max_consecutive_failures: u32) -> Self {
        Self {
            initial,
            max,
            max_consecutive_failures: max_consecutive_failures.max(1),
            consecutive_failures: 0,
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
    }

    pub fn record_failure(&mut self) -> Result<Duration, TelegramError> {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.max_consecutive_failures {
            return Err(TelegramError::PollingFailureCapExceeded);
        }

        let multiplier = 1_u32
            .checked_shl(self.consecutive_failures.saturating_sub(1))
            .unwrap_or(u32::MAX);
        Ok(self.initial.saturating_mul(multiplier).min(self.max))
    }
}

pub fn is_http_409_conflict(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(err) = current {
        if let Some(teloxide::RequestError::Network(network)) =
            err.downcast_ref::<teloxide::RequestError>()
            && network
                .status()
                .is_some_and(|status| status.as_u16() == 409)
        {
            return true;
        }
        current = err.source();
    }
    false
}
