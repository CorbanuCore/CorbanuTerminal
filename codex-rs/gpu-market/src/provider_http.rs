use crate::GpuCredentialError;
use crate::ProviderError;
use crate::ProviderErrorKind;
use reqwest::Response;

pub(crate) fn credential_error(error: GpuCredentialError) -> ProviderError {
    let kind = if error == GpuCredentialError::Missing {
        ProviderErrorKind::NotConfigured
    } else {
        ProviderErrorKind::Unauthorized
    };
    ProviderError::new(kind, error.to_string())
}

pub(crate) async fn decode_json(response: Response) -> Result<serde_json::Value, ProviderError> {
    let status = response.status();
    if status.is_success() {
        return response.json().await.map_err(|_| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "GPU provider returned a malformed success response.",
            )
            .with_diagnostic_ref(new_diagnostic_ref())
        });
    }

    let retry_after_ms = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i64>().ok())
        .map(|seconds| seconds.saturating_mul(1_000));
    let kind = match status.as_u16() {
        401 | 403 => ProviderErrorKind::Unauthorized,
        404 | 410 => ProviderErrorKind::OfferUnavailable,
        409 => ProviderErrorKind::Ambiguous,
        429 => ProviderErrorKind::RateLimited,
        500..=599 => ProviderErrorKind::Retryable,
        _ => ProviderErrorKind::InvalidRequest,
    };
    let mut error = ProviderError::new(kind, safe_status_message(status.as_u16()))
        .with_diagnostic_ref(new_diagnostic_ref());
    if let Some(retry_after_ms) = retry_after_ms {
        error = error.with_retry_after_ms(retry_after_ms);
    }
    Err(error)
}

pub(crate) fn transport_error() -> ProviderError {
    ProviderError::new(
        ProviderErrorKind::Ambiguous,
        "GPU provider request outcome is unknown; inventory reconciliation is required.",
    )
    .with_diagnostic_ref(new_diagnostic_ref())
}

pub(crate) fn parse_usd_micros(value: &serde_json::Value) -> Option<i64> {
    let dollars = value
        .as_f64()
        .or_else(|| value.as_str().and_then(|raw| raw.parse::<f64>().ok()))?;
    if !dollars.is_finite() || dollars < 0.0 {
        return None;
    }
    let micros = dollars * 1_000_000.0;
    if micros > i64::MAX as f64 {
        return None;
    }
    Some(micros.round() as i64)
}

fn safe_status_message(status: u16) -> &'static str {
    match status {
        401 | 403 => "GPU provider rejected the configured credential.",
        404 | 410 => "GPU provider resource or offer is unavailable.",
        409 => "GPU provider reported a conflicting or ambiguous operation.",
        429 => "GPU provider rate limit was reached.",
        500..=599 => "GPU provider is temporarily unavailable.",
        _ => "GPU provider rejected the request.",
    }
}

fn new_diagnostic_ref() -> String {
    format!("gpu-diag-{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
#[path = "provider_http_tests.rs"]
mod tests;
