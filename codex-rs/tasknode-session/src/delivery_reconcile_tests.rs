use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

const EVENT: &str = r#"{"content": "PF-80-S01: worker reports working. Recovery test passed; human acceptance still pending.\nProgress observation only; no acceptance or task-completion claim.", "coverage": "manager_observed_worker_report_not_independent_acceptance", "goal": {"active": true, "id": "PF-80-S01", "status": "reported_working"}, "id": "cc-cf3440216007cdf72d47f5467dbdc482d916b96d76635bc3537a222b83d67373", "instanceId": "corbanu-control", "kind": "goal", "occurredAt": "2026-09-11T12:00:00.000Z", "repository": {"branch": "test/fixture", "commit": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "label": "Corbanu Terminal"}, "sequence": 0, "sessionId": "run-1", "taskIds": ["task-a", "task-b"], "turnId": "PF-80-S01", "workspaceId": "synthetic-workspace"}"#;
// Literal output shaped by validateEvent -> trackerIngest -> trackerRead at
// 40d2df72710a644f33a2b30831061e8265716db0; never an actual activity response.
const ACTIVITY_BYTES: &str = r#"{"ok":true,"items":[{"content":"PF-80-S01: worker reports working. Recovery test passed; human acceptance still pending.\nProgress observation only; no acceptance or task-completion claim.","coverage":"manager_observed_worker_report_not_independent_acceptance","goal":{"active":true,"id":"PF-80-S01","status":"reported_working"},"id":"cc-cf3440216007cdf72d47f5467dbdc482d916b96d76635bc3537a222b83d67373","instanceId":"corbanu-control","kind":"goal","occurredAt":"2026-09-11T12:00:00.000Z","repository":{"branch":"test/fixture","commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","label":"Corbanu Terminal"},"sequence":0,"sessionId":"run-1","taskIds":["task-a","task-b"],"turnId":"PF-80-S01","workspaceId":"synthetic-workspace","facts":{},"model":"","sourceDigest":"37428aa800196c633406f3e21e5df025ad80928cf4f6a58814417502a7d0aab3","handleAtExecution":"fixture-handle","summary":null,"accountId":"synthetic-account","revision":1,"summaryState":"not_applicable","receivedAt":"2026-09-11T12:00:01.000Z","capabilities":["summary","prompt","replay","review","export"]}],"nextCursor":null,"coverage":"observed_tui"}"#;
const HASH: &str = "5c8cd79f2b097a9acc262c92ff159be7d12c4d41eefa9f49e15f3d09a259b4ad";
const TOKEN: &str = "synthetic-terminal-credential-canary";
const KEY: &str = "synthetic-api-credential-canary";

struct Timeout(usize);
impl FixtureExchange for Timeout {
    fn exchange(
        &mut self,
        _: &'static str,
        _: &Value,
        _: &mut Current,
    ) -> Result<Response, ExchangeFailure> {
        self.0 += 1;
        Err(ExchangeFailure::Timeout)
    }
}

fn fixture() -> (Attempt, Current, Outcome, RawActivity) {
    let now = "2026-09-11T12:00:00Z".parse().unwrap();
    let event: Value = serde_json::from_str(EVENT).unwrap();
    let session = ActiveSession {
        origin: "https://TASKNODE.example:443/".into(),
        account_id: Some("synthetic-account".into()),
        github_username: None,
        terminal_token: TOKEN.into(),
        expires_at: Some("2026-09-11T14:00:00Z".into()),
    };
    let b = Binding {
        invocation: "fixture-invocation".into(),
        profile: SessionScope::for_profile("fixture-profile"),
        account: "synthetic-account".into(),
        origin: "https://tasknode.example".into(),
        client_identity: Client::for_session(&session, &session.origin)
            .unwrap()
            .identity(),
        event_id: event["id"].as_str().unwrap().into(),
        payload_sha256: HASH.into(),
        sprint: "PF-80-S01".into(),
        workspace: "synthetic-workspace".into(),
        tasks: vec!["task-a".into(), "task-b".into()],
    };
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
    let mut current = Current {
        binding: b.clone(),
        session,
        now,
        cancelled: false,
        event_json: EVENT.into(),
        facts: REQUIRED
            .iter()
            .map(|kind| FixtureFact {
                kind: *kind,
                binding: b.clone(),
                knowledge: Knowledge::Verified,
            })
            .collect(),
    };
    let mut attempt = Attempt {
        binding: b.clone(),
        preparation,
        prepared_at: now,
        consumed: false,
    };
    let mut timeout = Timeout(0);
    let outcome = attempt.run(&mut current, &mut timeout);
    assert_eq!(timeout.0, 1);
    assert_eq!(outcome.state, State::OutcomeUnknown);
    current.facts.clear();
    let raw = RawActivity {
        path: activity_path(&b),
        binding: b,
        method: Method::GET,
        status: 200,
        bytes: ACTIVITY_BYTES.as_bytes().to_vec(),
    };
    (attempt, current, outcome, raw)
}

