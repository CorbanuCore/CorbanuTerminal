//! Shared retry and transport fallback decisions for Responses requests.
//! Content-filter guidance is recorded for sampling requests before retry decisions.
//! Server advice controls timing without extending configured retry limits.

use std::time::Duration;

use crate::client::ModelClientSession;
use crate::context::ContentFilterGuidance;
use crate::context::ContextualUserFragment;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use codex_client::RetryOperation;
use codex_features::Feature;
use codex_http_client::RetryAfter;
use codex_protocol::error::CodexErr;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use tokio::time::Instant;
use tracing::warn;

const INITIAL_CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(5);
const MAX_CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(60);
const MAX_SAME_REQUEST_IDLE_FAILURES: u64 = 2;
const SSE_IDLE_TIMEOUT_MESSAGE: &str = "idle timeout waiting for SSE";

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResponsesStreamRequest {
    Sampling,
    RemoteCompactionV2,
}

pub(crate) struct ResponsesStreamRetryState {
    retries: u64,
    connection_retries: u64,
    connection_retry_delay: Duration,
}

impl ResponsesStreamRetryState {
    /// Retries used so far; the next attempt's same-turn index is this plus one.
    pub(crate) fn retries(&self) -> u64 {
        self.retries
    }
}

impl Default for ResponsesStreamRetryState {
    fn default() -> Self {
        Self {
            retries: 0,
            connection_retries: 0,
            connection_retry_delay: INITIAL_CONNECTION_RETRY_DELAY,
        }
    }
}

pub(crate) fn guard_same_request_idle_retry(
    err: &CodexErr,
    same_request_idle_failures: &mut u64,
) -> Result<(), CodexErr> {
    if is_sse_idle_timeout(err) {
        *same_request_idle_failures = (*same_request_idle_failures).saturating_add(1);
        if *same_request_idle_failures >= MAX_SAME_REQUEST_IDLE_FAILURES {
            return Err(CodexErr::Stream(format!(
                "stream idle timeout repeated {} times for the same \
                     request; aborting instead of restarting it again",
                *same_request_idle_failures,
            )));
        }
    } else {
        *same_request_idle_failures = 0;
    }

    Ok(())
}

fn is_sse_idle_timeout(err: &CodexErr) -> bool {
    matches!(
        err.details(),
        CodexErrorDetails::Stream(message) if message.contains(SSE_IDLE_TIMEOUT_MESSAGE)
    )
}

/// Server retry advice retained after stream retries are exhausted. The turn ID
/// prevents a reused Guardian session from applying advice from an earlier review.
pub(crate) struct ExhaustedResponseRetry {
    pub(crate) turn_id: String,
    pub(crate) retry_at: Option<tokio::time::Instant>,
}

