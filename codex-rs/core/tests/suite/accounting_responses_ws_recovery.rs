use super::accounting_responses_ws_support::*;
use codex_protocol::protocol::{EventMsg, Op};
use codex_state::accounting::*;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::time::Duration;
use wiremock::MockServer;

#[tokio::test]
async fn accounting_responses_ws_native_admission_and_guard_barriers() -> anyhow::Result<()> {
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
        tokio::time::timeout(Duration::from_millis(150), gate.next())
            .await
            .is_err()
    );
    assert_eq!(gate.counts.lock().unwrap().2, 0);
    sqlx::query("ROLLBACK").execute(&mut lock).await?;
    gate.next().await?.complete().await?;
    terminal(&test).await?;
    sqlx::query("CREATE TRIGGER reject_ws_attempt BEFORE INSERT ON draft_accounting_attempts BEGIN SELECT RAISE(ABORT, 'fixture'); END").execute(&mut lock).await?;
    submit(&test).await?;
    assert!(
        terminal(&test)
            .await?
            .iter()
            .any(|e| matches!(e, EventMsg::Error(_)))
    );
    gate.no_pending().await;
    assert_eq!(attempts(&db).await?.len(), 1);
    let guard = test
        .codex
        .stage_one_memory_client(codex_protocol::ThreadId::new(), &test.config.model_provider)
        .await;
    assert!(guard.is_err());
    gate.no_pending().await;
    stop(&test).await;
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_native_observation_failure_no_repair() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let held = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    held.send(vec![event("response.usage", usage(Some(0)))])
        .await?;
    wait_observations(&db, 1).await?;
    let before = observations(&db).await?;
    sqlx::query("CREATE TRIGGER reject_ws_observation BEFORE INSERT ON draft_accounting_observations BEGIN SELECT RAISE(ABORT, 'fixture'); END").execute(&mut connection(&db).await?).await?;
    held.complete().await?;
    assert!(
        terminal(&test)
            .await?
            .iter()
            .any(|e| matches!(e, EventMsg::Error(_)))
    );
    gate.no_pending().await;
    assert_eq!(observations(&db).await?, before);
    assert_eq!(attempts(&db).await?.len(), 1);
    counts(&gate, (1, 1, 1, 0));
    stop(&test).await;
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_native_cancel_before_and_after_dispatch() -> anyhow::Result<()> {
    for dispatched in [false, true] {
        let server = MockServer::start().await;
        let mut gate = Gate::start().await?;
        let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
            .build_with_auto_env(&server)
            .await?;
        let db = test.codex.state_db().unwrap();
        let mut lock = connection(&db).await?;
        if !dispatched {
            sqlx::query("BEGIN IMMEDIATE").execute(&mut lock).await?;
        }
        submit(&test).await?;
        let held = if dispatched {
            Some(gate.next().await?)
        } else {
            assert!(
                tokio::time::timeout(Duration::from_millis(150), gate.next())
                    .await
                    .is_err()
            );
            None
        };
        test.codex.submit(Op::Interrupt).await?;
        assert!(
            terminal(&test)
                .await?
                .iter()
                .any(|e| matches!(e, EventMsg::TurnAborted(_)))
        );
        if !dispatched {
            sqlx::query("ROLLBACK").execute(&mut lock).await?;
        }
        drop(held);
        gate.no_pending().await;
        assert_eq!(gate.counts.lock().unwrap().2, usize::from(dispatched));
        if dispatched {
            assert_eq!(attempts(&db).await?.len(), 1);
            assert!(observations(&db).await?.is_empty());
        }
        stop(&test).await;
    }
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_native_two_reopens_and_original_prices() -> anyhow::Result<()> {
    for (prefix, model) in [
        (false, "gpt-5.6-sol"),
        (true, "gpt-5.6-sol"),
        (true, "gpt-6-astra"),
    ] {
        let server = MockServer::start().await;
        let mut gate = Gate::start().await?;
        let mode = enabled(&gate.endpoint);
        let test = builder(gate.endpoint.clone(), mode.clone())
            .with_config(move |config| config.model = Some(model.into()))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let held = gate.next().await?;
        let db = test.codex.state_db().unwrap();
        if prefix {
            held.send(vec![
                event("response.usage", usage(Some(0))),
                json!({"type":"response.created","response":{"id":"same"}}),
                event("response.usage", usage(Some(0))),
            ])
            .await?;
            wait_observations(&db, 2).await?;
        }
        test.codex.submit(Op::Interrupt).await?;
        terminal(&test).await?;
        drop(held);
        let records = attempts(&db).await?;
        let patches = observations(&db).await?;
        let prices: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
        let before = totals(&db, &records[0]).await?;
        if prefix {
            assert_eq!(
                before.known_usd,
                if model == "gpt-5.6-sol" {
                    "0.00161"
                } else {
                    "0"
                }
                .to_string()
                .try_into()?
            );
        }
        assert_eq!(prices.is_empty(), model == "gpt-6-astra");
        let home = test.home.clone();
        let rollout = test.codex.rollout_path().unwrap();
        stop(&test).await;
        drop(test);
        db.close().await;
        for index in 0..2 {
            let reopened = builder(gate.endpoint.clone(), mode.clone())
                .with_config(move |config| config.model = Some(model.into()))
                .resume(&server, home.clone(), rollout.clone())
                .await?;
            let db = reopened.codex.state_db().unwrap();
            assert_eq!(attempts(&db).await?, records);
            assert_eq!(observations(&db).await?, patches);
            assert_eq!(
                payloads::<Snapshot>(&db, "draft_accounting_price_snapshots").await?,
                prices
            );
            assert_eq!(totals(&db, &records[0]).await?, before);
            gate.no_pending().await;
            if index == 1 {
                submit(&reopened).await?;
                gate.next().await?.complete().await?;
                terminal(&reopened).await?;
                let fresh = attempts(&db).await?;
                assert_eq!(fresh.len(), 2);
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
async fn accounting_responses_ws_native_spawned_role_children_and_fork() -> anyhow::Result<()> {
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
            let role = config.codex_home.join("ws-child.toml");
            std::fs::write(&role, "developer_instructions = \"ws role fixture\"\n").unwrap();
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
    let (namespace, tool) = root.body["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|item| {
            if item["name"]
                .as_str()
                .is_some_and(|name| name.ends_with("spawn_agent"))
            {
                return Some((None, item["name"].as_str().unwrap()));
            }
            item["tools"].as_array()?.iter().find_map(|tool| {
                let name = tool["name"].as_str()?;
                name.ends_with("spawn_agent")
                    .then_some((item["name"].as_str(), name))
            })
        })
        .expect("native spawn tool");
    let mut events = vec![responses::ev_response_created("root")];
    for (index, role) in ["default", "fixture"].iter().enumerate() {
        let arguments = json!({"task_name":format!("child_{index}"),"message":"fixture","agent_type":role,"fork_turns":"none"}).to_string();
        events.push(match namespace {
            Some(namespace) => responses::ev_function_call_with_namespace(
                &format!("spawn-{index}"),
                namespace,
                tool,
                &arguments,
            ),
            None => responses::ev_function_call(&format!("spawn-{index}"), tool, &arguments),
        });
    }
    events.push(event("response.completed", usage(Some(0))));
    root.send(events).await?;
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
        .position(|request| request.body.to_string().contains("ws role fixture"))
        .expect("role reload");
    drop(held.remove(role_index));
    let retried = gate.next().await?;
    assert!(retried.body.to_string().contains("ws role fixture"));
    let retries = attempts(&db).await?;
    assert_eq!(retries.len(), 5);
    let retry = retries.last().unwrap();
    let predecessor = records
        .iter()
        .find(|attempt| Some(attempt.attempt_id) == retry.retry_of)
        .unwrap();
    assert_ne!(retry.thread_id, test.session_configured.thread_id);
    assert_eq!(
        (retry.thread_id, retry.request_id),
        (predecessor.thread_id, predecessor.request_id)
    );
    held.push(retried);
    for held in held {
        held.complete().await?;
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
    test.thread_manager
        .shutdown_all_threads_bounded(Duration::from_secs(3))
        .await;
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_native_delete_rejects_late_usage() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let held = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    assert_eq!(attempts(&db).await?.len(), 1);
    let unrelated = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_home(test.home.clone())
        .build_with_auto_env(&server)
        .await?;
    submit(&unrelated).await?;
    gate.next().await?.complete().await?;
    terminal(&unrelated).await?;
    let unrelated_records: Vec<_> = attempts(&db)
        .await?
        .into_iter()
        .filter(|a| a.thread_id == unrelated.session_configured.thread_id)
        .collect();
    assert_eq!(unrelated_records.len(), 1);
    let unrelated_usage = observations(&db).await?;
    db.delete_thread(test.session_configured.thread_id).await?;
    held.complete().await?;
    terminal(&test).await?;
    assert_eq!(attempts(&db).await?, unrelated_records);
    assert_eq!(observations(&db).await?, unrelated_usage);
    gate.no_pending().await;
    let home = unrelated.home.clone();
    let rollout = unrelated.codex.rollout_path().unwrap();
    let owner = unrelated.session_configured.thread_id;
    stop(&unrelated).await;
    drop(unrelated);
    let off = builder(
        gate.endpoint.clone(),
        codex_core::config::AccountingMode::Disabled,
    )
    .resume(&server, home, rollout)
    .await?;
    off.codex.state_db().unwrap().delete_thread(owner).await?;
    assert!(attempts(&db).await?.is_empty());
    assert!(observations(&db).await?.is_empty());
    stop(&off).await;
    stop(&test).await;
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_native_unknown_prices_and_no_usage() -> anyhow::Result<()> {
    for usage_present in [false, true] {
        let server = MockServer::start().await;
        let mut gate = Gate::start().await?;
        let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
            .with_config(|config| config.model = Some("gpt-6-astra".into()))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let held = gate.next().await?;
        if usage_present {
            held.complete().await?;
        } else {
            held.send(vec![
                responses::ev_response_created("same"),
                json!({"type":"response.completed","response":{"id":"same"}}),
            ])
            .await?;
        }
        terminal(&test).await?;
        let db = test.codex.state_db().unwrap();
        let records = attempts(&db).await?;
        assert_eq!(records.len(), 1);
        assert!(
            payloads::<Snapshot>(&db, "draft_accounting_price_snapshots")
                .await?
                .is_empty()
        );
        assert_eq!(observations(&db).await?.len(), usize::from(usage_present));
        let total = totals(&db, &records[0]).await?;
        assert_eq!(total.unknown_estimates, 1);
        assert_eq!(
            total.measured[6],
            Metric {
                known: if usage_present { 140 } else { 0 },
                unknown: i64::from(!usage_present)
            }
        );
        stop(&test).await;
    }
    Ok(())
}
#[tokio::test]
async fn accounting_responses_ws_native_auxiliary_scope_and_event_parity() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|config| {
            config
                .features
                .disable(codex_features::Feature::TokenBudget)
                .unwrap();
            config
                .features
                .disable(codex_features::Feature::RemoteCompactionV2)
                .unwrap();
        })
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    gate.next().await?.complete().await?;
    let events = terminal(&test).await?;
    assert!(!events.iter().any(|e| matches!(e, EventMsg::Error(_))));
    let db = test.codex.state_db().unwrap();
    let before = attempts(&db).await?;
    test.codex.submit(Op::Compact).await?;
    let compact_events = terminal(&test).await?;
    assert!(
        !compact_events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_)))
    );
    assert_eq!(attempts(&db).await?, before);
    assert_eq!(observations(&db).await?.len(), 1);
    counts(&gate, (1, 1, 1, 1));
    stop(&test).await;
    Ok(())
}
