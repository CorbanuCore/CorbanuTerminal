//! PF-60-S06: paid work done inside a temporary fork of a saved conversation
//! (#400, #401). A fork has no thread of its own, so its requests are recorded
//! under the conversation it was forked from, labelled with what they were:
//! `side:` only for a TUI `/side` conversation, `fork:` for any other fork, and
//! nested labels (`side:review:`, `side:agent:`) for a guardian review or a
//! sub-agent working inside `/side`.
use super::accounting_responses::support::*;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ThreadSettingsOverrides;
use codex_protocol::protocol::ThreadSource;
use codex_protocol::user_input::UserInput;
use codex_state::accounting::*;
use core_test_support::responses;
use core_test_support::test_codex::TestCodex;
use pretty_assertions::assert_eq;
use serde_json::json;
use wiremock::MockServer;

fn side_source() -> Option<ThreadSource> {
    Some(ThreadSource::Feature(
        ThreadSource::SIDE_CONVERSATION.to_string(),
    ))
}

/// An ephemeral fork of the test's saved conversation, as the TUI makes one.
async fn fork(
    test: &TestCodex,
    source: Option<ThreadSource>,
) -> anyhow::Result<codex_core::NewThread> {
    let mut config = test.config.clone();
    config.ephemeral = true;
    Ok(test
        .thread_manager
        .fork_thread(
            codex_core::ForkSnapshot::Interrupted,
            config,
            test.codex.rollout_path().expect("saved conversation"),
            source,
            /*parent_trace*/ None,
        )
        .await?)
}

async fn turn(
    thread: &codex_core::CodexThread,
    text: &str,
    thread_settings: ThreadSettingsOverrides,
) -> anyhow::Result<()> {
    thread
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: text.into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings,
        })
        .await?;
    core_test_support::wait_for_event(thread, |event| matches!(event, EventMsg::TurnComplete(_)))
        .await;
    Ok(())
}

/// The ledger is written by the collector, not the event stream: wait until
/// the recorded attempts satisfy `done`, and name what was found otherwise.
async fn wait_until(
    db: &codex_state::StateRuntime,
    done: impl Fn(&[Attempt]) -> bool,
) -> anyhow::Result<Vec<Attempt>> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let seen = attempts(db).await?;
        if done(&seen) {
            return Ok(seen);
        }
        if tokio::time::Instant::now() > deadline {
            anyhow::bail!(
                "ledger holds {:?}",
                seen.iter().map(|record| &record.turn).collect::<Vec<_>>()
            );
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

/// The label prefix of each attempt: the turn label without its submission id.
fn kinds(records: &[Attempt]) -> Vec<String> {
    records
        .iter()
        .map(|attempt| {
            attempt
                .turn
                .rsplit_once(':')
                .map(|(kind, _)| format!("{kind}:"))
                .unwrap_or_default()
        })
        .collect()
}

fn body_contains(request: &wiremock::Request, text: &str) -> bool {
    let zstd = request
        .headers
        .get("content-encoding")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .any(|entry| entry.trim().eq_ignore_ascii_case("zstd"))
        });
    let body = if zstd {
        zstd::stream::decode_all(std::io::Cursor::new(&request.body)).ok()
    } else {
        Some(request.body.clone())
    };
    body.and_then(|body| String::from_utf8(body).ok())
        .is_some_and(|body| body.contains(text))
}

/// #401: a temporary fork that is not `/side` - another client's ephemeral
/// fork, or another feature's - is labelled `fork:`, never `side:`, and still
/// counts once under the conversation it was forked from.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_other_temporary_forks_are_labelled_fork() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    responses::mount_sse_sequence(
        &server,
        vec![
            success(usage(Some(0))),
            success(usage(Some(0))),
            success(usage(Some(0))),
        ],
    )
    .await;
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("main conversation").await?;
    let db = test.codex.state_db().unwrap();
    wait_until(&db, |seen| seen.len() == 1).await?;
    let plain = fork(&test, /*source*/ None).await?;
    turn(&plain.thread, "plain fork", Default::default()).await?;
    let feature = fork(&test, Some(ThreadSource::Feature("automation".into()))).await?;
    turn(&feature.thread, "feature fork", Default::default()).await?;
    let records = wait_until(&db, |seen| seen.len() == 3).await?;
    let parent = test.session_configured.thread_id;
    assert!(
        records.iter().all(|attempt| attempt.thread_id == parent),
        "{records:#?}"
    );
    assert_eq!(kinds(&records), vec!["", "fork:", "fork:"], "{records:#?}");
    wait_observations(&db, /*count*/ 3).await?;
    test.thread_manager
        .shutdown_all_threads_bounded(std::time::Duration::from_secs(3))
        .await;
    Ok(())
}

