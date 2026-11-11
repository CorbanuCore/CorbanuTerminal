use hickory_resolver::config::NameServerConfig;
use hickory_resolver::config::ResolveHosts;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::config::ResolverOpts;
use hickory_resolver::proto::op::Message;
use hickory_resolver::proto::op::OpCode;
use hickory_resolver::proto::rr::Name;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::proto::rr::Record;
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::proto::rr::rdata::A;
use hickory_resolver::proto::rr::rdata::AAAA;
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

#[tokio::test]
async fn patched_adapter_preserves_a_aaaa_txt_and_absolute_names() {
    tokio::time::timeout(Duration::from_secs(/*secs*/ 10), async {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = socket.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let mut observed = Vec::new();
            let mut buffer = [0u8; 4096];
            for _ in 0..3 {
                let (size, peer) = socket.recv_from(&mut buffer).await.unwrap();
                let request = Message::from_vec(&buffer[..size]).unwrap();
                assert_eq!(request.queries.len(), 1);
                let query = &request.queries[0];
                assert_eq!(query.name, Name::from_utf8("fixture.example.").unwrap());
                assert!(query.name.is_fqdn());
                observed.push(query.query_type);
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
                let mut response = Message::response(request.metadata.id, OpCode::Query);
                response.metadata.authoritative = true;
                response.metadata.recursion_available = true;
                response.queries = request.queries.clone();
                for value in data {
                    response.add_answer(Record::from_rdata(
                        query.name.clone(),
                        /*ttl*/ 60,
                        value,
                    ));
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
        options.attempts = 1;
        options.timeout = Duration::from_secs(/*secs*/ 2);
        let dns = HickoryDns::builder()
            .with_config(ResolverConfig::from_name_servers(vec![nameserver]))
            .with_options(options)
            .build();
        let domain = Domain::try_from("fixture.example".to_owned()).unwrap();
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
            dns.txt_lookup(domain).await.unwrap(),
            vec![b"first chunk".to_vec(), b"second chunk".to_vec(),]
        );
        assert_eq!(
            server.await.unwrap(),
            vec![RecordType::A, RecordType::AAAA, RecordType::TXT]
        );
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
