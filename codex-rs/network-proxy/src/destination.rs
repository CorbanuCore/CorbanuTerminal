//! PF-33-S01: URL, DNS and redirect enforcement for the managed proxy
//! (feature `url_destination_policy`, default off).
//!
//! Applies the frozen PF-33-S03 contract to real resolver answers and to every
//! redirect the proxy relays. Runs after, never instead of, the existing host
//! allow/deny policy: it can only narrow what that policy allows. Connection
//! pinning to the checked answers and alternate-egress denial are PF-33-S02.

use crate::destination_contract::BodyReplay;
use crate::destination_contract::ContractError;
use crate::destination_contract::CredentialReplay;
use crate::destination_contract::DESTINATION_CONTRACT_VERSION;
use crate::destination_contract::DecisionReason;
use crate::destination_contract::DestinationDecision;
use crate::destination_contract::DestinationPolicy;
use crate::destination_contract::DnsAnswerSet;
use crate::destination_contract::NormalizedDestination;
use crate::destination_contract::PolicySpec;
use crate::destination_contract::PrivateServiceSpec;
use crate::destination_contract::PublicScope;
use crate::destination_contract::RequestInput;
use crate::destination_contract::RuleSpec;
use crate::destination_contract::evaluate_destination;
use crate::destination_contract::evaluate_redirect;
use crate::destination_contract::is_intrinsically_private_name;
use crate::destination_contract::normalize_destination;
use crate::state::BlockedRequest;
use crate::state::BlockedRequestArgs;
use crate::state::NetworkProxyState;
use rama_http::Body;
use rama_http::HeaderMap;
use rama_http::Response;
use rama_http::StatusCode;
use rama_http::header;
use std::collections::VecDeque;
use std::future::Future;
use std::io;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use tracing::warn;
use url::Url;

/// Redirects one chain may take through the proxy before it is cut off.
pub(crate) const MAX_REDIRECT_HOPS: u32 = 10;
/// Wall-clock bound on one redirect chain, measured from its first request.
pub(crate) const MAX_CHAIN_AGE: Duration = Duration::from_secs(120);
const DNS_TIMEOUT: Duration = Duration::from_secs(3);
/// Matches the contract's answer limit; one extra answer makes the set too large.
const MAX_DNS_ANSWERS: usize = 16;
const LEDGER_CAPACITY: usize = 256;
/// Public retrieval: HTTPS on 443 only. Method clamps of the network mode
/// still apply on top of this list; CONNECT is the tunnel-only authorization.
const GUARDED_METHODS: [&str; 8] = [
    "GET", "HEAD", "OPTIONS", "POST", "PUT", "PATCH", "DELETE", "CONNECT",
];
const BLOCKED_HEADER: &str = "blocked-by-destination-policy";

/// Why the destination guard refused a request, tunnel or redirect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DestinationDenial {
    Url(ContractError),
    DnsFailure,
    Decision(DecisionReason),
    RedirectLocation,
    RedirectHopLimit,
    RedirectChainExpired,
}

impl DestinationDenial {
    /// Stable, value-free reason code for logs, blocked records and responses.
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Url(ContractError::UserinfoForbidden) => "url_userinfo",
            Self::Url(ContractError::UnsupportedScheme(_)) => "url_scheme",
            Self::Url(ContractError::InvalidMethod(_)) => "method",
            Self::Url(ContractError::DnsAnswerSetTooLarge) => "dns_answers_too_many",
            Self::Url(ContractError::MalformedDnsAddress(_)) => "dns_failure",
            Self::Url(_) => "url_ambiguous",
            Self::DnsFailure => "dns_failure",
            Self::Decision(reason) => decision_code(*reason),
            Self::RedirectLocation => "redirect_location",
            Self::RedirectHopLimit => "redirect_hop_limit",
            Self::RedirectChainExpired => "redirect_chain_expired",
        }
    }
}

fn decision_code(reason: DecisionReason) -> &'static str {
    match reason {
        DecisionReason::PublicRuleMismatch | DecisionReason::ExplicitDenyAll => {
            "scheme_port_or_method"
        }
        DecisionReason::EmptyDnsAnswerSet => "dns_no_answers",
        DecisionReason::AddressLiteralMismatch => "address_literal",
        DecisionReason::PrivateDestinationNotAuthorized => "private_destination",
        DecisionReason::MixedPublicAndPrivateAnswers => "dns_mixed_answers",
        DecisionReason::PrivateAddressSetMismatch => "private_address_set",
        DecisionReason::RedirectStatusUnsupported => "redirect_status",
        DecisionReason::RedirectDowngrade => "redirect_downgrade",
        DecisionReason::CrossOriginCredentialReplay => "redirect_credentials",
        DecisionReason::RedirectMethodMismatch => "redirect_method",
        DecisionReason::RedirectBodyReplayForbidden => "redirect_body",
        DecisionReason::PublicPolicyAbsent
        | DecisionReason::ExplicitPublicRule
        | DecisionReason::WildcardPublicRule
        | DecisionReason::ExplicitPrivateService => "allowed",
    }
}

