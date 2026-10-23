use super::super::ResponseItem;
use crate::ResponseItemId;
use pretty_assertions::assert_eq;

fn reasoning() -> ResponseItem {
    ResponseItem::Reasoning {
        id: Some(ResponseItemId::from_server("rs_1".to_string())),
        summary: Vec::new(),
        content: None,
        encrypted_content: Some("sealed".to_string()),
        anthropic_content_block: None,
        internal_chat_message_metadata_passthrough: None,
    }
}

#[test]
fn producer_round_trips_through_the_rollout_but_not_requests() {
    let mut item = reasoning();
    item.set_producer("zai");
    assert_eq!(item.producer(), Some("zai"));
    let stored: ResponseItem =
        serde_json::from_value(serde_json::to_value(&item).expect("serialize")).expect("parse");
    assert_eq!(stored.producer(), Some("zai"));

    let mut sent = stored;
    sent.clear_local_provenance();
    assert_eq!(sent, reasoning());
}

#[test]
fn clearing_keeps_other_passthrough_metadata() {
    let mut item = reasoning();
    item.set_turn_id_if_missing("turn-1");
    let expected = item.clone();
    item.set_producer("openai");
    item.clear_local_provenance();
    assert_eq!(item, expected);
    assert_eq!(item.producer(), None);
}