fn body() -> Value {
    serde_json::from_str(ACTIVITY_BYTES).unwrap()
}

fn reject_body(body: Value, expected: Diagnostic) {
    let (attempt, current, outcome, mut raw) = fixture();
    raw.bytes = serde_json::to_vec(&body).unwrap();
    let result = reconcile(&attempt, &outcome, &current, &raw).err().unwrap();
    assert_eq!(result, expected);
    for canary in [TOKEN, KEY, "Recovery test passed"] {
        assert!(!format!("{result:?}").contains(canary));
    }
    assert_eq!(current.event_json, EVENT);
    assert_eq!(outcome.state, State::OutcomeUnknown);
}

#[test]
fn literal_goal_builds_exact_native_get_and_never_resets_attempt_or_grants_facts() {
    let (mut attempt, mut current, outcome, raw) = fixture();
    let before = format!("{:?}", attempt.preparation);
    let observed = reconcile(&attempt, &outcome, &current, &raw).unwrap();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {TOKEN}").parse().unwrap(),
    );
    assert_eq!(
        (
            observed.request.method(),
            observed.request.url().as_str(),
            observed.request.headers(),
            observed.request.body().is_none()
        ),
        (
            &Method::GET,
            format!(
                "https://tasknode.example{ACTIVITY}?accountId=synthetic-account&id={}",
                outcome.event_id
            )
            .as_str(),
            &headers,
            true
        )
    );
    assert_eq!(current.event_json.as_bytes(), EVENT.as_bytes());
    assert_eq!(
        attempt.preparation.payload(),
        &json!({"event": serde_json::from_str::<Value>(EVENT).unwrap()})
    );
    assert_eq!(attempt.preparation.payload_sha256(), HASH);
    assert_eq!(format!("{:?}", attempt.preparation), before);
    assert_eq!(
        outcome,
        Outcome {
            event_id: current.binding.event_id.clone(),
            payload_sha256: HASH.into(),
            state: State::OutcomeUnknown
        }
    );
    assert!(current.facts.is_empty());
    let mut retry = Timeout(0);
    assert_eq!(
        attempt.run(&mut current, &mut retry).state,
        State::AlreadyConsumed
    );
    assert_eq!(retry.0, 0);
}

#[test]
fn query_values_cannot_add_parameters_or_change_path() {
    let (_, current, _, _) = fixture();
    let mut b = current.binding;
    b.account = "account &id=other#fragment? /猫".into();
    b.event_id = "event&accountId=other".into();
    let url = url::Url::parse(&format!("{}{}", b.origin, activity_path(&b))).unwrap();
    assert_eq!(
        (
            url.path(),
            url.fragment(),
            url.query_pairs().into_owned().collect::<Vec<_>>()
        ),
        (
            ACTIVITY,
            None,
            vec![("accountId".into(), b.account), ("id".into(), b.event_id)]
        )
    );
}

#[test]
fn every_original_field_is_required_and_exact_including_nested_metadata() {
    let original: Value = serde_json::from_str(EVENT).unwrap();
    for key in original.as_object().unwrap().keys() {
        let mut changed = body();
        changed["items"][0].as_object_mut().unwrap().remove(key);
        reject_body(changed, Diagnostic::Conflict);
    }
    for (pointer, replacement) in [
        ("content", json!("")),
        ("coverage", json!("observed_tui")),
        ("goal/active", json!(false)),
        ("goal/id", json!("PF-79-S01")),
        ("goal/status", json!("reported_finished")),
        ("id", json!("cc-cf344021")),
        ("instanceId", json!("other")),
        ("kind", json!("agent_output")),
        ("occurredAt", json!("2026-09-11T12:00:00Z")),
        ("repository/branch", json!(TOKEN)),
        ("repository/commit", json!("aaaaaaaa")),
        ("repository/label", json!("other")),
        ("sequence", json!(1)),
        ("sessionId", json!("other")),
        ("taskIds", json!(["task-a"])),
        ("taskIds/0", json!("task")),
        ("turnId", json!("PF-79-S01")),
        ("workspaceId", json!("other")),
    ] {
        let mut changed = body();
        *changed.pointer_mut(&format!("/items/0/{pointer}")).unwrap() = replacement;
        reject_body(changed, Diagnostic::Conflict);
    }
    for field in ["goal", "repository"] {
        for key in original[field].as_object().unwrap().keys() {
            let mut changed = body();
            changed["items"][0][field]
                .as_object_mut()
                .unwrap()
                .remove(key);
            reject_body(changed, Diagnostic::Conflict);
        }
    }
    let mut changed = body();
    changed["items"][0]["promptWithheld"] = json!(false);
    reject_body(changed, Diagnostic::Conflict);
}