pub(crate) type LookupFuture<'a> =
    Pin<Box<dyn Future<Output = io::Result<Vec<IpAddr>>> + Send + 'a>>;

/// Source of A/AAAA answers. Production uses the system resolver; tests use
/// synthetic answer sets so no private endpoint is ever contacted.
pub(crate) trait Resolve: Send + Sync {
    fn lookup<'a>(&'a self, host: &'a str, port: u16) -> LookupFuture<'a>;
}

pub(crate) struct SystemResolver;

impl Resolve for SystemResolver {
    fn lookup<'a>(&'a self, host: &'a str, port: u16) -> LookupFuture<'a> {
        Box::pin(async move {
            tokio::net::lookup_host((host, port)).await.map(|answers| {
                answers
                    .take(MAX_DNS_ANSWERS + 1)
                    .map(|answer| answer.ip())
                    .collect()
            })
        })
    }
}

/// A request the guard authorized, with the chain position it belongs to.
#[derive(Debug)]
pub(crate) struct AuthorizedRequest {
    destination: NormalizedDestination,
    chain: ChainPosition,
}

impl AuthorizedRequest {
    /// The request must not carry `Authorization`: it follows a cross-origin
    /// redirect and credentials do not cross origins.
    pub(crate) fn strip_credentials(&self) -> bool {
        self.chain.strip_credentials
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Origin {
    scheme: String,
    host: String,
    port: u16,
}

impl Origin {
    fn of(destination: &NormalizedDestination) -> Self {
        Self {
            scheme: destination.scheme().to_owned(),
            host: destination.host().to_owned(),
            port: destination.port(),
        }
    }
}

#[derive(Debug, Clone)]
struct ChainPosition {
    hops: u32,
    started: Instant,
    origin: Origin,
    strip_credentials: bool,
}

/// Pending redirect targets, so a followed redirect inherits its chain's hop
/// count, start time and credential origin. Bounded and expiring: a request
/// that matches nothing simply starts a new, fully authorized chain.
pub(crate) struct RedirectLedger {
    pending: Mutex<VecDeque<(String, ChainPosition)>>,
}

impl Default for RedirectLedger {
    fn default() -> Self {
        Self {
            pending: Mutex::new(VecDeque::new()),
        }
    }
}

impl RedirectLedger {
    fn take(&self, key: &str, now: Instant) -> Option<ChainPosition> {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        pending.retain(|(_, position)| now.duration_since(position.started) <= MAX_CHAIN_AGE);
        let index = pending
            .iter()
            .position(|(pending_key, _)| pending_key == key)?;
        pending.remove(index).map(|(_, position)| position)
    }

    fn record(&self, key: String, position: ChainPosition) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        pending.retain(|(pending_key, _)| pending_key != &key);
        if pending.len() >= LEDGER_CAPACITY {
            pending.pop_front();
        }
        pending.push_back((key, position));
    }
}

/// Whether a request carries a body that a 307/308 follow-up would replay.
pub(crate) fn request_has_body(headers: &HeaderMap) -> bool {
    headers.contains_key(header::TRANSFER_ENCODING)
        || headers
            .get(header::CONTENT_LENGTH)
            .is_some_and(|value| value.to_str().map_or(true, |value| value.trim() != "0"))
}

pub(crate) struct DestinationGuard {
    policy: DestinationPolicy,
    /// Without private-service grants, reserved names never reach a resolver.
    has_private_grants: bool,
}

impl DestinationGuard {
    /// The protected-mode policy: any public HTTPS destination on 443 that the
    /// host policy already allowed. Private networks need an explicit
    /// private-service grant; `allow_local_binding` and literal local
    /// allowlist entries are not trust grants under this guard.
    pub(crate) fn protected() -> &'static Self {
        static PROTECTED: LazyLock<DestinationGuard> = LazyLock::new(|| {
            DestinationGuard::with_private_services(Vec::new())
                .unwrap_or_else(|_| unreachable!("the fixed protected policy compiles"))
        });
        &PROTECTED
    }

