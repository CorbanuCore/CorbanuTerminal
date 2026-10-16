//! Source-informed issue863 regression. Provider data is synthetic; no inference.
use super::*;
use crate::tools::collaboration_wire::message_source_provider_id;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn issue863_native_custom_same_source_is_admitted() {
    let (_session, mut turn) = crate::session::tests::make_session_and_context().await;
    let mut config = (*turn.config).clone();
    config.model_provider_id = "myopenai".to_string();
    config.model_provider.name = "My OpenAI".to_string();
    config.model_provider.base_url = Some("https://api.openai.com/v1".to_string());
    config.model_provider.wire_api = codex_model_provider_info::WireApi::Responses;
    config.model_provider.requires_openai_auth = false;
    config.model_provider.env_key = Some("SYNTHETIC_ISSUE863_API_KEY".to_string());
    turn.provider = codex_model_provider::create_model_provider(
        config.model_provider.clone(),
        turn.auth_manager.clone(),
    );
    turn.config = std::sync::Arc::new(config);
    assert_eq!(turn.config.model_provider_id, "myopenai");
    assert_eq!(message_source_provider_id(&turn), OPENAI_PROVIDER_ID);
    assert_eq!(
        ensure_message_encoding_matches_target(
            &turn.config.model_provider_id,
            message_source_provider_id(&turn),
            &crate::tools::context::ToolCallSource::Direct,
            "myopenai",
            CollaborationMessageEncoding::ProviderNative,
            "spawn_agent",
            PLAINTEXT_SPAWN_AGENT_TOOL,
        ),
        Ok(())
    );
}

#[tokio::test]
async fn issue863_native_custom_to_builtin_source_is_rejected_before_admission() {
    let (_session, mut turn) = crate::session::tests::make_session_and_context().await;
    let mut config = (*turn.config).clone();
    config.model_provider_id = "myopenai".to_string();
    config.model_provider.name = "My OpenAI".to_string();
    config.model_provider.base_url = Some("https://api.openai.com/v1".to_string());
    config.model_provider.wire_api = codex_model_provider_info::WireApi::Responses;
    config.model_provider.requires_openai_auth = false;
    config.model_provider.env_key = Some("SYNTHETIC_ISSUE863_API_KEY".to_string());
    turn.provider = codex_model_provider::create_model_provider(
        config.model_provider.clone(),
        turn.auth_manager.clone(),
    );
    turn.config = std::sync::Arc::new(config);
    assert_eq!(
        ensure_message_encoding_matches_target(
            &turn.config.model_provider_id,
            message_source_provider_id(&turn),
            &crate::tools::context::ToolCallSource::Direct,
            OPENAI_PROVIDER_ID,
            CollaborationMessageEncoding::ProviderNative,
            "followup_task",
            PLAINTEXT_FOLLOWUP_TASK_TOOL,
        ),
        Err(FunctionCallError::RespondToModel(
            "target provider `openai` cannot consume this OpenAI-encrypted collaboration payload. Call the function named `followup_task_plaintext` directly with exactly the same `target` and the plaintext `message`; do not call `followup_task` again and do not add an `adapter` field. No message was admitted and no target turn was started.".to_string()
        ))
    );
}

#[test]
fn issue863_existing_builtin_and_plaintext_controls() {
    let source = crate::tools::context::ToolCallSource::Direct;
    for (target, encoding) in [
        (
            OPENAI_PROVIDER_ID,
            CollaborationMessageEncoding::ProviderNative,
        ),
        ("myopenai", CollaborationMessageEncoding::PlaintextAdapter),
    ] {
        assert_eq!(
            ensure_message_encoding_matches_target(
                OPENAI_PROVIDER_ID,
                OPENAI_PROVIDER_ID,
                &source,
                target,
                encoding,
                "spawn_agent",
                PLAINTEXT_SPAWN_AGENT_TOOL,
            ),
            Ok(())
        );
    }
}

