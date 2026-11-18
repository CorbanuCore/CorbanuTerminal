//! #2401 / #2604: every URL password in an environment value is registered
//! for redaction, and every value with one is dropped at launch. Synthetic
//! credentials only.

use super::*;
use codex_protocol::protocol::ErrorEvent;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::ffi::OsString;

fn registered(name: &str, value: &str) -> Vec<String> {
    env_values([(OsString::from(name), OsString::from(value))])
        .into_iter()
        .filter(|(label, _)| label == &format!("env:{name}:password"))
        .map(|(_, secret)| secret.to_string())
        .collect()
}

fn launch_drops(value: &str) -> bool {
    let mut env = HashMap::from([("HTTPS_PROXY".to_string(), value.to_string())]);
    secretless_launch::retain_launch_env(&mut env, |_| false);
    env.is_empty() && secretless_launch::value_has_url_password(value)
}

#[test]
fn issue_2401_url_passwords_are_redacted_in_later_output() {
    for (name, value, passwords) in [
        (
            "PROXY_LIST",
            "https://public.example,http://u:First-2401@one https://v:Second-2401@two",
            &["First-2401", "Second-2401"][..],
        ),
        (
            "PROXY_LIST_V6",
            "u:Proxy-2401@[2001:db8::1]:3128;http://v:Last-2401@[::1]/",
            &["Proxy-2401", "Last-2401"],
        ),
        (
            "ENCODED_PROXY",
            "u:Raw%40Pass-2401@proxy https://v:Next%3aPass-2401@host",
            &[
                "Raw%40Pass-2401",
                "Raw@Pass-2401",
                "Next%3aPass-2401",
                "Next:Pass-2401",
            ],
        ),
        (
            "AMBIGUOUS_PROXY",
            "http://u:Pass@word,2401@host;u:Second-2401@proxy",
            &["Pass@word,2401", "Second-2401"],
        ),
        (
            "PORT_THEN_ITEM",
            "http://pub.example:8080,user:Pass-2401@proxy)/x",
            &["Pass-2401"],
        ),
        (
            "QUOTE_THEN_QUERY",
            "http://pub.example:8080,user:Pass-2401@proxy'?q",
            &["Pass-2401"],
        ),
        // #2604: a later `name@host` item must not hide the password.
        (
            "AMBIGUOUS_EMAIL",
            "http://u:Secret-2401@proxy:3128,mail@example.com",
            &["Secret-2401"],
        ),
    ] {
        assert!(launch_drops(value), "{value}");
        let values = env_values([(OsString::from(name), OsString::from(value))]);
        let gate = OutputGate::new();
        let entries: Vec<_> = values
            .iter()
            .map(|(label, value)| (label.as_str(), SecretClass::Operational, value.as_str()))
            .collect();
        gate.register_all(&entries).expect("register URL passwords");
        gate.arm().expect("arm");
        for password in passwords {
            let output = EventMsg::Error(ErrorEvent {
                message: format!("tool later printed {password}"),
                codex_error_info: None,
            });
            assert_eq!(
                serde_json::to_value(present(&gate, output)).expect("serialize output"),
                serde_json::json!({
                    "type": "error",
                    "message": format!("tool later printed [REDACTED:env:{name}:password]"),
                    "codex_error_info": null,
                }),
                "{value}: {password}",
            );
        }
        // Short readings such as `Pass` are never registered.
        let ordinary = EventMsg::Error(ErrorEvent {
            message: "Pass and word remain ordinary short words".to_string(),
            codex_error_info: None,
        });
        assert_eq!(
            serde_json::to_value(present(&gate, ordinary.clone())).expect("serialize output"),
            serde_json::to_value(ordinary).expect("serialize original"),
        );
    }
}

