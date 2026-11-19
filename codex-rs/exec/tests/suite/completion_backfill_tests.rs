use core_test_support::responses;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex_exec::test_codex_exec;
use pretty_assertions::assert_eq;
use serde_json::json;

const TRACE_REQUESTS: &str =
    "codex_app_server::message_processor=trace,codex_app_server::outgoing_message=trace";

const PARENT_PROMPT: &str = "spawn a child and wait for it";
const CHILD_PROMPT: &str = "child: finish first";
const SPAWN_CALL_ID: &str = "spawn-call";
const WAIT_CALL_ID: &str = "wait-call";
const LONG_PARENT_PROMPT: &str = "Produce a long turn";
const FAILED_FORK_PROMPT: &str = "fail this forked turn";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamedUsageRoute {
    ParentTurn,
    ChildTurn,
    ParentWait,
    ParentCompletion,
    Unmatched,
}

impl StreamedUsageRoute {
    fn for_request(request: &wiremock::Request) -> Self {
        serde_json::from_slice(&request.body).map_or(Self::Unmatched, |body| Self::for_body(&body))
    }

    fn for_body(body: &serde_json::Value) -> Self {
        let items = body["input"].as_array();
        let items = items.map(Vec::as_slice).unwrap_or_default();
        let call = |id: &str| items.iter().any(|item| item["call_id"] == id);
        let msg = |text: &str| items.iter().any(|item| message_contains(item, text));
        if call(WAIT_CALL_ID) {
            Self::ParentCompletion
        } else if msg(CHILD_PROMPT) {
            Self::ChildTurn
        } else if call(SPAWN_CALL_ID) {
            Self::ParentWait
        } else if msg(PARENT_PROMPT) {
            Self::ParentTurn
        } else {
            Self::Unmatched
        }
    }
}

fn message_contains(item: &serde_json::Value, needle: &str) -> bool {
    item["type"] == "message" && item["content"].to_string().contains(needle)
}

struct StreamedUsageResponder {
    parent: String,
    child: String,
    wait: String,
    done: String,
}

