pub use codex_api::ResponseEvent;
use codex_protocol::error::Result;
use codex_protocol::models::BaseInstructions;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::ResponseItem;
use codex_tools::ToolSpec;
use futures::Stream;
use serde_json::Value;
use std::collections::HashSet;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// API request payload for a single model turn
#[derive(Debug, Clone)]
pub struct Prompt {
    /// Conversation context input items.
    pub input: Vec<ResponseItem>,

    /// Tools available to the model, including additional tools sourced from
    /// external MCP servers.
    pub(crate) tools: Vec<ToolSpec>,

    /// Whether parallel tool calls are permitted for this prompt.
    pub(crate) parallel_tool_calls: bool,

    pub base_instructions: BaseInstructions,

    /// Optional the output schema for the model's response.
    pub output_schema: Option<Value>,

    /// Whether the Responses API should strictly validate `output_schema`.
    pub output_schema_strict: bool,
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            input: Vec::new(),
            tools: Vec::new(),
            parallel_tool_calls: false,
            base_instructions: BaseInstructions::default(),
            output_schema: None,
            output_schema_strict: true,
        }
    }
}

impl Prompt {
    pub(crate) fn get_formatted_input_for_request(
        &self,
        use_responses_lite: bool,
    ) -> Vec<ResponseItem> {
        let mut input = self.input.clone();
        if use_responses_lite {
            strip_image_details(&mut input);
        }
        input
    }
}

/// Keep only the newest version of each turn-scoped developer-context section.
///
/// OpenAI models understand append-only contextual updates and benefit from the stable prompt
/// prefix. Some compatible providers instead obey an older contradictory section. Their wire
/// adapters call this before serialization so persisted audit history remains intact while the
/// model receives one authoritative permissions/model/mode/environment section.
pub(crate) fn retain_latest_contextual_developer_fragments(items: &mut Vec<ResponseItem>) {
    let mut seen = HashSet::new();
    for item in items.iter_mut().rev() {
        let ResponseItem::Message { role, content, .. } = item else {
            continue;
        };
        if role != "developer" {
            continue;
        }

        let mut retained = Vec::with_capacity(content.len());
        for content_item in std::mem::take(content).into_iter().rev() {
            let keep = crate::event_mapping::contextual_dev_fragment_key(&content_item)
                .is_none_or(|key| seen.insert(key));
            if keep {
                retained.push(content_item);
            }
        }
        retained.reverse();
        *content = retained;
    }
    items.retain(|item| {
        !matches!(
            item,
            ResponseItem::Message { role, content, .. }
                if role == "developer" && content.is_empty()
        )
    });
}

/// Coalesce system messages into one leading Chat Completions message.
///
/// OpenAI's Chat Completions contract accepts system/developer context at any
/// position and in multiple messages, and some OpenAI-compatible gateways do
/// as well. Other compatible chat templates (for example Qwen3.8) accept only
/// a single system message at the beginning. Preserve audit history in
/// Responses form, but serialize the already-normalized wire messages in the
/// stricter shape.
pub(crate) fn coalesce_chat_system_messages(messages: &mut Vec<codex_api::ChatMessage>) {
    let original = std::mem::take(messages);
    let mut system_texts = Vec::new();
    let mut rest = Vec::with_capacity(original.len());
    for message in original {
        if message.role == "system" {
            match message.content {
                Some(codex_api::ChatMessageContent::Text(text)) => {
                    if !text.trim().is_empty() {
                        system_texts.push(text);
                    }
                }
                Some(other) => {
                    // Preserve non-text system payloads by placing a non-empty
                    // system message at the boundary rather than dropping data.
                    rest.push(codex_api::ChatMessage {
                        role: "system".to_string(),
                        content: Some(other),
                        reasoning_content: message.reasoning_content,
                        tool_call_id: message.tool_call_id,
                        tool_calls: message.tool_calls,
                    });
                }
                None => {}
            }
        } else {
            rest.push(message);
        }
    }

    if system_texts.is_empty() {
        *messages = rest;
        return;
    }
    let merged = codex_api::ChatMessage {
        role: "system".to_string(),
        content: Some(codex_api::ChatMessageContent::text(
            system_texts.join("\n\n"),
        )),
        reasoning_content: None,
        tool_call_id: None,
        tool_calls: Vec::new(),
    };
    messages.push(merged);
    messages.extend(rest);
}

fn strip_image_details(items: &mut [ResponseItem]) {
    for item in items {
        match item {
            ResponseItem::Message { content, .. } => {
                for content_item in content {
                    if let ContentItem::InputImage { detail, .. } = content_item {
                        *detail = None;
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
                            *detail = None;
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
            | ResponseItem::CompactionTrigger { .. }
            | ResponseItem::ContextCompaction { .. }
            | ResponseItem::Other => {}
        }
    }
}

pub struct ResponseStream {
    pub(crate) rx_event: mpsc::Receiver<Result<ResponseEvent>>,
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
