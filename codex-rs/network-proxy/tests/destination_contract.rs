#[path = "../src/destination_contract.rs"]
mod destination_contract;

use destination_contract::BodyReplay;
use destination_contract::ContractError;
use destination_contract::CredentialReplay;
use destination_contract::DESTINATION_CONTRACT_VERSION;
use destination_contract::DecisionReason;
use destination_contract::DestinationPolicy;
use destination_contract::DnsAnswerSet;
use destination_contract::PolicySpec;
use destination_contract::PrivateServiceSpec;
use destination_contract::PublicScope;
use destination_contract::RequestInput;
use destination_contract::RuleSpec;
use destination_contract::evaluate_destination;
use destination_contract::evaluate_redirect;
use destination_contract::normalize_destination;
use sha2::Digest;
use sha2::Sha256;

const FROZEN_FIXTURE_SHA256: &str =
    "1b05284a2c173bb4436f9eae913e0d47cd2a11a6df4ee7e5d5b9e7fa93d2eb1a";

fn request(url: &str, method: &str) -> destination_contract::NormalizedDestination {
    match normalize_destination(RequestInput { url, method }) {
        Ok(destination) => destination,
        Err(error) => panic!("request should normalize: {error}"),
    }
}

fn compile(
    public_scope: PublicScope,
    private_services: Vec<PrivateServiceSpec>,
) -> DestinationPolicy {
    match DestinationPolicy::compile(PolicySpec {
        version: DESTINATION_CONTRACT_VERSION.to_owned(),
        public_scope,
        private_services,
    }) {
        Ok(policy) => policy,
        Err(error) => panic!("policy should compile: {error}"),
    }
}