#[test]
fn strict_envelope_rejects_incomplete_duplicate_paginated_and_non_object_records() {
    for key in ["ok", "items", "nextCursor", "coverage"] {
        let mut changed = body();
        changed.as_object_mut().unwrap().remove(key);
        reject_body(changed, Diagnostic::Shape);
    }
    for (key, replacement) in [
        ("ok", json!(false)),
        ("ok", json!("true")),
        ("coverage", json!(null)),
        (
            "nextCursor",
            json!({"before":"2026-09-11T12:00:00.000Z","beforeId":"other"}),
        ),
        ("nextCursor", json!("")),
        ("items", json!([])),
        ("items", json!([null])),
        ("items", json!({})),
        ("items", json!([body()["items"][0], body()["items"][0]])),
        ("error", json!(KEY)),
    ] {
        let mut changed = body();
        changed[key] = replacement;
        reject_body(changed, Diagnostic::Shape);
    }
}

#[test]
fn annotations_require_complete_source_defaults_and_owner_visibility() {
    for key in [
        "accountId",
        "revision",
        "summaryState",
        "receivedAt",
        "capabilities",
        "handleAtExecution",
        "summary",
        "facts",
        "model",
        "sourceDigest",
    ] {
        let mut changed = body();
        changed["items"][0].as_object_mut().unwrap().remove(key);
        reject_body(changed, Diagnostic::Annotations);
        let mut changed = body();
        changed["items"][0][key] = json!([KEY]);
        reject_body(changed, Diagnostic::Annotations);
    }
    for (key, replacement) in [
        ("accountId", json!("other")),
        ("revision", json!(0)),
        ("revision", json!(1.0)),
        ("revision", json!(-1)),
        ("revision", json!(9007199254740992_u64)),
        ("receivedAt", json!("invalid")),
        ("receivedAt", json!("2026-09-11T12:00:00+01:00")),
        ("capabilities", json!(["summary"])),
        (
            "capabilities",
            json!(["summary", "prompt", "replay", "review", "export", "summary"]),
        ),
        ("summary", json!({"summary":KEY})),
        ("facts", json!({"status":"finished"})),
        ("model", json!("other")),
        ("sourceDigest", json!(HASH)),
    ] {
        let mut changed = body();
        changed["items"][0][key] = replacement;
        reject_body(changed, Diagnostic::Annotations);
    }
    for state in ["pending", "ready", "deleted", "unavailable", "unknown", ""] {
        let mut changed = body();
        changed["items"][0]["summaryState"] = json!(state);
        reject_body(changed, Diagnostic::Annotations);
    }
}

fn mutate_binding(b: &mut Binding, index: usize) {
    match index {
        0 => b.invocation.clear(),
        1 => b.profile = SessionScope::default(),
        2 => b.account.push('x'),
        3 => b.origin = "https://other.example".into(),
        4 => b.client_identity.push('x'),
        5 => b.event_id.truncate(12),
        6 => b.payload_sha256.truncate(12),
        7 => b.sprint = "PF-79-S01".into(),
        8 => b.workspace.push('x'),
        9 => b.tasks.push("task-c".into()),
        _ => unreachable!(),
    }
}

#[test]
fn all_raw_and_current_bindings_and_method_path_are_fenced() {
    for index in 0..10 {
        let (attempt, current, outcome, mut raw) = fixture();
        mutate_binding(&mut raw.binding, index);
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Unrelated)
        );
        let (attempt, mut current, outcome, raw) = fixture();
        mutate_binding(&mut current.binding, index);
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Fence(Hold::Identity))
        );
    }
    for method in [Method::POST, Method::HEAD] {
        let (attempt, current, outcome, mut raw) = fixture();
        raw.method = method;
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Unrelated)
        );
    }
    for path in [
        ACTIVITY,
        ENDPOINT,
        "/api/terminal/tasknode/campaign-tracker/activity?accountId=other&id=x",
    ] {
        let (attempt, current, outcome, mut raw) = fixture();
        raw.path = path.into();
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Unrelated)
        );
    }
}

