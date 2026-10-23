use super::*;
use pretty_assertions::assert_eq;

#[test]
fn remote_gateway_requires_https() {
    let result = gateway_client_for_origin("http://gateway.example.test".to_string());
    assert!(result.is_err());
}

#[test]
fn loopback_gateway_allows_http() {
    let result = gateway_client_for_origin("http://127.0.0.1:4021".to_string());
    assert!(result.is_ok());
}

fn no_stored_key() -> Result<Option<String>, String> {
    Ok(None)
}

/// #641: under Aggressive the credential broker takes the Corbanu API key out
/// of the environment. Account calls then use the key the broker holds (it was
/// the environment's, so it still comes before the stored key) instead of
/// finding none.
#[test]
fn a_corbanu_key_held_by_the_broker_is_used_for_account_calls() {
    let held = account_key_from(
        /*environment*/ None,
        /*held_by_broker*/ true,
        || Ok(Some("stored-key".to_string())),
    );
    assert!(matches!(held, Ok(Some(CorbanuAccountKey::HeldByBroker))));

    let environment = account_key_from(
        Some("env-key".to_string()),
        /*held_by_broker*/ true,
        no_stored_key,
    );
    assert!(matches!(
        environment,
        Ok(Some(CorbanuAccountKey::Value(key))) if key.as_str() == "env-key"
    ));
    let stored = account_key_from(
        /*environment*/ None,
        /*held_by_broker*/ false,
        || Ok(Some("stored-key".to_string())),
    );
    assert!(matches!(
        stored,
        Ok(Some(CorbanuAccountKey::Value(key))) if key.as_str() == "stored-key"
    ));
    assert!(matches!(
        account_key_from(
            /*environment*/ None,
            /*held_by_broker*/ false,
            no_stored_key
        ),
        Ok(None)
    ));
}

/// #641: the broker attaches the held key only for the gateway's `/v1` (as
/// for the Corbanu API model provider), as a bearer token, from the Corbanu
/// key variables it was handed (else the stored key, read by the broker).
#[test]
fn the_held_corbanu_key_is_bound_to_the_gateway_api() {
    let request = brokered_account_auth("https://api.corbanu.com");
    assert_eq!(request.base_url, "https://api.corbanu.com/v1");
    assert!(matches!(
        request.header,
        codex_model_provider::ProviderApiKeyHeader::Bearer
    ));
    match request.source {
        codex_model_provider::BrokeredKeySource::ProviderKey {
            provider_key_id,
            env_vars,
            account,
        } => {
            assert_eq!(
                provider_key_id,
                codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR
            );
            assert_eq!(
                env_vars,
                codex_model_provider_info::CORBANU_API_KEY_ENV_VARS
                    .iter()
                    .map(|name| (*name).to_string())
                    .collect::<Vec<_>>()
            );
            assert!(account.is_none());
        }
        other => panic!("unexpected source {other:?}"),
    }
    assert!(request.extra_headers.is_empty());
}

/// #641: without a running broker the call fails; the key is never sent
/// directly.
#[tokio::test]
async fn a_held_key_is_never_sent_without_the_broker() {
    let gateway = gateway_client_for_origin("http://127.0.0.1:9".to_string()).expect("client");
    let result = account_get(&gateway, "/v1/account", &CorbanuAccountKey::HeldByBroker).await;
    let Err(error) = result else {
        panic!("a held key must not be sent without the broker");
    };
    assert!(error.contains("broker"), "{error}");
}
