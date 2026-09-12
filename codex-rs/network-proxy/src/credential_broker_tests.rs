use super::*;

use codex_security_policy::ActorChain;
use codex_security_policy::AuthorizationContext;
use codex_security_policy::AuthorizationRequest;
use codex_security_policy::BoundedGrant;
use codex_security_policy::BoundedText;
use codex_security_policy::CapabilityId;
use codex_security_policy::CredentialCapabilityRequest;
use codex_security_policy::CredentialDestination;
use codex_security_policy::CredentialHttpMethod;
use codex_security_policy::CredentialReference;
use codex_security_policy::GrantContext;
use codex_security_policy::GrantScope;
use codex_security_policy::PolicyAction;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use codex_security_policy::ProtectedResource;
use codex_security_policy::ResourceKind;
use codex_security_policy::RevocationState;
use pretty_assertions::assert_eq;
use rama_http::HeaderValue;
use rama_http::header::AUTHORIZATION;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

fn env_map<const N: usize>(entries: [(&str, &str); N]) -> HashMap<String, String> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn headers_with_bearer(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {value}")).expect("valid bearer header"),
    );
    headers
}

fn authorization(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
}

fn assert_credential_shape(real_value: &str, dummy_value: &str, prefix: &str) {
    assert_ne!(dummy_value, real_value);
    assert_eq!(dummy_value.len(), real_value.len());
    assert_eq!(&dummy_value[..prefix.len()], prefix);
    let same_shape = real_value
        .bytes()
        .zip(dummy_value.bytes())
        .skip(prefix.len())
        .all(|(real, dummy)| {
            real.is_ascii_alphanumeric() && dummy.is_ascii_alphanumeric() || real == dummy
        });
    assert!(same_shape);
}

#[test]
fn virtualize_child_env_replaces_supported_credentials() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let github_token = "github_pat_11AA0bbCC_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGH";
    let openai_api_key = "sk-proj-abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
    let mut env = env_map([
        ("GH_TOKEN", github_token),
        ("OPENAI_API_KEY", openai_api_key),
        ("GH_ENTERPRISE_TOKEN", "ghp-enterprise-real"),
    ]);

    broker.virtualize_child_env(&mut env);

    let github_dummy = env.get("GH_TOKEN").expect("dummy GitHub token");
    let openai_dummy = env.get("OPENAI_API_KEY").expect("dummy OpenAI API key");
    assert_credential_shape(github_token, github_dummy, "github_pat_");
    assert_credential_shape(openai_api_key, openai_dummy, "sk-proj-");
    env.insert("OPENAI_API_KEY".to_string(), "sk-user-override".to_string());
    assert_eq!(
        brokered_credential_dummy_env_keys(&env),
        vec!["GH_TOKEN".to_string()]
    );
}

#[test]
fn virtualize_child_env_preserves_live_dummy_mappings() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut first_env = env_map([("GH_TOKEN", "ghp-real-one")]);
    let mut second_env = env_map([("GH_TOKEN", "ghp-real-two")]);

    broker.virtualize_child_env(&mut first_env);
    broker.virtualize_child_env(&mut second_env);
    let first_dummy = first_env.get("GH_TOKEN").expect("first dummy token");
    let second_dummy = second_env.get("GH_TOKEN").expect("second dummy token");
    let mut first_headers = headers_with_bearer(first_dummy);
    let mut second_headers = headers_with_bearer(second_dummy);

    broker.inject_request_headers("api.github.com", &mut first_headers);
    broker.inject_request_headers("api.github.com", &mut second_headers);

    assert_eq!(authorization(&first_headers), Some("Bearer ghp-real-one"));
    assert_eq!(authorization(&second_headers), Some("Bearer ghp-real-two"));
}

#[test]
fn virtualize_child_env_uses_fresh_dummy_capabilities() {
    let mut first_env = env_map([("OPENAI_API_KEY", "sk-proj-abcdefghijklmnopqrstuvwxyz")]);
    let mut second_env = first_env.clone();

    CredentialBroker::new(/*enabled*/ true).virtualize_child_env(&mut first_env);
    CredentialBroker::new(/*enabled*/ true).virtualize_child_env(&mut second_env);

    assert_ne!(first_env["OPENAI_API_KEY"], second_env["OPENAI_API_KEY"]);
}

