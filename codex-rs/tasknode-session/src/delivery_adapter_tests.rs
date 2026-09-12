use super::*;
use crate::tests::MemoryStore;
use crate::*;
use pretty_assertions::assert_eq;
use serde_json::json;

const EVENT: &str = r#"{"content": "PF-80-S01: worker reports working. Recovery test passed; human acceptance still pending.\nProgress observation only; no acceptance or task-completion claim.", "coverage": "manager_observed_worker_report_not_independent_acceptance", "goal": {"active": true, "id": "PF-80-S01", "status": "reported_working"}, "id": "cc-cf3440216007cdf72d47f5467dbdc482d916b96d76635bc3537a222b83d67373", "instanceId": "corbanu-control", "kind": "goal", "occurredAt": "2026-09-11T12:00:00.000Z", "repository": {"branch": "test/fixture", "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "label": "Corbanu Terminal"}, "sequence": 0, "sessionId": "run-1", "taskIds": ["task-a", "task-b"], "turnId": "PF-80-S01", "workspaceId": "synthetic-workspace"}"#;
const HASH: &str = "5c8cd79f2b097a9acc262c92ff159be7d12c4d41eefa9f49e15f3d09a259b4ad";
const TOKEN: &str = "synthetic-terminal-credential-canary";
const KEY: &str = "synthetic-api-credential-canary";

fn fixture(scope: SessionScope) -> (Attempt, Current, Vec<Raw>, Operator) {
    let store = MemoryStore::default();
    let saved = ActiveSession {
        origin: "https://TASKNODE.example:443/".into(),
        account_id: Some("synthetic-account".into()),
        github_username: None,
        terminal_token: TOKEN.into(),
        expires_at: Some((Utc::now() + chrono::Duration::days(1)).to_rfc3339()),
    };
    promote_active_scoped_to_store(&store, &scope, &saved).unwrap();
    let session = recovery::resolve_fixture(&store, &scope, Some(&saved.origin), |_, _| {
        panic!("unexpected exchange")
    })
    .unwrap();
    let event: Value = serde_json::from_str(EVENT).unwrap();
    let b = Binding {
        invocation: "fixture-run".into(),
        profile: scope,
        account: "synthetic-account".into(),
        origin: "https://tasknode.example".into(),
        client_identity: Client::for_session(&session, &saved.origin)
            .unwrap()
            .identity(),
        event_id: event["id"].as_str().unwrap().into(),
        payload_sha256: HASH.into(),
        sprint: "PF-80-S01".into(),
        workspace: "synthetic-workspace".into(),
        tasks: vec!["task-a".into(), "task-b".into()],
    };
    let now = "2026-09-11T12:00:00Z".parse().unwrap();
    let preparation = delivery_goal::prepare(
        EVENT,
        Selection {
            event_id: &b.event_id,
            payload_sha256: HASH,
            sprint_id: &b.sprint,
            workspace_id: &b.workspace,
            task_ids: &["task-a", "task-b"],
        },
        now,
    )
    .unwrap();
    let mut raw = vec![
        (STATUS.into(), json!({"ok":true,"accountId":b.account})),
        (
            ENROLLMENT.into(),
            json!({"ok":true,"accountId":b.account,"enrollments":[{"workspace_id":b.workspace,"enabled":true}]}),
        ),
        (
            GATEWAY.into(),
            json!({"corbanuApi":{"balanceMicrousd":"1"}}),
        ),
    ];
    raw.extend(b.tasks.iter().map(|id| {
        (
            task_path(id),
            json!({"ok":true,"task":{"id":id,"fullId":id,"taskId":id,"statusKey":"proposed"}}),
        )
    }));
    let raw = raw
        .into_iter()
        .map(|(path, body)| Raw {
            binding: b.clone(),
            method: Method::GET,
            key_handle: (path == GATEWAY).then(|| key_handle(KEY)),
            path,
            status: 200,
            bytes: serde_json::to_vec(&body).unwrap(),
        })
        .collect();
    let operator = Operator {
        binding: b.clone(),
        supported_statuses: vec!["proposed".into()],
        one_event: Knowledge::Verified,
        posting: Knowledge::Verified,
    };
    let attempt = Attempt {
        binding: b.clone(),
        preparation,
        prepared_at: now,
        consumed: false,
    };
    let current = Current {
        binding: b,
        session,
        now,
        cancelled: false,
        event_json: EVENT.into(),
        facts: vec![],
    };
    (attempt, current, raw, operator)
}

