use std::sync::Arc;
use std::sync::Weak;

use codex_protocol::items::TurnItem;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_tools::ConversationHistory;
use codex_tools::ExtensionTurnItem;
use codex_tools::ToolCall as ExtensionToolCall;
use codex_tools::ToolEnvironment;
use codex_tools::ToolName;
use codex_tools::ToolSearchInfo;
use codex_tools::ToolSpec;
use codex_tools::TurnItemEmissionFuture;
use codex_tools::TurnItemEmitter;
use codex_utils_string::to_ascii_json_string;

use crate::sandboxing::SandboxPermissions;
use crate::session::session::Session;
use crate::session::turn_context::TurnContext;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::apply_granted_turn_permissions;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use crate::turn_metadata::McpTurnMetadataContext;

pub(crate) struct ExtensionToolAdapter(Arc<dyn codex_tools::ToolExecutor<ExtensionToolCall>>);

impl ExtensionToolAdapter {
    pub(crate) fn new(executor: Arc<dyn codex_tools::ToolExecutor<ExtensionToolCall>>) -> Self {
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

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(async move {
            let call = current_extension_call(&invocation, &mut || {}).await?;
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
fn extension_call_basis(
    session: &Session,
) -> (Option<(u64, u64)>, codex_security_policy::SecurityLevel) {
    (
        session.services.authority_marker(),
        session.services.model_client().live_level(),
    )
}

/// #1057 (D71, option A): the extension call built under the policy in
/// force when the extension starts. The call's file contexts (grants
/// included) are built from the policy at build time; when a security
/// transition lands while it is built, it is rebuilt once under the new
/// policy, and refused when another lands meanwhile. `after_build` runs
/// after each build (a test seam). A transition while the extension runs is
/// not covered (option B).
async fn current_extension_call(
    invocation: &ToolInvocation,
    after_build: &mut (dyn FnMut() + Send),
) -> Result<ExtensionToolCall, codex_tools::FunctionCallError> {
    for _ in 0..2 {
        let built_under = extension_call_basis(invocation.session.as_ref());
        let call = to_extension_call(invocation).await;
        after_build();
        if extension_call_basis(invocation.session.as_ref()) == built_under {
            return Ok(call);
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

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
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

async fn to_extension_call(invocation: &ToolInvocation) -> ExtensionToolCall {
    let conversation_history =
        ConversationHistory::new(invocation.session.clone_history().await.into_raw_items());
    let codex_turn_metadata = invocation
        .turn
        .turn_metadata_state
        .current_meta_value_for_mcp_request(McpTurnMetadataContext {
            model: invocation.turn.model_info.slug.as_str(),
            reasoning_effort: invocation.turn.effective_reasoning_effort(),
        })
        .and_then(|metadata| to_ascii_json_string(&metadata).ok());
    let mut environments = Vec::new();
    for environment in invocation.step_context.environments.turn_environments() {
        // TODO(anp): Migrate extension ToolEnvironment and granted-permission lookup to PathUri
        // so extensions can receive foreign environment cwd values.
        let Ok(native_cwd) = environment.cwd().to_abs_path() else {
            continue;
        };
        let additional_permissions = apply_granted_turn_permissions(
            invocation.session.as_ref(),
            &environment.environment_id,
            native_cwd.as_path(),
            SandboxPermissions::UseDefault,
            /*additional_permissions*/ None,
        )
        .await
        .additional_permissions;
        let file_system_sandbox_context =
            crate::security::protected_surface::protect_file_tool_context(
                &invocation.session,
                &invocation.turn,
                invocation
                    .turn
                    .file_system_sandbox_context(additional_permissions, environment),
            );
        environments.push(ToolEnvironment {
            environment_id: environment.environment_id.clone(),
            cwd: native_cwd,
            file_system: environment.environment.get_filesystem(),
            file_system_sandbox_context,
        });
    }
    ExtensionToolCall {
        turn_id: invocation.turn.sub_id.clone(),
        call_id: invocation.call_id.clone(),
        tool_name: invocation.tool_name.clone(),
        model: invocation.turn.model_info.slug.clone(),
        codex_turn_metadata,
        truncation_policy: invocation.turn.model_info.truncation_policy.into(),
        conversation_history,
        turn_item_emitter: Arc::new(CoreTurnItemEmitter {
            session: Arc::downgrade(&invocation.session),
            turn: Arc::downgrade(&invocation.turn),
        }),
        environments,
        payload: invocation.payload.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use codex_extension_items::ExtensionItem;
    use codex_extension_items::image_generation::ImageGenerationItem;
    use codex_extension_items::web_search::WebSearchItem;
    use codex_protocol::items::TurnItem;
    use codex_protocol::models::ContentItem;
    use codex_protocol::models::ResponseItem;
    use codex_protocol::protocol::EventMsg;
    use codex_protocol::protocol::ImageGenerationBeginEvent;
    use codex_protocol::protocol::ImageGenerationEndEvent;
    use codex_tools::ExtensionTurnItem;
    use codex_utils_absolute_path::test_support::PathExt;
    use codex_utils_absolute_path::test_support::test_path_buf;
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

    impl codex_extension_api::ToolExecutor<codex_tools::ToolCall> for StubExtensionExecutor {
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

        fn handle(&self, _call: codex_tools::ToolCall) -> codex_tools::ToolExecutorFuture<'_> {
            Box::pin(async {
                Ok(
                    Box::new(codex_tools::JsonToolOutput::new(json!({ "ok": true })))
                        as Box<dyn codex_tools::ToolOutput>,
                )
            })
        }
    }

    struct CapturingExtensionExecutor {
        captured_call: Arc<Mutex<Option<codex_tools::ToolCall>>>,
    }

    impl codex_extension_api::ToolExecutor<codex_tools::ToolCall> for CapturingExtensionExecutor {
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

        fn handle(&self, call: codex_tools::ToolCall) -> codex_tools::ToolExecutorFuture<'_> {
            Box::pin(self.handle_call(call))
        }
    }

    impl CapturingExtensionExecutor {
        async fn handle_call(
            &self,
            call: codex_tools::ToolCall,
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
            *self.captured_call.lock().await = Some(call);
            Ok(
                Box::new(codex_tools::JsonToolOutput::new(json!({ "ok": true })))
                    as Box<dyn codex_tools::ToolOutput>,
            )
        }
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
                tool_name: HookToolName::new("extension_echo"),
                tool_use_id: "call-extension".to_string(),
                tool_input: json!({ "message": "hello" }),
                tool_response: json!({ "ok": true }),
            })
        );
    }

    #[tokio::test]
    async fn passes_turn_fields_and_scoped_turn_item_emitter_to_extension_call() {
        let captured_call = Arc::new(Mutex::new(None));
        let handler = ExtensionToolAdapter::new(Arc::new(CapturingExtensionExecutor {
            captured_call: Arc::clone(&captured_call),
        }));
        let (session, turn, rx) = crate::session::tests::make_session_and_context_with_rx().await;
        let weak_session = Arc::downgrade(&session);
        let weak_turn = Arc::downgrade(&turn);
        let turn_id = turn.sub_id.clone();
        let model = turn.model_info.slug.clone();
        let truncation_policy = turn.model_info.truncation_policy.into();
        let expected_sandbox_cwds = turn
            .environments
            .turn_environments()
            .map(|environment| Some(environment.cwd().clone()))
            .collect::<Vec<_>>();
        let history_item = ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "extension history".to_string(),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        };
        session
            .record_conversation_items(&turn, std::slice::from_ref(&history_item))
            .await;
        let mut expected_history_item = history_item.clone();
        expected_history_item.set_turn_id_if_missing(&turn_id);
        let raw_history_event = rx.recv().await.expect("history raw response item event");
        let EventMsg::RawResponseItem(raw_history_item) = raw_history_event.msg else {
            panic!("expected raw response item event");
        };
        assert_eq!(
            strip_response_item_id(raw_history_item.item),
            expected_history_item
        );
        let step_context = StepContext::for_test(Arc::clone(&turn));
        let invocation = ToolInvocation {
            session,
            step_context,
            turn,
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            tracker: Arc::new(tokio::sync::Mutex::new(TurnDiffTracker::new())),
            call_id: "call-extension".to_string(),
            tool_name: codex_tools::ToolName::plain("extension_echo"),
            source: ToolCallSource::Direct,
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
        assert_eq!(
            captured_call
                .environments
                .iter()
                .map(|environment| environment.file_system_sandbox_context.cwd.clone())
                .collect::<Vec<_>>(),
            expected_sandbox_cwds
        );
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
            result: String::new(),
            saved_path: None,
        });
        let expected_completed_item = ExtensionItem::ImageGeneration(ImageGenerationItem {
            id: "call-image".to_string(),
            status: "completed".to_string(),
            revised_prompt: Some("A tiny blue square".to_string()),
            result: "cG5n".to_string(),
            saved_path: Some(expected_path.clone()),
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
                    result: "cG5n".to_string(),
                    saved_path: Some(expected_path.clone()),
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