#[test]
fn child_without_dummy_cannot_use_previous_child_credential() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut first_env = env_map([("OPENAI_API_KEY", "sk-real")]);
    let mut second_env = HashMap::new();

    broker.virtualize_child_env(&mut first_env);
    broker.virtualize_child_env(&mut second_env);
    let mut headers = HeaderMap::new();

    broker.inject_request_headers("api.openai.com", &mut headers);

    assert_eq!(authorization(&headers), None);
}

#[test]
fn virtualize_child_env_preserves_unbound_enterprise_token() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut env = env_map([("GH_ENTERPRISE_TOKEN", "ghp-enterprise-real")]);

    broker.virtualize_child_env(&mut env);
    let inert_token = "ghp_abcdefghijklmnopqrstuvwxyz1234567890";
    let mut headers = headers_with_bearer(inert_token);
    broker.inject_request_headers("attacker.example", &mut headers);

    assert_eq!(env["GH_ENTERPRISE_TOKEN"], "ghp-enterprise-real");
    assert_eq!(headers, headers_with_bearer(inert_token));
    assert!(!broker.host_requires_mitm("attacker.example"));
}

#[test]
fn inject_request_headers_requires_dummy_to_select_ambiguous_github_credential() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut env = env_map([
        ("GH_TOKEN", "ghp-real-one"),
        ("GITHUB_TOKEN", "ghp-real-two"),
    ]);
    broker.virtualize_child_env(&mut env);
    let github_token = env.get("GITHUB_TOKEN").expect("dummy github token");
    let mut headers = HeaderMap::new();

    broker.inject_request_headers("api.github.com", &mut headers);
    assert_eq!(authorization(&headers), None);

    headers = headers_with_bearer(github_token);

    broker.inject_request_headers("api.github.com", &mut headers);

    assert_eq!(authorization(&headers), Some("Bearer ghp-real-two"));
}

#[test]
fn inject_request_headers_requires_dummy_and_preserves_explicit_authorization() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut env = env_map([("OPENAI_API_KEY", "sk-real")]);
    broker.virtualize_child_env(&mut env);
    let openai_api_key = env.get("OPENAI_API_KEY").expect("dummy OpenAI API key");
    let mut headers = HeaderMap::new();

    broker.inject_request_headers("api.openai.com", &mut headers);
    assert_eq!(authorization(&headers), None);

    headers = headers_with_bearer(openai_api_key);
    broker.inject_request_headers("api.openai.com", &mut headers);
    assert_eq!(authorization(&headers), Some("Bearer sk-real"));

    let mut explicit_headers = headers_with_bearer("sk-explicit");
    broker.inject_request_headers("api.openai.com", &mut explicit_headers);

    assert_eq!(authorization(&explicit_headers), Some("Bearer sk-explicit"));
}

#[test]
fn github_cloud_credentials_match_ghe_com_host_hint() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut env = env_map([("GH_HOST", "astemu.ghe.com"), ("GH_TOKEN", "ghp-real")]);
    broker.virtualize_child_env(&mut env);
    let github_token = env.get("GH_TOKEN").expect("dummy GitHub token");
    let mut headers = headers_with_bearer(github_token);

    broker.inject_request_headers("api.astemu.ghe.com", &mut headers);

    assert_eq!(authorization(&headers), Some("Bearer ghp-real"));
}

#[test]
fn github_cloud_credentials_do_not_bind_to_ghes_host_hint() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut env = env_map([("GH_HOST", "github.example.com"), ("GH_TOKEN", "ghp-real")]);
    broker.virtualize_child_env(&mut env);
    let github_token = env.get("GH_TOKEN").expect("dummy github token");
    let expected_authorization = format!("Bearer {github_token}");
    let mut headers = headers_with_bearer(github_token);

    broker.inject_request_headers("github.example.com", &mut headers);

    assert_eq!(
        authorization(&headers),
        Some(expected_authorization.as_str())
    );
    assert!(!broker.host_requires_mitm("github.example.com"));
    assert!(broker.host_requires_mitm("api.github.com"));
}

#[test]
fn github_enterprise_credentials_bind_to_gh_host() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let mut env = env_map([
        ("GH_HOST", "github.example.com"),
        ("GH_ENTERPRISE_TOKEN", "ghp-enterprise-real"),
    ]);
    broker.virtualize_child_env(&mut env);
    let github_token = env
        .get("GH_ENTERPRISE_TOKEN")
        .expect("dummy GitHub enterprise token");
    let mut headers = headers_with_bearer(github_token);

    broker.inject_request_headers("github.example.com", &mut headers);

    assert_eq!(authorization(&headers), Some("Bearer ghp-enterprise-real"));
    assert!(broker.host_requires_mitm("github.example.com"));
    assert!(!broker.host_requires_mitm("api.github.com"));
}

