use super::*;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_absolute_path::AbsolutePathBufGuard;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_deserialize_ollama_model_provider_toml() {
    let azure_provider_toml = r#"
name = "Ollama"
base_url = "http://localhost:11434/v1"
        "#;
    let expected_provider = ModelProviderInfo {
        name: "Ollama".into(),
        base_url: Some("http://localhost:11434/v1".into()),
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        runtime_policy: ProviderRuntimePolicy::default(),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
        supports_standalone_web_search: false,
    };

    let provider: ModelProviderInfo = toml::from_str(azure_provider_toml).unwrap();
    assert_eq!(expected_provider, provider);
}

#[test]
fn test_deserialize_azure_model_provider_toml() {
    let azure_provider_toml = r#"
name = "Azure"
base_url = "https://xxxxx.openai.azure.com/openai"
env_key = "AZURE_OPENAI_API_KEY"
query_params = { api-version = "2025-04-01-preview" }
        "#;
    let expected_provider = ModelProviderInfo {
        name: "Azure".into(),
        base_url: Some("https://xxxxx.openai.azure.com/openai".into()),
        env_key: Some("AZURE_OPENAI_API_KEY".into()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Responses,
        query_params: Some(maplit::hashmap! {
            "api-version".to_string() => "2025-04-01-preview".to_string(),
        }),
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        runtime_policy: ProviderRuntimePolicy::default(),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
        supports_standalone_web_search: false,
    };

    let provider: ModelProviderInfo = toml::from_str(azure_provider_toml).unwrap();
    assert_eq!(expected_provider, provider);
}

#[test]
fn test_deserialize_example_model_provider_toml() {
    let azure_provider_toml = r#"
name = "Example"
base_url = "https://example.com"
env_key = "API_KEY"
http_headers = { "X-Example-Header" = "example-value" }
env_http_headers = { "X-Example-Env-Header" = "EXAMPLE_ENV_VAR" }
supports_standalone_web_search = true
        "#;
    let expected_provider = ModelProviderInfo {
        name: "Example".into(),
        base_url: Some("https://example.com".into()),
        env_key: Some("API_KEY".into()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: Some(maplit::hashmap! {
            "X-Example-Header".to_string() => "example-value".to_string(),
        }),
        env_http_headers: Some(maplit::hashmap! {
            "X-Example-Env-Header".to_string() => "EXAMPLE_ENV_VAR".to_string(),
        }),
        chat_completions_provider: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        runtime_policy: ProviderRuntimePolicy::default(),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
        supports_standalone_web_search: true,
    };

    let provider: ModelProviderInfo = toml::from_str(azure_provider_toml).unwrap();
    assert_eq!(expected_provider, provider);
}

#[test]
fn test_deserialize_chat_wire_api() {
    let provider_toml = r#"
name = "OpenAI using Chat Completions"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
wire_api = "chat"
        "#;

    let provider: ModelProviderInfo = toml::from_str(provider_toml).unwrap();
    assert_eq!(provider.wire_api, WireApi::Chat);
}

#[test]
fn only_kimi_built_in_uses_ambiguous_action_stop_semantics() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    assert_eq!(
        providers[KIMI_CODE_PROVIDER_ID].chat_stop_semantics(),
        ChatStopSemantics::AmbiguousForActionTurns
    );
    for (provider_id, provider) in &providers {
        if provider_id != KIMI_CODE_PROVIDER_ID {
            assert_eq!(
                provider.chat_stop_semantics(),
                ChatStopSemantics::ReliableTerminal,
                "unexpected ambiguous stop semantics for {provider_id}"
            );
        }
    }
}

#[test]
fn built_in_command_auth_uses_the_canonical_installed_executable() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let command_auth_providers = providers
        .iter()
        .filter_map(|(provider_id, provider)| {
            provider
                .auth
                .as_ref()
                .map(|auth| (provider_id.as_str(), auth.command.as_str()))
        })
        .collect::<Vec<_>>();

    assert!(
        !command_auth_providers.is_empty(),
        "the invariant must exercise at least one built-in command-auth provider"
    );
    for (provider_id, command) in command_auth_providers {
        assert_eq!(
            command, CORBANU_PROVIDER_AUTH_COMMAND,
            "built-in provider {provider_id} depends on an executable that supported Corbanu installs do not guarantee"
        );
    }
}

#[test]
fn test_deserialize_anthropic_wire_api() {
    let provider_toml = r#"
name = "Anthropic-compatible"
base_url = "https://api.example.com/v1"
env_key = "ANTHROPIC_API_KEY"
wire_api = "anthropic"
        "#;

    let provider: ModelProviderInfo = toml::from_str(provider_toml).unwrap();
    assert_eq!(provider.wire_api, WireApi::Anthropic);
}

#[test]
fn test_deserialize_websocket_connect_timeout() {
    let provider_toml = r#"
name = "OpenAI"
base_url = "https://api.openai.com/v1"
websocket_connect_timeout_ms = 15000
supports_websockets = true
        "#;

    let provider: ModelProviderInfo = toml::from_str(provider_toml).unwrap();
    assert_eq!(provider.websocket_connect_timeout_ms, Some(15_000));
}

#[test]
fn test_supports_remote_compaction_for_openai() {
    let provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);

    assert!(provider.supports_remote_compaction());
}

#[test]
fn openai_provider_uses_codex_compat_version_header() {
    let provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);

    assert_eq!(
        provider
            .http_headers
            .as_ref()
            .and_then(|headers| headers.get("version"))
            .map(String::as_str),
        Some(OPENAI_CODEX_COMPAT_VERSION)
    );
}

#[test]
fn openai_provider_base_url_override_conservatively_disables_websockets() {
    let provider = ModelProviderInfo::create_openai_provider(Some(
        "https://openai-compatible.example/v1".to_string(),
    ));

    assert!(!provider.supports_websockets);
}

