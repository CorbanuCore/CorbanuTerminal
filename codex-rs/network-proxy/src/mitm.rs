use crate::certs::ManagedMitmCa;
use crate::config::NetworkMitmCaConfig;
use crate::config::NetworkMode;
use crate::credential_broker::CredentialRouting;
use crate::credential_broker::response_scrub;
use crate::destination;
use crate::destination::DenialSite;
use crate::destination::DestinationDenial;
use crate::destination::DestinationGuard;
use crate::destination::HostPatterns;
use crate::destination::RedirectLedger;
use crate::destination::RedirectScope;
use crate::destination::SystemResolver;
use crate::destination::is_redirect_response;
use crate::destination::request_has_body;
use crate::mitm_hook::HookEvaluation;
use crate::mitm_hook::MitmHookActions;
use crate::policy::normalize_host;
use crate::reasons::REASON_METHOD_NOT_ALLOWED;
use crate::reasons::REASON_MITM_HOOK_DENIED;
use crate::reasons::REASON_POLICY_DENIED;
use crate::responses::blocked_text_response;
use crate::responses::text_response;
use crate::runtime::HostAuthorization;
use crate::runtime::HostBlockDecision;
use crate::runtime::HostBlockReason;
use crate::state::BlockedRequest;
use crate::state::BlockedRequestArgs;
use crate::state::NetworkProxyState;
use crate::upstream::UpstreamClient;
use anyhow::Context as _;
use anyhow::Result;
use anyhow::anyhow;
use codex_utils_rustls_provider::ensure_rustls_crypto_provider;
use rama_core::Layer;
use rama_core::Service;
use rama_core::bytes::Bytes;
use rama_core::error::BoxError;
use rama_core::extensions::ExtensionsMut;
use rama_core::extensions::ExtensionsRef;
use rama_core::futures::stream::Stream as FuturesStream;
use rama_core::rt::Executor;
use rama_core::service::service_fn;
use rama_core::stream::Stream;
use rama_http::Body;
use rama_http::BodyDataStream;
use rama_http::HeaderMap;
use rama_http::HeaderValue;
use rama_http::Request;
use rama_http::Response;
use rama_http::StatusCode;
use rama_http::Uri;
use rama_http::header::AUTHORIZATION;
use rama_http::header::HOST;
use rama_http::layer::remove_header::RemoveRequestHeaderLayer;
use rama_http::layer::remove_header::RemoveResponseHeaderLayer;
use rama_http::uri::Scheme;
use rama_http_backend::server::HttpServer;
use rama_net::proxy::ProxyTarget;
use rama_net::stream::SocketInfo;
use rama_tls_rustls::dep::rustls;
use rama_tls_rustls::server::TlsAcceptorData;
use rama_tls_rustls::server::TlsAcceptorLayer;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context as TaskContext;
use std::task::Poll;
use tracing::info;
use tracing::warn;

/// State needed to terminate a CONNECT tunnel and enforce policy on inner HTTPS requests.
pub struct MitmState {
    ca: Arc<ManagedMitmCa>,
    allow_upstream_proxy: bool,
    upstream_tls_root_store: Arc<rustls::RootCertStore>,
    inspect: bool,
    max_body_bytes: usize,
    /// PF-33-S01 redirect chains relayed through this proxy.
    redirects: RedirectLedger,
}

pub(crate) struct MitmUpstreamConfig {
    pub(crate) allow_upstream_proxy: bool,
    pub(crate) external_ca: Option<NetworkMitmCaConfig>,
}

#[derive(Clone)]
struct MitmPolicyContext {
    target_host: String,
    target_port: u16,
    scheme: Scheme,
    mode: NetworkMode,
    app_state: Arc<NetworkProxyState>,
}