fn build(current: &mut Current, raw: &[Raw], operator: &Operator) -> BuildOnly {
    let receipt = json!({"ok":true,"id":current.binding.event_id,"summaryState":"not_applicable"});
    BuildOnly::prepare(
        current,
        raw,
        operator,
        KEY.into(),
        (200, serde_json::to_vec(&receipt).unwrap()),
    )
    .unwrap()
}

#[test]
fn schema_success_builds_real_request_with_approved_digest_and_no_execution() {
    for scope in [SessionScope::default(), SessionScope::for_profile("desk")] {
        let (mut attempt, mut current, raw, operator) = fixture(scope);
        let mut exchange = build(&mut current, &raw, &operator);
        assert_eq!(
            attempt.run(&mut current, &mut exchange).state,
            State::GoalRecorded
        );
        let request = exchange.request.as_ref().unwrap();
        assert_eq!(
            (request.method(), request.url().as_str()),
            (
                &Method::POST,
                "https://tasknode.example/api/terminal/tasknode/campaign-tracker/events"
            )
        );
        assert_eq!(
            request.headers()["authorization"],
            format!("Bearer {TOKEN}")
        );
        assert_eq!(request.headers()["content-type"], "application/json");
        let mut payload = attempt.preparation.payload().clone();
        payload["apiKey"] = json!(KEY);
        assert_eq!(
            request.body().unwrap().as_bytes().unwrap(),
            serde_json::to_vec(&payload).unwrap()
        );
        payload.as_object_mut().unwrap().remove("apiKey");
        assert_eq!(
            (&payload, attempt.preparation.payload_sha256()),
            (attempt.preparation.payload(), HASH)
        );
        assert!(!attempt.preparation.blockers().is_empty());
        assert_eq!(
            attempt.run(&mut current, &mut exchange).state,
            State::AlreadyConsumed
        );
    }
}

#[test]
fn raw_schema_and_source_fences_hold() {
    for index in [0, 1, 3, 4] {
        for body in [
            json!({}),
            json!({"ok":false}),
            json!({"ok":"true"}),
            json!([]),
        ] {
            let (_, current, mut raw, operator) = fixture(SessionScope::default());
            raw[index].bytes = serde_json::to_vec(&body).unwrap();
            assert!(derive_facts(&current, &raw, &operator, KEY).is_err());
        }
    }
    for index in 0..5 {
        for mutation in 0..8 {
            let (_, current, mut raw, operator) = fixture(SessionScope::default());
            match mutation {
                0 => raw[index].binding.profile = SessionScope::for_profile("other"),
                1 => raw[index].binding.account.push('x'),
                2 => raw[index].binding.client_identity.push('x'),
                3 => raw[index].method = Method::POST,
                4 => raw[index].path.push('/'),
                5 => raw[index].bytes = b"{".to_vec(),
                6 => raw[index].status = 302,
                7 => raw.push(raw[index].clone()),
                _ => unreachable!(),
            }
            assert!(derive_facts(&current, &raw, &operator, KEY).is_err());
        }
    }
}

#[test]
fn account_and_snake_case_enrollments_are_exact() {
    for rows in [
        json!([]),
        json!([{"workspaceId":"synthetic-workspace","enabled":true}]),
        json!([{"workspace_id":"other","enabled":true}]),
        json!([{"workspace_id":"synthetic-workspace","enabled":false}]),
        json!([{"workspace_id":"synthetic-workspace","enabled":"true"}]),
        json!([{"workspace_id":"synthetic-workspace","enabled":true},{"workspace_id":"synthetic-workspace","enabled":false}]),
    ] {
        let (_, current, mut raw, operator) = fixture(SessionScope::default());
        raw[1].bytes = serde_json::to_vec(
            &json!({"ok":true,"accountId":current.binding.account,"enrollments":rows}),
        )
        .unwrap();
        assert!(derive_facts(&current, &raw, &operator, KEY).is_err());
    }
    for index in [0, 1] {
        let (_, current, mut raw, operator) = fixture(SessionScope::default());
        let mut body: Value = serde_json::from_slice(&raw[index].bytes).unwrap();
        body["accountId"] = json!("other");
        raw[index].bytes = serde_json::to_vec(&body).unwrap();
        assert!(derive_facts(&current, &raw, &operator, KEY).is_err());
    }
}