#[test]
fn cancellation_session_changes_expiry_and_staleness_preserve_existing_holds() {
    let changes: [(fn(&mut Current), Hold); 12] = [
        (|c| c.cancelled = true, Hold::Cancelled),
        (|c| c.session.expires_at = None, Hold::ExpiryUnknown),
        (
            |c| c.session.expires_at = Some("invalid".into()),
            Hold::ExpiryUnknown,
        ),
        (
            |c| c.session.expires_at = Some("2026-09-11T12:00:00Z".into()),
            Hold::Expired,
        ),
        (|c| c.session.terminal_token = KEY.into(), Hold::Identity),
        (|c| c.session.terminal_token.clear(), Hold::Identity),
        (|c| c.session.account_id = None, Hold::Identity),
        (
            |c| c.session.account_id = Some("other".into()),
            Hold::Identity,
        ),
        (
            |c| c.session.origin = "https://other.example".into(),
            Hold::Identity,
        ),
        (|c| c.now -= chrono::Duration::seconds(1), Hold::Stale),
        (|c| c.now += chrono::Duration::minutes(46), Hold::Stale),
        (
            |c| c.event_json = EVENT.replace("Recovery", "Changed"),
            Hold::Preparation,
        ),
    ];
    for (change, reason) in changes {
        let (attempt, mut current, outcome, raw) = fixture();
        change(&mut current);
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Fence(reason))
        );
    }
}

#[test]
fn only_original_consumed_uncertain_selection_can_be_observed() {
    for index in 0..4 {
        let (mut attempt, current, mut outcome, raw) = fixture();
        match index {
            0 => attempt.consumed = false,
            1 => outcome.state = State::GoalRecorded,
            2 => outcome.event_id = KEY.into(),
            3 => outcome.payload_sha256 = TOKEN.into(),
            _ => unreachable!(),
        }
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::NotUncertain)
        );
    }
    for index in 5..10 {
        let (mut attempt, mut current, mut outcome, mut raw) = fixture();
        mutate_binding(&mut attempt.binding, index);
        current.binding = attempt.binding.clone();
        raw.binding = attempt.binding.clone();
        outcome.event_id = attempt.binding.event_id.clone();
        outcome.payload_sha256 = attempt.binding.payload_sha256.clone();
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Fence(Hold::Preparation))
        );
    }
}

#[test]
fn http_failures_and_404_never_export_server_text_or_clear_uncertainty() {
    for status in [
        100, 301, 302, 307, 308, 400, 401, 402, 403, 404, 409, 429, 500, 503,
    ] {
        let (mut attempt, mut current, outcome, mut raw) = fixture();
        raw.status = status;
        raw.bytes = json!({"error":KEY,"message":TOKEN})
            .to_string()
            .into_bytes();
        let expected = if status == 404 {
            Diagnostic::NotFoundUncertain
        } else {
            Diagnostic::Http(status)
        };
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(expected)
        );
        assert_eq!(outcome.state, State::OutcomeUnknown);
        let mut retry = Timeout(0);
        assert_eq!(
            attempt.run(&mut current, &mut retry).state,
            State::AlreadyConsumed
        );
        assert_eq!(retry.0, 0);
    }
}

#[test]
fn decoder_rejects_invalid_and_duplicate_json_at_every_depth() {
    let mut cases = vec![
        vec![],
        b"{".to_vec(),
        b"[]".to_vec(),
        vec![255],
        b"null".to_vec(),
        vec![b' '; 16 * 1024 * 1024 + 1],
    ];
    for (key, first) in [
        ("ok", "false"),
        ("id", "\"conflict\""),
        ("active", "false"),
        ("branch", "\"conflict\""),
        ("revision", "2"),
    ] {
        cases.push(
            ACTIVITY_BYTES
                .replacen(
                    &format!("\"{key}\":"),
                    &format!("\"{key}\":{first},\"{key}\":"),
                    1,
                )
                .into_bytes(),
        );
    }
    for bytes in cases {
        let (attempt, current, outcome, mut raw) = fixture();
        raw.bytes = bytes;
        assert_eq!(
            reconcile(&attempt, &outcome, &current, &raw).err(),
            Some(Diagnostic::Decoder)
        );
    }
}
