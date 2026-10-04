use pretty_assertions::assert_eq;

use super::*;

fn message(
    texts: &[&str],
    turn_id: Option<&str>,
    sources: Option<Vec<Option<&str>>>,
) -> ResponseItem {
    let metadata = InternalChatMessageMetadataPassthrough {
        turn_id: turn_id.map(str::to_string),
        context_fragment_sources: sources.map(|sources| {
            sources
                .into_iter()
                .map(|source| source.map(str::to_string))
                .collect()
        }),
        ..Default::default()
    };
    ResponseItem::Message {
        id: None,
        role: "developer".to_string(),
        content: texts
            .iter()
            .map(|text| ContentItem::InputText {
                text: (*text).to_string(),
            })
            .collect(),
        phase: None,
        internal_chat_message_metadata_passthrough: (metadata
            != InternalChatMessageMetadataPassthrough::default())
        .then_some(metadata),
    }
}

fn retain_texts(item: &mut ResponseItem, texts: &[&str]) {
    item.retain_message_content(|content_item| {
        matches!(content_item, ContentItem::InputText { text } if texts.contains(&text.as_str()))
    });
}

#[test]
fn retained_content_keeps_sources_aligned() {
    let mut item = message(
        &["a", "b", "c"],
        Some("turn-1"),
        Some(vec![Some("x"), None, Some("y")]),
    );
    assert_eq!(
        [0, 1, 2].map(|index| item.context_fragment_source(index)),
        [Some("x"), None, Some("y")]
    );

    retain_texts(&mut item, &["a", "c"]);
    assert_eq!(
        item,
        message(
            &["a", "c"],
            Some("turn-1"),
            Some(vec![Some("x"), Some("y")])
        )
    );

    retain_texts(&mut item, &["c"]);
    assert_eq!(item, message(&["c"], Some("turn-1"), Some(vec![Some("y")])));
}

#[test]
fn unattributed_or_misaligned_sources_are_dropped() {
    let mut item = message(
        &["a", "b"],
        /*turn_id*/ None,
        Some(vec![Some("x"), None]),
    );
    retain_texts(&mut item, &["b"]);
    assert_eq!(item, message(&["b"], /*turn_id*/ None, /*sources*/ None));

    let mut item = message(&["a", "b"], Some("turn-1"), Some(vec![Some("x")]));
    assert_eq!(item.context_fragment_source(/*index*/ 0), None);
    retain_texts(&mut item, &["a", "b"]);
    assert_eq!(item, message(&["a", "b"], Some("turn-1"), /*sources*/ None));
}

#[test]
fn clearing_sources_restores_the_unattributed_payload() {
    let mut item = message(&["a"], Some("turn-1"), Some(vec![Some("x")]));
    item.clear_context_fragment_sources();
    assert_eq!(item, message(&["a"], Some("turn-1"), /*sources*/ None));

    let mut item = message(&["a"], /*turn_id*/ None, Some(vec![Some("x")]));
    item.clear_context_fragment_sources();
    assert_eq!(item, message(&["a"], /*turn_id*/ None, /*sources*/ None));
}

#[test]
fn sources_round_trip_and_legacy_items_deserialize_unattributed() -> serde_json::Result<()> {
    let item = message(&["a", "b"], Some("turn-1"), Some(vec![Some("x"), None]));
    let json = serde_json::to_value(&item)?;
    assert_eq!(
        json["internal_chat_message_metadata_passthrough"],
        serde_json::json!({
            "turn_id": "turn-1",
            "context_fragment_sources": ["x", null],
        })
    );
    assert_eq!(serde_json::from_value::<ResponseItem>(json)?, item);

    let legacy = serde_json::json!({
        "type": "message",
        "role": "developer",
        "content": [{"type": "input_text", "text": "a"}],
        "internal_chat_message_metadata_passthrough": {"turn_id": "turn-1"},
    });
    assert_eq!(
        serde_json::from_value::<ResponseItem>(legacy)?,
        message(&["a"], Some("turn-1"), /*sources*/ None)
    );
    Ok(())
}
