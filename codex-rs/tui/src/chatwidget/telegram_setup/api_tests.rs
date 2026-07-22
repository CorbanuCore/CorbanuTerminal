use pretty_assertions::assert_eq;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::body_json;
use wiremock::matchers::method;
use wiremock::matchers::path;

use super::*;

#[test]
fn chat_display_name_uses_group_title_and_private_identity_fallbacks() {
    let actor = TelegramUser {
        id: 7,
        first_name: "Alice".to_string(),
        username: Some("alice".to_string()),
    };
    let group = TelegramChat {
        id: -42,
        kind: "supergroup".to_string(),
        first_name: None,
        last_name: None,
        title: Some("Engineering".to_string()),
        username: None,
    };
    let private = TelegramChat {
        id: 7,
        kind: "private".to_string(),
        first_name: Some("Alice".to_string()),
        last_name: Some("Example".to_string()),
        title: None,
        username: None,
    };

    assert_eq!(chat_display_name(&group, &actor), "Engineering");
    assert_eq!(chat_display_name(&private, &actor), "Alice Example");
}

#[tokio::test]
async fn telegram_identity_is_validated_without_exposing_token_in_errors() {
    let server = MockServer::start().await;
    let token = "12345:super-secret-token";
    Mock::given(method("POST"))
        .and(path(format!("/bot{token}/getMe")))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "ok": false,
            "description": "Unauthorized"
        })))
        .mount(&server)
        .await;

    let error = telegram_identity_at(&server.uri(), token)
        .await
        .expect_err("invalid token must fail");

    assert_eq!(
        error,
        "BotFather rejected this token. Paste the current bot token."
    );
    assert!(!error.contains(token));
}

#[tokio::test]
async fn telegram_identity_accepts_a_bot_and_requires_username() {
    let server = MockServer::start().await;
    let token = "12345:test-token";
    Mock::given(method("POST"))
        .and(path(format!("/bot{token}/getMe")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {"id": 99, "first_name": "Pumps", "username": "pumps_bot"}
        })))
        .mount(&server)
        .await;

    assert_eq!(
        telegram_identity_at(&server.uri(), token)
            .await
            .expect("valid bot"),
        TelegramBotIdentity {
            id: 99,
            username: "pumps_bot".to_string(),
            display_name: "Pumps".to_string(),
        }
    );
}

#[tokio::test]
async fn discovery_returns_exact_chat_and_actor_pairs() {
    let server = MockServer::start().await;
    let token = "12345:test-token";
    Mock::given(method("POST"))
        .and(path(format!("/bot{token}/getMe")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": {"id": 99, "first_name": "Pumps", "username": "pumps_bot"}
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/bot{token}/getUpdates")))
        .and(body_json(serde_json::json!({
            "timeout": 10,
            "allowed_updates": ["message"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true,
            "result": [{
                "update_id": 12,
                "message": {
                    "chat": {"id": -77, "type": "supergroup", "title": "Engineering"},
                    "from": {"id": 42, "first_name": "Alice", "username": "alice"}
                }
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let discovery = discover_at(&server.uri(), token)
        .await
        .expect("discover chat");

    assert_eq!(
        discovery.candidates,
        vec![TelegramChatCandidate {
            chat_id: -77,
            actor_user_id: 42,
            display_name: "Engineering".to_string(),
            chat_kind: "supergroup".to_string(),
        }]
    );
}
