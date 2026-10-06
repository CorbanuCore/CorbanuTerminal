use super::*;
use crate::oauth::WrappedOAuthTokenResponse;
use codex_secret_broker::output_gate::OutputSink;
use oauth2::AccessToken;
use oauth2::RefreshToken;
use oauth2::basic::BasicTokenType;
use pretty_assertions::assert_eq;
use rmcp::transport::auth::OAuthTokenResponse;
use rmcp::transport::auth::VendorExtraTokenFields;

// Synthetic tokens only.
fn tokens(access: &str, refresh: &str) -> StoredOAuthTokens {
    let mut response = OAuthTokenResponse::new(
        AccessToken::new(access.to_string()),
        BasicTokenType::Bearer,
        VendorExtraTokenFields::default(),
    );
    response.set_refresh_token(Some(RefreshToken::new(refresh.to_string())));
    StoredOAuthTokens {
        server_name: "pf28s02-mcp".to_string(),
        url: "https://mcp.example.test".to_string(),
        client_id: "client".to_string(),
        token_response: WrappedOAuthTokenResponse(response),
        expires_at: None,
    }
}

fn scrub(text: &str) -> String {
    output_gate::scrub_if_armed(OutputSink::ToolResult, text).unwrap_or_else(|| text.to_string())
}

#[test]
fn pf_28_s02_refreshed_mcp_oauth_tokens_are_registered() {
    output_gate::global().arm().expect("arm");
    protect(&tokens(
        "pf28s02-mcp-access-0001",
        "pf28s02-mcp-refresh-0001",
    ))
    .expect("protect");
    assert_eq!(
        scrub("echo pf28s02-mcp-access-0001 pf28s02-mcp-refresh-0001"),
        "echo [REDACTED:mcp-oauth:pf28s02-mcp:access_token] \
         [REDACTED:mcp-oauth:pf28s02-mcp:refresh_token]"
    );

    // Two refreshes: the newest and the one before stay protected; the
    // first access token is retired.
    protect(&tokens(
        "pf28s02-mcp-access-0002",
        "pf28s02-mcp-refresh-0001",
    ))
    .expect("refresh");
    protect(&tokens(
        "pf28s02-mcp-access-0003",
        "pf28s02-mcp-refresh-0001",
    ))
    .expect("refresh");
    assert_eq!(
        scrub("pf28s02-mcp-access-0003 pf28s02-mcp-access-0002 pf28s02-mcp-access-0001"),
        "[REDACTED:mcp-oauth:pf28s02-mcp:access_token] \
         [REDACTED:mcp-oauth:pf28s02-mcp:access_token] pf28s02-mcp-access-0001"
    );
}