const SCOPED_SECRET: &str = "sk-scoped-canary-never-retained";

#[derive(Clone, Debug, PartialEq, Eq)]
struct UseSnapshot {
    scheme: String,
    host: String,
    port: u16,
    method: String,
    path: String,
    capability_id: String,
    actors: Vec<String>,
    session_id: String,
    task_id: String,
}

#[derive(Clone, Copy)]
enum ResolverOutcome {
    Success,
    Expired,
    Revoked,
}

struct TestResolver {
    outcome: ResolverOutcome,
    requests: Mutex<Vec<UseSnapshot>>,
}

impl TestResolver {
    fn new(outcome: ResolverOutcome) -> Self {
        Self {
            outcome,
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<UseSnapshot> {
        self.requests.lock().expect("requests lock").clone()
    }
}

impl ScopedCredentialResolver for TestResolver {
    fn resolve(
        &self,
        request: &ScopedCredentialUse<'_>,
        callback: &mut dyn FnMut(&str) -> Result<(), ScopedCredentialCallbackError>,
    ) -> Result<(), ScopedCredentialResolverError> {
        self.requests
            .lock()
            .expect("requests lock")
            .push(UseSnapshot {
                scheme: request.scheme.to_string(),
                host: request.host.to_string(),
                port: request.port,
                method: request.method.to_string(),
                path: request.path.to_string(),
                capability_id: request.capability_id.as_str().to_string(),
                actors: request
                    .authority
                    .actor_chain()
                    .as_slice()
                    .iter()
                    .map(|actor| actor.id.as_str().to_string())
                    .collect(),
                session_id: request
                    .authority
                    .authorization
                    .context
                    .session_id
                    .as_str()
                    .to_string(),
                task_id: request
                    .authority
                    .authorization
                    .context
                    .task_id
                    .as_str()
                    .to_string(),
            });
        match self.outcome {
            ResolverOutcome::Success => {
                callback(SCOPED_SECRET).map_err(|ScopedCredentialCallbackError::Failed| {
                    ScopedCredentialResolverError::CallbackFailed
                })
            }
            ResolverOutcome::Expired => Err(ScopedCredentialResolverError::Expired),
            ResolverOutcome::Revoked => Err(ScopedCredentialResolverError::Revoked),
        }
    }
}

fn bounded(value: &str) -> BoundedText {
    BoundedText::new(value).expect("bounded text")
}

fn scoped_authority() -> CredentialCapabilityRequest {
    let human = PolicyPrincipal::new(PrincipalKind::Human, "human:owner").expect("human principal");
    let actors = ActorChain::new(vec![
        human.clone(),
        PolicyPrincipal::new(PrincipalKind::Agent, "agent:root").expect("agent principal"),
    ])
    .expect("actor chain");
    let revocations = RevocationState::new();
    let destination = CredentialDestination::https("api.openai.com", 443).expect("destination");
    let credential =
        CredentialReference::new("provider.openai", "responses.create").expect("credential");
    let authorization = AuthorizationRequest::new(
        actors.clone(),
        ProtectedResource::new(ResourceKind::VaultCredential, "provider.openai").expect("resource"),
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: bounded("session:scoped-proxy"),
            task_id: bounded("task:scoped-proxy"),
            purpose: bounded("model-inference"),
            operation: credential.scope.clone(),
            destination: Some(destination.authority().expect("authority")),
            quantity: None,
            grant_id: None,
        },
    )
    .expect("authorization");
    let grant = BoundedGrant::issue(
        human,
        actors,
        GrantScope::new(
            authorization.resource.clone(),
            [PolicyAction::Use],
            GrantContext::new(
                authorization.context.session_id.clone(),
                authorization.context.task_id.clone(),
                authorization.context.purpose.clone(),
                authorization.context.operation.clone(),
            ),
            authorization.context.destination.clone(),
            BTreeMap::new(),
        )
        .expect("grant scope"),
        90,
        200,
        bounded("scoped-proxy-grant"),
    )
    .expect("grant");
    CredentialCapabilityRequest::new(
        authorization,
        grant,
        credential,
        CredentialHttpMethod::Post,
        destination,
        "/v1/responses",
        100,
        180,
        &revocations,
        None,
    )
    .expect("credential authority")
}

fn scoped_broker(
    outcome: ResolverOutcome,
) -> (CredentialBroker, HashMap<String, String>, Arc<TestResolver>) {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let resolver = Arc::new(TestResolver::new(outcome));
    let route = ScopedCredentialRoute::new(
        CapabilityId::from_sha256_hex("d".repeat(64)).expect("capability id"),
        scoped_authority(),
        resolver.clone(),
    )
    .expect("scoped route");
    broker
        .install_scoped_openai_route(route)
        .expect("install scoped route");
    let mut env = env_map([
        ("OPENAI_API_KEY", "sk-raw-value-must-not-be-retained"),
        ("GH_TOKEN", "ghp-permissive-still-works"),
    ]);
    broker.virtualize_child_env(&mut env);
    (broker, env, resolver)
}

#[test]
fn scoped_openai_route_injects_once_and_passes_complete_context() {
    let (broker, env, resolver) = scoped_broker(ResolverOutcome::Success);
    let dummy = env.get("OPENAI_API_KEY").expect("scoped dummy").to_string();
    assert_ne!(dummy, "sk-raw-value-must-not-be-retained");
    assert!(broker.host_requires_mitm("api.openai.com"));

    let mut headers = headers_with_bearer(&dummy);
    broker
        .inject_request_headers_for_request(
            "https",
            "API.OPENAI.COM",
            443,
            "POST",
            "/v1/responses",
            &mut headers,
        )
        .expect("scoped injection");
    assert_eq!(
        authorization(&headers),
        Some("Bearer sk-scoped-canary-never-retained")
    );
    assert!(
        headers
            .get(AUTHORIZATION)
            .expect("injected header")
            .is_sensitive()
    );
    assert!(!format!("{headers:?}").contains("sk-scoped-canary-never-retained"));
    assert_eq!(
        resolver.requests(),
        vec![UseSnapshot {
            scheme: "https".to_string(),
            host: "api.openai.com".to_string(),
            port: 443,
            method: "POST".to_string(),
            path: "/v1/responses".to_string(),
            capability_id: "d".repeat(64),
            actors: vec!["human:owner".to_string(), "agent:root".to_string()],
            session_id: "session:scoped-proxy".to_string(),
            task_id: "task:scoped-proxy".to_string(),
        }]
    );

    headers = headers_with_bearer(&dummy);
    assert_eq!(
        broker
            .inject_request_headers_for_request(
                "https",
                "api.openai.com",
                443,
                "POST",
                "/v1/responses",
                &mut headers,
            )
            .expect_err("redirect or retry reuse must fail"),
        ScopedCredentialInjectionError::AlreadyUsed
    );
}

#[test]
fn scoped_openai_denial_matrix_fails_before_resolution() {
    macro_rules! assert_denied {
        ($scheme:expr, $host:expr, $port:expr, $method:expr, $path:expr, $authorization:expr, $expected:expr) => {{
            let (broker, env, resolver) = scoped_broker(ResolverOutcome::Success);
            let dummy = env.get("OPENAI_API_KEY").expect("dummy");
            let mut headers = match $authorization {
                None => headers_with_bearer(dummy),
                Some("") => HeaderMap::new(),
                Some(value) => {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(value).expect("authorization"),
                    );
                    headers
                }
            };
            let error = broker
                .inject_request_headers_for_request(
                    $scheme,
                    $host,
                    $port,
                    $method,
                    $path,
                    &mut headers,
                )
                .expect_err("denial case");
            assert_eq!(error, $expected);
            assert!(resolver.requests().is_empty());
            assert!(!format!("{error:?} {error}").contains(SCOPED_SECRET));
        }};
    }

