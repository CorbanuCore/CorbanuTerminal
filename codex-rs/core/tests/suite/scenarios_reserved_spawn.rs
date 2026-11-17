//! D682: host reservations cannot install a policy-less child on a hardened parent.

use super::super::agent_control::TestAgentControl;
use super::*;
use codex_core::CodexThread;
use codex_core::HostThreadSpawnReservation;
use codex_core::HostToolInvocationLookup;
use codex_core::SpawnAgentForkMode;
use codex_core::ThreadCreateRequest;
use codex_core::ThreadSpawnError;
use codex_protocol::ThreadId;
use codex_protocol::dynamic_tools::DynamicToolCallRequest;
use codex_protocol::error::CodexErrorDetails;
use codex_protocol::protocol::ThreadHistoryMode;
use pretty_assertions::assert_eq;
use wiremock::MockServer;

async fn host_spawn_invocation(
    server: &MockServer,
    parent: &Arc<CodexThread>,
    call_id: &str,
    message: &str,
) -> Result<DynamicToolCallRequest> {
    responses::mount_sse_once(
        server,
        sse(vec![
            responses::ev_function_call(call_id, "host_spawn", "{}"),
            ev_completed(call_id),
        ]),
    )
    .await;
    parent
        .start_or_steer_turn(TurnInputRequest::user_input(vec![text(message)]))
        .await?;
    Ok(wait_for_event_match(parent, |event| match event {
        EventMsg::DynamicToolCallRequest(call) => Some(call.clone()),
        _ => None,
    })
    .await)
}

async fn finish_spawn_tool(
    server: &MockServer,
    parent: &CodexThread,
    invocation: &DynamicToolCallRequest,
) -> Result<()> {
    responses::mount_sse_once(server, sse(vec![ev_completed("parent-done")])).await;
    parent
        .submit(Op::DynamicToolResponse {
            id: invocation.call_id.clone(),
            response: DynamicToolResponse {
                content_items: vec![DynamicToolCallOutputContentItem::InputText {
                    text: "Done.".into(),
                }],
                success: true,
            },
        })
        .await?;
    wait_for_event(parent, |event| matches!(event, EventMsg::TurnComplete(_))).await;
    assert!(
        parent
            .tool_invocation(&invocation.turn_id, &invocation.call_id)
            .is_err()
    );
    Ok(())
}

fn create_request(name: &str, fork_history: bool) -> ThreadCreateRequest {
    ThreadCreateRequest {
        task_name: name.into(),
        fork_mode: fork_history.then_some(SpawnAgentForkMode::FullHistory),
        agent_type: None,
        model: None,
        reasoning_effort: None,
    }
}

// D682 retires successful host child creation, inherited attachment/binding
// preservation and post-registration cancellation: D530 forbids registration.
#[test_case::test_case(false, false; "fresh")]
#[test_case::test_case(true, false; "forked")]
#[test_case::test_case(true, true; "revoked_invocation")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn policyless_reserved_child_is_never_registered(
    fork_history: bool,
    revoke: bool,
) -> Result<()> {
    skip_if_no_network!(Ok(()));
    let server = start_mock_server().await;
    let mut test = test_codex()
        .with_model("gpt-6-astra")
        .with_config(|config| {
            configure_scenario_catalog(config);
            config
                .features
                .enable(Feature::MultiAgentV2)
                .expect("enable V2");
            config
                .features
                .enable(Feature::MultiAgentV2DynamicTools)
                .expect("enable dynamic tool inheritance");
            config.multi_agent_v2.disable_native_tools = true;
        })
        .build_with_auto_env(&server)
        .await?;
    let mut options = test.start_thread_options().await;
    options
        .thread_extension_init
        .insert(HostToolInvocationLookup);
    options.history_mode = Some(ThreadHistoryMode::Legacy);
    options.dynamic_tools = vec![DynamicToolSpec::Function(DynamicToolFunctionSpec {
        name: "host_spawn".into(),
        description: "Ask the host to start a child from this invocation.".into(),
        input_schema: json!({"type": "object", "properties": {}}),
        defer_loading: false,
    })];
    test.codex.shutdown_and_wait().await?;
    test.thread_manager
        .remove_thread(&test.session_configured.thread_id)
        .await;
    let root = test.thread_manager.start_thread(options).await?;
    test.codex = Arc::clone(&root.thread);
    test.session_configured = root.session_configured;
    let parent = root.thread;
    let controller = Arc::new(TestAgentControl::new(root.thread_id));
    let invocation =
        host_spawn_invocation(&server, &parent, "spawn-worker", "Start a worker.").await?;
    let context = parent.tool_invocation(&invocation.turn_id, &invocation.call_id)?;
    if revoke {
        finish_spawn_tool(&server, &parent, &invocation).await?;
    }
    let child_id = ThreadId::new();
    let result = parent
        .create_child(
            &context,
            create_request("worker", fork_history),
            HostThreadSpawnReservation {
                thread_id: child_id,
                control: controller,
            },
        )
        .await;
    let error = result
        .err()
        .expect("policy-less reservation must not start a child");
    if revoke {
        assert!(
            matches!(error, ThreadSpawnError::Preparation(_)),
            "{error:?}"
        );
    } else {
        assert!(
            matches!(&error, ThreadSpawnError::Backend(error)
                if matches!(error.details(), CodexErrorDetails::UnsupportedOperation(message)
                    if message == "this agent controller has no Corbanu security policy")),
            "{error:?}"
        );
        finish_spawn_tool(&server, &parent, &invocation).await?;
    }
    assert!(test.thread_manager.get_thread(child_id).await.is_err());
    assert_eq!(
        test.thread_manager.list_thread_ids().await,
        vec![root.thread_id]
    );
    parent.shutdown_and_wait().await?;
    Ok(())
}
