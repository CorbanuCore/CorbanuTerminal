use crate::function_tool::FunctionCallError;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
#[cfg(test)]
use crate::session::turn_context::TurnContext;
use crate::tools::context::SharedTurnDiffTracker;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::flat_tool_names::FlatToolNames;
#[cfg(test)]
use crate::tools::handlers::ToolSearchHandlerCache;
use crate::tools::handlers::parse_arguments_for_tool;
use crate::tools::handlers::validate_structured_write_arguments;
use crate::tools::registry::AnyToolResult;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolArgumentDiffConsumer;
use crate::tools::registry::ToolRegistry;
#[cfg(test)]
use crate::tools::spec_plan::finalize_tool_router;
use codex_protocol::models::ResponseItem;
use codex_protocol::models::SearchToolCallParams;
use codex_tools::DiscoverableTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

pub use crate::tools::context::ToolCallSource;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub tool_name: ToolName,
    pub call_id: String,
    pub payload: ToolPayload,
    pub encrypted_function_args: Option<Vec<String>>,
}

impl ToolCall {
    pub(crate) fn direct_source(&self) -> ToolCallSource {
        let is_unencrypted_reserved_collaboration_call = self.tool_name.namespace.as_deref()
            == Some("collaboration")
            && matches!(
                self.tool_name.name.as_str(),
                "spawn_agent" | "send_message" | "followup_task"
            )
            && self
                .encrypted_function_args
                .as_ref()
                .is_some_and(Vec::is_empty);
        let is_cross_provider_plaintext_adapter = self.tool_name.namespace.is_none()
            && matches!(
                self.tool_name.name.as_str(),
                "spawn_agent_plaintext" | "send_message_plaintext" | "followup_task_plaintext"
            );
        if is_unencrypted_reserved_collaboration_call || is_cross_provider_plaintext_adapter {
            ToolCallSource::DirectPlaintextMessage
        } else {
            ToolCallSource::Direct
        }
    }
}

pub(crate) fn tool_log_payload<'a>(
    payload: &'a ToolPayload,
    source: &ToolCallSource,
) -> Cow<'a, str> {
    if matches!(source, ToolCallSource::DirectPlaintextMessage) {
        return Cow::Borrowed("[plaintext arguments]");
    }
    payload.log_payload()
}

pub struct ToolRouter {
    registry: ToolRegistry,
    model_visible_specs: Vec<ToolSpec>,
    /// Flat names offered for namespaced tools on wires without namespaces.
    flat_tool_names: FlatToolNames,
    /// Tools left out because a wire without namespaces caps the tool count.
    omitted_tools: Vec<ToolName>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolSuggestPresentation {
    ListTool,
    RecommendationContext,
}

#[derive(Clone, Debug)]
pub(crate) struct ToolSuggestCandidates {
    pub(crate) tools: Vec<DiscoverableTool>,
    pub(crate) presentation: ToolSuggestPresentation,
}

impl ToolRouter {
    #[cfg(test)]
    pub(crate) fn from_tools(
        turn_context: &TurnContext,
        tool_runtimes: Vec<Arc<dyn CoreToolRuntime>>,
        hosted_specs: Vec<ToolSpec>,
        tool_search_handler_cache: &ToolSearchHandlerCache,
    ) -> Self {
        finalize_tool_router(
            turn_context,
            ToolRegistry::from_tools(tool_runtimes),
            hosted_specs,
            tool_search_handler_cache,
        )
    }

    pub(crate) fn from_parts(registry: ToolRegistry, model_visible_specs: Vec<ToolSpec>) -> Self {
        Self {
            registry,
            model_visible_specs,
            flat_tool_names: FlatToolNames::default(),
            omitted_tools: Vec::new(),
        }
    }

    pub(crate) fn with_flat_tool_names(mut self, flat_tool_names: FlatToolNames) -> Self {
        self.flat_tool_names = flat_tool_names;
        self
    }

    pub(crate) fn with_omitted_tools(mut self, omitted_tools: Vec<ToolName>) -> Self {
        self.omitted_tools = omitted_tools;
        self
    }

    /// Flat names this router offers for namespaced tools (empty on wires
    /// with namespaces).
    pub(crate) fn flat_tool_names(&self) -> &FlatToolNames {
        &self.flat_tool_names
    }

    /// Tools not offered because of the tool-count limit, sorted by name.
    pub(crate) fn omitted_tools(&self) -> &[ToolName] {
        &self.omitted_tools
    }

    /// Maps a call to a flat tool name back to the namespaced tool it was
    /// offered for, so it runs through the same handler, approvals and gates
    /// as a namespaced call on the Responses wire. Registered plain tools always
    /// win, and calls that are already namespaced pass through unchanged.
    pub(crate) fn resolve_flat_tool_call(&self, mut call: ToolCall) -> ToolCall {
        if call.tool_name.namespace.is_none()
            && !self.flat_tool_names.is_empty()
            && self.registry.tool(&call.tool_name).is_none()
            && let Some(tool_name) = self.flat_tool_names.resolve(&call.tool_name.name)
        {
            call.tool_name = tool_name.clone();
        }
        call
    }

    pub(crate) fn model_visible_specs(&self) -> Vec<ToolSpec> {
        self.model_visible_specs.clone()
    }

    pub(crate) fn deferred_tool_namespaces(&self) -> BTreeMap<String, String> {
        self.registry.deferred_tool_namespaces()
    }

    #[cfg(test)]
    pub(crate) fn registered_tool_names_for_test(&self) -> Vec<ToolName> {
        self.registry.tool_names_for_test()
    }

