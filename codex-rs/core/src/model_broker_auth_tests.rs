use super::*;
use pretty_assertions::assert_eq;

use ProviderApiKeyHeader as ModelAuthHeader;

fn binding(host: &str, port: u16, path_prefix: &str, header: ModelAuthHeader) -> Binding {
    Binding {
        host: host.to_string(),
        port,
        path_prefix: path_prefix.to_string(),
        header,
    }
}

#[test]
fn pf_27_s05_base_url_binds_origin_and_path_prefix() {
    assert_eq!(
        binding_for_base_url("https://api.openai.com/v1", ProviderApiKeyHeader::Bearer),
        Some(binding(
            "api.openai.com",
            443,
            "/v1",
            ModelAuthHeader::Bearer
        ))
    );
    assert_eq!(
        binding_for_base_url(
            "https://API.Z.AI:8443/api/coding/paas/v4/",
            ProviderApiKeyHeader::Bearer
        ),
        Some(binding(
            "api.z.ai",
            8443,
            "/api/coding/paas/v4",
            ModelAuthHeader::Bearer
        ))
    );
    assert_eq!(
        binding_for_base_url("https://api.anthropic.com", ProviderApiKeyHeader::XApiKey),
        Some(binding(
            "api.anthropic.com",
            443,
            "/",
            ModelAuthHeader::XApiKey
        ))
    );
    // Plain HTTP, IPv6 literals, queries and garbage are not brokered.
    for url in [
        "http://localhost:11434/v1",
        "https://[::1]/v1",
        "https://api.example.com/v1?key=1",
        "not a url",
    ] {
        assert_eq!(
            binding_for_base_url(url, ProviderApiKeyHeader::Bearer),
            None,
            "{url}"
        );
    }
}

#[test]
fn pf_27_s05_requests_are_rewritten_to_plain_http_for_the_broker() {
    assert_eq!(
        BrokerRewrite::for_url("https://api.openai.com/v1/responses?stream=true"),
        Some(BrokerRewrite {
            host: "api.openai.com".to_string(),
            port: 443,
            path_and_query: "/v1/responses?stream=true".to_string(),
            broker_url: "http://api.openai.com:443/v1/responses?stream=true".to_string(),
        })
    );
    assert_eq!(
        BrokerRewrite::for_url("https://api.z.ai:8443/v4/chat/completions"),
        Some(BrokerRewrite {
            host: "api.z.ai".to_string(),
            port: 8443,
            path_and_query: "/v4/chat/completions".to_string(),
            broker_url: "http://api.z.ai:8443/v4/chat/completions".to_string(),
        })
    );
    for url in [
        "http://api.openai.com/v1/responses",
        "https://[::1]/v1",
        "https://a.example/v1#x",
    ] {
        assert_eq!(BrokerRewrite::for_url(url), None, "{url}");
    }
}

fn brokered_request(base_url: &str, source: BrokeredKeySource) -> BrokeredAuthRequest {
    BrokeredAuthRequest {
        base_url: base_url.to_string(),
        header: ProviderApiKeyHeader::Bearer,
        source,
        extra_headers: HeaderMap::new(),
    }
}

fn provider_key() -> BrokeredKeySource {
    BrokeredKeySource::ProviderKey {
        account: None,
        provider_key_id: "ZAI_API_KEY".to_string(),
        env_vars: vec!["ZAI_API_KEY".to_string()],
    }
}

fn held_value() -> BrokeredKeySource {
    BrokeredKeySource::Value {
        key: codex_model_provider::ProviderApiKey {
            value: "sk-pf27s05-held".to_string(),
            header: ProviderApiKeyHeader::Bearer,
        },
        slot: None,
    }
}

/// The error a brokered credential use fails with; it ends the turn.
fn refusal(broker: &CoreModelKeyBroker, base_url: &str, source: BrokeredKeySource) -> String {
    match broker.auth(brokered_request(base_url, source)) {
        Ok(_) => panic!("no auth may be produced"),
        Err(error) => {
            assert!(!error.is_retryable(), "{error:?}");
            let message = error.to_string();
            assert!(message.starts_with("Fatal error: "), "{message}");
            message
        }
    }
}

/// PF-27-S05 / PF-27-S06: the branch platforms without the broker take
/// (`start` returns this broker off Unix). Every credential use fails; none
/// is attached directly.
#[test]
fn pf_27_s05_platform_without_broker_refuses_every_credential() {
    let broker = CoreModelKeyBroker::unsupported();
    for source in [provider_key(), held_value()] {
        let error = refusal(&broker, "https://api.z.ai/api/paas/v4", source);
        assert!(error.contains("not available on this platform"), "{error}");
    }
}

