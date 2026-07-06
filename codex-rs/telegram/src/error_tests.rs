use std::time::Duration;

use pretty_assertions::assert_eq;

use codex_telegram::error::PollingBackoff;
use codex_telegram::error::is_http_409_conflict;
use codex_telegram::error::redact_secret;

#[test]
fn backoff_doubles_and_caps() {
    let mut backoff = PollingBackoff::new(Duration::from_secs(1), Duration::from_secs(4), 10);

    assert_eq!(
        backoff.record_failure().expect("failure 1"),
        Duration::from_secs(1)
    );
    assert_eq!(
        backoff.record_failure().expect("failure 2"),
        Duration::from_secs(2)
    );
    assert_eq!(
        backoff.record_failure().expect("failure 3"),
        Duration::from_secs(4)
    );
    assert_eq!(
        backoff.record_failure().expect("failure 4"),
        Duration::from_secs(4)
    );
    backoff.record_success();
    assert_eq!(
        backoff.record_failure().expect("reset"),
        Duration::from_secs(1)
    );
}

#[test]
fn conflict_detection_checks_error_text() {
    let err = anyhow::anyhow!("Telegram returned HTTP 409 Conflict");

    assert_eq!(is_http_409_conflict(err.as_ref()), true);
}

#[test]
fn redaction_replaces_secret_material() {
    assert_eq!(
        redact_secret("token abc123 failed", "abc123"),
        "token [REDACTED] failed"
    );
}
