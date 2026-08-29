use super::MAX_PENDING_OUTPUT_TEXT_BYTES;
use super::PendingOutputText;
use pretty_assertions::assert_eq;

#[test]
fn pending_output_text_binds_unidentified_text_to_the_first_item() {
    let mut pending = PendingOutputText::default();
    pending
        .push(None, "early ".to_string())
        .expect("first delta should buffer");
    pending
        .push(Some("msg-1".to_string()), "text".to_string())
        .expect("later item identity should bind the buffer");

    assert_eq!(pending.take_for_item("msg-1"), Some("early text".into()));
    assert!(pending.is_empty());
}

#[test]
fn pending_output_text_rejects_interleaved_item_ids() {
    let mut pending = PendingOutputText::default();
    pending
        .push(Some("msg-1".to_string()), "first".to_string())
        .expect("first delta should buffer");

    let error = pending
        .push(Some("msg-2".to_string()), "second".to_string())
        .expect_err("different item ids must not share a recovery buffer");

    assert!(error.contains("pending item `msg-1`, new item `msg-2`"));
}

#[test]
fn pending_output_text_enforces_a_hard_byte_limit() {
    let mut pending = PendingOutputText::default();
    let error = pending
        .push(
            Some("msg-1".to_string()),
            "x".repeat(MAX_PENDING_OUTPUT_TEXT_BYTES + 1),
        )
        .expect_err("oversized recovery buffers must fail closed");

    assert!(error.contains("exceeded"));
    assert!(pending.is_empty());
}
