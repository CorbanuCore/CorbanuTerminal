use super::accounting_responses::support::*;
use codex_core::config::AccountingMode;
use codex_protocol::protocol::{EventMsg, Op};
use codex_state::accounting::*;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn accounting_responses_native_http_retry_policy() -> anyhow::Result<()> {
    for (status, expected) in [(503, 2), (429, 1)] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(ResponseTemplate::new(status))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(success(usage(Some(0))), "text/event-stream"),
            )
            .with_priority(2)
            .mount(&server)
            .await;
        let endpoint = format!("{}/v1", server.uri());
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        terminal(&test).await?;
        let db = test.codex.state_db().unwrap();
        let records = attempts(&db).await?;
        assert_eq!(records.len(), expected);
        assert_eq!(server.received_requests().await.unwrap().len(), expected);
        assert_eq!(records[0].retry_of, None);
        if expected == 2 {
            assert_eq!(records[1].retry_of, Some(records[0].attempt_id));
            assert_eq!(records[1].request_id, records[0].request_id);
        }
        assert_eq!(totals(&db, &records[0]).await?.unknown_estimates, 1);
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_api_key_401() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let endpoint = format!("{}/v1", server.uri());
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    assert!(
        terminal(&test)
            .await?
            .iter()
            .any(|e| matches!(e, EventMsg::Error(_)))
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    let db = test.codex.state_db().unwrap();
    assert_eq!(attempts(&db).await?.len(), 1);
    assert!(observations(&db).await?.is_empty());
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_redirects() -> anyhow::Result<()> {
    for status in [301, 302, 303, 307, 308] {
        for on in [true, false] {
            let origin = MockServer::start().await;
            let target = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(status)
                        .insert_header("location", format!("{}/v1/responses", target.uri())),
                )
                .mount(&origin)
                .await;
            Mock::given(path("/v1/responses"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(success(usage(Some(0))), "text/event-stream"),
                )
                .mount(&target)
                .await;
            let endpoint = format!("{}/v1", origin.uri());
            let mode = if on {
                enabled(&endpoint)
            } else {
                AccountingMode::Disabled
            };
            let test = builder(endpoint, mode).build_with_auto_env(&origin).await?;
            submit(&test).await?;
            terminal(&test).await?;
            assert_eq!(origin.received_requests().await.unwrap().len(), 1);
            assert_eq!(
                target.received_requests().await.unwrap().len(),
                usize::from(!on)
            );
            let db = test.codex.state_db().unwrap();
            if on {
                assert_eq!(attempts(&db).await?.len(), 1);
                assert!(observations(&db).await?.is_empty());
            } else {
                absent(&db).await?;
            }
            stop(&test).await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_outer_retry_prefix() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let first = gate.next().await?;
    first
        .chunks
        .send(responses::sse(vec![event(
            "response.usage",
            usage(Some(0)),
        )]))
        .await?;
    let db = test.codex.state_db().unwrap();
    wait_observations(&db, 1).await?;
    drop(first);
    let second = gate.next().await?;
    second.chunks.send(success(usage(Some(0)))).await?;
    terminal(&test).await?;
    let records = attempts(&db).await?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].retry_of, Some(records[0].attempt_id));
    assert_eq!(records[1].request_id, records[0].request_id);
    let patches = observations(&db).await?;
    assert_ne!(patches[0].source, patches[1].source);
    let day = totals(&db, &records[0]).await?;
    assert_eq!(day.known_usd, "0.00322".to_string().try_into()?);
    assert_eq!(
        day.measured[6],
        Metric {
            known: 280,
            unknown: 0
        }
    );
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_observation_failure() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let held = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    held.chunks
        .send(responses::sse(vec![event(
            "response.usage",
            json!({"input_tokens":7}),
        )]))
        .await?;
    wait_observations(&db, 1).await?;
    let before = observations(&db).await?;
    sqlx::query("CREATE TRIGGER reject_responses_observation BEFORE INSERT ON draft_accounting_observations BEGIN SELECT RAISE(ABORT, 'fixture'); END")
        .execute(&mut connection(&db).await?).await?;
    held.chunks.send(success(usage(Some(0)))).await?;
    assert!(
        terminal(&test)
            .await?
            .iter()
            .any(|e| matches!(e, EventMsg::Error(_)))
    );
    assert_eq!(observations(&db).await?, before);
    assert_eq!(attempts(&db).await?.len(), 1);
    gate.no_pending();
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_admission_barrier() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    let db = test.codex.state_db().unwrap();
    let mut lock = connection(&db).await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut lock).await?;
    submit(&test).await?;
    assert!(
        tokio::time::timeout(Duration::from_millis(100), gate.next())
            .await
            .is_err()
    );
    sqlx::query("ROLLBACK").execute(&mut lock).await?;
    let held = gate.next().await?;
    assert_eq!(attempts(&db).await?.len(), 1);
    held.chunks.send(success(usage(Some(0)))).await?;
    terminal(&test).await?;
    sqlx::query("CREATE TRIGGER reject_responses_attempt BEFORE INSERT ON draft_accounting_attempts BEGIN SELECT RAISE(ABORT, 'fixture'); END").execute(&mut lock).await?;
    submit(&test).await?;
    assert!(
        terminal(&test)
            .await?
            .iter()
            .any(|e| matches!(e, EventMsg::Error(_)))
    );
    assert_eq!(attempts(&db).await?.len(), 1);
    gate.no_pending();
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_cancel_two_reopens() -> anyhow::Result<()> {
    for prefix in [false, true] {
        let server = MockServer::start().await;
        let mut gate = Gate::start().await?;
        let endpoint = gate.endpoint.clone();
        let mode = enabled(&endpoint);
        let test = builder(endpoint.clone(), mode.clone())
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let held = gate.next().await?;
        let db = test.codex.state_db().unwrap();
        if prefix {
            held.chunks
                .send(responses::sse(vec![event(
                    "response.usage",
                    usage(Some(0)),
                )]))
                .await?;
            wait_observations(&db, 1).await?;
        }
        test.codex.submit(Op::Interrupt).await?;
        assert!(
            terminal(&test)
                .await?
                .iter()
                .any(|e| matches!(e, EventMsg::TurnAborted(_)))
        );
        drop(held);
        let records = attempts(&db).await?;
        let patches = observations(&db).await?;
        let prices: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
        let home = test.home.clone();
        let rollout = test.codex.rollout_path().unwrap();
        stop(&test).await;
        drop(test);
        db.close().await;
        for index in 0..2 {
            let reopened = builder(endpoint.clone(), mode.clone())
                .resume(&server, home.clone(), rollout.clone())
                .await?;
            let db = reopened.codex.state_db().unwrap();
            assert_eq!(attempts(&db).await?, records);
            assert_eq!(observations(&db).await?, patches);
            assert_eq!(
                payloads::<Snapshot>(&db, "draft_accounting_price_snapshots").await?,
                prices
            );
            gate.no_pending();
            if index == 1 {
                submit(&reopened).await?;
                gate.next()
                    .await?
                    .chunks
                    .send(success(usage(Some(0))))
                    .await?;
                terminal(&reopened).await?;
                let fresh = attempts(&db).await?;
                assert_ne!(fresh[1].request_id, records[0].request_id);
                assert_eq!(fresh[1].retry_of, None);
            }
            stop(&reopened).await;
            drop(reopened);
            db.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_delete_rejects_late_usage() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let held = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    assert_eq!(attempts(&db).await?.len(), 1);
    db.delete_thread(test.session_configured.thread_id).await?;
    held.chunks.send(success(usage(Some(0)))).await?;
    terminal(&test).await?;
    assert!(attempts(&db).await?.is_empty());
    assert!(observations(&db).await?.is_empty());
    gate.no_pending();
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_spawned_role_children() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|config| {
            for feature in [
                codex_features::Feature::Collab,
                codex_features::Feature::MultiAgentV2,
            ] {
                config.features.enable(feature).unwrap();
            }
            config
                .features
                .disable(codex_features::Feature::CodeModeOnly)
                .unwrap();
            config.agents_enabled = true;
            config.model_provider.stream_idle_timeout_ms = Some(30000);
            for model in &mut config.model_catalog.as_mut().unwrap().models {
                model.multi_agent_version = Some(codex_protocol::protocol::MultiAgentVersion::V2);
                model.tool_mode = None;
            }
            let role = config.codex_home.join("responses-child.toml");
            std::fs::write(
                &role,
                "[model_providers.openai]\nname = \"fixture child\"\nrequest_max_retries = 0\n",
            )
            .unwrap();
            config.agent_roles.insert(
                "fixture".into(),
                codex_core::config::AgentRoleConfig {
                    config_file: Some(role.to_path_buf()),
                    description: None,
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
        .filter_map(|v| v["name"].as_str())
        .find(|name| name.ends_with("spawn_agent"))
        .unwrap();
    let mut events = vec![responses::ev_response_created("root")];
    for (index, role) in ["default", "fixture"].iter().enumerate() {
        events.push(responses::ev_function_call(&format!("spawn-{index}"), tool,
            &json!({"task_name":format!("child_{index}"),"message":"fixture","agent_type":role,"fork_turns":"none"}).to_string()));
    }
    events.push(event("response.completed", usage(Some(0))));
    root.chunks.send(responses::sse(events)).await?;
    let mut held = Vec::new();
    for _ in 0..3 {
        held.push(gate.next().await?);
    }
    let db = test.codex.state_db().unwrap();
    let records = attempts(&db).await?;
    assert_eq!(records.len(), 4);
    let owners: std::collections::HashSet<_> = records.iter().map(|a| a.thread_id).collect();
    assert_eq!(owners.len(), 3);
    let edges: Vec<(String, String)> =
        sqlx::query_as("SELECT parent_thread_id, child_thread_id FROM thread_spawn_edges")
            .fetch_all(&mut connection(&db).await?)
            .await?;
    assert_eq!(edges.len(), 2);
    for (parent, child) in edges {
        assert_eq!(parent, test.session_configured.thread_id.to_string());
        assert!(owners.contains(&codex_protocol::ThreadId::from_string(&child)?));
    }
    for held in held {
        held.chunks.send(success(usage(Some(0)))).await?;
    }
    terminal(&test).await?;
    wait_observations(&db, 4).await?;
    let before = attempts(&db).await?;
    test.thread_manager
        .fork_thread(
            codex_core::ForkSnapshot::Interrupted,
            test.config.clone(),
            test.codex.rollout_path().unwrap(),
            None,
            None,
        )
        .await?;
    assert_eq!(attempts(&db).await?, before);
    stop(&test).await;
    Ok(())
}