#[cfg(not(any(unix, windows)))]
#[test]
fn pf_27_s05_non_unix_start_is_the_refusing_broker() {
    let broker = CoreModelKeyBroker::start(BrokerSettings {
        runtime_dir: std::env::temp_dir(),
        scrub_responses: false,
        program: None,
        store_home: std::env::temp_dir(),
        env_names: Vec::new(),
    });
    let error = refusal(&broker, "https://api.z.ai/api/paas/v4", held_value());
    assert!(error.contains("not available on this platform"), "{error}");
}

/// PF-27-S09: on Windows a broker that cannot start fails every credential
/// use (nothing is sent directly), and the provider keys it would have been
/// handed are still removed from Core's environment.
#[cfg(windows)]
#[test]
fn pf_27_s09_windows_broker_that_cannot_start_fails_closed_and_scrubs_keys() {
    let name = format!("PF27_S09_CORE_KEY_{}", std::process::id());
    // SAFETY: a variable unique to this test; nothing else reads it.
    unsafe { std::env::set_var(&name, "synthetic-pf27s09-core-key") };
    let home = tempfile::tempdir().expect("home");
    let broker = CoreModelKeyBroker::start(BrokerSettings {
        runtime_dir: home.path().join("run"),
        scrub_responses: false,
        program: Some(home.path().join("missing-corbanu.exe")),
        store_home: home.path().to_path_buf(),
        env_names: vec![name.clone()],
    });
    assert_eq!(std::env::var_os(&name), None);
    for source in [provider_key(), held_value()] {
        let error = refusal(&broker, "https://api.z.ai/api/paas/v4", source);
        assert!(error.contains("unavailable"), "{error}");
    }
}

#[test]
fn pf_27_s05_failed_broker_and_unbindable_urls_fail_closed() {
    let failed = CoreModelKeyBroker::new(BrokerHandle::Failed(BrokerModelAuthError::Unavailable));
    let error = refusal(&failed, "https://api.z.ai/api/paas/v4", provider_key());
    assert!(error.contains("unavailable"), "{error}");

    // A URL the broker cannot bind is refused before any broker call.
    for base_url in ["http://localhost:11434/v1", "https://[::1]/v1"] {
        let error = refusal(&CoreModelKeyBroker::unsupported(), base_url, held_value());
        assert!(error.contains("cannot be brokered"), "{base_url}: {error}");
    }
}

/// PF-27-S09: what the Windows sender puts on the pipe for a rewritten URL.
#[test]
fn pf_27_s09_pipe_request_target_keeps_the_signed_path_and_query() {
    assert_eq!(
        pipe_request_target("http://api.z.ai:443/api/paas/v4/chat/completions?x=1&y=2"),
        Some((
            "api.z.ai:443".to_string(),
            "/api/paas/v4/chat/completions?x=1&y=2".to_string()
        ))
    );
    // The broker URL is what `broker_request` produced for this request.
    let rewrite =
        BrokerRewrite::for_url("https://api.z.ai/api/paas/v4/models?page=2").expect("rewrite");
    assert_eq!(
        pipe_request_target(&rewrite.broker_url),
        Some(("api.z.ai:443".to_string(), rewrite.path_and_query))
    );
    for url in [
        "https://api.z.ai:443/v1",
        "http://user:secret@api.z.ai:443/v1",
        "http://user@api.z.ai:443/v1",
        "http://api.z.ai:443/v1#fragment",
        "not a url",
    ] {
        assert_eq!(pipe_request_target(url), None, "{url}");
    }
}

/// #390: a broker refused because another process holds or serves its named
/// pipe fails every credential use with a message that says so; nothing is
/// sent directly.
#[test]
fn sec_390_squatted_broker_pipe_fails_closed_with_a_clear_error() {
    let squatted =
        CoreModelKeyBroker::new(BrokerHandle::Failed(BrokerModelAuthError::PipeSquatted));
    for source in [provider_key(), held_value()] {
        let error = refusal(&squatted, "https://api.z.ai/api/paas/v4", source);
        assert!(
            error.contains("held or served by another process"),
            "{error}"
        );
        assert!(error.contains("sent nothing"), "{error}");
    }
}
