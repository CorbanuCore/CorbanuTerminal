//! #898: `/cost` shows a `/review` code review's requests under the
//! conversation that ran it, labelled `code-review:`, counted once in its
//! total. The ledger is written by a real core session against a mocked
//! Responses server; the pages are the ones `/cost` renders from it.
use super::*;
use codex_features::Feature;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::ReviewTarget;
use codex_state::accounting::AccountingStore;
use codex_state::accounting::Inspection;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use wiremock::MockServer;

fn completed(id: &str, message: &str) -> String {
    responses::sse(vec![
        responses::ev_response_created(id),
        responses::ev_assistant_message(&format!("{id}-message"), message),
        serde_json::json!({"type": "response.completed", "response": {"id": id, "usage": {
            "input_tokens": 100, "input_tokens_details": {"cached_tokens": 20, "cache_write_tokens": 0},
            "output_tokens": 40, "output_tokens_details": {"reasoning_tokens": 10},
            "total_tokens": 140
        }}}),
    ])
}

/// Day inspections of the conversation, read until `ready` holds or a
/// deadline passes: the ledger is written by the collector, after the events.
async fn inspect_until(
    db: &codex_state::StateRuntime,
    owner: ThreadId,
    ready: impl Fn(&Inspection) -> bool,
) -> anyhow::Result<InspectionDay> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let now = chrono::Utc::now().timestamp_millis();
        let day = AccountingStore::inspect_day(db, owner, now / 86_400_000, now).await?;
        if matches!(&day, InspectionDay::Ready(view) if ready(view))
            || tokio::time::Instant::now() > deadline
        {
            return Ok(day);
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accounting_cost_shows_code_review_under_the_conversation() -> anyhow::Result<()> {
    core_test_support::skip_if_no_network!(Ok(()));
    let server = MockServer::start().await;
    let endpoint = format!("{}/v1", server.uri());
    let review = serde_json::json!({
        "findings": [],
        "overall_correctness": "good",
        "overall_explanation": "fixture review",
        "overall_confidence_score": 0.5
    })
    .to_string();
    responses::mount_sse_sequence(
        &server,
        vec![
            completed("main-response", "main done"),
            completed("review-response", &review),
        ],
    )
    .await;
    let test = core_test_support::test_codex::test_codex()
        .with_auth(CodexAuth::from_api_key("synthetic-code-review-accounting"))
        .with_direct_responses_accounting(endpoint.clone())
        .with_config(move |config| {
            config.model = Some("gpt-5.6-terra".into());
            config.model_provider_id = "openai".into();
            config.model_provider = ModelProviderInfo {
                request_max_retries: Some(1),
                stream_max_retries: Some(1),
                stream_idle_timeout_ms: Some(2000),
                ..ModelProviderInfo::create_openai_provider(Some(endpoint))
            };
            config.model_catalog =
                Some(codex_models_manager::bundled_models_response().expect("bundled catalogue"));
            config
                .features
                .enable(Feature::Sqlite)
                .expect("sqlite feature");
        })
        .build_with_auto_env(&server)
        .await?;
    test.submit_turn("main conversation").await?;
    test.codex
        .submit(Op::Review {
            review_request: ReviewRequest {
                target: ReviewTarget::Custom {
                    instructions: "Review the fixture".to_string(),
                },
                user_facing_hint: None,
            },
        })
        .await?;
    core_test_support::wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::ExitedReviewMode(_))
    })
    .await;
    let db = test.codex.state_db().expect("state database");
    let owner = test.session_configured.thread_id;
    let day = inspect_until(&db, owner, |view| {
        view.totals.attempts + view.unknown_parent_totals.attempts >= 2
            && view
                .requests
                .values()
                .chain(view.unknown_parent_requests.values())
                .flatten()
                .all(|quote| quote.usage.output.is_some())
    })
    .await?;
    test.thread_manager
        .shutdown_all_threads_bounded(std::time::Duration::from_secs(3))
        .await;

    let pages = inspection_pages(Ok(day));
    // The first screen: both requests in this conversation's total, nothing
    // left outside it.
    let first: Vec<&str> = pages[0]
        .text
        .iter()
        .map(String::as_str)
        .filter(|line| {
            line.starts_with("• OpenAI")
                || line.starts_with("Estimated cost")
                || line.starts_with("Unknown parent population")
                || line.contains("other conversation")
        })
        .collect();
    insta::assert_snapshot!(first.join("\n"), @r"
    • OpenAI · GPT-5.6-Terra — Pay per use. 2 requests, 280 tokens. Estimated cost: $0.001288.
    No other conversation recorded requests on this day.
    Unknown parent population: 0 attempts, excluded from root total
    ");
    // Each request page names its turn: the conversation's own and the
    // review's, labelled `code-review:`.
    let kinds: std::collections::BTreeSet<String> = pages
        .iter()
        .flat_map(|page| page.text.iter())
        .filter_map(|line| line.strip_prefix("Turn: "))
        .map(|turn| {
            turn.rsplit_once(':')
                .map_or_else(String::new, |(kind, _)| format!("{kind}:"))
        })
        .collect();
    assert_eq!(
        kinds.into_iter().collect::<Vec<_>>(),
        vec![String::new(), "code-review:".to_string()]
    );
    Ok(())
}