/// #400: a guardian review of a tool call inside `/side` is recorded under the
/// saved conversation the side conversation was forked from, labelled
/// `side:review:`, next to the side conversation's own `side:` requests.
#[cfg(not(target_os = "windows"))]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_guardian_review_inside_side_is_recorded_under_the_conversation()
-> anyhow::Result<()> {
    use codex_core::config::Constrained;
    use codex_core::sandboxing::SandboxPermissions;
    use codex_protocol::config_types::ApprovalsReviewer;
    use codex_protocol::protocol::AskForApproval;
    use codex_protocol::protocol::SandboxPolicy;
    use responses::ev_assistant_message;
    use responses::ev_function_call;
    use responses::ev_response_created;
    use responses::sse;
    core_test_support::skip_if_no_network!(Ok(()));
    core_test_support::skip_if_sandbox!(Ok(()));

    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    let approval_policy = AskForApproval::OnRequest;
    let sandbox_policy = SandboxPolicy::WorkspaceWrite {
        writable_roots: vec![],
        network_access: false,
        exclude_tmpdir_env_var: true,
        exclude_slash_tmp: true,
    };
    let sandbox_for_config = sandbox_policy.clone();
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(move |config| {
            config.permissions.approval_policy = Constrained::allow_any(approval_policy);
            config
                .set_legacy_sandbox_policy(sandbox_for_config)
                .expect("set sandbox policy");
        })
        .build_with_auto_env(&server)
        .await?;
    let output_file = test.cwd.path().join("side-guardian-denied.txt");
    let tool_args = json!({
        "cmd": format!("printf should-not-run > {}", output_file.display()),
        "yield_time_ms": 1_000_u64,
        "sandbox_permissions": SandboxPermissions::RequireEscalated,
        "justification": "Exercise Guardian accounting inside /side.",
    });
    responses::mount_sse_sequence(
        &server,
        vec![
            success(usage(Some(0))),
            sse(vec![
                ev_response_created("resp-side-tool"),
                ev_function_call(
                    "side-exec-call",
                    "exec_command",
                    &serde_json::to_string(&tool_args)?,
                ),
                completed("resp-side-tool"),
            ]),
            sse(vec![
                ev_response_created("resp-guardian"),
                ev_assistant_message(
                    "msg-guardian",
                    &json!({
                        "risk_level": "high",
                        "user_authorization": "low",
                        "outcome": "deny",
                        "rationale": "Fixture denial.",
                    })
                    .to_string(),
                ),
                completed("resp-guardian"),
            ]),
            sse(vec![
                ev_response_created("resp-side-after"),
                ev_assistant_message("msg-side-after", "denied"),
                completed("resp-side-after"),
            ]),
        ],
    )
    .await;
    test.submit_turn("main conversation").await?;
    let db = test.codex.state_db().unwrap();
    wait_until(&db, |seen| seen.len() == 1).await?;
    let side = fork(&test, side_source()).await?;
    turn(
        &side.thread,
        "run a command the reviewer denies",
        ThreadSettingsOverrides {
            approval_policy: Some(approval_policy),
            approvals_reviewer: Some(ApprovalsReviewer::AutoReview),
            sandbox_policy: Some(sandbox_policy),
            ..Default::default()
        },
    )
    .await?;
    let records = wait_until(&db, |seen| seen.len() == 4).await?;
    let parent = test.session_configured.thread_id;
    assert!(
        records.iter().all(|attempt| attempt.thread_id == parent),
        "{records:#?}"
    );
    assert_eq!(
        kinds(&records),
        vec!["", "side:", "side:review:", "side:"],
        "{records:#?}"
    );
    assert!(!output_file.exists());
    // Each request's tokens are counted once in the conversation's day.
    wait_observations(&db, /*count*/ 4).await?;
    let total = totals(&db, &records[0]).await?;
    assert_eq!(total.measured[0].known, 400);
    test.thread_manager
        .shutdown_all_threads_bounded(std::time::Duration::from_secs(3))
        .await;
    Ok(())
}

