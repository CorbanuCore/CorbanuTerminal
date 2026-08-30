//! Pure destination-policy decisions for PF-33-S03.
//!
//! This module deliberately opens no sockets and is not registered in the
//! network-proxy runtime. Callers must supply the complete DNS answer set they
//! observed; filtering that set is a security-significant contract violation.
//! Redirect decisions cover one hop, so consumers must retain chain history and
//! enforce hop limits. Real resolver, connected-peer, retry, pool, proxy,
//! operator-specific translation-prefix, and alternate-egress enforcement
//! belongs to PF-33-S01/S02.

use std::collections::BTreeSet;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use thiserror::Error;
use url::Host;
use url::Url;

pub const DESTINATION_CONTRACT_VERSION: &str = "pf33-destination-policy/v1";
const MAX_URL_BYTES: usize = 4096;
const MAX_DNS_ANSWERS: usize = 16;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContractError {
    #[error("unsupported destination contract version: {0}")]
    UnsupportedVersion(String),
    #[error("malformed URL: {0}")]
    MalformedUrl(String),
    #[error("URL userinfo is forbidden")]
    UserinfoForbidden,
    #[error("URL fragments are forbidden")]
    FragmentForbidden,
    #[error("unsupported URL scheme: {0}")]
    UnsupportedScheme(String),
    #[error("URL has no host")]
    MissingHost,
    #[error("ambiguous URL syntax is forbidden")]
    AmbiguousUrl,
    #[error("invalid HTTP method: {0}")]
    InvalidMethod(String),
    #[error("malformed policy: {0}")]
    MalformedPolicy(String),
    #[error("malformed DNS address: {0}")]
    MalformedDnsAddress(String),
    #[error("DNS answer set exceeds the contract limit")]
    DnsAnswerSetTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestInput<'a> {
    pub url: &'a str,
    pub method: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDestination {
    scheme: String,
    host: String,
    port: u16,
    method: String,
    path: String,
}

impl NormalizedDestination {
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn origin_matches(&self, other: &Self) -> bool {
        self.scheme == other.scheme && self.host == other.host && self.port == other.port
    }
}

pub fn normalize_destination(
    input: RequestInput<'_>,
) -> Result<NormalizedDestination, ContractError> {
    if input.url.len() > MAX_URL_BYTES
        || input.url.trim() != input.url
        || input.url.contains('\\')
        || input
            .url
            .chars()
            .any(|character| character.is_ascii_control())
    {
        return Err(ContractError::AmbiguousUrl);
    }
    if let Some((_, remainder)) = input.url.split_once("://") {
        let authority = remainder.split(['/', '?', '#']).next().unwrap_or_default();
        if authority.ends_with(':') {
            return Err(ContractError::AmbiguousUrl);
        }
    }
    let parsed =
        Url::parse(input.url).map_err(|error| ContractError::MalformedUrl(error.to_string()))?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(ContractError::UserinfoForbidden);
    }
    if parsed.fragment().is_some() {
        return Err(ContractError::FragmentForbidden);
    }
    let scheme = parsed.scheme().to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") {
        return Err(ContractError::UnsupportedScheme(scheme));
    }
    let host = parsed.host().ok_or(ContractError::MissingHost)?;
    let host = normalize_url_host(host)?;
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| ContractError::UnsupportedScheme(scheme.clone()))?;
    let method = normalize_method(input.method)?;
    let path = parsed.path().to_owned();
    if !path.starts_with('/') {
        return Err(ContractError::MalformedUrl(
            "normalized path must be absolute".to_owned(),
        ));
    }
    Ok(NormalizedDestination {
        scheme,
        host,
        port,
        method,
        path,
    })
}

fn normalize_method(method: &str) -> Result<String, ContractError> {
    if method.is_empty()
        || !method
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ContractError::InvalidMethod(method.to_owned()));
    }
    Ok(method.to_owned())
}

fn normalize_url_host(host: Host<&str>) -> Result<String, ContractError> {
    match host {
        Host::Domain(domain) => normalize_domain(domain),
        Host::Ipv4(address) => Ok(address.to_string()),
        Host::Ipv6(address) => Ok(canonical_ip(IpAddr::V6(address)).to_string()),
    }
}

