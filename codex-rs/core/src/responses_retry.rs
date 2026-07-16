//! Shared retry and transport fallback decisions for Responses requests.

use std::time::Duration;

use crate::client::ModelClientSession;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::util::backoff;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use tracing::warn;

const MAX_SAME_REQUEST_IDLE_FAILURES: u64 = 2;
const SSE_IDLE_TIMEOUT_MESSAGE: &str = "idle timeout waiting for SSE";

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResponsesStreamRequest {
    Sampling,
    RemoteCompactionV2,
}

pub(crate) fn guard_same_request_idle_retry(
    err: &CodexErr,
    same_request_idle_failures: &mut u64,
) -> Result<(), CodexErr> {
    if is_sse_idle_timeout(err) {
        *same_request_idle_failures = (*same_request_idle_failures).saturating_add(1);
        if *same_request_idle_failures >= MAX_SAME_REQUEST_IDLE_FAILURES {
            return Err(CodexErr::Stream(
                format!(
                    "stream idle timeout repeated {} times for the same \
                     request; aborting instead of restarting it again",
                    *same_request_idle_failures,
                ),
                None,
            ));
        }
    } else {
        *same_request_idle_failures = 0;
    }

    Ok(())
}

fn is_sse_idle_timeout(err: &CodexErr) -> bool {
    matches!(
        err,
        CodexErr::Stream(message, _) if message.contains(SSE_IDLE_TIMEOUT_MESSAGE)
    )
}

/// Handles a retryable stream error and returns `Ok(())` when the caller should
/// retry the request loop.
pub(crate) async fn handle_retryable_response_stream_error(
    retries: &mut u64,
    max_retries: u64,
    err: CodexErr,
    client_session: &mut ModelClientSession,
    sess: &Session,
    turn_context: &TurnContext,
    request: ResponsesStreamRequest,
    attempt_elapsed: Duration,
) -> Result<(), CodexErr> {
    ensure_gpu_runtime_provider_active(sess, turn_context).await?;
    let long_failure = attempt_elapsed
        >= turn_context
            .provider
            .info()
            .stream_long_failure_retry_threshold();
    let effective_max_retries = effective_max_stream_retries(
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
            *retries + 1,
            effective_max_retries,
            attempt_elapsed,
            &err,
        )
        .await;
    }

    if *retries >= effective_max_retries && long_failure {
        return Err(err);
    }

    if *retries >= effective_max_retries
        && client_session.try_switch_fallback_transport(
            &turn_context.session_telemetry,
            &turn_context.model_info,
        )
    {
        sess.send_event(
            turn_context,
            EventMsg::Warning(WarningEvent {
                message: format!("Falling back from WebSockets to HTTPS transport. {err:#}"),
            }),
        )
        .await;
        *retries = 0;
        return Ok(());
    }

    if *retries < effective_max_retries {
        *retries += 1;
        let retry_count = *retries;
        let delay = match &err {
            CodexErr::Stream(_, requested_delay) => {
                requested_delay.unwrap_or_else(|| backoff(retry_count))
            }
            _ => backoff(retry_count),
        };
        log_retry(
            request,
            turn_context,
            &err,
            retry_count,
            effective_max_retries,
            delay,
        );

        // In release builds, hide the first websocket retry notification to reduce noisy
        // transient reconnect messages. In debug builds, keep full visibility for diagnosis.
        let report_error = retry_count > 1
            || cfg!(debug_assertions)
            || !sess.services.responses_websocket_enabled();
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
        tokio::time::sleep(delay).await;
        return Ok(());
    }

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
    if gpu_runtime_provider_is_ready(provider_id, &records) {
        return Ok(());
    }
    Err(CodexErr::InvalidRequest(
        "The selected rented GPU is no longer active. This turn was not reconnected to its dead endpoint; select another model with /model or start a new rental from /gpu."
            .to_string(),
    ))
}

fn gpu_runtime_provider_is_ready(
    provider_id: &str,
    records: &[codex_state::GpuRuntimeProvider],
) -> bool {
    records
        .iter()
        .any(|record| record.provider_id == provider_id && record.health == "ready")
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
mod tests {
    use super::*;

    #[test]
    fn same_request_idle_guard_aborts_after_second_idle_failure() {
        let mut failures = 0;
        let idle = CodexErr::Stream(SSE_IDLE_TIMEOUT_MESSAGE.to_string(), None);

        guard_same_request_idle_retry(&idle, &mut failures).expect("first idle failure can retry");
        assert_eq!(failures, 1);

        let err = guard_same_request_idle_retry(&idle, &mut failures)
            .expect_err("second same-request idle failure should abort");
        assert_eq!(failures, 2);
        assert!(
            err.to_string()
                .contains("aborting instead of restarting it again")
        );
    }

    #[test]
    fn same_request_idle_guard_resets_on_non_idle_error() {
        let mut failures = 1;
        let other = CodexErr::Stream("stream closed before completion".to_string(), None);

        guard_same_request_idle_retry(&other, &mut failures)
            .expect("non-idle stream error should not abort");

        assert_eq!(failures, 0);
    }

    #[test]
    fn long_stream_failure_retries_are_capped_to_one() {
        assert_eq!(
            effective_max_stream_retries(
                /*max_retries*/ 5, /*long_failure*/ true,
                /*long_failure_max_retries*/ 1,
            ),
            1
        );
    }

    #[test]
    fn normal_stream_failures_keep_provider_retry_count() {
        assert_eq!(
            effective_max_stream_retries(
                /*max_retries*/ 5, /*long_failure*/ false,
                /*long_failure_max_retries*/ 1,
            ),
            5
        );
    }

    #[test]
    fn gpu_retry_liveness_requires_the_selected_provider_to_remain_ready() {
        let record = codex_state::GpuRuntimeProvider {
            rental_id: "gpu-rental".to_string(),
            infrastructure_provider: "vast".to_string(),
            provider_id: "gpu-gpu-rental".to_string(),
            base_url: "http://127.0.0.1:1234/v1".to_string(),
            model_id: "test-model".to_string(),
            wire_api: "chat".to_string(),
            health: "ready".to_string(),
            display_hourly_microusd: 1_000_000,
            maximum_context_tokens: Some(32_768),
            catalog_sequence: 1,
            updated_at_ms: 1,
        };

        assert!(gpu_runtime_provider_is_ready(
            "gpu-gpu-rental",
            std::slice::from_ref(&record)
        ));
        assert!(!gpu_runtime_provider_is_ready(
            "gpu-another-rental",
            std::slice::from_ref(&record)
        ));
        assert!(!gpu_runtime_provider_is_ready(
            "gpu-gpu-rental",
            &[codex_state::GpuRuntimeProvider {
                health: "degraded".to_string(),
                ..record
            }]
        ));
    }
}