/// #400: a sub-agent spawned from `/side` has no thread of its own either; its
/// requests are recorded under the saved conversation, labelled
/// `side:agent:`, and the side conversation's own requests stay `side:`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_sub_agent_spawned_from_side_is_recorded_under_the_conversation()
-> anyhow::Result<()> {
    use codex_features::Feature;
    use responses::ev_assistant_message;
    use responses::ev_function_call_with_namespace;
    use responses::ev_response_created;
    use responses::sse;
    core_test_support::skip_if_no_network!(Ok(()));

    const SIDE_PROMPT: &str = "side question that spawns a helper";
    const CHILD_PROMPT: &str = "helper task from the side conversation";
    const SPAWN_CALL_ID: &str = "side-spawn-call";
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(|config| {
            config
                .features
                .enable(Feature::Collab)
                .expect("test config should allow feature update");
        })
        .build_with_auto_env(&server)
        .await?;
    responses::mount_sse_once(&server, success(usage(Some(0)))).await;
    test.submit_turn("main conversation").await?;
    let db = test.codex.state_db().unwrap();
    wait_until(&db, |seen| seen.len() == 1).await?;

    responses::mount_sse_once_match(
        &server,
        |request: &wiremock::Request| {
            body_contains(request, SIDE_PROMPT) && !body_contains(request, SPAWN_CALL_ID)
        },
        sse(vec![
            ev_response_created("resp-side-spawn"),
            ev_function_call_with_namespace(
                SPAWN_CALL_ID,
                "collaboration",
                "spawn_agent",
                &json!({ "message": CHILD_PROMPT, "task_name": "helper", "fork_turns": "none" })
                    .to_string(),
            ),
            completed("resp-side-spawn"),
        ]),
    )
    .await;
    responses::mount_sse_once_match(
        &server,
        |request: &wiremock::Request| {
            body_contains(request, CHILD_PROMPT) && !body_contains(request, SPAWN_CALL_ID)
        },
        sse(vec![
            ev_response_created("resp-child"),
            ev_assistant_message("msg-child", "helper done"),
            completed("resp-child"),
        ]),
    )
    .await;
    responses::mount_sse_once_match(
        &server,
        |request: &wiremock::Request| body_contains(request, SPAWN_CALL_ID),
        sse(vec![
            ev_response_created("resp-side-after"),
            ev_assistant_message("msg-side-after", "side done"),
            completed("resp-side-after"),
        ]),
    )
    .await;
    let side = fork(&test, side_source()).await?;
    turn(&side.thread, SIDE_PROMPT, Default::default()).await?;
    wait_until(&db, |seen| {
        let kinds = kinds(seen);
        kinds.iter().any(|kind| kind == "side:agent:")
            && kinds.iter().filter(|kind| *kind == "side:").count() >= 2
    })
    .await?;
    // Nothing else arrives later: each request is recorded once.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let records = attempts(&db).await?;
    let parent = test.session_configured.thread_id;
    assert!(
        records.iter().all(|attempt| attempt.thread_id == parent),
        "{records:#?}"
    );
    let mut kinds = kinds(&records);
    kinds.sort();
    assert_eq!(
        kinds,
        vec!["", "side:", "side:", "side:agent:"],
        "{records:#?}"
    );
    // The conversation's day counts all four requests' tokens, each once.
    wait_observations(&db, /*count*/ 4).await?;
    let total = totals(&db, &records[0]).await?;
    assert_eq!(total.measured[0].known, 400);
    test.thread_manager
        .shutdown_all_threads_bounded(std::time::Duration::from_secs(3))
        .await;
    Ok(())
}

/// AC5: work for an `exec --ephemeral` conversation stays uncollected. A
/// sub-agent it spawns looks the conversation up, finds it excluded and is
/// excluded too (`parent_not_persisted`); nothing reaches the ledger.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_sub_agent_of_an_ephemeral_run_stays_uncollected() -> anyhow::Result<()> {
    use responses::ev_assistant_message;
    use responses::ev_function_call_with_namespace;
    use responses::ev_response_created;
    use responses::sse;
    core_test_support::skip_if_no_network!(Ok(()));

    const ROOT_PROMPT: &str = "ephemeral run that spawns a helper";
    const CHILD_PROMPT: &str = "helper task from the ephemeral run";
    const SPAWN_CALL_ID: &str = "ephemeral-spawn-call";
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(|config| config.ephemeral = true)
        .build_with_auto_env(&server)
        .await?;
    responses::mount_sse_once_match(
        &server,
        |request: &wiremock::Request| {
            body_contains(request, ROOT_PROMPT) && !body_contains(request, SPAWN_CALL_ID)
        },
        sse(vec![
            ev_response_created("resp-root-spawn"),
            ev_function_call_with_namespace(
                SPAWN_CALL_ID,
                "collaboration",
                "spawn_agent",
                &json!({ "message": CHILD_PROMPT, "task_name": "helper", "fork_turns": "none" })
                    .to_string(),
            ),
            completed("resp-root-spawn"),
        ]),
    )
    .await;
    let child = responses::mount_sse_once_match(
        &server,
        |request: &wiremock::Request| {
            body_contains(request, CHILD_PROMPT) && !body_contains(request, SPAWN_CALL_ID)
        },
        sse(vec![
            ev_response_created("resp-child"),
            ev_assistant_message("msg-child", "helper done"),
            completed("resp-child"),
        ]),
    )
    .await;
    responses::mount_sse_once_match(
        &server,
        |request: &wiremock::Request| body_contains(request, SPAWN_CALL_ID),
        sse(vec![
            ev_response_created("resp-root-after"),
            ev_assistant_message("msg-root-after", "root done"),
            completed("resp-root-after"),
        ]),
    )
    .await;
    test.submit_turn(ROOT_PROMPT).await?;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    while child.requests().is_empty() {
        anyhow::ensure!(
            tokio::time::Instant::now() < deadline,
            "the helper never sent its request"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let db = codex_state::StateRuntime::init(test.config.sqlite.clone(), "openai".into()).await?;
    // The ledger table only exists once something was collected in this home.
    match attempts(&db).await {
        Ok(seen) => assert_eq!(seen.len(), 0, "{seen:#?}"),
        Err(error) => assert!(
            error.to_string().contains("no such table"),
            "unexpected ledger error: {error}"
        ),
    }
    test.thread_manager
        .shutdown_all_threads_bounded(std::time::Duration::from_secs(3))
        .await;
    Ok(())
}

/// A completed response with fixture usage.
fn completed(id: &str) -> serde_json::Value {
    json!({"type": "response.completed", "response": {"id": id, "usage": usage(Some(0))}})
}
