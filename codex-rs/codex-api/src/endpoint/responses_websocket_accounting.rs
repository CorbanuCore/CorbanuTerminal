//! Optional admission at the socket pump and response-local numeric ordering.
use crate::endpoint::responses::accounting::InvalidResponsesUsage;
use crate::endpoint::responses::accounting::ResponsesUsageObserver;
use crate::endpoint::responses::accounting::decode;
use crate::error::ApiError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Resolves to the usage observer for an admitted request.
pub type AdmissionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Arc<dyn ResponsesUsageObserver>, ApiError>> + Send + 'a>>;

/// A validated sampling route. Admission must durably bind the final request's
/// model and tier before returning its immutable, physical-response observer.
/// Cancellation may have committed an unknown intent; implementations must fail
/// closed rather than guess a predecessor on a later dispatch.
pub trait ResponsesWebsocketAdmission: Send + Sync {
    /// Rechecks denial-only host guards before processing each in-flight text event.
    fn check(&self) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn admit(&self, model: String, tier: Option<String>) -> AdmissionFuture<'_>;
}

pub(super) struct Dispatch {
    pub model: String,
    pub tier: Option<String>,
    pub admission: Arc<dyn ResponsesWebsocketAdmission>,
}

pub(super) struct Evidence {
    observer: Arc<dyn ResponsesUsageObserver>,
    position: i64,
    /// Evidence was rejected; nothing more is observed for this response.
    closed: bool,
}
impl Evidence {
    pub(super) fn new(observer: Arc<dyn ResponsesUsageObserver>) -> Self {
        Self {
            observer,
            position: 0,
            closed: false,
        }
    }

    /// Observe one text event. Accounting observes the stream; it never ends
    /// it: rejected evidence stops observation for the rest of this response.
    pub(super) async fn text(&mut self, text: &str) -> Result<(), ApiError> {
        if self.closed {
            return Ok(());
        }
        let Some(position) = self.position.checked_add(1) else {
            let _ = self
                .observer
                .observe(self.position, Err(InvalidResponsesUsage))
                .await;
            self.closed = true;
            return Ok(());
        };
        self.position = position;
        let result = match decode(text) {
            Ok(None) => return Ok(()),
            Ok(Some(patch)) => self.observer.observe(position, Ok(patch)).await.is_ok(),
            Err(error) => {
                let _ = self.observer.observe(position, Err(error)).await;
                false
            }
        };
        self.closed = !result;
        Ok(())
    }
}
