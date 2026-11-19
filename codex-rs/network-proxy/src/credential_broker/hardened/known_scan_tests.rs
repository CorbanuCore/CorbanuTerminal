use super::KnownCredentialScanError;
use crate::credential_broker::CredentialBroker;
use crate::credential_broker::hardened::HardenedBroker;
use crate::credential_broker::hardened::RecordSecret;
use crate::credential_broker::hardened::fingerprint;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use zeroize::Zeroizing;

const SHORT_GH_TOKEN: &str = "ghp_hidden_alias_secret";

fn register_gh_token(broker: &CredentialBroker, value: &str) -> String {
    let mut env = HashMap::from([("GH_TOKEN".to_string(), value.to_string())]);
    broker.virtualize_child_env(&mut env);
    let dummy = env["GH_TOKEN"].clone();
    assert_ne!(dummy, value);
    dummy
}

#[cfg(any(unix, windows))]
fn unavailable_isolated_broker() -> CredentialBroker {
    use crate::credential_broker::isolated::IsolatedBrokerLauncher;
    use crate::credential_broker::isolated::IsolatedBrokerOptions;
    CredentialBroker::new_isolated_with_launcher(
        /*enabled*/ true,
        IsolatedBrokerOptions {
            allow_local_binding: true,
            allow_upstream_proxy: false,
            runtime_dir: None,
            require_containment: true,
            scrub_responses: false,
            pin_connections: false,
        },
        IsolatedBrokerLauncher::test_harness(
            Some(std::path::PathBuf::from("/nonexistent/corbanu-broker")),
            Vec::new(),
            Vec::new(),
            /*controller_pid_override*/ None,
        ),
    )
    .with_main_engine()
}

#[test]
fn refuses_when_the_hardened_engine_does_not_own_brokering() {
    let broker = HardenedBroker::new(/*enabled*/ true);

    assert_eq!(
        broker.known_credential_occurs_in("anything"),
        Err(KnownCredentialScanError::NotArmed)
    );
}

#[test]
fn finds_in_process_credential_copied_into_other_text() {
    let broker = CredentialBroker::new(/*enabled*/ true).with_main_engine();
    assert_eq!(broker.known_credential_occurs_in("nothing yet"), Ok(false));
    let dummy = register_gh_token(&broker, SHORT_GH_TOKEN);

    for text in [
        SHORT_GH_TOKEN.to_string(),
        format!("prefix\"{SHORT_GH_TOKEN}\"suffix"),
        format!("export HIDDEN_HEADER='Bearer {SHORT_GH_TOKEN}'\n"),
        format!("x{SHORT_GH_TOKEN}y"),
    ] {
        assert_eq!(broker.known_credential_occurs_in(&text), Ok(true), "{text}");
    }
    for text in [
        String::new(),
        dummy,
        SHORT_GH_TOKEN[..SHORT_GH_TOKEN.len() - 1].to_string(),
        "export SAFE_EXPORT='ordinary'".to_string(),
    ] {
        assert_eq!(
            broker.known_credential_occurs_in(&text),
            Ok(false),
            "{text}"
        );
    }
}

#[test]
fn short_in_process_credential_matches_only_as_a_whole_word() {
    let broker = CredentialBroker::new(/*enabled*/ true).with_main_engine();
    register_gh_token(&broker, SHORT_GH_TOKEN);
    {
        let mut state = broker.hardened.write_state();
        let record = &mut state.credentials[0];
        record.secret = RecordSecret::Raw(Zeroizing::new("tok123".to_string()));
        record.value_len = "tok123".len();
    }

    for text in ["tok123", "Bearer tok123", "x=\"tok123\"", "a-tok123.b"] {
        assert_eq!(broker.known_credential_occurs_in(text), Ok(true), "{text}");
    }
    for text in ["xtok123", "tok123x", "tok1234", "_tok123", "stock1234"] {
        assert_eq!(broker.known_credential_occurs_in(text), Ok(false), "{text}");
    }
}

