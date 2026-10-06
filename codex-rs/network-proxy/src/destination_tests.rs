use super::*;
use pretty_assertions::assert_eq;
use rama_http::HeaderValue;
use std::collections::HashMap;
use std::sync::Mutex as StdMutex;

/// Synthetic answers only: no test contacts a real private endpoint.
#[derive(Default)]
struct FakeResolver {
    answers: HashMap<&'static str, Vec<&'static str>>,
    lookups: StdMutex<Vec<String>>,
}

impl FakeResolver {
    fn new(entries: &[(&'static str, &[&'static str])]) -> Self {
        Self {
            answers: entries
                .iter()
                .map(|(host, answers)| (*host, answers.to_vec()))
                .collect(),
            lookups: StdMutex::new(Vec::new()),
        }
    }

    fn lookups(&self) -> Vec<String> {
        self.lookups.lock().expect("lookups").clone()
    }
}

impl Resolve for FakeResolver {
    fn lookup<'a>(&'a self, host: &'a str, _port: u16) -> LookupFuture<'a> {
        self.lookups.lock().expect("lookups").push(host.to_owned());
        let answers = self.answers.get(host).cloned();
        Box::pin(async move {
            answers
                .map(|answers| {
                    answers
                        .iter()
                        .map(|answer| answer.parse().expect("fixture address"))
                        .collect()
                })
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "synthetic NXDOMAIN"))
        })
    }
}

const PUBLIC_V4: &str = "93.184.216.34";
const PUBLIC_V6: &str = "2606:2800:220:1:248:1893:25c8:1946";

fn resolver() -> FakeResolver {
    FakeResolver::new(&[
        ("public.example", &[PUBLIC_V4, PUBLIC_V6]),
        ("other.example", &[PUBLIC_V4]),
        ("rebind.example", &["127.0.0.1"]),
        ("mapped.example", &["::ffff:10.0.0.7"]),
        ("metadata.example", &["169.254.169.254"]),
        ("ula.example", &["fd00::1"]),
        ("cgnat.example", &["100.64.0.1"]),
        ("multicast.example", &["224.0.0.1"]),
        ("nat64.example", &["64:ff9b::a00:1"]),
        ("mixed.example", &[PUBLIC_V4, "10.0.0.5"]),
        ("mixed-v6.example", &[PUBLIC_V6, "fe80::1"]),
        ("empty.example", &[]),
        (
            "many.example",
            &[
                "1.1.1.1", "1.1.1.2", "1.1.1.3", "1.1.1.4", "1.1.1.5", "1.1.1.6", "1.1.1.7",
                "1.1.1.8", "1.1.1.9", "1.1.1.10", "1.1.1.11", "1.1.1.12", "1.1.1.13", "1.1.1.14",
                "1.1.1.15", "1.1.1.16", "1.1.1.17",
            ],
        ),
        ("svc.corp", &["10.1.2.3"]),
        ("svc-moved.corp", &["10.1.2.4"]),
    ])
}

fn denial_code<T: std::fmt::Debug>(result: Result<T, DestinationDenial>) -> &'static str {
    match result {
        Ok(value) => panic!("expected a denial, got {value:?}"),
        Err(denial) => denial.code(),
    }
}

async fn tunnel(host: &str, port: u16) -> Result<(), DestinationDenial> {
    DestinationGuard::protected()
        .authorize_tunnel(host, port, &resolver())
        .await
}

async fn url(url: &str, method: &str) -> Result<NormalizedDestination, DestinationDenial> {
    DestinationGuard::protected()
        .authorize_url(url, method, &resolver())
        .await
}

#[tokio::test]
async fn pf_33_s01_every_dns_answer_must_be_public() {
    assert_eq!(tunnel("public.example", 443).await, Ok(()));
    for (host, code) in [
        ("rebind.example", "private_destination"),
        ("mapped.example", "private_destination"),
        ("metadata.example", "private_destination"),
        ("ula.example", "private_destination"),
        ("cgnat.example", "private_destination"),
        ("multicast.example", "private_destination"),
        ("nat64.example", "private_destination"),
        ("mixed.example", "dns_mixed_answers"),
        ("mixed-v6.example", "dns_mixed_answers"),
        ("empty.example", "dns_no_answers"),
        ("nxdomain.example", "dns_failure"),
        ("many.example", "dns_answers_too_many"),
    ] {
        assert_eq!(denial_code(tunnel(host, 443).await), code, "{host}");
    }
}

