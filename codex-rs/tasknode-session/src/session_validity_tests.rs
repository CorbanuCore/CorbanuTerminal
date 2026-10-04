use super::*;
use pretty_assertions::assert_eq;
use std::cell::Cell;
use std::future::pending;
use std::future::ready;
use std::pin::pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

const GOOD: &str = r#"{"ok":true,"accountId":"account","github":{"linked":true,"terminalBridgeEligible":true,"username":"new-handle"},"wallet":{},"counts":{},"sync":{},"server":{}}"#;
const CANARY: &str = "SYNTHETIC_SECRET_CANARY";

fn current() -> ValidityCurrent {
    ValidityCurrent {
        invocation: "observation-1".into(),
        generation: 7,
        scope: SessionScope::default(),
        session: ActiveSession {
            origin: "https://tasknode.example".into(),
            account_id: Some("account".into()),
            github_username: Some("old-handle".into()),
            terminal_token: CANARY.into(),
            expires_at: None,
        },
        now: "2026-09-11T12:00:00Z".parse().unwrap(),
        control: ObservationControl::Active,
    }
}

fn finish<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("offline response unexpectedly pending"),
    }
}

fn pipeline(response: Result<(u16, Vec<u8>), ClientError>) -> Result<DateTime<Utc>, ValidityHold> {
    let calls = Cell::new(0);
    let result = finish(check_identity_with(
        ValidityBinding::new(&current()).unwrap(),
        || Ok(current()),
        |_| {
            calls.set(calls.get() + 1);
            ready(response)
        },
    ));
    assert_eq!(calls.get(), 1);
    result?
        .consume_for_observation(|| Ok(current()))
        .map(|value| value.observed_at)
}

#[test]
fn null_and_future_expiry_require_exact_client_request_and_one_consumption() {
    for expiry in [None, Some("2026-09-12T00:00:00Z")] {
        let sample = || {
            let mut value = current();
            value.session.expires_at = expiry.map(str::to_owned);
            value
        };
        let calls = Cell::new(0);
        let checked = finish(check_identity_with(
            ValidityBinding::new(&sample()).unwrap(),
            || Ok(sample()),
            |client| {
                calls.set(calls.get() + 1);
                let request = client
                    .build_fixture_request(Method::GET, STATUS_PATH, /*body*/ None)
                    .unwrap();
                assert_eq!(request.method(), Method::GET);
                assert_eq!(
                    request.url().as_str(),
                    "https://tasknode.example/api/terminal/tasknode/status"
                );
                assert_eq!(request.url().query(), None);
                assert!(request.body().is_none());
                assert_eq!(
                    request.headers()["authorization"],
                    format!("Bearer {CANARY}")
                );
                ready(Ok((200, GOOD.as_bytes().to_vec())))
            },
        ))
        .unwrap();
        let observation = checked.consume_for_observation(|| Ok(sample())).unwrap();
        assert_eq!((observation.observed_at, calls.get()), (sample().now, 1));
        // `checked` and the original binding moved; neither has Clone or Copy.
        assert_eq!(sample().session.expires_at, expiry.map(str::to_owned));
    }
}

