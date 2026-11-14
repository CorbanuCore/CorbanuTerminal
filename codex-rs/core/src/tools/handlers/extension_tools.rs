use std::sync::Arc;
use std::sync::Weak;

use codex_file_system::FileSystemEnvironmentAccessor;
use codex_file_system::FileSystemSandboxContext;
use codex_history::ResponseItemEnvelope;
use codex_protocol::items::TurnItem;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_tools::ConversationHistory;
use codex_tools::ExtensionTurnItem;
use codex_tools::ResponsesApiNamespaceTool;
use codex_tools::ToolCall as ExtensionToolCall;
use codex_tools::ToolEnvironment;
use codex_tools::ToolName;
use codex_tools::ToolSearchInfo;
use codex_tools::ToolSpec;
use codex_tools::TurnItemEmissionFuture;
use codex_tools::TurnItemEmitter;
use codex_utils_string::to_ascii_json_string;

use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::session::turn_context::TurnEnvironment;
use crate::tools::context::ToolEnvironments;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::read_granted_permissions;
use crate::tools::lifecycle::extension_tool_call_source;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use crate::turn_metadata::ExecutionMetadata;

pub(crate) struct ExtensionToolAdapter(
    Arc<dyn for<'call> codex_tools::ToolExecutor<ExtensionToolCall<'call>>>,
);

impl ExtensionToolAdapter {
    pub(crate) fn new(
        executor: Arc<dyn for<'call> codex_tools::ToolExecutor<ExtensionToolCall<'call>>>,
    ) -> Self {
        Self(executor)
    }
}

impl ToolExecutor<ToolInvocation> for ExtensionToolAdapter {
    fn tool_name(&self) -> ToolName {
        self.0.tool_name()
    }

    fn spec(&self) -> ToolSpec {
        self.0.spec()
    }

    fn exposure(&self) -> crate::tools::registry::ToolExposure {
        self.0.exposure()
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        self.0.supports_parallel_tool_calls()
    }

    fn search_info(&self) -> Option<ToolSearchInfo> {
        self.0.search_info()
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            let (contexts, built_under) =
                current_extension_file_contexts(&invocation, &mut || {}).await?;
            let accessors: Vec<_> = contexts
                .into_iter()
                .map(|(environment, sandbox)| {
                    (
                        environment,
                        FileSystemEnvironmentAccessor::new(
                            environment.environment.filesystem_ref(),
                            sandbox,
                        ),
                    )
                })
                .collect();
            let environments = accessors
                .iter()
                .map(|(environment, fs)| {
                    ToolEnvironment::new(
                        environment.selection.environment_id.clone(),
                        environment.cwd().clone(),
                        fs,
                    )
                })
                .collect();
            let call = to_extension_call(&invocation, environments).await;
            // D71 (a): building the call awaits the history; a transition meanwhile
            // refuses it, as one during the file contexts' build would.
            if extension_call_basis(invocation.session.as_ref()) != built_under {
                return Err(codex_tools::FunctionCallError::RespondToModel(
                    EXTENSION_PREPARED_REFUSAL.to_string(),
                ));
            }
            self.0.handle(call).await
        })
    }
}

/// #1057 (D71, option A): why an extension call was not started.
pub(crate) const EXTENSION_PREPARED_REFUSAL: &str = "Not run: the security policy changed \
     while this tool call was being prepared, so the file access it was prepared with was for \
     the earlier policy. Ask again if it is still needed.";

/// The level-dependent state an extension call is built under: the policy
/// marker its grants are fenced by and the live level its file context is
/// protected for.
type ExtensionCallBasis = (Option<(u64, u64)>, codex_security_policy::SecurityLevel);

fn extension_call_basis(session: &Session) -> ExtensionCallBasis {
    (
        session.services.authority_marker(),
        session.services.model_client().live_level(),
    )
}

