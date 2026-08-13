use super::*;

use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call_with_namespace;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_sequence;
use core_test_support::responses::mount_sse_repeating;
use core_test_support::responses::mount_sse_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::sse_completed;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

#[tokio::test]
async fn operator_assignment_birth_brief_routes_as_normal_user_pane_turn() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "operator-birth-brief",
        "Manager must coordinate the operator-owned Worker pane.",
    );
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000463").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000464").expect("worker id");
    app.upsert_agent_picker_thread(manager_thread_id, Some("Manager".to_string()), None, false);
    app.upsert_agent_picker_thread(worker_thread_id, Some("Worker".to_string()), None, false);
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);

    app.handle_orchestrate_command(format!(
        "attach {worker_node} operator-birth-brief --mode review --holder {manager_node} --for 1h"
    ));

    let mut manager_tasks = Vec::new();
    while let Ok(event) = app_event_rx.try_recv() {
        match event {
            AppEvent::SubmitCodexUserPaneTask { thread_id, task }
                if thread_id == manager_thread_id =>
            {
                manager_tasks.push(task);
            }
            AppEvent::SubmitSpawnAgentTask { thread_id, .. } if thread_id == manager_thread_id => {
                panic!("operator-owned Manager must never receive a collaboration mailbox task");
            }
            _ => {}
        }
    }
    pretty_assertions::assert_eq!(manager_tasks.len(), 1);
    assert!(manager_tasks[0].contains("Worker"));
}

#[tokio::test]
async fn native_manager_host_adapter_dispatches_only_from_completed_assignment_turn() {
    let (mut app, mut app_event_rx, _op_rx) = make_test_app_with_channels().await;
    write_test_whip(
        &app,
        "native-manager-dispatch",
        "Manager must dispatch the implementation to Worker.",
    );
    let manager_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000465").expect("manager id");
    let worker_thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000466").expect("worker id");
    app.upsert_agent_picker_thread(manager_thread_id, Some("Manager".to_string()), None, false);
    app.upsert_agent_picker_thread(worker_thread_id, Some("Worker".to_string()), None, false);
    let manager_node = crate::spawn_orchestration::thread_node_id(manager_thread_id);
    let worker_node = crate::spawn_orchestration::thread_node_id(worker_thread_id);
    app.handle_orchestrate_command(format!(
        "attach {worker_node} native-manager-dispatch --mode review --holder {manager_node} --for 1h"
    ));
    while app_event_rx.try_recv().is_ok() {}

    let dispatch = format!(
        "```pfterminal-send-task\n{{\"target\":\"{worker_node}\",\"task\":\"Review the branch and report concrete defects.\"}}\n```"
    );
    let ServerNotification::ItemCompleted(mut injected_instruction) = item_completed_notification(
        manager_thread_id,
        "manager-dispatch-1",
        "injected-manager-brief",
        &dispatch,
    ) else {
        unreachable!("helper returns ItemCompleted");
    };
    let ThreadItem::AgentMessage { phase, .. } = &mut injected_instruction.item else {
        unreachable!("helper returns AgentMessage");
    };
    *phase = Some(codex_protocol::models::MessagePhase::Commentary);
    app.enqueue_thread_notification(
        manager_thread_id,
        ServerNotification::ItemCompleted(injected_instruction),
    )
    .await
    .expect("enqueue injected Manager instruction");
    assert!(
        drain_spawn_agent_task_for(&mut app_event_rx, worker_thread_id).is_none(),
        "an injected inter-agent instruction must never execute its example host block"
    );
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Drafting,
            ..
        })
    ));

    app.enqueue_thread_notification(
        manager_thread_id,
        item_completed_notification(
            manager_thread_id,
            "manager-dispatch-1",
            "agent-message-1",
            &dispatch,
        ),
    )
    .await
    .expect("enqueue inactive Manager message completion");
    assert!(
        drain_spawn_agent_task_for(&mut app_event_rx, worker_thread_id).is_none(),
        "native Manager output is display text, never an inter-agent transport"
    );

    let completed = turn_completed_with_agent_message(
        manager_thread_id,
        "manager-dispatch-1",
        TurnStatus::Completed,
        &dispatch,
    );
    app.enqueue_thread_notification(manager_thread_id, completed.clone())
        .await
        .expect("enqueue inactive Manager completion");
    let worker_task = drain_spawn_agent_task_for(&mut app_event_rx, worker_thread_id)
        .expect("completed Manager output should dispatch through the assignment host adapter");
    assert!(worker_task.contains("Review the branch and report concrete defects."));
    assert!(matches!(
        app.orchestrate_whips
            .get("assignment-1")
            .map(|whip| &whip.kind),
        Some(crate::orchestrate::WhipKind::Assignment {
            phase: crate::orchestrate::AssignmentPhase::Executing,
            ..
        })
    ));

    app.enqueue_thread_notification(manager_thread_id, completed)
        .await
        .expect("replay inactive Manager completion");
    assert!(
        drain_spawn_agent_task_for(&mut app_event_rx, worker_thread_id).is_none(),
        "terminal notification replay must not dispatch the same host task twice"
    );
}

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
        if let Ok(crash_home) = std::env::var("PFTERMINAL_DISPATCH_CRASH_HOME") {
            app.config.codex_home = codex_utils_absolute_path::AbsolutePathBuf::try_from(
                std::path::PathBuf::from(crash_home),
            )?;
        }
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

