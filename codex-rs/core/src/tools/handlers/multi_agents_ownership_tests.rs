use super::*;
use codex_protocol::protocol::InternalSessionSource;
use pretty_assertions::assert_eq;

struct UnloadedAgent {
    session: Arc<crate::session::session::Session>,
    turn: Arc<TurnContext>,
    manager: ThreadManager,
    state: Arc<codex_state::StateRuntime>,
    owner: ThreadId,
    other_root: ThreadId,
    agent: ThreadId,
}

impl UnloadedAgent {
    async fn new() -> Self {
        let (mut session, mut turn) = make_session_and_context().await;
        let mut config = (*turn.config).clone();
        config.features.enable(Feature::MultiAgentV2).unwrap();
        config.features.enable(Feature::Sqlite).unwrap();
        let state = init_state_db(&config).await.expect("state db");
        let manager = ThreadManager::with_models_provider_home_and_state_for_tests(
            CodexAuth::from_api_key("dummy"),
            config.model_provider.clone(),
            config.codex_home.to_path_buf(),
            Arc::new(codex_exec_server::EnvironmentManager::default_for_tests()),
            Some(state.clone()),
        );
        let root = manager
            .start_thread(StartThreadOptions::new(config.clone()))
            .await
            .expect("owner root");
        let other_root = manager
            .start_thread(StartThreadOptions::new(config.clone()))
            .await
            .expect("other root");
        session.services.agent_control = manager.agent_control();
        session.thread_id = root.thread_id;
        set_turn_config(&mut turn, config);
        let session = Arc::new(session);
        let turn = Arc::new(turn);
        SpawnAgentHandlerV2::default()
            .handle(invocation(
                session.clone(),
                turn.clone(),
                "spawn_agent",
                function_payload(json!({"message": "inspect fixture", "task_name": "worker"})),
            ))
            .await
            .expect("spawn owned worker");
        let agent = session
            .services
            .agent_control
            .resolve_agent_reference(session.thread_id, &turn.session_source, "worker")
            .await
            .expect("worker id");
        let thread = manager
            .remove_thread(&agent)
            .await
            .expect("resident worker");
        thread.submit(Op::Shutdown {}).await.expect("stop worker");
        thread.wait_until_terminated().await;
        Self {
            session,
            turn,
            manager,
            state,
            owner: root.thread_id,
            other_root: other_root.thread_id,
            agent,
        }
    }

    fn use_other_root(&mut self) {
        Arc::get_mut(&mut self.session)
            .expect("unique test session")
            .thread_id = self.other_root;
    }

    async fn assert_still_open_and_unloaded(&self) {
        assert!(self.manager.get_thread(self.agent).await.is_err());
        assert!(
            self.session
                .services
                .agent_control
                .get_agent_metadata(self.agent)
                .is_some()
        );
        assert_eq!(
            self.state
                .list_thread_spawn_children_with_status(
                    self.owner,
                    DirectionalThreadSpawnEdgeStatus::Open,
                )
                .await
                .expect("open edges"),
            vec![self.agent],
        );
    }

    async fn stop(self) {
        self.manager
            .shutdown_all_threads_bounded(Duration::from_secs(5))
            .await;
    }
}

#[tokio::test]
async fn issue_1779_close_refuses_unloaded_foreign_agent_without_closing_edge() {
    let mut fixture = UnloadedAgent::new().await;
    fixture.use_other_root();
    let error = CloseAgentHandler
        .handle(invocation(
            fixture.session.clone(),
            fixture.turn.clone(),
            "close_agent",
            function_payload(json!({"target": fixture.agent.to_string()})),
        ))
        .await
        .err()
        .expect("foreign close refused");
    assert!(
        model_facing_error_message(&error)
            .unwrap()
            .contains("can only target agents spawned")
    );
    fixture.assert_still_open_and_unloaded().await;
    fixture.stop().await;
}

#[tokio::test]
async fn issue_1779_send_input_refuses_foreign_agent_before_cold_load() {
    let mut fixture = UnloadedAgent::new().await;
    fixture.use_other_root();
    let before = fixture.manager.captured_ops();
    let error = SendInputHandler.handle(invocation(
        fixture.session.clone(), fixture.turn.clone(), "send_input",
        function_payload(json!({"target": fixture.agent.to_string(), "message": "must not arrive", "interrupt": true})),
    )).await.err().expect("foreign input refused");
    assert!(
        model_facing_error_message(&error)
            .unwrap()
            .contains("can only target agents spawned")
    );
    fixture.assert_still_open_and_unloaded().await;
    assert_eq!(fixture.manager.captured_ops(), before);
    fixture.stop().await;
}

#[tokio::test]
async fn issue_1779_send_input_rechecks_actual_parent_after_cold_load() {
    let mut fixture = UnloadedAgent::new().await;
    // A stale graph says B owns the thread, but its stored source still names A.
    // The pre-load check can pass; the loaded source must win on the second check.
    fixture
        .state
        .upsert_thread_spawn_edge(
            fixture.other_root,
            fixture.agent,
            DirectionalThreadSpawnEdgeStatus::Open,
        )
        .await
        .expect("stale graph fixture");
    fixture.use_other_root();
    let before = fixture.manager.captured_ops();
    let error = SendInputHandler.handle(invocation(
        fixture.session.clone(), fixture.turn.clone(), "send_input",
        function_payload(json!({"target": fixture.agent.to_string(), "message": "must not arrive", "interrupt": true})),
    )).await.err().expect("post-load ownership check");
    assert!(
        model_facing_error_message(&error)
            .unwrap()
            .contains("can only target agents spawned")
    );
    assert!(
        fixture.manager.get_thread(fixture.agent).await.is_ok(),
        "must exercise the post-load check"
    );
    assert_eq!(fixture.manager.captured_ops(), before);
    fixture.stop().await;
}

