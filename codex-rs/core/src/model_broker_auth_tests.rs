use super::*;
use pretty_assertions::assert_eq;

fn binding(
    host: &str,
    port: u16,
    path_prefix: &str,
    header: ModelAuthHeader,
) -> ModelCredentialBinding {
    ModelCredentialBinding {
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
