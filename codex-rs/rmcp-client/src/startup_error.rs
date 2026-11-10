use anyhow::Error;
use codex_exec_server::ExecServerError;
use rmcp::service::ClientInitializeError;
use rmcp::service::ServiceError;
use rmcp::transport::DynamicTransportError;
use rmcp::transport::auth::AuthError;
use rmcp::transport::streamable_http_client::StreamableHttpError;

use crate::ema_auth_policy::EmaAuthFailure;
use crate::ema_auth_policy::EmaInvalidGrantSource;
use crate::http_client_adapter::StreamableHttpClientAdapterError;
use crate::rmcp_client::ClientOperationError;

/// Converts an rmcp OAuth error into `anyhow::Error` without losing its reason.
///
/// rmcp >= 3.9 keeps the cause of a failed discovery request in
/// `AuthError::DiscoveryRequestFailed`'s `source()`, which `Display` (used for
/// startup and login errors) omits. The `AuthError` stays downcastable.
pub(crate) fn auth_error_with_reason(error: AuthError) -> Error {
    match &error {
        AuthError::DiscoveryRequestFailed { source, .. } => {
            let message = format!("{error}: {source}");
            Error::new(error).context(message)
        }
        _ => Error::new(error),
    }
}

/// Returns whether an RMCP client error indicates that authentication is required.
///
/// This does not distinguish first-time login from reauthentication.
/// Streamable HTTP initialization errors are stored inside RMCP's dynamic
/// transport error, which is not part of the standard error source chain.
pub fn is_authentication_required_error(error: &Error) -> bool {
    error.chain().any(|source| {
        if let Some(error) = source.downcast_ref::<crate::EmaTokenSourceError>() {
            return error.is_authentication_required();
        }
        source
            .downcast_ref::<AuthError>()
            .is_some_and(auth_error_requires_authentication)
            || source
                .downcast_ref::<EmaAuthFailure>()
                .is_some_and(|error| match error {
                    EmaAuthFailure::InvalidGrant {
                        grant_source: EmaInvalidGrantSource::EnterpriseIdentity,
                    }
                    | EmaAuthFailure::InsufficientUserAuthentication {
                        grant_source: EmaInvalidGrantSource::EnterpriseIdentity,
                    }
                    | EmaAuthFailure::IdentityClientRejected
                    | EmaAuthFailure::ReauthenticationRequired => true,
                    // Rejecting an ID-JAG does not invalidate the enterprise login itself.
                    EmaAuthFailure::InvalidGrant {
                        grant_source: EmaInvalidGrantSource::ResourceAuthorization,
                    }
                    | EmaAuthFailure::InsufficientUserAuthentication {
                        grant_source: EmaInvalidGrantSource::ResourceAuthorization,
                    } => false,
                })
            || source
                .downcast_ref::<ExecServerError>()
                .is_some_and(exec_server_error_requires_authentication)
            || source
                .downcast_ref::<ClientInitializeError>()
                .is_some_and(|mut error| {
                    while let ClientInitializeError::LegacyFallbackFailed { fallback, .. } = error {
                        error = fallback;
                    }
                    matches!(
                        error,
                        ClientInitializeError::TransportError { error, .. }
                            if transport_error_requires_authentication(error)
                    )
                })
            || source.downcast_ref::<ServiceError>().is_some_and(|error| {
                matches!(
                    error,
                    ServiceError::TransportSend(error)
                        if transport_error_requires_authentication(error)
                )
            })
            || source
                .downcast_ref::<ClientOperationError>()
                .is_some_and(|error| {
                    matches!(
                        error,
                        ClientOperationError::Service(ServiceError::TransportSend(error))
                            if transport_error_requires_authentication(error)
                    )
                })
    })
}

fn transport_error_requires_authentication(error: &DynamicTransportError) -> bool {
    error
        .error
        .downcast_ref::<StreamableHttpError<StreamableHttpClientAdapterError>>()
        .is_some_and(|error| match error {
            StreamableHttpError::AuthRequired(_) => true,
            StreamableHttpError::Auth(auth_error) => auth_error_requires_authentication(auth_error),
            StreamableHttpError::Client(StreamableHttpClientAdapterError::HttpRequest(error)) => {
                exec_server_error_requires_authentication(error)
            }
            _ => false,
        })
}

fn exec_server_error_requires_authentication(error: &ExecServerError) -> bool {
    matches!(error, ExecServerError::AuthenticationRequired(_))
}

fn auth_error_requires_authentication(error: &AuthError) -> bool {
    matches!(
        error,
        AuthError::AuthorizationRequired | AuthError::TokenExpired
    )
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use rmcp::transport::DynamicTransportError;
    use rmcp::transport::streamable_http_client::AuthRequiredError;

    use super::*;

    fn initialize_transport_error(
        error: StreamableHttpError<StreamableHttpClientAdapterError>,
    ) -> ClientInitializeError {
        ClientInitializeError::TransportError {
            error: DynamicTransportError::from_parts(
                "streamable_http",
                TypeId::of::<()>(),
                Box::new(error),
            ),
            context: "send initialize request".into(),
        }
    }

    #[test]
    fn authentication_required_looks_through_a_failed_legacy_fallback() {
        let fallback_failed = |fallback| {
            Error::new(ClientInitializeError::LegacyFallbackFailed {
                discover: Box::new(ClientInitializeError::ConnectionClosed(
                    "discover".to_string(),
                )),
                fallback: Box::new(fallback),
            })
        };

        assert!(is_authentication_required_error(&fallback_failed(
            initialize_transport_error(StreamableHttpError::AuthRequired(AuthRequiredError::new(
                "Bearer".to_string()
            ),)),
        )));
        assert!(!is_authentication_required_error(&fallback_failed(
            initialize_transport_error(StreamableHttpError::UnexpectedServerResponse(
                "HTTP 500".into(),
            )),
        )));
    }
}
