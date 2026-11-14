use hickory_resolver::config::NameServerConfig;
use hickory_resolver::config::ResolveHosts;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::config::ResolverOpts;
use hickory_resolver::proto::op::Message;
use hickory_resolver::proto::op::OpCode;
use hickory_resolver::proto::rr::DNSClass;
use hickory_resolver::proto::rr::Name;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::proto::rr::Record;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::proto::rr::rdata::A;
use hickory_resolver::proto::rr::rdata::AAAA;
use hickory_resolver::proto::rr::rdata::CNAME;
use hickory_resolver::proto::rr::rdata::TXT;
use pretty_assertions::assert_eq;
use rama_dns::DnsResolver;
use rama_dns::HickoryDns;
use rama_dns::hickory::resolver as hickory_resolver;
use rama_net::address::Domain;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;

#[derive(Clone, Copy)]
enum ResponseShape {
    Direct,
    MixedOwner,
    ForeignClass,
    CnameAdditional,
    CnameChain,
    CnameSeparate,
}

#[derive(Clone, Copy)]
enum Intermediates {
    Preserve,
    Discard,
}

#[tokio::test]
async fn patched_adapter_preserves_a_aaaa_txt_and_absolute_names() {
    check_response(ResponseShape::Direct, Intermediates::Preserve).await;
}

#[tokio::test]
async fn patched_adapter_ignores_unrelated_answer_owners() {
    check_response(ResponseShape::MixedOwner, Intermediates::Preserve).await;
}

#[tokio::test]
async fn patched_adapter_rejects_foreign_class_responses() {
    check_response(ResponseShape::ForeignClass, Intermediates::Preserve).await;
}

#[tokio::test]
async fn patched_adapter_preserves_cname_additional_targets() {
    check_response(ResponseShape::CnameAdditional, Intermediates::Preserve).await;
}

#[tokio::test]
async fn patched_adapter_follows_only_the_requested_cname_chain() {
    check_response(ResponseShape::CnameChain, Intermediates::Preserve).await;
}

#[tokio::test]
async fn patched_adapter_preserves_recursive_and_cached_cname_results() {
    check_response(ResponseShape::CnameSeparate, Intermediates::Preserve).await;
}

#[tokio::test]
async fn discarded_intermediates_do_not_admit_unrelated_owners() {
    check_response(ResponseShape::MixedOwner, Intermediates::Discard).await;
}

#[tokio::test]
async fn discarded_intermediates_preserve_additional_targets_without_duplicates() {
    check_response(ResponseShape::CnameAdditional, Intermediates::Discard).await;
}

#[tokio::test]
async fn discarded_intermediates_preserve_recursive_and_cached_targets() {
    check_response(ResponseShape::CnameSeparate, Intermediates::Discard).await;
}