#[test]
fn strict_raw_identity_projection_rejects_missing_types_duplicates_and_false_facts() {
    let fields = [
        ("\"ok\":true", "\"ok\":\"true\""),
        ("\"accountId\":\"account\"", "\"accountId\":123"),
        ("\"linked\":true", "\"linked\":1"),
        (
            "\"terminalBridgeEligible\":true",
            "\"terminalBridgeEligible\":null",
        ),
        ("\"username\":\"new-handle\"", "\"username\":false"),
        (
            "\"github\":{\"linked\":true,\"terminalBridgeEligible\":true,\"username\":\"new-handle\"}",
            "\"github\":[]",
        ),
    ];
    for (field, wrong_type) in fields {
        let missing = GOOD
            .replace(&format!("{field},"), "")
            .replace(&format!(",{field}"), "");
        for raw in [
            missing,
            GOOD.replace(field, wrong_type),
            GOOD.replace(field, &format!("{field},{field}")),
            GOOD.replace(field, &format!("{field},{wrong_type}")),
        ] {
            assert_eq!(
                pipeline(Ok((200, raw.into_bytes()))),
                Err(ValidityHold::InvalidIdentity)
            );
        }
    }
    for (from, to) in [
        ("\"ok\":true", "\"ok\":false"),
        ("\"linked\":true", "\"linked\":false"),
        (
            "\"terminalBridgeEligible\":true",
            "\"terminalBridgeEligible\":false",
        ),
        ("\"account\"", "\"other\""),
        ("\"account\"", "\"\""),
        ("\"account\"", "null"),
        ("\"username\":\"new-handle\"", "\"username\":null"),
        (
            "{\"linked\":true,\"terminalBridgeEligible\":true,\"username\":\"new-handle\"}",
            "[true,true,\"new-handle\"]",
        ),
    ] {
        assert_eq!(
            pipeline(Ok((200, GOOD.replace(from, to).into_bytes()))),
            Err(ValidityHold::InvalidIdentity)
        );
    }
    for raw in [
        "",
        "{",
        "null",
        "1",
        "true",
        "[]",
        "[true,\"account\",{}]",
        "{}",
        "{}{}",
    ] {
        assert_eq!(
            pipeline(Ok((200, raw.as_bytes().to_vec()))),
            Err(ValidityHold::InvalidIdentity)
        );
    }
    assert_eq!(
        pipeline(Ok((200, vec![0xff]))),
        Err(ValidityHold::InvalidIdentity)
    );
    // Unknown data is not a wallet/entitlement/schema assertion, even if duplicated.
    let ignored = GOOD.replace(
        "\"wallet\":{}",
        "\"wallet\":null,\"wallet\":{\"x\":1,\"x\":2}",
    );
    assert_eq!(pipeline(Ok((200, ignored.into_bytes()))), Ok(current().now));
    let mut bounded = GOOD.as_bytes().to_vec();
    bounded.resize(RESPONSE_LIMIT, b' ');
    assert_eq!(pipeline(Ok((200, bounded.clone()))), Ok(current().now));
    bounded.push(b' ');
    assert_eq!(pipeline(Ok((200, bounded))), Err(ValidityHold::Unreadable));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Before,
    During,
    Consume,
}

// Exercise every mutation at all three boundaries through the shared pipeline.
fn held_at_every_phase(change: fn(&mut ValidityCurrent), expected: ValidityHold) {
    for phase in [Phase::Before, Phase::During, Phase::Consume] {
        let changed = Cell::new(phase == Phase::Before);
        let calls = Cell::new(0);
        let sample = || {
            let mut value = current();
            if changed.get() {
                change(&mut value);
            }
            Ok(value)
        };
        let checked = finish(check_identity_with(
            ValidityBinding::new(&current()).unwrap(),
            sample,
            |_| {
                calls.set(calls.get() + 1);
                changed.set(phase == Phase::During);
                ready(Ok((200, GOOD.as_bytes().to_vec())))
            },
        ));
        changed.set(true);
        let outcome = checked
            .and_then(|value| value.consume_for_observation(sample))
            .map(|_| ());
        assert_eq!(outcome.as_ref().err(), Some(&expected));
        assert_eq!(calls.get(), usize::from(phase != Phase::Before));
    }
}

#[test]
fn all_binding_mutations_hold_before_during_and_at_consumption() {
    let mutations: &[fn(&mut ValidityCurrent)] = &[
        |v| v.invocation = "other".into(),
        |v| v.generation += 1,
        |v| v.scope = SessionScope::for_profile("default"),
        |v| v.session.account_id = Some("other".into()),
        |v| v.session.origin = "https://other.example".into(),
        |v| v.session.terminal_token = "rotated".into(),
        |v| v.session.expires_at = Some("2026-09-12T00:00:00Z".into()),
        |v| v.invocation.clear(),
        |v| v.scope = SessionScope::for_profile(""),
        |v| v.session.account_id = None,
        |v| v.session.account_id = Some("".into()),
        |v| v.session.terminal_token.clear(),
        |v| v.session.origin = "http://localhost".into(),
        |v| v.session.origin.push('/'),
        |v| v.session.origin = "https://user:secret@tasknode.example".into(),
    ];
    for change in mutations {
        held_at_every_phase(*change, ValidityHold::BindingMismatch);
    }
}

#[test]
fn expiry_clock_and_named_control_holds_cover_every_boundary() {
    held_at_every_phase(
        |v| v.control = ObservationControl::Cancelled,
        ValidityHold::Cancelled,
    );
    held_at_every_phase(
        |v| v.control = ObservationControl::SessionChanged,
        ValidityHold::SessionChanged,
    );
    held_at_every_phase(
        |v| v.now -= chrono::Duration::seconds(1),
        ValidityHold::BackwardTime,
    );
    held_at_every_phase(
        |v| v.session.expires_at = Some("".into()),
        ValidityHold::InvalidExpiry,
    );
    held_at_every_phase(
        |v| v.session.expires_at = Some("invalid".into()),
        ValidityHold::InvalidExpiry,
    );
    held_at_every_phase(
        |v| v.session.expires_at = Some(v.now.to_rfc3339()),
        ValidityHold::KnownExpiry,
    );
    for phase in [Phase::During, Phase::Consume] {
        let crossed = Cell::new(false);
        let sample = || {
            let mut value = current();
            value.session.expires_at = Some("2026-09-11T12:00:01Z".into());
            if crossed.get() {
                value.now += chrono::Duration::seconds(1);
            }
            Ok(value)
        };
        let checked = finish(check_identity_with(
            ValidityBinding::new(&sample().unwrap()).unwrap(),
            sample,
            |_| {
                crossed.set(phase == Phase::During);
                ready(Ok((200, GOOD.as_bytes().to_vec())))
            },
        ));
        crossed.set(true);
        assert_eq!(
            checked
                .and_then(|value| value.consume_for_observation(sample))
                .err(),
            Some(ValidityHold::KnownExpiry)
        );
    }
}