    assert_denied!(
        "http",
        "api.openai.com",
        443,
        "POST",
        "/v1/responses",
        None,
        ScopedCredentialInjectionError::SchemeDenied
    );
    for host in ["api.openai.com.evil.example", "sub.api.openai.com"] {
        assert_denied!(
            "https",
            host,
            443,
            "POST",
            "/v1/responses",
            None,
            ScopedCredentialInjectionError::HostDenied
        );
    }
    assert_denied!(
        "https",
        "api.openai.com",
        8443,
        "POST",
        "/v1/responses",
        None,
        ScopedCredentialInjectionError::PortDenied
    );
    assert_denied!(
        "https",
        "api.openai.com",
        443,
        "GET",
        "/v1/responses",
        None,
        ScopedCredentialInjectionError::MethodDenied
    );
    for path in ["/v1/chat/completions", "/v1/responses?redirect=true"] {
        assert_denied!(
            "https",
            "api.openai.com",
            443,
            "POST",
            path,
            None,
            ScopedCredentialInjectionError::PathDenied
        );
    }
    assert_denied!(
        "https",
        "api.openai.com",
        443,
        "POST",
        "/v1/responses",
        Some(""),
        ScopedCredentialInjectionError::MissingReference
    );
    assert_denied!(
        "https",
        "api.openai.com",
        443,
        "POST",
        "/v1/responses",
        Some("Bearer sk-explicit"),
        ScopedCredentialInjectionError::AuthorizationConflict
    );
}

