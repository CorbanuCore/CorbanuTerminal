#[path = "accounting_anthropic_support.rs"]
mod support;
use codex_core::config::AccountingMode;
use codex_features::Feature;
use codex_protocol::protocol::EventMsg;
use codex_state::accounting::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use support::*;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_native_spawned_children_role_reload_and_fork_own_only_new_sends()
-> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = GateServer::start().await?;
    let endpoint = gate.endpoint.clone();
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(move |config| {
            config.features.enable(Feature::Collab).unwrap();
            config.features.enable(Feature::MultiAgentV2).unwrap();
            config.features.disable(Feature::CodeModeOnly).unwrap();
            config.agents_enabled = true;
            config.model_provider.stream_idle_timeout_ms = Some(30000);
            for model in &mut config.model_catalog.as_mut().unwrap().models {
                model.multi_agent_version = Some(codex_protocol::protocol::MultiAgentVersion::V2);
                model.tool_mode = None;
            }
            let role_path = config.codex_home.join("accounting-role.toml");
            std::fs::write(&role_path, "model_reasoning_effort = \"low\"\n").unwrap();
            config.agent_roles.insert(
                "accounting-role".into(),
                codex_core::config::AgentRoleConfig {
                    description: Some("synthetic accounting child".into()),
                    config_file: Some(role_path.to_path_buf()),
                    nickname_candidates: None,
                },
            );
        })
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let root = gate.next().await?;
    assert!(root.head.starts_with("POST /v1/messages "));
    let tool = root.body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|tool| {
            tool["name"]
                .as_str()
                .filter(|name| name.ends_with("spawn_agent"))
        })
        .unwrap_or_else(|| {
            panic!(
                "actual native tool names: {:?}",
                root.body["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|tool| &tool["name"])
                    .collect::<Vec<_>>()
            )
        });
    let mut events = vec![start(
        json!({"input_tokens":1,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}),
    )];
    for (index, role) in ["default", "accounting-role"].iter().enumerate() {
        events.push(
            json!({"type":"content_block_start","index":index,"content_block":{
                "type":"tool_use","id":format!("spawn-{index}"),"name":tool,
                "input":{"message":format!("synthetic child {index}"),"task_name":format!("child_{index}"),"agent_type":role,"fork_turns":"none"}
            }}),
        );
        events.push(json!({"type":"content_block_stop","index":index}));
    }
    events.push(json!({"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":1}}));
    events.push(json!({"type":"message_stop"}));
    root.chunks.send(sse(&events)).await?;
    drop(root);
    // Both children and the parent's next sampling request must reach their
    // physical connection while all three response bodies remain held.
    let mut held: Vec<support::GateRequest> = Vec::new();
    for _ in 0..3 {
        let request = match gate.next().await {
            Ok(request) => request,
            Err(error) => {
                let mut errors = Vec::new();
                for id in test.thread_manager.list_thread_ids().await {
                    let native = test.thread_manager.get_thread(id).await?;
                    let config = native.config().await;
                    errors.push(format!(
                        "owner={id}, provider={}, sqlite={}, state={}",
                        config.model_provider_id,
                        config.sqlite.home().display(),
                        native.state_db().is_some()
                    ));
                    while let Ok(Ok(event)) =
                        tokio::time::timeout(Duration::from_millis(10), native.next_event()).await
                    {
                        if let EventMsg::Error(error) = event.msg {
                            errors.push(format!("{id}: {error:?}"));
                        }
                    }
                }
                let results = held
                    .iter()
                    .flat_map(|request| request.body["messages"].as_array().into_iter().flatten())
                    .flat_map(|message| message["content"].as_array().into_iter().flatten())
                    .filter(|block| block["type"] == "tool_result")
                    .collect::<Vec<_>>();
                anyhow::bail!(
                    "{error}; held {} physical requests; {errors:?}; tool results {results:?}",
                    held.len()
                );
            }
        };
        held.push(request);
    }
    let db = test.codex.state_db().unwrap();
    let admitted = attempts(&db).await?;
    assert_eq!(admitted.len(), 4);
    assert_eq!(
        admitted
            .iter()
            .map(|attempt| attempt.request_id)
            .collect::<HashSet<_>>()
            .len(),
        4
    );
    assert_eq!(
        admitted
            .iter()
            .map(|attempt| attempt.attempt_id)
            .collect::<HashSet<_>>()
            .len(),
        4
    );
    assert!(admitted.iter().all(|attempt| attempt.retry_of.is_none()
        && attempt.provider == "anthropic"
        && attempt.model == "claude-opus-5"
        && attempt.dialect == Dialect::NativeAnthropic));
    let mut owned = HashMap::new();
    for attempt in &admitted {
        *owned.entry(attempt.thread_id).or_insert(0) += 1;
    }
    assert_eq!(owned.len(), 3, "admitted {admitted:?}");
    assert_eq!(owned[&test.session_configured.thread_id], 2);
    for request in held {
        request.chunks.send(sse(&[start(json!({"input_tokens":2,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}))])).await?;
        request
            .chunks
            .send(sse(&ending(json!({"output_tokens":1}))))
            .await?;
    }
    let events = terminal(&test).await?;
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_))),
        "{events:?}"
    );
    wait_observations(&db, 8).await?;
    let edges: Vec<(String, String)> = sqlx::query_as(
        "SELECT parent_thread_id, child_thread_id FROM thread_spawn_edges ORDER BY child_thread_id",
    )
    .fetch_all(&mut connection(&db).await?)
    .await?;
    assert_eq!(edges.len(), 2);
    for (parent, child) in &edges {
        assert_eq!(parent, &test.session_configured.thread_id.to_string());
        let id = codex_protocol::ThreadId::from_string(child)?;
        assert_eq!(owned[&id], 1);
        let native = test.thread_manager.get_thread(id).await?;
        assert_eq!(native.config().await.accounting, test.config.accounting);
    }
    assert!(test.thread_manager.list_thread_ids().await.len() >= 3);
    let before = attempts(&db).await?;
    let fork = test
        .thread_manager
        .fork_thread(
            codex_core::ForkSnapshot::Interrupted,
            test.config.clone(),
            test.codex.rollout_path().unwrap(),
            /*thread_source*/ None,
            /*parent_trace*/ None,
        )
        .await?;
    assert!(!owned.contains_key(&fork.thread_id));
    assert_eq!(attempts(&db).await?, before);
    test.thread_manager
        .shutdown_all_threads_bounded(Duration::from_secs(3))
        .await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_native_presence_prices_and_two_reopens() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/v1/messages"))
        .respond_with(success(json!({"input_tokens":7,"cache_read_input_tokens":0,"cache_creation_input_tokens":null,"output_tokens":0}),
            json!({"output_tokens":3}))).expect(1).mount(&server).await;
    let endpoint = format!("{}/v1", server.uri());
    let mode = enabled(&endpoint);
    let test = builder(endpoint.clone(), mode.clone())
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("native accounting presence").await?;
    let db = test.codex.state_db().unwrap();
    let records = attempts(&db).await?;
    assert_eq!(records.len(), 1);
    let attempt = &records[0];
    assert_eq!(
        (
            attempt.thread_id,
            attempt.provider.as_str(),
            attempt.model.as_str(),
            attempt.dialect,
            attempt.retry_of
        ),
        (
            test.session_configured.thread_id,
            "anthropic",
            "claude-opus-5",
            Dialect::NativeAnthropic,
            None
        )
    );
    let patches = observations(&db).await?;
    assert_eq!(
        patches
            .iter()
            .map(|row| (i64::from(row.revision), row.patch.clone()))
            .collect::<Vec<_>>(),
        vec![
            (
                1,
                Patch {
                    input: Presence::Number(7.try_into()?),
                    read: Presence::Number(0.try_into()?),
                    write: Presence::Null,
                    output: Presence::Number(0.try_into()?),
                    ..Default::default()
                }
            ),
            (
                5,
                Patch {
                    output: Presence::Number(3.try_into()?),
                    ..Default::default()
                }
            ),
        ]
    );
    assert_eq!(patches[0].source, patches[1].source);
    let snapshots: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
    assert_eq!(snapshots.len(), 1);
    assert_eq!(
        (
            snapshots[0].scope,
            snapshots[0].source_kind.clone(),
            snapshots[0].rates.write
        ),
        (attempt.scope, SourceKind::NativeCatalog, None)
    );
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&requests[0].body)?["model"],
        "claude-opus-5"
    );
    assert_eq!(requests[0].headers["anthropic-version"], "2023-06-01");
    let home = test.home.clone();
    let rollout = test.codex.rollout_path().unwrap();
    stop(&test).await;
    drop(test);
    drop(db);
    for _ in 0..2 {
        let reopened = builder(endpoint.clone(), mode.clone())
            .resume(&server, home.clone(), rollout.clone())
            .await?;
        let db = reopened.codex.state_db().unwrap();
        assert_eq!(attempts(&db).await?, records);
        assert_eq!(observations(&db).await?, patches);
        assert_eq!(
            payloads::<Snapshot>(&db, "draft_accounting_price_snapshots").await?,
            snapshots
        );
        let as_of = chrono::Utc::now().timestamp_millis();
        let store = AccountingStore::open(&db, as_of).await?;
        let day = store
            .read_day(
                attempt.thread_id,
                i64::from(attempt.dispatched_at_ms) / 86_400_000,
                as_of,
            )
            .await?;
        let RetainedDay::Available {
            totals: Current::Ready(totals),
            ..
        } = day
        else {
            panic!("fresh retained native day: {day:?}");
        };
        assert_eq!(
            totals,
            DayTotals {
                measured: [
                    Metric {
                        known: 0,
                        unknown: 1
                    },
                    Metric {
                        known: 7,
                        unknown: 0
                    },
                    Metric {
                        known: 0,
                        unknown: 0
                    },
                    Metric {
                        known: 0,
                        unknown: 1
                    },
                    Metric {
                        known: 3,
                        unknown: 0
                    },
                    Metric {
                        known: 0,
                        unknown: 1
                    },
                    Metric {
                        known: 0,
                        unknown: 1
                    },
                ],
                known_usd: "0.00011".to_string().try_into()?,
                unknown_estimates: 1,
                attempts: 1,
            }
        );
        stop(&reopened).await;
    }
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_default_off_does_not_install_and_installed_off_deletion_cleans()
-> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(success(
            json!({"input_tokens":0}),
            json!({"output_tokens":0}),
        ))
        .expect(2)
        .mount(&server)
        .await;
    let endpoint = format!("{}/v1", server.uri());
    let test = builder(endpoint, AccountingMode::Disabled)
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("off native turn").await?;
    let db = test.codex.state_db().unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_schema WHERE name GLOB 'draft_accounting_*' OR name = '_accounting_migrations'")
        .fetch_one(&mut connection(&db).await?).await?;
    assert_eq!(count, 0);
    AccountingStore::open(&db, chrono::Utc::now().timestamp_millis()).await?;
    test.submit_turn("still off with installed store").await?;
    assert_eq!(attempts(&db).await?, vec![]);
    let owner = test.session_configured.thread_id;
    stop(&test).await;
    db.delete_thread(owner).await?;
    assert!(db.get_thread(owner).await?.is_none());
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum RoleOverride {
    StreamRetry,
    RequestRetry,
    IdleTimeout,
    Endpoint,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_role_disables_actual_child_stream_retry() -> anyhow::Result<()> {
    role_override_native(RoleOverride::StreamRetry).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_role_disables_actual_child_request_retry() -> anyhow::Result<()> {
    role_override_native(RoleOverride::RequestRetry).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_role_extends_actual_child_idle_timeout() -> anyhow::Result<()> {
    role_override_native(RoleOverride::IdleTimeout).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_role_endpoint_mismatch_has_no_unapproved_send() -> anyhow::Result<()>
{
    role_override_native(RoleOverride::Endpoint).await
}

async fn role_override_native(case: RoleOverride) -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let unapproved = MockServer::start().await;
    let mut gate = GateServer::start().await?;
    let endpoint = gate.endpoint.clone();
    let other_endpoint = format!("{}/v1", unapproved.uri());
    let explicit = match case {
        RoleOverride::StreamRetry => "stream_max_retries = 0".into(),
        RoleOverride::RequestRetry => "request_max_retries = 0\nstream_max_retries = 0".into(),
        RoleOverride::IdleTimeout => "stream_idle_timeout_ms = 10000".into(),
        RoleOverride::Endpoint => format!("base_url = {other_endpoint:?}"),
    };
    let status = if case == RoleOverride::RequestRetry {
        "503"
    } else {
        "200"
    };
    let role_text = format!(
        "[model_providers.anthropic]\nname = \"synthetic role transport\"\n{explicit}\n[model_providers.anthropic.http_headers]\nx-accounting-role = \"child\"\nx-accounting-fixture-status = {status:?}\n"
    );
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(move |config| {
            config.features.enable(Feature::Collab).unwrap();
            config.features.enable(Feature::MultiAgentV2).unwrap();
            config.features.disable(Feature::CodeModeOnly).unwrap();
            config.agents_enabled = true;
            config.model_provider.stream_idle_timeout_ms = Some(1000);
            for model in &mut config.model_catalog.as_mut().unwrap().models {
                model.multi_agent_version = Some(codex_protocol::protocol::MultiAgentVersion::V2);
                model.tool_mode = None;
            }
            let role_path = config.codex_home.join("accounting-transport-role.toml");
            std::fs::write(&role_path, &role_text).unwrap();
            config.agent_roles.insert(
                "transport-role".into(),
                codex_core::config::AgentRoleConfig {
                    description: Some("synthetic transport override".into()),
                    config_file: Some(role_path.to_path_buf()),
                    nickname_candidates: None,
                },
            );
        })
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let root = gate.next().await?;
    let tool = root.body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|tool| {
            tool["name"]
                .as_str()
                .filter(|name| name.ends_with("spawn_agent"))
        })
        .expect("native spawn tool");
    let spawn = vec![
        start(json!({"input_tokens":1})),
        json!({"type":"content_block_start","index":0,"content_block":{
            "type":"tool_use","id":"role-spawn","name":tool,
            "input":{"message":"synthetic role child","task_name":"role_child",
                "agent_type":"transport-role","fork_turns":"none"}
        }}),
        json!({"type":"content_block_stop","index":0}),
        json!({"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":1}}),
        json!({"type":"message_stop"}),
    ];
    root.chunks.send(sse(&spawn)).await?;
    drop(root);
    let mut child_request = None;
    let expected_requests = if case == RoleOverride::Endpoint { 1 } else { 2 };
    for _ in 0..expected_requests {
        let request = gate.next().await?;
        assert!(request.head.starts_with("POST /v1/messages "));
        assert_eq!(request.body["model"], "claude-opus-5");
        if request
            .head
            .to_ascii_lowercase()
            .contains("x-accounting-role: child")
        {
            assert!(child_request.is_none());
            child_request = Some(request);
        } else {
            let mut response = vec![start(json!({"input_tokens":2}))];
            response.extend(ending(json!({"output_tokens":2})));
            request.chunks.send(sse(&response)).await?;
        }
    }
    let db = test.codex.state_db().unwrap();
    let edges: Vec<(String, String)> =
        sqlx::query_as("SELECT parent_thread_id, child_thread_id FROM thread_spawn_edges")
            .fetch_all(&mut connection(&db).await?)
            .await?;
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].0, test.session_configured.thread_id.to_string());
    let child_id = codex_protocol::ThreadId::from_string(&edges[0].1)?;
    let native = test.thread_manager.get_thread(child_id).await?;
    let child_config = native.config().await;
    assert_eq!(child_config.accounting, test.config.accounting);
    let mut expected_provider = test.config.model_provider.clone();
    expected_provider.name = "synthetic role transport".into();
    expected_provider
        .http_headers
        .get_or_insert_default()
        .extend([
            ("x-accounting-role".into(), "child".into()),
            ("x-accounting-fixture-status".into(), status.into()),
        ]);
    match case {
        RoleOverride::StreamRetry => expected_provider.stream_max_retries = Some(0),
        RoleOverride::RequestRetry => {
            expected_provider.request_max_retries = Some(0);
            expected_provider.stream_max_retries = Some(0);
        }
        RoleOverride::IdleTimeout => expected_provider.stream_idle_timeout_ms = Some(10000),
        RoleOverride::Endpoint => expected_provider.base_url = Some(other_endpoint),
    }
    assert_eq!(child_config.model_provider, expected_provider);
    assert_eq!(
        test.codex.config().await.model_provider,
        test.config.model_provider
    );
    match case {
        RoleOverride::StreamRetry => {
            let request = child_request.take().expect("child physical send");
            request
                .chunks
                .send(sse(&[start(json!({"input_tokens":9}))]))
                .await?;
            wait_observations(&db, 5).await?;
            drop(request); // EOF after durable usage must not cause a role-disabled retry.
        }
        RoleOverride::IdleTimeout => {
            // Longer than the unchanged parent's 1s deadline, shorter than the
            // explicit child's 10s deadline. A premature retry adds an attempt.
            tokio::time::sleep(Duration::from_secs(2)).await;
            assert_eq!(attempts(&db).await?.len(), 3);
            let request = child_request.take().expect("held child response");
            let mut response = vec![start(json!({"input_tokens":9}))];
            response.extend(ending(json!({"output_tokens":3})));
            request.chunks.send(sse(&response)).await?;
        }
        RoleOverride::RequestRetry => {
            assert!(child_request.take().is_some()); // Server emitted actual HTTP503.
        }
        RoleOverride::Endpoint => assert!(child_request.is_none()),
    }
    let child_events = tokio::time::timeout(Duration::from_secs(15), async {
        let mut events = Vec::new();
        loop {
            let event = native.next_event().await?.msg;
            let finished = matches!(event, EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_));
            events.push(event);
            if finished {
                return anyhow::Ok(events);
            }
        }
    })
    .await??;
    assert_eq!(
        child_events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_))),
        case != RoleOverride::IdleTimeout,
        "{child_events:?}"
    );
    let root_events = terminal(&test).await?;
    assert!(
        !root_events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_)))
    );
    let records = attempts(&db).await?;
    let mut owned = HashMap::new();
    for record in &records {
        *owned.entry(record.thread_id).or_insert(0) += 1;
        assert!(record.retry_of.is_none());
    }
    let mut expected_owners = HashMap::from([(test.session_configured.thread_id, 2)]);
    if case != RoleOverride::Endpoint {
        expected_owners.insert(child_id, 1);
    }
    assert_eq!(owned, expected_owners);
    assert_eq!(
        records
            .iter()
            .map(|row| row.request_id)
            .collect::<HashSet<_>>()
            .len(),
        records.len()
    );
    assert_eq!(
        records
            .iter()
            .map(|row| row.attempt_id)
            .collect::<HashSet<_>>()
            .len(),
        records.len()
    );
    assert!(unapproved.received_requests().await.unwrap().is_empty());
    assert!(
        gate.incoming.try_recv().is_err(),
        "no extra retry or unowned request"
    );
    stop(&test).await;
    Ok(())
}

