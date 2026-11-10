use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use codex_exec_server::HTTP_ADDRESS_POLICY_DENIED_PREFIX;
use codex_exec_server::HTTP_ADDRESS_POLICY_UNSUPPORTED;
use codex_exec_server::HttpAddressClass;
use codex_exec_server::HttpAddressPolicy;
use codex_exec_server::HttpClient;
use codex_exec_server::HttpHeader;
use codex_exec_server::HttpRedirectPolicy;
use codex_exec_server::HttpRequestParams;
use http::HeaderMap;
use oauth2::HttpRequest;
use oauth2::HttpResponse;
use rmcp::transport::auth::OAuthHttpClient;
use rmcp::transport::auth::OAuthHttpClientError;
use rmcp::transport::auth::OAuthHttpClientFuture;
use rmcp::transport::auth::OAuthHttpRedirectPolicy;
use rmcp::transport::auth::OAuthHttpRequest;
use url::Host;
use url::Url;

use crate::auth_status::OAuthDiscoveryTimeout;

const MAX_OAUTH_HTTP_RESPONSE_BODY_BYTES: usize = 1024 * 1024;
static NEXT_OAUTH_REQUEST_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, thiserror::Error)]
enum OAuthHttpClientAdapterError {
    #[error("unsupported OAuth HTTP redirect policy")]
    UnsupportedRedirectPolicy,
    #[error("OAuth HTTP response body exceeds {maximum_bytes} bytes")]
    ResponseBodyTooLarge { maximum_bytes: usize },
    #[error(
        "MCP OAuth request blocked: {0}. OAuth requests may reach loopback or private \
         addresses only when the MCP server URL itself is localhost or such an IP address \
         (see \"MCP OAuth Address Policy\" in docs/config.md)"
    )]
    AddressPolicy(String),
    #[error("MCP OAuth request not sent: {HTTP_ADDRESS_POLICY_UNSUPPORTED}")]
    AddressPolicyUnsupported,
}

/// Outbound address policy for OAuth requests made for the MCP server at
/// `server_url` (#1947), following rmcp's own OAuth client policy.
///
/// Public addresses are always allowed. Loopback addresses are allowed when
/// the server URL is `localhost` or a loopback literal, private addresses when
/// it is a private literal, and the server's own literal address always. A
/// host name's DNS answers never widen the policy, since a remote server may
/// control them.
pub(crate) fn oauth_address_policy(server_url: &str) -> HttpAddressPolicy {
    let Ok(url) = Url::parse(server_url) else {
        return HttpAddressPolicy::default();
    };
    let literal = |ip: IpAddr| {
        let class = HttpAddressClass::of(ip);
        HttpAddressPolicy {
            allow_loopback: class == HttpAddressClass::Loopback,
            allow_private: class == HttpAddressClass::Private,
            allowed_address: Some(ip),
        }
    };
    match url.host() {
        Some(Host::Ipv4(ip)) => literal(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => literal(IpAddr::V6(ip)),
        Some(Host::Domain(domain)) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            HttpAddressPolicy {
                allow_loopback: domain == "localhost" || domain.ends_with(".localhost"),
                ..HttpAddressPolicy::default()
            }
        }
        None => HttpAddressPolicy::default(),
    }
}

fn oauth_http_client_error(
    error: impl std::error::Error + Send + Sync + 'static,
) -> OAuthHttpClientError {
    Box::new(error)
}

#[derive(Clone)]
pub(crate) struct OAuthHttpClientAdapter {
    http_client: Arc<dyn HttpClient>,
    default_headers: HeaderMap,
    timeout: OAuthDiscoveryTimeout,
    address_policy: HttpAddressPolicy,
}

impl OAuthHttpClientAdapter {
    /// `server_url` is the MCP server's URL; it sets the address policy.
    pub(crate) fn new(
        http_client: Arc<dyn HttpClient>,
        default_headers: HeaderMap,
        server_url: &str,
    ) -> Self {
        Self {
            http_client,
            default_headers,
            timeout: OAuthDiscoveryTimeout::Requested,
            address_policy: oauth_address_policy(server_url),
        }
    }

