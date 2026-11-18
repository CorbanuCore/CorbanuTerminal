//! Bounded discovery of the same authorized catalog used by spawn_agent.
use super::multi_agents_common::function_arguments;
use super::multi_agents_common::tool_output_code_mode_result;
use super::multi_agents_common::tool_output_json_text;
use super::multi_agents_common::tool_output_response_item;
use super::multi_agents_spec::format_model_billing;
use super::parse_arguments;
use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use crate::tools::spec_plan::spawn_agent_available_models;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ModelCapabilityTier;
use codex_protocol::openai_models::ReasoningEffort;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSearchInfo;
use codex_tools::ToolSpec;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

const PAGE_SIZE: usize = 5;
// Leave room for the result envelope; even operator-supplied catalog strings cannot
// produce an unbounded context item. Pagination never silently drops an entry.
const MAX_PAGE_BYTES: usize = 3_200;

pub(crate) struct Handler {
    pub supports_cross_provider_overrides: bool,
    pub expose_service_tiers: bool,
}

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("list_models")
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::Function(ResponsesApiTool {
            name: "list_models".to_string(),
            description: "List authorized picker-visible models available to spawn_agent, with exact provider/model names, display names, catalog context windows (operator overrides may differ), reasoning efforts and allocation hints. Returns a bounded page; call again with next_offset until null. Restart without offset if the catalog changes. Oversized metadata is shortened or reported by offset so later entries remain reachable. Hidden/custom exact runtime selection remains supported by spawn_agent.".to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                BTreeMap::from([("offset".to_string(), JsonSchema::integer(Some(
                    "Offset from the previous result's next_offset; omit for the first page.".to_string()
                )))]),
                /*required*/ None,
                Some(false.into()),
            ),
            output_schema: None,
        })
    }

    fn search_info(&self) -> Option<ToolSearchInfo> {
        ToolSearchInfo::from_spec(
            "list_models models catalog provider spawn_agent reasoning context capacity"
                .to_string(),
            self.spec(),
            /*source_info*/ None,
        )
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            let args: Args = parse_arguments(&function_arguments(invocation.payload)?)?;
            let turn = invocation.turn;
            let catalog =
                spawn_agent_available_models(&turn, self.supports_cross_provider_overrides)
                    .into_iter()
                    .filter(|model| model.show_in_picker)
                    .filter_map(|model| model.provider_id.clone().map(|provider| (model, provider)))
                    .collect::<Vec<_>>();
            let offset = args.offset.unwrap_or_default();
            if offset > catalog.len() {
                return Err(FunctionCallError::RespondToModel(
                    "list_models offset is past the catalog; omit offset to restart.".to_string(),
                ));
            }
            let mut models = Vec::new();
            let mut bytes = 0;
            let mut oversized_entries = Vec::new();
            let mut end = offset;
            for (model, provider) in catalog.iter().skip(offset).take(PAGE_SIZE) {
                let mut config = turn.config.as_ref().clone();
                config.model = Some(model.model.clone());
                config.model_provider_id = provider.clone();
                config.model_provider = config.model_providers[provider].clone();
                // Report catalog capacity, not a context override for the parent model.
                config.model_context_window = None;
                let info = crate::model_info_for_provider::model_info_for_config(
                    &invocation.session.services.models_manager,
                    &model.model,
                    &config,
                )
                .await;
                let entry = Model {
                    model_provider: provider.clone(),
                    model: model.model.clone(),
                    display_name: model.display_name.chars().take(128).collect(),
                    display_name_truncated: model.display_name.chars().count() > 128,
                    context_window: (!crate::model_info_for_provider::is_synthetic(&info))
                        .then(|| info.resolved_context_window())
                        .flatten(),
                    reasoning_efforts: model
                        .supported_reasoning_efforts
                        .iter()
                        .map(|preset| preset.effort.clone())
                        .collect(),
                    default_reasoning_effort: model.default_reasoning_effort.clone(),
                    input_modalities: model.input_modalities.clone(),
                    capability: model
                        .orchestration
                        .as_ref()
                        .map(codex_protocol::openai_models::ModelOrchestrationMetadata::capability),
                    billing: model
                        .orchestration
                        .as_ref()
                        .and_then(|metadata| metadata.billing())
                        .map(format_model_billing),
                    explicit_choice_only: model
                        .orchestration
                        .as_ref()
                        .and_then(|metadata| metadata.billing())
                        .is_none(),
                    service_tiers: if self.expose_service_tiers {
                        model
                            .service_tiers
                            .iter()
                            .map(|tier| tier.id.clone())
                            .collect()
                    } else {
                        Vec::new()
                    },
                };
                let entry_bytes = tool_output_json_text(&entry, "list_models").len();
                if entry_bytes > MAX_PAGE_BYTES {
                    oversized_entries.push(end);
                    end += 1;
                    continue;
                }
                if bytes + entry_bytes > MAX_PAGE_BYTES {
                    break;
                }
                bytes += entry_bytes;
                models.push(entry);
                end += 1;
            }
            Ok(boxed_tool_output(ResultPage {
                models,
                oversized_entries,
                total: catalog.len(),
                next_offset: (end < catalog.len()).then_some(end),
            }))
        })
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    offset: Option<usize>,
}

#[derive(Serialize)]
struct Model {
    model_provider: String,
    model: String,
    display_name: String,
    display_name_truncated: bool,
    context_window: Option<i64>,
    reasoning_efforts: Vec<ReasoningEffort>,
    default_reasoning_effort: ReasoningEffort,
    input_modalities: Vec<InputModality>,
    capability: Option<ModelCapabilityTier>,
    billing: Option<String>,
    explicit_choice_only: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    service_tiers: Vec<String>,
}

#[derive(Serialize)]
struct ResultPage {
    models: Vec<Model>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    oversized_entries: Vec<usize>,
    total: usize,
    next_offset: Option<usize>,
}

impl ToolOutput for ResultPage {
    fn log_output(&self) -> String {
        tool_output_json_text(self, "list_models")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "list_models")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> Value {
        tool_output_code_mode_result(self, "list_models")
    }
}