/// Returns `Ok(())` when the caller should retry the request loop, or the original error when
/// it is terminal or the retry budget is exhausted.
#[tracing::instrument(level = "trace", skip_all, fields(request = ?request))]
#[allow(
    clippy::too_many_arguments,
    reason = "retry state is explicit so both stream callers share one policy boundary"
)]
pub(crate) async fn handle_response_stream_error(
    retry_state: &mut ResponsesStreamRetryState,
    max_retries: u64,
    err: CodexErr,
    client_session: &mut ModelClientSession,
    sess: &Session,
    step_context: &StepContext,
    request: ResponsesStreamRequest,
    attempt_elapsed: Duration,
) -> Result<(), CodexErr> {
    let turn_context = &step_context.turn;
    ensure_gpu_runtime_provider_active(sess, turn_context).await?;
    if matches!(request, ResponsesStreamRequest::Sampling)
        && matches!(err.details(), CodexErrorDetails::ContentFilter)
    {
        let model_info = &step_context.settings.model_info;
        let guidance = ContentFilterGuidance {
            text: codex_prompts::ResolvedModelMessages::from_model(model_info)
                .content_filter_guidance()
                .to_string(),
        };
        sess.record_conversation_items(
            turn_context,
            model_info,
            &[ContextualUserFragment::into(guidance)],
        )
        .await;
    }
    let operation = match request {
        ResponsesStreamRequest::Sampling => RetryOperation::Sampling,
        ResponsesStreamRequest::RemoteCompactionV2 => RetryOperation::RemoteCompactionV2,
    };
    let retry_count = retry_state.retries.saturating_add(1);
    let Some(delay) = err.retry_delay(retry_count) else {
        return Err(err);
    };
    let retry_after = err.retry_after();
    // Corbanu: an attempt that streamed for a long time before failing gets a
    // smaller retry cap, and the user is told how long it ran.
    let long_failure = attempt_elapsed
        >= turn_context
            .provider
            .info()
            .stream_long_failure_retry_threshold();
    let max_retries = effective_max_stream_retries(
        max_retries,
        long_failure,
        turn_context
            .provider
            .info()
            .stream_long_failure_max_retries(),
    );
    if long_failure {
        warn_long_stream_failure(
            sess,
            turn_context,
            retry_count,
            max_retries,
            attempt_elapsed,
            &err,
        )
        .await;
        if retry_state.retries >= max_retries {
            sess.services
                .thread_extension_data
                .insert(ExhaustedResponseRetry {
                    turn_id: turn_context.sub_id.clone(),
                    retry_at: retry_after.map(RetryAfter::deadline),
                });
            return Err(err);
        }
    }

    if turn_context
        .config
        .features
        .enabled(Feature::UnboundedConnectionRetries)
        && matches!(request, ResponsesStreamRequest::Sampling)
        && matches!(err.details(), CodexErrorDetails::ConnectionFailed(_))
        && !turn_context.session_source.is_internal()
        && !turn_context.provider.info().is_amazon_bedrock()
    {
        let retry_delay = retry_state.connection_retry_delay;
        warn!(
            turn_id = %turn_context.sub_id,
            error = %err,
            ?retry_delay,
            "stream connection failed; waiting to retry"
        );
        sess.notify_stream_error(turn_context, "Reconnecting... waiting for network", err)
            .await;
        retry_state.connection_retries = retry_state.connection_retries.saturating_add(1);
        codex_client::record_retry!(retry_state.connection_retries, retry_delay, operation);
        tokio::time::sleep(retry_delay).await;
        retry_state.connection_retry_delay = retry_delay
            .saturating_mul(2)
            .min(MAX_CONNECTION_RETRY_DELAY);
        return Ok(());
    }

    if retry_state.retries >= max_retries
        && client_session.try_switch_fallback_transport(
            &turn_context.session_telemetry,
            turn_context.model_info(),
        )
    {
        // Changing transport must not bypass the server's retry deadline.
        if let Some(retry_after) = retry_after {
            tokio::time::sleep_until(retry_after.deadline()).await;
        }
        sess.send_event(
            turn_context,
            EventMsg::Warning(WarningEvent {
                message: format!("Falling back from WebSockets to HTTPS transport. {err:#}"),
            }),
        )
        .await;
        retry_state.retries = 0;
        return Ok(());
    }

    if retry_state.retries < max_retries {
        retry_state.retries = retry_count;
        log_retry(request, turn_context, &err, retry_count, max_retries, delay);

        // In release builds, hide the first websocket retry notification to reduce noisy
        // transient reconnect messages. In debug builds, keep full visibility for diagnosis.
        let report_error = retry_count > 1
            || cfg!(debug_assertions)
            || !sess.services.model_client().responses_websocket_enabled();
        if report_error {
            // Surface retry information to any UI/front-end so the user understands what is
            // happening instead of staring at a seemingly frozen screen.
            sess.notify_stream_error(
                turn_context,
                format!("Reconnecting... {retry_count}/{max_retries}"),
                err,
            )
            .await;
        }
        // Use one clock sample so local backoff telemetry retains the selected delay.
        let now = Instant::now();
        let retry_at = retry_after.map(RetryAfter::deadline).unwrap_or(now + delay);
        let delay = retry_at.saturating_duration_since(now);
        codex_client::record_retry!(retry_count, delay, operation);
        tokio::time::sleep_until(retry_at).await;
        return Ok(());
    }

    sess.services
        .thread_extension_data
        .insert(ExhaustedResponseRetry {
            turn_id: turn_context.sub_id.clone(),
            retry_at: retry_after.map(RetryAfter::deadline),
        });
    Err(err)
}

