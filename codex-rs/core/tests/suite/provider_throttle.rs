//! #476: a provider 429 holds later requests on the same key back locally for
//! the cooldown the provider asked for, in every session on the home, even
//! when the request held no lease (any human-driven session).
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_response_sequence;
use core_test_support::responses::sse;
use core_test_support::responses::sse_response;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use serde_json::json;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

fn rate_limited() -> ResponseTemplate {
    ResponseTemplate::new(429)
        .insert_header("retry-after", "120")
        .set_body_json(json!({
            "error": { "message": "Rate limit reached", "type": "requests", "code": "rate_limit_exceeded" }
        }))
}

/// Submits one turn and returns the message of the error it ended with.
async fn turn_error(test: &TestCodex) -> anyhow::Result<String> {
    test.codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "hello".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: Default::default(),
        })
        .await?;
    let EventMsg::Error(error) =
        wait_for_event(&test.codex, |event| matches!(event, EventMsg::Error(_))).await
    else {
        unreachable!("waited for an error");
    };
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    Ok(error.message)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_429_holds_later_requests_back_in_every_session() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    let mock = mount_response_sequence(&server, vec![rate_limited()]).await;

    let mut builder = test_codex();
    let first = builder.build(&server).await?;
    let error = turn_error(&first).await?;
    assert!(error.contains("rate limited by provider"), "{error}");
    assert_eq!(mock.requests().len(), 1);

    // The same session, and a new one on the same home, are held back
    // locally: no further request reaches the provider.
    let blocked = turn_error(&first).await?;
    assert!(
        blocked.contains("provider request blocked by local cooldown"),
        "{blocked}"
    );
    assert!(blocked.contains("Last status: 429"), "{blocked}");
    let second = test_codex()
        .with_home(first.home.clone())
        .build(&server)
        .await?;
    let blocked = turn_error(&second).await?;
    assert!(
        blocked.contains("provider request blocked by local cooldown"),
        "{blocked}"
    );
    // The cooldown is the 120 s the provider's Retry-After asked for.
    let wait_seconds: u64 = blocked
        .split("for about ")
        .nth(1)
        .and_then(|rest| rest.split('s').next())
        .and_then(|seconds| seconds.parse().ok())
        .unwrap_or_default();
    assert!((100..=120).contains(&wait_seconds), "{blocked}");
    assert_eq!(mock.requests().len(), 1);
    Ok(())
}

/// One provider's named accounts on a test home: `alpha` and `beta` hold the
/// same key, `gamma` another.
fn named_account_session(account: &'static str) -> core_test_support::test_codex::TestCodexBuilder {
    test_codex()
        .with_config(move |config| {
            let vault = codex_vault::Vault::new(config.codex_home.to_path_buf());
            for (name, key) in [
                ("alpha", "pf84-shared-fixture-key"),
                ("beta", "pf84-shared-fixture-key"),
                ("gamma", "pf84-other-fixture-key"),
            ] {
                vault
                    .write_provider_account(
                        "my-llm",
                        &codex_vault::ProviderAccountName::parse(name).expect("account name"),
                        codex_vault::ProviderAccountKind::ApiKey,
                        key,
                    )
                    .expect("enroll the account");
            }
            config.model_provider_id = "my-llm".into();
            config.model_provider.name = "My LLM".into();
            config.model_provider.requires_openai_auth = false;
            config.model_provider.env_key = Some("PF84_THROTTLE_FP_UNSET_KEY".into());
            config.model_provider.account = Some(codex_model_provider_info::NamedProviderAccount {
                provider_id: "my-llm".into(),
                name: account.into(),
            });
            config
                .features
                .enable(codex_features::Feature::NamedAccounts)
                .expect("enable named accounts");
        })
        .with_auth_in_home()
}

/// Owner decision 2026-10-11: providers limit by key, so named accounts are
/// throttled by their key's fingerprint. A 429 on `alpha` holds `beta` (same
/// key) back locally, and `gamma` (another key) keeps answering.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn named_accounts_on_one_key_share_a_cooldown() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    let mock = mount_response_sequence(
        &server,
        vec![
            rate_limited(),
            sse_response(sse(vec![
                ev_response_created("resp-gamma"),
                ev_assistant_message("msg-gamma", "pong"),
                ev_completed("resp-gamma"),
            ])),
        ],
    )
    .await;

    let alpha = named_account_session("alpha").build(&server).await?;
    let error = turn_error(&alpha).await?;
    assert!(error.contains("rate limited by provider"), "{error}");

    let beta = named_account_session("beta")
        .with_home(alpha.home.clone())
        .build(&server)
        .await?;
    let blocked = turn_error(&beta).await?;
    assert!(
        blocked.contains("provider request blocked by local cooldown"),
        "{blocked}"
    );
    // The key is the credential's salted fingerprint: no account name, no key.
    assert!(blocked.contains("(account-key:"), "{blocked}");
    for leaked in ["alpha", "beta", "pf84-shared"] {
        assert!(!blocked.contains(leaked), "{blocked}");
    }
    assert_eq!(mock.requests().len(), 1);

    let gamma = named_account_session("gamma")
        .with_home(alpha.home.clone())
        .build(&server)
        .await?;
    gamma.submit_turn("hello").await?;
    assert_eq!(mock.requests().len(), 2);
    Ok(())
}