#[test]
fn issue_2401_noncredentials_and_short_decoded_passwords_are_not_registered() {
    for value in [
        "status: ready contact: alice@example.com git@host:repo",
        "http://user@host user@[::1]:3128",
        "http://u:@host u:@proxy http://u:   @host",
        "u:%20%20%20%20%20%20%20%20@proxy",
        "http://u:Word-12@host",
        "u:%57%6f%72%64%2d%31%32@proxy",
        "http://u:postgres@host",
        "user:ordinary@proxy",
        "10:30@home",
        "a:b@c",
        "/opt/homebrew/opt/node@20/bin:/usr/bin",
    ] {
        assert_eq!(
            registered("SETTING", value),
            Vec::<String>::new(),
            "{value}"
        );
    }
    // Over-registration, shown: short or one-class passwords are never
    // registered on their own, but the reading cut after a space includes the
    // user name and can be long enough.
    assert_eq!(
        registered("PROXY", "http://u:Word-12@host u:%57%6f%72%64%2d%31%32@proxy"),
        vec!["u:%57%6f%72%64%2d%31%32", "u:Word-12"],
    );
    assert_eq!(
        registered("PROXY", "http://u:postgres@host user:ordinary@proxy"),
        vec!["user:ordinary"],
    );
    // Eight decoded bytes and mixed character classes meet the existing
    // environment-seeding threshold; encoded length alone does not count.
    assert_eq!(
        registered("PROXY", "u:%57%6f%72%64%2d%31%32%33@proxy"),
        vec!["%57%6f%72%64%2d%31%32%33", "Word-123"],
    );
}

/// Deterministic xorshift generator: the property test needs no new
/// dependency and fails reproducibly.
struct Rng(u64);

impl Rng {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

/// A plausible password (lowercase, uppercase and a digit once decoded)
/// salted with punctuation that has broken hand-written userinfo parsing.
fn random_password(rng: &mut Rng) -> String {
    const PIECES: &[&str] = &[
        "a", "b", "x", "Q", "Z", "7", "4", ",", ";", ":", "@", ")", "(", "'", "\"", "/", "?", "#",
        "%40", "%3A", "%2F", "%25", "%2C", "%22", "@proxy", ",u:", "//", "]", "-",
    ];
    let mut password = String::from(rng.pick(&["Pw7", "kQ2", "zZ9"]));
    for _ in 0..5 + rng.below(12) {
        let piece = rng.pick(PIECES);
        let at = rng.below(2) * password.len();
        password.insert_str(at, piece);
    }
    password
}

#[test]
fn issue_2401_random_url_passwords_are_registered_and_launch_filtered() {
    const USERS: &[&str] = &[
        "u",
        "user",
        "proxy-user",
        "alice@corp.example",
        r"DOMAIN\svc",
    ];
    const HOSTS: &[&str] = &[
        "proxy",
        "proxy.example.com",
        "proxy_internal.corp",
        "10.0.0.1",
        "[::1]",
        "[2001:db8::1]",
    ];
    const PORTS: &[&str] = &["", ":3128", ":8080"];
    const TAILS: &[&str] = &["", "/", "/path?x=1", "?q=1", "#frag", "/x)"];
    const WRAPPERS: &[(&str, &str)] = &[("", ""), ("\"", "\""), ("'", "'"), ("(", ")"), ("<", ">")];
    const SEPARATORS: &[&str] = &[",", ";", " ", ", ", "\t", "; "];
    const NOISE: &[&str] = &[
        "localhost",
        "https://public.example",
        "http://pub.example:8080",
        "pub.example:8080",
        "user@host",
        "10:30@home",
        "mail@example.com",
        "http://u:Other-2401@other",
    ];
    let mut rng = Rng(0x2401_2604_5eed_0001);
    for case in 0..400 {
        let mut password = random_password(&mut rng);
        let schemeless = rng.below(2) == 0;
        if schemeless {
            // `user://password@host` is also a valid URL with no password.
            password.insert(0, 'P');
        }
        let user = rng.pick(USERS);
        let authority = format!(
            "{user}:{password}@{}{}{}",
            rng.pick(HOSTS),
            rng.pick(PORTS),
            rng.pick(TAILS)
        );
        let url = if schemeless {
            authority
        } else {
            let scheme = rng.pick(&["http", "https", "socks5h", "postgres"]);
            format!("{scheme}://{authority}")
        };
        let (open, close) = WRAPPERS[rng.below(WRAPPERS.len())];
        let mut value = format!("{open}{url}{close}");
        for _ in 0..rng.below(3) {
            value = format!("{}{}{value}", rng.pick(NOISE), rng.pick(SEPARATORS));
        }
        for _ in 0..rng.below(3) {
            value = format!("{value}{}{}", rng.pick(SEPARATORS), rng.pick(NOISE));
        }

        assert!(launch_drops(&value), "case {case}: {value}");
        let registered = registered("PROXY_LIST", &value);
        assert!(
            registered.contains(&password),
            "case {case}: {password} not registered from {value}"
        );
        if let Ok(decoded) = urlencoding::decode(&password)
            && decoded != password
        {
            assert!(
                registered.iter().any(|secret| *secret == *decoded),
                "case {case}: decoded {decoded} not registered from {value}"
            );
        }
    }
}