#[test]
fn status_and_transport_diagnostics_are_fixed_and_redacted() {
    for status in [
        201, 202, 204, 300, 301, 302, 303, 307, 308, 401, 402, 403, 404, 409, 429, 500, 502, 503,
    ] {
        let held = pipeline(Ok((status, CANARY.as_bytes().to_vec()))).unwrap_err();
        assert_eq!(held, ValidityHold::Http(status));
        assert_eq!(
            held.to_string(),
            format!("Identity rejected at check time (HTTP {status}).")
        );
    }
    for (error, expected, display) in [
        (
            ClientError::Transport(CANARY.into()),
            ValidityHold::Transport,
            "Identity observation transport failed.",
        ),
        (
            ClientError::InvalidResponse(200),
            ValidityHold::Unreadable,
            "Identity response unreadable.",
        ),
        (
            ClientError::InvalidOrigin,
            ValidityHold::BindingMismatch,
            "Identity observation binding mismatch.",
        ),
        (
            ClientError::OriginMismatch {
                saved: CANARY.into(),
                requested: CANARY.into(),
            },
            ValidityHold::BindingMismatch,
            "Identity observation binding mismatch.",
        ),
    ] {
        let held = pipeline(Err(error)).unwrap_err();
        assert_eq!((&held, held.to_string()), (&expected, display.to_owned()));
        assert!(!format!("{held:?} {held}").contains(CANARY));
    }
    let held = pipeline(Ok((200, format!("{{\"ok\":\"{CANARY}\"}}").into_bytes()))).unwrap_err();
    assert_eq!(
        format!("{held:?} {held}"),
        "InvalidIdentity Invalid server identity."
    );
}

#[test]
fn unavailable_authority_and_dropped_pending_request_produce_no_proof() {
    // Exercise the actual async entry without permitting it to reach transport.
    let cancelled = finish(check_identity(
        ValidityBinding::new(&current()).unwrap(),
        || Err(ValidityHold::Cancelled),
    ));
    assert_eq!(cancelled.err(), Some(ValidityHold::Cancelled));
    let calls = Cell::new(0);
    let result = finish(check_identity_with(
        ValidityBinding::new(&current()).unwrap(),
        || Err(ValidityHold::SessionChanged),
        |_| {
            calls.set(calls.get() + 1);
            ready(Ok((200, GOOD.as_bytes().to_vec())))
        },
    ));
    assert_eq!(
        (result.err(), calls.get()),
        (Some(ValidityHold::SessionChanged), 0)
    );
    let samples = Cell::new(0);
    let mut future = Box::pin(check_identity_with(
        ValidityBinding::new(&current()).unwrap(),
        || {
            samples.set(samples.get() + 1);
            Ok(current())
        },
        |_| {
            calls.set(calls.get() + 1);
            pending::<Result<(u16, Vec<u8>), ClientError>>()
        },
    ));
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    drop(future);
    assert_eq!((calls.get(), samples.get()), (1, 1));
}

#[test]
fn receipt_time_and_unavailable_current_authority_are_rechecked() {
    for unavailable_at in [2, 3, 4] {
        let samples = Cell::new(0);
        let sample = || {
            samples.set(samples.get() + 1);
            if samples.get() == unavailable_at {
                return Err(ValidityHold::SessionChanged);
            }
            let mut value = current();
            if samples.get() == 2 {
                value.now += chrono::Duration::seconds(1);
            }
            Ok(value)
        };
        let checked = finish(check_identity_with(
            ValidityBinding::new(&current()).unwrap(),
            sample,
            |_| ready(Ok((200, GOOD.as_bytes().to_vec()))),
        ));
        let held = checked
            .and_then(|value| value.consume_for_observation(sample))
            .err();
        let expected = if unavailable_at == 4 {
            ValidityHold::BackwardTime
        } else {
            ValidityHold::SessionChanged
        };
        assert_eq!(held, Some(expected));
    }
}