#[tokio::test]
async fn pf_33_s01_literals_and_reserved_names_need_a_private_grant() {
    for host in [
        "127.0.0.1",
        "::1",
        "::ffff:127.0.0.1",
        "169.254.169.254",
        "10.0.0.1",
        "fe80::1",
    ] {
        assert_eq!(
            denial_code(tunnel(host, 443).await),
            "private_destination",
            "{host}"
        );
    }
    for host in ["localhost", "intranet", "printer.local", "db.internal"] {
        assert_eq!(
            denial_code(tunnel(host, 443).await),
            "private_destination",
            "{host}"
        );
    }
    assert_eq!(tunnel(PUBLIC_V4, 443).await, Ok(()));
}

#[tokio::test]
async fn pf_33_s01_scheme_port_and_method_are_authorized() {
    assert_eq!(
        denial_code(tunnel("public.example", 22).await),
        "scheme_port_or_method"
    );
    assert_eq!(
        denial_code(url("http://public.example/", "GET").await),
        "scheme_port_or_method"
    );
    assert_eq!(
        denial_code(url("https://public.example:8443/", "GET").await),
        "scheme_port_or_method"
    );
    assert_eq!(
        denial_code(url("https://public.example/", "TRACE").await),
        "scheme_port_or_method"
    );
    assert_eq!(
        denial_code(url("https://public.example/", "get").await),
        "method"
    );
    assert!(url("https://public.example/a?b=c", "POST").await.is_ok());
}

#[tokio::test]
async fn pf_33_s01_urls_are_canonicalized_and_ambiguous_forms_rejected() {
    for (input, code) in [
        ("https://user:secret@public.example/", "url_userinfo"),
        ("https://user@public.example/", "url_userinfo"),
        ("https://public.example/#fragment", "url_ambiguous"),
        ("https://public.example:/", "url_ambiguous"),
        ("https:\\\\public.example/", "url_ambiguous"),
        (" https://public.example/", "url_ambiguous"),
        ("ftp://public.example/", "url_scheme"),
        // Unusual IPv4 encodings canonicalize to the loopback they name.
        ("https://2130706433/", "private_destination"),
        ("https://0x7f.1/", "private_destination"),
        ("https://0177.0.0.1/", "private_destination"),
        ("https://[::ffff:7f00:1]/", "private_destination"),
    ] {
        assert_eq!(denial_code(url(input, "GET").await), code, "{input}");
    }
    let normalized = url("https://PUBLIC.Example./a/../b", "GET")
        .await
        .expect("trailing dot and case normalize");
    assert_eq!(
        (normalized.host(), normalized.port(), normalized.path()),
        ("public.example", 443, "/b")
    );
}

#[tokio::test]
async fn pf_33_s01_idna_hosts_resolve_in_ascii_form() {
    let resolver = resolver();
    let result = DestinationGuard::protected()
        .authorize_url("https://bücher.example/", "GET", &resolver)
        .await;
    assert_eq!(denial_code(result), "dns_failure");
    assert_eq!(
        resolver.lookups(),
        vec!["xn--bcher-kva.example".to_string()]
    );
}

#[tokio::test]
async fn pf_33_s01_suffix_confusion_does_not_reach_lookup_of_another_name() {
    // `public.example.evil` is its own name with its own (absent) answers.
    let resolver = resolver();
    let result = DestinationGuard::protected()
        .authorize_url("https://public.example.evil/", "GET", &resolver)
        .await;
    assert_eq!(denial_code(result), "dns_failure");
    assert_eq!(resolver.lookups(), vec!["public.example.evil".to_string()]);
}

const CLIENT: &str = "exec-1|127.0.0.1";

fn hosts() -> HostPatterns {
    let allowed = [
        "public.example",
        "other.example",
        "*.example",
        "127.0.0.1",
        "169.254.169.254",
        "::ffff:169.254.169.254",
    ]
    .map(String::from);
    HostPatterns::compile(&allowed, &["denied.example".to_string()]).expect("patterns")
}

async fn start(
    ledger: &RedirectLedger,
    target: &str,
    method: &str,
) -> Result<AuthorizedRequest, DestinationDenial> {
    DestinationGuard::protected()
        .authorize_request(target, method, CLIENT, ledger, &resolver())
        .await
}

