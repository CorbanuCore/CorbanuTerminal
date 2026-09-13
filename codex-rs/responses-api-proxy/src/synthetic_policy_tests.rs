use super::*;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

#[test]
fn exact_wire_packets_admit_without_rewriting() {
    for packet in PACKETS {
        admit(packet.as_bytes()).unwrap();
        let parsed: Value = serde_json::from_str(packet).unwrap();
        assert_eq!(parsed["store"], false);
        assert_eq!(parsed["model"], "synthetic-no-inference");
        assert!(parsed["input"].is_array());
        // This is a wire compatibility check, not static-constant equality.
        assert!(admit(serde_json::to_vec(&parsed).unwrap().as_slice()).is_err());
    }
}

#[test]
fn altered_authority_and_references_never_admit() {
    let mutations = [
        ("model", json!("arbitrary-model")),
        ("store", json!(true)),
        ("previous_response_id", json!("remote-response")),
        ("conversation", json!("remote-conversation")),
        ("providerOptions", json!({"url":"https://example.invalid"})),
        ("input", json!([{"type":"item_reference","id":"remote"}])),
        (
            "input",
            json!([{"type":"message","role":"user","content":[
            {"type":"input_image","image_url":"https://example.invalid/image.png"}]}]),
        ),
        (
            "input",
            json!([{"type":"reasoning","encrypted_content":"opaque"}]),
        ),
    ];
    for packet in PACKETS {
        for (key, value) in &mutations {
            let mut parsed: Value = serde_json::from_str(packet).unwrap();
            parsed[key] = value.clone();
            assert!(
                admit(&serde_json::to_vec(&parsed).unwrap()).is_err(),
                "{key}"
            );
        }
        for tool in [
            "web_search",
            "file_search",
            "mcp",
            "computer",
            "code_interpreter",
            "tool_search",
            "image_generation",
        ] {
            let mut parsed: Value = serde_json::from_str(packet).unwrap();
            parsed["tools"] = json!([{"type":tool}]);
            assert!(
                admit(&serde_json::to_vec(&parsed).unwrap()).is_err(),
                "{tool}"
            );
        }
        for suffix in [",\"store\":false}", ",\"unknown\":null}"] {
            let changed = format!("{}{suffix}", &packet[..packet.len() - 1]);
            assert!(admit(changed.as_bytes()).is_err());
        }
        assert!(admit(format!(" {packet}").as_bytes()).is_err());
        assert!(admit(&packet.as_bytes()[..packet.len() - 1]).is_err());
    }
}
