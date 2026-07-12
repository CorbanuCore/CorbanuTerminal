use super::*;

use core_test_support::responses::ResponseMock;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_sequence;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::sse_completed;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

struct RealDispatchFixture {
    app: App,
    app_events: tokio::sync::mpsc::UnboundedReceiver<AppEvent>,
    server: AppServerSession,
    tui: crate::tui::Tui,
    root: ThreadId,
}

impl RealDispatchFixture {
    async fn start(mock: &MockServer, max_threads: usize) -> Result<Self> {
        let (mut app, app_events, _op_rx) = make_test_app_with_channels().await;
        let mut provider = app.config.model_provider.clone();
        provider.name = "Dispatch integration mock".to_string();
        provider.base_url = Some(format!("{}/v1", mock.uri()));
        provider.env_key = None;
        provider.experimental_bearer_token = None;
        provider.supports_websockets = false;
        provider.requires_openai_auth = false;
        provider.request_max_retries = Some(0);
        provider.stream_max_retries = Some(0);
        app.config.model = Some("dispatch-mock-model".to_string());
        app.config.model_provider_id = "dispatch_mock".to_string();
        app.config.model_provider = provider.clone();
        app.config
            .model_providers
            .insert(app.config.model_provider_id.clone(), provider);
        std::fs::write(
            app.config.codex_home.join("config.toml"),
            format!(
                r#"model = "dispatch-mock-model"
model_provider = "dispatch_mock"

[model_providers.dispatch_mock]
name = "Dispatch integration mock"
base_url = "{}/v1"
wire_api = "responses"
request_max_retries = 0
stream_max_retries = 0
supports_websockets = false
"#,
                mock.uri()
            ),
        )?;
        app.config.multi_agent_v2.max_concurrent_threads_per_session = max_threads;
        if let Some(env_key) = app.config.model_provider.env_key.as_deref() {
            std::fs::write(
                app.config.codex_home.join("provider_auth.json"),
                format!(r#"{{"api_keys":{{"{env_key}":"qualification-key"}}}}"#),
            )?;
        }
        let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
        let root = server.start_thread(&app.config).await?;
        let root_id = root.session.thread_id;
        app.primary_thread_id = Some(root_id);
        app.active_thread_id = Some(root_id);
        app.primary_session_configured = Some(root.session);
        let tui = crate::tui::Tui::for_dispatch_integration_test()?;
        Ok(Self {
            app,
            app_events,
            server,
            tui,
            root: root_id,
        })
    }

    async fn spawn_agent_under(
        &mut self,
        nickname: &str,
        role: &str,
        parent: ThreadId,
    ) -> Result<ThreadId> {
        let config = self.app.native_spawn_agent_config()?;
        let started = self
            .server
            .spawn_agent_thread(
                &config,
                parent,
                role.to_string(),
                Some(nickname.to_string()),
                self.app
                    .config
                    .model
                    .clone()
                    .unwrap_or_else(|| "gpt-5".to_string()),
                Some(self.app.config.model_provider_id.clone()),
                self.app.config.model_reasoning_effort.clone(),
                None,
            )
            .await?;
        let thread_id = started.session.thread_id;
        self.app
            .register_spawn_agent_pane(
                thread_id,
                parent,
                crate::spawn_orchestration::thread_node_id(parent),
                Some(nickname.to_string()),
                role,
                started,
                false,
            )
            .await;
        Ok(thread_id)
    }

    async fn spawn_agent(&mut self, nickname: &str, role: &str) -> Result<ThreadId> {
        self.spawn_agent_under(nickname, role, self.root).await
    }

    async fn spawn_target(&mut self, nickname: &str) -> Result<ThreadId> {
        self.spawn_agent(nickname, "orc").await
    }

    async fn route_once(&mut self) -> Result<()> {
        while let Ok(event) = self.app_events.try_recv() {
            self.app
                .handle_event(&mut self.tui, &mut self.server, event)
                .await?;
        }
        if let Ok(Some(event)) = tokio::time::timeout(
            std::time::Duration::from_millis(10),
            self.server.next_event(),
        )
        .await
        {
            self.app.handle_app_server_event(&self.server, event).await;
        }
        tokio::task::yield_now().await;
        Ok(())
    }

    async fn route_until(
        &mut self,
        timeout: std::time::Duration,
        mut done: impl FnMut(&App) -> bool,
    ) -> Result<()> {
        tokio::time::timeout(timeout, async {
            while !done(&self.app) {
                self.route_once().await?;
            }
            Ok::<_, color_eyre::Report>(())
        })
        .await
        .map_err(|_| color_eyre::eyre::eyre!("real dispatch event path timed out"))??;
        Ok(())
    }
}

fn newest_user_text(request: &core_test_support::responses::ResponsesRequest) -> String {
    let Some(item) = request
        .input()
        .into_iter()
        .rev()
        .find(|item| item.get("role").and_then(serde_json::Value::as_str) == Some("user"))
    else {
        return String::new();
    };
    item.get("content")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|content| content.get("text").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>()
        .join("\n")
}

fn run_dispatch_integration<F, Fut>(test: F) -> Result<()>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<()>> + 'static,
{
    std::thread::Builder::new()
        .name("dispatch-integration".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || -> Result<()> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(32 * 1024 * 1024)
                .enable_all()
                .build()?;
            tokio::task::LocalSet::new().block_on(&runtime, test())
        })?
        .join()
        .map_err(|_| color_eyre::eyre::eyre!("dispatch integration thread panicked"))?
}