async fn redirect(
    ledger: &RedirectLedger,
    request: &AuthorizedRequest,
    status: u16,
    location: &str,
    has_body: bool,
) -> Result<String, DestinationDenial> {
    DestinationGuard::protected()
        .authorize_redirect(
            request,
            has_body,
            status,
            location,
            RedirectScope {
                hosts: &hosts(),
                ledger,
                resolver: &resolver(),
            },
        )
        .await
}

fn stripped(request: &AuthorizedRequest) -> bool {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        "Bearer fake-key-0001".parse().expect("header"),
    );
    request.strip_cross_origin_credentials(&mut headers);
    !headers.contains_key(header::AUTHORIZATION)
}

#[tokio::test]
async fn pf_33_s01_every_redirect_hop_is_reauthorized() {
    let ledger = RedirectLedger::default();
    let first = start(&ledger, "https://public.example/start", "GET")
        .await
        .expect("first hop");
    assert_eq!(
        redirect(&ledger, &first, 302, "/next?x=1", false).await,
        Ok("https://public.example/next?x=1".to_string())
    );
    for (location, code) in [
        ("http://public.example/", "redirect_downgrade"),
        ("https://rebind.example/", "private_destination"),
        ("https://127.0.0.1/", "private_destination"),
        ("https://[::ffff:a9fe:a9fe]/", "private_destination"),
        ("https://mixed.example/", "dns_mixed_answers"),
        ("https://nxdomain.example/", "dns_failure"),
        ("https://user:pw@public.example/", "url_userinfo"),
        ("https://public.example:8443/", "scheme_port_or_method"),
        ("https://unlisted.test/", "redirect_host_not_allowed"),
        ("https://denied.example/", "redirect_host_not_allowed"),
        ("https:\\\\public.example/", "redirect_location"),
        (" /next", "redirect_location"),
        ("/next\tx", "redirect_location"),
        ("", "redirect_location"),
    ] {
        assert_eq!(
            denial_code(redirect(&ledger, &first, 302, location, false).await),
            code,
            "{location}"
        );
    }
    assert_eq!(
        redirect(&ledger, &first, 302, "/next?x=1#section", false).await,
        Ok("https://public.example/next?x=1".to_string())
    );
    let second = start(&ledger, "https://public.example/next?x=1", "GET")
        .await
        .expect("followed hop");
    assert_eq!(second.chain.hops, 1);
}

#[tokio::test]
async fn pf_33_s01_redirect_method_and_body_replay() {
    let ledger = RedirectLedger::default();
    let post = start(&ledger, "https://public.example/form", "POST")
        .await
        .expect("post");
    assert!(redirect(&ledger, &post, 307, "/again", true).await.is_ok());
    assert!(redirect(&ledger, &post, 308, "/again", true).await.is_ok());
    assert!(redirect(&ledger, &post, 303, "/done", true).await.is_ok());
    assert_eq!(
        denial_code(redirect(&ledger, &post, 302, "/moved", true).await),
        "redirect_body"
    );
    assert_eq!(
        denial_code(redirect(&ledger, &post, 301, "/moved", false).await),
        "redirect_body"
    );
    let get = start(&ledger, "https://public.example/page", "GET")
        .await
        .expect("get");
    assert_eq!(
        denial_code(redirect(&ledger, &get, 300, "/choice", false).await),
        "redirect_status"
    );
    assert_eq!(
        denial_code(redirect(&ledger, &get, 302, "https://[bad/", false).await),
        "redirect_location"
    );
}

#[tokio::test]
async fn pf_33_s01_redirect_chains_have_a_hop_limit() {
    let ledger = RedirectLedger::default();
    let mut current = "https://public.example/0".to_string();
    for hop in 1..=MAX_REDIRECT_HOPS {
        let request = start(&ledger, &current, "GET").await.expect("hop");
        current = redirect(&ledger, &request, 302, &format!("/{hop}"), false)
            .await
            .expect("within the limit");
    }
    let last = start(&ledger, &current, "GET").await.expect("last hop");
    assert_eq!(last.chain.hops, MAX_REDIRECT_HOPS);
    assert_eq!(
        denial_code(redirect(&ledger, &last, 302, "/one-more", false).await),
        "redirect_hop_limit"
    );
}

#[tokio::test]
async fn pf_33_s01_redirect_chains_have_a_time_limit() {
    let ledger = RedirectLedger::default();
    let mut request = start(&ledger, "https://public.example/slow", "GET")
        .await
        .expect("request");
    request.chain.started = Instant::now()
        .checked_sub(MAX_CHAIN_AGE + Duration::from_secs(1))
        .expect("monotonic clock has run long enough");
    assert_eq!(
        denial_code(redirect(&ledger, &request, 302, "/later", false).await),
        "redirect_chain_expired"
    );
}