#[test]
fn test_personal_access_token_uses_chatgpt_codex_base_url() {
    let api_provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None)
        .to_api_provider(Some(AuthMode::PersonalAccessToken))
        .expect("OpenAI provider should build API provider");

    assert_eq!(api_provider.base_url, CHATGPT_CODEX_BASE_URL);
}

#[test]
fn test_header_auth_uses_chatgpt_codex_base_url() {
    let api_provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None)
        .to_api_provider(Some(AuthMode::Headers))
        .expect("OpenAI provider should build API provider");

    assert_eq!(api_provider.base_url, CHATGPT_CODEX_BASE_URL);
}

#[test]
fn test_supports_remote_compaction_for_azure_name() {
    let provider = ModelProviderInfo {
        name: "Azure".into(),
        base_url: Some("https://example.com/openai".into()),
        env_key: Some("AZURE_OPENAI_API_KEY".into()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        runtime_policy: ProviderRuntimePolicy::default(),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
        supports_standalone_web_search: false,
    };

    assert!(provider.supports_remote_compaction());
}

#[test]
fn test_supports_remote_compaction_for_non_openai_non_azure_provider() {
    let provider = ModelProviderInfo {
        name: "Example".into(),
        base_url: Some("https://example.com/v1".into()),
        env_key: Some("API_KEY".into()),
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        chat_completions_provider: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        stream_actionable_timeout_ms: None,
        stream_long_failure_retry_threshold_ms: None,
        stream_long_failure_max_retries: None,
        runtime_policy: ProviderRuntimePolicy::default(),
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: false,
        supports_standalone_web_search: false,
    };

    assert!(!provider.supports_remote_compaction());
}

#[test]
fn test_uses_openai_actor_authorization() {
    let mut provider = ModelProviderInfo {
        http_headers: Some(maplit::hashmap! {
            "X-OpenAI-Actor-Authorization".to_string() => "actor-token".to_string(),
        }),
        ..ModelProviderInfo::default()
    };
    assert!(provider.uses_openai_actor_authorization());

    provider.http_headers = None;
    assert!(!provider.uses_openai_actor_authorization());

    provider.http_headers = Some(maplit::hashmap! {
        OPENAI_ACTOR_AUTHORIZATION_HEADER.to_string() => "  ".to_string(),
    });
    assert!(!provider.uses_openai_actor_authorization());

    provider.http_headers = Some(maplit::hashmap! {
        OPENAI_ACTOR_AUTHORIZATION_HEADER.to_string() => "actor-token".to_string(),
    });
    provider.requires_openai_auth = true;
    assert!(!provider.uses_openai_actor_authorization());
}

#[test]
fn test_deserialize_provider_auth_config_defaults() {
    let base_dir = tempdir().unwrap();
    let provider_toml = r#"
name = "Corp"

[auth]
command = "./scripts/print-token"
args = ["--format=text"]
        "#;

    let provider: ModelProviderInfo = {
        let _guard = AbsolutePathBufGuard::new(base_dir.path());
        toml::from_str(provider_toml).unwrap()
    };

    assert_eq!(
        provider.auth,
        Some(ModelProviderAuthInfo {
            command: "./scripts/print-token".to_string(),
            args: vec!["--format=text".to_string()],
            timeout_ms: NonZeroU64::new(5_000).unwrap(),
            refresh_interval_ms: 300_000,
            cwd: AbsolutePathBuf::resolve_path_against_base(".", base_dir.path()),
        })
    );
}

#[test]
fn test_deserialize_provider_aws_config() {
    let provider_toml = r#"
name = "Amazon Bedrock"
base_url = "https://bedrock.example.com/v1"

[aws]
profile = "codex-bedrock"
region = "us-west-2"
        "#;

    let provider: ModelProviderInfo = toml::from_str(provider_toml).unwrap();

    assert_eq!(
        provider.aws,
        Some(ModelProviderAwsAuthInfo {
            profile: Some("codex-bedrock".to_string()),
            region: Some("us-west-2".to_string()),
        })
    );
}

#[test]
fn test_create_amazon_bedrock_provider() {
    assert_eq!(
        ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None),
        ModelProviderInfo {
            name: "Amazon Bedrock".to_string(),
            base_url: None,
            env_key: None,
            env_key_instructions: None,
            experimental_bearer_token: None,
            auth: None,
            aws: Some(ModelProviderAwsAuthInfo {
                profile: None,
                region: None,
            }),
            wire_api: WireApi::Responses,
            query_params: None,
            http_headers: Some(maplit::hashmap! {
                AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_HEADER.to_string() =>
                    AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_VALUE.to_string(),
            }),
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: ProviderRuntimePolicy::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    );
}

fn provider_auth_for_test() -> ModelProviderAuthInfo {
    ModelProviderAuthInfo {
        command: "token-fetcher".to_string(),
        args: vec!["fetch".to_string()],
        timeout_ms: NonZeroU64::new(5_000).expect("timeout should be non-zero"),
        refresh_interval_ms: 300_000,
        cwd: std::env::current_dir()
            .expect("current directory should be available")
            .try_into()
            .expect("current directory should be absolute"),
    }
}

#[test]
fn test_create_ambient_provider() {
    assert_eq!(
        ModelProviderInfo::create_ambient_provider(),
        ModelProviderInfo {
            name: "Ambient".to_string(),
            base_url: Some(AMBIENT_BASE_URL.to_string()),
            env_key: Some(AMBIENT_API_KEY_ENV_VAR.to_string()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: ProviderRuntimePolicy::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    );
    assert_eq!(AMBIENT_DEFAULT_MODEL, "z-ai/glm-5.2");
    assert_eq!(AMBIENT_LEGACY_GLM_5_2_FP8_MODEL, "zai-org/GLM-5.2-FP8");
    assert_eq!(AMBIENT_KIMI_K2_7_CODE_MODEL, "moonshotai/kimi-k2.7-code");
}

#[test]
fn ambient_glm_context_ceiling_is_scoped_to_ambient_routes() {
    assert_eq!(
        default_model_context_window_for_provider(AMBIENT_PROVIDER_ID, AMBIENT_DEFAULT_MODEL),
        Some(AMBIENT_GLM_5_2_CONTEXT_WINDOW)
    );
    assert_eq!(
        default_model_context_window_for_provider(
            PFTERMINAL_PLAN_PROVIDER_ID,
            AMBIENT_DEFAULT_MODEL
        ),
        Some(AMBIENT_GLM_5_2_CONTEXT_WINDOW)
    );
    assert_eq!(
        default_model_context_window_for_provider(OPENROUTER_PROVIDER_ID, AMBIENT_DEFAULT_MODEL),
        None
    );
}

#[test]
fn plan_fable_context_ceiling_is_scoped_to_the_skyapi_route() {
    assert_eq!(
        default_model_context_window_for_provider(
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
            CLAUDE_FABLE_5_MODEL,
        ),
        Some(PFTERMINAL_PLAN_FABLE_CONTEXT_WINDOW)
    );
    assert_eq!(
        default_model_context_window_for_provider(
            CORBANU_PLAN_ANTHROPIC_PROVIDER_ID,
            CLAUDE_FABLE_5_MODEL,
        ),
        Some(PFTERMINAL_PLAN_FABLE_CONTEXT_WINDOW)
    );
    assert_eq!(
        default_model_context_window_for_provider(ANTHROPIC_PROVIDER_ID, CLAUDE_FABLE_5_MODEL),
        None
    );
}

#[test]
fn plan_fable_output_ceiling_is_scoped_to_the_skyapi_route() {
    for provider in [
        PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID,
        CORBANU_PLAN_ANTHROPIC_PROVIDER_ID,
    ] {
        assert_eq!(
            default_model_max_output_tokens_for_provider(provider, CLAUDE_FABLE_5_MODEL),
            Some(PFTERMINAL_PLAN_FABLE_MAX_OUTPUT_TOKENS)
        );
    }
    assert_eq!(
        default_model_max_output_tokens_for_provider(ANTHROPIC_PROVIDER_ID, CLAUDE_FABLE_5_MODEL,),
        None
    );
}

#[test]
fn test_create_zai_provider() {
    assert_eq!(
        ModelProviderInfo::create_zai_provider(),
        ModelProviderInfo {
            name: "Z.AI".to_string(),
            base_url: Some(ZAI_BASE_URL.to_string()),
            env_key: Some(ZAI_API_KEY_ENV_VAR.to_string()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Chat,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: ProviderRuntimePolicy::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    );
    assert_eq!(ZAI_DEFAULT_MODEL, "glm-5.2");
}

#[test]
fn test_create_anthropic_provider() {
    assert_eq!(
        ModelProviderInfo::create_anthropic_provider(),
        ModelProviderInfo {
            name: "Anthropic".to_string(),
            base_url: Some(ANTHROPIC_BASE_URL.to_string()),
            env_key: Some(ANTHROPIC_API_KEY_ENV_VAR.to_string()),
            env_key_instructions: Some(provider_api_key_vault_instructions()),
            experimental_bearer_token: None,
            auth: None,
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: None,
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: ProviderRuntimePolicy::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    );
    assert_eq!(ANTHROPIC_DEFAULT_MODEL, "claude-opus-5");
    assert_eq!(CLAUDE_FABLE_5_1_MODEL, "claude-fable-5-1");
    assert_eq!(CLAUDE_FABLE_5_MODEL, "claude-fable-5");
}

#[test]
fn test_create_claude_plan_provider() {
    let expected_auth_cwd =
        AbsolutePathBuf::current_dir().expect("current directory should be absolute");

    assert_eq!(
        ModelProviderInfo::create_claude_plan_provider(),
        ModelProviderInfo {
            name: "Claude Plan".to_string(),
            base_url: Some(ANTHROPIC_BASE_URL.to_string()),
            env_key: None,
            env_key_instructions: None,
            experimental_bearer_token: None,
            auth: Some(ModelProviderAuthInfo {
                command: CORBANU_PROVIDER_AUTH_COMMAND.to_string(),
                args: vec!["internal-claude-oauth-token".to_string()],
                timeout_ms: NonZeroU64::new(60_000).expect("timeout should be non-zero"),
                refresh_interval_ms: 0,
                cwd: expected_auth_cwd,
            }),
            aws: None,
            wire_api: WireApi::Anthropic,
            query_params: None,
            http_headers: Some(maplit::hashmap! {
                "anthropic-beta".to_string() => "claude-code-20250219,oauth-2025-04-20".to_string(),
            }),
            env_http_headers: None,
            chat_completions_provider: None,
            request_max_retries: None,
            stream_max_retries: None,
            stream_idle_timeout_ms: None,
            stream_actionable_timeout_ms: None,
            stream_long_failure_retry_threshold_ms: None,
            stream_long_failure_max_retries: None,
            runtime_policy: ProviderRuntimePolicy::default(),
            websocket_connect_timeout_ms: None,
            requires_openai_auth: false,
            supports_websockets: false,
            supports_standalone_web_search: false,
        }
    );
    assert_eq!(CLAUDE_PLAN_MODEL, "claude-opus-5-plan");
    assert_eq!(CLAUDE_PLAN_UPSTREAM_MODEL, ANTHROPIC_DEFAULT_MODEL);
    assert_eq!(CLAUDE_FABLE_5_1_PLAN_MODEL, "claude-fable-5-1-plan");
    assert_eq!(CLAUDE_FABLE_5_1_PLAN_UPSTREAM_MODEL, CLAUDE_FABLE_5_1_MODEL);
    assert_eq!(CLAUDE_FABLE_5_PLAN_MODEL, "claude-fable-5-plan");
    assert_eq!(CLAUDE_FABLE_5_PLAN_UPSTREAM_MODEL, CLAUDE_FABLE_5_MODEL);
}

#[test]
fn test_amazon_bedrock_provider_adds_mantle_client_agent_header() {
    let api_provider = ModelProviderInfo::create_amazon_bedrock_provider(/*aws*/ None)
        .to_api_provider(/*auth_mode*/ None)
        .expect("Amazon Bedrock provider should build API provider");

    assert_eq!(
        api_provider
            .headers
            .get(AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(AMAZON_BEDROCK_MANTLE_CLIENT_AGENT_VALUE)
    );
}

#[test]
fn test_built_in_model_providers_include_amazon_bedrock() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    assert_eq!(
        providers
            .get(AMAZON_BEDROCK_PROVIDER_ID)
            .map(ModelProviderInfo::is_amazon_bedrock),
        Some(true)
    );
}

#[test]
fn test_built_in_model_providers_include_ambient() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    assert_eq!(
        providers
            .get(AMBIENT_PROVIDER_ID)
            .map(ModelProviderInfo::is_ambient),
        Some(true)
    );
}

#[test]
fn test_corbanu_plan_uses_the_branded_public_api_origin() {
    assert_eq!(PFTERMINAL_PLAN_GATEWAY_ORIGIN, "https://api.corbanu.com");
    assert_eq!(
        PFTERMINAL_PLAN_DEFAULT_BASE_URL,
        "https://api.corbanu.com/v1"
    );
    assert!(!PFTERMINAL_PLAN_GATEWAY_ORIGIN.contains("fly.dev"));
}

#[test]
fn test_built_in_model_providers_keep_legacy_plan_id_with_corbanu_name() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let provider = providers
        .get(PFTERMINAL_PLAN_PROVIDER_ID)
        .expect("Corbanu Plan provider");

    assert!(provider.is_pfterminal_plan());
    assert_eq!(provider.name, PLAN_NAME);
    assert_eq!(
        provider.env_key.as_deref(),
        Some(PFTERMINAL_PLAN_API_KEY_ENV_VAR)
    );
    assert_eq!(provider.wire_api, WireApi::Chat);
    assert!(!provider.requires_openai_auth);
    assert_eq!(
        provider.api_key_env_vars(),
        vec![
            CORBANU_PLAN_API_KEY_ENV_VAR,
            PFTERMINAL_PLAN_API_KEY_ENV_VAR
        ]
    );
    assert_eq!(
        canonical_provider_id(CORBANU_PLAN_PROVIDER_ID),
        PFTERMINAL_PLAN_PROVIDER_ID
    );
    assert_eq!(
        canonical_provider_id(CORBANU_TERMINAL_PLAN_PROVIDER_ID),
        PFTERMINAL_PLAN_PROVIDER_ID
    );
    assert_eq!(
        canonical_provider_id(PFTERMINAL_PLAN_PROVIDER_ID),
        PFTERMINAL_PLAN_PROVIDER_ID
    );
}

#[test]
fn test_built_in_model_providers_include_anthropic() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let anthropic = providers
        .get(ANTHROPIC_PROVIDER_ID)
        .expect("Anthropic provider should be built in");
    assert!(anthropic.is_anthropic());
    assert_eq!(anthropic.base_url.as_deref(), Some(ANTHROPIC_BASE_URL));
    assert_eq!(
        anthropic.env_key.as_deref(),
        Some(ANTHROPIC_API_KEY_ENV_VAR)
    );
    assert_eq!(anthropic.wire_api, WireApi::Anthropic);
    assert_eq!(anthropic.api_key_header_name(), Some("x-api-key"));
    assert!(!anthropic.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_claude_plan() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let claude_plan = providers
        .get(CLAUDE_PLAN_PROVIDER_ID)
        .expect("Claude Plan provider should be built in");
    assert!(claude_plan.is_claude_plan());
    assert_eq!(claude_plan.base_url.as_deref(), Some(ANTHROPIC_BASE_URL));
    assert!(claude_plan.env_key.is_none());
    assert!(claude_plan.auth.is_some());
    assert_eq!(claude_plan.wire_api, WireApi::Anthropic);
    assert_eq!(
        claude_plan
            .http_headers
            .as_ref()
            .and_then(|headers| headers.get("anthropic-beta"))
            .map(String::as_str),
        Some("claude-code-20250219,oauth-2025-04-20")
    );
    assert!(!claude_plan.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_zai() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    assert_eq!(
        providers
            .get(ZAI_PROVIDER_ID)
            .map(ModelProviderInfo::is_zai),
        Some(true)
    );
}

#[test]
fn test_built_in_model_providers_include_zai_anthropic() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let zai_anthropic = providers
        .get(ZAI_ANTHROPIC_PROVIDER_ID)
        .expect("Z.AI Anthropic provider should be built in");
    assert_eq!(
        zai_anthropic.base_url.as_deref(),
        Some(ZAI_ANTHROPIC_BASE_URL)
    );
    assert_eq!(zai_anthropic.env_key.as_deref(), Some(ZAI_API_KEY_ENV_VAR));
    assert_eq!(zai_anthropic.wire_api, WireApi::Anthropic);
    assert!(!zai_anthropic.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_openrouter() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let openrouter = providers
        .get(OPENROUTER_PROVIDER_ID)
        .expect("OpenRouter provider should be built in");
    assert!(openrouter.is_openrouter());
    assert_eq!(openrouter.base_url.as_deref(), Some(OPENROUTER_BASE_URL));
    assert_eq!(
        openrouter.env_key.as_deref(),
        Some(OPENROUTER_API_KEY_ENV_VAR)
    );
    assert_eq!(openrouter.wire_api, WireApi::Chat);
    assert_eq!(
        openrouter.stream_idle_timeout(),
        Duration::from_millis(600_000)
    );
    assert!(!openrouter.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_deepseek_flash_responses() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let deepseek = providers
        .get(DEEPSEEK_PROVIDER_ID)
        .expect("DeepSeek provider should be built in");

    assert!(deepseek.is_deepseek());
    assert_eq!(deepseek.base_url.as_deref(), Some(DEEPSEEK_BASE_URL));
    assert_eq!(deepseek.env_key.as_deref(), Some(DEEPSEEK_API_KEY_ENV_VAR));
    assert_eq!(deepseek.wire_api, WireApi::Responses);
    assert_eq!(deepseek.api_key_header_name(), None);
    assert!(!deepseek.requires_openai_auth);
    assert!(!deepseek.supports_websockets);
    assert_eq!(
        resolve_model_for_provider(/*model*/ None, DEEPSEEK_PROVIDER_ID).as_deref(),
        Some(DEEPSEEK_DEFAULT_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(
            Some(DEEPSEEK_DEFAULT_MODEL.to_string()),
            DEEPSEEK_PROVIDER_ID,
        )
        .as_deref(),
        Some(DEEPSEEK_DEFAULT_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(Some(DEEPSEEK_PRO_MODEL.to_string()), DEEPSEEK_PROVIDER_ID,)
            .as_deref(),
        Some(DEEPSEEK_PRO_MODEL),
        "DeepSeek Pro should remain on the authenticated direct provider"
    );
}

#[test]
fn test_built_in_model_providers_include_meta() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let meta = providers
        .get(META_PROVIDER_ID)
        .expect("Meta provider should be built in");

    assert_eq!(meta.base_url.as_deref(), Some(META_BASE_URL));
    assert_eq!(meta.env_key.as_deref(), Some(META_API_KEY_ENV_VAR));
    assert_eq!(meta.wire_api, WireApi::Responses);
    assert!(meta.is_meta());
    assert!(!meta.requires_openai_auth);
    assert!(!meta.supports_websockets);
    assert_eq!(
        resolve_model_for_provider(/*model*/ None, META_PROVIDER_ID).as_deref(),
        Some(META_DEFAULT_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(Some(META_DEFAULT_MODEL.to_string()), META_PROVIDER_ID)
            .as_deref(),
        Some(META_DEFAULT_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(Some("gpt-5.6-sol".to_string()), META_PROVIDER_ID).as_deref(),
        Some(META_DEFAULT_MODEL)
    );
}

#[test]
fn kimi_code_provider_is_builtin_and_resolves_k3() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let kimi_code = providers
        .get(KIMI_CODE_PROVIDER_ID)
        .expect("Kimi Code provider should be built in");

    assert_eq!(kimi_code.base_url.as_deref(), Some(KIMI_CODE_BASE_URL));
    assert_eq!(
        kimi_code.env_key.as_deref(),
        Some(KIMI_CODE_API_KEY_ENV_VAR)
    );
    assert_eq!(kimi_code.wire_api, WireApi::Chat);
    assert!(kimi_code.is_kimi_code());
    assert_eq!(
        resolve_model_for_provider(/*model*/ None, KIMI_CODE_PROVIDER_ID).as_deref(),
        Some(KIMI_CODE_K3_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(
            Some("moonshotai/kimi-k3".to_string()),
            KIMI_CODE_PROVIDER_ID,
        )
        .as_deref(),
        Some(KIMI_CODE_K3_MODEL)
    );
}

#[test]
fn openrouter_preserves_nonempty_model_slugs() {
    for provider in [OPENROUTER_PROVIDER_ID, OPENROUTER_ANTHROPIC_PROVIDER_ID] {
        for model in [
            "minimax/minimax-m3",
            "google/gemini-3.5-flash",
            OPENROUTER_GROK_4_6_MODEL,
            "x-ai/grok-4.5",
            OPENROUTER_DEEPSEEK_V4_PRO_0813_MODEL,
            "deepseek/deepseek-v4-pro",
            "deepseek/deepseek-v4-flash-0731",
            "tencent/hy3:free",
            "vendor/future-model",
        ] {
            assert_eq!(
                resolve_model_for_provider(Some(model.to_string()), provider).as_deref(),
                Some(model),
                "expected {provider} to preserve {model}"
            );
        }
        assert_eq!(
            resolve_model_for_provider(Some("  ".to_string()), provider).as_deref(),
            Some(OPENROUTER_DEFAULT_MODEL)
        );
        assert_eq!(
            resolve_model_for_provider(/*model*/ None, provider).as_deref(),
            Some(OPENROUTER_DEFAULT_MODEL)
        );
    }
}

#[test]
fn direct_zai_retries_transient_provider_rate_limits() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);
    let zai = providers
        .get(ZAI_PROVIDER_ID)
        .expect("Z.AI provider should be built in");
    let openrouter = providers
        .get(OPENROUTER_PROVIDER_ID)
        .expect("OpenRouter provider should be built in");

    assert!(
        zai.to_api_provider(/*auth_mode*/ None)
            .expect("Z.AI should convert to API provider")
            .retry
            .retry_429
    );
    assert!(
        !openrouter
            .to_api_provider(/*auth_mode*/ None)
            .expect("OpenRouter should convert to API provider")
            .retry
            .retry_429
    );
}

#[test]
fn configured_built_in_provider_can_override_transport_knobs() {
    let configured = HashMap::from([(
        OPENROUTER_PROVIDER_ID.to_string(),
        ModelProviderInfo {
            base_url: Some("https://ignored.example/v1".to_string()),
            wire_api: WireApi::Responses,
            request_max_retries: Some(2),
            stream_max_retries: Some(3),
            stream_idle_timeout_ms: Some(900_000),
            stream_actionable_timeout_ms: Some(240_000),
            stream_long_failure_retry_threshold_ms: Some(90_000),
            stream_long_failure_max_retries: Some(0),
            websocket_connect_timeout_ms: Some(30_000),
            ..ModelProviderInfo::default()
        },
    )]);

    let merged = merge_configured_model_providers(
        built_in_model_providers(/*openai_base_url*/ None),
        configured,
    )
    .expect("merge should allow transport overrides");
    let openrouter = merged
        .get(OPENROUTER_PROVIDER_ID)
        .expect("OpenRouter provider should remain present");

    assert_eq!(openrouter.base_url.as_deref(), Some(OPENROUTER_BASE_URL));
    assert_eq!(openrouter.wire_api, WireApi::Chat);
    assert_eq!(openrouter.request_max_retries(), 2);
    assert_eq!(openrouter.stream_max_retries(), 3);
    assert_eq!(
        openrouter.stream_idle_timeout(),
        Duration::from_millis(900_000)
    );
    assert_eq!(
        openrouter.stream_actionable_timeout(),
        Duration::from_millis(240_000)
    );
    assert_eq!(
        openrouter.stream_long_failure_retry_threshold(),
        Duration::from_millis(90_000)
    );
    assert_eq!(openrouter.stream_long_failure_max_retries(), 0);
    assert_eq!(
        openrouter.websocket_connect_timeout(),
        Duration::from_millis(30_000)
    );
}

#[test]
fn test_built_in_model_providers_include_openrouter_anthropic() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let openrouter_anthropic = providers
        .get(OPENROUTER_ANTHROPIC_PROVIDER_ID)
        .expect("OpenRouter Anthropic provider should be built in");
    assert_eq!(
        openrouter_anthropic.base_url.as_deref(),
        Some(OPENROUTER_BASE_URL)
    );
    assert_eq!(
        openrouter_anthropic.env_key.as_deref(),
        Some(OPENROUTER_API_KEY_ENV_VAR)
    );
    assert_eq!(openrouter_anthropic.wire_api, WireApi::Anthropic);
    assert!(!openrouter_anthropic.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_baseten() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let baseten = providers
        .get(BASETEN_PROVIDER_ID)
        .expect("Baseten provider should be built in");
    assert!(baseten.is_baseten());
    assert_eq!(baseten.base_url.as_deref(), Some(BASETEN_BASE_URL));
    assert_eq!(baseten.env_key.as_deref(), Some(BASETEN_API_KEY_ENV_VAR));
    assert_eq!(baseten.wire_api, WireApi::Chat);
    assert!(!baseten.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_baseten_anthropic() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let baseten_anthropic = providers
        .get(BASETEN_ANTHROPIC_PROVIDER_ID)
        .expect("Baseten Anthropic provider should be built in");
    assert_eq!(
        baseten_anthropic.base_url.as_deref(),
        Some(BASETEN_BASE_URL)
    );
    assert_eq!(
        baseten_anthropic.env_key.as_deref(),
        Some(BASETEN_API_KEY_ENV_VAR)
    );
    assert_eq!(baseten_anthropic.wire_api, WireApi::Anthropic);
    assert!(!baseten_anthropic.requires_openai_auth);
}

#[test]
fn test_built_in_model_providers_include_vercel() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    let vercel = providers
        .get(VERCEL_PROVIDER_ID)
        .expect("Vercel provider should be built in");
    assert!(vercel.is_vercel());
    assert_eq!(vercel.base_url.as_deref(), Some(VERCEL_BASE_URL));
    assert_eq!(vercel.env_key.as_deref(), Some(VERCEL_API_KEY_ENV_VAR));
    assert_eq!(vercel.wire_api, WireApi::Responses);
    assert!(!vercel.requires_openai_auth);
    assert_eq!(VERCEL_DEFAULT_MODEL, "zai/glm-5.2");
    assert_eq!(VERCEL_GLM_5_2_FAST_MODEL, "zai/glm-5.2-fast");
}

#[test]
fn test_built_in_model_providers_include_vercel_anthropic() {
    let providers = built_in_model_providers(/*openai_base_url*/ None);

    for provider_id in [
        VERCEL_ANTHROPIC_PROVIDER_ID,
        VERCEL_ANTHROPIC_FAST_PROVIDER_ID,
    ] {
        let vercel_anthropic = providers
            .get(provider_id)
            .expect("Vercel Anthropic provider should be built in");
        assert_eq!(vercel_anthropic.base_url.as_deref(), Some(VERCEL_BASE_URL));
        assert_eq!(
            vercel_anthropic.env_key.as_deref(),
            Some(VERCEL_API_KEY_ENV_VAR)
        );
        assert_eq!(vercel_anthropic.wire_api, WireApi::Anthropic);
        assert!(!vercel_anthropic.requires_openai_auth);
    }
}

#[test]
fn test_merge_configured_model_providers_adds_custom_provider() {
    let custom_provider = ModelProviderInfo {
        name: "Custom".to_string(),
        base_url: Some("https://example.com/v1".to_string()),
        ..ModelProviderInfo::default()
    };
    let configured_model_providers =
        std::collections::HashMap::from([("custom".to_string(), custom_provider.clone())]);

    let mut expected = built_in_model_providers(/*openai_base_url*/ None);
    expected.insert("custom".to_string(), custom_provider);

    assert_eq!(
        merge_configured_model_providers(
            built_in_model_providers(/*openai_base_url*/ None),
            configured_model_providers,
        ),
        Ok(expected)
    );
}

#[test]
fn test_merge_configured_model_providers_applies_amazon_bedrock_profile_override() {
    let configured_model_providers = std::collections::HashMap::from([(
        AMAZON_BEDROCK_PROVIDER_ID.to_string(),
        ModelProviderInfo {
            aws: Some(ModelProviderAwsAuthInfo {
                profile: Some("codex-bedrock".to_string()),
                region: Some("us-west-2".to_string()),
            }),
            ..ModelProviderInfo::default()
        },
    )]);

    let mut expected = built_in_model_providers(/*openai_base_url*/ None);
    expected
        .get_mut(AMAZON_BEDROCK_PROVIDER_ID)
        .expect("Amazon Bedrock provider should be built in")
        .aws = Some(ModelProviderAwsAuthInfo {
        profile: Some("codex-bedrock".to_string()),
        region: Some("us-west-2".to_string()),
    });

    assert_eq!(
        merge_configured_model_providers(
            built_in_model_providers(/*openai_base_url*/ None),
            configured_model_providers,
        ),
        Ok(expected)
    );
}

#[test]
fn test_merge_configured_model_providers_applies_amazon_bedrock_transport_overrides() {
    let auth = provider_auth_for_test();
    let configured_model_providers = std::collections::HashMap::from([(
        AMAZON_BEDROCK_PROVIDER_ID.to_string(),
        ModelProviderInfo {
            base_url: Some("https://proxy.example.com/v1".to_string()),
            auth: Some(auth.clone()),
            aws: Some(ModelProviderAwsAuthInfo {
                profile: Some("codex-bedrock".to_string()),
                region: Some("us-west-2".to_string()),
            }),
            http_headers: Some(maplit::hashmap! {
                "x-example-header".to_string() => "value".to_string(),
            }),
            ..ModelProviderInfo::default()
        },
    )]);

    let mut expected = built_in_model_providers(/*openai_base_url*/ None);
    let expected_provider = expected
        .get_mut(AMAZON_BEDROCK_PROVIDER_ID)
        .expect("Amazon Bedrock provider should be built in");
    expected_provider.base_url = Some("https://proxy.example.com/v1".to_string());
    expected_provider.auth = Some(auth);
    expected_provider.aws = Some(ModelProviderAwsAuthInfo {
        profile: Some("codex-bedrock".to_string()),
        region: Some("us-west-2".to_string()),
    });
    expected_provider
        .http_headers
        .get_or_insert_default()
        .insert("x-example-header".to_string(), "value".to_string());

    assert_eq!(
        merge_configured_model_providers(
            built_in_model_providers(/*openai_base_url*/ None),
            configured_model_providers,
        ),
        Ok(expected)
    );
}

#[test]
fn test_merge_configured_model_providers_rejects_amazon_bedrock_non_default_fields() {
    let configured_model_providers = std::collections::HashMap::from([(
        AMAZON_BEDROCK_PROVIDER_ID.to_string(),
        ModelProviderInfo {
            name: "Custom Bedrock".to_string(),
            aws: Some(ModelProviderAwsAuthInfo {
                profile: Some("codex-bedrock".to_string()),
                region: None,
            }),
            ..ModelProviderInfo::default()
        },
    )]);

    assert_eq!(
        merge_configured_model_providers(
            built_in_model_providers(/*openai_base_url*/ None),
            configured_model_providers,
        ),
        Err(
            "model_providers.amazon-bedrock only supports changing `base_url`, `auth`, `http_headers`, `aws.profile`, and `aws.region`; other non-default provider fields are not supported"
                .to_string()
        )
    );
}

#[test]
fn test_merge_configured_model_providers_allows_amazon_bedrock_default_fields() {
    let configured_model_providers = std::collections::HashMap::from([(
        AMAZON_BEDROCK_PROVIDER_ID.to_string(),
        ModelProviderInfo {
            aws: Some(ModelProviderAwsAuthInfo {
                profile: None,
                region: None,
            }),
            wire_api: WireApi::Responses,
            ..ModelProviderInfo::default()
        },
    )]);

    assert_eq!(
        merge_configured_model_providers(
            built_in_model_providers(/*openai_base_url*/ None),
            configured_model_providers,
        ),
        Ok(built_in_model_providers(/*openai_base_url*/ None))
    );
}

#[test]
fn test_validate_provider_aws_rejects_conflicting_auth() {
    let provider = ModelProviderInfo {
        aws: Some(ModelProviderAwsAuthInfo {
            profile: None,
            region: None,
        }),
        env_key: Some("AWS_BEARER_TOKEN_BEDROCK".to_string()),
        supports_websockets: false,
        ..ModelProviderInfo::create_openai_provider(/*base_url*/ None)
    };

    assert_eq!(
        provider.validate(),
        Err("provider aws cannot be combined with env_key, requires_openai_auth".to_string())
    );
}

#[test]
fn test_validate_provider_aws_rejects_websockets() {
    let provider = ModelProviderInfo {
        aws: Some(ModelProviderAwsAuthInfo {
            profile: None,
            region: None,
        }),
        requires_openai_auth: false,
        supports_websockets: true,
        ..ModelProviderInfo::create_openai_provider(/*base_url*/ None)
    };

    assert_eq!(
        provider.validate(),
        Err("provider aws cannot be combined with supports_websockets".to_string())
    );
}

#[test]
fn test_deserialize_provider_auth_config_allows_zero_refresh_interval() {
    let base_dir = tempdir().unwrap();
    let provider_toml = r#"
name = "Corp"

[auth]
command = "./scripts/print-token"
refresh_interval_ms = 0
        "#;

    let provider: ModelProviderInfo = {
        let _guard = AbsolutePathBufGuard::new(base_dir.path());
        toml::from_str(provider_toml).unwrap()
    };

    let auth = provider.auth.expect("auth config should deserialize");
    assert_eq!(auth.refresh_interval_ms, 0);
    assert_eq!(auth.refresh_interval(), None);
}

#[test]
fn corrected_catalog_provider_fixes_impossible_pairs_only() {
    // Field incidents: fixed.
    assert_eq!(
        corrected_catalog_provider("zai/glm-5.2-fast", ZAI_PROVIDER_ID),
        Some(VERCEL_ANTHROPIC_FAST_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider("zai/glm-5.2-fast", AMBIENT_PROVIDER_ID),
        Some(VERCEL_ANTHROPIC_FAST_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider(CLAUDE_FABLE_5_PLAN_MODEL, AMBIENT_PROVIDER_ID),
        Some(CLAUDE_PLAN_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider(CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL, AMBIENT_PROVIDER_ID),
        Some(CLAUDE_PLAN_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider(CLAUDE_FABLE_5_MODEL, AMBIENT_PROVIDER_ID),
        Some(CLAUDE_PLAN_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider(CLAUDE_FABLE_5_MODEL, ANTHROPIC_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(ZAI_DEFAULT_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        Some(ZAI_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider("glm-5.3", AMBIENT_PROVIDER_ID),
        Some(ZAI_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider("gpt-5.5", CLAUDE_PLAN_PROVIDER_ID),
        Some(OPENAI_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider("gpt-5.5", AMBIENT_PROVIDER_ID),
        Some(OPENAI_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider(KIMI_CODE_K3_MODEL, OPENROUTER_PROVIDER_ID),
        Some(KIMI_CODE_PROVIDER_ID)
    );

    // Consistent pairs and legitimate family variants: untouched.
    assert_eq!(
        corrected_catalog_provider("zai/glm-5.2-fast", VERCEL_ANTHROPIC_FAST_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(VERCEL_DEFAULT_MODEL, VERCEL_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(VERCEL_DEFAULT_MODEL, VERCEL_ANTHROPIC_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(CLAUDE_FABLE_5_PLAN_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(CLAUDE_PLAN_LEGACY_OPUS_4_8_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(ZAI_DEFAULT_MODEL, ZAI_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(ZAI_DEFAULT_MODEL, ZAI_ANTHROPIC_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider("gpt-5.5", OPENAI_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(KIMI_CODE_K3_MODEL, KIMI_CODE_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(DEEPSEEK_DEFAULT_MODEL, OPENROUTER_PROVIDER_ID),
        Some(DEEPSEEK_PROVIDER_ID)
    );
    assert_eq!(
        corrected_catalog_provider(DEEPSEEK_DEFAULT_MODEL, DEEPSEEK_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider(DEEPSEEK_PRO_MODEL, OPENROUTER_PROVIDER_ID),
        Some(DEEPSEEK_PROVIDER_ID)
    );

    // Servable cross-provider pairs, unknown models, user-defined providers: untouched.
    assert_eq!(
        corrected_catalog_provider(AMBIENT_DEFAULT_MODEL, AMBIENT_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider("zai-org/GLM-5.1-FP8", AMBIENT_PROVIDER_ID),
        None
    );
    assert_eq!(
        corrected_catalog_provider("gpt-5.5", "my-azure-provider"),
        None
    );
    assert_eq!(
        corrected_catalog_provider(AMAZON_BEDROCK_GPT_5_5_MODEL_ID, AMBIENT_PROVIDER_ID),
        None
    );
    assert_eq!(corrected_catalog_provider("", AMBIENT_PROVIDER_ID), None);
    assert_eq!(corrected_catalog_provider("gpt-5.5", ""), None);
}

#[test]
fn canonical_catalog_provider_exposes_exact_picker_runtime_pairs() {
    for (model, expected_provider) in [
        (AMBIENT_DEFAULT_MODEL, AMBIENT_PROVIDER_ID),
        (AMBIENT_KIMI_K2_7_CODE_MODEL, AMBIENT_PROVIDER_ID),
        (KIMI_CODE_K3_MODEL, KIMI_CODE_PROVIDER_ID),
        (ZAI_DEFAULT_MODEL, ZAI_PROVIDER_ID),
        ("glm-5.3", ZAI_PROVIDER_ID),
        (CLAUDE_PLAN_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        (CLAUDE_FABLE_5_1_PLAN_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        (CLAUDE_FABLE_5_PLAN_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        (ANTHROPIC_DEFAULT_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        (CLAUDE_FABLE_5_1_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        (CLAUDE_FABLE_5_MODEL, CLAUDE_PLAN_PROVIDER_ID),
        (OPENROUTER_GROK_4_6_MODEL, OPENROUTER_PROVIDER_ID),
        ("x-ai/grok-4.5", OPENROUTER_PROVIDER_ID),
        ("moonshotai/kimi-k3", OPENROUTER_PROVIDER_ID),
        (DEEPSEEK_DEFAULT_MODEL, DEEPSEEK_PROVIDER_ID),
        (DEEPSEEK_PRO_MODEL, DEEPSEEK_PROVIDER_ID),
        (
            OPENROUTER_DEEPSEEK_V4_PRO_0813_MODEL,
            OPENROUTER_PROVIDER_ID,
        ),
        (META_DEFAULT_MODEL, META_PROVIDER_ID),
        (VERCEL_DEFAULT_MODEL, VERCEL_PROVIDER_ID),
        (VERCEL_GLM_5_2_FAST_MODEL, VERCEL_ANTHROPIC_FAST_PROVIDER_ID),
        (BASETEN_DEFAULT_MODEL, BASETEN_PROVIDER_ID),
        ("gpt-5.6-sol", OPENAI_PROVIDER_ID),
    ] {
        assert_eq!(
            canonical_catalog_provider(model),
            Some(expected_provider),
            "unexpected canonical provider for {model}"
        );
    }
    assert_eq!(canonical_catalog_provider(""), None);
    assert_eq!(canonical_catalog_provider("private/custom-model"), None);
}

#[test]
fn fable_5_1_resolves_on_anthropic_routes() {
    assert_eq!(
        resolve_model_for_provider(
            Some(CLAUDE_FABLE_5_1_MODEL.to_string()),
            ANTHROPIC_PROVIDER_ID
        )
        .as_deref(),
        Some(CLAUDE_FABLE_5_1_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(
            Some(CLAUDE_FABLE_5_1_MODEL.to_string()),
            CLAUDE_PLAN_PROVIDER_ID
        )
        .as_deref(),
        Some(CLAUDE_FABLE_5_1_PLAN_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(
            Some(CLAUDE_FABLE_5_1_MODEL.to_string()),
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID
        )
        .as_deref(),
        Some(CLAUDE_FABLE_5_1_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(/*model*/ None, PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID)
            .as_deref(),
        Some(CLAUDE_FABLE_5_MODEL)
    );
    assert_eq!(
        resolve_model_for_provider(
            Some("unsupported-model".to_string()),
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID
        )
        .as_deref(),
        Some(CLAUDE_FABLE_5_MODEL)
    );
}

#[test]
fn zai_glm_5_3_resolves_only_on_the_direct_zai_route() {
    assert_eq!(canonical_catalog_provider("glm-5.3"), Some(ZAI_PROVIDER_ID));
    assert_eq!(
        resolve_model_for_provider(Some("glm-5.3".to_string()), ZAI_PROVIDER_ID).as_deref(),
        Some("glm-5.3")
    );
    assert_eq!(
        resolve_model_for_provider(Some("glm-5.3".to_string()), AMBIENT_PROVIDER_ID).as_deref(),
        Some(AMBIENT_DEFAULT_MODEL)
    );
}