/// The file contexts an extension call gets for `environment` (upstream passes each selected
/// environment's filesystem to the extension): the session's usable grants (#245), Corbanu's
/// launch contract and Windows no-sandbox refusal (PF-27-S02, D491) and the protected-surface
/// denials (PF-23-S01), as for the other in-process file tools.
async fn extension_file_contexts(
    invocation: &ToolInvocation,
) -> Vec<(&TurnEnvironment, FileSystemSandboxContext)> {
    let selected: Vec<&TurnEnvironment> = match &invocation.environments {
        ToolEnvironments::Arguments(environments) => {
            environments.iter().map(|(_, env)| env.as_ref()).collect()
        }
        _ => invocation
            .step_context
            .environments
            .turn_environments()
            .collect(),
    };
    let mut contexts = Vec::with_capacity(selected.len());
    for environment in selected {
        let granted =
            read_granted_permissions(&invocation.session, &environment.selection.environment_id)
                .await;
        let context = crate::security::protected_surface::protect_file_tool_context(
            &invocation.session,
            &invocation.turn,
            environment,
            invocation
                .turn
                .file_system_sandbox_context(granted, environment),
        );
        contexts.push((environment, context));
    }
    contexts
}

/// #1057 (D71, option A): the extension's file contexts built under the
/// policy in force when the extension starts, with that policy's basis (the
/// caller re-checks it once the call is built). They (grants included) are
/// built from the policy at build time; when a security transition lands
/// while they are built, they are rebuilt once under the new policy, and the
/// call is refused when another lands meanwhile. `after_build` runs after each
/// build (a test seam). A transition while the extension runs is not covered
/// (option B).
async fn current_extension_file_contexts<'i>(
    invocation: &'i ToolInvocation,
    after_build: &mut (dyn FnMut() + Send),
) -> Result<
    (
        Vec<(&'i TurnEnvironment, FileSystemSandboxContext)>,
        ExtensionCallBasis,
    ),
    codex_tools::FunctionCallError,
> {
    for _ in 0..2 {
        let built_under = extension_call_basis(invocation.session.as_ref());
        let contexts = extension_file_contexts(invocation).await;
        after_build();
        if extension_call_basis(invocation.session.as_ref()) == built_under {
            return Ok((contexts, built_under));
        }
    }
    Err(codex_tools::FunctionCallError::RespondToModel(
        EXTENSION_PREPARED_REFUSAL.to_string(),
    ))
}

impl CoreToolRuntime for ExtensionToolAdapter {
    fn tool_origin(&self) -> crate::security::protected_surface::ToolOrigin {
        crate::security::protected_surface::ToolOrigin::Extension
    }

    fn environments_for_input(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<ToolEnvironments, codex_tools::FunctionCallError> {
        let arguments = self.0.environment_arguments(&invocation.payload)?;
        if arguments.is_empty() {
            return Ok(ToolEnvironments::All);
        }
        arguments
            .into_iter()
            .map(|(field, id)| {
                let env = invocation
                    .step_context
                    .environments
                    .turn_environments()
                    .find(|env| env.selection.environment_id == id)
                    .ok_or_else(|| {
                        codex_tools::FunctionCallError::RespondToModel(format!(
                            "unknown or unavailable environment ID: {id}"
                        ))
                    })?;
                Ok((field, Arc::new(env.clone())))
            })
            .collect::<Result<_, _>>()
            .map(ToolEnvironments::Arguments)
    }

    fn is_builtin_control_tool(&self) -> bool {
        let tool_name = self.0.tool_name();
        if tool_name.is_default_namespace() {
            return matches!(
                tool_name.name.as_str(),
                "get_goal" | "create_goal" | "update_goal"
            );
        }
        matches!(
            (tool_name.namespace.as_deref(), tool_name.name.as_str()),
            (
                Some("notes"),
                "list_files_by_prefix"
                    | "read_file"
                    | "search_contents"
                    | "append_to_file"
                    | "write_file"
            ) | (
                Some("history"),
                "list_windows" | "list_items" | "read_item" | "search_contents"
            )
        )
    }

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        match payload {
            ToolPayload::Function { .. } => true,
            ToolPayload::Custom { .. } => match self.0.spec() {
                ToolSpec::Freeform(_) => true,
                ToolSpec::Namespace(namespace) => namespace.tools.iter().any(|tool| {
                    matches!(
                        tool,
                        ResponsesApiNamespaceTool::Custom(tool)
                            if tool.name == self.0.tool_name().name
                    )
                }),
                ToolSpec::Function(_)
                | ToolSpec::ToolSearch { .. }
                | ToolSpec::WebSearch { .. } => false,
            },
            ToolPayload::ToolSearch { .. } => false,
        }
    }
}

struct CoreTurnItemEmitter {
    session: Weak<Session>,
    turn: Weak<TurnContext>,
}

async fn emit_legacy_events(session: &Session, turn: &TurnContext, legacy_events: Vec<EventMsg>) {
    for msg in legacy_events {
        session
            .send_event_raw(Event {
                id: turn.sub_id.clone(),
                msg,
            })
            .await;
    }
}

impl TurnItemEmitter for CoreTurnItemEmitter {
    fn emit_started<'a>(&'a self, item: ExtensionTurnItem) -> TurnItemEmissionFuture<'a> {
        Box::pin(async move {
            let (Some(session), Some(turn)) = (self.session.upgrade(), self.turn.upgrade()) else {
                return;
            };
            let ExtensionTurnItem {
                item,
                legacy_events,
            } = item;
            let item = TurnItem::Extension(item);
            session.emit_turn_item_started(turn.as_ref(), &item).await;
            emit_legacy_events(session.as_ref(), turn.as_ref(), legacy_events).await;
        })
    }

    fn emit_completed<'a>(&'a self, item: ExtensionTurnItem) -> TurnItemEmissionFuture<'a> {
        Box::pin(async move {
            let (Some(session), Some(turn)) = (self.session.upgrade(), self.turn.upgrade()) else {
                return;
            };
            let ExtensionTurnItem {
                item,
                legacy_events,
            } = item;
            let item = TurnItem::Extension(item);
            session.emit_turn_item_completed(turn.as_ref(), item).await;
            emit_legacy_events(session.as_ref(), turn.as_ref(), legacy_events).await;
        })
    }
}

