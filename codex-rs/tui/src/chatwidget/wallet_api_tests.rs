use super::*;

fn account() -> CorbanuApiAccount {
    CorbanuApiAccount {
        balance: CorbanuApiBalance {
            balance_microusd: "12450000".to_string(),
            reserved_microusd: "250000".to_string(),
            available_microusd: "12200000".to_string(),
            balance_usd: "12.45".to_string(),
            reserved_usd: "0.25".to_string(),
            available_usd: "12.20".to_string(),
        },
        keys: vec![codex_wallet::CorbanuApiKeySummary {
            id: "2f9350c1-0cf6-4af1-bb90-cc693c923bb3".to_string(),
            display_prefix: "cbn_live_4D7K".to_string(),
            created_at: "2026-08-30T12:00:00.000Z".to_string(),
            revoked_at: None,
            last_used_at: Some("2026-08-30T12:10:00.000Z".to_string()),
        }],
        models: vec![
            CorbanuApiModel {
                id: "corbanu/glm-5.3-flash".to_string(),
                display_name: "GLM 5.3 Flash".to_string(),
                recommended: true,
                balance_rate: "standard".to_string(),
                privacy: "third-party".to_string(),
                pricing: codex_wallet::CorbanuApiPricing {
                    input_usd: "0.15".to_string(),
                    output_usd: "0.50".to_string(),
                    cache_read_usd: "0.03".to_string(),
                    cache_write_usd: "0.15".to_string(),
                    version: "at-cost-2026-08-30.1".to_string(),
                },
            },
            CorbanuApiModel {
                id: "corbanu/glm-5.3".to_string(),
                display_name: "GLM 5.3".to_string(),
                recommended: false,
                balance_rate: "faster".to_string(),
                privacy: "third-party".to_string(),
                pricing: codex_wallet::CorbanuApiPricing {
                    input_usd: "1.40".to_string(),
                    output_usd: "4.40".to_string(),
                    cache_read_usd: "0.14".to_string(),
                    cache_write_usd: "1.40".to_string(),
                    version: "at-cost-2026-08-30.1".to_string(),
                },
            },
        ],
    }
}

#[test]
fn top_up_amounts_are_exact_and_bounded() {
    assert_eq!(parse_top_up_amount("$7.25"), Ok(7_250_000));
    assert_eq!(parse_top_up_amount("0.000001"), Ok(1));
    for invalid in ["", "0", ".5", "1.", "-1", "1e3", "1.0000001"] {
        assert!(parse_top_up_amount(invalid).is_err(), "{invalid}");
    }
    assert_eq!(format_usd_micros(7_250_000), "7.25");
}

#[tokio::test]
async fn funded_account_surface_shows_balance_prices_keys_and_no_plan_tiers() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.show_corbanu_api_loading();
    chat.on_corbanu_api_loaded(Ok(CorbanuApiView {
        account: Some(account()),
        key_summaries_loaded: true,
        notice: None,
    }));

    let rendered =
        crate::chatwidget::tests::helpers::render_bottom_popup(&chat, /*width*/ 100);
    assert!(rendered.contains("$12.20 available"));
    assert!(rendered.contains("GLM 5.3 Flash · Recommended"));
    assert!(rendered.contains("GLM 5.3 · uses balance faster"));
    assert!(rendered.contains("API key cbn_live_4D7K"));
    assert!(!rendered.contains("Starter"));
    assert!(!rendered.contains("Basic"));
    insta::assert_snapshot!(rendered);
}

#[tokio::test]
async fn unfunded_surface_explains_arbitrary_top_up_without_a_tier() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.show_corbanu_api_loading();
    chat.on_corbanu_api_loaded(Ok(CorbanuApiView {
        account: None,
        key_summaries_loaded: false,
        notice: Some("Top up with USDC to create the first API key.".to_string()),
    }));

    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, /*width*/ 88);
    assert!(rendered.contains("$0 available"));
    assert!(rendered.contains("Add any positive canonical-USDC amount"));
    assert!(!rendered.contains("Starter"));
    assert!(!rendered.contains("Basic"));
    insta::assert_snapshot!(rendered);
}

#[tokio::test]
async fn stale_key_surface_routes_back_through_wallet_ownership() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.show_corbanu_api_loading();
    chat.on_corbanu_api_loaded(Ok(CorbanuApiView {
        account: None,
        key_summaries_loaded: false,
        notice: Some(
            "The stored Corbanu API key is no longer valid. Unlock this wallet to load its balance or create a replacement key."
                .to_string(),
        ),
    }));

    let rendered =
        crate::chatwidget::tests::helpers::render_bottom_popup(&chat, /*width*/ 100);
    assert!(rendered.contains("stored Corbanu API key is no longer valid"));
    assert!(rendered.contains("Manage API keys"));
    assert!(rendered.contains("Create API key"));
    assert!(!rendered.contains("Unavailable:"));
}