impl wiremock::Respond for StreamedUsageResponder {
    fn respond(&self, request: &wiremock::Request) -> wiremock::ResponseTemplate {
        responses::validate_request_body_invariants(request);
        match StreamedUsageRoute::for_request(request) {
            StreamedUsageRoute::ParentTurn => responses::sse_response(self.parent.clone()),
            StreamedUsageRoute::ChildTurn => responses::sse_response(self.child.clone()),
            StreamedUsageRoute::ParentWait => responses::sse_response(self.wait.clone()),
            StreamedUsageRoute::ParentCompletion => responses::sse_response(self.done.clone()),
            StreamedUsageRoute::Unmatched => wiremock::ResponseTemplate::new(404),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailedForkRoute {
    ParentTurn,
    FailedFork,
    Unmatched,
}

impl FailedForkRoute {
    fn for_request(request: &wiremock::Request) -> Self {
        serde_json::from_slice(&request.body).map_or(Self::Unmatched, |body| Self::for_body(&body))
    }

    fn for_body(body: &serde_json::Value) -> Self {
        let items = body["input"].as_array();
        let items = items.map(Vec::as_slice).unwrap_or_default();
        let msg = |text: &str| items.iter().any(|item| message_contains(item, text));
        if msg(FAILED_FORK_PROMPT) {
            Self::FailedFork
        } else if msg(LONG_PARENT_PROMPT) {
            Self::ParentTurn
        } else {
            Self::Unmatched
        }
    }
}

struct FailedForkResponder {
    parent: String,
    error: wiremock::ResponseTemplate,
}

impl wiremock::Respond for FailedForkResponder {
    fn respond(&self, request: &wiremock::Request) -> wiremock::ResponseTemplate {
        responses::validate_request_body_invariants(request);
        match FailedForkRoute::for_request(request) {
            FailedForkRoute::ParentTurn => responses::sse_response(self.parent.clone()),
            FailedForkRoute::FailedFork => self.error.clone(),
            FailedForkRoute::Unmatched => wiremock::ResponseTemplate::new(404),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ignores_unrelated_turn_completion_before_backfilling_primary_turn() -> anyhow::Result<()> {
    streamed_usage("none", Some(200)).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue290_forked_child_without_authoritative_baseline_is_unknown() -> anyhow::Result<()> {
    streamed_usage("all", Some(200)).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn issue290_missing_child_usage_makes_the_aggregate_unknown() -> anyhow::Result<()> {
    streamed_usage("none", /*child_tokens*/ None).await
}

#[test]
fn streamed_usage_route_prefers_the_child_message_over_inherited_tool_calls() {
    let body = json!({"input": [
        {"type": "message", "role": "user", "content": [{"type": "input_text", "text": CHILD_PROMPT}]},
        {"type": "function_call", "call_id": SPAWN_CALL_ID, "name": "spawn_agent"},
    ]});
    let route = StreamedUsageRoute::for_body(&body);
    assert_eq!(route, StreamedUsageRoute::ChildTurn);
}

async fn streamed_usage(fork_turns: &str, child_tokens: Option<i64>) -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let spawn_args = json!({
        "message": CHILD_PROMPT,
        "task_name": "worker",
        "fork_turns": fork_turns,
    })
    .to_string();
    let parent = responses::sse(vec![
        responses::ev_response_created("resp-parent-1"),
        responses::ev_function_call_with_namespace(
            SPAWN_CALL_ID,
            "collaboration",
            "spawn_agent",
            &spawn_args,
        ),
        responses::ev_completed_with_tokens("resp-parent-1", /*total_tokens*/ 100),
    ]);
    let child = responses::sse(vec![
        responses::ev_response_created("resp-child"),
        responses::ev_assistant_message("msg-child", "child done"),
        match child_tokens {
            Some(tokens) => responses::ev_completed_with_tokens("resp-child", tokens),
            None => json!({"type": "response.completed", "response": {"id": "resp-child"}}),
        },
    ]);
    let wait = responses::sse(vec![
        responses::ev_response_created("resp-parent-2"),
        responses::ev_function_call_with_namespace(
            WAIT_CALL_ID,
            "collaboration",
            "wait_agent",
            "{}",
        ),
        responses::ev_completed_with_tokens("resp-parent-2", /*total_tokens*/ 300),
    ]);
    let done = responses::sse(vec![
        responses::ev_response_created("resp-parent-3"),
        responses::ev_assistant_message("msg-parent", "parent done"),
        responses::ev_completed_with_tokens("resp-parent-3", /*total_tokens*/ 400),
    ]);
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path_regex(".*/responses$"))
        .respond_with(StreamedUsageResponder {
            parent,
            child,
            wait,
            done,
        })
        .mount(&server)
        .await;

    let test = test_codex_exec();
    let mock_provider = format!(
        "model_providers.mock_provider={{name=\"Mock provider for test\",base_url=\"{}/v1\",wire_api=\"responses\",supports_websockets=false}}",
        server.uri()
    );
    let output = test
        .cmd()
        .env("RUST_LOG", TRACE_REQUESTS)
        .arg("--skip-git-repo-check")
        .arg("--json")
        .arg("-c")
        .arg(mock_provider)
        .args([
            "-c",
            "model_provider=\"mock_provider\"",
            "-c",
            "features.multi_agent=true",
            "-c",
            "features.multi_agent_v2=true",
            "-c",
            "features.enable_request_compression=false",
        ])
        .arg(PARENT_PROMPT)
        .output()?;
    let requests = server.received_requests().await.unwrap_or_default();
    let routed_requests = requests
        .iter()
        .filter(|request| request.method == "POST" && request.url.path().ends_with("/responses"))
        .map(|request| (StreamedUsageRoute::for_request(request), request))
        .collect::<Vec<_>>();
    let unmatched_body_report = routed_requests
        .iter()
        .filter(|(route, _)| *route == StreamedUsageRoute::Unmatched)
        .map(|(_, request)| format!("{}\n", String::from_utf8_lossy(&request.body)))
        .collect::<String>();
    assert!(
        output.status.success(),
        "exec run failed: {output:?}\nunmatched request bodies:\n{unmatched_body_report}"
    );
    assert!(
        routed_requests
            .iter()
            .all(|(_, request)| request.headers.get("content-encoding").is_none()),
        "this fixture routes on raw request bodies, so compression must stay disabled"
    );
    assert!(
        routed_requests
            .iter()
            .all(|(route, _)| *route != StreamedUsageRoute::Unmatched),
        "every responses request must match a route: {unmatched_body_report}"
    );
    for route in [
        StreamedUsageRoute::ParentTurn,
        StreamedUsageRoute::ChildTurn,
        StreamedUsageRoute::ParentWait,
        StreamedUsageRoute::ParentCompletion,
    ] {
        assert!(
            routed_requests
                .iter()
                .any(|(candidate, _)| *candidate == route),
            "expected the {route:?} route to be hit"
        );
    }
    let stdout = String::from_utf8(output.stdout)?;
    if fork_turns == "all" {
        assert!(
            !stdout.contains("Usage is unknown: could not resolve"),
            "fork usage must be unknown because of fork detection, not a failed ancestry read: {stdout}"
        );
    }
    assert!(
        stdout.contains("parent done"),
        "primary completion was not processed: {stdout}"
    );
    let events = stdout
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    let completions = events
        .iter()
        .filter(|event| event["type"] == "turn.completed")
        .collect::<Vec<_>>();
    assert_eq!(completions.len(), 1, "child completion must not end exec");
    assert_eq!(
        completions[0]["usage"],
        child_tokens
            .filter(|_| fork_turns == "none")
            .map(|tokens| json!({
                "input_tokens": 800 + tokens,
                "cached_input_tokens": 0,
                "cache_write_input_tokens": 0,
                "output_tokens": 0,
                "reasoning_output_tokens": 0,
            }))
            .unwrap_or(serde_json::Value::Null),
        "root 100 + 300 + 400 and child 200 must each be counted once"
    );
    let stderr = String::from_utf8(output.stderr)?;
    let lines = stderr.lines().collect::<Vec<_>>();
    let turn_completions = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            line.contains("app-server event: turn/completed")
                .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        turn_completions.len(),
        2,
        "expected the child completion before the primary completion: {stderr}"
    );

    let [child_completion, primary_completion] = turn_completions.as_slice() else {
        unreachable!("checked turn/completed count")
    };
    assert_eq!(
        lines[*child_completion + 1..*primary_completion]
            .iter()
            .filter(|line| line.contains("app-server typed request"))
            .count(),
        0,
        "the unrelated completion must not read history: {stderr}"
    );
    assert_eq!(
        lines[*primary_completion + 1..]
            .iter()
            .filter(|line| line.contains("app-server typed request"))
            .count(),
        2,
        "the primary completion should read its items and unsubscribe: {stderr}"
    );

    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("app-server typed request"))
            .count(),
        6,
        "one metadata read resolves child ancestry; only the primary completion backfills items: {stderr}"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_fork_recovers_only_its_own_turn() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = responses::start_mock_server().await;
    let test = test_codex_exec();
    let last_message = test.cwd_path().join("last-message");
    let mut events = vec![responses::ev_response_created("parent")];
    // The parent's final answer must survive recovery across multiple item pages.
    for index in 0..120 {
        events.push(responses::ev_assistant_message(
            &format!("msg-{index}"),
            "working",
        ));
    }
    events.push(responses::ev_assistant_message("answer", "parent done"));
    events.push(responses::ev_completed("parent"));
    let fork_error = wiremock::ResponseTemplate::new(400).set_body_json(json!({
        "error": {"code": "model_not_found", "message": "Unknown model", "type": "invalid_request_error"}
    }));
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path_regex(".*/responses$"))
        .respond_with(FailedForkResponder {
            parent: responses::sse(events),
            error: fork_error,
        })
        .mount(&server)
        .await;
    let parent = test
        .cmd_with_server(&server)
        .args(["--skip-git-repo-check", "--json", "--output-last-message"])
        .arg(&last_message)
        .arg(LONG_PARENT_PROMPT)
        .assert()
        .success();
    assert_eq!(std::fs::read_to_string(last_message)?, "parent done");
    let stdout = std::str::from_utf8(&parent.get_output().stdout)?;
    let started: serde_json::Value = serde_json::from_str(stdout.lines().next().unwrap())?;
    let parent_id = started["thread_id"].as_str().expect("parent thread ID");

    let fork = test
        .cmd_with_server(&server)
        .env("RUST_LOG", TRACE_REQUESTS)
        .args([
            "--skip-git-repo-check",
            "--json",
            "fork",
            parent_id,
            FAILED_FORK_PROMPT,
        ])
        .assert()
        .code(1);
    let requests = server.received_requests().await.unwrap_or_default();
    let routed_requests = requests
        .iter()
        .filter(|request| request.method == "POST" && request.url.path().ends_with("/responses"))
        .map(|request| (FailedForkRoute::for_request(request), request))
        .collect::<Vec<_>>();
    let unmatched_body_report = routed_requests
        .iter()
        .filter(|(route, _)| *route == FailedForkRoute::Unmatched)
        .map(|(_, request)| format!("{}\n", String::from_utf8_lossy(&request.body)))
        .collect::<String>();
    assert!(
        routed_requests
            .iter()
            .all(|(route, _)| *route != FailedForkRoute::Unmatched),
        "every responses request must match a route: {unmatched_body_report}"
    );
    for route in [FailedForkRoute::ParentTurn, FailedForkRoute::FailedFork] {
        assert!(
            routed_requests
                .iter()
                .any(|(candidate, _)| *candidate == route),
            "expected the {route:?} route to be hit"
        );
    }
    let stdout = std::str::from_utf8(&fork.get_output().stdout)?;
    let terminal: serde_json::Value = serde_json::from_str(stdout.lines().last().unwrap())?;
    assert_eq!(
        terminal,
        json!({"type": "turn.failed", "error": {"message": "Unknown model"}})
    );

    let stderr = std::str::from_utf8(&fork.get_output().stderr)?;
    let (_, completion) = stderr
        .split_once("app-server event: turn/completed")
        .expect("fork completed");
    assert!(
        !completion.contains("app-server event: deprecationNotice"),
        "{completion}"
    );
    assert_eq!(
        completion.matches("app-server typed request").count(),
        2,
        "the failed turn needs one item page and an unsubscribe: {completion}"
    );
    Ok(())
}
