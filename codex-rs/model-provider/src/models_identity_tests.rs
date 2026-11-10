//! Provider and authentication dimensions that partition model catalogs.

use super::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use pretty_assertions::assert_eq;

fn chatgpt_auth(email: &str, user: &str, account: &str, plan: &str, signature: &str) -> CodexAuth {
    let claims = serde_json::json!({
        "email": email,
        "https://api.openai.com/auth": {"chatgpt_user_id": user}
    });
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap());
    CodexAuth::from_external_chatgpt_tokens(
        &format!("e30.{payload}.{signature}"),
        account,
        Some(plan),
    )
    .unwrap()
}

#[test]
fn cache_identity_tracks_account_email_user_plan_and_auth_mode_but_not_token_refresh() {
    let mut provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
    // The endpoint residency test temporarily sets the process default to US.
    provider.http_headers = Some(std::collections::HashMap::from([(
        codex_login::default_client::RESIDENCY_HEADER_NAME.to_string(),
        "us".into(),
    )]));
    let initial = chatgpt_auth("one@example.com", "user", "account", "team", "first");
    let key = identity(&provider, Some(&initial)).unwrap();
    let refreshed = chatgpt_auth("one@example.com", "user", "account", "team", "second");
    assert_eq!(identity(&provider, Some(&refreshed)).unwrap(), key);
    for auth in [
        chatgpt_auth("two@example.com", "user", "account", "team", "first"),
        chatgpt_auth("one@example.com", "other", "account", "team", "first"),
        chatgpt_auth("one@example.com", "user", "other", "team", "first"),
        chatgpt_auth("one@example.com", "user", "account", "plus", "first"),
        CodexAuth::from_api_key("api-key"),
    ] {
        assert_ne!(identity(&provider, Some(&auth)).unwrap(), key);
    }
    assert_ne!(identity(&provider, /*auth*/ None).unwrap(), key);
}

#[test]
fn cache_identity_tracks_provider_routing_and_effective_api_credentials() {
    let auth = CodexAuth::from_api_key("sk-proj-fedramp-synthetic-test-key");
    let provider = ModelProviderInfo::create_openai_provider(Some("https://one.example/v1".into()));
    let key = identity(&provider, Some(&auth)).unwrap();
    let other_key = CodexAuth::from_api_key("sk-proj-fedramp-rotated-test-key");
    assert_ne!(identity(&provider, Some(&other_key)).unwrap(), key);
    let mut other_provider = provider.clone();
    other_provider.base_url = Some("https://two.example/v1".into());
    assert_ne!(identity(&other_provider, Some(&auth)).unwrap(), key);
    other_provider = provider.clone();
    other_provider.experimental_bearer_token = Some("provider-key".into());
    assert_ne!(identity(&other_provider, Some(&auth)).unwrap(), key);
    other_provider = provider;
    other_provider.http_headers = Some(std::collections::HashMap::from([(
        "openai-project".into(),
        "another-project".into(),
    )]));
    assert_ne!(identity(&other_provider, Some(&auth)).unwrap(), key);

    let mut provider = ModelProviderInfo::create_openai_provider(/*base_url*/ None);
    // Keep identities stable while the endpoint residency test runs in parallel.
    provider.http_headers = Some(std::collections::HashMap::from([(
        codex_login::default_client::RESIDENCY_HEADER_NAME.to_string(),
        "us".into(),
    )]));
    let key = identity(&provider, Some(&auth)).unwrap();
    provider.base_url = Some("https://gov.api.openai.com/v1".into());
    assert_eq!(identity(&provider, Some(&auth)).unwrap(), key);
    provider.base_url = Some("https://api.openai.com/v1".into());
    assert_ne!(identity(&provider, Some(&auth)).unwrap(), key);
}