#[test]
fn real_event_path_delivers_arbitrary_tasks_once_in_fifo_order() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses: ResponseMock = mount_sse_sequence(
            &mock,
            (1..=3)
                .map(|index| sse_completed(&format!("dispatch-response-{index}")))
                .collect(),
        )
        .await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 4).await?;
        let target = fixture.spawn_target("FIFO target").await?;
        let tasks = [
            "arbitrary punctuation: []{} <> & unicode λ",
            "second task\nwith a newline and no protocol markers",
            "third task: exact-once/FIFO? yes!",
        ];
        for task in tasks {
            fixture
                .app
                .app_event_tx
                .send(AppEvent::SubmitSpawnAgentTask {
                    thread_id: target,
                    task: task.to_string(),
                    delivery_id: None,
                });
        }

        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                responses.requests().len() == tasks.len()
                    && app.spawn_pending_dispatches.is_empty()
                    && app.spawn_dispatch_inflight_targets.is_empty()
            })
            .await?;

        let requests = responses.requests();
        pretty_assertions::assert_eq!(requests.len(), tasks.len());
        let presented = requests.iter().map(newest_user_text).collect::<Vec<_>>();
        for (index, task) in tasks.iter().enumerate() {
            assert!(
                presented[index].contains(task),
                "request {index} did not contain its FIFO task"
            );
            pretty_assertions::assert_eq!(
                presented.iter().filter(|text| text.contains(task)).count(),
                1,
                "task {index} was presented more than once"
            );
        }
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn completed_source_replay_does_not_reenqueue_on_real_event_path() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_sse_sequence(&mock, vec![sse_completed("replay-response")]).await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 4).await?;
        let source = fixture.spawn_agent("ReplayManager", "troll").await?;
        let target = fixture.spawn_target("ReplayTarget").await?;
        let task = "replayed source turn must enqueue this exactly once";
        let notification = turn_completed_with_agent_message(
            source,
            "completed-source-turn",
            TurnStatus::Completed,
            &format!(
                "<pfterminal_send_task target=\"ReplayTarget\">\n{task}\n</pfterminal_send_task>"
            ),
        );
        fixture
            .app
            .update_spawn_status_for_thread_notification(&notification);
        fixture
            .app
            .update_spawn_status_for_thread_notification(&notification);

        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                responses.requests().len() == 1
                    && app.spawn_pending_dispatches.is_empty()
                    && app.spawn_dispatch_inflight_targets.is_empty()
            })
            .await?;

        let presented = responses
            .requests()
            .iter()
            .map(newest_user_text)
            .collect::<Vec<_>>();
        pretty_assertions::assert_eq!(presented.len(), 1);
        assert!(presented[0].contains(task));
        pretty_assertions::assert_eq!(
            fixture.app.spawn_processed_terminal_turns.len(),
            1,
            "the completed source turn must have one durable replay marker"
        );
        assert!(fixture.app.agent_navigation.get(&target).is_some());
        fixture.server.shutdown().await?;
        Ok(())
    })
}

