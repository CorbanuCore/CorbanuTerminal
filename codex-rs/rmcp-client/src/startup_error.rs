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
<<<<<<< HEAD
                .is_some_and(client_initialize_error_requires_authentication)
            || source
                .downcast_ref::<rmcp::service::ServiceError>()
                .is_some_and(service_error_requires_authentication)
            || source
                .downcast_ref::<crate::rmcp_client::ClientOperationError>()
                .is_some_and(|error| {
                    // The transparent operation wrapper forwards its source, skipping
                    // ServiceError itself in anyhow's ordinary error-chain traversal.
                    matches!(error, crate::rmcp_client::ClientOperationError::Service(error)
                    if service_error_requires_authentication(error))
||||||| 413492cd6c
                .is_some_and(client_initialize_error_requires_authentication)
=======
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
            || source
                .downcast_ref::<ClientOperationError>()
                .is_some_and(|error| {
                    matches!(
                        error,
                        ClientOperationError::Service(ServiceError::TransportSend(error))
                            if transport_error_requires_authentication(error)
                    )
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                })
    })
}

<<<<<<< HEAD
fn service_error_requires_authentication(error: &rmcp::service::ServiceError) -> bool {
    let rmcp::service::ServiceError::TransportSend(error) = error else {
        return false;
    };
    error
        .error
        .downcast_ref::<StreamableHttpError<StreamableHttpClientAdapterError>>()
        .is_some_and(streamable_error_requires_authentication)
}

fn client_initialize_error_requires_authentication(error: &ClientInitializeError) -> bool {
    let ClientInitializeError::TransportError { error, .. } = error else {
        return false;
    };

||||||| 413492cd6c
fn client_initialize_error_requires_authentication(error: &ClientInitializeError) -> bool {
    let ClientInitializeError::TransportError { error, .. } = error else {
        return false;
    };

=======
fn transport_error_requires_authentication(error: &DynamicTransportError) -> bool {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    error
        .error
        .downcast_ref::<StreamableHttpError<StreamableHttpClientAdapterError>>()
<<<<<<< HEAD
        .is_some_and(streamable_error_requires_authentication)
}

fn streamable_error_requires_authentication(
    error: &StreamableHttpError<StreamableHttpClientAdapterError>,
) -> bool {
    match error {
        StreamableHttpError::AuthRequired(_) => true,
        StreamableHttpError::Auth(error) => auth_error_requires_authentication(error),
        _ => false,
    }
||||||| 413492cd6c
        .is_some_and(|error| {
            matches!(
                error,
                StreamableHttpError::Auth(auth_error)
                    if auth_error_requires_authentication(auth_error)
            )
        })
=======
        .is_some_and(|error| match error {
            StreamableHttpError::AuthRequired(_) => true,
            StreamableHttpError::Auth(auth_error) => auth_error_requires_authentication(auth_error),
            StreamableHttpError::Client(StreamableHttpClientAdapterError::HttpRequest(error)) => {
                exec_server_error_requires_authentication(error)
            }
            _ => false,
        })
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
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