fn normalize_domain(domain: &str) -> Result<String, ContractError> {
    let normalized = domain
        .strip_suffix('.')
        .unwrap_or(domain)
        .to_ascii_lowercase();
    if normalized.is_empty()
        || normalized.len() > 253
        || normalized.starts_with('.')
        || normalized.ends_with('.')
        || normalized.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || !label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                || !label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                || !label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
        })
    {
        return Err(ContractError::MalformedUrl(
            "invalid host labels".to_owned(),
        ));
    }
    Ok(normalized)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySpec {
    pub version: String,
    /// Public scope must be named explicitly. `Rules([])` is an explicit
    /// deny-all; a rule with host `*` is wildcard public scope.
    pub public_scope: PublicScope,
    /// Private services are separate exact-host grants with pinned address sets.
    pub private_services: Vec<PrivateServiceSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicScope {
    /// Deliberately impose no additional restriction on public destinations.
    /// Configuration loaders must never infer this from absent, unknown, or
    /// malformed protected configuration.
    Unrestricted,
    Rules(Vec<RuleSpec>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSpec {
    pub host: String,
    pub schemes: Vec<String>,
    pub ports: Vec<u16>,
    pub methods: Vec<String>,
    pub path_prefixes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateServiceSpec {
    pub host: String,
    pub port: u16,
    pub methods: Vec<String>,
    pub path_prefixes: Vec<String>,
    pub approved_addresses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationPolicy {
    public_scope: CompiledPublicScope,
    private_services: Vec<CompiledPrivateService>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompiledPublicScope {
    Unrestricted,
    Rules(Vec<CompiledRule>),
}

impl DestinationPolicy {
    pub fn compile(spec: PolicySpec) -> Result<Self, ContractError> {
        if spec.version != DESTINATION_CONTRACT_VERSION {
            return Err(ContractError::UnsupportedVersion(spec.version));
        }
        let public_scope = match spec.public_scope {
            PublicScope::Unrestricted => CompiledPublicScope::Unrestricted,
            PublicScope::Rules(rules) => CompiledPublicScope::Rules(
                rules
                    .into_iter()
                    .map(CompiledRule::compile)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        };
        let private_services = spec
            .private_services
            .into_iter()
            .map(CompiledPrivateService::compile)
            .collect::<Result<Vec<_>, _>>()?;
        let mut private_identities = BTreeSet::new();
        for service in &private_services {
            if !private_identities.insert((service.host.clone(), service.port)) {
                return Err(ContractError::MalformedPolicy(format!(
                    "duplicate private service {}:{}",
                    service.host, service.port
                )));
            }
        }
        Ok(Self {
            public_scope,
            private_services,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HostPattern {
    AnyPublic,
    Exact(String),
    Subdomains(String),
}

impl HostPattern {
    fn compile(input: &str) -> Result<Self, ContractError> {
        if input == "*" {
            return Ok(Self::AnyPublic);
        }
        if let Some(suffix) = input.strip_prefix("*.") {
            let suffix = normalize_policy_host(suffix)?;
            if suffix.parse::<IpAddr>().is_ok() {
                return Err(ContractError::MalformedPolicy(
                    "IP literals cannot use wildcard host rules".to_owned(),
                ));
            }
            return Ok(Self::Subdomains(suffix));
        }
        if input.contains('*') {
            return Err(ContractError::MalformedPolicy(format!(
                "unsupported wildcard host {input:?}"
            )));
        }
        Ok(Self::Exact(normalize_policy_host(input)?))
    }

    fn matches(&self, host: &str) -> bool {
        match self {
            Self::AnyPublic => true,
            Self::Exact(expected) => host == expected,
            Self::Subdomains(suffix) => {
                host.len() > suffix.len()
                    && host.ends_with(suffix)
                    && host.as_bytes()[host.len() - suffix.len() - 1] == b'.'
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledRule {
    host: HostPattern,
    schemes: BTreeSet<String>,
    ports: BTreeSet<u16>,
    methods: BTreeSet<String>,
    path_prefixes: Vec<String>,
}

impl CompiledRule {
    fn compile(spec: RuleSpec) -> Result<Self, ContractError> {
        let host = HostPattern::compile(&spec.host)?;
        let schemes = compile_schemes(spec.schemes)?;
        let ports = compile_ports(spec.ports)?;
        let methods = compile_methods(spec.methods)?;
        let path_prefixes = compile_paths(spec.path_prefixes)?;
        Ok(Self {
            host,
            schemes,
            ports,
            methods,
            path_prefixes,
        })
    }

    fn matches(&self, destination: &NormalizedDestination) -> bool {
        self.host.matches(&destination.host)
            && self.schemes.contains(&destination.scheme)
            && self.ports.contains(&destination.port)
            && self.methods.contains(&destination.method)
            && self
                .path_prefixes
                .iter()
                .any(|prefix| path_prefix_matches(prefix, &destination.path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledPrivateService {
    host: String,
    port: u16,
    methods: BTreeSet<String>,
    path_prefixes: Vec<String>,
    approved_addresses: BTreeSet<IpAddr>,
}

impl CompiledPrivateService {
    fn compile(spec: PrivateServiceSpec) -> Result<Self, ContractError> {
        if spec.host == "*" || spec.host.starts_with("*.") || spec.host.contains('*') {
            return Err(ContractError::MalformedPolicy(
                "private services require an exact host".to_owned(),
            ));
        }
        if spec.port == 0 {
            return Err(ContractError::MalformedPolicy(
                "private service port cannot be zero".to_owned(),
            ));
        }
        let host = normalize_policy_host(&spec.host)?;
        let methods = compile_methods(spec.methods)?;
        let path_prefixes = compile_paths(spec.path_prefixes)?;
        let approved_addresses = spec
            .approved_addresses
            .into_iter()
            .map(|address| parse_ip(&address))
            .collect::<Result<BTreeSet<_>, _>>()?;
        if approved_addresses.is_empty() {
            return Err(ContractError::MalformedPolicy(
                "private service address set cannot be empty".to_owned(),
            ));
        }
        if approved_addresses
            .iter()
            .any(|address| is_public_address(*address))
        {
            return Err(ContractError::MalformedPolicy(
                "private service address set cannot contain public addresses".to_owned(),
            ));
        }
        Ok(Self {
            host,
            port: spec.port,
            methods,
            path_prefixes,
            approved_addresses,
        })
    }

    fn identity_matches(&self, destination: &NormalizedDestination) -> bool {
        destination.scheme == "https"
            && destination.host == self.host
            && destination.port == self.port
            && self.methods.contains(&destination.method)
            && self
                .path_prefixes
                .iter()
                .any(|prefix| path_prefix_matches(prefix, &destination.path))
    }
}

fn normalize_policy_host(input: &str) -> Result<String, ContractError> {
    if input.trim() != input
        || input.contains('\\')
        || input.contains(['/', '?', '#', '@'])
        || input.chars().any(char::is_control)
    {
        return Err(ContractError::MalformedPolicy(format!(
            "ambiguous host syntax: {input:?}"
        )));
    }
    if (input.starts_with('[') && !input.ends_with(']'))
        || (!input.starts_with('[') && input.contains(':'))
    {
        return Err(ContractError::MalformedPolicy(format!(
            "host must not contain port syntax: {input:?}"
        )));
    }
    let candidate = format!("https://{input}/");
    let parsed = Url::parse(&candidate)
        .map_err(|error| ContractError::MalformedPolicy(format!("invalid host: {error}")))?;
    if !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(ContractError::MalformedPolicy(format!(
            "host must not contain authority or path syntax: {input:?}"
        )));
    }
    let host = parsed
        .host()
        .ok_or_else(|| ContractError::MalformedPolicy(format!("host is missing: {input:?}")))?;
    normalize_url_host(host).map_err(|error| ContractError::MalformedPolicy(error.to_string()))
}

fn compile_schemes(values: Vec<String>) -> Result<BTreeSet<String>, ContractError> {
    if values.is_empty() {
        return Err(ContractError::MalformedPolicy(
            "scheme set cannot be empty".to_owned(),
        ));
    }
    values
        .into_iter()
        .map(|value| {
            let normalized = value.to_ascii_lowercase();
            if !matches!(normalized.as_str(), "http" | "https") {
                return Err(ContractError::MalformedPolicy(format!(
                    "unsupported policy scheme {value:?}"
                )));
            }
            Ok(normalized)
        })
        .collect()
}

fn compile_ports(values: Vec<u16>) -> Result<BTreeSet<u16>, ContractError> {
    if values.is_empty() || values.contains(&0) {
        return Err(ContractError::MalformedPolicy(
            "port set must be non-empty and non-zero".to_owned(),
        ));
    }
    Ok(values.into_iter().collect())
}

fn compile_methods(values: Vec<String>) -> Result<BTreeSet<String>, ContractError> {
    if values.is_empty() {
        return Err(ContractError::MalformedPolicy(
            "method set cannot be empty".to_owned(),
        ));
    }
    values
        .into_iter()
        .map(|value| {
            normalize_method(&value)
                .map_err(|error| ContractError::MalformedPolicy(error.to_string()))
        })
        .collect()
}

fn compile_paths(values: Vec<String>) -> Result<Vec<String>, ContractError> {
    if values.is_empty() {
        return Err(ContractError::MalformedPolicy(
            "path prefix set cannot be empty".to_owned(),
        ));
    }
    values
        .into_iter()
        .map(|value| {
            if !value.starts_with('/') || value.contains('?') || value.contains('#') {
                return Err(ContractError::MalformedPolicy(format!(
                    "invalid path prefix {value:?}"
                )));
            }
            let parsed = Url::parse(&format!("https://policy.invalid{value}"))
                .map_err(|error| ContractError::MalformedPolicy(error.to_string()))?;
            if parsed.path() != value {
                return Err(ContractError::MalformedPolicy(format!(
                    "path prefix is not canonical: {value:?}"
                )));
            }
            Ok(value)
        })
        .collect()
}

fn path_prefix_matches(prefix: &str, path: &str) -> bool {
    prefix == "/"
        || path == prefix
        || (path.starts_with(prefix)
            && (prefix.ends_with('/') || path.as_bytes().get(prefix.len()) == Some(&b'/')))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsAnswerSet {
    addresses: BTreeSet<IpAddr>,
}

impl DnsAnswerSet {
    pub fn parse(values: &[&str]) -> Result<Self, ContractError> {
        // Bound resolver work before deduplication so repeated wire answers do
        // not raise the effective limit.
        if values.len() > MAX_DNS_ANSWERS {
            return Err(ContractError::DnsAnswerSetTooLarge);
        }
        let addresses = values
            .iter()
            .map(|value| parse_ip(value))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(Self { addresses })
    }

    pub fn addresses(&self) -> &BTreeSet<IpAddr> {
        &self.addresses
    }
}

fn parse_ip(input: &str) -> Result<IpAddr, ContractError> {
    input
        .parse::<IpAddr>()
        .map(canonical_ip)
        .map_err(|_| ContractError::MalformedDnsAddress(input.to_owned()))
}

fn canonical_ip(address: IpAddr) -> IpAddr {
    match address {
        IpAddr::V6(address) => address
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(address)),
        address => address,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionReason {
    PublicPolicyAbsent,
    ExplicitPublicRule,
    WildcardPublicRule,
    ExplicitPrivateService,
    ExplicitDenyAll,
    PublicRuleMismatch,
    EmptyDnsAnswerSet,
    AddressLiteralMismatch,
    PrivateDestinationNotAuthorized,
    MixedPublicAndPrivateAnswers,
    PrivateAddressSetMismatch,
    RedirectStatusUnsupported,
    RedirectDowngrade,
    CrossOriginCredentialReplay,
    RedirectMethodMismatch,
    RedirectBodyReplayForbidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationDecision {
    allowed: bool,
    reason: DecisionReason,
}

impl DestinationDecision {
    pub fn allowed(&self) -> bool {
        self.allowed
    }

    pub fn reason(&self) -> DecisionReason {
        self.reason
    }

    fn allow(reason: DecisionReason) -> Self {
        Self {
            allowed: true,
            reason,
        }
    }

    fn deny(reason: DecisionReason) -> Self {
        Self {
            allowed: false,
            reason,
        }
    }
}

pub fn evaluate_destination(
    policy: &DestinationPolicy,
    destination: &NormalizedDestination,
    answers: &DnsAnswerSet,
) -> DestinationDecision {
    if answers.addresses.is_empty() {
        return DestinationDecision::deny(DecisionReason::EmptyDnsAnswerSet);
    }
    let private_service = policy
        .private_services
        .iter()
        .find(|service| service.identity_matches(destination));
    if let Ok(literal) = destination.host.parse::<IpAddr>() {
        let literal = canonical_ip(literal);
        if answers.addresses.len() != 1 || !answers.addresses.contains(&literal) {
            return DestinationDecision::deny(DecisionReason::AddressLiteralMismatch);
        }
    } else if is_intrinsically_private_name(&destination.host) && private_service.is_none() {
        return DestinationDecision::deny(DecisionReason::PrivateDestinationNotAuthorized);
    }
    if let Some(service) = private_service {
        let public_count = answers
            .addresses
            .iter()
            .filter(|address| is_public_address(**address))
            .count();
        if public_count != 0 && public_count != answers.addresses.len() {
            return DestinationDecision::deny(DecisionReason::MixedPublicAndPrivateAnswers);
        }
        return evaluate_private_service(service, answers);
    }
    let public_count = answers
        .addresses
        .iter()
        .filter(|address| is_public_address(**address))
        .count();
    if public_count == answers.addresses.len() {
        return evaluate_public(policy, destination);
    }
    if public_count != 0 {
        return DestinationDecision::deny(DecisionReason::MixedPublicAndPrivateAnswers);
    }
    evaluate_private(policy, destination, answers)
}

fn evaluate_public(
    policy: &DestinationPolicy,
    destination: &NormalizedDestination,
) -> DestinationDecision {
    let CompiledPublicScope::Rules(rules) = &policy.public_scope else {
        return DestinationDecision::allow(DecisionReason::PublicPolicyAbsent);
    };
    if rules.is_empty() {
        return DestinationDecision::deny(DecisionReason::ExplicitDenyAll);
    }
    for rule in rules {
        if rule.matches(destination) {
            return DestinationDecision::allow(match &rule.host {
                HostPattern::AnyPublic => DecisionReason::WildcardPublicRule,
                _ => DecisionReason::ExplicitPublicRule,
            });
        }
    }
    DestinationDecision::deny(DecisionReason::PublicRuleMismatch)
}

fn evaluate_private(
    policy: &DestinationPolicy,
    destination: &NormalizedDestination,
    answers: &DnsAnswerSet,
) -> DestinationDecision {
    let Some(service) = policy
        .private_services
        .iter()
        .find(|service| service.identity_matches(destination))
    else {
        return DestinationDecision::deny(DecisionReason::PrivateDestinationNotAuthorized);
    };
    evaluate_private_service(service, answers)
}

fn evaluate_private_service(
    service: &CompiledPrivateService,
    answers: &DnsAnswerSet,
) -> DestinationDecision {
    if !answers
        .addresses
        .iter()
        .all(|address| service.approved_addresses.contains(address))
    {
        return DestinationDecision::deny(DecisionReason::PrivateAddressSetMismatch);
    }
    DestinationDecision::allow(DecisionReason::ExplicitPrivateService)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialReplay {
    None,
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyReplay {
    None,
    Replayable,
    NonReplayable,
}

pub fn evaluate_redirect(
    policy: &DestinationPolicy,
    from: &NormalizedDestination,
    to: &NormalizedDestination,
    status: u16,
    credentials: CredentialReplay,
    body: BodyReplay,
    answers: &DnsAnswerSet,
) -> DestinationDecision {
    if !matches!(status, 301 | 302 | 303 | 307 | 308) {
        return DestinationDecision::deny(DecisionReason::RedirectStatusUnsupported);
    }
    if from.scheme == "https" && to.scheme != "https" {
        return DestinationDecision::deny(DecisionReason::RedirectDowngrade);
    }
    if credentials == CredentialReplay::Present && !from.origin_matches(to) {
        return DestinationDecision::deny(DecisionReason::CrossOriginCredentialReplay);
    }
    match status {
        303 => {
            if to.method != "GET" || body != BodyReplay::None {
                return DestinationDecision::deny(DecisionReason::RedirectMethodMismatch);
            }
        }
        307 | 308 => {
            if to.method != from.method {
                return DestinationDecision::deny(DecisionReason::RedirectMethodMismatch);
            }
            if body == BodyReplay::NonReplayable {
                return DestinationDecision::deny(DecisionReason::RedirectBodyReplayForbidden);
            }
        }
        301 | 302 => {
            if !matches!(from.method.as_str(), "GET" | "HEAD")
                || to.method != from.method
                || body != BodyReplay::None
            {
                return DestinationDecision::deny(DecisionReason::RedirectBodyReplayForbidden);
            }
        }
        _ => unreachable!("redirect status was validated"),
    }
    evaluate_destination(policy, to, answers)
}

fn is_public_address(address: IpAddr) -> bool {
    match canonical_ip(address) {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let [a, b, c, d] = address.octets();
    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 192 && b == 88 && c == 99)
        || (a == 192 && b == 168)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224
        || (a == 255 && b == 255 && c == 255 && d == 255))
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    let segments = address.segments();
    // Accept ordinary global unicast only. Translation, tunnel, documentation,
    // local, and multicast prefixes remain private-policy territory.
    segments[0] & 0xe000 == 0x2000
        && !(segments[0] == 0x2001 && (segments[1] < 0x0200 || segments[1] == 0x0db8))
        && segments[0] != 0x2002
        && !(segments[0] == 0x3fff && segments[1] & 0xf000 == 0)
}

fn is_intrinsically_private_name(host: &str) -> bool {
    if !host.contains('.') {
        return true;
    }
    [
        "localhost",
        "local",
        "internal",
        "invalid",
        "test",
        "onion",
        "arpa",
        "alt",
        "lan",
        "corp",
        "home",
        "mail",
        "intranet",
        "intra",
        "private",
    ]
    .iter()
    .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
}