    pub(crate) fn new_with_max_timeout(
        http_client: Arc<dyn HttpClient>,
        default_headers: HeaderMap,
        server_url: &str,
        max_timeout: Duration,
    ) -> Self {
        Self {
            timeout: OAuthDiscoveryTimeout::Capped(max_timeout),
            ..Self::new(http_client, default_headers, server_url)
        }
    }

    async fn execute_request(
        &self,
        request: HttpRequest,
        redirect_policy: OAuthHttpRedirectPolicy,
        timeout: Option<Duration>,
    ) -> Result<HttpResponse, OAuthHttpClientError> {
        let redirect_policy = match redirect_policy {
            OAuthHttpRedirectPolicy::Follow => HttpRedirectPolicy::Follow,
            OAuthHttpRedirectPolicy::Stop => HttpRedirectPolicy::Stop,
            _ => {
                return Err(oauth_http_client_error(
                    OAuthHttpClientAdapterError::UnsupportedRedirectPolicy,
                ));
            }
        };
        let (parts, body) = request.into_parts();
        let mut headers = self.default_headers.clone();
        for name in parts.headers.keys() {
            headers.remove(name);
        }
        headers.extend(parts.headers);

        let headers = headers
            .iter()
            .map(|(name, value)| {
                Ok(HttpHeader {
                    name: name.as_str().to_string(),
                    value: value.to_str().map_err(oauth_http_client_error)?.to_string(),
                })
            })
            .collect::<Result<Vec<_>, OAuthHttpClientError>>()?;
        let timeout = match self.timeout {
            OAuthDiscoveryTimeout::Requested => timeout,
            OAuthDiscoveryTimeout::Capped(max_timeout) => {
                Some(timeout.map_or(max_timeout, |timeout| timeout.min(max_timeout)))
            }
        };
        let timeout_ms = timeout.map(|timeout| {
            u64::try_from(timeout.as_millis())
                .unwrap_or(u64::MAX)
                .max(1)
        });
        let request_id = NEXT_OAUTH_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let (response, mut body_stream) = self
            .http_client
            .http_request_stream(HttpRequestParams {
                method: parts.method.to_string(),
                url: parts.uri.to_string(),
                headers,
                body: (!body.is_empty()).then_some(body.into()),
                timeout_ms,
                redirect_policy,
                request_id: format!("oauth-request-{request_id}"),
                stream_response: true,
                address_policy: Some(self.address_policy.clone()),
            })
            .await
            .map_err(|error| {
                let message = error.to_string();
                if message.contains(HTTP_ADDRESS_POLICY_UNSUPPORTED) {
                    return oauth_http_client_error(
                        OAuthHttpClientAdapterError::AddressPolicyUnsupported,
                    );
                }
                match message.find(HTTP_ADDRESS_POLICY_DENIED_PREFIX) {
                    Some(start) => {
                        oauth_http_client_error(OAuthHttpClientAdapterError::AddressPolicy(
                            message[start + HTTP_ADDRESS_POLICY_DENIED_PREFIX.len()..]
                                .trim_start_matches(':')
                                .trim()
                                .to_string(),
                        ))
                    }
                    None => oauth_http_client_error(error),
                }
            })?;
        let mut body = Vec::new();
        while let Some(chunk) = body_stream.recv().await.map_err(oauth_http_client_error)? {
            if chunk.len() > MAX_OAUTH_HTTP_RESPONSE_BODY_BYTES - body.len() {
                return Err(oauth_http_client_error(
                    OAuthHttpClientAdapterError::ResponseBodyTooLarge {
                        maximum_bytes: MAX_OAUTH_HTTP_RESPONSE_BODY_BYTES,
                    },
                ));
            }
            body.extend_from_slice(&chunk);
        }
        let mut builder = oauth2::http::Response::builder().status(response.status);
        for header in response.headers {
            builder = builder.header(header.name, header.value);
        }
        builder.body(body).map_err(oauth_http_client_error)
    }
}

impl OAuthHttpClient for OAuthHttpClientAdapter {
    fn execute(&self, request: OAuthHttpRequest) -> OAuthHttpClientFuture<'_> {
        Box::pin(self.execute_request(request.request, request.redirect_policy, request.timeout))
    }
}

#[cfg(test)]
#[path = "oauth_http_client_tests.rs"]
mod tests;