#[tokio::test]
async fn pf_33_s01_credentials_do_not_cross_origins() {
    let ledger = RedirectLedger::default();
    let first = start(&ledger, "https://public.example/login", "GET")
        .await
        .expect("first");
    assert!(!stripped(&first));
    let target = redirect(&ledger, &first, 302, "https://other.example/cdn", false)
        .await
        .expect("cross-origin redirect is allowed without credentials");
    let second = start(&ledger, &target, "GET").await.expect("second");
    assert!(stripped(&second));
    // Returning to the chain origin restores normal credential handling.
    let target = redirect(&ledger, &second, 302, "https://public.example/home", false)
        .await
        .expect("back to origin");
    let third = start(&ledger, &target, "GET").await.expect("third");
    assert!(!stripped(&third));
    assert_eq!(third.chain.hops, 2);
}

#[tokio::test]
async fn pf_33_s01_private_networks_need_an_exact_service_grant() {
    let guard = DestinationGuard::with_private_services(vec![PrivateServiceSpec {
        host: "svc.corp".to_string(),
        port: 443,
        methods: vec!["GET".to_string()],
        path_prefixes: vec!["/api/".to_string()],
        approved_addresses: vec!["10.1.2.3".to_string()],
    }])
    .expect("grant compiles");
    let resolver = resolver();
    assert!(
        guard
            .authorize_url("https://svc.corp/api/v1", "GET", &resolver)
            .await
            .is_ok()
    );
    for (input, method, code) in [
        ("https://svc.corp/admin", "GET", "private_destination"),
        ("https://svc.corp/api/v1", "POST", "private_destination"),
        (
            "https://svc-moved.corp/api/v1",
            "GET",
            "private_destination",
        ),
    ] {
        assert_eq!(
            denial_code(guard.authorize_url(input, method, &resolver).await),
            code,
            "{input} {method}"
        );
    }
    // The protected default has no grants at all.
    assert_eq!(
        denial_code(url("https://svc.corp/api/v1", "GET").await),
        "private_destination"
    );
}

fn position(started: Instant) -> ChainPosition {
    ChainPosition {
        hops: 1,
        started,
        origin: Origin {
            scheme: "https".to_string(),
            host: "public.example".to_string(),
            port: 443,
        },
        strip_credentials: false,
    }
}

#[test]
fn pf_33_s01_ledger_is_scoped_bounded_and_keeps_expired_markers() {
    let ledger = RedirectLedger::default();
    let now = Instant::now();
    for index in 0..=LEDGER_CLIENT_CAPACITY {
        ledger.record(
            CLIENT,
            format!("https://public.example:443/{index}"),
            position(now),
        );
    }
    // One client cannot hold more than its share; its oldest entry goes first.
    assert!(matches!(
        ledger.take(CLIENT, "https://public.example:443/0", now),
        LedgerMatch::Fresh
    ));
    // Another client never inherits this client's chain.
    assert!(matches!(
        ledger.take("exec-2|127.0.0.1", "https://public.example:443/1", now),
        LedgerMatch::Fresh
    ));
    assert!(matches!(
        ledger.take(CLIENT, "https://public.example:443/1", now),
        LedgerMatch::Continued(_)
    ));
    // Past the chain age the marker refuses the follow-up instead of starting fresh.
    let later = now + MAX_CHAIN_AGE + Duration::from_secs(1);
    for _ in 0..2 {
        assert!(matches!(
            ledger.take(CLIENT, "https://public.example:443/2", later),
            LedgerMatch::Expired
        ));
    }
    // Markers are dropped after twice the chain age.
    let much_later = now + MAX_CHAIN_AGE * 2 + Duration::from_secs(1);
    assert!(matches!(
        ledger.take(CLIENT, "https://public.example:443/3", much_later),
        LedgerMatch::Fresh
    ));
    // The global bound holds across clients.
    for index in 0..=LEDGER_CAPACITY {
        ledger.record(
            &format!("exec-{index}|127.0.0.1"),
            "https://public.example:443/x".to_string(),
            position(now),
        );
    }
    assert_eq!(
        ledger.pending.lock().expect("ledger").len(),
        LEDGER_CAPACITY
    );
}

