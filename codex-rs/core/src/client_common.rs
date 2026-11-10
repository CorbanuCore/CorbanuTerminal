pub use codex_api::ResponseEvent;
use codex_protocol::error::Result;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::ContentItem;
use codex_protocol::models::DEFAULT_IMAGE_DETAIL;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::ImageDetail;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ModelInfo;
use codex_tools::ToolSpec;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

/// API request payload for a single model turn
#[derive(Debug, Clone)]
pub struct Prompt {
    /// Conversation context input items.
    pub input: Vec<ResponseItem>,

    /// Tool definitions to inject into this request, empty when supplied by history.
    pub(crate) tools: Arc<[ToolSpec]>,

    /// Whether parallel tool calls are permitted for this prompt.
    pub(crate) parallel_tool_calls: bool,

    pub base_instructions: BaseInstructions,

    /// Optional the output schema for the model's response.
    pub output_schema: Option<Value>,

    /// Whether the Responses API should strictly validate `output_schema`.
    pub output_schema_strict: bool,

    /// #578: the producer of this request's provider (see
    /// `TurnContext::output_producer`). Another producer's items lose their
    /// IDs on a Responses request, after source labelling looked them up by
    /// their recorded bytes. `None` leaves every item as recorded.
    pub(crate) producer: Option<String>,

    /// #688: the flat names `tools` offers for namespaced tools on a wire
    /// without namespaces, so recorded namespaced calls replay under exactly
    /// the name offered for them. Empty on wires with namespaces.
    pub(crate) flat_tool_names: crate::tools::flat_tool_names::FlatToolNames,
    pub(crate) cyber_access_program: Option<codex_protocol::turn_input::CyberAccessProgram>,
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            input: Vec::new(),
            tools: Arc::default(),
            parallel_tool_calls: false,
            base_instructions: BaseInstructions::default(),
            output_schema: None,
            output_schema_strict: true,
            producer: None,
            flat_tool_names: Default::default(),
            cyber_access_program: None,
        }
    }
}

/// D508 (owner decision D521 pending; option (a) provisional): models main's
/// catalog marked `supports_parallel_tool_calls: false` before upstream dropped
/// that field (rust-v0.167.0). Requests for them keep parallel tool calls off.
const SERIAL_TOOL_CALL_MODELS: [&str; 17] = [
    "x-ai/grok-4.7",
    "deepseek/deepseek-v4.1-flash",
    "xiaomi/mimo-v2.6-pro",
    "nvidia/nemotron-3-ultra-550b-a55b:free",
    "vercel/moonshotai/kimi-k3",
    "vercel/deepseek/deepseek-v4-pro",
    "deepseek-v4-pro",
    "x-ai/grok-4.6",
    "x-ai/grok-4.5",
    "deepseek-flash",
    "deepseek-v4-flash",
    "deepseek/deepseek-v4-pro",
    "deepseek/deepseek-v4-pro-0813",
    "deepseek/deepseek-v4-flash-0731",
    "tencent/hy3:free",
    "moonshotai/kimi-k3",
    "corbanu/deepseek-v4.1-flash",
];

/// Whether a request for the model `slug` may ask for parallel tool calls.
pub(crate) fn model_allows_parallel_tool_calls(slug: &str) -> bool {
    !SERIAL_TOOL_CALL_MODELS.contains(&slug)
}

impl Prompt {
    pub(crate) fn get_formatted_input_for_request(
        &self,
        model_info: &ModelInfo,
    ) -> Vec<ResponseItem> {
        let mut input = self.input.clone();
        normalize_image_details(&mut input, model_info);
        input
    }
}

