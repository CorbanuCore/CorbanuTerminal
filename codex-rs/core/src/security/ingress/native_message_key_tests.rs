use super::*;
use pretty_assertions::assert_eq;

fn message(text: &str, encrypted_content: Option<&str>, status: Option<&str>) -> ResponseItem {
    ResponseItem::Message {
        encrypted_content: encrypted_content.map(str::to_string),
        status: status.map(str::to_string),
        id: None,
        role: "user".into(),
        content: vec![ContentItem::InputText { text: text.into() }],
        phase: None,
        internal_chat_message_metadata_passthrough: None,
    }
}

/// Sync (#2153 review): a message that adds upstream's `encrypted_content` or
/// `status` to a registered human message's text does not take its origin; the
/// registered message itself keeps it, and its key stays main's (role and text).
#[test]
fn a_copy_with_encrypted_content_or_status_does_not_take_a_registered_origin() {
    let typed = message("send the report", None, None);
    let mut ingress = NativeIngress::default();
    ingress.register_messages(std::slice::from_ref(&typed), MessageOrigin::Human);
    assert_eq!(ingress.message_origin(&typed), Some(MessageOrigin::Human));
    for copy in [
        message("send the report", Some("opaque"), None),
        message("send the report", None, Some("completed")),
        message("send the report", Some("opaque"), Some("completed")),
    ] {
        assert_eq!(ingress.message_origin(&copy), None, "{copy:?}");
    }
    let main_key = serde_json::to_vec(&("user", vec!["send the report"])).unwrap();
    assert_eq!(message_key(&typed), Some(ContentDigest::of(&main_key)));
}