impl MitmPolicyContext {
    async fn deny_hook(&self, method: &str, client: Option<String>) -> Response {
        let _ = self
            .app_state
            .record_blocked(BlockedRequest::new(BlockedRequestArgs {
                host: self.target_host.clone(),
                reason: REASON_MITM_HOOK_DENIED.to_string(),
                client,
                method: Some(method.to_string()),
                mode: Some(self.mode),
                protocol: self.scheme.to_string(),
                decision: None,
                source: None,
                port: Some(self.target_port),
            }))
            .await;
        warn!(
            "MITM blocked by hook policy (host={}, method={method}, mode={:?})",
            self.target_host, self.mode
        );
        blocked_text_response(REASON_MITM_HOOK_DENIED)
    }

    fn placeholders_match(&self, req: &Request, actions: &MitmHookActions) -> bool {
        actions.inject_request_headers.iter().all(|header| {
            header.placeholder.as_ref().is_none_or(|placeholder| {
                self.scheme == Scheme::HTTPS
                    && self.target_port == 443
                    && req
                        .headers()
                        .get_all(&header.name)
                        .iter()
                        .eq(std::iter::once(placeholder))
            })
        })
    }

    fn credential_destination(&self, req: &Request) -> String {
        let authority = authority_header_value(&self.target_host, self.target_port, &self.scheme);
        let path = path_and_query(req.uri());
        // Server-wide OPTIONS must not borrow authority from a narrower credential URL prefix.
        let credential_path = if path == "*" { "/" } else { &path };
        format!("{}://{authority}{credential_path}", self.scheme)
    }
}

#[derive(Clone)]
struct MitmRequestContext {
    policy: MitmPolicyContext,
    mitm: Arc<MitmState>,
    upstream: UpstreamClient,
}

enum MitmPolicyDecision {
    Allow {
        hook_actions: Option<MitmHookActions>,
    },
    Block(Response),
}

const MITM_INSPECT_BODIES: bool = false;
const MITM_MAX_BODY_BYTES: usize = 4096;
impl std::fmt::Debug for MitmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Avoid dumping internal state (CA material, connectors, etc.) to logs.
        f.debug_struct("MitmState")
            .field("inspect", &self.inspect)
            .field("max_body_bytes", &self.max_body_bytes)
            .finish_non_exhaustive()
    }
}

impl MitmState {
    pub(crate) fn new(config: MitmUpstreamConfig) -> Result<Self> {
        ensure_rustls_crypto_provider();

        // MITM exists when HTTPS policy depends on the inner request: limited-mode method clamps
        // and host-specific hooks both need visibility after CONNECT is established. We
        // generate a process-local CA and issue per-host leaf certs so we can terminate TLS and
        // apply policy.
        let ca = match config.external_ca.as_ref() {
            Some(ca) => ManagedMitmCa::load_from_files(ca)?,
            None => ManagedMitmCa::load_or_create()?,
        };
        let upstream_tls_root_store = crate::certs::upstream_tls_root_store_for_cert_path(
            ca.certificate_path(),
            &crate::certs::ca_env_from_process(),
        )?;

        Ok(Self {
            ca,
            allow_upstream_proxy: config.allow_upstream_proxy,
            upstream_tls_root_store,
            inspect: MITM_INSPECT_BODIES,
            max_body_bytes: MITM_MAX_BODY_BYTES,
            redirects: RedirectLedger::default(),
        })
    }

    fn tls_acceptor_data_for_host(&self, host: &str) -> Result<TlsAcceptorData> {
        self.ca.tls_acceptor_data_for_host(host)
    }

    pub(crate) fn inspect_enabled(&self) -> bool {
        self.inspect
    }

    pub(crate) fn max_body_bytes(&self) -> usize {
        self.max_body_bytes
    }
}