/// Keep only the newest version of each turn-scoped developer-context section.
///
/// OpenAI models understand append-only contextual updates and benefit from the stable prompt
/// prefix. Some compatible providers instead obey an older contradictory section. Their wire
/// adapters call this before serialization so persisted audit history remains intact while the
/// model receives one authoritative permissions/model/mode/environment section.
///
/// Sections are keyed by marker and recorded producer, so distinct producers sharing a marker
/// (such as the host, executor and thread skill catalogs) each keep their newest copy. An
/// unattributed section, including one persisted before producers were recorded, still replaces
/// every older section with its marker.
pub(crate) fn retain_latest_contextual_developer_fragments(items: &mut Vec<ResponseItem>) {
    let mut newer_sections = HashMap::<&'static str, NewerSections>::new();
    for item in items.iter_mut().rev() {
        let ResponseItem::Message { role, content, .. } = &*item else {
            continue;
        };
        if role != "developer" {
            continue;
        }

        let mut keep = vec![true; content.len()];
        for (index, content_item) in content.iter().enumerate().rev() {
            if let Some(marker) = crate::event_mapping::contextual_dev_fragment_key(content_item) {
                keep[index] = newer_sections
                    .entry(marker)
                    .or_default()
                    .admit(item.context_fragment_source(index));
            }
        }
        let mut keep = keep.into_iter();
        item.retain_message_content(|_| keep.next().unwrap_or(true));
    }
    items.retain(|item| {
        !matches!(
            item,
            ResponseItem::Message { role, content, .. }
                if role == "developer" && content.is_empty()
        )
    });
}

/// Newer copies of one contextual developer marker, seen while walking history backwards.
#[derive(Default)]
struct NewerSections {
    unattributed: bool,
    sources: HashSet<String>,
}

impl NewerSections {
    /// Returns whether an older section from `source` is still the newest copy of its source.
    fn admit(&mut self, source: Option<&str>) -> bool {
        if self.unattributed {
            return false;
        }
        match source {
            Some(source) => self.sources.insert(source.to_string()),
            None => {
                self.unattributed = true;
                self.sources.is_empty()
            }
        }
    }
}

pub(crate) fn normalize_image_details(items: &mut [ResponseItem], model_info: &ModelInfo) {
    for item in items {
        match item {
            ResponseItem::Message { content, .. } => {
                for content_item in content {
                    if let ContentItem::InputImage { detail, .. } = content_item {
                        normalize_image_detail(detail, model_info);
                    }
                }
            }
            ResponseItem::FunctionCallOutput { output, .. }
            | ResponseItem::CustomToolCallOutput { output, .. } => {
                if let Some(content) = output.content_items_mut() {
                    for content_item in content {
                        if let FunctionCallOutputContentItem::InputImage { detail, .. } =
                            content_item
                        {
                            normalize_image_detail(detail, model_info);
                        }
                    }
                }
            }
            ResponseItem::AdditionalTools { .. }
            | ResponseItem::Reasoning { .. }
            | ResponseItem::AgentMessage { .. }
            | ResponseItem::LocalShellCall { .. }
            | ResponseItem::FunctionCall { .. }
            | ResponseItem::ToolSearchCall { .. }
            | ResponseItem::CustomToolCall { .. }
            | ResponseItem::ToolSearchOutput { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::Compaction { .. }
            | ResponseItem::ConfigurationUpdate { .. }
            | ResponseItem::CompactionTrigger { .. }
            | ResponseItem::ContextCompaction { .. }
            | ResponseItem::Other => {}
        }
    }
}

fn normalize_image_detail(detail: &mut Option<ImageDetail>, model_info: &ModelInfo) {
    if model_info.use_responses_lite {
        *detail = None;
    } else if *detail == Some(ImageDetail::Original) && !model_info.supports_image_detail_original {
        *detail = Some(DEFAULT_IMAGE_DETAIL);
    }
}

pub struct ResponseStream {
    pub(crate) routing_affinity: Option<codex_http_client::ChatGptRoutingAffinity>,
    pub(crate) rx_event: mpsc::Receiver<Result<ResponseEvent>>,
    pub(crate) interrupt: Option<oneshot::Sender<codex_api::ResponseInterrupt>>,
    /// Signals the mapper task that the consumer stopped polling before the
    /// provider stream reached its own terminal event.
    pub(crate) consumer_dropped: CancellationToken,
}

impl Stream for ResponseStream {
    type Item = Result<ResponseEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx_event.poll_recv(cx)
    }
}

impl Drop for ResponseStream {
    fn drop(&mut self) {
        self.consumer_dropped.cancel();
    }
}

#[cfg(test)]
#[path = "client_common_tests.rs"]
mod tests;