    pub(crate) fn with_private_services(
        private_services: Vec<PrivateServiceSpec>,
    ) -> Result<Self, ContractError> {
        let has_private_grants = !private_services.is_empty();
        let policy = DestinationPolicy::compile(PolicySpec {
            version: DESTINATION_CONTRACT_VERSION.to_owned(),
            public_scope: PublicScope::Rules(vec![RuleSpec {
                host: "*".to_owned(),
                schemes: vec!["https".to_owned()],
                ports: vec![443],
                methods: GUARDED_METHODS.iter().map(ToString::to_string).collect(),
                path_prefixes: vec!["/".to_owned()],
            }]),
            private_services,
        })?;
        Ok(Self {
            policy,
            has_private_grants,
        })
    }

    /// Authorize a CONNECT or SOCKS tunnel, where only the authority is known.
    pub(crate) async fn authorize_tunnel(
        &self,
        host: &str,
        port: u16,
        resolver: &dyn Resolve,
    ) -> Result<(), DestinationDenial> {
        let url = format!("https://{}:{port}/", bracket_ipv6(host));
        let destination = normalize(&url, "CONNECT")?;
        let answers = self.resolve(&destination, resolver).await?;
        decide(evaluate_destination(&self.policy, &destination, &answers))
    }

    /// Authorize one HTTP request against its URL and current DNS answers.
    pub(crate) async fn authorize_url(
        &self,
        url: &str,
        method: &str,
        resolver: &dyn Resolve,
    ) -> Result<NormalizedDestination, DestinationDenial> {
        let destination = normalize(url, method)?;
        let answers = self.resolve(&destination, resolver).await?;
        decide(evaluate_destination(&self.policy, &destination, &answers))?;
        Ok(destination)
    }

    /// Authorize one HTTP request and place it in its redirect chain.
    pub(crate) async fn authorize_request(
        &self,
        url: &str,
        method: &str,
        ledger: &RedirectLedger,
        resolver: &dyn Resolve,
    ) -> Result<AuthorizedRequest, DestinationDenial> {
        let destination = self.authorize_url(url, method, resolver).await?;
        let origin = Origin::of(&destination);
        let now = Instant::now();
        let chain = match ledger.take(&chain_key(url)?, now) {
            Some(mut position) => {
                position.strip_credentials = position.origin != origin;
                position
            }
            None => ChainPosition {
                hops: 0,
                started: now,
                origin,
                strip_credentials: false,
            },
        };
        Ok(AuthorizedRequest { destination, chain })
    }

    /// Re-authorize a redirect the upstream returned before relaying it. The
    /// target is resolved now and checked again when the client follows it.
    /// Returns the absolute target URL on success.
    pub(crate) async fn authorize_redirect(
        &self,
        request: &AuthorizedRequest,
        has_body: bool,
        status: u16,
        location: &str,
        ledger: &RedirectLedger,
        resolver: &dyn Resolve,
    ) -> Result<String, DestinationDenial> {
        let hops = request.chain.hops + 1;
        if hops > MAX_REDIRECT_HOPS {
            return Err(DestinationDenial::RedirectHopLimit);
        }
        if request.chain.started.elapsed() > MAX_CHAIN_AGE {
            return Err(DestinationDenial::RedirectChainExpired);
        }
        let from = &request.destination;
        let base = Url::parse(&format!(
            "{}://{}:{}{}",
            from.scheme(),
            bracket_ipv6(from.host()),
            from.port(),
            from.path()
        ))
        .map_err(|_| DestinationDenial::RedirectLocation)?;
        let mut target = base
            .join(location)
            .map_err(|_| DestinationDenial::RedirectLocation)?;
        // Clients never send a fragment; it is not part of the next request.
        target.set_fragment(None);
        let target = target.as_str();
        // 303 turns the follow-up into a bodiless GET; 301/302/307/308 keep the
        // method and (for 307/308) replay the body.
        let (method, body) = match status {
            303 => ("GET", BodyReplay::None),
            _ if has_body => (from.method(), BodyReplay::Replayable),
            _ => (from.method(), BodyReplay::None),
        };
        let to = normalize(target, method)?;
        let answers = self.resolve(&to, resolver).await?;
        // The proxy strips `Authorization` from every hop outside the chain's
        // origin (`strip_credentials`), so no credential is replayed across
        // origins and the contract sees none.
        decide(evaluate_redirect(
            &self.policy,
            from,
            &to,
            status,
            CredentialReplay::None,
            body,
            &answers,
        ))?;
        ledger.record(
            chain_key(target)?,
            ChainPosition {
                hops,
                started: request.chain.started,
                origin: request.chain.origin.clone(),
                strip_credentials: false,
            },
        );
        Ok(target.to_owned())
    }

