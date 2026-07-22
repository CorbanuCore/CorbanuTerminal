use super::PER_CONVERSATION_QUEUE_BYTES;
use super::queue_capacity_error;
use super::turn_input;
use super::turn_items_contain_client_message;
use codex_app_server_protocol::ThreadItem;

#[test]
fn image_only_message_yields_only_image_items() {
    assert_eq!(
        turn_input(String::new(), vec!["/tmp/a.jpg".into()]).len(),
        1
    );
}

#[test]
fn caption_follows_images() {
    let input = turn_input("look at this".into(), vec!["/tmp/a.jpg".into()]);
    let json = serde_json::to_value(&input).unwrap();
    assert_eq!(input.len(), 2);
    assert!(json[0].to_string().contains("a.jpg"));
    assert!(json[1].to_string().contains("look at this"));
}

#[test]
fn text_only_unchanged() {
    assert_eq!(turn_input("hi".into(), Vec::new()).len(), 1);
}

#[test]
fn queue_limits_are_hard_for_items_and_bytes() {
    assert!(queue_capacity_error(15, 0, 1).is_none());
    assert!(
        queue_capacity_error(16, 0, 1)
            .unwrap()
            .contains("16 messages")
    );
    assert!(
        queue_capacity_error(0, PER_CONVERSATION_QUEUE_BYTES, 1)
            .unwrap()
            .contains("256 KiB")
    );
}

#[test]
fn replay_reconciliation_finds_only_the_matching_client_message() {
    let first_turn = vec![ThreadItem::UserMessage {
        id: "item-1".into(),
        client_id: Some("telegram:1:41".into()),
        content: Vec::new(),
    }];
    let second_turn = vec![ThreadItem::AgentMessage {
        id: "item-2".into(),
        text: "done".into(),
        phase: None,
        memory_citation: None,
    }];
    let turns = [first_turn.as_slice(), second_turn.as_slice()];

    assert!(turn_items_contain_client_message(turns, "telegram:1:41"));
    assert!(!turn_items_contain_client_message(turns, "telegram:1:42"));
}
