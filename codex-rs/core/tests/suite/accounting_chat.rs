#[path = "accounting_chat_support.rs"]
pub(super) mod support;
use codex_core::config::AccountingMode;
use codex_protocol::protocol::{EventMsg, Op};
use codex_state::accounting::*;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::*;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn accounting_chat_custom_provider_collects_with_real_identity() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    mount(&server, success(usage())).await;
    let mode = AccountingMode::Provider {
        scope: uuid::Uuid::new_v4(),
        provider_id: "custom-plan".into(),
        wire_api: codex_model_provider_info::WireApi::Chat,
        approved_endpoint: endpoint.clone(),
        api_key_pricing: false,
    };
    let test = builder(endpoint, mode)
        .with_config(|config| {
            config.model_provider_id = "custom-plan".into();
            config.model_provider.name = "Custom Plan".into();
        })
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("custom provider accounting").await?;
    let db = test.codex.state_db().unwrap();
    let records = attempts(&db).await?;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].provider, "custom-plan");
    let prices: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
    assert!(prices.is_empty());
    assert_eq!(observations(&db).await?.len(), 1);
    posts(&server, 1).await;
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_off_and_mode_isolation() -> anyhow::Result<()> {
    for index in 0..5 {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        mount(&server, success(usage())).await;
        let mode = match index {
            0 | 1 => AccountingMode::Disabled,
            2 => AccountingMode::DirectAnthropic {
                scope: uuid::Uuid::new_v4(),
                approved_endpoint: endpoint.clone(),
            },
            3 => AccountingMode::DirectOpenAiResponsesHttp {
                scope: uuid::Uuid::new_v4(),
                approved_endpoint: endpoint.clone(),
            },
            _ => AccountingMode::DirectOpenAiResponses {
                scope: uuid::Uuid::new_v4(),
                approved_endpoint: endpoint.clone(),
            },
        };
        let test = builder(endpoint, mode)
            .with_config(move |c| {
                if index == 1 {
                    c.features.disable(codex_features::Feature::Sqlite).unwrap();
                }
            })
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn("fixture").await?;
        posts(&server, 1).await;
        if let Some(db) = test.codex.state_db() {
            absent(&db).await?;
        }
        stop(&test).await;
    }
    for wire in [
        codex_model_provider_info::WireApi::Responses,
        codex_model_provider_info::WireApi::Anthropic,
    ] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(move |c| {
                c.model_provider.wire_api = wire;
                for model in &mut c.model_catalog.as_mut().unwrap().models {
                    model.max_output_tokens = Some(1024);
                }
            })
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        terminal(&test).await?;
        assert_eq!(
            server.received_requests().await.unwrap().len(),
            1,
            "{wire:?}"
        );
        absent(&test.codex.state_db().unwrap()).await?;
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_literal_partial_and_zero_goldens() -> anyhow::Result<()> {
    for (cached, zero) in [(true, false), (false, false), (true, true)] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let mut raw = usage();
        if !cached {
            raw.as_object_mut().unwrap().remove("prompt_tokens_details");
        }
        if zero {
            raw = json!({"prompt_tokens":0,"completion_tokens":0,"total_tokens":0,
            "prompt_tokens_details":{"cached_tokens":0},"completion_tokens_details":{"reasoning_tokens":0}});
        }
        mount(&server, success(raw)).await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn("fixture").await?;
        posts(&server, 1).await;
        let db = test.codex.state_db().unwrap();
        let rows = attempts(&db).await?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].thread_id, test.session_configured.thread_id);
        assert_eq!(rows[0].dialect, Dialect::Inclusive);
        assert_eq!(totals(&db, &rows[0]).await?, golden(cached, zero, 1)?);
        let patches = observations(&db).await?;
        assert_eq!(patches.len(), 1);
        assert_eq!(patches[0].patch.write, Presence::Missing);
        let prices: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
        assert_eq!(prices.len(), 1);
        eprintln!("CHAT_PRICES {}", serde_json::to_string(&prices)?);
        assert_eq!(prices[0].rates.write, None);
        assert_eq!(prices[0].effective_from_ms, rows[0].dispatched_at_ms);
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_cumulative_and_separate_usage() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    mount(
        &server,
        event(usage())
            + ": comment\n\n"
            + &data(json!({"choices":[]}))
            + &event(usage())
            + &data(json!({"choices":[]}))
            + &event(json!({"prompt_tokens":null}))
            + &ending("stop"),
    )
    .await;
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("fixture").await?;
    posts(&server, 1).await;
    let db = test.codex.state_db().unwrap();
    let rows = attempts(&db).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(totals(&db, &rows[0]).await?, golden(true, false, 1)?);
    let patches = observations(&db).await?;
    assert_eq!(
        patches
            .iter()
            .map(|p| i64::from(p.sequence))
            .collect::<Vec<_>>(),
        vec![1, 3, 5]
    );
    assert!(patches.iter().all(|p| p.source == patches[0].source));
    stop(&test).await;
    drop(test);
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_top_level_error_retains_usage() -> anyhow::Result<()> {
    for sibling in [true, false] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let payload = if sibling {
            data(json!({"error":{"message":"fixture terminal"},"usage":usage()}))
        } else {
            event(usage()) + &data(json!({"error":{"message":"fixture terminal"}}))
        };
        mount(&server, payload).await;
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
        posts(&server, 1).await;
        let db = test.codex.state_db().unwrap();
        let rows = attempts(&db).await?;
        assert_eq!(rows.len(), 1);
        assert_eq!(observations(&db).await?.len(), 1);
        assert_eq!(totals(&db, &rows[0]).await?, golden(true, false, 1)?);
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_mismatched_endpoint_never_sends() -> anyhow::Result<()> {
    for suffix in ["/wrong", "?q=1", ":1", "/v1", "#fragment"] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let test = builder(endpoint.clone(), enabled(&format!("{endpoint}{suffix}")))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        assert!(
            terminal(&test)
                .await?
                .iter()
                .any(|e| matches!(e, EventMsg::Error(_)))
        );
        assert!(server.received_requests().await.unwrap().is_empty());
        let db = test.codex.state_db().unwrap();
        absent(&db).await?;
        stop(&test).await;
        drop(test);
        db.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_immutable_prices_and_unpriced_rows() -> anyhow::Result<()> {
    for model in ["gpt-5.6-sol", "gpt-6-astra", "remote-only-fixture"] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        mount(&server, success(usage())).await;
        let mode = enabled(&endpoint);
        let test = builder(endpoint.clone(), mode.clone())
            .with_config(move |c| c.model = Some(model.into()))
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn_with_service_tier("fixture", Some("priority"))
            .await?;
        let request = server.received_requests().await.unwrap().remove(0);
        let body: Value = serde_json::from_slice(&request.body)?;
        assert_eq!(body["model"], model);
        assert!(body.get("service_tier").is_none());
        let db = test.codex.state_db().unwrap();
        let rows = attempts(&db).await?;
        let prices: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
        assert_eq!(prices.len(), usize::from(model == "gpt-5.6-sol"));
        let before = totals(&db, &rows[0]).await?;
        assert_eq!(before.unknown_estimates, 1);
        assert_eq!(
            before.known_usd,
            if model == "gpt-5.6-sol" {
                "0.00121"
            } else {
                "0"
            }
            .to_string()
            .try_into()?
        );
        let home = test.home.clone();
        let rollout = test.codex.rollout_path().unwrap();
        stop(&test).await;
        drop(test);
        db.close().await;
        for index in 0..2 {
            let reopened = builder(endpoint.clone(), mode.clone())
                .with_config(|c| c.model_catalog.as_mut().unwrap().models.clear())
                .resume(&server, home.clone(), rollout.clone())
                .await?;
            let db = reopened.codex.state_db().unwrap();
            assert_eq!(attempts(&db).await?, rows);
            assert_eq!(
                payloads::<Snapshot>(&db, "draft_accounting_price_snapshots").await?,
                prices
            );
            assert_eq!(totals(&db, &rows[0]).await?, before);
            assert_eq!(server.received_requests().await.unwrap().len(), 1);
            if index == 1 {
                reopened.submit_turn("fresh").await?;
                assert_eq!(attempts(&db).await?.len(), 2);
                assert_eq!(
                    payloads::<Snapshot>(&db, "draft_accounting_price_snapshots").await?
                        [..prices.len()],
                    prices
                );
            }
            stop(&reopened).await;
            drop(reopened);
            db.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_missing_usage_and_correlation() -> anyhow::Result<()> {
    for payload in [
        ending("stop"),
        success(Value::Null),
        data(json!({"id":"reused-provider-id","choices":[]})),
    ] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        mount(&server, payload).await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .build_with_auto_env(&server)
            .await?;
        for _ in 0..2 {
            submit(&test).await?;
            terminal(&test).await?;
        }
        posts(&server, 2).await;
        let db = test.codex.state_db().unwrap();
        let rows = attempts(&db).await?;
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].request_id, rows[1].request_id);
        assert!(rows.iter().all(|r| r.retry_of.is_none()));
        assert!(observations(&db).await?.is_empty());
        assert_eq!(
            totals(&db, &rows[0]).await?,
            DayTotals {
                measured: std::array::from_fn(|_| Metric {
                    known: 0,
                    unknown: 2
                }),
                known_usd: "0".to_string().try_into()?,
                unknown_estimates: 2,
                attempts: 2,
            }
        );
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_finish_reason_and_tool_parity() -> anyhow::Result<()> {
    for reason in ["stop", "length", "tool_calls", "error"] {
        let server = MockServer::start().await;
        let mut gate = Gate::start(GateRoutes::ChatOnly).await?;
        let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
            .with_config(|c| {
                c.features
                    .disable(codex_features::Feature::CodeModeOnly)
                    .unwrap();
            })
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let held = gate.next().await?;
        let payload = if reason == "tool_calls" {
            data(
                json!({"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"safe-call",
                "type":"function","function":{"name":"shell_command","arguments":"{\"command\":\"echo fixture\"}"}}]},
                "finish_reason":"tool_calls"}],"usage":usage()}),
            ) + "data: [DONE]\n\n"
        } else {
            event(usage()) + &ending(reason)
        };
        held.chunks.send(payload).await?;
        let expected = if reason == "tool_calls" {
            gate.next().await?.chunks.send(success(usage())).await?;
            2
        } else {
            1
        };
        let events = terminal(&test).await?;
        if reason == "length" {
            assert!(events.iter().any(|event| matches!(event, EventMsg::Error(error) if error.message.contains("without executing further model work"))));
        }
        if reason == "tool_calls" {
            assert!(
                events.iter().any(
                    |event| matches!(event, EventMsg::ExecCommandEnd(end) if end.exit_code == 0)
                )
            );
        }
        let db = test.codex.state_db().unwrap();
        let rows = attempts(&db).await?;
        assert_eq!(rows.len(), expected);
        assert_eq!(observations(&db).await?.len(), expected);
        if expected == 2 {
            assert_ne!(rows[0].request_id, rows[1].request_id);
        }
        gate.no_pending();
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_sampling_auxiliary_scope() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = Gate::start(GateRoutes::ChatAndCompact).await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .with_config(|c| {
            c.features
                .disable(codex_features::Feature::CodeModeOnly)
                .unwrap();
            c.features
                .disable(codex_features::Feature::TokenBudget)
                .unwrap();
            c.features
                .disable(codex_features::Feature::RemoteCompactionV2)
                .unwrap();
        })
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    gate.next().await?.chunks.send(data(json!({"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"safe-call",
        "type":"function","function":{"name":"shell_command","arguments":"{\"command\":\"echo fixture\"}"}}]},
        "finish_reason":"tool_calls"}],"usage":usage()})) + "data: [DONE]\n\n").await?;
    gate.next().await?.chunks.send(success(usage())).await?;
    let events = terminal(&test).await?;
    assert!(
        events
            .iter()
            .any(|event| matches!(event, EventMsg::ExecCommandEnd(end) if end.exit_code == 0))
    );
    let db = test.codex.state_db().unwrap();
    let rows = attempts(&db).await?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].turn, rows[1].turn);
    assert_ne!(rows[0].request_id, rows[1].request_id);
    test.codex.submit(Op::Compact).await?;
    let compact = gate.next().await?;
    assert!(compact.headers.starts_with("POST /v1/responses/compact "));
    terminal(&test).await?;
    assert_eq!(attempts(&db).await?, rows);
    assert_eq!(observations(&db).await?.len(), 2);
    gate.no_pending();
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_chat_native_gateway_exclusion_and_header_parity() -> anyhow::Result<()> {
    for retry in [false, true] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        if retry {
            Mock::given(method("POST"))
                .respond_with(|request: &wiremock::Request| {
                    ResponseTemplate::new(503)
                        .insert_header("x-corbanu-request-state", "released")
                        .insert_header(
                            "x-corbanu-request-id",
                            request
                                .headers
                                .get("x-pfterminal-request-id")
                                .unwrap()
                                .to_str()
                                .unwrap(),
                        )
                })
                .up_to_n_times(1)
                .with_priority(1)
                .mount(&server)
                .await;
        }
        mount(&server, success(usage())).await;
        let gateway_endpoint = endpoint.clone();
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(move |c| {
                let mut provider =
                    codex_model_provider_info::ModelProviderInfo::create_pfterminal_plan_provider();
                provider.base_url = Some(gateway_endpoint);
                provider.env_key = None;
                provider.experimental_bearer_token = Some("synthetic-gateway".into());
                provider.request_max_retries = Some(1);
                provider.stream_max_retries = Some(0);
                c.model_provider = provider;
            })
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn("fixture").await?;
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), if retry { 2 } else { 1 });
        let ids: Vec<_> = requests
            .iter()
            .map(|r| r.headers.get("x-pfterminal-request-id").unwrap())
            .collect();
        if retry {
            assert_ne!(ids[0], ids[1]);
        }
        absent(&test.codex.state_db().unwrap()).await?;
        stop(&test).await;
    }
    Ok(())
}