    async fn resolve(
        &self,
        destination: &NormalizedDestination,
        resolver: &dyn Resolve,
    ) -> Result<DnsAnswerSet, DestinationDenial> {
        let host = destination.host();
        let answers = if let Ok(literal) = host.parse::<IpAddr>() {
            vec![literal]
        } else if !self.has_private_grants && is_intrinsically_private_name(host) {
            return Err(DestinationDenial::Decision(
                DecisionReason::PrivateDestinationNotAuthorized,
            ));
        } else {
            tokio::time::timeout(DNS_TIMEOUT, resolver.lookup(host, destination.port()))
                .await
                .map_err(|_| DestinationDenial::DnsFailure)?
                .map_err(|_| DestinationDenial::DnsFailure)?
        };
        let answers = answers.iter().map(ToString::to_string).collect::<Vec<_>>();
        let answers = answers.iter().map(String::as_str).collect::<Vec<_>>();
        DnsAnswerSet::parse(&answers).map_err(DestinationDenial::Url)
    }
}

fn normalize(url: &str, method: &str) -> Result<NormalizedDestination, DestinationDenial> {
    normalize_destination(RequestInput { url, method }).map_err(DestinationDenial::Url)
}

fn decide(decision: DestinationDecision) -> Result<(), DestinationDenial> {
    if decision.allowed() {
        Ok(())
    } else {
        Err(DestinationDenial::Decision(decision.reason()))
    }
}

/// Ledger key: the canonical absolute URL, including query, without fragment.
fn chain_key(url: &str) -> Result<String, DestinationDenial> {
    let mut parsed = Url::parse(url).map_err(|_| DestinationDenial::RedirectLocation)?;
    parsed.set_fragment(None);
    let host = parsed
        .host_str()
        .ok_or(DestinationDenial::RedirectLocation)?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let port = parsed
        .port_or_known_default()
        .ok_or(DestinationDenial::RedirectLocation)?;
    let query = parsed
        .query()
        .map(|query| format!("?{query}"))
        .unwrap_or_default();
    Ok(format!(
        "{}://{host}:{port}{}{query}",
        parsed.scheme(),
        parsed.path()
    ))
}

fn bracket_ipv6(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_owned()
    }
}

/// Whether the session armed the guard (Core sets this from the feature flag).
/// Callers treat an error like any other unreadable proxy config: they refuse.
pub(crate) async fn guard_enabled(state: &NetworkProxyState) -> anyhow::Result<bool> {
    Ok(state.current_cfg().await?.url_destination_policy)
}

/// Where a denial happened, for the blocked-request record. Holds no URL
/// path or query.
pub(crate) struct DenialSite<'a> {
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    pub(crate) method: Option<&'a str>,
    pub(crate) protocol: &'a str,
    pub(crate) client: Option<String>,
    /// Report a `deny` decision, which fails the agent command with the
    /// reason (tunnel and plain-HTTP refusals, like the host policy). Denials
    /// of inner HTTPS requests and redirects only answer 403, like the
    /// existing interception checks.
    pub(crate) fail_command: bool,
}

/// Record and log a denial by reason code only.
pub(crate) async fn record_denial(
    state: &NetworkProxyState,
    denial: &DestinationDenial,
    site: DenialSite<'_>,
) {
    let code = denial.code();
    let _ = state
        .record_blocked(BlockedRequest::new(BlockedRequestArgs {
            host: site.host.to_owned(),
            reason: format!("destination_policy:{code}"),
            client: site.client,
            method: site.method.map(str::to_owned),
            mode: None,
            protocol: site.protocol.to_owned(),
            decision: site.fail_command.then(|| "deny".to_owned()),
            source: site.fail_command.then(|| "destination_policy".to_owned()),
            port: Some(site.port),
        }))
        .await;
    warn!(
        "destination policy denied {} (host={}, port={}, reason={code})",
        site.protocol, site.host, site.port
    );
}

/// Record a denial and build the 403 the client sees.
pub(crate) async fn blocked(
    state: &NetworkProxyState,
    denial: &DestinationDenial,
    site: DenialSite<'_>,
) -> Response {
    record_denial(state, denial, site).await;
    blocked_response(denial.code())
}

pub(crate) fn blocked_response(code: &str) -> Response {
    let body = format!("Destination policy denied the request ({code}).");
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("content-type", "text/plain")
        .header("x-proxy-error", BLOCKED_HEADER)
        .body(Body::from(body.clone()))
        .unwrap_or_else(|_| Response::new(Body::from(body)))
}

#[cfg(test)]
#[path = "destination_tests.rs"]
mod tests;