#[test]
fn scoped_openai_stale_authority_and_unsupported_route_fail_closed() {
    for outcome in [ResolverOutcome::Expired, ResolverOutcome::Revoked] {
        let (broker, env, resolver) = scoped_broker(outcome);
        let mut headers = headers_with_bearer(env.get("OPENAI_API_KEY").expect("dummy"));
        assert_eq!(
            broker
                .inject_request_headers_for_request(
                    "https",
                    "api.openai.com",
                    443,
                    "POST",
                    "/v1/responses",
                    &mut headers,
                )
                .expect_err("stale route"),
            ScopedCredentialInjectionError::ResolutionFailed
        );
        assert_eq!(resolver.requests().len(), 1);
        assert!(!authorization(&headers).is_some_and(|value| value.contains(SCOPED_SECRET)));
    }

    let resolver = Arc::new(TestResolver::new(ResolverOutcome::Success));
    let mut wrong_host = scoped_authority();
    wrong_host.destination =
        CredentialDestination::https("adjacent.openai.com", 443).expect("adjacent destination");
    assert_eq!(
        ScopedCredentialRoute::new(
            CapabilityId::from_sha256_hex("e".repeat(64)).expect("capability id"),
            wrong_host,
            resolver,
        )
        .expect_err("invalid mutated host"),
        ScopedCredentialRouteError::InvalidAuthority
    );
}

#[derive(Default)]
struct IsolatedTestDispatcher {
    calls: Mutex<Vec<String>>,
}

impl IsolatedCredentialDispatcher for IsolatedTestDispatcher {
    fn dispatch(
        &self,
        request: &IsolatedCredentialUse<'_>,
    ) -> Result<IsolatedCredentialReceipt, IsolatedCredentialDispatchError> {
        self.calls
            .lock()
            .expect("calls")
            .push(request.path.to_string());
        Ok(IsolatedCredentialReceipt {
            response_status: 200,
            uploaded_bytes: 10,
            downloaded_bytes: 20,
        })
    }
}

#[test]
fn pf_27_s04_isolated_route_never_injects_raw_auth_and_supports_fresh_dispatches() {
    let broker = CredentialBroker::new(/*enabled*/ true);
    let dispatcher = Arc::new(IsolatedTestDispatcher::default());
    let route = IsolatedCredentialRoute::new(
        CapabilityId::from_sha256_hex("f".repeat(64)).expect("capability id"),
        scoped_authority(),
        dispatcher.clone(),
    )
    .expect("isolated route");
    broker
        .install_isolated_openai_route(route)
        .expect("install route");
    assert!(broker.scoped_openai_enabled());
    assert!(broker.scoped_openai_matches_host("API.OPENAI.COM"));
    assert!(!broker.scoped_openai_matches_host("attacker.example"));
    let mut env = env_map([("OPENAI_API_KEY", "sk-raw-must-not-enter-proxy")]);
    broker.virtualize_child_env(&mut env);
    let dummy = env.get("OPENAI_API_KEY").expect("dummy");
    assert_ne!(dummy, "sk-raw-must-not-enter-proxy");

    let mut headers = headers_with_bearer(dummy);
    assert_eq!(
        broker.inject_request_headers_for_request(
            "https",
            "api.openai.com",
            443,
            "POST",
            "/v1/responses",
            &mut headers,
        ),
        Err(ScopedCredentialInjectionError::IsolatedBrokerRequired)
    );
    assert!(!format!("{headers:?}").contains("sk-raw-must-not-enter-proxy"));

    for _ in 0..2 {
        assert_eq!(
            broker
                .dispatch_isolated_openai("https", "API.OPENAI.COM", 443, "POST", "/v1/responses",)
                .expect("broker receipt")
                .response_status,
            200
        );
    }
    assert_eq!(dispatcher.calls.lock().expect("calls").len(), 2);
    assert_eq!(
        broker.dispatch_isolated_openai(
            "https",
            "api.openai.com",
            443,
            "POST",
            "/v1/chat/completions",
        ),
        Err(IsolatedCredentialDispatchError::Denied)
    );
}