#[tokio::test]
async fn pf_33_s01_late_follow_up_of_an_expired_chain_is_refused() {
    let ledger = RedirectLedger::default();
    let started = Instant::now()
        .checked_sub(MAX_CHAIN_AGE + Duration::from_secs(1))
        .expect("monotonic clock has run long enough");
    ledger.record(
        CLIENT,
        "https://public.example:443/late".to_string(),
        position(started),
    );
    assert_eq!(
        denial_code(start(&ledger, "https://public.example/late", "GET").await),
        "redirect_chain_expired"
    );
}

#[tokio::test]
async fn pf_33_s01_unlisted_redirect_targets_never_reach_the_resolver() {
    let ledger = RedirectLedger::default();
    let resolver = resolver();
    let guard = DestinationGuard::protected();
    let first = guard
        .authorize_request(
            "https://public.example/r",
            "GET",
            CLIENT,
            &ledger,
            &resolver,
        )
        .await
        .expect("first");
    for location in ["https://denied.example/", "https://secret-data.exfil.test/"] {
        let result = guard
            .authorize_redirect(
                &first,
                false,
                302,
                location,
                RedirectScope {
                    hosts: &hosts(),
                    ledger: &ledger,
                    resolver: &resolver,
                },
            )
            .await;
        assert_eq!(denial_code(result), "redirect_host_not_allowed");
    }
    assert_eq!(resolver.lookups(), vec!["public.example".to_string()]);
}

#[tokio::test]
async fn pf_33_s01_response_check_rewrites_or_refuses_location() {
    let ledger = RedirectLedger::default();
    let first = start(&ledger, "https://public.example/a/b", "GET")
        .await
        .expect("first");
    let check = |status: u16, headers: HeaderMap| {
        let first = &first;
        let ledger = &ledger;
        async move {
            let mut headers = headers;
            let result = DestinationGuard::protected()
                .check_response(
                    first,
                    false,
                    status,
                    &mut headers,
                    RedirectScope {
                        hosts: &hosts(),
                        ledger,
                        resolver: &resolver(),
                    },
                )
                .await;
            (result, headers)
        }
    };
    let location = |values: &[&[u8]]| {
        let mut headers = HeaderMap::new();
        for value in values {
            headers.append(
                header::LOCATION,
                HeaderValue::from_bytes(value).expect("header bytes"),
            );
        }
        headers
    };
    // Relative targets are relayed as the exact absolute URL that was checked.
    let (result, headers) = check(302, location(&[b"../c?x=1#frag"])).await;
    assert_eq!(result, Ok(()));
    assert_eq!(
        headers.get(header::LOCATION).expect("location"),
        "https://public.example/c?x=1"
    );
    // Responses without a redirect are untouched, even with a Location.
    let (result, headers) = check(201, location(&[b"http://127.0.0.1/"])).await;
    assert_eq!(result, Ok(()));
    assert_eq!(
        headers.get(header::LOCATION).expect("location"),
        "http://127.0.0.1/"
    );
    assert_eq!(check(304, HeaderMap::new()).await.0, Ok(()));
    // Ambiguous headers are refused.
    for headers in [location(&[b"/one", b"/two"]), location(&[b"/caf\xe9"])] {
        assert_eq!(
            check(302, headers).await.0.map_err(|denial| denial.code()),
            Err("redirect_location")
        );
    }
    assert_eq!(
        check(302, location(&[b"https://127.0.0.1/"]))
            .await
            .0
            .map_err(|denial| denial.code()),
        Err("private_destination")
    );
}

#[test]
fn pf_33_s01_request_body_detection() {
    let mut headers = HeaderMap::new();
    assert!(!request_has_body(&headers));
    headers.insert(header::CONTENT_LENGTH, "0".parse().expect("header"));
    assert!(!request_has_body(&headers));
    headers.insert(header::CONTENT_LENGTH, "12".parse().expect("header"));
    assert!(request_has_body(&headers));
    let mut headers = HeaderMap::new();
    headers.insert(
        header::TRANSFER_ENCODING,
        "chunked".parse().expect("header"),
    );
    assert!(request_has_body(&headers));
}

#[test]
fn pf_33_s01_blocked_response_names_only_the_reason() {
    let response = blocked_response("redirect_downgrade");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response
            .headers()
            .get("x-proxy-error")
            .and_then(|value| value.to_str().ok()),
        Some(BLOCKED_HEADER)
    );
}