#[test]
fn full_ids_and_lifecycle_cannot_be_inferred_from_display_or_ingest() {
    let (_, mut current, _, mut operator) = fixture(SessionScope::default());
    current.binding.tasks = vec!["123456789012/a ?".into()];
    operator.binding = current.binding.clone();
    for field in ["fullId", "taskId", "statusKey"] {
        for wrong in [Value::Null, json!("123456789012/b ?"), json!("unknown")] {
            let (_, _, original, _) = fixture(SessionScope::default());
            let mut raw = original[..3].to_vec();
            for item in &mut raw {
                item.binding = current.binding.clone();
            }
            let mut body = json!({"ok":true,"task":{"id":"123456789012","fullId":current.binding.tasks[0],"taskId":current.binding.tasks[0],"statusKey":"proposed"}});
            raw.push(Raw {
                binding: current.binding.clone(),
                method: Method::GET,
                path: task_path(&current.binding.tasks[0]),
                key_handle: None,
                status: 200,
                bytes: serde_json::to_vec(&body).unwrap(),
            });
            assert_eq!(
                raw[3].path,
                "/api/terminal/tasknode/tasks/123456789012%2Fa%20%3F"
            );
            assert!(derive_facts(&current, &raw, &operator, KEY).is_ok());
            body["task"][field] = wrong;
            raw[3].bytes = serde_json::to_vec(&body).unwrap();
            assert!(derive_facts(&current, &raw, &operator, KEY).is_err());
        }
    }
    let (mut attempt, mut current, raw, mut operator) = fixture(SessionScope::default());
    operator.supported_statuses.clear();
    let mut exchange = build(&mut current, &raw, &operator);
    assert_eq!(
        attempt.run(&mut current, &mut exchange).state,
        State::NotAttempted(Hold::Preflight)
    );
    assert!(exchange.request.is_none());
}

#[test]
fn exact_entitlement_periods_and_integer_funding() {
    let now = "2026-09-11T12:00:00Z".parse().unwrap();
    for (start, end, expected) in [
        ("11:00", "12:00", false),
        ("12:00", "13:00", true),
        ("12:01", "13:00", false),
    ] {
        let period = json!({"startsAt":format!("2026-09-11T{start}:00Z"),"endsAt":format!("2026-09-11T{end}:00Z")});
        for body in [
            json!({"period":period}),
            json!({"legacy":{"period":period}}),
            json!({"period":null,"legacy":{"period":period}}),
        ] {
            assert_eq!(entitlement(&body, now), expected);
        }
    }
    for (balance, expected) in [
        (json!("1"), true),
        (json!(1), true),
        (json!("999999999999999999999999999999999"), true),
        (json!("0"), false),
        (json!(-1), false),
        (json!("-1"), false),
        (json!("1.0"), false),
        (json!(1.5), false),
        (json!(true), false),
        (json!("garbage"), false),
        (json!(9007199254740992_u64), false),
    ] {
        assert_eq!(
            entitlement(
                &json!({"period":{},"corbanuApi":{"balanceMicrousd":balance}}),
                now
            ),
            expected
        );
    }
    assert!(!entitlement(
        &json!({"period":{},"legacy":{"period":{"startsAt":"2026-09-11T11:00:00Z","endsAt":"2026-09-11T13:00:00Z"}}}),
        now
    ));
    let (_, current, mut raw, operator) = fixture(SessionScope::default());
    raw[2].key_handle = Some(key_handle(TOKEN));
    assert!(derive_facts(&current, &raw, &operator, KEY).is_err());
    assert!(derive_facts(&current, &raw, &operator, TOKEN).is_err());
}

#[test]
fn engine_fences_cancel_expiry_and_changed_identity_before_and_after_build() {
    for change in [
        |c: &mut Current| c.cancelled = true,
        |c: &mut Current| c.binding.profile = SessionScope::for_profile("other"),
        |c: &mut Current| c.session.terminal_token.push('x'),
    ] {
        for after in [false, true] {
            let (mut attempt, mut current, raw, operator) = fixture(SessionScope::default());
            let mut exchange = build(&mut current, &raw, &operator);
            if after {
                exchange.during = change;
            } else {
                change(&mut current);
            }
            let state = attempt.run(&mut current, &mut exchange).state;
            assert!(if after {
                state == State::OutcomeUnknown
            } else {
                matches!(state, State::NotAttempted(_))
            });
            assert_eq!(exchange.request.is_some(), after);
        }
    }
    let (mut attempt, mut current, raw, operator) = fixture(SessionScope::default());
    current.session.expires_at = None;
    let mut exchange = build(&mut current, &raw, &operator);
    assert_eq!(
        attempt.run(&mut current, &mut exchange).state,
        State::NotAttempted(Hold::ExpiryUnknown)
    );
}

