#[path = "accounting_anthropic_support.rs"]
mod support;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_state::accounting::AccountingStore;
use pretty_assertions::assert_eq;
use serde_json::json;
use support::*;
use wiremock::MockServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_redirects_never_send_or_attribute_to_unapproved_endpoint()
-> anyhow::Result<()> {
    use wiremock::Mock;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::{method, path};
    for status in [307, 308] {
        for on in [true, false] {
            let approved = MockServer::start().await;
            let target = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/messages"))
                .respond_with(
                    ResponseTemplate::new(status)
                        .insert_header("location", format!("{}/v1/messages", target.uri())),
                )
                .expect(1)
                .mount(&approved)
                .await;
            Mock::given(method("POST"))
                .and(path("/v1/messages"))
                .respond_with(success(
                    json!({"input_tokens":71}),
                    json!({"output_tokens":19}),
                ))
                .expect(if on { 0 } else { 1 })
                .mount(&target)
                .await;
            let endpoint = format!("{}/v1", approved.uri());
            let mode = if on {
                enabled(&endpoint)
            } else {
                codex_core::config::AccountingMode::Disabled
            };
            let test = builder(endpoint, mode)
                .build_with_auto_env(&approved)
                .await?;
            submit(&test).await?;
            let events = terminal(&test).await?;
            assert_eq!(
                events
                    .iter()
                    .any(|event| matches!(event, EventMsg::Error(_))),
                on,
                "{events:?}"
            );
            let sent = approved.received_requests().await.unwrap();
            assert_eq!(sent.len(), 1);
            assert_eq!(
                sent[0].body_json::<serde_json::Value>()?["model"],
                "claude-opus-5"
            );
            assert!(sent[0].headers.contains_key("user-agent"));
            assert_eq!(
                target.received_requests().await.unwrap().len(),
                usize::from(!on)
            );
            let db = test.codex.state_db().unwrap();
            if on {
                let records = attempts(&db).await?;
                assert_eq!(records.len(), 1);
                assert_eq!(records[0].thread_id, test.session_configured.thread_id);
                assert!(records[0].retry_of.is_none());
                assert_eq!(observations(&db).await?, vec![]);
                let estimates: Vec<String> =
                    sqlx::query_scalar("SELECT payload FROM draft_accounting_estimates")
                        .fetch_all(&mut connection(&db).await?)
                        .await?;
                assert_eq!(estimates.len(), 1);
                let quote: serde_json::Value = serde_json::from_str(&estimates[0])?;
                assert_eq!(
                    quote["usage"],
                    serde_json::to_value(codex_state::accounting::Usage::default())?
                );
                assert_eq!(
                    quote["known_subtotal"],
                    serde_json::to_value(codex_state::accounting::Decimal::default())?
                );
                assert_eq!(quote["all_buckets_priced"], serde_json::Value::Null);
            } else {
                let installed: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM sqlite_schema WHERE name = '_accounting_migrations'",
                )
                .fetch_one(&mut connection(&db).await?)
                .await?;
                assert_eq!(installed, 0);
            }
            stop(&test).await;
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_opt_in_does_not_activate_other_native_routes() -> anyhow::Result<()> {
    use core_test_support::responses;
    let server = MockServer::start().await;
    let mock = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_response_created("unsupported-responses"),
            responses::ev_assistant_message("answer", "ordinary response"),
            responses::ev_completed("unsupported-responses"),
        ]),
    )
    .await;
    let test = core_test_support::test_codex::test_codex()
        .with_config(|config| {
            config.accounting = enabled("https://api.anthropic.com/v1");
            config
                .features
                .enable(codex_features::Feature::Sqlite)
                .unwrap();
        })
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("unchanged Responses route").await?;
    assert_eq!(mock.single_request().path(), "/v1/responses");
    let db = test.codex.state_db().unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE name = '_accounting_migrations'",
    )
    .fetch_one(&mut connection(&db).await?)
    .await?;
    assert_eq!(count, 0);
    stop(&test).await;

    let server = MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/messages"))
        .respond_with(success(
            json!({"input_tokens":0}),
            json!({"output_tokens":0}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let endpoint = format!("{}/v1", server.uri());
    let test = builder(endpoint.clone(), enabled(&endpoint))
        .with_config(|config| {
            config.model_provider_id = "compatible-fixture".into();
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
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE name = '_accounting_migrations'",
    )
    .fetch_one(&mut connection(&db).await?)
    .await?;
    assert_eq!(count, 0);
    stop(&test).await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_stream_retry_retains_start_usage_without_merging_reused_provider_id()
-> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = GateServer::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let first = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    first
        .chunks
        .send(sse(&[start(json!({"input_tokens":7,"output_tokens":0}))]))
        .await?;
    wait_observations(&db, 1).await?;
    drop(first); // Real EOF after committed numeric evidence, before completion.
    let second = gate.next().await?;
    second
        .chunks
        .send(sse(&[start(
            json!({"input_tokens":11,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}),
        )]))
        .await?;
    second
        .chunks
        .send(sse(&ending(json!({"output_tokens":2}))))
        .await?;
    drop(second);
    let events = terminal(&test).await?;
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_))),
        "{events:?}"
    );
    let rows = attempts(&db).await?;
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (rows[1].request_id, rows[1].retry_of),
        (rows[0].request_id, Some(rows[0].attempt_id))
    );
    let evidence = observations(&db).await?;
    assert_eq!(evidence.len(), 3);
    assert_ne!(evidence[0].source, evidence[1].source);
    assert_eq!(
        evidence[0].patch.input,
        codex_state::accounting::Presence::Number(7.try_into()?)
    );
    stop(&test).await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_missing_optional_state_is_only_fatal_when_enabled()
