//! Issue863 call-path tests using the existing capturing ThreadManager; no inference.
use super::*;
use pretty_assertions::assert_eq;

async fn custom_source_context() -> (crate::session::session::Session, TurnContext) {
    let (session, turn) = make_session_and_context().await;
    let mut turn = turn
        .with_model("gpt-5.6-sol".to_string(), &session.services.models_manager)
        .await;
    let mut config = (*turn.config).clone();
    let mut provider = config.model_provider.clone();
    provider.name = "My OpenAI".to_string();
    provider.base_url = Some("https://api.openai.com/v1".to_string());
    provider.wire_api = codex_model_provider_info::WireApi::Responses;
    provider.requires_openai_auth = false;
    provider.env_key = Some("SYNTHETIC_ISSUE863_API_KEY".to_string());
    config.model_provider_id = "myopenai".to_string();
    config.model_provider = provider.clone();
    config
        .model_providers
        .insert("myopenai".to_string(), provider.clone());
    config
        .features
        .enable(Feature::MultiAgentV2)
        .expect("feature");
    turn.provider = create_model_provider(provider, turn.auth_manager.clone());
    set_turn_config(&mut turn, config);
    (session, turn)
}

#[tokio::test]
async fn issue863_custom_native_spawn_send_followup_keep_source_and_ciphertext() {
    let (mut session, turn) = custom_source_context().await;
    let manager = thread_manager();
    let root = manager
        .start_thread((*turn.config).clone())
        .await
        .expect("root");
    session.services.agent_control = manager.agent_control();
    session.thread_id = root.thread_id;
    let session = Arc::new(session);
    let turn = Arc::new(turn);
    let spawned = SpawnAgentHandlerV2::default()
        .handle(invocation(
            session.clone(),
            turn.clone(),
            "spawn_agent",
            function_payload(json!({"message":"opaque-spawn", "task_name":"issue863_child"})),
        ))
        .await
        .expect("native same-source spawn must be admitted");
    let (text, _) = expect_text_output(spawned);
    let result: serde_json::Value = serde_json::from_str(&text).expect("spawn JSON");
    assert_eq!(result["model_provider"], json!("myopenai"));
    assert_eq!(result["task_name"], json!("/root/issue863_child"));
    let child = session
        .services
        .agent_control
        .resolve_agent_reference(session.thread_id, &turn.session_source, "issue863_child")
        .await
        .expect("child reference");
    let child_config = manager
        .get_thread(child)
        .await
        .expect("child")
        .config_snapshot()
        .await;
    assert_eq!(child_config.model_provider_id, "myopenai");

    SendMessageHandlerV2
        .handle(invocation(
            session.clone(),
            turn.clone(),
            "send_message",
            function_payload(json!({"target":"issue863_child", "message":"opaque-send"})),
        ))
        .await
        .expect("same-source send");
    FollowupTaskHandlerV2
        .handle(invocation(
            session.clone(),
            turn.clone(),
            "followup_task",
            function_payload(json!({"target":"issue863_child", "message":"opaque-followup"})),
        ))
        .await
        .expect("same-source followup");
    let messages: Vec<_> = manager
        .captured_ops()
        .iter()
        .filter_map(|(id, op)| {
            if *id != child {
                return None;
            }
            match op {
                Op::InterAgentCommunication { communication } => Some(json!({
                    "content": communication.content,
                    "encrypted_content": communication.encrypted_content,
                    "trigger_turn": communication.trigger_turn,
                    "recipient": communication.recipient.as_str(),
                })),
                _ => None,
            }
        })
        .collect();
    assert_eq!(
        messages,
        vec![
            json!({"content":"", "encrypted_content":"opaque-spawn", "trigger_turn":true, "recipient":"/root/issue863_child"}),
            json!({"content":"", "encrypted_content":"opaque-send", "trigger_turn":false, "recipient":"/root/issue863_child"}),
            json!({"content":"", "encrypted_content":"opaque-followup", "trigger_turn":true, "recipient":"/root/issue863_child"}),
        ]
    );
}

#[tokio::test]
async fn issue863_custom_native_send_followup_reject_builtin_before_mailbox_mutation() {
    let (mut session, turn) = custom_source_context().await;
    let manager = thread_manager();
    let root = manager
        .start_thread((*turn.config).clone())
        .await
        .expect("root");
    session.services.agent_control = manager.agent_control();
    session.thread_id = root.thread_id;
    let builtin_path = AgentPath::try_from("/root/builtin").expect("path");
    let mut config = (*turn.config).clone();
    config.model_provider_id = OPENAI_PROVIDER_ID.to_string();
    config.model_provider = config.model_providers[OPENAI_PROVIDER_ID].clone();
    let builtin = session
        .services
        .agent_control
        .spawn_agent_with_metadata(
            config,
            Vec::new(),
            Some(SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
                parent_thread_id: root.thread_id,
                depth: 1,
                agent_path: Some(builtin_path.clone()),
                agent_nickname: Some("Synthetic".to_string()),
                agent_role: Some("orc".to_string()),
                agent_class: None,
            })),
            crate::agent::control::SpawnAgentOptions::default(),
        )
        .await
        .expect("register built-in child without inference");
    let session = Arc::new(session);
    let turn = Arc::new(turn);
    for tool in ["send_message", "followup_task"] {
        let call = invocation(
            session.clone(),
            turn.clone(),
            tool,
            function_payload(json!({"target":builtin_path, "message":"opaque-cross-source"})),
        );
        let result = if tool == "send_message" {
            SendMessageHandlerV2.handle(call).await
        } else {
            FollowupTaskHandlerV2.handle(call).await
        };
        let error = result
            .err()
            .expect("cross-source native payload must be rejected");
        assert_eq!(
            error,
            FunctionCallError::RespondToModel(format!(
                "target provider `openai` cannot consume this OpenAI-encrypted collaboration payload. Call the function named `{tool}_plaintext` directly with exactly the same `target` and the plaintext `message`; do not call `{tool}` again and do not add an `adapter` field. No message was admitted and no target turn was started."
            ))
        );
    }
    let captured: Vec<_> = manager
        .captured_ops()
        .iter()
        .filter_map(|(id, op)| {
            if *id == builtin.thread_id {
                if let Op::InterAgentCommunication { communication } = op {
                    return Some(communication.clone());
                }
            }
            None
        })
        .collect();
    assert_eq!(captured, Vec::new());
}
