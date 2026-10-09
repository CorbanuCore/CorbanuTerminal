//! The declared billing basis decides a turn's economics (PF-60-S05).
use super::*;
use crate::config::AccountingMode;
use crate::config::PriceAuthority;
use codex_model_provider_info::BillingBasis;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::auth::AuthMode;
use codex_protocol::protocol::SessionSource;
use codex_state::SqliteConfig;
use codex_state::ThreadMetadataBuilder;
use codex_state::accounting::Basis;
use codex_state::accounting::BasisSource;
use codex_state::accounting::Rates;
use codex_state::accounting::Snapshot;
use codex_utils_absolute_path::AbsolutePathBuf;
use pretty_assertions::assert_eq;

fn bound(
    id: &str,
    provider: &ModelProviderInfo,
    auth: Option<AuthMode>,
    endpoint: &str,
) -> (PriceAuthority, BasisSource) {
    let mode = AccountingMode::Provider {
        scope: Uuid::new_v4(),
        provider_id: id.into(),
        wire_api: provider.wire_api,
        approved_endpoint: endpoint.into(),
        approved_query: None,
        pricing: PriceAuthority::Unavailable,
        basis_source: Default::default(),
    };
    let AccountingMode::Provider {
        pricing,
        basis_source,
        ..
    } = turn_mode(&mode, id, provider, auth, endpoint)
    else {
        panic!("{id}: provider mode must survive rebinding");
    };
    (pricing, basis_source)
}

fn built_in(id: &str) -> ModelProviderInfo {
    codex_model_provider_info::built_in_model_providers(/*openai_base_url*/ None)[id].clone()
}

fn own_route(id: &str, auth: Option<AuthMode>) -> String {
    built_in(id).to_api_provider(auth).unwrap().base_url
}

/// The declared table, not the kind of login, picks the economics. Each row
/// would fail if inference from `AuthMode::ApiKey` or `provider.auth` came
/// back (review Blocker 1 and Minor 10).
#[test]
fn declared_basis_decides_the_economics() {
    let mut bedrock = built_in("amazon-bedrock");
    bedrock.auth = Some(toml::from_str("command = \"print-token\"").unwrap());
    let custom = ModelProviderInfo {
        name: "custom".into(),
        env_key: Some("CUSTOM_KEY".into()),
        wire_api: codex_model_provider_info::WireApi::Chat,
        ..ModelProviderInfo::default()
    };
    let cases = [
        // Kimi Code membership through an environment key: subscription.
        (
            "kimi-code",
            built_in("kimi-code"),
            Some(AuthMode::ApiKey),
            own_route("kimi-code", None),
            PriceAuthority::PlanRate,
        ),
        (
            "zai-anthropic",
            built_in("zai-anthropic"),
            Some(AuthMode::ApiKey),
            own_route("zai-anthropic", None),
            PriceAuthority::PlanRate,
        ),
        (
            "zai",
            built_in("zai"),
            Some(AuthMode::ApiKey),
            own_route("zai", None),
            PriceAuthority::ApiKeyRates,
        ),
        (
            "claude-plan",
            built_in("claude-plan"),
            None,
            own_route("claude-plan", None),
            PriceAuthority::PlanRate,
        ),
        // A command login on Bedrock is still pay per use; AWS routes carry no
        // attributable catalogue rate.
        (
            "amazon-bedrock",
            bedrock,
            None,
            "https://bedrock-mantle.us-east-1.api.aws/openai/v1".to_string(),
            PriceAuthority::Unavailable,
        ),
        // Realtime: the ChatGPT sign-in at the API host is subscription work.
        (
            "openai",
            built_in("openai"),
            Some(AuthMode::Chatgpt),
            "https://api.openai.com/v1".to_string(),
            PriceAuthority::PlanBasis,
        ),
        // A credential with no declared row.
        (
            "openai",
            built_in("openai"),
            Some(AuthMode::Headers),
            own_route("openai", Some(AuthMode::Headers)),
            PriceAuthority::Undeclared,
        ),
        (
            "ollama",
            built_in("ollama"),
            None,
            own_route("ollama", None),
            PriceAuthority::Local,
        ),
        (
            "openai",
            built_in("openai"),
            Some(AuthMode::Chatgpt),
            own_route("openai", Some(AuthMode::Chatgpt)),
            PriceAuthority::PlanRate,
        ),
        (
            "openai",
            built_in("openai"),
            Some(AuthMode::ApiKey),
            own_route("openai", Some(AuthMode::ApiKey)),
            PriceAuthority::ApiKeyRates,
        ),
        // Option B: a custom provider follows the route it used.
        (
            "my-glm",
            custom.clone(),
            None,
            "https://api.z.ai/api/coding/paas/v4".to_string(),
            PriceAuthority::PlanBasis,
        ),
        (
            "my-llm",
            custom,
            None,
            "https://llm.example.com/v1".to_string(),
            PriceAuthority::Undeclared,
        ),
        // A built-in credential moved off its route is judged by that route.
        (
            "openai",
            built_in("openai"),
            Some(AuthMode::ApiKey),
            "https://relay.invalid/v1".to_string(),
            PriceAuthority::Undeclared,
        ),
    ];
    for (id, provider, auth, endpoint, expected) in cases {
        assert_eq!(
            bound(id, &provider, auth, &endpoint),
            (expected, BasisSource::BuiltIn),
            "{id} at {endpoint} with {auth:?}"
        );
    }
}