#[test]
fn cache_identity_distinguishes_auth_requirements_and_unknown_plans() {
    let mut provider =
        ModelProviderInfo::create_openai_provider(Some("https://example.com/v1".into()));
    let first = chatgpt_auth(
        "one@example.com",
        "user",
        "account",
        "future-plan-one",
        "first",
    );
    let second = chatgpt_auth(
        "one@example.com",
        "user",
        "account",
        "future-plan-two",
        "first",
    );
    let key = identity(&provider, Some(&first)).unwrap();
    assert_ne!(key, identity(&provider, Some(&second)).unwrap());
    provider.requires_openai_auth = false;
    assert_ne!(key, identity(&provider, Some(&first)).unwrap());
}

#[test]
fn cache_identity_tracks_catalog_url() {
    let auth = CodexAuth::from_api_key("test-key");
    let mut provider =
        ModelProviderInfo::create_openai_provider(Some("https://gateway.example/v1".into()));
    let bundled = identity(&provider, Some(&auth)).unwrap();
    provider.model_catalog_url = Some("https://gateway.example/catalog-one".into());
    let first = identity(&provider, Some(&auth)).unwrap();
    provider.model_catalog_url = Some("https://gateway.example/catalog-two".into());
    let second = identity(&provider, Some(&auth)).unwrap();
    assert_ne!(bundled, first);
    assert_ne!(first, second);
}

/// Corbanu: an env_key provider's identity comes from its resolved key (a
/// stored or named-account key), so it is computed without the environment
/// variable, and a different key is a different catalog.
#[test]
fn env_key_identity_uses_the_resolved_key_without_reading_the_environment() {
    let mut provider = ModelProviderInfo {
        env_key: Some("CORBANU_TEST_S1B2_UNSET_IDENTITY_KEY".to_string()),
        base_url: Some("https://provider.example/v1".to_string()),
        requires_openai_auth: false,
        ..ModelProviderInfo::default()
    };
    let first = CodexAuth::from_api_key("stored-key-one");
    let key = identity(&provider, Some(&first)).expect("identity without the env var");
    assert_ne!(
        identity(&provider, Some(&CodexAuth::from_api_key("stored-key-two"))).unwrap(),
        key
    );
    provider.account = Some(codex_model_provider_info::NamedProviderAccount {
        provider_id: "custom".to_string(),
        name: "work".to_string(),
    });
    // The same key value is the same credential, so the catalog is shared.
    identity(&provider, Some(&first)).expect("a named account's identity");
}

/// PF-27-S05: in a brokered process the identity names the key's source
/// (env_key and account) without asking the broker for headers (none is
/// installed here, so a broker call would fail), and different sources never
/// share a catalog. A rotated key under the same source keeps the catalog
/// until its cache expires (the broker gives Core no key fingerprint).
#[test]
fn brokered_env_key_identity_names_the_key_source_without_the_broker() {
    crate::model_key_broker::set_test_broker_required(/*required*/ true);
    let placeholder = CodexAuth::from_api_key(crate::model_key_broker::BROKERED_KEY_PLACEHOLDER);
    let provider = |env_key: &str, account: Option<&str>| ModelProviderInfo {
        env_key: Some(env_key.to_string()),
        base_url: Some("https://provider.example/v1".to_string()),
        requires_openai_auth: false,
        account: account.map(|name| codex_model_provider_info::NamedProviderAccount {
            provider_id: "custom".to_string(),
            name: name.to_string(),
        }),
        ..ModelProviderInfo::default()
    };
    let identities = [
        provider("KEY_ONE", None),
        provider("KEY_TWO", None),
        provider("KEY_ONE", Some("work")),
        provider("KEY_ONE", Some("home")),
    ]
    .iter()
    .map(|provider| identity(provider, Some(&placeholder)))
    .collect::<CoreResult<Vec<_>>>();
    crate::model_key_broker::set_test_broker_required(/*required*/ false);
    let identities = identities.expect("no broker call");
    let unique = identities.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), identities.len());
}