#[tokio::test]
async fn issue_1779_own_unloaded_agent_accepts_input_and_close() {
    let fixture = UnloadedAgent::new().await;
    SendInputHandler
        .handle(invocation(
            fixture.session.clone(),
            fixture.turn.clone(),
            "send_input",
            function_payload(
                json!({"target": fixture.agent.to_string(), "message": "own follow-up"}),
            ),
        ))
        .await
        .expect("own agent can cold-load");
    assert!(fixture.manager.get_thread(fixture.agent).await.is_ok());
    let thread = fixture.manager.remove_thread(&fixture.agent).await.unwrap();
    thread.submit(Op::Shutdown {}).await.unwrap();
    thread.wait_until_terminated().await;
    CloseAgentHandler
        .handle(invocation(
            fixture.session.clone(),
            fixture.turn.clone(),
            "close_agent",
            function_payload(json!({"target": fixture.agent.to_string()})),
        ))
        .await
        .expect("own unloaded agent can close");
    assert_eq!(
        fixture
            .state
            .list_thread_spawn_children_with_status(
                fixture.owner,
                DirectionalThreadSpawnEdgeStatus::Closed,
            )
            .await
            .unwrap(),
        vec![fixture.agent],
    );
    fixture.stop().await;
}

#[tokio::test]
async fn issue_1779_list_does_not_alias_another_roots_entry() {
    let (mut session, turn) = make_session_and_context().await;
    let manager = thread_manager();
    let config = (*turn.config).clone();
    let registered = manager
        .start_thread(StartThreadOptions::new(config.clone()))
        .await
        .unwrap();
    let caller = manager
        .start_thread(StartThreadOptions::new(config.clone()))
        .await
        .unwrap();
    let pathless_child = start_own_child(&manager, config, caller.thread_id).await;
    session.services.agent_control = manager.agent_control();
    session
        .services
        .agent_control
        .register_session_root(registered.thread_id, /*current_parent_thread_id*/ None);
    let turn = Arc::new(turn);
    // Neither an unregistered root nor a path-less child may borrow /root.
    for id in [caller.thread_id, pathless_child.thread_id] {
        session.thread_id = id;
        let session_arc = Arc::new(session);
        let output = ListAgentsHandlerV2
            .handle(invocation(
                session_arc.clone(),
                turn.clone(),
                "list_agents",
                function_payload(json!({})),
            ))
            .await
            .unwrap();
        let (content, _) = expect_text_output(output);
        let result: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(result["agents"], json!([]), "{content}");
        session = Arc::try_unwrap(session_arc)
            .ok()
            .expect("test owns session");
    }
    manager
        .shutdown_all_threads_bounded(Duration::from_secs(5))
        .await;
}

#[tokio::test]
async fn issue_1779_wait_lists_own_unloaded_agent() {
    let fixture = UnloadedAgent::new().await;
    fixture
        .session
        .input_queue
        .enqueue_mailbox_communication(
            InterAgentCommunication::new(
                AgentPath::try_from("/root/worker").unwrap(),
                AgentPath::root(),
                Vec::new(),
                "completed".to_string(),
                /*trigger_turn*/ false,
            ),
            /*parent_turn_id*/ None,
        )
        .await;
    let output = WaitAgentHandlerV2::default()
        .handle(invocation(
            fixture.session.clone(),
            fixture.turn.clone(),
            "wait_agent",
            function_payload(json!({"timeout_ms": 10_000})),
        ))
        .await
        .unwrap();
    let (content, _) = expect_text_output(output);
    let result: crate::tools::handlers::multi_agents_v2::wait::WaitAgentResult =
        serde_json::from_str(&content).unwrap();
    assert_eq!(result.agents.len(), 1, "{content}");
    assert_eq!(result.agents[0].agent_name, "/root/worker");
    assert_eq!(result.agents[0].agent_status, AgentStatus::Unloaded);
    fixture.stop().await;
}

#[tokio::test]
async fn issue_1779_agent_shutdown_cannot_remove_internal_threads() {
    let (mut session, turn) = make_session_and_context().await;
    let manager = thread_manager();
    let thread = manager
        .start_thread(StartThreadOptions {
            session_source: Some(SessionSource::Internal(
                InternalSessionSource::MemoryConsolidation,
            )),
            environments: Some(Vec::new()),
            ..StartThreadOptions::new((*turn.config).clone())
        })
        .await
        .expect("internal thread");
    session.services.agent_control = manager.agent_control();
    let control = manager.agent_control();
    assert!(
        CloseAgentHandler
            .handle(invocation(
                Arc::new(session),
                Arc::new(turn),
                "close_agent",
                function_payload(json!({"target": thread.thread_id.to_string()})),
            ))
            .await
            .is_err()
    );
    assert!(control.close_agent(thread.thread_id).await.is_err());
    assert!(control.shutdown_live_agent(thread.thread_id).await.is_err());
    assert!(
        manager.remove_thread(&thread.thread_id).await.is_some(),
        "agent operations must leave the internal lifecycle owner in charge"
    );
    thread.thread.submit(Op::Shutdown {}).await.unwrap();
    thread.thread.wait_until_terminated().await;
}
