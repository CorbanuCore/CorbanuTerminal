//! Optional admission at the socket pump and response-local numeric ordering.
use crate::endpoint::responses::accounting::InvalidResponsesUsage;
use crate::endpoint::responses::accounting::ResponsesUsageObserver;
use crate::endpoint::responses::accounting::decode;
use crate::error::ApiError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A validated sampling route. Admission must durably bind the final request's
/// model and tier before returning its immutable, physical-response observer.
/// Cancellation may have committed an unknown intent; implementations must fail
/// closed rather than guess a predecessor on a later dispatch.
pub trait ResponsesWebsocketAdmission: Send + Sync {
    /// Rechecks denial-only host guards before processing each in-flight text event.
    fn check(&self) -> Pin<Box<dyn Future<Output = Result<(), ApiError>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn admit(
        &self,
        model: String,
        tier: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Arc<dyn ResponsesUsageObserver>, ApiError>> + Send + '_>>;
}

pub(super) struct Dispatch {
    pub model: String,
    pub tier: Option<String>,
    pub admission: Arc<dyn ResponsesWebsocketAdmission>,
}

pub(super) struct Evidence {
    observer: Arc<dyn ResponsesUsageObserver>,
    position: i64,
}
impl Evidence {
    pub(super) fn new(observer: Arc<dyn ResponsesUsageObserver>) -> Self {
        Self {
            observer,
            position: 0,
        }
    }

    pub(super) async fn text(&mut self, text: &str) -> Result<(), ApiError> {
        let Some(position) = self.position.checked_add(1) else {
            self.observer
                .observe(self.position, Err(InvalidResponsesUsage))
                .await?;
            return Err(ApiError::Stream(
                "Responses accounting position overflow".into(),
            ));
        };
        self.position = position;
        match decode(text) {
            Ok(None) => Ok(()),
            Ok(Some(patch)) => self.observer.observe(position, Ok(patch)).await,
            Err(error) => {
                self.observer.observe(position, Err(error)).await?;
                Err(ApiError::Stream(
                    "Invalid Responses accounting evidence".into(),
                ))
            }
        }
    }
}