-> anyhow::Result<()> {
    for on in [false, true] {
        let server = MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/v1/messages"))
            .respond_with(success(
                json!({"input_tokens":0}),
                json!({"output_tokens":0}),
            ))
            .expect(if on { 0 } else { 1 })
            .mount(&server)
            .await;
        let endpoint = format!("{}/v1", server.uri());
        let mode = if on {
            enabled(&endpoint)
        } else {
            codex_core::config::AccountingMode::Disabled
        };
        let test = builder(endpoint, mode)
            .with_config(|config| {
                let path = config.codex_home.join("unusable-sqlite-home");
                std::fs::write(&path, b"synthetic non-directory").unwrap();
                config.sqlite = codex_state::SqliteConfig::from_sqlite_home(
                    codex_utils_absolute_path::AbsolutePathBuf::try_from(path).unwrap(),
                );
            })
            .build_with_auto_env(&server)
            .await?;
        assert!(test.codex.state_db().is_none());
        submit(&test).await?;
        let events = terminal(&test).await?;
        assert_eq!(events.iter().any(|event| matches!(event, EventMsg::Error(error) if error.message.contains("accounting"))), on);
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_401_and_429_are_terminal_and_preflight_schema_fault_has_no_send()
-> anyhow::Result<()> {
    // The existing direct Anthropic policy does not retry HTTP 429 (only Z.ai
    // enables that policy). Accounting must not introduce a repair send.
    for (status, schema_fault) in [(401, false), (429, false), (401, true)] {
        let server = MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .and(wiremock::matchers::path("/v1/messages"))
            .respond_with(
                wiremock::ResponseTemplate::new(status)
                    .set_body_json(json!({"error":{"message":"synthetic denied"}})),
            )
            .expect(if schema_fault { 0 } else { 1 })
            .mount(&server)
            .await;
        let endpoint = format!("{}/v1", server.uri());
        let test = builder(endpoint.clone(), enabled(&endpoint))
            .build_with_auto_env(&server)
            .await?;
        let db = test.codex.state_db().unwrap();
        if schema_fault {
            sqlx::query("CREATE TABLE draft_accounting_invalid (id INTEGER)")
                .execute(&mut connection(&db).await?)
                .await?;
        }
        submit(&test).await?;
        let events = terminal(&test).await?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EventMsg::Error(_))),
            "{events:?}"
        );
        if !schema_fault {
            assert_eq!(attempts(&db).await?.len(), 1);
            assert_eq!(observations(&db).await?, vec![]);
        }
        stop(&test).await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_observation_failure_stops_real_sampling_without_repair_send()
-> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = GateServer::start().await?;
    let test = builder(gate.endpoint.clone(), enabled(&gate.endpoint))
        .build_with_auto_env(&server)
        .await?;
    submit(&test).await?;
    let request = gate.next().await?;
    assert_eq!(request.body["model"], "claude-opus-5");
    assert!(request.head.starts_with("POST /v1/messages "));
    let db = test.codex.state_db().unwrap();
    request
        .chunks
        .send(sse(&[start(json!({"input_tokens":7,"output_tokens":0}))]))
        .await?;
    wait_observations(&db, 1).await?;
    let before = observations(&db).await?;
    sqlx::query("CREATE TRIGGER fail_observation BEFORE INSERT ON draft_accounting_observations BEGIN SELECT RAISE(ABORT, 'caller-observation-fault'); END")
        .execute(&mut connection(&db).await?).await?;
    request
        .chunks
        .send(sse(&ending(json!({"output_tokens":3}))))
        .await?;
    drop(request);
    let events = terminal(&test).await?;
    assert!(events.iter().any(
        |event| matches!(event, EventMsg::Error(error) if error.message.contains("accounting"))
    ));
    assert_eq!(observations(&db).await?, before);
    assert_eq!(attempts(&db).await?.len(), 1);
    assert!(gate.incoming.try_recv().is_err());
    stop(&test).await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_cancel_keeps_intent_or_observed_start_through_reopen()
-> anyhow::Result<()> {
    for with_usage in [false, true] {
        let server = MockServer::start().await;
        let mut gate = GateServer::start().await?;
        let mode = enabled(&gate.endpoint);
        let test = builder(gate.endpoint.clone(), mode.clone())
            .build_with_auto_env(&server)
            .await?;
        submit(&test).await?;
        let request = gate.next().await?;
        let db = test.codex.state_db().unwrap();
        if with_usage {
            request
                .chunks
                .send(sse(&[start(json!({"input_tokens":7}))]))
                .await?;
            wait_observations(&db, 1).await?;
        }
        test.codex.submit(Op::Interrupt).await?;
        let events = terminal(&test).await?;
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EventMsg::TurnAborted(_)))
        );
        drop(request);
        let records = attempts(&db).await?;
        let rows = observations(&db).await?;
        assert_eq!((records.len(), rows.len()), (1, usize::from(with_usage)));
        let rollout = test.codex.rollout_path().unwrap();
        let home = test.home.clone();
        stop(&test).await;
        drop(test);
        drop(db);
        for reopen in 0..2 {
            let resumed = builder(gate.endpoint.clone(), mode.clone())
                .resume(&server, home.clone(), rollout.clone())
                .await?;
            let db = resumed.codex.state_db().unwrap();
            assert_eq!(attempts(&db).await?, records);
            assert_eq!(observations(&db).await?, rows);
            assert!(gate.incoming.try_recv().is_err());
            if reopen == 1 {
                submit(&resumed).await?;
                let fresh = gate.next().await?;
                fresh
                    .chunks
                    .send(sse(&[start(json!({"input_tokens":3}))]))
                    .await?;
                fresh
                    .chunks
                    .send(sse(&ending(json!({"output_tokens":1}))))
                    .await?;
                drop(fresh);
                let events = terminal(&resumed).await?;
                assert!(
                    !events
                        .iter()
                        .any(|event| matches!(event, EventMsg::Error(_))),
                    "{events:?}"
                );
                let all = attempts(&db).await?;
                assert_eq!(all.len(), 2);
                assert_eq!(all[0], records[0]);
                assert_ne!(all[1].request_id, records[0].request_id);
                assert_eq!(all[1].retry_of, None);
                let evidence = observations(&db).await?;
                assert_eq!(&evidence[..rows.len()], rows.as_slice());
                assert_eq!(evidence.len(), rows.len() + 2);
            }
            stop(&resumed).await;
        }
        assert!(gate.incoming.try_recv().is_err());
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_anthropic_deleted_native_owner_cannot_be_resurrected_by_delayed_usage()
-> anyhow::Result<()> {
    let server = MockServer::start().await;
    let mut gate = GateServer::start().await?;
    let mode = enabled(&gate.endpoint);
    let test = builder(gate.endpoint.clone(), mode.clone())
        .build_with_auto_env(&server)
        .await?;
    let unrelated = builder(gate.endpoint.clone(), mode)
        .with_home(test.home.clone())
        .build_with_auto_env(&server)
        .await?;
    submit(&unrelated).await?;
    let completed = gate.next().await?;
    completed
        .chunks
        .send(sse(&[start(json!({"input_tokens":4}))]))
        .await?;
    completed
        .chunks
        .send(sse(&ending(json!({"output_tokens":2}))))
        .await?;
    drop(completed);
    let events = terminal(&unrelated).await?;
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, EventMsg::Error(_))),
        "{events:?}"
    );
    let unrelated_db = unrelated.codex.state_db().unwrap();
    let saved_attempts = attempts(&unrelated_db).await?;
    let saved_observations = observations(&unrelated_db).await?;
    assert_eq!((saved_attempts.len(), saved_observations.len()), (1, 2));
    assert_eq!(
        saved_attempts[0].thread_id,
        unrelated.session_configured.thread_id
    );
    submit(&test).await?;
    let request = gate.next().await?;
    let db = test.codex.state_db().unwrap();
    db.delete_thread(test.session_configured.thread_id).await?;
    request
        .chunks
        .send(sse(&[start(json!({"input_tokens":9}))]))
        .await?;
    request
        .chunks
        .send(sse(&ending(json!({"output_tokens":2}))))
        .await?;
    drop(request);
    let events = terminal(&test).await?;
    assert!(events.iter().any(
        |event| matches!(event, EventMsg::Error(error) if error.message.contains("accounting"))
    ));
    assert_eq!(attempts(&db).await?, saved_attempts);
    assert_eq!(observations(&db).await?, saved_observations);
    assert!(gate.incoming.try_recv().is_err());
    AccountingStore::open(&db, chrono::Utc::now().timestamp_millis()).await?;
    stop(&test).await;
    stop(&unrelated).await;
    db.delete_thread(unrelated.session_configured.thread_id)
        .await?;
    assert_eq!(attempts(&db).await?, vec![]);
    assert_eq!(observations(&db).await?, vec![]);
    Ok(())
}