struct Retry {
    calls: AtomicUsize,
    status: u16,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_actual_presence_revisions_and_remote_only_price_unknown()
-> anyhow::Result<()> {
    for remote_only in [false, true] {
        let server = MockServer::start().await;
        let mut events = vec![start(json!({}))];
        let updates = [
            json!({"input_tokens":null,"cache_read_input_tokens":null,"cache_creation_input_tokens":null,"output_tokens":null}),
            json!({"input_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"output_tokens":0}),
            json!({"input_tokens":7}),
            json!({"cache_read_input_tokens":2,"cache_creation_input_tokens":4}),
            json!({"output_tokens":3}),
            json!({"input_tokens":null,"cache_read_input_tokens":null,"cache_creation_input_tokens":null,"output_tokens":null}),
        ];
        for usage in updates {
            events.push(json!({"type":"message_delta","delta":{},"usage":usage}));
        }
        events.extend(ending(json!({"output_tokens":5})));
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(sse(&events), "text/event-stream"),
            )
            .expect(1)
            .mount(&server)
            .await;
        let endpoint = format!("{}/v1", server.uri());
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(move |config| {
                if remote_only {
                    let catalog = config.model_catalog.as_mut().unwrap();
                    let mut row = catalog
                        .models
                        .iter()
                        .find(|row| row.slug == "claude-opus-5")
                        .unwrap()
                        .clone();
                    row.slug = "remote-only-accounting-fixture".into();
                    config.model = Some(row.slug.clone());
                    catalog.models.push(row);
                }
            })
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let events = terminal(&test).await?;
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, EventMsg::Error(_))),
            "{events:?}"
        );
        let db = test.codex.state_db().unwrap();
        let rows = observations(&db).await?;
        let expected = [
            json!({}),
            json!({"input":null,"read":null,"write":null,"output":null}),
            json!({"input":0,"read":0,"write":0,"output":0}),
            json!({"input":7}),
            json!({"read":2,"write":4}),
            json!({"output":3}),
            json!({"input":null,"read":null,"write":null,"output":null}),
            json!({"output":5}),
        ];
        let expected = expected
            .into_iter()
            .map(serde_json::from_value::<Patch>)
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            rows.iter().map(|row| row.patch.clone()).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            rows.iter()
                .map(|row| i64::from(row.sequence))
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 11]
        );
        assert!(
            rows.iter()
                .all(|row| row.revision == row.sequence && row.source == rows[0].source)
        );
        let records = attempts(&db).await?;
        assert_eq!(records.len(), 1);
        let snapshots: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
        assert_eq!(snapshots.len(), usize::from(!remote_only));
        let as_of = chrono::Utc::now().timestamp_millis();
        let store = AccountingStore::open(&db, as_of).await?;
        let RetainedDay::Available {
            totals: Current::Ready(totals),
            ..
        } = store
            .read_day(
                records[0].thread_id,
                i64::from(records[0].dispatched_at_ms) / 86_400_000,
                as_of,
            )
            .await?
        else {
            panic!("fresh complete native day");
        };
        assert_eq!(
            totals,
            DayTotals {
                measured: [
                    Metric {
                        known: 13,
                        unknown: 0
                    },
                    Metric {
                        known: 7,
                        unknown: 0
                    },
                    Metric {
                        known: 2,
                        unknown: 0
                    },
                    Metric {
                        known: 4,
                        unknown: 0
                    },
                    Metric {
                        known: 5,
                        unknown: 0
                    },
                    Metric {
                        known: 0,
                        unknown: 1
                    },
                    Metric {
                        known: 18,
                        unknown: 0
                    }
                ],
                known_usd: if remote_only { "0" } else { "0.000161" }
                    .to_string()
                    .try_into()?,
                unknown_estimates: 1,
                attempts: 1,
            }
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &server.received_requests().await.unwrap()[0].body
            )?["model"],
            records[0].model
        );
        stop(&test).await;
    }
    Ok(())
}

