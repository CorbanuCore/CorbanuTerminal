use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

// Literal accepted Python golden: no file inclusion or live target identity.
const EVENT: &str = r#"{"content": "PF-80-S01: worker reports working. Recovery test passed; human acceptance still pending.\nProgress observation only; no acceptance or task-completion claim.", "coverage": "manager_observed_worker_report_not_independent_acceptance", "goal": {"active": true, "id": "PF-80-S01", "status": "reported_working"}, "id": "cc-cf3440216007cdf72d47f5467dbdc482d916b96d76635bc3537a222b83d67373", "instanceId": "corbanu-control", "kind": "goal", "occurredAt": "2026-09-11T12:00:00.000Z", "repository": {"branch": "test/fixture", "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "label": "Corbanu Terminal"}, "sequence": 0, "sessionId": "run-1", "taskIds": ["task-a", "task-b"], "turnId": "PF-80-S01", "workspaceId": "synthetic-workspace"}"#;
const ID: &str = "cc-cf3440216007cdf72d47f5467dbdc482d916b96d76635bc3537a222b83d67373";
const HASH: &str = "5c8cd79f2b097a9acc262c92ff159be7d12c4d41eefa9f49e15f3d09a259b4ad";
const TOKEN: &str = "synthetic-terminal-credential-canary";
const API_KEY: &str = "synthetic-api-credential-canary";

struct Transport {
    calls: Vec<(&'static str, &'static str, Value)>,
    response: Option<Result<Response, ExchangeFailure>>,
    during: fn(&mut Current),
}

impl FixtureExchange for Transport {
    fn exchange(
        &mut self,
        endpoint: &'static str,
        approved_payload: &Value,
        current: &mut Current,
    ) -> Result<Response, ExchangeFailure> {
        let mut wire = approved_payload.clone();
        wire["apiKey"] = json!(API_KEY);
        self.calls.push(("POST", endpoint, wire));
        (self.during)(current);
        self.response.take().expect("fixture exchange called twice")
    }
}

fn fixture() -> (Attempt, Current, Transport) {
    let now = "2026-09-11T12:00:00Z".parse().unwrap();
    let session = ActiveSession {
        origin: "https://TASKNODE.example:443/".into(),
        account_id: Some("synthetic-account".into()),
        github_username: None,
        terminal_token: TOKEN.into(),
        expires_at: Some("2026-09-11T14:00:00Z".into()),
    };
    let binding = Binding {
        invocation: "synthetic-invocation".into(),
        profile: SessionScope::for_profile("synthetic-profile"),
        account: "synthetic-account".into(),
        origin: "https://tasknode.example".into(),
        client_identity: Client::for_session(&session, "https://tasknode.example")
            .unwrap()
            .identity(),
        event_id: ID.into(),
        payload_sha256: HASH.into(),
        sprint: "PF-80-S01".into(),
        workspace: "synthetic-workspace".into(),
        tasks: vec!["task-a".into(), "task-b".into()],
    };
    let preparation = delivery_goal::prepare(
        EVENT,
        Selection {
            event_id: ID,
            payload_sha256: HASH,
            sprint_id: &binding.sprint,
            workspace_id: &binding.workspace,
            task_ids: &["task-a", "task-b"],
        },
        now,
    )
    .unwrap();
    let facts = REQUIRED
        .iter()
        .map(|kind| FixtureFact {
            kind: *kind,
            binding: binding.clone(),
            knowledge: Knowledge::Verified,
        })
        .collect();
    let attempt = Attempt {
        binding: binding.clone(),
        preparation,
        prepared_at: now,
        consumed: false,
    };
    let current = Current {
        binding,
        session,
        now,
        cancelled: false,
        event_json: EVENT.into(),
        facts,
    };
    let transport = Transport {
        calls: vec![],
        during: |_| {},
        response: Some(Ok(Response {
            status: 200,
            body: json!({"ok": true, "id": ID, "summaryState": "not_applicable", "replayed": false}),
        })),
    };
    (attempt, current, transport)
}

