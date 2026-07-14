use super::*;
use codex_gpu_market::RecipeCatalog;
use codex_state::GpuRental;

impl ChatWidget {
    pub(crate) fn open_gpu_menu(&mut self, rentals: Vec<GpuRental>) {
        let mut items = Vec::new();
        for rental in rentals {
            let rental_id = rental.rental_id.clone();
            let hourly = rental.max_hourly_microusd as f64 / 1_000_000.0;
            let accrued = rental.estimated_accrued_microusd as f64 / 1_000_000.0;
            let billable = rental.observed_state.may_be_billable();
            items.push(SelectionItem {
                name: format!("{} · {}", rental.recipe_id, rental.observed_state.as_str()),
                description: Some(format!(
                    "{} · {} · authorized ${hourly:.4}/hr · estimated ${accrued:.4}",
                    rental.provider, rental_id
                )),
                is_current: billable,
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenGpuRental {
                        rental_id: rental_id.clone(),
                    });
                })],
                dismiss_on_select: false,
                ..Default::default()
            });
        }

        for recipe in RecipeCatalog::default().list() {
            let verified = recipe.manifest_verified;
            let recipe_id = recipe.id.clone();
            let status = if verified {
                "Select to search compatible capacity"
            } else {
                "Unavailable until its immutable deployment manifest is verified"
            };
            items.push(SelectionItem {
                name: format!("Rent {}", recipe.model_id),
                description: Some(format!(
                    "{} rev {} · {}× {} · {status}",
                    recipe.id,
                    recipe.revision,
                    recipe.hardware.gpu_count,
                    recipe.hardware.gpu_model
                )),
                is_disabled: !verified,
                disabled_reason: (!verified).then(|| {
                    "Creation is fail-closed: no billable request can use an unverified recipe."
                        .to_string()
                }),
                actions: verified
                    .then(|| {
                        vec![Box::new(move |tx: &AppEventSender| {
                            tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                                recipe_id: recipe_id.clone(),
                            });
                        }) as SelectionAction]
                    })
                    .unwrap_or_default(),
                dismiss_on_select: verified,
                ..Default::default()
            });
        }

        self.show_selection_view(SelectionViewParams {
            title: Some("GPU rentals".to_string()),
            subtitle: Some(
                "Durable rentals remain visible across PFTerminal processes.".to_string(),
            ),
            items,
            is_searchable: false,
            ..Default::default()
        });
    }

    pub(crate) fn open_gpu_authorization_prompt(&mut self, recipe_id: String) {
        let submit_recipe_id = recipe_id.clone();
        let tx = self.app_event_tx.clone();
        let view = CustomPromptView::new(
            "Authorize bounded GPU lease".to_string(),
            "Enter: <max hourly USD> <max total USD> <TTL minutes>".to_string(),
            String::new(),
            None,
            Box::new(move |value| match parse_gpu_authorization(value.as_str()) {
                Ok((maximum_hourly_microusd, maximum_total_microusd, ttl_minutes)) => {
                    tx.send(AppEvent::SearchGpuOffers {
                        recipe_id: submit_recipe_id.clone(),
                        maximum_hourly_microusd,
                        maximum_total_microusd,
                        ttl_minutes,
                    });
                }
                Err(message) => tx.send(AppEvent::GpuOffersLoaded {
                    recipe_id: submit_recipe_id.clone(),
                    authorization: codex_gpu_market::RentalAuthorization {
                        client_operation_id: "invalid".to_string(),
                        maximum_hourly_microusd: 1,
                        maximum_total_microusd: 1,
                        terminate_at_ms: 1,
                        acknowledged_local_enforcement: false,
                    },
                    offers: Err(message),
                }),
            }),
        );
        self.show_custom_prompt_view(view);
    }

    pub(crate) fn open_gpu_offers(
        &mut self,
        recipe_id: String,
        authorization: codex_gpu_market::RentalAuthorization,
        offers: Vec<codex_gpu_market::GpuOffer>,
    ) {
        if offers.is_empty() {
            self.add_info_message(
                "No compatible verified GPU capacity is currently available.".to_string(),
                None,
            );
            return;
        }
        let items = offers
            .into_iter()
            .map(|offer| {
                let offer_for_action = offer.clone();
                let recipe_for_action = recipe_id.clone();
                let authorization_for_action = authorization.clone();
                SelectionItem {
                    name: format!(
                        "{} · {}× {} · ${:.4}/hr",
                        offer.provider,
                        offer.gpu_count,
                        offer.gpu_model,
                        offer.hourly_microusd as f64 / 1_000_000.0
                    ),
                    description: Some(format!(
                        "{} · {} · {} · quote {}",
                        offer.offer_id,
                        offer.security_class,
                        offer.region,
                        if offer.expires_at_ms.is_some() {
                            "best-effort/expiring"
                        } else {
                            "best-effort"
                        }
                    )),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenGpuConfirmation {
                            recipe_id: recipe_for_action.clone(),
                            authorization: authorization_for_action.clone(),
                            offer: offer_for_action.clone(),
                        });
                    })],
                    dismiss_on_select: false,
                    ..Default::default()
                }
            })
            .collect();
        self.show_selection_view(SelectionViewParams {
            title: Some("Compatible GPU offers".to_string()),
            subtitle: Some("Hard compatibility and security filters have already run.".to_string()),
            items,
            ..Default::default()
        });
    }

    pub(crate) fn open_gpu_confirmation(
        &mut self,
        recipe_id: String,
        authorization: codex_gpu_market::RentalAuthorization,
        offer: codex_gpu_market::GpuOffer,
    ) {
        let confirm_recipe = recipe_id.clone();
        let confirm_authorization = authorization.clone();
        let confirm_offer = offer.clone();
        self.show_selection_view(SelectionViewParams {
            title: Some("Confirm billable GPU rental".to_string()),
            subtitle: Some(format!(
                "{} · {} · up to ${:.4}/hr · ${:.2} total · local controller enforces TTL/spend",
                offer.provider,
                recipe_id,
                authorization.maximum_hourly_microusd as f64 / 1_000_000.0,
                authorization.maximum_total_microusd as f64 / 1_000_000.0
            )),
            items: vec![
                SelectionItem {
                    name: "Confirm and rent".to_string(),
                    description: Some(
                        "Revalidate this exact offer, persist authorization, then start the independent controller."
                            .to_string(),
                    ),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::ConfirmGpuRental {
                            recipe_id: confirm_recipe.clone(),
                            authorization: confirm_authorization.clone(),
                            offer: confirm_offer.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                },
                SelectionItem {
                    name: "Cancel".to_string(),
                    actions: vec![Box::new(|tx| tx.send(AppEvent::OpenGpuMenu))],
                    dismiss_on_select: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
    }

    pub(crate) fn open_gpu_rental(&mut self, rental: GpuRental) {
        let rental_id = rental.rental_id.clone();
        let can_stop_serving = matches!(
            rental.observed_state,
            codex_state::GpuRentalState::Ready | codex_state::GpuRentalState::Degraded
        );
        let can_terminate = rental.observed_state.may_be_billable();
        let stop_id = rental_id.clone();
        let terminate_id = rental_id.clone();
        let items = vec![
            SelectionItem {
                name: "Stop serving".to_string(),
                description: Some(
                    "Remove this endpoint from model selection. Provider billing continues."
                        .to_string(),
                ),
                is_disabled: !can_stop_serving,
                disabled_reason: (!can_stop_serving)
                    .then(|| "The endpoint is not currently selectable.".to_string()),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::DisableGpuServing {
                        rental_id: stop_id.clone(),
                    });
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Terminate rental".to_string(),
                description: Some(
                    "Request provider cleanup. Billing remains unresolved until absence is confirmed."
                        .to_string(),
                ),
                is_disabled: !can_terminate,
                disabled_reason: (!can_terminate)
                    .then(|| "This rental is not potentially billable.".to_string()),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::TerminateGpuRental {
                        rental_id: terminate_id.clone(),
                    });
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Back".to_string(),
                actions: vec![Box::new(|tx| tx.send(AppEvent::OpenGpuMenu))],
                dismiss_on_select: false,
                ..Default::default()
            },
        ];

        self.show_selection_view(SelectionViewParams {
            title: Some(format!("GPU rental {rental_id}")),
            subtitle: Some(format!(
                "desired {} · observed {} · estimated ${:.4} · max ${:.4}",
                rental.desired_state.as_str(),
                rental.observed_state.as_str(),
                rental.estimated_accrued_microusd as f64 / 1_000_000.0,
                rental.max_total_microusd as f64 / 1_000_000.0
            )),
            items,
            is_searchable: false,
            ..Default::default()
        });
    }
}

fn parse_gpu_authorization(value: &str) -> Result<(i64, i64, i64), String> {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("Enter exactly: <max hourly USD> <max total USD> <TTL minutes>.".to_string());
    }
    let hourly = parse_positive_usd(parts[0])?;
    let total = parse_positive_usd(parts[1])?;
    let ttl_minutes = parts[2]
        .parse::<i64>()
        .ok()
        .filter(|minutes| (1..=10_080).contains(minutes))
        .ok_or_else(|| "TTL must be an integer from 1 to 10080 minutes.".to_string())?;
    Ok((hourly, total, ttl_minutes))
}

fn parse_positive_usd(value: &str) -> Result<i64, String> {
    let parsed = value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| "Dollar limits must be positive numbers.".to_string())?;
    let microusd = parsed * 1_000_000.0;
    if microusd > i64::MAX as f64 {
        return Err("Dollar limit is too large.".to_string());
    }
    Ok(microusd.round() as i64)
}

#[cfg(test)]
#[path = "gpu_menu_tests.rs"]
mod tests;