#[tokio::test]
async fn issue863_reserved_schema_endpoint_matrix_stays_conservative() {
    use crate::tools::collaboration_wire::uses_openai_reserved_collaboration_schema;
    use codex_model_provider_info::WireApi;

    let (_session, mut turn) = crate::session::tests::make_session_and_context().await;
    let cases = [
        ("https://api.openai.com/v1", true),
        ("https://api.openai.com/v1/", true),
        ("https://api.openai.com:443/v1", true),
        ("http://api.openai.com/v1", false),
        ("https://api.openai.com./v1", false),
        ("https://eu.api.openai.com/v1", false),
        ("https://gateway.example/v1", false),
        ("https://api.openai.com:444/v1", false),
        ("https://api.openai.com/v2", false),
        ("https://api.openai.com/v1?tenant=synthetic", false),
        ("https://api.openai.com/v1#synthetic", false),
        ("https://synthetic@api.openai.com/v1", false),
        ("https://user:synthetic@api.openai.com/v1", false),
    ];
    for wire in [WireApi::Responses, WireApi::Chat, WireApi::Anthropic] {
        for (url, responses_supported) in cases {
            let mut config = (*turn.config).clone();
            config.model_provider_id = "myopenai".to_string();
            config.model_provider.name = "My OpenAI".to_string();
            config.model_provider.base_url = Some(url.to_string());
            config.model_provider.wire_api = wire;
            config.model_provider.requires_openai_auth = false;
            turn.provider = codex_model_provider::create_model_provider(
                config.model_provider.clone(),
                turn.auth_manager.clone(),
            );
            turn.config = std::sync::Arc::new(config);
            let expected = wire == WireApi::Responses && responses_supported;
            assert_eq!(
                uses_openai_reserved_collaboration_schema(&turn),
                expected,
                "{wire}: {url}"
            );
            assert_eq!(turn.config.model_provider_id, "myopenai");
            assert_eq!(
                message_source_provider_id(&turn),
                if expected {
                    OPENAI_PROVIDER_ID
                } else {
                    "myopenai"
                }
            );
        }
    }
}

#[test]
fn issue863_real_source_codec_target_and_plaintext_matrix() {
    let source = crate::tools::context::ToolCallSource::Direct;
    for (real_source, codec, target, encoding, admitted) in [
        (
            "myopenai",
            OPENAI_PROVIDER_ID,
            "myopenai",
            CollaborationMessageEncoding::ProviderNative,
            true,
        ),
        (
            "myopenai",
            OPENAI_PROVIDER_ID,
            OPENAI_PROVIDER_ID,
            CollaborationMessageEncoding::ProviderNative,
            false,
        ),
        (
            "myopenai",
            OPENAI_PROVIDER_ID,
            "other-custom",
            CollaborationMessageEncoding::ProviderNative,
            false,
        ),
        (
            OPENAI_PROVIDER_ID,
            OPENAI_PROVIDER_ID,
            OPENAI_PROVIDER_ID,
            CollaborationMessageEncoding::ProviderNative,
            true,
        ),
        (
            OPENAI_PROVIDER_ID,
            OPENAI_PROVIDER_ID,
            "myopenai",
            CollaborationMessageEncoding::ProviderNative,
            false,
        ),
        (
            "myopenai",
            OPENAI_PROVIDER_ID,
            OPENAI_PROVIDER_ID,
            CollaborationMessageEncoding::PlaintextAdapter,
            true,
        ),
        (
            "myopenai",
            OPENAI_PROVIDER_ID,
            "other-custom",
            CollaborationMessageEncoding::PlaintextAdapter,
            true,
        ),
        (
            "local-provider",
            "local-provider",
            "other-custom",
            CollaborationMessageEncoding::ProviderNative,
            true,
        ),
    ] {
        let expected = if admitted {
            Ok(())
        } else {
            Err(FunctionCallError::RespondToModel(format!(
                "target provider `{target}` cannot consume this OpenAI-encrypted collaboration payload. Call the function named `followup_task_plaintext` directly with exactly the same `target` and the plaintext `message`; do not call `followup_task` again and do not add an `adapter` field. No message was admitted and no target turn was started."
            )))
        };
        assert_eq!(
            ensure_message_encoding_matches_target(
                real_source,
                codec,
                &source,
                target,
                encoding,
                "followup_task",
                PLAINTEXT_FOLLOWUP_TASK_TOOL,
            ),
            expected
        );
    }
    for source in [
        crate::tools::context::ToolCallSource::DirectPlaintextMessage,
        crate::tools::context::ToolCallSource::CodeMode {
            cell_id: "synthetic-cell".to_string(),
            runtime_tool_call_id: "synthetic-tool".to_string(),
        },
    ] {
        assert_eq!(
            ensure_message_encoding_matches_target(
                "myopenai",
                OPENAI_PROVIDER_ID,
                &source,
                "other-custom",
                CollaborationMessageEncoding::ProviderNative,
                "followup_task",
                PLAINTEXT_FOLLOWUP_TASK_TOOL,
            ),
            Ok(())
        );
    }
}
