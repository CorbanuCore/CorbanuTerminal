//! Corbanu `/cost` and accounting views (PF-60): app-level tests with a real accounting store.

use super::*;
use pretty_assertions::assert_eq;

async fn accounting_fixture(
    app: &mut App,
    path: &std::path::Path,
) -> anyhow::Result<(ThreadId, i64)> {
    accounting_fixture_at(app, path, chrono::Utc::now().timestamp_millis()).await
}

/// `accounting_fixture`, with the ledger written at `now` rather than the clock.
async fn accounting_fixture_at(
    app: &mut App,
    path: &std::path::Path,
    now: i64,
) -> anyhow::Result<(ThreadId, i64)> {
    use codex_state::accounting::*;
    let owner = ThreadId::new();
    let db = codex_state::StateRuntime::init(
        codex_state::SqliteConfig::new_for_testing(path.abs()),
        "synthetic".into(),
    )
    .await?;
    let metadata = codex_state::ThreadMetadataBuilder::new(
        owner,
        path.join("synthetic.jsonl"),
        chrono::Utc::now(),
        codex_protocol::protocol::SessionSource::Cli,
    );
    db.upsert_thread(&metadata.build("synthetic")).await?;
    let a: Attempt = serde_json::from_value(serde_json::json!({
        "attempt_id":uuid::Uuid::from_u128(1),"request_id":uuid::Uuid::from_u128(2),
        "thread_id":owner,"turn":"synthetic","retry_of":null,"provider":"fixture-only",
        "model":"fixture-model","scope":uuid::Uuid::nil(),"dialect":"Inclusive","dispatched_at_ms":now
    }))?;
    let s: Snapshot = serde_json::from_value(serde_json::json!({
        "id":uuid::Uuid::from_u128(3),"provider":"fixture-only","model":"fixture-model",
        "scope":uuid::Uuid::nil(),"currency":"USD","unit":"PerMillionTokens",
        "rates":{"noncached":"1","read":"1","write":"2","output":"4"},
        "source_reference":uuid::Uuid::from_u128(4),"source_kind":"ProviderPublished",
        "observed_at_ms":0,"approved_at_ms":0,"effective_from_ms":0,"effective_end_ms":null
    }))?;
    let o: Observation = serde_json::from_value(serde_json::json!({
        "revision":1,"source":uuid::Uuid::from_u128(5),"sequence":1,
        "patch":{"input":100,"read":20,"output":40}
    }))?;
    let store = AccountingStore::open(&db, now).await?;
    store.admit(owner, &a, &[s], now).await?;
    store.observe(owner, &a, &[o], now).await?;
    app.chat_widget
        .handle_thread_session_quiet(test_thread_session(owner, path.to_path_buf()));
    app.active_thread_id = Some(owner);
    app.state_db = Some(db);
    Ok((owner, now / 86_400_000))
}

async fn accounting_load(
    app: &mut App,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    day: i64,
) -> AppEvent {
    while rx.try_recv().is_ok() {}
    app.handle_accounting_inspector_event(AppEvent::OpenAccountingInspector { day });
    let load = rx.recv().await.unwrap();
    assert!(matches!(load, AppEvent::LoadAccountingInspector { .. }));
    app.handle_accounting_inspector_event(load);
    tokio::time::timeout(Duration::from_secs(20), rx.recv())
        .await
        .unwrap()
        .unwrap()
}

fn accounting_scroll(app: &mut App) -> String {
    app.chat_widget.on_terminal_resize(/*width*/ 40);
    let mut output = String::new();
    for _ in 0..120 {
        output.push_str(&render_bottom_popup(&app.chat_widget, /*width*/ 40));
        app.chat_widget
            .handle_key_event(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Down,
                crossterm::event::KeyModifiers::NONE,
            ));
    }
    output
}

