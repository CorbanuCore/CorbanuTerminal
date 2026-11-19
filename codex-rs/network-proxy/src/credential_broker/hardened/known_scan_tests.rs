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