fn delayed_sse_response(id: &str, delay: std::time::Duration) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(sse_completed(id))
        .set_delay(delay)
}

#[test]
fn three_real_turns_saturate_and_one_release_schedules_one_followup() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_response_sequence(
            &mock,
            vec![
                delayed_sse_response("capacity-1", std::time::Duration::from_millis(700)),
                delayed_sse_response("capacity-2", std::time::Duration::from_millis(1800)),
                delayed_sse_response("capacity-3", std::time::Duration::from_millis(1800)),
                delayed_sse_response("capacity-4", std::time::Duration::from_millis(120)),
                delayed_sse_response("capacity-5", std::time::Duration::from_millis(120)),
                delayed_sse_response("capacity-6", std::time::Duration::from_millis(120)),
            ],
        )
        .await;
        // Four includes the root, leaving exactly three child execution slots.
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 4).await?;
        let targets = [
            fixture.spawn_target("CapacityA").await?,
            fixture.spawn_target("CapacityB").await?,
            fixture.spawn_target("CapacityC").await?,
        ];
        for (index, target) in targets.iter().enumerate() {
            for suffix in ["initial", "followup"] {
                fixture
                    .app
                    .app_event_tx
                    .send(AppEvent::SubmitSpawnAgentTask {
                        thread_id: *target,
                        task: format!("capacity-target-{index}-{suffix}"),
                        delivery_id: None,
                    });
            }
        }

        fixture
            .route_until(std::time::Duration::from_secs(5), |app| {
                responses.requests().len() == 3
                    && targets.iter().all(|target| {
                        app.agent_navigation
                            .get(target)
                            .is_some_and(|entry| entry.is_running)
                    })
            })
            .await?;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        for _ in 0..5 {
            fixture.route_once().await?;
        }
        pretty_assertions::assert_eq!(
            responses.requests().len(),
            3,
            "queued follow-ups must not start while all three core slots are active"
        );

        fixture
            .route_until(std::time::Duration::from_secs(2), |_| {
                responses.requests().len() == 4
            })
            .await?;
        let first_four = responses
            .requests()
            .iter()
            .map(newest_user_text)
            .collect::<Vec<_>>();
        pretty_assertions::assert_eq!(
            first_four
                .iter()
                .filter(|text| text.contains("followup"))
                .count(),
            1,
            "one released execution slot must schedule exactly one queued target"
        );

        fixture
            .route_until(std::time::Duration::from_secs(8), |app| {
                responses.requests().len() == 6
                    && app.spawn_pending_dispatches.is_empty()
                    && app.spawn_dispatch_inflight_targets.is_empty()
            })
            .await?;
        pretty_assertions::assert_eq!(responses.requests().len(), 6);
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn lost_wait_steer_response_reconciles_without_start_fallback() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 4).await?;
        let target = fixture.spawn_agent("WaitingManager", "troll").await?;
        let _child = fixture
            .spawn_agent_under("WaitingChild", "orc", target)
            .await?;
        let responses = mount_sse_sequence(
            &mock,
            vec![
                sse(vec![
                    ev_response_created("wait-response"),
                    ev_function_call("wait-call", "wait_agent", r#"{"timeout_ms":10000}"#),
                    ev_completed("wait-response"),
                ]),
                sse_completed("steered-response"),
            ],
        )
        .await;
        let initial_task = "enter a real wait until more work arrives";
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: initial_task.to_string(),
                delivery_id: None,
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |app| {
                responses.requests().len() == 1
                    && app.spawn_waiting_for_agents_by_thread.contains_key(&target)
            })
            .await
            .wrap_err("waiting target never entered the real wait state")?;

        let steer_task = "continue from the wait using this steered task";
        fixture
            .server
            .inject_lost_next_turn_steer_response_after_acceptance();
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: steer_task.to_string(),
                delivery_id: None,
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |app| {
                responses.requests().len() == 2
                    && app.spawn_pending_dispatches.is_empty()
                    && app.spawn_dispatch_inflight_targets.is_empty()
            })
            .await
            .wrap_err("lost steer response did not reconcile to an empty queue")?;

        let requests = responses.requests();
        pretty_assertions::assert_eq!(
            requests.len(),
            2,
            "an ambiguous steer response must not fall back to a new turn/start"
        );
        assert!(newest_user_text(&requests[0]).contains(initial_task));
        assert!(!requests[0].body_json().to_string().contains(steer_task));
        assert!(
            requests[1].body_json().to_string().contains(steer_task),
            "the accepted steer identity must appear in the follow-up provider request; users={:?}",
            requests[1].message_input_texts("user")
        );
        assert!(
            fixture.app.spawn_accepted_delivery_ids.len() >= 2,
            "reconciliation must tombstone the accepted steer identity"
        );
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn queue_bound_rejection_is_visible_and_accepts_nothing() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 3).await?;
        let target = fixture.spawn_target("BoundedTarget").await?;
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: "x".repeat(crate::dispatch_queue::MAX_DISPATCH_TASK_BYTES + 1),
                delivery_id: None,
            });
        for _ in 0..20 {
            fixture.route_once().await?;
        }

        assert!(fixture.app.spawn_pending_dispatches.is_empty());
        assert!(fixture.app.spawn_dispatch_inflight_targets.is_empty());
        assert!(
            mock.received_requests()
                .await
                .unwrap_or_default()
                .is_empty()
        );
        let rendered = fixture
            .app
            .transcript_cells
            .iter()
            .map(|cell| lines_to_single_string(&cell.display_lines(/*width*/ 120)))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("maximum") && rendered.contains("Cannot queue task"),
            "bound rejection must be visible to the source; rendered={rendered:?}"
        );
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn low_context_agent_compacts_and_continues_real_dispatch() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_sse_sequence(
            &mock,
            vec![
                sse_completed("before-compact"),
                sse(vec![
                    ev_response_created("compact-response"),
                    ev_assistant_message("compact-message", "durable compact summary"),
                    ev_completed("compact-response"),
                ]),
                sse_completed("after-compact"),
            ],
        )
        .await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 3).await?;
        let target = fixture.spawn_agent("CompactManager", "troll").await?;
        let before = "dispatch before real compaction";
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: before.to_string(),
                delivery_id: None,
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |app| {
                responses.requests().len() == 1 && app.spawn_pending_dispatches.is_empty()
            })
            .await?;

        fixture
            .app
            .handle_app_server_event(
                &fixture.server,
                codex_app_server_client::AppServerEvent::ServerNotification(
                    token_usage_notification_with_total(
                        target,
                        "pressure-turn",
                        99_000,
                        Some(100_000),
                    ),
                ),
            )
            .await;
        assert!(
            fixture
                .app
                .spawn_context_left_by_thread
                .get(&target)
                .is_some_and(|left| *left <= 1)
        );

        fixture.server.thread_compact_start(target).await?;
        fixture
            .route_until(std::time::Duration::from_secs(10), |app| {
                responses.requests().len() == 2
                    && app
                        .agent_navigation
                        .get(&target)
                        .is_some_and(|entry| !entry.is_running)
            })
            .await?;

        let after = "dispatch after real compaction";
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: after.to_string(),
                delivery_id: None,
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |app| {
                responses.requests().len() == 3
                    && app.spawn_pending_dispatches.is_empty()
                    && app.spawn_dispatch_inflight_targets.is_empty()
            })
            .await?;

        let requests = responses.requests();
        assert!(newest_user_text(&requests[0]).contains(before));
        assert!(newest_user_text(&requests[2]).contains(after));
        pretty_assertions::assert_eq!(requests.len(), 3);
        fixture.server.shutdown().await?;
        Ok(())
    })
}