async fn to_extension_call<'a>(
    invocation: &ToolInvocation,
    environments: Vec<ToolEnvironment<'a>>,
) -> ExtensionToolCall<'a> {
    let history = invocation
        .session
        .clone_history()
        .await
        .into_shared_annotated_items();
    let conversation_history = ConversationHistory::new_deferred(move || {
        Arc::unwrap_or_clone(history)
            .into_iter()
            .map(ResponseItemEnvelope::into_item)
            .collect()
    });
    let settings = &invocation.step_context.settings;
    let codex_turn_metadata = invocation
        .turn
        .turn_metadata_state
        .current_meta_value_for_extension_request(ExecutionMetadata::from_settings(
            &invocation.step_context.settings,
        ))
        .and_then(|metadata| to_ascii_json_string(&metadata).ok());
    ExtensionToolCall {
        turn_id: invocation.turn.sub_id.clone(),
        call_id: invocation.call_id.clone(),
        tool_name: invocation.tool_name.clone(),
        model: settings.model_info.slug.clone(),
        codex_turn_metadata,
        truncation_policy: settings.model_info.truncation_policy.into(),
        source: extension_tool_call_source(invocation.source.clone()),
        conversation_history,
        turn_item_emitter: Arc::new(CoreTurnItemEmitter {
            session: Arc::downgrade(&invocation.session),
            turn: Arc::downgrade(&invocation.turn),
        }),
        attachment_store: Arc::clone(&invocation.session.services.image_store),
        ephemeral: invocation.turn.config.ephemeral,
        environments,
        payload: invocation.payload.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use codex_extension_items::ExtensionItem;
    use codex_extension_items::image_generation::GeneratedImageRef;
    use codex_extension_items::image_generation::ImageGenerationItem;
    use codex_extension_items::web_search::WebSearchItem;
    use codex_protocol::items::TurnItem;
    use codex_protocol::models::ContentItem;
    use codex_protocol::models::ResponseItem;
    use codex_protocol::protocol::EventMsg;
    use codex_protocol::protocol::ImageGenerationBeginEvent;
    use codex_protocol::protocol::ImageGenerationEndEvent;
    use codex_tools::ExtensionTurnItem;
    use codex_tools::ToolCallSource as ExtensionToolCallSource;
    use codex_utils_absolute_path::test_support::PathExt;
    use codex_utils_absolute_path::test_support::test_path_buf;
    use codex_utils_path_uri::PathUri;
    use core_test_support::responses::strip_response_item_id;
    use core_test_support::responses::strip_response_item_ids;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tokio::sync::Mutex;

    use super::CoreTurnItemEmitter;
    use super::ExtensionToolAdapter;
    use crate::session::step_context::StepContext;
    use crate::tools::context::ToolCallSource;
    use crate::tools::context::ToolInvocation;
    use crate::tools::context::ToolPayload;
    use crate::tools::hook_names::HookToolName;
    use crate::tools::registry::CoreToolRuntime;
    use crate::tools::registry::PostToolUsePayload;
    use crate::tools::registry::PreToolUsePayload;
    use crate::turn_diff_tracker::TurnDiffTracker;

    struct StubExtensionExecutor;

    impl<'call> codex_extension_api::ToolExecutor<codex_tools::ToolCall<'call>>
        for StubExtensionExecutor
    {
        fn tool_name(&self) -> codex_tools::ToolName {
            codex_tools::ToolName::plain("extension_echo")
        }

        fn spec(&self) -> codex_tools::ToolSpec {
            codex_tools::ToolSpec::Function(codex_tools::ResponsesApiTool {
                name: "extension_echo".to_string(),
                description: "Echoes arguments.".to_string(),
                strict: true,
                parameters: codex_tools::parse_tool_input_schema(&json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" },
                    },
                    "required": ["message"],
                    "additionalProperties": false,
                }))
                .expect("extension schema should parse"),
                output_schema: None,
                defer_loading: None,
            })
        }

        fn handle<'a>(
            &'a self,
            _call: codex_tools::ToolCall<'call>,
        ) -> codex_tools::ToolExecutorFuture<'a>
        where
            'call: 'a,
        {
            Box::pin(async {
                Ok(
                    Box::new(codex_tools::JsonToolOutput::new(json!({ "ok": true })))
                        as Box<dyn codex_tools::ToolOutput>,
                )
            })
        }
    }

    struct CapturingExtensionExecutor {
        captured_call: Arc<Mutex<Option<codex_tools::ToolCall<'static>>>>,
        captured_sandbox_cwds: Arc<Mutex<Vec<PathUri>>>,
    }

    impl<'call> codex_extension_api::ToolExecutor<codex_tools::ToolCall<'call>>
        for CapturingExtensionExecutor
    {
        fn tool_name(&self) -> codex_tools::ToolName {
            codex_tools::ToolName::plain("extension_echo")
        }

        fn spec(&self) -> codex_tools::ToolSpec {
            codex_tools::ToolSpec::Function(codex_tools::ResponsesApiTool {
                name: "extension_echo".to_string(),
                description: "Captures arguments.".to_string(),
                strict: false,
                parameters: codex_tools::JsonSchema::default(),
                output_schema: None,
                defer_loading: None,
            })
        }

        fn handle<'a>(
            &'a self,
            call: codex_tools::ToolCall<'call>,
        ) -> codex_tools::ToolExecutorFuture<'a>
        where
            'call: 'a,
        {
            Box::pin(self.handle_call(call))
        }
    }

    impl CapturingExtensionExecutor {
        async fn handle_call(
            &self,
            call: codex_tools::ToolCall<'_>,
        ) -> Result<Box<dyn codex_tools::ToolOutput>, codex_tools::FunctionCallError> {
            call.turn_item_emitter
                .emit_started(ExtensionTurnItem {
                    item: ExtensionItem::WebSearch(WebSearchItem {
                        id: call.call_id.clone(),
                        query: String::new(),
                        action: None,
                        results: None,
                    }),
                    legacy_events: Vec::new(),
                })
                .await;
            // Record owned metadata only; the invocation lifetime belongs to this callback.
            *self.captured_sandbox_cwds.lock().await = call
                .environments
                .iter()
                .map(|environment| environment.cwd.clone())
                .collect();
            let call = codex_tools::ToolCall {
                environments: Vec::new(),
                ..call
            };
            *self.captured_call.lock().await = Some(call);
            Ok(
                Box::new(codex_tools::JsonToolOutput::new(json!({ "ok": true })))
                    as Box<dyn codex_tools::ToolOutput>,
            )
        }
    }

    #[test]
    fn function_extensions_reject_custom_payloads() {
        let handler = ExtensionToolAdapter::new(Arc::new(StubExtensionExecutor));

        assert!(handler.matches_kind(&ToolPayload::Function {
            arguments: "{}".to_string(),
        }));
        assert!(!handler.matches_kind(&ToolPayload::Custom {
            input: "raw input".to_string(),
        }));
    }

    #[tokio::test]
    async fn exposes_generic_hook_payloads() {
        let handler = ExtensionToolAdapter::new(Arc::new(StubExtensionExecutor));
        let (session, turn) = crate::session::tests::make_session_and_context().await;
        let turn = Arc::new(turn);
        let invocation = ToolInvocation {
            session: session.into(),
            step_context: StepContext::for_test(Arc::clone(&turn)),
            turn,
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "call-extension".to_string(),
            tool_name: codex_tools::ToolName::plain("extension_echo"),
            source: ToolCallSource::Direct,
            native_call: None,
            environments: Default::default(),
            payload: ToolPayload::Function {
                arguments: json!({ "message": "hello" }).to_string(),
            },
        };
        let output = codex_tools::JsonToolOutput::new(json!({ "ok": true }));

        assert_eq!(
            CoreToolRuntime::pre_tool_use_payload(&handler, &invocation),
            Some(PreToolUsePayload {
                tool_name: HookToolName::new("extension_echo"),
                tool_input: json!({ "message": "hello" }),
            })
        );
        assert_eq!(
            CoreToolRuntime::post_tool_use_payload(&handler, &invocation, &output),
            Some(PostToolUsePayload {
                tool_cwd: None,
                tool_name: HookToolName::new("extension_echo"),
                tool_use_id: "call-extension".to_string(),
                tool_input: json!({ "message": "hello" }),
                tool_response: json!({ "ok": true }),
            })
        );
    }

    #[tokio::test]
    async fn passes_step_node_repl_flag_and_scoped_turn_item_emitter_to_extension_call() {
        let captured_call = Arc::new(Mutex::new(None));
        let captured_sandbox_cwds = Arc::new(Mutex::new(Vec::new()));
        let handler = ExtensionToolAdapter::new(Arc::new(CapturingExtensionExecutor {
            captured_call: Arc::clone(&captured_call),
            captured_sandbox_cwds: Arc::clone(&captured_sandbox_cwds),
        }));
        let (session, turn, rx) = crate::session::tests::make_session_and_context_with_rx().await;
        let weak_session = Arc::downgrade(&session);
        let weak_turn = Arc::downgrade(&turn);
        let turn_id = turn.sub_id.clone();
        let model = turn.model_info().slug.clone();
        let truncation_policy = turn.model_info().truncation_policy.into();
        let expected_sandbox_cwds = turn
            .initial_environments
            .turn_environments()
            .map(|environment| environment.cwd().clone())
            .collect::<Vec<_>>();
        let history_item = ResponseItem::Message {
            status: None,
            encrypted_content: None,
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "extension history".to_string(),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        };
        session
            .record_conversation_items(
                &turn,
                turn.model_info(),
                std::slice::from_ref(&history_item),
            )
            .await;
        let expected_history_item = strip_response_item_id(
            session
                .clone_history()
                .await
                .raw_items()
                .next()
                .expect("history item")
                .clone(),
        );
        let raw_history_event = rx.recv().await.expect("history raw response item event");
        let EventMsg::RawResponseItem(raw_history_item) = raw_history_event.msg else {
            panic!("expected raw response item event");
        };
        assert_eq!(
            strip_response_item_id(raw_history_item.item),
            expected_history_item
        );
        let mut step_context = StepContext::for_test(Arc::clone(&turn));
        let settings = Arc::make_mut(
            &mut Arc::get_mut(&mut step_context)
                .expect("unshared step")
                .settings,
        );
        let model_info = Arc::make_mut(&mut settings.model_info);
        model_info.node_repl_disabled = !turn.model_info().node_repl_disabled;
        let node_repl_disabled = model_info.node_repl_disabled;
        let invocation = ToolInvocation {
            session,
            step_context,
            turn,
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "call-extension".to_string(),
            tool_name: codex_tools::ToolName::plain("extension_echo"),
            source: ToolCallSource::CodeMode {
                cell_id: "cell-1".to_string(),
                runtime_tool_call_id: "nested-call-1".to_string(),
            },
            native_call: None,
            environments: Default::default(),
            payload: ToolPayload::Function {
                arguments: json!({ "message": "hello" }).to_string(),
            },
        };

        crate::tools::registry::ToolExecutor::handle(&handler, invocation)
            .await
            .expect("extension call should succeed");

        let captured_call = captured_call.lock().await.clone().expect("captured call");
        assert!(weak_session.upgrade().is_none());
        assert!(weak_turn.upgrade().is_none());
        assert_eq!(captured_call.turn_id, turn_id);
        assert_eq!(captured_call.call_id, "call-extension");
        assert_eq!(
            captured_call.tool_name,
            codex_tools::ToolName::plain("extension_echo")
        );
        assert_eq!(captured_call.model, model);
        assert_eq!(captured_call.truncation_policy, truncation_policy);
        let metadata: serde_json::Value = serde_json::from_str(
            captured_call
                .codex_turn_metadata
                .as_deref()
                .expect("turn metadata"),
        )
        .expect("metadata JSON");
        assert_eq!(metadata["node_repl_disabled"], json!(node_repl_disabled));
        assert_eq!(
            captured_call.source,
            ExtensionToolCallSource::CodeMode {
                cell_id: "cell-1".to_string(),
                runtime_tool_call_id: "nested-call-1".to_string(),
            }
        );
        assert_eq!(*captured_sandbox_cwds.lock().await, expected_sandbox_cwds);
        assert_eq!(
            strip_response_item_ids(captured_call.conversation_history.items()),
            vec![expected_history_item]
        );
        match captured_call.payload {
            ToolPayload::Function { arguments } => {
                assert_eq!(arguments, json!({ "message": "hello" }).to_string());
            }
            payload => panic!("expected function payload, got {payload:?}"),
        }

        let started = rx.recv().await.expect("item started event");
        let EventMsg::ItemStarted(started) = started.msg else {
            panic!("expected item started event");
        };
        let TurnItem::Extension(ExtensionItem::WebSearch(started_item)) = started.item else {
            panic!("expected extension web search item");
        };
        assert_eq!(
            started_item,
            WebSearchItem {
                id: "call-extension".to_string(),
                query: String::new(),
                action: None,
                results: None,
            }
        );
    }

    #[tokio::test]
    async fn image_generation_publication_preserves_extension_saved_path() {
        let (session, turn, rx) = crate::session::tests::make_session_and_context_with_rx().await;
        let expected_path = test_path_buf("/tmp/extension-claimed.png").abs();
        let emitter = CoreTurnItemEmitter {
            session: Arc::downgrade(&session),
            turn: Arc::downgrade(&turn),
        };
        let expected_started_item = ExtensionItem::ImageGeneration(ImageGenerationItem {
            id: "call-image".to_string(),
            status: "in_progress".to_string(),
            revised_prompt: None,
            image: GeneratedImageRef::Inline {
                result: String::new(),
            },
            transparent_background: None,
            failure: None,
            saved_path: None,
            input_images: Vec::new(),
            imagegen_request_id: None,
            generation_id: None,
        });
        let expected_completed_item = ExtensionItem::ImageGeneration(ImageGenerationItem {
            id: "call-image".to_string(),
            status: "completed".to_string(),
            revised_prompt: Some("A tiny blue square".to_string()),
            image: GeneratedImageRef::Inline {
                result: "cG5n".to_string(),
            },
            transparent_background: Some(true),
            failure: None,
            saved_path: Some(expected_path.clone()),
            input_images: Vec::new(),
            imagegen_request_id: None,
            generation_id: None,
        });
        codex_tools::TurnItemEmitter::emit_started(
            &emitter,
            ExtensionTurnItem {
                item: expected_started_item.clone(),
                legacy_events: vec![EventMsg::ImageGenerationBegin(ImageGenerationBeginEvent {
                    call_id: "call-image".to_string(),
                })],
            },
        )
        .await;
        codex_tools::TurnItemEmitter::emit_completed(
            &emitter,
            ExtensionTurnItem {
                item: expected_completed_item.clone(),
                legacy_events: vec![EventMsg::ImageGenerationEnd(ImageGenerationEndEvent {
                    call_id: "call-image".to_string(),
                    status: "completed".to_string(),
                    revised_prompt: Some("A tiny blue square".to_string()),
                    image: GeneratedImageRef::Inline {
                        result: "cG5n".to_string(),
                    },
                    transparent_background: Some(true),
                    failure: None,
                    saved_path: Some(expected_path.clone()),
                    input_images: Vec::new(),
                    generation_id: None,
                })],
            },
        )
        .await;

        let started = rx.recv().await.expect("item started event");
        let EventMsg::ItemStarted(started) = started.msg else {
            panic!("expected item started event");
        };
        let TurnItem::Extension(started_item) = started.item else {
            panic!("expected extension item");
        };
        let begin = rx.recv().await.expect("legacy image start event");
        assert!(matches!(begin.msg, EventMsg::ImageGenerationBegin(_)));
        let completed = rx.recv().await.expect("item completed event");
        let EventMsg::ItemCompleted(completed) = completed.msg else {
            panic!("expected item completed event");
        };
        let TurnItem::Extension(completed_item) = completed.item else {
            panic!("expected extension item");
        };
        let end = rx.recv().await.expect("legacy image end event");
        assert!(matches!(end.msg, EventMsg::ImageGenerationEnd(_)));

        assert_eq!(started_item, expected_started_item);
        assert_eq!(completed_item, expected_completed_item);
    }

    fn initialize_policy(
        session: &crate::session::session::Session,
    ) -> crate::security::TrustedSecurityController {
        use codex_security_policy::PolicyPrincipal;
        use codex_security_policy::PrincipalKind;
        use codex_security_policy::RevocationState;
        use codex_security_policy::SecurityLevel;
        use codex_security_policy::SecuritySettings;
        crate::security::TrustedSecurityController::initialize(
            &session.services.agent_control.effective_security_policy(),
            crate::security::PersistedHumanSecurityState::new(
                SecuritySettings::new(SecurityLevel::Permissive),
                PolicyPrincipal::new(PrincipalKind::Human, "human:issue-1057").unwrap(),
                RevocationState::new(),
            )
            .unwrap(),
            session.thread_id,
            codex_protocol::SessionId::from(session.thread_id),
            crate::security::EffectivePolicyInitialization::Root,
        )
        .expect("initialize the session's policy tree")
    }

    /// Commits a level change (the policy marker moves; the model client's
    /// live level stays Permissive).
    fn commit_transition(
        controller: &crate::security::TrustedSecurityController,
        level: codex_security_policy::SecurityLevel,
    ) {
        let change = controller
            .confirm_level_change(level, codex_security_policy::RevocationState::new())
            .unwrap();
        controller.apply_confirmed_change(change).unwrap();
    }

    fn file_contexts(
        call: &codex_tools::ToolCall,
    ) -> Vec<codex_file_system::FileSystemSandboxContext> {
        call.environments
            .iter()
            .map(|environment| environment.file_system_sandbox_context.clone())
            .collect()
    }

    /// #1057 (D71): a session with a read grant outside the workspace, an
    /// extension invocation, and the file contexts built with the grant.
    async fn granted_invocation() -> (
        crate::security::TrustedSecurityController,
        ToolInvocation,
        Vec<codex_file_system::FileSystemSandboxContext>,
        tempfile::TempDir,
    ) {
        let (session, turn) = crate::session::tests::make_session_and_context().await;
        let session = Arc::new(session);
        let turn = Arc::new(turn);
        let controller = initialize_policy(&session);
        let outside = tempfile::TempDir::new().expect("tempdir");
        let invocation = ToolInvocation {
            session: Arc::clone(&session),
            step_context: StepContext::for_test(Arc::clone(&turn)),
            turn,
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "call-1057".to_string(),
            tool_name: codex_tools::ToolName::plain("extension_echo"),
            source: ToolCallSource::Direct,
            payload: ToolPayload::Function {
                arguments: "{}".to_string(),
            },
        };
        let ungranted = file_contexts(&super::to_extension_call(&invocation).await);
        session
            .record_granted_request_permissions_for_turn(
                &codex_protocol::request_permissions::RequestPermissionsResponse {
                    permissions: codex_protocol::request_permissions::RequestPermissionProfile {
                        file_system: Some(
                            codex_protocol::models::FileSystemPermissions::from_read_write_roots(
                                Some(vec![outside.path().abs()]),
                                /*write*/ None,
                            ),
                        ),
                        ..Default::default()
                    },
                    scope: codex_protocol::request_permissions::PermissionGrantScope::Session,
                    strict_auto_review: false,
                },
                codex_exec_server::LOCAL_ENVIRONMENT_ID,
                /*originating_turn_state*/ None,
                session.services.authority_marker(),
            )
            .await;
        let granted = file_contexts(&super::to_extension_call(&invocation).await);
        assert!(!granted.is_empty(), "the test turn has an environment");
        assert_ne!(granted, ungranted, "the grant widens the file context");
        (controller, invocation, granted, outside)
    }

    /// #1057 (D71): with no transition, the call is built once and carries
    /// the grant.
    #[tokio::test]
    async fn issue_1057_extension_call_carries_grants_without_a_transition() {
        let (_controller, invocation, granted, _outside) = granted_invocation().await;
        let mut builds = 0;
        let call = super::current_extension_call(&invocation, &mut || builds += 1)
            .await
            .expect("no transition: the call starts");
        assert_eq!(builds, 1);
        assert_eq!(file_contexts(&call), granted);
    }

    /// #1057 (D71): a transition between building the call (its grants
    /// read) and the extension starting: the call is rebuilt under the new
    /// policy, so the ended grant never reaches the extension.
    #[tokio::test]
    async fn issue_1057_extension_call_built_before_a_transition_is_rebuilt() {
        let (controller, invocation, granted, _outside) = granted_invocation().await;
        let mut builds = 0;
        let call = super::current_extension_call(&invocation, &mut || {
            builds += 1;
            if builds == 1 {
                commit_transition(&controller, codex_security_policy::SecurityLevel::Moderate);
            }
        })
        .await
        .expect("rebuilt under the new policy");
        let current = file_contexts(&super::to_extension_call(&invocation).await);
        assert_ne!(current, granted, "the transition ended the grant");
        assert_eq!(
            file_contexts(&call),
            current,
            "no stale grant reaches the extension"
        );
        assert_eq!(builds, 2, "rebuilt once");
    }

    /// #1057 (D71): when a transition lands during the rebuild too, the
    /// call is refused and the extension never starts.
    #[tokio::test]
    async fn issue_1057_extension_call_refused_when_transitions_keep_landing() {
        let (controller, invocation, _granted, _outside) = granted_invocation().await;
        let mut builds = 0;
        let result = super::current_extension_call(&invocation, &mut || {
            builds += 1;
            let level = if builds % 2 == 1 {
                codex_security_policy::SecurityLevel::Moderate
            } else {
                codex_security_policy::SecurityLevel::Permissive
            };
            commit_transition(&controller, level);
        })
        .await;
        assert_eq!(builds, 2);
        assert_eq!(
            result.err(),
            Some(codex_tools::FunctionCallError::RespondToModel(
                super::EXTENSION_PREPARED_REFUSAL.to_string()
            ))
        );
    }
}
