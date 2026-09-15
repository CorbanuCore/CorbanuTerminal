use super::accounting_chat::support::*;
use codex_core::config::AccountingMode;
use codex_protocol::protocol::{EventMsg, Op};
use codex_state::accounting::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn accounting_chat_native_http_retry_policy() -> anyhow::Result<()> {
    for (status, expected) in [(503, 2), (429, 1)] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(status))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(success(usage()), "text/event-stream"),
            )
            .with_priority(2)
            .mount(&server)
            .await;
        let endpoint = format!("{}/v1", server.uri());
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(|config| config.model_provider.request_max_retries = Some(1))
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
        assert_eq!(totals(&db, &records[0]).await?.unknown_estimates, expected as i64);
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_api_key_401_no_invented_refresh() -> anyhow::Result<()> {
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
async fn accounting_chat_native_redirects_no_follow_or_repair() -> anyhow::Result<()> {
    for status in [301, 302, 303, 307, 308] {
        for on in [true, false] {
            if !on && !matches!(status, 307 | 308) { continue; }
            let origin = MockServer::start().await;
            let target = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(status)
                        .insert_header("location", format!("{}/v1/chat/completions", target.uri())),
                )
                .mount(&origin)
                .await;
            Mock::given(path("/v1/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_raw(success(usage()), "text/event-stream"),
                )
                .mount(&target)
                .await;
            let endpoint = format!("{}/v1", origin.uri());
            let mode = if on {
                enabled(&endpoint)
            } else {
                AccountingMode::Disabled
            };
            let test = builder(endpoint, mode)
                .with_config(|config| config.model_provider.stream_max_retries = Some(1))
                .build_with_auto_env(&origin).await?;
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
async fn accounting_chat_native_outer_retry_prefix_and_ids() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|config| config.model_provider.stream_max_retries = Some(1))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let first = gate.next().await?;
    first
        .chunks
        .send(event(usage()))
        .await?;
    let db = test.codex.state_db().unwrap();
    wait_observations(&db, 1).await?;
    drop(first);
    let second = gate.next().await?;
    second.chunks.send(success(usage())).await?;
    terminal(&test).await?;
    let records = attempts(&db).await?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].retry_of, Some(records[0].attempt_id));
    assert_eq!(records[1].request_id, records[0].request_id);
    let patches = observations(&db).await?;
    assert_ne!(patches[0].source, patches[1].source);
    let day = totals(&db, &records[0]).await?;
    assert_eq!(day, golden(true, false, 2)?);
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
async fn accounting_chat_native_observation_failure_no_repair() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|config| config.model_provider.stream_max_retries = Some(1))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let held = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    held.chunks
        .send(event(usage()))
        .await?;
    wait_observations(&db, 1).await?;
    let before = observations(&db).await?;
    sqlx::query("CREATE TRIGGER reject_responses_observation BEFORE INSERT ON draft_accounting_observations BEGIN SELECT RAISE(ABORT, 'fixture'); END")
        .execute(&mut connection(&db).await?).await?;
    held.chunks.send(success(usage())).await?;
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
async fn accounting_chat_native_admission_barrier_and_failure() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|config| config.model_provider.stream_max_retries = Some(1))
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
    held.chunks.send(success(usage())).await?;
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
async fn accounting_chat_native_cancellation_and_two_reopens() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint)).build_with_auto_env(&server).await?;
    let db = test.codex.state_db().unwrap();
    let mut lock = connection(&db).await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut lock).await?;
    submit(&test).await?;
    assert!(tokio::time::timeout(Duration::from_millis(100), gate.next()).await.is_err());
    test.codex.submit(Op::Interrupt).await?;
    terminal(&test).await?;
    sqlx::query("ROLLBACK").execute(&mut lock).await?;
    gate.no_pending();
    assert!(attempts(&db).await.unwrap_or_default().is_empty());
    stop(&test).await;
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
                .send(event(usage()))
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
                    .send(success(usage()))
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
async fn accounting_chat_native_delete_rejects_late_usage() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|config| config.model_provider.stream_max_retries = Some(1))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let held = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    assert_eq!(attempts(&db).await?.len(), 1);
    let other = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server).await?;
    submit(&other).await?;
    gate.next().await?.chunks.send(success(usage())).await?;
    terminal(&other).await?;
    let other_db = other.codex.state_db().unwrap();
    let other_rows = attempts(&other_db).await?;
    let other_prices: Vec<Snapshot> = payloads(&other_db, "draft_accounting_price_snapshots").await?;
    db.delete_thread(test.session_configured.thread_id).await?;
    held.chunks.send(success(usage())).await?;
    terminal(&test).await?;
    assert!(attempts(&db).await?.is_empty());
    assert!(observations(&db).await?.is_empty());
    assert_eq!(attempts(&other_db).await?, other_rows);
    assert_eq!(payloads::<Snapshot>(&other_db, "draft_accounting_price_snapshots").await?, other_prices);
    let home = other.home.clone();
    let rollout = other.codex.rollout_path().unwrap();
    let owner = other.session_configured.thread_id;
    stop(&other).await;
    drop(other);
    other_db.close().await;
    let off = builder(gate.endpoint.clone(), AccountingMode::Disabled)
        .resume(&server,home,rollout).await?;
    let off_db = off.codex.state_db().unwrap();
    off_db.delete_thread(owner).await?;
    assert!(attempts(&off_db).await?.is_empty());
    gate.no_pending();
    stop(&off).await;
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_spawned_role_children_and_fork() -> anyhow::Result<()> {
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
            config.model_provider.stream_max_retries = Some(1);
            for model in &mut config.model_catalog.as_mut().unwrap().models {
                model.multi_agent_version = Some(codex_protocol::protocol::MultiAgentVersion::V2);
                model.tool_mode = None;
                model.use_responses_lite = false;
            }
            let role = config.codex_home.join("responses-child.toml");
            std::fs::write(
                &role,
                "developer_instructions = \"responses role fixture\"\n",
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
    let tool = root.body["tools"].as_array().unwrap().iter()
        .find_map(|item| item["function"]["name"].as_str().filter(|n| n.ends_with("spawn_agent")))
        .expect("native Chat spawn tool");
    let calls: Vec<_> = ["default", "fixture"].iter().enumerate().map(|(index, role)| {
        let arguments = json!({"task_name":format!("child_{index}"),"message":"fixture","agent_type":role,"fork_turns":"none"}).to_string();
        json!({"index":index,"id":format!("spawn-{index}"),"type":"function","function":{"name":tool,"arguments":arguments}})
    }).collect();
    root.chunks.send(data(json!({"choices":[{"index":0,"delta":{"role":"assistant","tool_calls":calls},"finish_reason":"tool_calls"}],"usage":usage()})) + "data: [DONE]\n\n").await?;
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
    let role_index = held
        .iter()
        .position(|request| request.body.to_string().contains("responses role fixture"))
        .expect("reloaded role instructions in a native child request");
    let role = held.remove(role_index);
    role.chunks.send(event(usage())).await?;
    wait_observations(&db, 2).await?;
    drop(role); // EOF forces the role child's native stream retry.
    let retried = gate.next().await?;
    assert!(retried.body.to_string().contains("responses role fixture"));
    let retries = attempts(&db).await?;
    assert_eq!(retries.len(), 5);
    assert!(records.iter().all(|attempt| attempt.retry_of.is_none()));
    let retry = retries.last().unwrap();
    let predecessor = records
        .iter()
        .find(|attempt| Some(attempt.attempt_id) == retry.retry_of)
        .expect("child retry references its own admitted predecessor");
    assert_ne!(retry.thread_id, test.session_configured.thread_id);
    assert_eq!(
        (retry.thread_id, retry.request_id),
        (predecessor.thread_id, predecessor.request_id)
    );
    held.push(retried);
    for held in held {
        held.chunks.send(success(usage())).await?;
    }
    terminal(&test).await?;
    wait_observations(&db, 5).await?;
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

#[tokio::test]
async fn accounting_chat_native_invalid_evidence_no_repair() -> anyhow::Result<()> {
    for bad in ["malformed".to_string(), json!({"usage":{"prompt_tokens":-1}}).to_string(),
        json!({"usage":{"prompt_tokens":100,"prompt_tokens_details":{"cached_tokens":101}}}).to_string(),
        json!({"usage":{"completion_tokens":40,"completion_tokens_details":{"reasoning_tokens":41}}}).to_string(),
        json!({"usage":{"prompt_tokens":100,"completion_tokens":40,"total_tokens":139}}).to_string()] {
        let server = MockServer::start().await;
        let mut gate = Gate::start().await?;
        let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
            .with_config(|c| c.model_provider.stream_max_retries = Some(1))
            .build_with_auto_env(&server).await?;
        submit(&test).await?;
        let held = gate.next().await?;
        held.chunks.send(event(usage())).await?;
        let db = test.codex.state_db().unwrap();
        wait_observations(&db, 1).await?;
        let before = observations(&db).await?;
        held.chunks.send(format!("data: {bad}\n\ndata: [DONE]\n\n")).await?;
        assert!(terminal(&test).await?.iter().any(|e| matches!(e, EventMsg::Error(_))));
        assert_eq!(observations(&db).await?, before);
        assert_eq!(attempts(&db).await?.len(), 1);
        gate.no_pending();
        stop(&test).await;
    }
    Ok(())
}