fn expected(state: State) -> Outcome {
    Outcome {
        event_id: ID.into(),
        payload_sha256: HASH.into(),
        state,
    }
}

fn change_binding(binding: &mut Binding, field: usize) {
    match field {
        0 => binding.invocation.push('x'),
        1 => binding.profile = SessionScope::default(),
        2 => binding.account.push('x'),
        3 => binding.origin = "https://other.example".into(),
        4 => binding.client_identity.push('x'),
        5 => binding.event_id.push('x'),
        6 => binding.payload_sha256.push('x'),
        7 => binding.sprint = "PF-79-S01".into(),
        8 => binding.workspace.push('x'),
        9 => binding.tasks.push("task-c".into()),
        _ => panic!("unknown test field"),
    }
}

#[test]
fn success_records_exact_approved_payload_once_and_preserves_advisory() {
    let (mut attempt, mut current, mut transport) = fixture();
    let event: Value = serde_json::from_str(EVENT).unwrap();
    assert_eq!(
        attempt.run(&mut current, &mut transport),
        expected(State::GoalRecorded)
    );
    assert_eq!(
        transport.calls,
        vec![(
            "POST",
            "/api/terminal/tasknode/campaign-tracker/events",
            json!({"event": event, "apiKey": API_KEY})
        )]
    );
    let mut recorded = transport.calls[0].2.clone();
    recorded.as_object_mut().unwrap().remove("apiKey");
    assert_eq!(&recorded, attempt.preparation.payload());
    assert_eq!(attempt.preparation.payload_sha256(), HASH);
    assert_eq!(
        attempt.preparation.blockers(),
        &["live_authority_entitlement_owner_and_target_lifecycle_unverified"]
    );
    assert_eq!(
        attempt.run(&mut current, &mut transport),
        expected(State::AlreadyConsumed)
    );
    assert_eq!(transport.calls.len(), 1);
    // This becomes ambiguous (and fails compilation) if Attempt gains Clone.
    trait AmbiguousIfClone<A> {
        fn check() {}
    }
    impl<T: ?Sized> AmbiguousIfClone<()> for T {}
    struct Cloned;
    impl<T: ?Sized + Clone> AmbiguousIfClone<Cloned> for T {}
    let _ = <Attempt as AmbiguousIfClone<_>>::check;
}

#[test]
fn explicit_default_and_named_profiles_do_not_share_the_client_fence() {
    for profile in [
        SessionScope::default(),
        SessionScope::for_profile("default"),
        SessionScope::for_profile("other"),
    ] {
        let (mut attempt, mut current, mut transport) = fixture();
        // Client identity is unchanged; the separate profile fence must reject.
        current.binding.profile = profile.clone();
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::NotAttempted(Hold::Identity))
        );
        assert!(transport.calls.is_empty());
        let (mut attempt, mut current, mut transport) = fixture();
        attempt.binding.profile = profile.clone();
        current.binding.profile = profile.clone();
        for fact in &mut current.facts {
            fact.binding.profile = profile.clone();
        }
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::GoalRecorded)
        );
    }
}

#[test]
fn every_invocation_binding_must_match() {
    for field in 0..10 {
        let (mut attempt, mut current, mut transport) = fixture();
        change_binding(&mut current.binding, field);
        let result = attempt.run(&mut current, &mut transport);
        assert_eq!(result, expected(State::NotAttempted(Hold::Identity)));
        assert!(transport.calls.is_empty());
    }
}

#[test]
fn empty_scope_and_noncanonical_approved_origins_fail_closed() {
    let changes: [fn(&mut Binding); 7] = [
        |b| b.invocation.clear(),
        |b| b.profile = SessionScope::for_profile(""),
        |b| b.account.clear(),
        |b| b.origin.clear(),
        |b| b.origin = "http://localhost:8080".into(),
        |b| b.origin = "https://tasknode.example/".into(),
        |b| b.origin = "https://tasknode.example/path".into(),
    ];
    for change in changes {
        let (mut attempt, mut current, mut transport) = fixture();
        change(&mut attempt.binding);
        current.binding = attempt.binding.clone();
        assert_eq!(
            attempt.run(&mut current, &mut transport).state,
            State::NotAttempted(Hold::Identity)
        );
        assert!(transport.calls.is_empty());
    }
}