pub(crate) async fn ensure_gpu_runtime_provider_active(
    sess: &Session,
    turn_context: &TurnContext,
) -> Result<(), CodexErr> {
    let provider_id = turn_context.config.model_provider_id.as_str();
    if !provider_id.starts_with("gpu-") {
        return Ok(());
    }
    let Some(state_db) = sess.state_db() else {
        warn!(
            "could not verify rented GPU liveness because session state is unavailable; preserving normal retry policy"
        );
        return Ok(());
    };
    let records = match state_db.list_gpu_runtime_providers().await {
        Ok(records) => records,
        Err(error) => {
            warn!(%error, "could not read rented GPU liveness; preserving normal retry policy");
            return Ok(());
        }
    };
    match gpu_runtime_provider_availability(provider_id, &records) {
        GpuRuntimeProviderAvailability::Ready => Ok(()),
        GpuRuntimeProviderAvailability::Recovering => Err(CodexErr::InvalidRequest(
            "The selected rented GPU is temporarily unavailable while its controller reconnects. Wait for the GPU READY notification, then retry this turn; no /model reselection is required. The rental may still be billing, so use /gpu to inspect or terminate it."
                .to_string(),
        )),
        GpuRuntimeProviderAvailability::Gone => Err(CodexErr::InvalidRequest(
            "The selected rented GPU is no longer active. Select another model with /model or start a new rental from /gpu."
                .to_string(),
        )),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum GpuRuntimeProviderAvailability {
    Ready,
    Recovering,
    Gone,
}

fn gpu_runtime_provider_availability(
    provider_id: &str,
    records: &[codex_state::GpuRuntimeProvider],
) -> GpuRuntimeProviderAvailability {
    match records
        .iter()
        .find(|record| record.provider_id == provider_id)
    {
        Some(record) if record.health == "ready" => GpuRuntimeProviderAvailability::Ready,
        Some(_) => GpuRuntimeProviderAvailability::Recovering,
        None => GpuRuntimeProviderAvailability::Gone,
    }
}

fn effective_max_stream_retries(
    max_retries: u64,
    long_failure: bool,
    long_failure_max_retries: u64,
) -> u64 {
    if long_failure {
        max_retries.min(long_failure_max_retries)
    } else {
        max_retries
    }
}

async fn warn_long_stream_failure(
    sess: &Session,
    turn_context: &TurnContext,
    attempt_number: u64,
    effective_max_retries: u64,
    attempt_elapsed: Duration,
    err: &CodexErr,
) {
    let elapsed_seconds = attempt_elapsed.as_secs_f64();
    let retry_state = if attempt_number <= effective_max_retries {
        format!("retrying with long-failure cap {effective_max_retries}")
    } else {
        format!("not retrying; long-failure cap {effective_max_retries} reached")
    };
    sess.send_event(
        turn_context,
        EventMsg::Warning(WarningEvent {
            message: format!(
                "Provider stream failed after {elapsed_seconds:.1}s on attempt \
                 {attempt_number}: {err:#}. {retry_state}."
            ),
        }),
    )
    .await;
}

fn log_retry(
    request: ResponsesStreamRequest,
    turn_context: &TurnContext,
    err: &CodexErr,
    retries: u64,
    max_retries: u64,
    delay: Duration,
) {
    match request {
        ResponsesStreamRequest::Sampling => {
            warn!(
                turn_id = %turn_context.sub_id,
                retries,
                max_retries,
                sampling_error = %err,
                "stream disconnected - retrying sampling request ({retries}/{max_retries} in {delay:?})...",
            );
        }
        ResponsesStreamRequest::RemoteCompactionV2 => {
            warn!(
                turn_id = %turn_context.sub_id,
                retries,
                max_retries,
                compact_error = %err,
                "remote compaction v2 stream failed; retrying request after delay"
            );
        }
    }
}

#[cfg(test)]
#[path = "responses_retry_tests.rs"]
mod tests;
