//! Bounded, request-local repair before any Anthropic output reaches the consumer.

use codex_api::ApiError;
use codex_api::ResponseEvent;
use codex_api::ResponseStream;
use futures::StreamExt;

/// Hold only the small pre-output prefix. Once any output, unknown event or bound
/// is reached, return everything unchanged and leave subsequent errors to the caller.
pub(super) async fn prefetch(
    stream: &mut ResponseStream,
    on_cancel: impl FnOnce(),
) -> Result<Vec<Result<ResponseEvent, ApiError>>, ApiError> {
    let mut guard = CancellationGuard(Some(on_cancel));
    let result = read_prefix(stream).await;
    guard.0.take();
    result
}

struct CancellationGuard<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for CancellationGuard<F> {
    fn drop(&mut self) {
        if let Some(on_cancel) = self.0.take() {
            on_cancel();
        }
    }
}

/// A rejection not intercepted by the one request-local repair is terminal.
/// Otherwise the outer sampling loop would replay output and reset the budget.
pub(super) fn terminalize(
    event: Result<ResponseEvent, ApiError>,
) -> Result<ResponseEvent, ApiError> {
    event.map_err(|error| {
        if super::is_anthropic_signed_thinking_history_rejection(&error)
            && let ApiError::Stream(message) = error
        {
            return ApiError::InvalidRequest { message };
        }
        error
    })
}

async fn read_prefix(
    stream: &mut ResponseStream,
) -> Result<Vec<Result<ResponseEvent, ApiError>>, ApiError> {
    let mut prefix = Vec::new();
    while prefix.len() < 32 {
        let Some(event) = stream.next().await else {
            break;
        };
        match event {
            Err(error) if super::is_anthropic_signed_thinking_history_rejection(&error) => {
                return Err(error);
            }
            event => {
                let metadata = matches!(
                    &event,
                    Ok(ResponseEvent::Created { .. } | ResponseEvent::ServerReasoningIncluded(_))
                ) || matches!(&event, Ok(ResponseEvent::ServerModel(model)) if model.len() <= 1024);
                prefix.push(event);
                if !metadata {
                    break;
                }
            }
        }
    }
    Ok(prefix)
}

#[cfg(test)]
#[path = "client_anthropic_recovery_tests.rs"]
mod tests;
