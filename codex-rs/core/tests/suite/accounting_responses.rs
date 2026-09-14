#[path = "accounting_responses_support.rs"]
pub(super) mod support;
use codex_core::config::AccountingMode;
use codex_protocol::protocol::EventMsg;
use codex_state::accounting::*;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use support::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn accounting_responses_native_off_and_unsupported() -> anyhow::Result<()> {
    for variant in 0..4 {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let mock = responses::mount_sse_once(&server, success(usage(Some(0)))).await;
        let mode = if variant < 2 {
            AccountingMode::Disabled
        } else {
            enabled(&endpoint)
        };
        let test = builder(endpoint, mode)
            .with_config(move |config| {
                if variant == 1 {
                    config
                        .features
                        .disable(codex_features::Feature::Sqlite)
                        .unwrap();
                }
                if variant == 2 {
                    config.model_provider_id = "compatible-fixture".into();
                }
                if variant == 3 {
                    config.model_provider.experimental_bearer_token =
                        Some("synthetic-override".into());
                }
            })
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn("fixture").await?;
        assert_eq!(mock.requests().len(), 1);
        if let Some(db) = test.codex.state_db() {
            absent(&db).await?;
        }
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_complete_and_partial_goldens() -> anyhow::Result<()> {
    for write in [Some(0), None] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let mock = responses::mount_sse_once(&server, success(usage(write))).await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn("fixture").await?;
        let db = test.codex.state_db().unwrap();
        let records = attempts(&db).await?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].thread_id, test.session_configured.thread_id);
        assert_eq!(
            (&records[0].provider, &records[0].model, records[0].dialect),
            (
                &"openai".to_string(),
                &"gpt-5.6-sol".to_string(),
                Dialect::Inclusive
            )
        );
        let request = mock.single_request();
        assert_eq!(request.path(), "/v1/responses");
        assert_eq!(request.body_json()["model"], "gpt-5.6-sol");
        let expected = [
            100,
            if write.is_some() { 80 } else { 0 },
            20,
            0,
            40,
            10,
            140,
        ];
        let actual = totals(&db, &records[0]).await?;
        assert_eq!(
            actual,
            DayTotals {
                measured: std::array::from_fn(|i| Metric {
                    known: expected[i],
                    unknown: i64::from(write.is_none() && (i == 1 || i == 3))
                }),
                known_usd: if write.is_some() {
                    "0.00161"
                } else {
                    "0.00121"
                }
                .to_string()
                .try_into()?,
                unknown_estimates: i64::from(write.is_none()),
                attempts: 1,
            }
        );
        let evidence = observations(&db).await?;
        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].patch.write,
            write.map_or(Presence::Missing, |_| Presence::Number(
                0.try_into().unwrap()
            ))
        );
        let prices: Vec<Snapshot> = payloads(&db, "draft_accounting_price_snapshots").await?;
        assert_eq!(prices.len(), 1);
        assert_eq!(prices[0].provider, "openai");
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_separate_and_terminal_usage() -> anyhow::Result<()> {
    for kind in [
        "response.completed",
        "response.failed",
        "response.incomplete",
    ] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let payload = responses::sse(vec![
            event("response.usage", json!({"input_tokens":7})),
            event(kind, json!({"output_tokens":3})),
        ]);
        let mock = responses::mount_sse_once(&server, payload).await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(|config| config.model_provider.stream_max_retries = Some(0))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let output = terminal(&test).await?;
        let db = test.codex.state_db().unwrap();
        assert_eq!(mock.requests().len(), 1);
        let patches = observations(&db).await?;
        assert_eq!(patches.len(), 2);
        assert_eq!(patches[0].source, patches[1].source);
        assert_eq!(patches[0].patch.input, Presence::Number(7.try_into()?));
        assert_eq!(patches[1].patch.output, Presence::Number(3.try_into()?));
        assert!(output.iter().any(|e| matches!(e, EventMsg::Error(_))));
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_unknown_prices_and_tiers() -> anyhow::Result<()> {
    for (model, tier) in [
        ("gpt-6-astra", None),
        ("remote-only-fixture", None),
        ("gpt-5.6-sol", Some("priority")),
    ] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let mock = responses::mount_sse_once(&server, success(usage(Some(0)))).await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(move |config| config.model = Some(model.into()))
            .build_with_auto_env(&server)
            .await?;
        test.submit_turn_with_service_tier("fixture", tier).await?;
        let db = test.codex.state_db().unwrap();
        let records = attempts(&db).await?;
        assert_eq!(mock.single_request().body_json()["model"], model);
        assert_eq!(records.len(), 1);
        assert!(
            payloads::<Snapshot>(&db, "draft_accounting_price_snapshots")
                .await?
                .is_empty()
        );
        let before = totals(&db, &records[0]).await?;
        assert_eq!(
            before.measured[6],
            Metric {
                known: 140,
                unknown: 0
            }
        );
        assert_eq!(before.unknown_estimates, 1);
        let home = test.home.clone();
        let rollout = test.codex.rollout_path().unwrap();
        stop(&test).await;
        drop(test);
        let reopened = builder(endpoint.clone(), enabled(&endpoint))
            .resume(&server, home, rollout)
            .await?;
        assert_eq!(
            totals(&reopened.codex.state_db().unwrap(), &records[0]).await?,
            before
        );
        assert_eq!(mock.requests().len(), 1);
        stop(&reopened).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_endpoint_mismatch() -> anyhow::Result<()> {
    for suffix in ["/wrong", "?unexpected=1", ":1234", "/v1"] {
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
        absent(&test.codex.state_db().unwrap()).await?;
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_no_usage_is_unknown() -> anyhow::Result<()> {
    for payload in [
        success(Value::Null),
        responses::sse(vec![responses::ev_response_created("reused-provider-id")]),
    ] {
        let server = MockServer::start().await;
        let endpoint = format!("{}/v1", server.uri());
        let mock = responses::mount_sse_once(&server, payload).await;
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .with_config(|config| config.model_provider.stream_max_retries = Some(0))
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        terminal(&test).await?;
        let db = test.codex.state_db().unwrap();
        assert_eq!(mock.requests().len(), 1);
        let records = attempts(&db).await?;
        assert_eq!(records.len(), 1);
        assert!(observations(&db).await?.is_empty());
        assert_eq!(
            totals(&db, &records[0]).await?,
            DayTotals {
                measured: std::array::from_fn(|_| Metric {
                    known: 0,
                    unknown: 1
                }),
                known_usd: "0".to_string().try_into()?,
                unknown_estimates: 1,
                attempts: 1,
            }
        );
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_ws_fallback_http_segment() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    Mock::given(method("GET"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(426))
        .mount(&server)
        .await;
    let mock = responses::mount_sse_once(&server, success(usage(Some(0)))).await;
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(|config| config.model_provider.supports_websockets = true)
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("fixture").await?;
    assert_eq!(mock.requests().len(), 1);
    assert_eq!(
        server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.method.as_str() == "GET")
            .count(),
        1
    );
    let db = test.codex.state_db().unwrap();
    assert_eq!(attempts(&db).await?.len(), 1);
    assert_eq!(observations(&db).await?.len(), 1);
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_sampling_and_auxiliary_scope() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    let mock = responses::mount_sse_sequence(
        &server,
        vec![
            responses::sse(vec![
                responses::ev_response_created("tools"),
                responses::ev_shell_command_call("shell-fixture", "echo accounting"),
                event("response.completed", usage(Some(0))),
            ]),
            success(usage(Some(0))),
        ],
    )
    .await;
    let compact = responses::mount_compact_json_once(
        &server,
        json!({"output":[{
            "type":"compaction","encrypted_content":"synthetic-summary"
        }]}),
    )
    .await;
    let test = builder(endpoint.clone(), enabled(&endpoint))
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
    test.submit_turn("fixture").await?;
    let db = test.codex.state_db().unwrap();
    let records = attempts(&db).await?;
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].turn, records[1].turn);
    assert_ne!(records[0].request_id, records[1].request_id);
    assert_eq!(records[1].retry_of, None);
    assert_eq!(mock.requests().len(), 2);
    test.codex
        .submit(codex_protocol::protocol::Op::Compact)
        .await?;
    terminal(&test).await?;
    assert_eq!(compact.single_request().path(), "/v1/responses/compact");
    assert_eq!(attempts(&db).await?, records);
    assert_eq!(observations(&db).await?.len(), 2);
    stop(&test).await;
    Ok(())
}

#[tokio::test]
async fn accounting_responses_native_ws_only_no_install() -> anyhow::Result<()> {
    let server = responses::start_websocket_server(vec![vec![
        vec![
            responses::ev_response_created("warm"),
            responses::ev_completed("warm"),
        ],
        vec![
            responses::ev_response_created("fixture"),
            responses::ev_assistant_message("m", "fixture"),
            event("response.completed", usage(Some(0))),
        ],
    ]])
    .await;
    let mut builder = builder(
        "http://127.0.0.1:1/v1".into(),
        enabled("https://api.openai.com/v1"),
    )
    .with_config(|config| config.model_provider.supports_websockets = true);
    let test = builder.build_with_websocket_server(&server).await?;
    test.submit_turn("fixture").await?;
    assert_eq!(server.handshakes().len(), 1);
    assert_eq!(server.single_connection().len(), 2);
    absent(&test.codex.state_db().unwrap()).await?;
    stop(&test).await;
    server.shutdown().await;
    Ok(())
}