fn rule(host: &str, methods: &[&str], paths: &[&str]) -> RuleSpec {
    RuleSpec {
        host: host.to_owned(),
        schemes: vec!["https".to_owned()],
        ports: vec![443],
        methods: methods.iter().map(|value| (*value).to_owned()).collect(),
        path_prefixes: paths.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn private_service(addresses: &[&str]) -> PrivateServiceSpec {
    PrivateServiceSpec {
        host: "internal.example.test".to_owned(),
        port: 443,
        methods: vec!["GET".to_owned(), "POST".to_owned()],
        path_prefixes: vec!["/api".to_owned()],
        approved_addresses: addresses.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn strings(value: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(items) = value[key].as_array() else {
        panic!("fixture field {key:?} must be an array");
    };
    items
        .iter()
        .map(|item| match item.as_str() {
            Some(value) => value.to_owned(),
            None => panic!("fixture field {key:?} must contain strings"),
        })
        .collect()
}

fn fixture_string<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    match value[key].as_str() {
        Some(value) => value,
        None => panic!("fixture field {key:?} must be a string"),
    }
}

fn fixture_u16(value: &serde_json::Value, key: &str) -> u16 {
    fixture_u16_value(&value[key], key)
}

fn fixture_u16_value(value: &serde_json::Value, label: &str) -> u16 {
    match value.as_u64().and_then(|value| u16::try_from(value).ok()) {
        Some(value) => value,
        None => panic!("fixture field {label:?} must be a u16"),
    }
}

fn fixture_policy(case: &serde_json::Value) -> DestinationPolicy {
    let scope = case
        .get("public_scope")
        .unwrap_or_else(|| panic!("fixture must name public_scope explicitly"));
    let public_scope = match fixture_string(scope, "kind") {
        "unrestricted" => PublicScope::Unrestricted,
        "rules" => {
            let Some(rules) = scope["rules"].as_array() else {
                panic!("rules public_scope must contain a rules array");
            };
            PublicScope::Rules(
                rules
                    .iter()
                    .map(|rule| RuleSpec {
                        host: fixture_string(rule, "host").to_owned(),
                        schemes: strings(rule, "schemes"),
                        ports: {
                            let Some(ports) = rule["ports"].as_array() else {
                                panic!("fixture rule ports must be an array");
                            };
                            ports
                                .iter()
                                .map(|port| fixture_u16_value(port, "port"))
                                .collect()
                        },
                        methods: strings(rule, "methods"),
                        path_prefixes: strings(rule, "path_prefixes"),
                    })
                    .collect(),
            )
        }
        kind => panic!("unknown public_scope kind: {kind}"),
    };
    let private_services = case
        .get("private_service")
        .map(|service| {
            vec![PrivateServiceSpec {
                host: fixture_string(service, "host").to_owned(),
                port: fixture_u16(service, "port"),
                methods: strings(service, "methods"),
                path_prefixes: strings(service, "path_prefixes"),
                approved_addresses: strings(service, "approved_addresses"),
            }]
        })
        .unwrap_or_default();
    compile(public_scope, private_services)
}

fn fixture_answers(case: &serde_json::Value) -> DnsAnswerSet {
    let owned = strings(case, "dns_answers");
    let borrowed: Vec<&str> = owned.iter().map(String::as_str).collect();
    match DnsAnswerSet::parse(&borrowed) {
        Ok(answers) => answers,
        Err(error) => panic!("fixture DNS answers must compile: {error}"),
    }
}

#[test]
fn normalizes_idna_case_trailing_dot_default_port_and_path() {
    let normalized = request("HTTPS://BÜCHER.Example.:443/a/../v1/items?x=1", "GET");
    assert_eq!(normalized.scheme(), "https");
    assert_eq!(normalized.host(), "xn--bcher-kva.example");
    assert_eq!(normalized.port(), 443);
    assert_eq!(normalized.method(), "GET");
    assert_eq!(normalized.path(), "/v1/items");
}

#[test]
fn rejects_userinfo_fragments_backslashes_and_non_http_schemes() {
    assert_eq!(
        normalize_destination(RequestInput {
            url: "https://user@example.com/",
            method: "GET",
        }),
        Err(ContractError::UserinfoForbidden)
    );
    assert_eq!(
        normalize_destination(RequestInput {
            url: "https://example.com/#fragment",
            method: "GET",
        }),
        Err(ContractError::FragmentForbidden)
    );
    assert_eq!(
        normalize_destination(RequestInput {
            url: "https:\\example.com\\admin",
            method: "GET",
        }),
        Err(ContractError::AmbiguousUrl)
    );
    assert!(matches!(
        normalize_destination(RequestInput {
            url: "file:///etc/passwd",
            method: "GET",
        }),
        Err(ContractError::UnsupportedScheme(_))
    ));
    for url in [" https://example.com/", "https://example.com/ "] {
        assert_eq!(
            normalize_destination(RequestInput { url, method: "GET" }),
            Err(ContractError::AmbiguousUrl)
        );
    }
    for url in [
        "https://example.com../",
        "https://example.com:/",
        "https://-bad.example/",
    ] {
        assert!(
            normalize_destination(RequestInput { url, method: "GET" }).is_err(),
            "{url}"
        );
    }
    let long_label = format!("https://{}.example/", "a".repeat(64));
    assert!(
        normalize_destination(RequestInput {
            url: &long_label,
            method: "GET"
        })
        .is_err()
    );
    let oversized = format!("https://example.com/{}", "a".repeat(4096));
    assert_eq!(
        normalize_destination(RequestInput {
            url: &oversized,
            method: "GET",
        }),
        Err(ContractError::AmbiguousUrl)
    );
}

#[test]
fn canonicalizes_unusual_ipv4_and_mapped_ipv6() {
    assert_eq!(request("http://0177.0.0.1/", "GET").host(), "127.0.0.1");
    assert_eq!(request("http://2130706433/", "GET").host(), "127.0.0.1");
    let mapped = DnsAnswerSet::parse(&["::ffff:127.0.0.1"]).unwrap();
    assert!(mapped.addresses().contains(&"127.0.0.1".parse().unwrap()));
}

#[test]
fn unrestricted_empty_and_wildcard_public_polarities_are_distinct() {
    let destination = request("https://public.example/v1", "GET");
    let public_answers = DnsAnswerSet::parse(&["93.184.216.34"]).unwrap();

    let unrestricted = evaluate_destination(
        &compile(PublicScope::Unrestricted, vec![]),
        &destination,
        &public_answers,
    );
    assert_eq!(unrestricted.reason(), DecisionReason::PublicPolicyAbsent);
    assert!(unrestricted.allowed());

    let empty = evaluate_destination(
        &compile(PublicScope::Rules(vec![]), vec![]),
        &destination,
        &public_answers,
    );
    assert_eq!(empty.reason(), DecisionReason::ExplicitDenyAll);
    assert!(!empty.allowed());

    let wildcard = evaluate_destination(
        &compile(
            PublicScope::Rules(vec![rule("*", &["GET"], &["/"])]),
            vec![],
        ),
        &destination,
        &public_answers,
    );
    assert_eq!(wildcard.reason(), DecisionReason::WildcardPublicRule);
    assert!(wildcard.allowed());

    let private_answers = DnsAnswerSet::parse(&["10.0.0.8"]).unwrap();
    let denied_private = evaluate_destination(
        &compile(
            PublicScope::Rules(vec![rule("*", &["GET"], &["/"])]),
            vec![],
        ),
        &destination,
        &private_answers,
    );
    assert_eq!(
        denied_private.reason(),
        DecisionReason::PrivateDestinationNotAuthorized
    );
    assert!(!denied_private.allowed());
}

#[test]
fn exact_and_suffix_rules_resist_host_and_path_confusion() {
    let answers = DnsAnswerSet::parse(&["93.184.216.34"]).unwrap();
    let exact = compile(
        PublicScope::Rules(vec![rule("api.example.com.", &["GET"], &["/v1"])]),
        vec![],
    );
    assert!(
        evaluate_destination(
            &exact,
            &request("https://api.example.com./v1/items", "GET"),
            &answers,
        )
        .allowed()
    );
    for url in [
        "https://api.example.com.evil.test/v1/items",
        "https://api.example.com/v10/items",
    ] {
        assert!(!evaluate_destination(&exact, &request(url, "GET"), &answers).allowed());
    }

    let suffix = compile(
        PublicScope::Rules(vec![rule("*.example.com", &["GET"], &["/"])]),
        vec![],
    );
    assert!(
        evaluate_destination(
            &suffix,
            &request("https://child.example.com/", "GET"),
            &answers,
        )
        .allowed()
    );
    for url in [
        "https://example.com/",
        "https://badexample.com/",
        "https://example.com.evil.test/",
    ] {
        assert!(!evaluate_destination(&suffix, &request(url, "GET"), &answers).allowed());
    }
}

#[test]
fn idna_policy_and_request_compare_in_one_ascii_form() {
    let policy = compile(
        PublicScope::Rules(vec![rule("bücher.example", &["GET"], &["/"])]),
        vec![],
    );
    let answers = DnsAnswerSet::parse(&["93.184.216.34"]).unwrap();
    let decision = evaluate_destination(
        &policy,
        &request("https://xn--bcher-kva.example/", "GET"),
        &answers,
    );
    assert_eq!(decision.reason(), DecisionReason::ExplicitPublicRule);
    assert!(decision.allowed());
}

#[test]
fn dns_answer_sets_fail_closed_for_empty_mixed_private_and_literal_mismatch() {
    let policy = compile(PublicScope::Unrestricted, vec![]);
    let destination = request("https://public.example/", "GET");
    assert_eq!(
        evaluate_destination(&policy, &destination, &DnsAnswerSet::parse(&[]).unwrap()).reason(),
        DecisionReason::EmptyDnsAnswerSet
    );
    assert_eq!(
        evaluate_destination(
            &policy,
            &destination,
            &DnsAnswerSet::parse(&["93.184.216.34", "127.0.0.1"]).unwrap(),
        )
        .reason(),
        DecisionReason::MixedPublicAndPrivateAnswers
    );
    assert_eq!(
        evaluate_destination(
            &policy,
            &request("https://93.184.216.34/", "GET"),
            &DnsAnswerSet::parse(&["93.184.216.35"]).unwrap(),
        )
        .reason(),
        DecisionReason::AddressLiteralMismatch
    );
}

#[test]
fn reserved_and_local_address_table_is_never_public() {
    let policy = compile(PublicScope::Unrestricted, vec![]);
    let destination = request("https://public.example/", "GET");
    for address in [
        "0.0.0.0",
        "10.0.0.1",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.1.1",
        "172.16.0.1",
        "192.0.2.1",
        "192.168.1.1",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "255.255.255.255",
        "::",
        "::1",
        "fc00::1",
        "fe80::1",
        "fec0::1",
        "ff02::1",
        "2001:db8::1",
        "64:ff9b::7f00:1",
        "2001::1",
        "2002:7f00:1::",
        "3fff::1",
        "::ffff:192.168.1.1",
    ] {
        let decision = evaluate_destination(
            &policy,
            &destination,
            &DnsAnswerSet::parse(&[address]).unwrap(),
        );
        assert!(!decision.allowed(), "{address} was treated as public");
    }
    for address in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
        assert!(
            evaluate_destination(
                &policy,
                &destination,
                &DnsAnswerSet::parse(&[address]).unwrap(),
            )
            .allowed(),
            "{address} was not treated as public"
        );
    }
}

#[test]
fn private_service_requires_exact_https_identity_and_approved_addresses() {
    let policy = compile(
        PublicScope::Unrestricted,
        vec![private_service(&["10.0.0.8", "10.0.0.9"])],
    );
    let target = request("https://internal.example.test/api/items", "GET");
    let allowed = evaluate_destination(
        &policy,
        &target,
        &DnsAnswerSet::parse(&["10.0.0.8", "10.0.0.9"]).unwrap(),
    );
    assert_eq!(allowed.reason(), DecisionReason::ExplicitPrivateService);
    assert!(allowed.allowed());

    let changed = evaluate_destination(
        &policy,
        &target,
        &DnsAnswerSet::parse(&["10.0.0.8", "10.0.0.10"]).unwrap(),
    );
    assert_eq!(changed.reason(), DecisionReason::PrivateAddressSetMismatch);
    assert!(!changed.allowed());

    let rebound_public = evaluate_destination(
        &policy,
        &target,
        &DnsAnswerSet::parse(&["93.184.216.34"]).unwrap(),
    );
    assert_eq!(
        rebound_public.reason(),
        DecisionReason::PrivateAddressSetMismatch
    );
    assert!(!rebound_public.allowed());

    let mixed = evaluate_destination(
        &policy,
        &target,
        &DnsAnswerSet::parse(&["10.0.0.8", "93.184.216.34"]).unwrap(),
    );
    assert_eq!(mixed.reason(), DecisionReason::MixedPublicAndPrivateAnswers);
    assert!(!mixed.allowed());

    for url in [
        "http://internal.example.test/api/items",
        "https://internal.example.test.evil.test/api/items",
        "https://internal.example.test/other",
    ] {
        assert!(
            !evaluate_destination(
                &policy,
                &request(url, "GET"),
                &DnsAnswerSet::parse(&["10.0.0.8"]).unwrap(),
            )
            .allowed(),
            "{url} bypassed the private identity"
        );
    }
}

#[test]
fn private_service_ip_literal_requires_the_same_literal_and_pinned_answer() {
    let policy = compile(
        PublicScope::Unrestricted,
        vec![PrivateServiceSpec {
            host: "10.0.0.8".to_owned(),
            port: 443,
            methods: vec!["GET".to_owned()],
            path_prefixes: vec!["/api".to_owned()],
            approved_addresses: vec!["10.0.0.8".to_owned()],
        }],
    );
    let target = request("https://10.0.0.8/api/items", "GET");
    assert!(
        evaluate_destination(
            &policy,
            &target,
            &DnsAnswerSet::parse(&["10.0.0.8"]).unwrap(),
        )
        .allowed()
    );
    assert_eq!(
        evaluate_destination(
            &policy,
            &target,
            &DnsAnswerSet::parse(&["10.0.0.9"]).unwrap(),
        )
        .reason(),
        DecisionReason::AddressLiteralMismatch
    );
}

#[test]
fn reserved_names_require_explicit_private_service_identity() {
    let public_answers = DnsAnswerSet::parse(&["93.184.216.34"]).unwrap();
    let unrestricted = compile(PublicScope::Unrestricted, vec![]);
    for url in [
        "https://localhost/",
        "https://sub.localhost/",
        "https://wpad/",
        "https://service.local/",
        "https://metadata.internal/",
        "https://example.invalid/",
        "https://example.test/",
        "https://service.onion/",
        "https://in-addr.arpa/",
        "https://service.alt/",
        "https://service.lan/",
        "https://service.corp/",
        "https://service.home/",
        "https://service.mail/",
        "https://service.intranet/",
        "https://service.intra/",
        "https://service.private/",
    ] {
        let decision = evaluate_destination(&unrestricted, &request(url, "GET"), &public_answers);
        assert_eq!(
            decision.reason(),
            DecisionReason::PrivateDestinationNotAuthorized,
            "{url}"
        );
        assert!(!decision.allowed(), "{url}");
    }

    let private = compile(
        PublicScope::Unrestricted,
        vec![private_service(&["10.0.0.8"])],
    );
    assert!(
        evaluate_destination(
            &private,
            &request("https://internal.example.test/api/items", "GET"),
            &DnsAnswerSet::parse(&["10.0.0.8"]).unwrap(),
        )
        .allowed()
    );
}

#[test]
fn redirects_deny_downgrade_cross_origin_credentials_and_unsafe_replay() {
    let policy = compile(
        PublicScope::Rules(vec![rule("*", &["GET", "POST"], &["/"])]),
        vec![],
    );
    let answers = DnsAnswerSet::parse(&["93.184.216.34"]).unwrap();
    let from = request("https://a.example/start", "POST");

    assert_eq!(
        evaluate_redirect(
            &policy,
            &from,
            &request("http://a.example/next", "POST"),
            307,
            CredentialReplay::None,
            BodyReplay::Replayable,
            &answers,
        )
        .reason(),
        DecisionReason::RedirectDowngrade
    );
    assert_eq!(
        evaluate_redirect(
            &policy,
            &from,
            &request("https://b.example/next", "POST"),
            307,
            CredentialReplay::Present,
            BodyReplay::Replayable,
            &answers,
        )
        .reason(),
        DecisionReason::CrossOriginCredentialReplay
    );
    assert_eq!(
        evaluate_redirect(
            &policy,
            &from,
            &request("https://a.example/next", "POST"),
            307,
            CredentialReplay::None,
            BodyReplay::NonReplayable,
            &answers,
        )
        .reason(),
        DecisionReason::RedirectBodyReplayForbidden
    );
    assert_eq!(
        evaluate_redirect(
            &policy,
            &from,
            &request("https://a.example/next", "GET"),
            307,
            CredentialReplay::None,
            BodyReplay::Replayable,
            &answers,
        )
        .reason(),
        DecisionReason::RedirectMethodMismatch
    );
}

#[test]
fn redirect_semantics_allow_only_explicitly_safe_method_body_combinations() {
    let policy = compile(
        PublicScope::Rules(vec![rule("*", &["GET", "POST"], &["/"])]),
        vec![],
    );
    let answers = DnsAnswerSet::parse(&["93.184.216.34"]).unwrap();
    let post = request("https://a.example/start", "POST");
    assert!(
        evaluate_redirect(
            &policy,
            &post,
            &request("https://a.example/next", "GET"),
            303,
            CredentialReplay::None,
            BodyReplay::None,
            &answers,
        )
        .allowed()
    );
    assert_eq!(
        evaluate_redirect(
            &policy,
            &post,
            &request("https://a.example/next", "POST"),
            302,
            CredentialReplay::None,
            BodyReplay::Replayable,
            &answers,
        )
        .reason(),
        DecisionReason::RedirectBodyReplayForbidden
    );
    let get = request("https://a.example/start", "GET");
    assert!(
        evaluate_redirect(
            &policy,
            &get,
            &request("https://a.example/next", "GET"),
            301,
            CredentialReplay::Present,
            BodyReplay::None,
            &answers,
        )
        .allowed()
    );
    assert_eq!(
        evaluate_redirect(
            &policy,
            &get,
            &request("https://a.example/next", "GET"),
            200,
            CredentialReplay::None,
            BodyReplay::None,
            &answers,
        )
        .reason(),
        DecisionReason::RedirectStatusUnsupported
    );
}

#[test]
fn malformed_policy_fails_visibly() {
    let invalid_version = DestinationPolicy::compile(PolicySpec {
        version: "future".to_owned(),
        public_scope: PublicScope::Unrestricted,
        private_services: vec![],
    });
    assert!(matches!(
        invalid_version,
        Err(ContractError::UnsupportedVersion(_))
    ));

    for bad_rule in [
        RuleSpec {
            host: "example.*.com".to_owned(),
            schemes: vec!["https".to_owned()],
            ports: vec![443],
            methods: vec!["GET".to_owned()],
            path_prefixes: vec!["/".to_owned()],
        },
        RuleSpec {
            host: "example.com".to_owned(),
            schemes: vec!["https".to_owned()],
            ports: vec![443],
            methods: vec![],
            path_prefixes: vec!["/".to_owned()],
        },
        RuleSpec {
            host: "example.com".to_owned(),
            schemes: vec!["https".to_owned()],
            ports: vec![443],
            methods: vec!["GET".to_owned()],
            path_prefixes: vec!["/a/../admin".to_owned()],
        },
    ] {
        assert!(matches!(
            DestinationPolicy::compile(PolicySpec {
                version: DESTINATION_CONTRACT_VERSION.to_owned(),
                public_scope: PublicScope::Rules(vec![bad_rule]),
                private_services: vec![],
            }),
            Err(ContractError::MalformedPolicy(_))
        ));
    }

    assert!(matches!(
        DestinationPolicy::compile(PolicySpec {
            version: DESTINATION_CONTRACT_VERSION.to_owned(),
            public_scope: PublicScope::Unrestricted,
            private_services: vec![PrivateServiceSpec {
                host: "*.internal.test".to_owned(),
                port: 443,
                methods: vec!["GET".to_owned()],
                path_prefixes: vec!["/".to_owned()],
                approved_addresses: vec![],
            }],
        }),
        Err(ContractError::MalformedPolicy(_))
    ));

    for host in [
        " example.com",
        "example.com ",
        "example.com?query",
        "example.com#fragment",
        "example.com/..",
        "user@example.com",
        "example.com..",
        "example.com:",
        "-bad.example",
    ] {
        assert!(matches!(
            DestinationPolicy::compile(PolicySpec {
                version: DESTINATION_CONTRACT_VERSION.to_owned(),
                public_scope: PublicScope::Rules(vec![rule(host, &["GET"], &["/"])]),
                private_services: vec![],
            }),
            Err(ContractError::MalformedPolicy(_))
        ));
    }

    assert!(matches!(
        DestinationPolicy::compile(PolicySpec {
            version: DESTINATION_CONTRACT_VERSION.to_owned(),
            public_scope: PublicScope::Unrestricted,
            private_services: vec![private_service(&["93.184.216.34"])],
        }),
        Err(ContractError::MalformedPolicy(_))
    ));

    let oversized_answers = vec!["1.1.1.1"; 17];
    assert_eq!(
        DnsAnswerSet::parse(&oversized_answers),
        Err(ContractError::DnsAnswerSetTooLarge)
    );
}

#[test]
fn destination_policy_contract_has_no_runtime_registration() {
    let source = include_str!("../src/destination_contract.rs");
    for forbidden in [
        "TcpStream",
        "UdpSocket",
        "connect(",
        "reqwest",
        "hyper::Client",
    ] {
        assert!(
            !source.contains(forbidden),
            "pure contract unexpectedly contains {forbidden}"
        );
    }
}

#[test]
fn frozen_fixture_uses_the_compiled_contract_version() {
    let fixture_source = include_str!("contract-v1.json");
    assert_eq!(
        format!("{:x}", Sha256::digest(fixture_source.as_bytes())),
        FROZEN_FIXTURE_SHA256
    );
    let fixture: serde_json::Value = serde_json::from_str(fixture_source).unwrap();
    assert_eq!(
        fixture["contract_version"].as_str(),
        Some(DESTINATION_CONTRACT_VERSION)
    );
    assert_eq!(fixture["runtime_registered"].as_bool(), Some(false));
    assert_eq!(fixture["cases"].as_array().map(Vec::len), Some(6));

    for case in fixture["cases"].as_array().unwrap() {
        let policy = fixture_policy(case);
        let answers = fixture_answers(case);
        let decision = if let Some(from) = case.get("from") {
            let to = &case["to"];
            evaluate_redirect(
                &policy,
                &request(
                    from["url"].as_str().unwrap(),
                    from["method"].as_str().unwrap(),
                ),
                &request(to["url"].as_str().unwrap(), to["method"].as_str().unwrap()),
                u16::try_from(case["status"].as_u64().unwrap()).unwrap(),
                match case["credentials"].as_str().unwrap() {
                    "None" => CredentialReplay::None,
                    "Present" => CredentialReplay::Present,
                    value => panic!("unknown credential fixture value: {value}"),
                },
                match case["body"].as_str().unwrap() {
                    "None" => BodyReplay::None,
                    "Replayable" => BodyReplay::Replayable,
                    "NonReplayable" => BodyReplay::NonReplayable,
                    value => panic!("unknown body fixture value: {value}"),
                },
                &answers,
            )
        } else {
            evaluate_destination(
                &policy,
                &request(
                    case["url"].as_str().unwrap(),
                    case["method"].as_str().unwrap(),
                ),
                &answers,
            )
        };
        assert_eq!(decision.allowed(), case["allowed"].as_bool().unwrap());
        assert_eq!(
            format!("{:?}", decision.reason()),
            case["reason"].as_str().unwrap()
        );
    }
}