fn pending_agent_message_text(request: &core_test_support::responses::ResponsesRequest) -> String {
    let mut messages = Vec::new();
    for item in request.input().into_iter().rev() {
        if item.get("type").and_then(serde_json::Value::as_str) == Some("message")
            && item.get("role").and_then(serde_json::Value::as_str) == Some("assistant")
        {
            break;
        }

        let item_type = item.get("type").and_then(serde_json::Value::as_str);
        let is_native_mail = item_type == Some("agent_message");
        let is_adapted_mail = item_type == Some("message")
            && item.get("role").and_then(serde_json::Value::as_str) == Some("user")
            && item
                .get("content")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|content| {
                    content.iter().any(|span| {
                        span.get("text")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|text| text.contains("<inter_agent_message "))
                    })
                });
        if !is_native_mail && !is_adapted_mail {
            continue;
        }
        let text = item
            .get("content")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|content| content.get("text").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            messages.push(text);
        }
    }
    messages.reverse();
    messages.join("\n")
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
fn duplicate_nazgul_creation_is_rejected_before_spawning_a_thread() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 4).await?;
        fixture.app.ensure_custom_spawn_root(CODEX_MAIN_PANE_ID)?;
        fixture
            .app
            .set_spawn_nazgul_pane_binding(CODEX_MAIN_PANE_ID.to_string());

        let channels_before = fixture.app.thread_event_channels.len();
        let navigation_before = fixture.app.agent_navigation.ordered_threads().len();
        let spawn_edges_before = fixture.app.spawn_parent_by_thread.len();
        fixture
            .app
            .handle_event(
                &mut fixture.tui,
                &mut fixture.server,
                AppEvent::CreateSpawnAgent {
                    role: crate::spawn_orchestration::SpawnRole::Nazgul,
                    parent_node_id: None,
                    agent_nickname: Some("Duplicate".to_string()),
                    model: "dispatch-mock-model".to_string(),
                    provider: Some("dispatch_mock".to_string()),
                    effort: None,
                },
            )
            .await?;

        pretty_assertions::assert_eq!(fixture.app.thread_event_channels.len(), channels_before);
        pretty_assertions::assert_eq!(
            fixture.app.agent_navigation.ordered_threads().len(),
            navigation_before
        );
        pretty_assertions::assert_eq!(fixture.app.spawn_parent_by_thread.len(), spawn_edges_before);
        pretty_assertions::assert_eq!(
            fixture.app.spawn_nazgul_pane_id.as_deref(),
            Some(CODEX_MAIN_PANE_ID)
        );
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn direct_orc_task_reaches_provider_through_core_mailbox_without_prompt_wrapper() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_sse_repeating(&mock, sse_completed("orc-direct-response")).await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 3).await?;
        let target = fixture.spawn_target("Snaga").await?;
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: "run the direct acceptance command".to_string(),
            });

        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                !responses.requests().is_empty()
                    && app
                        .agent_navigation
                        .get(&target)
                        .is_some_and(|entry| !entry.is_running)
            })
            .await?;

        let presented = responses
            .requests()
            .iter()
            .map(pending_agent_message_text)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(presented.contains("run the direct acceptance command"));
        assert!(!presented.contains("<pfterminal_spawn_orc_task_context>"));
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn operator_pane_dispatch_starts_normal_turn_with_target_session_model() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_sse_repeating(&mock, sse_completed("operator-pane-response")).await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 3).await?;
        fixture
            .app
            .handle_event(
                &mut fixture.tui,
                &mut fixture.server,
                AppEvent::CreateCodexPane {
                    model: "dispatch-worker-model".to_string(),
                    provider: Some("dispatch_mock".to_string()),
                    effort: None,
                    display_name: Some("Worker".to_string()),
                },
            )
            .await?;
        let target = fixture
            .app
            .active_thread_id
            .expect("created operator pane is selected");
        assert_ne!(target, fixture.root);

        // Exercise inactive-pane routing: dispatch must use the target's cached session instead of
        // whichever model happens to be visible in Main.
        fixture.app.active_thread_id = Some(fixture.root);
        let task = "run the operator-pane acceptance command";
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitCodexUserPaneTask {
                thread_id: target,
                task: task.to_string(),
            });
        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                !responses.requests().is_empty()
                    && app
                        .agent_navigation
                        .get(&target)
                        .is_some_and(|entry| !entry.is_running)
            })
            .await?;

        let requests = responses.requests();
        pretty_assertions::assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert!(
            request
                .message_input_texts("user")
                .join("\n")
                .contains(task)
        );
        assert!(
            !request
                .body_json()
                .to_string()
                .contains("<inter_agent_message ")
        );
        pretty_assertions::assert_eq!(
            request
                .body_json()
                .get("model")
                .and_then(serde_json::Value::as_str),
            Some("dispatch-worker-model")
        );
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn cold_restored_orc_task_is_readmitted_through_core_mailbox() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_sse_repeating(&mock, sse_completed("cold-restored-response")).await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 3).await?;
        let target = fixture.spawn_target("Snaga").await?;
        let root_seed = AppCommand::user_turn(
            vec![codex_app_server_protocol::UserInput::Text {
                text: "seed the root rollout before restart".to_string(),
                text_elements: Vec::new(),
            }],
            fixture.app.config.cwd.to_path_buf(),
            codex_app_server_protocol::AskForApproval::Never,
            /*active_permission_profile*/ None,
            fixture.app.config.model.clone().expect("fixture model"),
            fixture.app.config.model_reasoning_effort.clone(),
            fixture.app.config.model_reasoning_summary,
            /*service_tier*/ None,
            /*final_output_json_schema*/ None,
            /*collaboration_mode*/ None,
            fixture.app.config.personality,
        );
        fixture
            .app
            .submit_thread_op(&mut fixture.server, fixture.root, root_seed)
            .await?;
        fixture
            .route_until(std::time::Duration::from_secs(20), |_app| {
                !responses.requests().is_empty()
            })
            .await?;
        for _ in 0..10 {
            fixture.route_once().await?;
        }
        let requests_after_root_seed = responses.requests().len();
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: "seed the child rollout before restart".to_string(),
            });
        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                responses.requests().len() > requests_after_root_seed
                    && app
                        .agent_navigation
                        .get(&target)
                        .is_some_and(|entry| !entry.is_running)
            })
            .await?;
        let requests_before_restart = responses.requests().len();

        fixture.server.shutdown().await?;
        let mut restarted = Box::pin(crate::start_embedded_app_server_for_picker(
            &fixture.app.config,
        ))
        .await?;
        let root = restarted
            .resume_thread(
                fixture.app.config.clone(),
                fixture.root,
                crate::app_server_session::ResumeModelSettings::RestoreFromThread,
                crate::app_server_session::ResumePermissionSettings::RestoreFromThread,
            )
            .await?;
        fixture.app.primary_session_configured = Some(root.session);
        let restored_target = restarted
            .resume_thread(
                fixture.app.config.clone(),
                target,
                crate::app_server_session::ResumeModelSettings::RestoreFromThread,
                crate::app_server_session::ResumePermissionSettings::RestoreFromThread,
            )
            .await?;
        let channel = fixture.app.ensure_thread_channel(target);
        channel.mark_live();
        channel
            .set_session(restored_target.session, restored_target.turns)
            .await;
        fixture.server = restarted;

        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: "run the cold-restored acceptance command".to_string(),
            });
        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                responses
                    .requests()
                    .into_iter()
                    .skip(requests_before_restart)
                    .any(|request| {
                        pending_agent_message_text(&request)
                            .contains("run the cold-restored acceptance command")
                    })
                    && app
                        .agent_navigation
                        .get(&target)
                        .is_some_and(|entry| !entry.is_running)
            })
            .await?;

        let presented = responses
            .requests()
            .into_iter()
            .skip(requests_before_restart)
            .map(|request| pending_agent_message_text(&request))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(presented.contains("run the cold-restored acceptance command"));
        assert!(!presented.contains("<pfterminal_spawn_orc_task_context>"));
        assert!(
            !fixture.app.spawn_processed_dispatch_origins.is_empty(),
            "Core mailbox admission must commit dispatch identity"
        );
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn real_event_path_preserves_fifo_when_mailbox_coalesces_turns() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let responses = mount_sse_repeating(&mock, sse_completed("dispatch-response")).await;
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
                });
        }

        fixture
            .route_until(std::time::Duration::from_secs(20), |app| {
                !responses.requests().is_empty()
                    && app
                        .agent_navigation
                        .get(&target)
                        .is_some_and(|entry| !entry.is_running)
            })
            .await?;

        let requests = responses.requests();
        assert!(
            (1..=tasks.len()).contains(&requests.len()),
            "mailbox coalescing must use between one and one request per task"
        );
        let presented = requests
            .iter()
            .map(pending_agent_message_text)
            .collect::<Vec<_>>();
        let first_request_by_task = tasks
            .iter()
            .map(|task| {
                presented
                    .iter()
                    .position(|body| body.contains(task))
                    .expect("every mailbox task must reach a provider boundary")
            })
            .collect::<Vec<_>>();
        pretty_assertions::assert_eq!(first_request_by_task.first(), Some(&0));
        pretty_assertions::assert_eq!(
            first_request_by_task.last(),
            Some(&(requests.len().saturating_sub(1)))
        );
        assert!(
            first_request_by_task
                .windows(2)
                .all(|pair| pair[0] <= pair[1]),
            "mailbox tasks crossed provider request boundaries out of order"
        );
        for (index, pair) in first_request_by_task.windows(2).enumerate() {
            if pair[0] == pair[1] {
                let earlier = presented[pair[0]].find(tasks[index]).expect("earlier task");
                let later = presented[pair[1]]
                    .find(tasks[index + 1])
                    .expect("later task");
                assert!(
                    earlier < later,
                    "coalesced mailbox work must retain FIFO order"
                );
            }
        }
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn native_assistant_text_replay_never_becomes_mailbox_work() -> Result<()> {
    run_dispatch_integration(|| async {
        let mock = MockServer::start().await;
        let mut fixture = RealDispatchFixture::start(&mock, /*max_threads*/ 4).await?;
        let source = fixture.spawn_agent("ReplayManager", "troll").await?;
        let target = fixture.spawn_target("ReplayTarget").await?;
        let task = "native assistant text must never enter the mailbox";
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

        for _ in 0..10 {
            fixture.route_once().await?;
        }
        assert!(
            mock.received_requests()
                .await
                .unwrap_or_default()
                .is_empty(),
            "native assistant prose must not be parsed as an assignment"
        );
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
            fixture
                .app
                .app_event_tx
                .send(AppEvent::SubmitSpawnAgentTask {
                    thread_id: *target,
                    task: format!("capacity-target-{index}-initial"),
                });
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
        for (index, target) in targets.iter().enumerate() {
            fixture
                .app
                .app_event_tx
                .send(AppEvent::SubmitSpawnAgentTask {
                    thread_id: *target,
                    task: format!("capacity-target-{index}-followup"),
                });
        }
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
            .map(pending_agent_message_text)
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
            .route_until(std::time::Duration::from_secs(8), |_| {
                responses.requests().len() == 6
            })
            .await?;
        pretty_assertions::assert_eq!(responses.requests().len(), 6);
        fixture.server.shutdown().await?;
        Ok(())
    })
}

