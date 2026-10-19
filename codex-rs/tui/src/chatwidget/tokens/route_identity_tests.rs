//! Issue352: cost labels identify the route recorded for each request, even
//! when local providers share a transport name or catalogues reuse a slug.

use super::*;

fn route_day(routes: &[(&str, &str, Option<&str>)]) -> InspectionDay {
    let InspectionDay::Ready(mut view) = packet() else {
        unreachable!()
    };
    view.requests.clear();
    for (index, &(provider, model, account)) in routes.iter().enumerate() {
        let mut q = quote();
        q.attempt.attempt_id = Uuid::from_u128(index as u128 + 10);
        q.attempt.request_id = Uuid::from_u128(index as u128 + 100);
        q.attempt.retry_of = None;
        q.attempt.provider = provider.into();
        q.attempt.model = model.into();
        q.attempt.account = account.map(|name| codex_state::accounting::AttemptAccount {
            name: name.to_string(),
            fingerprint: None,
        });
        if matches!(provider, "ollama" | "lmstudio") {
            let mut snapshot = native_snapshot(
                serde_json::json!({}),
                "Local",
                /*basis_source*/ None,
                /*plan_burn_millis*/ None,
            );
            snapshot.provider = provider.into();
            snapshot.model = model.into();
            q.snapshot = Some(snapshot);
            q.buckets = [BucketQuote::MissingRate; 4];
            q.known_subtotal = Decimal::default();
            q.all_buckets_priced = None;
            q.subtotal_display = Decimal::default().display();
        }
        view.requests.insert(q.attempt.request_id, vec![q]);
    }
    view.totals = DayTotals::from_quotes(view.requests.values().flatten()).unwrap();
    view.own_totals = view.totals.clone();
    InspectionDay::Ready(view)
}

fn overview_routes(pages: &[InspectorPage]) -> String {
    pages[0]
        .text
        .iter()
        .filter(|line| line.starts_with("• "))
        .map(|line| line.split(" — ").next().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn local_providers_keep_distinct_identities_in_the_cost_view() {
    let pages = inspection_pages(Ok(route_day(&[
        ("ollama", "mock-model", None),
        ("lmstudio", "mock-model", None),
    ])));
    insta::assert_snapshot!(overview_routes(&pages), @"
    • LM Studio · mock-model
    • Ollama · mock-model
    ");
    for provider in ["Ollama", "LM Studio"] {
        assert!(pages.iter().any(|page| {
            page.title == "Request"
                && page.text.contains(&format!("Provider: {provider}"))
                && page.text.contains(&"Model: mock-model".to_string())
        }));
    }
}

#[test]
fn shared_model_slug_does_not_borrow_another_providers_brand() {
    let pages = inspection_pages(Ok(route_day(&[
        ("openrouter", "z-ai/glm-5.2", None),
        ("ambient", "z-ai/glm-5.2", None),
        ("custom-route", "z-ai/glm-5.2", None),
    ])));
    insta::assert_snapshot!(overview_routes(&pages), @"
    • Ambient · Ambient GLM 5.2
    • custom-route · z-ai/glm-5.2
    • OpenRouter · z-ai/glm-5.2
    ");
    for page in &pages {
        if page.text.contains(&"Provider: OpenRouter".to_string()) {
            assert!(page.text.contains(&"Model: z-ai/glm-5.2".to_string()));
        }
    }
}

#[test]
fn matching_catalogue_routes_and_unknown_models_remain_identifiable() {
    let pages = inspection_pages(Ok(route_day(&[
        ("openai", "gpt-6-sol", None),
        ("openrouter", "moonshotai/kimi-k3", None),
        ("openrouter", "vendor/not-in-catalogue", None),
        ("custom-route", "", None),
    ])));
    insta::assert_snapshot!(overview_routes(&pages), @"
    • custom-route · unknown model
    • OpenAI · GPT-6 Sol
    • OpenRouter · OpenRouter Kimi K3
    • OpenRouter · vendor/not-in-catalogue
    ");
}

#[test]
fn named_account_pages_keep_the_recorded_provider_and_model() {
    let pages = inspection_pages(Ok(route_day(&[
        ("openrouter", "z-ai/glm-5.2", None),
        ("openrouter", "z-ai/glm-5.2", Some("work")),
    ])));
    insta::assert_snapshot!(overview_routes(&pages), @"
    • OpenRouter · account default · z-ai/glm-5.2
    • OpenRouter · account work · z-ai/glm-5.2
    ");
    for account in ["default", "work"] {
        assert!(pages.iter().any(|page| {
            page.title == "Request"
                && page.text.contains(&"Provider: OpenRouter".to_string())
                && page.text.contains(&format!("Account: {account}"))
                && page.text.contains(&"Model: z-ai/glm-5.2".to_string())
        }));
    }
}