/// `model_providers.<id>.billing` overrides the table and is recorded as such.
#[test]
fn configured_basis_overrides_and_is_recorded() {
    let mut zai = built_in("zai");
    zai.billing = Some(BillingBasis::Subscription);
    assert_eq!(
        bound("zai", &zai, Some(AuthMode::ApiKey), &own_route("zai", None)),
        (PriceAuthority::PlanRate, BasisSource::UserConfig)
    );
    let custom = ModelProviderInfo {
        name: "custom".into(),
        billing: Some(BillingBasis::Local),
        ..ModelProviderInfo::default()
    };
    assert_eq!(
        bound("my-box", &custom, None, "http://192.168.1.5:8000/v1"),
        (PriceAuthority::Local, BasisSource::UserConfig)
    );
}

fn mode(pricing: PriceAuthority, basis_source: BasisSource) -> AccountingMode {
    AccountingMode::Provider {
        scope: Uuid::new_v4(),
        provider_id: "my-llm".into(),
        wire_api: codex_model_provider_info::WireApi::Chat,
        approved_endpoint: "https://llm.example.com/v1".into(),
        approved_query: None,
        pricing,
        basis_source,
    }
}

/// Every basis is bound to the attempt at admission, with where it came from:
/// a record with no rates when there is no price - pay per use included, so a
/// user-declared pay-per-use route with no price still says it was declared.
#[tokio::test(flavor = "current_thread")]
async fn admission_binds_the_basis_to_every_attempt() -> anyhow::Result<()> {
    let home = tempfile::tempdir()?;
    let db = StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path())?),
        "my-llm".into(),
    )
    .await?;
    let owner = ThreadId::new();
    db.upsert_thread(
        &ThreadMetadataBuilder::new(
            owner,
            home.path().join("basis.jsonl"),
            chrono::Utc::now(),
            SessionSource::Cli,
        )
        .build("my-llm"),
    )
    .await?;
    // One UTC day for every read; the cases run within it.
    let day = chrono::Utc::now().timestamp_millis() / 86_400_000;
    let cases = [
        (
            PriceAuthority::Undeclared,
            BasisSource::BuiltIn,
            Some(Basis::Undeclared),
        ),
        (
            PriceAuthority::Local,
            BasisSource::UserConfig,
            Some(Basis::Local),
        ),
        (
            PriceAuthority::PlanBasis,
            BasisSource::UserConfig,
            Some(Basis::PlanEquivalent),
        ),
        (
            PriceAuthority::Unavailable,
            BasisSource::UserConfig,
            Some(Basis::Billed),
        ),
    ];
    for (pricing, source, expected) in cases {
        let sampling = Sampling::start(
            db.clone(),
            owner,
            format!("{pricing:?}"),
            &mode(pricing, source),
        )
        .await?;
        sampling
            .admit("some-model", "https://llm.example.com/v1/chat/completions")
            .await?;
        let quote = codex_state::accounting::AccountingStore::inspect_day(
            &db,
            owner,
            day,
            chrono::Utc::now().timestamp_millis(),
        )
        .await?;
        let codex_state::accounting::InspectionDay::Ready(view) = quote else {
            panic!("{quote:?}");
        };
        let snapshot: Option<Snapshot> = view
            .requests
            .values()
            .flatten()
            .find(|quote| quote.attempt.turn == format!("{pricing:?}"))
            .and_then(|quote| quote.snapshot.clone());
        assert_eq!(
            snapshot.map(|s| (s.basis, s.rates, s.basis_source)),
            expected.map(|basis| (basis, Rates::default(), source)),
            "{pricing:?}"
        );
    }
    // A changed declaration (each case above is a new one for the same
    // provider) applies to new attempts only: every earlier attempt still
    // reads with the basis it was admitted under (AC3).
    let codex_state::accounting::InspectionDay::Ready(view) =
        codex_state::accounting::AccountingStore::inspect_day(
            &db,
            owner,
            day,
            chrono::Utc::now().timestamp_millis(),
        )
        .await?
    else {
        panic!("expected the day");
    };
    let mut bases: Vec<(String, Basis)> = view
        .requests
        .values()
        .flatten()
        .map(|quote| (quote.attempt.turn.clone(), quote.basis()))
        .collect();
    bases.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        bases,
        vec![
            ("Local".to_string(), Basis::Local),
            ("PlanBasis".to_string(), Basis::PlanEquivalent),
            ("Unavailable".to_string(), Basis::Billed),
            ("Undeclared".to_string(), Basis::Undeclared),
        ]
    );
    Ok(())
}