/// Proxy inner HTTP traffic, terminating TLS with a generated leaf certificate for HTTPS.
pub(crate) async fn mitm_stream<S>(stream: S, scheme: Scheme) -> Result<()>
where
    S: Stream + Unpin + ExtensionsMut,
{
    let mitm = stream
        .extensions()
        .get::<Arc<MitmState>>()
        .cloned()
        .context("missing MITM state")?;
    let app_state = stream
        .extensions()
        .get::<Arc<NetworkProxyState>>()
        .cloned()
        .context("missing app state")?;
    let target = stream
        .extensions()
        .get::<ProxyTarget>()
        .context("missing proxy target")?
        .0
        .clone();
    let target_host = normalize_host(&target.host.to_string());
    let target_port = target.port;
    let acceptor_data = if scheme == Scheme::HTTPS {
        Some(mitm.tls_acceptor_data_for_host(&target_host)?)
    } else {
        None
    };
    let mode = stream
        .extensions()
        .get::<NetworkMode>()
        .copied()
        .unwrap_or(NetworkMode::Full);
    let upstream = if mitm.allow_upstream_proxy {
        UpstreamClient::from_env_proxy_with_tls_root_store(
            app_state.clone(),
            mitm.upstream_tls_root_store.clone(),
        )
    } else {
        UpstreamClient::direct_with_tls_root_store(
            app_state.clone(),
            mitm.upstream_tls_root_store.clone(),
        )
    };
    let request_ctx = Arc::new(MitmRequestContext {
        policy: MitmPolicyContext {
            target_host,
            target_port,
            scheme,
            mode,
            app_state,
        },
        mitm,
        upstream,
    });

    let executor = stream
        .extensions()
        .get::<Executor>()
        .cloned()
        .unwrap_or_default();

    let http_service = HttpServer::auto(executor).service(
        (request_ctx.policy.scheme == Scheme::HTTPS)
            .then(RemoveResponseHeaderLayer::hop_by_hop)
            .into_layer(service_fn({
                let request_ctx = request_ctx.clone();
                move |req| {
                    let request_ctx = request_ctx.clone();
                    async move { handle_mitm_request(req, request_ctx).await }
                }
            })),
    );

    match acceptor_data {
        Some(acceptor_data) => {
            TlsAcceptorLayer::new(acceptor_data)
                .with_store_client_hello(true)
                .into_layer(http_service)
                .serve(stream)
                .await
        }
        None => http_service.serve(stream).await,
    }
    .map_err(|err| anyhow!("MITM serve error: {err}"))?;
    Ok(())
}

async fn handle_mitm_request(
    mut req: Request,
    request_ctx: Arc<MitmRequestContext>,
) -> Result<Response, std::convert::Infallible> {
    crate::aws_sigv4::preserve_signing_headers(&mut req);
    crate::secret_substitution::preserve_body_substitution_headers(&mut req);
    let response = match (request_ctx.policy.scheme == Scheme::HTTPS)
        .then(RemoveRequestHeaderLayer::hop_by_hop)
        .into_layer(service_fn(move |req| {
            let request_ctx = request_ctx.clone();
            async move { forward_request(req, &request_ctx).await }
        }))
        .serve(req)
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            warn!("MITM request handling failed: {err}");
            text_response(StatusCode::BAD_GATEWAY, "mitm upstream error")
        }
    };
    Ok(response)
}

