use super::PER_CONVERSATION_QUEUE_BYTES;
use super::queue_capacity_error;
use super::thread_read_unavailable_before_first_message;
use super::turn_input;
use super::turn_items_contain_client_message;
use codex_app_server_client::TypedRequestError;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::THREAD_UNMATERIALIZED_INCLUDE_TURNS_MESSAGE;
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

/// The raw message the app server sends for an unmaterialized thread. It
/// carries no `thread/read failed:` prefix; that is added by
/// `TypedRequestError`'s `Display`, which the helper never sees.
fn unmaterialized_thread_read_error() -> JSONRPCErrorError {
    JSONRPCErrorError {
        code: -32600,
        message: format!("thread abc {THREAD_UNMATERIALIZED_INCLUDE_TURNS_MESSAGE}"),
        data: None,
    }
}

#[test]
fn unmaterialized_thread_read_is_treated_as_an_empty_reconciliation_source() {
    assert!(thread_read_unavailable_before_first_message(
        "thread/read",
        &unmaterialized_thread_read_error()
    ));
}

/// Regression test for the shipped bug: the helper was matching text that only
/// exists after formatting, so it returned false for every real response and
/// the first message after `/new` blocked the conversation. Both forms must be
/// tolerated, but only the raw one ever reaches the helper.
#[test]
fn the_formatted_prefix_is_absent_from_the_message_the_helper_receives() {
    let raw = unmaterialized_thread_read_error();
    assert!(!raw.message.starts_with("thread/read failed:"));

    let formatted = TypedRequestError::Server {
        method: "thread/read".into(),
        source: raw.clone(),
    }
    .to_string();
    assert!(formatted.starts_with("thread/read failed:"));

    assert!(thread_read_unavailable_before_first_message(
        "thread/read",
        &raw
    ));
    assert!(thread_read_unavailable_before_first_message(
        "thread/read",
        &JSONRPCErrorError {
            message: formatted,
            ..raw
        }
    ));
}

#[test]
fn unrelated_invalid_requests_are_not_suppressed_during_reconciliation() {
    for message in [
        "thread abc does not exist",
        "includeTurns is unavailable for archived threads",
        "thread abc is not materialized yet; thread/turns/list is unavailable before first user message",
    ] {
        let error = JSONRPCErrorError {
            code: -32600,
            message: message.into(),
            data: None,
        };
        assert!(!thread_read_unavailable_before_first_message(
            "thread/read",
            &error
        ));
    }

    let wrong_code = JSONRPCErrorError {
        code: -32603,
        ..unmaterialized_thread_read_error()
    };
    assert!(!thread_read_unavailable_before_first_message(
        "thread/read",
        &wrong_code
    ));

    assert!(!thread_read_unavailable_before_first_message(
        "turn/start",
        &unmaterialized_thread_read_error()
    ));
}