#[test]
fn strict_receipts_and_diagnostics_never_export_credentials() {
    for status in [200, 202, 302, 401, 402, 403, 409, 429, 500, 503] {
        for summary in ["deleted", "pending", "not_applicable"] {
            let (mut attempt, mut current, raw, operator) = fixture(SessionScope::default());
            let mut exchange = build(&mut current, &raw, &operator);
            exchange.receipt = (status, serde_json::to_vec(&json!({"ok":true,"id":current.binding.event_id,"summaryState":summary,"error":format!("{TOKEN} {KEY}")})).unwrap());
            let outcome = attempt.run(&mut current, &mut exchange);
            let expected = match status {
                401 => State::RelinkCurrentProfile,
                200..=299 if summary == "not_applicable" => State::GoalRecorded,
                200..=299 => State::Held(Hold::Receipt),
                _ => State::Held(Hold::Http(status)),
            };
            assert_eq!(&outcome.state, &expected);
            let diagnostic = format!("{outcome:?}");
            assert!(!diagnostic.contains(TOKEN) && !diagnostic.contains(KEY));
        }
    }
    for bytes in [
        b"{".as_slice(),
        b"[]",
        br#"{"id":"wrong","summaryState":"not_applicable"}"#,
    ] {
        let (mut attempt, mut current, raw, operator) = fixture(SessionScope::default());
        let mut exchange = build(&mut current, &raw, &operator);
        exchange.receipt.1 = bytes.to_vec();
        assert!(matches!(
            attempt.run(&mut current, &mut exchange).state,
            State::OutcomeUnknown | State::Held(Hold::Receipt)
        ));
    }
}

#[test]
fn inner_resolver_uses_injected_codec_and_preserves_failed_profile_state() {
    for scope in [SessionScope::default(), SessionScope::for_profile("desk")] {
        for account in ["synthetic-account", "other"] {
            let (_, current, _, _) = fixture(scope.clone());
            let store = MemoryStore::default();
            promote_active_scoped_to_store(&store, &scope, &current.session).unwrap();
            let pending = PendingLink {
                origin: current.binding.origin.clone(),
                request_id: "pending".into(),
                poll_token: "synthetic-poll".into(),
                verification_url: "https://tasknode.example/choose".into(),
                started_at: None,
            };
            save_pending_scoped_to_store(&store, &scope, &pending).unwrap();
            let before = load_scoped_from_store(&store, &scope).unwrap();
            let calls = std::cell::Cell::new(0);
            let resolved = recovery::resolve_fixture(
                &store,
                &scope,
                Some(&current.binding.origin),
                |client, path| {
                    calls.set(calls.get() + 1);
                    let body = if path
                        == "/api/auth/terminal/session?requestId=pending&pollToken=synthetic-poll"
                    {
                        json!({"ok":true,"accountId":"synthetic-account","terminalToken":"synthetic-replacement","expiresAt":current.session.expires_at})
                    } else {
                        assert_eq!(path, STATUS);
                        assert_ne!(client.identity(), current.binding.client_identity);
                        json!({"ok":true,"accountId":account})
                    };
                    Client::decode_fixture(200, &serde_json::to_vec(&body).unwrap())
                        .map_err(|_| "fixture decode".into())
                },
            );
            assert_eq!(calls.get(), 2);
            if account == "other" {
                assert!(resolved.is_err());
                assert_eq!(load_scoped_from_store(&store, &scope).unwrap(), before);
            } else {
                assert_eq!(resolved.unwrap().terminal_token, "synthetic-replacement");
                assert!(
                    load_scoped_from_store(&store, &scope)
                        .unwrap()
                        .pending
                        .is_none()
                );
            }
            assert!(
                recovery::resolve_fixture(
                    &store,
                    &SessionScope::for_profile("unlinked"),
                    None,
                    |_, _| panic!("no profile fallback")
                )
                .is_err()
            );
        }
    }
}