#[test]
fn actual_client_session_fences_reject_missing_or_changed_identity() {
    let changes: [fn(&mut Current); 9] = [
        |c| c.session.account_id = None,
        |c| c.session.account_id = Some(String::new()),
        |c| c.session.account_id = Some("wrong".into()),
        |c| c.session.origin.clear(),
        |c| c.session.origin = "https://other.example".into(),
        |c| c.session.origin = "http://localhost:8080".into(),
        |c| c.session.terminal_token.clear(),
        |c| c.session.terminal_token = "rotated-synthetic-canary".into(),
        |c| c.session.origin = "https://user:secret@tasknode.example".into(),
    ];
    for change in changes {
        let (mut attempt, mut current, mut transport) = fixture();
        change(&mut current);
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::NotAttempted(Hold::Identity))
        );
        assert!(transport.calls.is_empty());
    }
}

#[test]
fn all_seven_preflight_facts_require_separate_known_matching_evidence() {
    for index in 0..REQUIRED.len() {
        for mode in 0..14 {
            let (mut attempt, mut current, mut transport) = fixture();
            match mode {
                0 => {
                    current.facts.remove(index);
                }
                1 => current.facts[index].knowledge = Knowledge::Unknown,
                2 => current.facts[index].knowledge = Knowledge::Denied,
                3 => current.facts[index].kind = REQUIRED[(index + 1) % REQUIRED.len()],
                field => change_binding(&mut current.facts[index].binding, field - 4),
            }
            assert_eq!(
                attempt.run(&mut current, &mut transport),
                expected(State::NotAttempted(Hold::Preflight))
            );
            assert!(transport.calls.is_empty());
        }
    }
}

#[test]
fn recomputes_selection_digest_mapping_and_observation_before_exchange() {
    for field in 5..10 {
        let (mut attempt, mut current, mut transport) = fixture();
        // Change the proposed selection on both sides to exercise preparation,
        // rather than only the outer invocation-equality gate.
        change_binding(&mut attempt.binding, field);
        current.binding = attempt.binding.clone();
        assert_eq!(
            attempt.run(&mut current, &mut transport).state,
            State::NotAttempted(Hold::Preparation)
        );
        assert!(transport.calls.is_empty());
    }
    for json in [
        EVENT.replace("Recovery", "Changed"),
        "{}".into(),
        EVENT.replace("test/fixture", TOKEN),
    ] {
        let (mut attempt, mut current, mut transport) = fixture();
        current.event_json = json;
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::NotAttempted(Hold::Preparation))
        );
        assert!(transport.calls.is_empty());
    }
    for minutes in [-1, 46] {
        let (mut attempt, mut current, mut transport) = fixture();
        current.now += chrono::Duration::minutes(minutes);
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::NotAttempted(Hold::Stale))
        );
        assert!(transport.calls.is_empty());
    }
}

#[test]
fn expiry_is_rechecked_and_unknown_metadata_is_not_validity_evidence() {
    for (expiry, reason) in [
        (Some("2026-09-11T12:00:00Z"), Hold::Expired),
        (Some("invalid"), Hold::ExpiryUnknown),
        (None, Hold::ExpiryUnknown),
    ] {
        let (mut attempt, mut current, mut transport) = fixture();
        current.session.expires_at = expiry.map(str::to_owned);
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::NotAttempted(reason))
        );
        assert!(transport.calls.is_empty());
    }
}

#[test]
fn cancel_before_exchange_consumes_attempt_without_a_call() {
    let (mut attempt, mut current, mut transport) = fixture();
    current.cancelled = true;
    assert_eq!(
        attempt.run(&mut current, &mut transport),
        expected(State::NotAttempted(Hold::Cancelled))
    );
    current.cancelled = false;
    assert_eq!(
        attempt.run(&mut current, &mut transport),
        expected(State::AlreadyConsumed)
    );
    assert!(transport.calls.is_empty());
}