#[test]
fn mailbox_delivery_wakes_waiting_target_without_turn_start_fallback() -> Result<()> {
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
                    ev_function_call_with_namespace(
                        "wait-call",
                        "collaboration",
                        "wait_agent",
                        r#"{"timeout_ms":10000}"#,
                    ),
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
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |app| {
                responses.requests().len() == 1
                    && app.spawn_waiting_for_agents_by_thread.contains_key(&target)
            })
            .await
            .wrap_err("waiting target never entered the real wait state")?;

        let steer_task = "continue from the wait using this mailbox task";
        fixture
            .app
            .app_event_tx
            .send(AppEvent::SubmitSpawnAgentTask {
                thread_id: target,
                task: steer_task.to_string(),
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |_| {
                responses.requests().len() == 2
            })
            .await
            .wrap_err("mailbox delivery did not wake the waiting target")?;

        let requests = responses.requests();
        pretty_assertions::assert_eq!(
            requests.len(),
            2,
            "mailbox delivery must wake the existing wait without duplicate provider turns"
        );
        assert!(requests[0].body_json().to_string().contains(initial_task));
        assert!(!requests[0].body_json().to_string().contains(steer_task));
        assert!(
            requests[1].body_json().to_string().contains(steer_task),
            "the mailbox task must appear in the follow-up provider request; users={:?}",
            requests[1].message_input_texts("user")
        );
        assert!(
            fixture.app.spawn_processed_dispatch_origins.len() >= 2,
            "canonical mailbox admissions must retain their stable source origins"
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
            });
        for _ in 0..20 {
            fixture.route_once().await?;
        }

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
            rendered.contains("maximum") && rendered.contains("Could not admit task"),
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
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |_| {
                responses.requests().len() == 1
            })
            .await?;

        fixture
            .app
            .handle_app_server_event(
                &fixture.server,
                codex_app_server_client::AppServerEvent::ServerNotification(Box::new(
                    token_usage_notification_with_total(
                        target,
                        "pressure-turn",
                        99_000,
                        Some(100_000),
                    ),
                )),
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
            });
        fixture
            .route_until(std::time::Duration::from_secs(10), |_| {
                responses.requests().len() == 3
            })
            .await?;

        let requests = responses.requests();
        assert!(pending_agent_message_text(&requests[0]).contains(before));
        assert!(pending_agent_message_text(&requests[2]).contains(after));
        pretty_assertions::assert_eq!(requests.len(), 3);
        fixture.server.shutdown().await?;
        Ok(())
    })
}
