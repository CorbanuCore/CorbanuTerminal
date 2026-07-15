use super::*;

#[test]
fn gpu_limit_inputs_accept_familiar_currency_and_duration_values() {
    assert_eq!(parse_positive_usd("3.25"), Ok(3_250_000));
    assert_eq!(parse_positive_usd("$10.50"), Ok(10_500_000));
    assert_eq!(parse_positive_usd("1,250"), Ok(1_250_000_000));
    assert_eq!(parse_ttl_minutes(" 120 "), Ok(120));
}

#[test]
fn gpu_limit_inputs_reject_invalid_or_unbounded_values() {
    assert!(parse_positive_usd("documentation").is_err());
    assert!(parse_positive_usd("-1").is_err());
    assert!(parse_ttl_minutes("0").is_err());
    assert!(parse_ttl_minutes("10081").is_err());
}

#[test]
fn gpu_provider_names_are_presented_as_user_facing_marketplaces() {
    assert_eq!(gpu_provider_display_name("runpod"), "RunPod");
    assert_eq!(gpu_provider_display_name("vast"), "Vast.ai");
    assert_eq!(gpu_provider_display_name("unexpected"), "GPU marketplace");
}

#[tokio::test]
async fn gpu_authorization_uses_one_labeled_input_and_retries_in_place_snapshot() {
    let (mut chat, _tx, mut event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.open_gpu_authorization_prompt(
        "test-recipe".to_string(),
        crate::app_event::GpuAuthorizationPromptState::default(),
    );

    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 80);
    assert!(rendered.contains("Maximum hourly price"));
    assert!(rendered.contains("USD per hour, for example 10"));
    assert!(!rendered.contains("<max hourly USD>"));
    insta::assert_snapshot!(rendered);

    while event_rx.try_recv().is_ok() {}
    chat.handle_paste("documentation".to_string());
    chat.handle_key_event(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ));

    assert!(matches!(
        event_rx.try_recv(),
        Ok(AppEvent::OpenGpuAuthorizationPrompt {
            recipe_id,
            state: crate::app_event::GpuAuthorizationPromptState {
                maximum_hourly_microusd: None,
                maximum_total_microusd: None,
                validation_error: Some(_),
            },
        }) if recipe_id == "test-recipe"
    ));
}

#[tokio::test]
async fn gpu_menu_excludes_nonbillable_history_snapshot() {
    let (mut chat, _tx, _event_rx, _op_rx) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    chat.open_gpu_menu(vec![
        rental(
            "terminated-rental",
            codex_state::GpuRentalState::TerminatedConfirmed,
            Some("vast-1"),
        ),
        rental("failed-rental", codex_state::GpuRentalState::Failed, None),
        rental(
            "active-rental",
            codex_state::GpuRentalState::Ready,
            Some("vast-2"),
        ),
    ]);

    let rendered = crate::chatwidget::tests::helpers::render_bottom_popup(&chat, 100);
    assert!(rendered.contains("active-rental"));
    assert!(!rendered.contains("terminated-rental"));
    assert!(!rendered.contains("failed-rental"));
    insta::assert_snapshot!(rendered);
}

fn rental(
    rental_id: &str,
    observed_state: codex_state::GpuRentalState,
    provider_resource_id: Option<&str>,
) -> GpuRental {
    GpuRental {
        rental_id: rental_id.to_string(),
        installation_id: "test-installation".to_string(),
        client_operation_id: format!("operation-{rental_id}"),
        provider: "vast".to_string(),
        recipe_id: "deepseek-flash-2xh200".to_string(),
        recipe_revision: "test-revision".to_string(),
        offer_snapshot_json: "{}".to_string(),
        quote_expires_at_ms: None,
        max_hourly_microusd: 10_000_000,
        max_total_microusd: 20_000_000,
        terminate_at_ms: 2_000,
        enforcement_class: codex_state::GpuLimitEnforcement::LocalControllerDependent,
        desired_state: observed_state,
        observed_state,
        provider_resource_id: provider_resource_id.map(str::to_string),
        ownership_tag: "test-owner".to_string(),
        state_sequence: 1,
        controller_lease_owner: None,
        controller_lease_until_ms: 0,
        provision_step: None,
        endpoint_base_url: None,
        endpoint_provider_id: None,
        last_error_code: None,
        last_error_message: None,
        diagnostic_ref: None,
        last_reconciled_at_ms: None,
        next_retry_at_ms: 0,
        retry_count: 0,
        estimated_accrued_microusd: 1_250_000,
        provider_reported_cost_microusd: None,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
        terminated_confirmed_at_ms: None,
    }
}