#[test]
fn changes_during_exchange_are_unknown_and_never_retry() {
    let changes: [fn(&mut Current); 10] = [
        |c| c.cancelled = true,
        |c| c.session.terminal_token.push('x'),
        |c| c.session.account_id = Some("rotated".into()),
        |c| c.session.origin = "https://rotated.example".into(),
        |c| c.binding.profile = SessionScope::default(),
        |c| c.now += chrono::Duration::hours(3),
        |c| c.event_json = "{}".into(),
        |c| c.binding.payload_sha256.push('x'),
        |c| c.facts.clear(),
        |c| c.session.expires_at = None,
    ];
    for change in changes {
        let (mut attempt, mut current, mut transport) = fixture();
        transport.during = change;
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::OutcomeUnknown)
        );
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::AlreadyConsumed)
        );
        assert_eq!(transport.calls.len(), 1);
    }
}

#[test]
fn timeout_and_transport_failure_preserve_original_identity_without_retry() {
    for error in [ExchangeFailure::Timeout, ExchangeFailure::Transport] {
        let (mut attempt, mut current, mut transport) = fixture();
        transport.response = Some(Err(error));
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::OutcomeUnknown)
        );
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::AlreadyConsumed)
        );
        assert_eq!(transport.calls.len(), 1);
    }
}

#[test]
fn http_failures_are_typed_and_do_not_export_credential_canaries() {
    for status in [301, 302, 307, 308, 401, 402, 403, 409, 429, 500, 503] {
        let (mut attempt, mut current, mut transport) = fixture();
        transport.response = Some(Ok(Response {
            status,
            body: json!({"message": TOKEN, "error": API_KEY}),
        }));
        let outcome = attempt.run(&mut current, &mut transport);
        let state = if status == 401 {
            State::RelinkCurrentProfile
        } else {
            State::Held(Hold::Http(status))
        };
        assert_eq!(outcome, expected(state));
        let diagnostic = format!("{outcome:?}");
        for secret in [TOKEN, API_KEY] {
            assert!(!diagnostic.contains(secret));
        }
        assert_eq!(
            attempt.run(&mut current, &mut transport),
            expected(State::AlreadyConsumed)
        );
        assert_eq!(transport.calls.len(), 1);
    }
}

#[test]
fn receipt_requires_explicit_ok_exact_id_and_goal_summary() {
    let bodies = [
        json!({"id": ID, "summaryState": "not_applicable"}),
        json!({"ok": false, "id": ID, "summaryState": "not_applicable"}),
        json!({"ok": "true", "id": ID, "summaryState": "not_applicable"}),
        json!({"ok": true, "id": TOKEN, "summaryState": "not_applicable"}),
        json!({"ok": true, "eventId": ID, "summaryState": "not_applicable"}),
        json!({"ok": true, "id": ID}),
        json!(null),
        json!([]),
    ];
    for body in bodies.into_iter().chain(["deleted", "pending", "ready", "unavailable", "unknown", ""].map(|summary| {
        json!({"ok": true, "id": ID, "summaryState": summary, "replayed": true, "message": API_KEY})
    })) {
        let (mut attempt, mut current, mut transport) = fixture();
        transport.response = Some(Ok(Response { status: 200, body }));
        let outcome = attempt.run(&mut current, &mut transport);
        assert_eq!(outcome, expected(State::Held(Hold::Receipt)));
        for secret in [TOKEN, API_KEY] { assert!(!format!("{outcome:?}").contains(secret)); }
        assert_eq!(transport.calls.len(), 1);
    }
    // Replay is only a response property; it supplies none of the preflight facts.
    let (mut attempt, mut current, mut transport) = fixture();
    transport.response = Some(Ok(Response {
        status: 200,
        body: json!({"ok": true, "id": ID, "summaryState": "not_applicable", "replayed": true}),
    }));
    current.facts.clear();
    assert_eq!(
        attempt.run(&mut current, &mut transport),
        expected(State::NotAttempted(Hold::Preflight))
    );
    assert!(transport.calls.is_empty());
}