#[test]
fn credential_too_short_to_recognize_fails_closed() {
    let broker = CredentialBroker::new(/*enabled*/ true).with_main_engine();
    register_gh_token(&broker, SHORT_GH_TOKEN);
    {
        let mut state = broker.hardened.write_state();
        let record = &mut state.credentials[0];
        record.secret = RecordSecret::Raw(Zeroizing::new("ab".to_string()));
        record.value_len = 2;
    }

    assert_eq!(
        broker.known_credential_occurs_in("unrelated text"),
        Err(KnownCredentialScanError::Unrecognizable)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn finds_isolated_credential_by_fingerprint_without_its_raw_value() {
    let broker = unavailable_isolated_broker();
    let dummy = register_gh_token(&broker, SHORT_GH_TOKEN);
    assert!(!broker.holds_raw_value(SHORT_GH_TOKEN));

    assert_eq!(
        broker.known_credential_occurs_in(&format!("prefix\"{SHORT_GH_TOKEN}\"suffix")),
        Ok(true)
    );
    assert_eq!(broker.known_credential_occurs_in(&dummy), Ok(false));
    assert_eq!(
        broker.known_credential_occurs_in("export SAFE_EXPORT='ordinary'"),
        Ok(false)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn isolated_scan_fails_closed_when_its_window_budget_is_spent() {
    let broker = unavailable_isolated_broker();
    register_gh_token(&broker, SHORT_GH_TOKEN);
    let padding = 50;
    // The value occupies the last window: `padding + 1` fingerprints in all.
    let text = format!("{}{SHORT_GH_TOKEN}", "x".repeat(padding));

    assert_eq!(
        broker
            .hardened
            .known_credential_occurs_in_with_budget(&text, padding),
        Err(KnownCredentialScanError::BudgetExceeded)
    );
    assert_eq!(
        broker
            .hardened
            .known_credential_occurs_in_with_budget(&text, padding + 1),
        Ok(true)
    );
    assert_eq!(
        broker
            .hardened
            .known_credential_occurs_in_with_budget(&"x".repeat(padding), 0),
        Err(KnownCredentialScanError::BudgetExceeded)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn short_isolated_credential_matches_only_as_a_whole_word() {
    let broker = unavailable_isolated_broker();
    register_gh_token(&broker, SHORT_GH_TOKEN);
    let key = broker
        .hardened
        .fingerprint_key()
        .expect("isolated fingerprint key");
    {
        let mut state = broker.hardened.write_state();
        let record = &mut state.credentials[0];
        record.secret = RecordSecret::Unavailable {
            fingerprint: fingerprint(&key, &record.env_var, "tok123"),
        };
        record.value_len = "tok123".len();
    }

    assert_eq!(broker.known_credential_occurs_in("Bearer tok123"), Ok(true));
    assert_eq!(
        broker.known_credential_occurs_in("xtok123 tok1234"),
        Ok(false)
    );
}

const D732_TOKEN: &str = "ghp_d732DummyCopyCanary0000000000000000000";

fn d732_env(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

fn d732_route(
    broker: &CredentialBroker,
    dummy: &str,
) -> Result<(), crate::credential_broker::ScopedCredentialInjectionError> {
    let mut headers = rama_http::HeaderMap::new();
    headers.insert(
        rama_http::header::AUTHORIZATION,
        rama_http::HeaderValue::from_str(&format!("Bearer {dummy}")).expect("header"),
    );
    broker
        .route_request_credentials(
            "https",
            "api.github.com",
            /*port*/ 443,
            "GET",
            "/user",
            &mut headers,
        )
        .map(|_| ())
}

/// D732: copies of a held key under other names (a header, a second
/// variable, an embedded value) are removed from the command environment;
/// other variables stay and the provider variable keeps its dummy.
#[test]
fn d732_copies_of_a_held_key_are_removed_from_the_command_env() {
    let broker = CredentialBroker::new(/*enabled*/ true).with_main_engine();
    let header = format!("Bearer {D732_TOKEN}");
    let embedded = format!("https://x.test/?k={D732_TOKEN}&v=1");
    let mut env = d732_env(&[
        ("GH_TOKEN", D732_TOKEN),
        ("AUTH_HEADER", &header),
        ("TOKEN_COPY", D732_TOKEN),
        ("EMBEDDED", &embedded),
        ("SAFE", "ordinary value"),
        ("PATH", "/usr/local/bin:/bin"),
    ]);
    broker.virtualize_child_env(&mut env);

    for name in ["AUTH_HEADER", "TOKEN_COPY", "EMBEDDED"] {
        assert_eq!(env.get(name), None, "{name}");
    }
    assert_eq!(env.get("SAFE").map(String::as_str), Some("ordinary value"));
    assert_eq!(
        env.get("PATH").map(String::as_str),
        Some("/usr/local/bin:/bin")
    );
    let dummy = env.get("GH_TOKEN").expect("dummy").clone();
    assert_ne!(dummy, D732_TOKEN);
    assert!(env.values().all(|value| !value.contains(D732_TOKEN)));

    // A later command or shell snapshot carrying only a copy loses it too.
    let mut later = d732_env(&[("AUTH_HEADER", &header)]);
    broker.virtualize_child_env(&mut later);
    assert_eq!(later.get("AUTH_HEADER"), None);
    let mut snapshot = d732_env(&[("AUTH_HEADER", &header), ("SAFE", "ordinary value")]);
    broker.virtualize_snapshot_env(&mut snapshot, /*environment_id*/ None);
    assert_eq!(snapshot.get("AUTH_HEADER"), None);
    assert_eq!(
        snapshot.get("SAFE").map(String::as_str),
        Some("ordinary value")
    );

    // Removing copies does not stop the dummy from being honored.
    assert_eq!(d732_route(&broker, &dummy), Ok(()));
}

/// D732: a key held only as an isolated record (no raw value in this
/// process) is found by fingerprint and its copy removed.
#[cfg(any(unix, windows))]
#[test]
fn d732_copy_of_an_isolated_key_is_removed_by_fingerprint() {
    let broker = unavailable_isolated_broker();
    register_gh_token(&broker, D732_TOKEN);
    assert!(!broker.holds_raw_value(D732_TOKEN));
    let header = format!("Bearer {D732_TOKEN}");
    let mut env = d732_env(&[("AUTH_HEADER", &header), ("SAFE", "ordinary value")]);
    broker.virtualize_child_env(&mut env);
    assert_eq!(env.get("AUTH_HEADER"), None);
    assert_eq!(env.get("SAFE").map(String::as_str), Some("ordinary value"));
}

/// D732: a held key shorter than 8 bytes is not looked for (and does not
/// make the check fail), so ordinary values containing it stay.
#[test]
fn d732_short_keys_are_not_looked_for() {
    let broker = CredentialBroker::new(/*enabled*/ true).with_main_engine();
    register_gh_token(&broker, SHORT_GH_TOKEN);
    {
        let mut state = broker.hardened.write_state();
        let record = &mut state.credentials[0];
        record.secret = RecordSecret::Raw(Zeroizing::new("tok1234".to_string()));
        record.value_len = "tok1234".len();
    }
    let mut env = d732_env(&[("NOTE", "uses tok1234 here")]);
    broker.virtualize_child_env(&mut env);
    assert_eq!(
        env.get("NOTE").map(String::as_str),
        Some("uses tok1234 here")
    );
    assert!(!broker.hardened.read_state().env_copy_check_failed);
}

/// D732: a value that cannot be checked fails closed: copies found are still
/// removed, and no request is given a key from then on, even after a later
/// environment checks cleanly.
#[test]
fn d732_unchecked_env_fails_closed_and_stays_failed() {
    let broker = CredentialBroker::new(/*enabled*/ true).with_main_engine();
    let header = format!("Bearer {D732_TOKEN}");
    let mut env = d732_env(&[("GH_TOKEN", D732_TOKEN)]);
    broker.virtualize_child_env(&mut env);
    let dummy = env["GH_TOKEN"].clone();
    assert_eq!(d732_route(&broker, &dummy), Ok(()));

    let large = "x".repeat(codex_secret_broker::output_gate::MAX_SCAN_BYTES + 1);
    let mut env = d732_env(&[("LARGE", &large), ("AUTH_HEADER", &header)]);
    broker.virtualize_child_env(&mut env);
    assert_eq!(env.get("AUTH_HEADER"), None);
    assert!(env.contains_key("LARGE"));
    let refused =
        Err(crate::credential_broker::ScopedCredentialInjectionError::IsolatedBrokerUnavailable);
    assert_eq!(d732_route(&broker, &dummy), refused);

    let mut clean = d732_env(&[("SAFE", "ordinary value")]);
    broker.virtualize_child_env(&mut clean);
    assert_eq!(d732_route(&broker, &dummy), refused);
}

/// D732/D733: with the broker off nothing is removed.
#[test]
fn d732_disabled_broker_leaves_the_env_unchanged() {
    let broker = CredentialBroker::new(/*enabled*/ false).with_main_engine();
    let header = format!("Bearer {D732_TOKEN}");
    let mut env = d732_env(&[("GH_TOKEN", D732_TOKEN), ("AUTH_HEADER", &header)]);
    let before = env.clone();
    broker.virtualize_child_env(&mut env);
    assert_eq!(env, before);
}