    #[cfg(test)]
    pub(crate) fn tool_exposure_for_test(
        &self,
        name: &ToolName,
    ) -> Option<crate::tools::registry::ToolExposure> {
        self.registry.tool_exposure(name)
    }

    pub(crate) fn create_diff_consumer(
        &self,
        tool_name: &ToolName,
    ) -> Option<Box<dyn ToolArgumentDiffConsumer>> {
        self.registry.create_diff_consumer(tool_name)
    }

    pub fn tool_supports_parallel(&self, call: &ToolCall) -> bool {
        self.registry
            .supports_parallel_tool_calls(&call.tool_name)
            .unwrap_or(false)
    }

    pub(crate) fn tool_runtime(&self, call: &ToolCall) -> Option<Arc<dyn CoreToolRuntime>> {
        self.registry.tool(&call.tool_name)
    }

    pub fn tool_waits_for_runtime_cancellation(&self, call: &ToolCall) -> bool {
        self.registry
            .waits_for_runtime_cancellation(&call.tool_name)
            .unwrap_or(false)
    }

    #[instrument(level = "trace", skip_all, err)]
    pub fn build_tool_call(item: ResponseItem) -> Result<Option<ToolCall>, FunctionCallError> {
        match item {
            ResponseItem::FunctionCall {
                name,
                namespace,
                arguments,
                encrypted_function_args,
                call_id,
                ..
            } => {
                let tool_name = ToolName::new(namespace, name);
                let tool_name_text = tool_name.to_string();
                let _: serde_json::Value = parse_arguments_for_tool(&tool_name_text, &arguments)?;
                if tool_name.namespace.is_none() && tool_name.name == "structured_write" {
                    validate_structured_write_arguments(&arguments)?;
                }
                Ok(Some(ToolCall {
                    tool_name,
                    call_id,
                    payload: ToolPayload::Function { arguments },
                    encrypted_function_args,
                }))
            }
            ResponseItem::ToolSearchCall {
                call_id: Some(call_id),
                execution,
                arguments,
                ..
            } if execution == "client" => {
                let arguments: SearchToolCallParams =
                    serde_json::from_value(arguments).map_err(|err| {
                        FunctionCallError::RespondToModel(format!(
                            "failed to parse tool_search arguments: {err}"
                        ))
                    })?;
                Ok(Some(ToolCall {
                    tool_name: ToolName::plain("tool_search"),
                    call_id,
                    payload: ToolPayload::ToolSearch { arguments },
                    encrypted_function_args: None,
                }))
            }
            ResponseItem::ToolSearchCall { .. } => Ok(None),
            ResponseItem::CustomToolCall {
                name,
                namespace,
                input,
                call_id,
                ..
            } => Ok(Some(ToolCall {
                tool_name: ToolName::new(namespace, name),
                call_id,
                payload: ToolPayload::Custom { input },
                encrypted_function_args: None,
            })),
            _ => Ok(None),
        }
    }

    #[allow(dead_code)]
    #[instrument(level = "trace", skip_all, err)]
    pub async fn dispatch_tool_call_with_code_mode_result(
        &self,
        session: Arc<Session>,
        step_context: Arc<StepContext>,
        cancellation_token: CancellationToken,
        tracker: SharedTurnDiffTracker,
        call: ToolCall,
        source: ToolCallSource,
    ) -> Result<AnyToolResult, FunctionCallError> {
        self.dispatch_tool_call_with_code_mode_result_inner(
            session,
            step_context,
            cancellation_token,
            tracker,
            call,
            source,
            /*terminal_outcome_reached*/ None,
        )
        .await
    }

    #[instrument(level = "trace", skip_all, err)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn dispatch_tool_call_with_terminal_outcome(
        &self,
        session: Arc<Session>,
        step_context: Arc<StepContext>,
        cancellation_token: CancellationToken,
        tracker: SharedTurnDiffTracker,
        call: ToolCall,
        source: ToolCallSource,
        terminal_outcome_reached: Arc<AtomicBool>,
    ) -> Result<AnyToolResult, FunctionCallError> {
        self.dispatch_tool_call_with_code_mode_result_inner(
            session,
            step_context,
            cancellation_token,
            tracker,
            call,
            source,
            Some(terminal_outcome_reached),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn dispatch_tool_call_with_code_mode_result_inner(
        &self,
        session: Arc<Session>,
        step_context: Arc<StepContext>,
        cancellation_token: CancellationToken,
        tracker: SharedTurnDiffTracker,
        call: ToolCall,
        source: ToolCallSource,
        terminal_outcome_reached: Option<Arc<AtomicBool>>,
    ) -> Result<AnyToolResult, FunctionCallError> {
        // #507: a tool call streamed before the kill switch was turned on
        // does not run.
        crate::security::kill_switch_stop::refuse_tool_call(&session)?;
        let ToolCall {
            tool_name,
            call_id,
            payload,
            ..
        } = call;

        session
            .services
            .model_client()
            .register_native_tool_origin(&call_id, codex_protocol::provenance::SourceKind::Tool);

        // Keep the legacy ToolInvocation.turn field tied to the same request state until handlers migrate.
        let turn = Arc::clone(&step_context.turn);
        let invocation = ToolInvocation {
            session,
            turn,
            step_context,
            cancellation_token,
            tracker,
            call_id,
            tool_name,
            source,
            payload,
        };

        self.registry
            .dispatch_any_with_terminal_outcome(invocation, terminal_outcome_reached)
            .await
    }
}

#[cfg(test)]
#[path = "router_tests.rs"]
mod tests;