async fn check_response(shape: ResponseShape, intermediates: Intermediates) {
    tokio::time::timeout(Duration::from_secs(/*secs*/ 10), async {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = socket.local_addr().unwrap();
        let (stop, mut stopped) = oneshot::channel::<()>();
        let server = tokio::spawn(async move {
            let mut observed = Vec::new();
            let mut buffer = [0u8; 4096];
            loop {
                let (size, peer) = tokio::select! {
                    result = socket.recv_from(&mut buffer) => result.unwrap(),
                    _ = &mut stopped => break,
                };
                let request = Message::from_vec(&buffer[..size]).unwrap();
                assert_eq!(request.queries.len(), 1);
                let query = &request.queries[0];
                let original = Name::from_utf8("fixture.example.").unwrap();
                let target = Name::from_utf8("target.example.").unwrap();
                let intermediate = Name::from_utf8("intermediate.example.").unwrap();
                let unrelated = Name::from_utf8("unrelated.example.").unwrap();
                assert!(query.name == original || query.name == target);
                assert!(query.name.is_fqdn());
                observed.push(query.query_type);
                assert!(observed.len() <= 24, "unexpected resolver query loop");
                let mut response = Message::response(request.metadata.id, OpCode::Query);
                response.metadata.authoritative = true;
                response.metadata.recursion_available = true;
                response.queries = request.queries.clone();

                let alias = matches!(
                    shape,
                    ResponseShape::CnameAdditional
                        | ResponseShape::CnameChain
                        | ResponseShape::CnameSeparate
                ) && query.name == original;
                if alias {
                    // Unrelated aliases must not change the selected target.
                    response.add_answer(Record::from_rdata(
                        unrelated.clone(),
                        /*ttl*/ 60,
                        RData::CNAME(CNAME(Name::from_utf8("wrong.example.").unwrap())),
                    ));
                    if matches!(shape, ResponseShape::CnameChain) {
                        response.add_answer(Record::from_rdata(
                            original.clone(),
                            /*ttl*/ 60,
                            RData::CNAME(CNAME(intermediate.clone())),
                        ));
                        response.add_answer(Record::from_rdata(
                            intermediate,
                            /*ttl*/ 60,
                            RData::CNAME(CNAME(target.clone())),
                        ));
                    } else {
                        response.add_answer(Record::from_rdata(
                            original,
                            /*ttl*/ 60,
                            RData::CNAME(CNAME(target.clone())),
                        ));
                    }
                }
                if query.query_type != RecordType::CNAME
                    && !(alias && matches!(shape, ResponseShape::CnameSeparate))
                {
                    let data = match query.query_type {
                        RecordType::A => vec![
                            RData::A(A("203.0.113.10".parse().unwrap())),
                            RData::A(A("203.0.113.11".parse().unwrap())),
                        ],
                        RecordType::AAAA => vec![
                            RData::AAAA(AAAA("2001:db8::10".parse().unwrap())),
                            RData::AAAA(AAAA("2001:db8::11".parse().unwrap())),
                        ],
                        RecordType::TXT => vec![RData::TXT(TXT::new(vec![
                            "first chunk".to_owned(),
                            "second chunk".to_owned(),
                        ]))],
                        other => panic!("unexpected query: {other}"),
                    };
                    let wrong = match query.query_type {
                        RecordType::A => RData::A(A("203.0.113.99".parse().unwrap())),
                        RecordType::AAAA => RData::AAAA(AAAA("2001:db8::99".parse().unwrap())),
                        RecordType::TXT => RData::TXT(TXT::new(vec!["wrong owner".to_owned()])),
                        other => panic!("unexpected query: {other}"),
                    };
                    if matches!(shape, ResponseShape::ForeignClass) {
                        let mut wrong_class =
                            Record::from_rdata(query.name.clone(), /*ttl*/ 60, wrong.clone());
                        wrong_class.dns_class = DNSClass::CH;
                        response.add_answer(wrong_class);
                    }
                    if !matches!(shape, ResponseShape::Direct) {
                        response.add_answer(Record::from_rdata(
                            unrelated.clone(),
                            /*ttl*/ 60,
                            wrong.clone(),
                        ));
                        response
                            .add_additional(Record::from_rdata(unrelated, /*ttl*/ 60, wrong));
                    }
                    for value in data {
                        let record = Record::from_rdata(
                            if alias {
                                target.clone()
                            } else {
                                query.name.clone()
                            },
                            /*ttl*/ 60,
                            value,
                        );
                        if alias && matches!(shape, ResponseShape::CnameAdditional) {
                            response.add_additional(record);
                        } else {
                            response.add_answer(record);
                        }
                    }
                }
                socket
                    .send_to(&response.to_vec().unwrap(), peer)
                    .await
                    .unwrap();
            }
            observed
        });

        let mut nameserver = NameServerConfig::udp(address.ip());
        nameserver.connections[0].port = address.port();
        let mut options = ResolverOpts::default();
        options.use_hosts_file = ResolveHosts::Never;
        options.preserve_intermediates = matches!(intermediates, Intermediates::Preserve);
        options.attempts = 1;
        options.timeout = Duration::from_secs(/*secs*/ 2);
        let dns = HickoryDns::builder()
            .with_config(ResolverConfig::from_name_servers(vec![nameserver]))
            .with_options(options)
            .build();
        let domain = Domain::try_from("fixture.example".to_owned()).unwrap();
        // Repeating every lookup exercises Hickory's cache as well as wire replies.
        for _ in 0..2 {
            if matches!(shape, ResponseShape::ForeignClass) {
                // Hickory rejects mixed classes before exposing a Lookup.
                for result in [
                    dns.ipv4_lookup(domain.clone()).await.map(|_| ()),
                    dns.ipv6_lookup(domain.clone()).await.map(|_| ()),
                    dns.txt_lookup(domain.clone()).await.map(|_| ()),
                ] {
                    assert!(result.is_err());
                }
                continue;
            }
            assert_eq!(
                dns.ipv4_lookup(domain.clone()).await.unwrap(),
                vec![
                    "203.0.113.10".parse::<Ipv4Addr>().unwrap(),
                    "203.0.113.11".parse::<Ipv4Addr>().unwrap(),
                ]
            );
            assert_eq!(
                dns.ipv6_lookup(domain.clone()).await.unwrap(),
                vec![
                    "2001:db8::10".parse::<Ipv6Addr>().unwrap(),
                    "2001:db8::11".parse::<Ipv6Addr>().unwrap(),
                ]
            );
            assert_eq!(
                dns.txt_lookup(domain.clone()).await.unwrap(),
                vec![b"first chunk".to_vec(), b"second chunk".to_vec()]
            );
        }
        stop.send(()).unwrap();
        let observed = server.await.unwrap();
        for kind in [RecordType::A, RecordType::AAAA, RecordType::TXT] {
            assert!(observed.contains(&kind));
        }
    })
    .await
    .expect("loopback DNS fixture must complete without external resolution");
}

#[tokio::test]
async fn empty_resolver_returns_lookup_errors() {
    let dns = HickoryDns::builder()
        .with_config(ResolverConfig::from_name_servers(Vec::new()))
        .build();
    let domain = Domain::try_from("fixture.example".to_owned()).unwrap();
    for result in [
        dns.ipv4_lookup(domain.clone()).await.map(|_| ()),
        dns.ipv6_lookup(domain.clone()).await.map(|_| ()),
        dns.txt_lookup(domain).await.map(|_| ()),
    ] {
        assert!(result.is_err());
    }
}
