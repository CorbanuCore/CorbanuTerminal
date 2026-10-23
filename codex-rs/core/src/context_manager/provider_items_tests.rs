use super::*;
use codex_protocol::ResponseItemId;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ReasoningItemContent;
use pretty_assertions::assert_eq;
use serde_json::json;

fn id(value: &str) -> Option<ResponseItemId> {
    Some(ResponseItemId::from_server(value.to_string()))
}

fn from(producer: Option<&str>, mut item: ResponseItem) -> ResponseItem {
    item.set_turn_id_if_missing("turn-1");
    if let Some(producer) = producer {
        item.set_producer(producer);
    }
    item
}

/// Reasoning as each wire's adapter records it.
fn responses_reasoning(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::Reasoning {
            id: id("rs_openai"),
            summary: Vec::new(),
            content: None,
            encrypted_content: Some("sealed".into()),
            anthropic_content_block: None,
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn chat_reasoning(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::Reasoning {
            id: id("rs_2026101213421902b19754973e467e"),
            summary: Vec::new(),
            content: Some(vec![ReasoningItemContent::ReasoningText {
                text: "glm thinking".into(),
            }]),
            encrypted_content: None,
            anthropic_content_block: None,
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn anthropic_reasoning(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::Reasoning {
            id: id("rs_msg_01"),
            summary: Vec::new(),
            content: Some(vec![ReasoningItemContent::ReasoningText {
                text: "claude thinking".into(),
            }]),
            encrypted_content: None,
            anthropic_content_block: Some(
                json!({"type": "thinking", "thinking": "claude thinking", "signature": "sig"}),
            ),
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn anthropic_web_search(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::WebSearchCall {
            id: id("srvtoolu_1"),
            status: Some("completed".into()),
            action: None,
            anthropic_content_block: Some(json!({"type": "server_tool_use", "id": "srvtoolu_1"})),
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn assistant(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::Message {
            id: id("msg_chatcmpl-1"),
            role: "assistant".into(),
            content: vec![ContentItem::OutputText {
                text: "answer".into(),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn call(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::FunctionCall {
            id: id("fc_call_-1"),
            name: "shell".into(),
            namespace: None,
            arguments: "{}".into(),
            encrypted_function_args: Some(vec!["sealed".into()]),
            call_id: "call_-1".into(),
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn target(producer: &str, wire: WireApi, stores_items: bool) -> ProviderTarget {
    ProviderTarget {
        producer: producer.into(),
        wire,
        stores_items,
    }
}

/// Both steps: the prompt's drops, then (Responses) the client's ID removal.
fn adapted(items: Vec<ResponseItem>, target: &ProviderTarget) -> Vec<ResponseItem> {
    let mut items = items;
    adapt_for_provider(&mut items, target);
    if target.wire == WireApi::Responses {
        strip_foreign_references(&mut items, &target.producer);
    }
    items
}

fn without_id(mut item: ResponseItem) -> ResponseItem {
    item.set_id(/*new_id*/ None);
    item
}

#[test]
fn same_provider_items_are_unchanged_on_every_wire() {
    for (producer, wire, items) in [
        (
            "openai",
            WireApi::Responses,
            vec![responses_reasoning(Some("openai")), call(Some("openai"))],
        ),
        (
            "zai",
            WireApi::Chat,
            vec![chat_reasoning(Some("zai")), assistant(Some("zai"))],
        ),
        (
            "anthropic",
            WireApi::Anthropic,
            vec![
                anthropic_reasoning(Some("anthropic")),
                anthropic_web_search(Some("anthropic")),
            ],
        ),
    ] {
        let target = target(producer, wire, /*stores_items*/ false);
        assert_eq!(adapted(items.clone(), &target), items, "{producer}");
    }
}

/// The issue: GLM (Chat) reasoning reached OpenAI as `rs_…` references.
#[test]
fn chat_and_anthropic_items_reach_responses_inline_without_references() {
    let openai = target("openai", WireApi::Responses, /*stores_items*/ false);
    let items = vec![
        responses_reasoning(Some("openai")),
        chat_reasoning(Some("zai")),
        assistant(Some("zai")),
        call(Some("zai")),
        anthropic_reasoning(Some("anthropic")),
        anthropic_web_search(Some("anthropic")),
        assistant(Some("anthropic")),
    ];
    let mut expected_call = without_id(call(Some("zai")));
    if let ResponseItem::FunctionCall {
        encrypted_function_args,
        ..
    } = &mut expected_call
    {
        *encrypted_function_args = None;
    }
    assert_eq!(
        adapted(items, &openai),
        vec![
            responses_reasoning(Some("openai")),
            without_id(assistant(Some("zai"))),
            expected_call,
            without_id(assistant(Some("anthropic"))),
        ]
    );
}

/// Another Responses provider (or account): its encrypted reasoning is bound to it.
#[test]
fn another_responses_providers_reasoning_is_dropped() {
    let work = target(
        "openai:work",
        WireApi::Responses,
        /*stores_items*/ false,
    );
    assert_eq!(
        adapted(
            vec![
                responses_reasoning(Some("openai")),
                assistant(Some("openai"))
            ],
            &work
        ),
        vec![without_id(assistant(Some("openai")))]
    );
}

#[test]
fn another_anthropic_hosts_signed_blocks_are_dropped() {
    let anthropic = target("anthropic", WireApi::Anthropic, /*stores_items*/ false);
    let items = vec![
        anthropic_reasoning(Some("zai-anthropic")),
        anthropic_web_search(Some("zai-anthropic")),
        assistant(Some("zai-anthropic")),
        call(Some("openai")),
        responses_reasoning(Some("openai")),
    ];
    assert_eq!(
        adapted(items, &anthropic),
        vec![assistant(Some("zai-anthropic")), call(Some("openai"))]
    );
}

#[test]
fn chat_requests_keep_every_item() {
    let zai = target("zai", WireApi::Chat, /*stores_items*/ false);
    let items = vec![
        responses_reasoning(Some("openai")),
        anthropic_reasoning(Some("anthropic")),
        call(Some("openai")),
    ];
    assert_eq!(adapted(items.clone(), &zai), items);
}

/// Sessions recorded before #578: only what a stateless Responses host
/// could read as a reference, or Anthropic-native blocks, are dropped.
#[test]
fn unattributed_items_keep_only_what_the_target_can_read() {
    let items = vec![
        responses_reasoning(/*producer*/ None),
        chat_reasoning(/*producer*/ None),
        anthropic_reasoning(/*producer*/ None),
        anthropic_web_search(/*producer*/ None),
        assistant(/*producer*/ None),
        call(/*producer*/ None),
    ];
    assert_eq!(
        adapted(
            items.clone(),
            &target("openai", WireApi::Responses, /*stores_items*/ false)
        ),
        vec![
            responses_reasoning(/*producer*/ None),
            assistant(/*producer*/ None),
            call(/*producer*/ None)
        ]
    );
    assert_eq!(
        adapted(
            items.clone(),
            &target("azure", WireApi::Responses, /*stores_items*/ true)
        ),
        vec![
            responses_reasoning(/*producer*/ None),
            chat_reasoning(/*producer*/ None),
            assistant(/*producer*/ None),
            call(/*producer*/ None)
        ]
    );
    for wire in [WireApi::Chat, WireApi::Anthropic] {
        let target = target("other", wire, /*stores_items*/ false);
        assert_eq!(adapted(items.clone(), &target), items);
    }
}

fn compaction(producer: Option<&str>) -> ResponseItem {
    from(
        producer,
        ResponseItem::Compaction {
            id: id("cmp_1"),
            encrypted_content: "sealed summary".into(),
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

fn context_compaction(producer: Option<&str>, sealed: bool) -> ResponseItem {
    from(
        producer,
        ResponseItem::ContextCompaction {
            id: id("cmp_2"),
            encrypted_content: sealed.then(|| "sealed".to_string()),
            internal_chat_message_metadata_passthrough: None,
        },
    )
}

/// Remote compaction summaries are sealed for the provider (and account)
/// that made them, like reasoning.
#[test]
fn another_providers_compaction_is_dropped_on_responses() {
    let items = vec![
        compaction(Some("openai")),
        context_compaction(Some("openai"), /*sealed*/ true),
        assistant(Some("openai")),
    ];
    assert_eq!(
        adapted(
            items.clone(),
            &target("openai", WireApi::Responses, /*stores_items*/ false)
        ),
        items
    );
    assert_eq!(
        adapted(
            items,
            &target(
                "openai:work",
                WireApi::Responses,
                /*stores_items*/ false
            )
        ),
        vec![without_id(assistant(Some("openai")))]
    );
}

/// An unattributed compaction without encrypted content is a reference.
#[test]
fn an_unattributed_compaction_reference_is_dropped_when_nothing_is_stored() {
    let items = vec![
        compaction(/*producer*/ None),
        context_compaction(/*producer*/ None, /*sealed*/ true),
        context_compaction(/*producer*/ None, /*sealed*/ false),
    ];
    assert_eq!(
        adapted(
            items.clone(),
            &target("openai", WireApi::Responses, /*stores_items*/ false)
        ),
        items[..2].to_vec()
    );
    assert_eq!(
        adapted(
            items.clone(),
            &target("azure", WireApi::Responses, /*stores_items*/ true)
        ),
        items
    );
}

/// A named account is its own producer: its items are another provider's
/// for the default account (and for other named accounts).
#[test]
fn a_named_account_is_its_own_producer() {
    let account = |name: &str| NamedProviderAccount {
        provider_id: "openai".into(),
        name: name.into(),
    };
    let work = producer_key("openai", Some(&account("work")));
    let personal = producer_key("openai", Some(&account("personal")));
    let default = producer_key("openai", /*account*/ None);
    assert_eq!(default, "openai");
    assert_eq!(work, "openai:work");
    let items = vec![responses_reasoning(Some(&work))];
    let to = |producer: &str| target(producer, WireApi::Responses, /*stores_items*/ false);
    assert_eq!(adapted(items.clone(), &to(&work)), items);
    assert_eq!(adapted(items.clone(), &to(&personal)), Vec::new());
    assert_eq!(adapted(items, &to(&default)), Vec::new());
}

/// The local producer never reaches the model, so it is not counted.
#[test]
fn the_producer_is_not_counted_as_context() {
    let plain = from(/*producer*/ None, assistant(/*producer*/ None));
    let stamped = assistant(Some("a-provider-with-a-long-name"));
    assert_eq!(
        crate::context_manager::estimate_item_token_count(&stamped),
        crate::context_manager::estimate_item_token_count(&plain)
    );
}

/// Source labelling looks items up by their recorded bytes, so the prompt
/// step only drops items; another provider's kept items lose their IDs in
/// the client afterwards.
#[test]
fn kept_items_stay_as_recorded_until_the_client_strips_them() {
    let openai = target("openai", WireApi::Responses, /*stores_items*/ false);
    let mut items = vec![
        chat_reasoning(Some("zai")),
        assistant(Some("zai")),
        call(Some("zai")),
    ];
    adapt_for_provider(&mut items, &openai);
    assert_eq!(items, vec![assistant(Some("zai")), call(Some("zai"))]);
    strip_foreign_references(&mut items, "openai");
    assert!(items.iter().all(|item| item.id().is_none()));
}

/// The prompt step reports a dropped compaction summary, so the session can
/// say so once.
#[test]
fn a_dropped_compaction_summary_is_reported() {
    let items = vec![compaction(Some("openai")), assistant(Some("openai"))];
    let mut same = items.clone();
    assert!(!adapt_for_provider(
        &mut same,
        &target("openai", WireApi::Responses, /*stores_items*/ false)
    ));
    let mut other = items;
    assert!(adapt_for_provider(
        &mut other,
        &target(
            "openai:work",
            WireApi::Responses,
            /*stores_items*/ false
        )
    ));
}