impl wiremock::Respond for Retry {
    fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
        if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
            ResponseTemplate::new(self.status).set_body_json(
                json!({"type":"error","error":{"type":"request_too_large","message":"fixture"}}),
            )
        } else {
            success(
                json!({"input_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}),
                json!({"output_tokens":2}),
            )
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_http_and_payload_retries_are_separate_durable_attempts()
-> anyhow::Result<()> {
    for status in [503, 413] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(Retry {
                calls: AtomicUsize::new(0),
                status,
            })
            .expect(2)
            .mount(&server)
            .await;
        let endpoint = format!("{}/v1", server.uri());
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let events = terminal(&test).await?;
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, EventMsg::Error(_))),
            "HTTP {status}: {events:?}"
        );
        let db = test.codex.state_db().unwrap();
        let rows = attempts(&db).await?;
        assert_eq!(rows.len(), 2, "HTTP {status}");
        assert_ne!(rows[0].attempt_id, rows[1].attempt_id);
        assert_eq!(
            (rows[1].request_id, rows[1].thread_id, rows[1].retry_of),
            (
                rows[0].request_id,
                rows[0].thread_id,
                Some(rows[0].attempt_id)
            )
        );
        let observed: Vec<String> =
            sqlx::query_scalar("SELECT DISTINCT attempt_id FROM draft_accounting_observations")
                .fetch_all(&mut connection(&db).await?)
                .await?;
        assert_eq!(observed, vec![rows[1].attempt_id.to_string()]);
        stop(&test).await;
    }
    Ok(())
}