#[tokio::test]
async fn accounting_inspect_app_real_store_to_view() -> anyhow::Result<()> {
    use codex_state::accounting::InspectionDay;
    let path = tempdir()?;
    let (mut app, mut rx, mut ops) = make_test_app_with_channels().await;
    let (_, day) = accounting_fixture(&mut app, path.path()).await?;
    while ops.try_recv().is_ok() {}
    let event = accounting_load(&mut app, &mut rx, day).await;
    let generation = match &event {
        AppEvent::AccountingInspectorLoaded {
            result: Ok(InspectionDay::Ready(v)),
            generation,
            ..
        } => {
            assert_eq!(
                serde_json::to_value(v.totals.known_usd)?,
                serde_json::json!("0.00018")
            );
            assert_eq!(v.totals.unknown_estimates, 1);
            assert_eq!(v.requests.len(), 1);
            *generation
        }
        other => panic!("{other:?}"),
    };
    app.handle_accounting_inspector_event(event);
    let summary = accounting_scroll(&mut app);
    assert!(
        summary.contains("$0.000180") && summary.contains("+ unknown") && summary.contains("costs")
    );
    for page in [1, 2] {
        app.handle_accounting_inspector_event(AppEvent::NavigateAccountingInspector {
            generation,
            page,
        });
    }
    let detail = accounting_scroll(&mut app);
    for required in [
        "Cache write: unknown",
        "fixture-only",
        "fixture-model",
        "0.00018",
        "Price ID:",
    ] {
        assert!(detail.contains(required), "{required}");
    }
    assert!(ops.try_recv().is_err());
    assert!(rx.try_recv().is_err());
    app.state_db.as_ref().unwrap().close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_range_app_dispatch_preserves_query() -> anyhow::Result<()> {
    use codex_state::accounting::InspectionDay;
    use codex_state::accounting::InspectionGrouping;
    use codex_state::accounting::InspectionRange;
    let path = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    let started = chrono::Utc::now().timestamp_millis();
    let (_, day) = accounting_fixture(&mut app, path.path()).await?;
    while rx.try_recv().is_ok() {}
    app.handle_accounting_inspector_event(AppEvent::OpenAccountingInspector { day });
    let mut load = rx.recv().await.unwrap();
    let requested = InspectionRange {
        start_ms: day * 86_400_000,
        end_ms: (day + 1) * 86_400_000,
        grouping: InspectionGrouping::Hour,
    };
    let AppEvent::LoadAccountingInspector { range, .. } = &mut load else {
        panic!()
    };
    *range = Some(requested);
    app.handle_accounting_inspector_event(load);
    let event = tokio::time::timeout(Duration::from_secs(20), rx.recv())
        .await?
        .unwrap();
    assert!(matches!(&event, AppEvent::AccountingInspectorLoaded {
        result: Ok(InspectionDay::Range { requested: actual, buckets, .. }), ..
    } if *actual == requested && buckets.len() == 24));
    app.handle_accounting_inspector_event(event);
    let text = accounting_scroll(&mut app);
    assert!(text.contains("timezone: UTC"));
    // Today's hours: the current one is in progress and later ones have not
    // started, so the total so far is shown (#289 R2). Unless the hour turned
    // while the test ran, which leaves the request's hour partial.
    let hour = |ms: i64| ms / 3_600_000;
    if hour(started) == hour(chrono::Utc::now().timestamp_millis()) {
        assert!(text.contains("In progress — totals so far"), "{text}");
        assert!(!text.contains("Range total unavailable"), "{text}");
    }
    app.state_db.as_ref().unwrap().close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_app_remote_does_not_read_local() -> anyhow::Result<()> {
    let path = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    let (_, day) = accounting_fixture(&mut app, path.path()).await?;
    app.app_server_target = crate::AppServerTarget::Remote {
        endpoint: crate::RemoteAppServerEndpoint::UnixSocket {
            socket_path: path.path().join("missing.sock").abs(),
        },
    };
    let event = accounting_load(&mut app, &mut rx, day).await;
    assert!(
        matches!(&event, AppEvent::AccountingInspectorLoaded { result: Err(message), .. } if message.contains("remote") && message.contains("Next step: "))
    );
    app.handle_accounting_inspector_event(event);
    let screen = accounting_scroll(&mut app);
    assert!(!screen.contains("0.000180"));
    assert!(!screen.contains("fixture-only"));
    app.state_db.as_ref().unwrap().close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_app_thread_switch_stale_reply() -> anyhow::Result<()> {
    let path = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    let (_, day) = accounting_fixture(&mut app, path.path()).await?;
    let held_result = accounting_load(&mut app, &mut rx, day).await;
    let other = ThreadId::new();
    app.chat_widget
        .handle_thread_session_quiet(test_thread_session(other, path.path().to_path_buf()));
    app.active_thread_id = Some(other);
    app.handle_accounting_inspector_event(held_result);
    assert!(
        !render_bottom_popup(&app.chat_widget, /*width*/ 80).contains("Cost — this conversation")
    );
    let next = accounting_load(&mut app, &mut rx, day).await;
    assert!(matches!(next, AppEvent::AccountingInspectorLoaded {
        thread: Some(id), result: Ok(codex_state::accounting::InspectionDay::MissingThread), ..
    } if id == other));
    app.state_db.as_ref().unwrap().close().await;
    Ok(())
}

/// #1518: a ledger written while the clock read ahead (here, three hours) is
/// intact once the clock is set right, yet newer than it. `/cost` names the
/// clock rather than calling the evidence corrupt.
#[tokio::test]
async fn accounting_inspect_app_clock_behind_ledger_names_the_clock() -> anyhow::Result<()> {
    let path = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    let ahead = chrono::Utc::now().timestamp_millis() + 3 * 3_600_000;
    accounting_fixture_at(&mut app, path.path(), ahead).await?;
    let day = chrono::Utc::now().timestamp_millis() / 86_400_000;
    let loaded = accounting_load(&mut app, &mut rx, day).await;
    let AppEvent::AccountingInspectorLoaded {
        result: Err(message),
        ..
    } = &loaded
    else {
        panic!("expected a refused read, got {loaded:?}");
    };
    assert_eq!(
        message,
        &crate::app::event_dispatch::clock_behind_ledger_message(3 * 3_600_000)
    );
    app.handle_accounting_inspector_event(loaded);
    let rendered = render_bottom_popup(&app.chat_widget, /*width*/ 80);
    assert!(
        rendered.contains("your system clock is behind the newest recorded usage"),
        "{rendered}"
    );
    assert!(!rendered.contains("corrupt"), "{rendered}");
    app.state_db.as_ref().unwrap().close().await;
    Ok(())
}

/// #1522: a view whose UTC day starts after the clock (opened while the clock
/// read ahead, refreshed after it was set back) names the clock, not damage.
#[tokio::test]
async fn accounting_inspect_app_day_after_clock_names_the_clock() -> anyhow::Result<()> {
    let path = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    accounting_fixture(&mut app, path.path()).await?;
    // Two days ahead, so the test holds even if it runs across UTC midnight.
    let ahead = chrono::Utc::now().timestamp_millis() / 86_400_000 + 2;
    let loaded = accounting_load(&mut app, &mut rx, ahead).await;
    let AppEvent::AccountingInspectorLoaded {
        result: Err(message),
        ..
    } = &loaded
    else {
        panic!("expected a refused read, got {loaded:?}");
    };
    assert!(
        message.starts_with(
            "Unavailable — your system clock reads earlier than the day this view shows ("
        ),
        "{message}"
    );
    app.handle_accounting_inspector_event(loaded);
    let rendered = render_bottom_popup(&app.chat_widget, /*width*/ 80);
    assert!(
        rendered.contains("your system clock reads earlier"),
        "{rendered}"
    );
    assert!(!rendered.contains("corrupt"), "{rendered}");
    app.state_db.as_ref().unwrap().close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_app_error_retry_and_timeout() -> anyhow::Result<()> {
    let path = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    let (_, day) = accounting_fixture(&mut app, path.path()).await?;
    let db = app.state_db.take().unwrap();
    let missing = accounting_load(&mut app, &mut rx, day).await;
    assert!(
        matches!(&missing, AppEvent::AccountingInspectorLoaded { result: Err(s), .. } if s.contains("database is not open"))
    );
    app.handle_accounting_inspector_event(missing);
    app.state_db = Some(db.clone());
    let event = accounting_load(&mut app, &mut rx, day).await;
    let AppEvent::AccountingInspectorLoaded { generation, .. } = &event else {
        panic!()
    };
    let generation = *generation;
    app.handle_accounting_inspector_event(event);
    tokio::time::pause();
    let timeout =
        crate::app::event_dispatch::accounting_inspector_read_result(std::future::pending()).await;
    tokio::time::resume();
    assert!(timeout.unwrap_err().contains("timed out"));
    app.handle_accounting_inspector_event(AppEvent::RefreshAccountingInspector { generation });
    let AppEvent::LoadAccountingInspector {
        generation: next, ..
    } = rx.recv().await.unwrap()
    else {
        panic!()
    };
    assert_ne!(generation, next);
    db.close().await;
    let failed = accounting_load(&mut app, &mut rx, day).await;
    assert!(
        matches!(&failed, AppEvent::AccountingInspectorLoaded { result: Err(s), .. } if s.contains("could not be read") && !s.contains("SELECT"))
    );
    app.handle_accounting_inspector_event(failed);
    app.chat_widget
        .handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        ));
    app.handle_accounting_inspector_event(rx.recv().await.unwrap());
    assert!(
        !render_bottom_popup(&app.chat_widget, /*width*/ 80).contains("Cost — this conversation")
    );
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_app_restart_and_profile_isolation() -> anyhow::Result<()> {
    let path = tempdir()?;
    let fresh = tempdir()?;
    let (mut app, mut rx, _ops) = make_test_app_with_channels().await;
    let (owner, day) = accounting_fixture(&mut app, path.path()).await?;
    let before = accounting_load(&mut app, &mut rx, day).await;
    app.handle_accounting_inspector_event(before);
    app.state_db.take().unwrap().close().await;
    app.chat_widget.clear_accounting_inspector();
    app.state_db = Some(
        codex_state::StateRuntime::init(
            codex_state::SqliteConfig::new_for_testing(path.path().abs()),
            "synthetic".into(),
        )
        .await?,
    );
    let reopened = accounting_load(&mut app, &mut rx, day).await;
    assert!(
        matches!(reopened, AppEvent::AccountingInspectorLoaded { result: Ok(codex_state::accounting::InspectionDay::Ready(v)), .. } if v.owner == owner && v.totals.attempts == 1)
    );
    app.state_db.take().unwrap().close().await;
    app.chat_widget.update_account_state(
        /*status_account_display*/ None, /*plan_type*/ None,
        /*has_chatgpt_account*/ false, /*has_codex_backend_auth*/ false,
    );
    app.state_db = Some(
        codex_state::StateRuntime::init(
            codex_state::SqliteConfig::new_for_testing(fresh.path().abs()),
            "synthetic".into(),
        )
        .await?,
    );
    let empty = accounting_load(&mut app, &mut rx, day).await;
    assert!(matches!(
        &empty,
        AppEvent::AccountingInspectorLoaded {
            result: Ok(codex_state::accounting::InspectionDay::Absent),
            ..
        }
    ));
    app.handle_accounting_inspector_event(empty);
    assert!(!accounting_scroll(&mut app).contains("0.000180"));
    app.state_db.take().unwrap().close().await;
    Ok(())
}

#[tokio::test]
async fn accounting_inspect_app_off_and_old_usage_routes() -> anyhow::Result<()> {
    let (mut app, mut rx, mut ops) = make_test_app_with_channels().await;
    let event = accounting_load(&mut app, &mut rx, /*day*/ 0).await;
    assert!(
        matches!(&event, AppEvent::AccountingInspectorLoaded { result: Err(s), .. } if s.contains("no current native thread"))
    );
    app.handle_accounting_inspector_event(event);
    assert!(ops.try_recv().is_err());
    assert!(rx.try_recv().is_err());
    app.chat_widget.clear_accounting_inspector();
    set_chatgpt_auth(&mut app.chat_widget);
    app.chat_widget.handle_paste("/usage".into());
    app.chat_widget
        .handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
    app.chat_widget
        .handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
    // Upstream's analytics dashboard replaced main's token-activity card; the
    // menu may send other events first, but never the accounting inspector's.
    let events = std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AppEvent::OpenAnalytics { view: None })),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|event| matches!(
            event,
            AppEvent::OpenAccountingInspector { .. } | AppEvent::LoadAccountingInspector { .. }
        )),
        "{events:?}"
    );
    Ok(())
}
