//! Collaboration wire compatibility, independent of provider identity and authentication.
use crate::session::turn_context::TurnContext;
use codex_model_provider_info::OPENAI_PROVIDER_ID;

/// The collaboration wire contract follows the endpoint, not a custom display name.
/// Keep this narrower than provider identity: it must not grant OpenAI auth or capabilities.
pub(super) fn uses_openai_reserved_collaboration_schema(turn_context: &TurnContext) -> bool {
    let provider = turn_context.provider.info();
    if provider.is_openai() {
        return true;
    }
    provider.wire_api == codex_model_provider_info::WireApi::Responses
        && provider.base_url.as_deref().is_some_and(|base_url| {
            url::Url::parse(base_url).is_ok_and(|url| {
                url.scheme() == "https"
                    && url.host_str() == Some("api.openai.com")
                    && url.port_or_known_default() == Some(443)
                    && url.path().trim_end_matches('/') == "/v1"
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.query().is_none()
                    && url.fragment().is_none()
            })
        })
}

/// Select the existing message codec without changing the configured provider or account.
/// Recipients keep their existing fail-closed identity check and can use plaintext adapters.
pub(super) fn message_source_provider_id(turn_context: &TurnContext) -> &str {
    if uses_openai_reserved_collaboration_schema(turn_context) {
        OPENAI_PROVIDER_ID
    } else {
        &turn_context.config.model_provider_id
    }
}
