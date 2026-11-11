use pretty_assertions::assert_eq;

use super::*;

fn class(addr: &str) -> HttpAddressClass {
    HttpAddressClass::of(addr.parse().expect("test address should parse"))
}

#[test]
fn classifies_addresses() {
    for (addr, expected) in [
        ("93.184.216.34", HttpAddressClass::Public),
        ("2606:4700::1111", HttpAddressClass::Public),
        ("127.0.0.1", HttpAddressClass::Loopback),
        ("127.1.2.3", HttpAddressClass::Loopback),
        ("::1", HttpAddressClass::Loopback),
        ("::ffff:127.0.0.1", HttpAddressClass::Loopback),
        ("::ffff:0:127.0.0.1", HttpAddressClass::Loopback),
        ("10.1.2.3", HttpAddressClass::Private),
        ("172.16.0.1", HttpAddressClass::Private),
        ("192.168.1.1", HttpAddressClass::Private),
        ("100.64.0.1", HttpAddressClass::Private),
        ("fd12:3456::1", HttpAddressClass::Private),
        ("64:ff9b::10.0.0.1", HttpAddressClass::Private),
        ("2002:0a00:0001::", HttpAddressClass::Private),
        ("169.254.1.1", HttpAddressClass::LinkLocal),
        ("fe80::1", HttpAddressClass::LinkLocal),
        ("169.254.169.254", HttpAddressClass::Blocked),
        ("::ffff:169.254.169.254", HttpAddressClass::Blocked),
        ("::169.254.169.254", HttpAddressClass::Blocked),
        ("169.254.170.2", HttpAddressClass::Blocked),
        ("100.100.100.200", HttpAddressClass::Blocked),
        ("fd00:ec2::254", HttpAddressClass::Blocked),
        ("fd20:ce::254", HttpAddressClass::Blocked),
        ("0.0.0.0", HttpAddressClass::Blocked),
        ("::", HttpAddressClass::Blocked),
        ("224.0.0.1", HttpAddressClass::Blocked),
        ("ff02::1", HttpAddressClass::Blocked),
        ("255.255.255.255", HttpAddressClass::Blocked),
        ("240.0.0.1", HttpAddressClass::Blocked),
        ("198.18.0.1", HttpAddressClass::Blocked),
        ("2001::1", HttpAddressClass::Blocked),
    ] {
        assert_eq!(class(addr), expected, "{addr}");
    }
}

#[test]
fn public_policy_allows_only_public_addresses() {
    let policy = HttpAddressPolicy::default();
    assert!(policy.allows_resolved("93.184.216.34".parse().unwrap()));
    for addr in ["127.0.0.1", "10.0.0.1", "169.254.169.254", "fd00:ec2::254"] {
        assert!(!policy.allows_resolved(addr.parse().unwrap()), "{addr}");
        assert!(!policy.allows_literal(addr.parse().unwrap()), "{addr}");
    }
}

#[test]
fn private_policy_never_allows_metadata_or_link_local() {
    let policy = HttpAddressPolicy {
        allow_loopback: false,
        allow_private: true,
        allowed_address: Some("10.0.0.5".parse().unwrap()),
    };
    assert!(policy.allows_resolved("10.9.9.9".parse().unwrap()));
    assert!(policy.allows_resolved("fd12::1".parse().unwrap()));
    assert!(!policy.allows_resolved("127.0.0.1".parse().unwrap()));
    for addr in ["169.254.169.254", "fd00:ec2::254", "169.254.1.1", "fe80::1"] {
        assert!(!policy.allows_literal(addr.parse().unwrap()), "{addr}");
    }
}

#[test]
fn allowed_address_applies_to_literals_only() {
    let policy = HttpAddressPolicy {
        allow_loopback: false,
        allow_private: false,
        allowed_address: Some("169.254.10.10".parse().unwrap()),
    };
    assert!(policy.allows_literal("169.254.10.10".parse().unwrap()));
    assert!(policy.allows_literal("::ffff:169.254.10.10".parse().unwrap()));
    assert!(!policy.allows_resolved("169.254.10.10".parse().unwrap()));
    assert!(!policy.allows_literal("169.254.10.11".parse().unwrap()));
}

#[test]
fn policy_round_trips_and_defaults() {
    let policy: HttpAddressPolicy = serde_json::from_str("{}").unwrap();
    assert_eq!(policy, HttpAddressPolicy::default());
    let policy = HttpAddressPolicy {
        allow_loopback: true,
        allow_private: false,
        allowed_address: Some("::1".parse().unwrap()),
    };
    let json = serde_json::to_value(&policy).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"allowLoopback": true, "allowPrivate": false, "allowedAddress": "::1"})
    );
    assert_eq!(
        serde_json::from_value::<HttpAddressPolicy>(json).unwrap(),
        policy
    );
}

#[test]
fn host_names_follow_the_policy() {
    let public = HttpAddressPolicy::default();
    let loopback = HttpAddressPolicy {
        allow_loopback: true,
        ..HttpAddressPolicy::default()
    };
    assert!(public.allows_host_name("example.com"));
    for host in ["localhost", "LOCALHOST.", "dev.localhost"] {
        assert!(!public.allows_host_name(host), "{host}");
        assert!(loopback.allows_host_name(host), "{host}");
    }
    for host in [
        "metadata",
        "Metadata.Google.Internal.",
        "metadata.azure.internal",
    ] {
        assert!(!loopback.allows_host_name(host), "{host}");
    }
}