async fn forward_request(req: Request, request_ctx: &MitmRequestContext) -> Result<Response> {
    let credential_destination = request_ctx.policy.credential_destination(&req);
    let hook_actions = match evaluate_mitm_policy(&req, &request_ctx.policy).await? {
        MitmPolicyDecision::Allow { hook_actions } => hook_actions,
        MitmPolicyDecision::Block(response) => return Ok(response),
    };

    // Select the credential from the guest's headers before injecting or stripping anything.
    let mut signing = None;
    let client = req
        .extensions()
        .get::<SocketInfo>()
        .map(|info| info.peer_addr().to_string());
    if let Some(signer) = hook_actions
        .as_ref()
        .and_then(|actions| actions.aws_sigv4.as_ref())
    {
        match signer.plan(&req) {
            Ok(Some(guest)) => signing = Some((signer, guest)),
            Ok(None) => {}
            Err(_) => {
                return Ok(request_ctx
                    .policy
                    .deny_hook(req.method().as_str(), client)
                    .await);
            }
        }
    }
    let target_host = request_ctx.policy.target_host.clone();
    let target_port = request_ctx.policy.target_port;
    let mitm = request_ctx.mitm.clone();

    let method = req.method().as_str().to_string();
    let path = path_and_query(req.uri());
    let log_path = path_for_log(req.uri());
    let authority = authority_header_value(&target_host, target_port);
    let app_state = &request_ctx.policy.app_state;

    // PF-33-S01: authorize the full URL, method and DNS answers, and place the
    // request in its client's redirect chain.
    let destination = if destination::guard_enabled(app_state).await? {
        let url = format!("https://{authority}{path}");
        let client = req
            .extensions()
            .get::<SocketInfo>()
            .map(|info| info.peer_addr().to_string());
        let chain_client = format!(
            "{}|{}",
            app_state.execution_id().unwrap_or_default(),
            req.extensions()
                .get::<SocketInfo>()
                .map(|info| info.peer_addr().ip().to_string())
                .unwrap_or_default()
        );
        // PF-33-S02: an upstream proxy would resolve and connect on its own,
        // so the checked answers could not be pinned.
        if request_ctx.upstream.has_upstream_proxy() {
            let site = DenialSite {
                host: &target_host,
                port: target_port,
                method: Some(&method),
                protocol: "https",
                client,
                fail_command: false,
            };
            let denial = DestinationDenial::UpstreamProxy;
            return Ok(destination::blocked(app_state, &denial, site).await);
        }
        match DestinationGuard::protected()
            .authorize_request(
                &url,
                &method,
                &chain_client,
                &mitm.redirects,
                &SystemResolver,
            )
            .await
        {
            Ok(authorized) => Some((authorized, client)),
            Err(denial) => {
                let site = DenialSite {
                    host: &target_host,
                    port: target_port,
                    method: Some(&method),
                    protocol: "https",
                    client,
                    fail_command: false,
                };
                return Ok(destination::blocked(app_state, &denial, site).await);
            }
        }
    } else {
        None
    };

    let (mut parts, body) = req.into_parts();
<<<<<<< HEAD
    let has_body = request_has_body(&parts.headers);
    if let Some((authorized, _)) = destination.as_ref() {
        authorized.strip_cross_origin_credentials(&mut parts.headers);
        // PF-33-S02: the connector dials only these checked answers.
        parts.extensions.insert(authorized.pinned_peers());
    }
    if request_ctx
        .policy
        .app_state
        .scoped_credential_route_enabled()
        && hook_actions_touch_authorization(hook_actions.as_ref())
    {
        warn!("scoped credential request denied: authorization hook collision");
        return Ok(blocked_text_response(REASON_POLICY_DENIED));
    }
    let routing = match request_ctx.policy.app_state.route_request_credentials(
        "https",
        &target_host,
        target_port,
        &method,
        &path,
        &mut parts.headers,
    ) {
        Ok(routing) => routing,
        Err(error) => {
            warn!("scoped credential request denied: {error}");
            return Ok(blocked_text_response(REASON_POLICY_DENIED));
        }
    };
    #[cfg(any(unix, windows))]
    if matches!(routing, CredentialRouting::Brokered(_))
        && hook_actions_touch_authorization(hook_actions.as_ref())
    {
        warn!("brokered credential request denied: authorization hook collision");
        return Ok(blocked_text_response(REASON_POLICY_DENIED));
    }
    apply_mitm_hook_actions(&mut parts.headers, hook_actions.as_ref());
    // PF-28-S02: values injected here (credential or hook) are scrubbed from
    // the response the agent receives; the isolated broker scrubs its own.
    let reflection = if app_state.credential_response_gate() {
        let injected = match &routing {
            CredentialRouting::Direct(gate) => gate.clone(),
            #[cfg(any(unix, windows))]
            CredentialRouting::Brokered(_) => None,
        };
        match response_scrub::with_hook_values(injected, hook_actions.as_ref()) {
            Ok(reflection) => reflection,
            Err(error) => {
                warn!("credentialed request denied: response gate unavailable: {error}");
                return Ok(blocked_text_response(REASON_POLICY_DENIED));
            }
        }
    } else {
        None
    };
    if reflection.is_some() {
        response_scrub::request_identity_body(&mut parts.headers);
    }
    parts.uri = build_https_uri(&authority, &path)?;
||||||| 413492cd6c
    request_ctx
        .policy
        .app_state
        .inject_request_credentials(&target_host, &mut parts.headers);
    apply_mitm_hook_actions(&mut parts.headers, hook_actions.as_ref());
    let authority = authority_header_value(&target_host, target_port);
    parts.uri = build_https_uri(&authority, &path)?;
=======
    let authority = authority_header_value(&target_host, target_port, &request_ctx.policy.scheme);
    parts.uri = Uri::builder()
        .scheme(request_ctx.policy.scheme.clone())
        .authority(authority.as_str())
        .path_and_query(path.as_str())
        .build()?;
    apply_request_credentials(
        &mut parts.headers,
        hook_actions.as_ref(),
        request_ctx.policy.app_state.as_ref(),
        &credential_destination,
    );
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    parts
        .headers
        .insert(HOST, HeaderValue::from_str(&authority)?);

    let inspect = mitm.inspect_enabled();
    let max_body_bytes = mitm.max_body_bytes();
    let body = if inspect {
        inspect_body(
            body,
            max_body_bytes,
            RequestLogContext {
                host: authority.clone(),
                method: method.clone(),
                path: log_path.clone(),
            },
        )
    } else {
        body
    };

<<<<<<< HEAD
    let upstream_req = Request::from_parts(parts, body);
    let mut upstream_resp = match routing {
        CredentialRouting::Direct(_) => request_ctx.upstream.serve(upstream_req).await?,
        #[cfg(any(unix, windows))]
        CredentialRouting::Brokered(route) => match route.forward(upstream_req).await {
            Ok(response) => response,
            Err(error) => {
                // Fail closed: the proxy holds no raw value to fall back to.
                warn!("brokered credential request failed: {error}");
                return Ok(text_response(
                    StatusCode::BAD_GATEWAY,
                    "credential broker unavailable",
                ));
            }
        },
    };
    if let Some((authorized, client)) = destination
        && is_redirect_response(upstream_resp.status().as_u16(), upstream_resp.headers())
    {
        let status = upstream_resp.status().as_u16();
        let checked = match HostPatterns::current(app_state).await {
            Ok(hosts) => {
                DestinationGuard::protected()
                    .check_response(
                        &authorized,
                        has_body,
                        status,
                        upstream_resp.headers_mut(),
                        RedirectScope {
                            hosts: &hosts,
                            ledger: &mitm.redirects,
                            resolver: &SystemResolver,
                        },
                    )
                    .await
            }
            Err(err) => return Err(err.context("read host policy for redirect check")),
        };
        if let Err(denial) = checked {
            let site = DenialSite {
                host: &target_host,
                port: target_port,
                method: Some(&method),
                protocol: "https-redirect",
                client,
                fail_command: false,
            };
            return Ok(destination::blocked(app_state, &denial, site).await);
        }
    }
    if let Some(gate) = &reflection {
        upstream_resp = response_scrub::scrub_response(gate, upstream_resp);
    }
||||||| 413492cd6c
    let upstream_req = Request::from_parts(parts, body);
    let upstream_resp = request_ctx.upstream.serve(upstream_req).await?;
=======
    let mut upstream_req = Request::from_parts(parts, body);
    if let Some(substitution) = hook_actions
        .as_ref()
        .and_then(|actions| actions.body_secret_substitution.as_ref())
    {
        let Ok(substituted) = substitution.apply(upstream_req).await else {
            return Ok(request_ctx.policy.deny_hook(&method, client).await);
        };
        upstream_req = substituted;
    }
    if let Some((signer, guest)) = signing {
        let Ok(signed) = signer.sign(upstream_req, &guest).await else {
            return Ok(request_ctx.policy.deny_hook(&method, client).await);
        };
        upstream_req = signed;
    }
    let upstream_resp = if request_ctx.policy.scheme == Scheme::HTTP {
        crate::brokered_tunnel::forward_http_request(upstream_req, &request_ctx.upstream).await?
    } else {
        request_ctx.upstream.serve(upstream_req).await?
    };
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    respond_with_inspection(
        upstream_resp,
        inspect,
        max_body_bytes,
        &method,
        &log_path,
        &authority,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
async fn mitm_blocking_response(
    req: &Request,
    policy: &MitmPolicyContext,
) -> Result<Option<Response>> {
    match evaluate_mitm_policy(req, policy).await? {
        MitmPolicyDecision::Allow { .. } => Ok(None),
        MitmPolicyDecision::Block(response) => Ok(Some(response)),
    }
}

async fn evaluate_mitm_policy(
    req: &Request,
    policy: &MitmPolicyContext,
) -> Result<MitmPolicyDecision> {
    if policy.scheme == Scheme::HTTP {
        let matches_target = |authority: &rama_http::uri::Authority| {
            !authority.as_str().contains('@')
                && normalize_host(authority.host()) == policy.target_host
                && authority.port_u16().unwrap_or(80) == policy.target_port
        };
        let invalid_destination = req
            .uri()
            .scheme()
            .is_some_and(|scheme| scheme != &Scheme::HTTP)
            || req
                .uri()
                .authority()
                .is_some_and(|authority| !matches_target(authority))
            || req.headers().get_all(HOST).iter().any(|value| {
                value
                    .to_str()
                    .ok()
                    .and_then(|value| value.parse::<rama_http::uri::Authority>().ok())
                    .is_none_or(|authority| !matches_target(&authority))
            });
        if invalid_destination {
            return Ok(MitmPolicyDecision::Block(text_response(
                StatusCode::BAD_REQUEST,
                "tunnel destination mismatch",
            )));
        }
    }
    if req.method().as_str() == "CONNECT" {
        return Ok(MitmPolicyDecision::Block(text_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "CONNECT not supported inside MITM",
        )));
    }

    let method = req.method().as_str().to_string();
    let log_path = path_for_log(req.uri());
    let client = req
        .extensions()
        .get::<SocketInfo>()
        .map(|info| info.peer_addr().to_string());

    if let Some(request_host) = extract_request_host(req) {
        let normalized = normalize_host(&request_host);
        if !normalized.is_empty() && normalized != policy.target_host {
            warn!(
                "MITM host mismatch (target={}, request_host={normalized})",
                policy.target_host
            );
            return Ok(MitmPolicyDecision::Block(text_response(
                StatusCode::BAD_REQUEST,
                "host mismatch",
            )));
        }
    }

    // CONNECT already handled allowlist/denylist + decider policy. Re-check local/private
    // resolution here to defend against DNS rebinding between CONNECT and inner HTTPS requests.
<<<<<<< HEAD
    let host_decision = policy
        .app_state
        .host_blocked(&policy.target_host, policy.target_port)
        .await?;
    // PF-33-S02: under the destination guard, a host denied after its tunnel
    // opened is refused on the open tunnel too, not only on the next CONNECT.
    if host_decision == HostBlockDecision::Blocked(HostBlockReason::Denied)
        && destination::guard_enabled(&policy.app_state).await?
    {
        let site = DenialSite {
            host: &policy.target_host,
            port: policy.target_port,
            method: Some(&method),
            protocol: "https",
            client,
            fail_command: false,
        };
        let denial = DestinationDenial::HostDeniedAfterConnect;
        return Ok(MitmPolicyDecision::Block(
            destination::blocked(&policy.app_state, &denial, site).await,
        ));
    }
    if host_decision == HostBlockDecision::Blocked(HostBlockReason::NotAllowedLocal) {
||||||| 413492cd6c
    if matches!(
        policy
            .app_state
            .host_blocked(&policy.target_host, policy.target_port)
            .await?,
        HostBlockDecision::Blocked(HostBlockReason::NotAllowedLocal)
    ) {
=======
    if matches!(
        policy
            .app_state
            .host_blocked_with_local_binding(
                &policy.target_host,
                policy.target_port,
                /*allow_local_binding*/ None,
                HostAuthorization::Approved,
            )
            .await?,
        HostBlockDecision::Blocked(HostBlockReason::NotAllowedLocal)
    ) {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        let reason = HostBlockReason::NotAllowedLocal.as_str();
        let _ = policy
            .app_state
            .record_blocked(BlockedRequest::new(BlockedRequestArgs {
                host: policy.target_host.clone(),
                reason: reason.to_string(),
                client: client.clone(),
                method: Some(method.clone()),
                mode: Some(policy.mode),
                protocol: policy.scheme.to_string(),
                decision: None,
                source: None,
                port: Some(policy.target_port),
            }))
            .await;
        warn!(
            "MITM blocked local/private target after CONNECT (host={}, port={}, method={method}, path={log_path})",
            policy.target_host, policy.target_port
        );
        return Ok(MitmPolicyDecision::Block(blocked_text_response(reason)));
    }

    let hook_actions = match policy
        .app_state
        .evaluate_mitm_hook_request(
            &policy.target_host,
            req,
            &crate::mitm_hook::MitmHookOrigin {
                scheme: policy.scheme.clone(),
                port: policy.target_port,
            },
            &[policy.credential_destination(req)],
        )
        .await?
    {
        HookEvaluation::Matched { actions } if policy.placeholders_match(req, &actions) => {
            Some(actions)
        }
        HookEvaluation::Matched { .. } | HookEvaluation::HookedHostNoMatch => {
            return Ok(MitmPolicyDecision::Block(
                policy.deny_hook(&method, client).await,
            ));
        }
        HookEvaluation::NoHooksForHost => None,
    };

    let opaque_http_upgrade = policy.scheme == Scheme::HTTP
        && policy.mode == NetworkMode::Limited
        && req.headers().contains_key(rama_http::header::UPGRADE);
    if !policy.mode.allows_method(&method) || opaque_http_upgrade {
        let _ = policy
            .app_state
            .record_blocked(BlockedRequest::new(BlockedRequestArgs {
                host: policy.target_host.clone(),
                reason: REASON_METHOD_NOT_ALLOWED.to_string(),
                client: client.clone(),
                method: Some(method.clone()),
                mode: Some(policy.mode),
                protocol: policy.scheme.to_string(),
                decision: None,
                source: None,
                port: Some(policy.target_port),
            }))
            .await;
        warn!(
            "MITM blocked by method policy (host={}, method={method}, path={log_path}, mode={:?}, allowed_methods=GET, HEAD, OPTIONS)",
            policy.target_host, policy.mode
        );
        return Ok(MitmPolicyDecision::Block(blocked_text_response(
            REASON_METHOD_NOT_ALLOWED,
        )));
    }

    Ok(MitmPolicyDecision::Allow { hook_actions })
}

<<<<<<< HEAD
fn hook_actions_touch_authorization(actions: Option<&MitmHookActions>) -> bool {
    actions.is_some_and(|actions| {
        actions.strip_request_headers.contains(&AUTHORIZATION)
            || actions
                .inject_request_headers
                .iter()
                .any(|header| header.name == AUTHORIZATION)
    })
}

fn apply_mitm_hook_actions(headers: &mut HeaderMap, actions: Option<&MitmHookActions>) {
||||||| 413492cd6c
fn apply_mitm_hook_actions(headers: &mut HeaderMap, actions: Option<&MitmHookActions>) {
=======
fn apply_request_credentials(
    headers: &mut HeaderMap,
    actions: Option<&MitmHookActions>,
    app_state: &NetworkProxyState,
    credential_destination: &str,
) {
    app_state.inject_request_credentials(credential_destination, headers);
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    let Some(actions) = actions else {
        return;
    };
    for header_name in &actions.strip_request_headers {
        headers.remove(header_name);
    }
    for injected_header in &actions.inject_request_headers {
        headers.insert(injected_header.name.clone(), injected_header.value.clone());
    }
}

fn respond_with_inspection(
    resp: Response,
    inspect: bool,
    max_body_bytes: usize,
    method: &str,
    log_path: &str,
    authority: &str,
) -> Result<Response> {
    if !inspect {
        return Ok(resp);
    }

    let (parts, body) = resp.into_parts();
    let body = inspect_body(
        body,
        max_body_bytes,
        ResponseLogContext {
            host: authority.to_string(),
            method: method.to_string(),
            path: log_path.to_string(),
            status: parts.status,
        },
    );
    Ok(Response::from_parts(parts, body))
}

fn inspect_body<T: BodyLoggable + Send + 'static>(
    body: Body,
    max_body_bytes: usize,
    ctx: T,
) -> Body {
    Body::from_stream(InspectStream {
        inner: Box::pin(body.into_data_stream()),
        ctx: Some(Box::new(ctx)),
        len: 0,
        max_body_bytes,
    })
}

struct InspectStream<T> {
    inner: Pin<Box<BodyDataStream>>,
    ctx: Option<Box<T>>,
    len: usize,
    max_body_bytes: usize,
}

impl<T: BodyLoggable> FuturesStream for InspectStream<T> {
    type Item = Result<Bytes, BoxError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                this.len = this.len.saturating_add(bytes.len());
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => {
                if let Some(ctx) = this.ctx.take() {
                    ctx.log(this.len, this.len > this.max_body_bytes);
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

struct RequestLogContext {
    host: String,
    method: String,
    path: String,
}

struct ResponseLogContext {
    host: String,
    method: String,
    path: String,
    status: StatusCode,
}

trait BodyLoggable {
    fn log(self, len: usize, truncated: bool);
}

impl BodyLoggable for RequestLogContext {
    fn log(self, len: usize, truncated: bool) {
        let host = self.host;
        let method = self.method;
        let path = self.path;
        info!(
            "MITM inspected request body (host={host}, method={method}, path={path}, body_len={len}, truncated={truncated})"
        );
    }
}

impl BodyLoggable for ResponseLogContext {
    fn log(self, len: usize, truncated: bool) {
        let host = self.host;
        let method = self.method;
        let path = self.path;
        let status = self.status;
        info!(
            "MITM inspected response body (host={host}, method={method}, path={path}, status={status}, body_len={len}, truncated={truncated})"
        );
    }
}

fn extract_request_host(req: &Request) -> Option<String> {
    req.headers()
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string)
        .or_else(|| req.uri().authority().map(|a| a.as_str().to_string()))
}

fn authority_header_value(host: &str, port: u16, scheme: &Scheme) -> String {
    // Host header / URI authority formatting.
    let default_port = if scheme == &Scheme::HTTP { 80 } else { 443 };
    if host.contains(':') {
        if port == default_port {
            format!("[{host}]")
        } else {
            format!("[{host}]:{port}")
        }
    } else if port == default_port {
        host.to_string()
    } else {
        format!("{host}:{port}")
    }
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(rama_http::uri::PathAndQuery::as_str)
        .unwrap_or("/")
        .to_string()
}

fn path_for_log(uri: &Uri) -> String {
    uri.path().to_string()
}

#[cfg(test)]
#[path = "mitm_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "mitm_forwarding_tests.rs"]
mod forwarding_tests;