/// Image generation takes OpenAI's published image rates only where API-key
/// rates apply: OpenAI's own provider, pay per use. A plan route, an unpriced
/// route or another provider gets no image rates, and keeps the ordinary
/// usage reading.
#[tokio::test(flavor = "current_thread")]
async fn image_generation_rates_apply_only_to_openai_api_key_rates() -> anyhow::Result<()> {
    let home = tempfile::tempdir()?;
    let db = StateRuntime::init(
        SqliteConfig::from_sqlite_home(AbsolutePathBuf::try_from(home.path())?),
        "openai".into(),
    )
    .await?;
    let owner = ThreadId::new();
    db.upsert_thread(
        &ThreadMetadataBuilder::new(
            owner,
            home.path().join("image.jsonl"),
            chrono::Utc::now(),
            SessionSource::Cli,
        )
        .build("openai"),
    )
    .await?;
    let day = chrono::Utc::now().timestamp_millis() / 86_400_000;
    let endpoint = "https://api.openai.com/v1";
    let image_rates = Rates {
        noncached: Some(serde_json::from_value(serde_json::json!("5"))?),
        read: Some(serde_json::from_value(serde_json::json!("1.25"))?),
        write: None,
        output: Some(serde_json::from_value(serde_json::json!("30"))?),
    };
    let cases = [
        ("openai", PriceAuthority::ApiKeyRates, true, image_rates),
        ("openai", PriceAuthority::PlanRate, false, Rates::default()),
        (
            "openai",
            PriceAuthority::Unavailable,
            false,
            Rates::default(),
        ),
        (
            "openrouter",
            PriceAuthority::ApiKeyRates,
            true,
            Rates::default(),
        ),
    ];
    for (provider, pricing, reads_images, expected) in cases {
        let turn = format!("{provider}-{pricing:?}");
        let mode = AccountingMode::Provider {
            scope: Uuid::new_v4(),
            provider_id: provider.into(),
            wire_api: codex_model_provider_info::WireApi::Responses,
            approved_endpoint: endpoint.into(),
            approved_query: None,
            pricing,
            basis_source: BasisSource::BuiltIn,
        };
        let sampling = Sampling::start_at_path(
            db.clone(),
            owner,
            turn.clone(),
            &mode,
            prices::IMAGE_GENERATIONS_PATH,
        )
        .await?;
        assert_eq!(sampling.image_generation, reads_images, "{turn}");
        sampling
            .admit_with_tier(
                "gpt-image-2",
                "https://api.openai.com/v1/images/generations",
                /*tier*/ None,
            )
            .await?;
        let codex_state::accounting::InspectionDay::Ready(view) =
            codex_state::accounting::AccountingStore::inspect_day(
                &db,
                owner,
                day,
                chrono::Utc::now().timestamp_millis(),
            )
            .await?
        else {
            panic!("expected the day");
        };
        let rates = view
            .requests
            .values()
            .flatten()
            .find(|quote| quote.attempt.turn == turn)
            .and_then(|quote| quote.snapshot.clone())
            .map(|snapshot| snapshot.rates);
        assert_eq!(rates, Some(expected), "{turn}");
    }
    Ok(())
}
