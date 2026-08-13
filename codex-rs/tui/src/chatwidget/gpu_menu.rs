use super::*;
use codex_gpu_market::RecipeCatalog;
use codex_state::GpuRental;

impl ChatWidget {
    pub(crate) fn open_gpu_menu(&mut self, rentals: Vec<GpuRental>) {
        let mut items = Vec::new();
        for rental in rentals
            .into_iter()
            .filter(codex_state::GpuRental::may_be_billable)
        {
            let rental_id = rental.rental_id.clone();
            let hourly = rental.max_hourly_microusd as f64 / 1_000_000.0;
            let accrued = rental.estimated_accrued_microusd as f64 / 1_000_000.0;
            let progress = gpu_progress_summary(&rental, chrono::Utc::now().timestamp_millis());
            items.push(SelectionItem {
                name: format!("{} · {}", rental.recipe_id, rental.observed_state.as_str()),
                description: Some(format!(
                    "{} · {} · {progress} · authorized ${hourly:.4}/hr · estimated ${accrued:.4}",
                    rental.provider, rental_id,
                )),
                is_current: true,
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenGpuRental {
                        rental_id: rental_id.clone(),
                    });
                })],
                dismiss_on_select: false,
                ..Default::default()
            });
        }

        for (provider, display_name) in [("runpod", "RunPod"), ("vast", "Vast.ai")] {
            let provider = provider.to_string();
            items.push(SelectionItem {
                name: format!("Configure {display_name} API key"),
                description: Some(
                    "Masked entry stored in the PFTerminal vault; replacing a key is supported."
                        .to_string(),
                ),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenGpuProviderCredential {
                        provider: provider.clone(),
                    });
                })],
                dismiss_on_select: true,
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
                name: format!(
                    "Rent {} · {}× {}",
                    recipe.model_id, recipe.hardware.gpu_count, recipe.hardware.gpu_model
                ),
                description: Some(format!(
                    "{} rev {} · {} · {}× {} · {status}",
                    recipe.id,
                    recipe.revision,
                    recipe.stability.label(),
                    recipe.hardware.gpu_count,
                    recipe.hardware.gpu_model
                )),
                is_disabled: !verified,
                disabled_reason: (!verified).then(|| {
                    "Creation is fail-closed: no billable request can use an unverified recipe."
                        .to_string()
                }),
                actions: if verified {
                    vec![Box::new(move |tx: &AppEventSender| {
                        tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                            recipe_id: recipe_id.clone(),
                            state: crate::app_event::GpuAuthorizationPromptState::default(),
                        });
                    }) as SelectionAction]
                } else {
                    Default::default()
                },
                dismiss_on_select: verified,
                ..Default::default()
            });
        }

        self.show_selection_view(SelectionViewParams {
            title: Some("GPU rentals".to_string()),
            subtitle: Some(
                "Active and potentially billable rentals remain visible across PFTerminal processes."
                    .to_string(),
            ),
            items,
            is_searchable: false,
            ..Default::default()
        });
    }

    pub(crate) fn open_gpu_provider_credential(&mut self, provider: String) {
        let (display_name, label) = match provider.as_str() {
            "runpod" => ("RunPod", codex_gpu_market::RUNPOD_API_KEY_LABEL),
            "vast" => ("Vast.ai", codex_gpu_market::VAST_API_KEY_LABEL),
            _ => {
                self.add_error_message("Unsupported GPU provider credential.".to_string());
                return;
            }
        };
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            label.to_string(),
            format!("Add or replace {display_name} API key"),
            "API key — masked".to_string(),
            "API key (masked - not shown, not stored in chat)".to_string(),
            Box::new(move |_label: String, secret: String| {
                tx.send(AppEvent::SaveGpuProviderCredential {
                    provider: provider.clone(),
                    api_key: crate::app_event::ProviderApiKeySecret::new(secret),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_gpu_authorization_prompt(
        &mut self,
        recipe_id: String,
        state: crate::app_event::GpuAuthorizationPromptState,
    ) {
        let tx = self.app_event_tx.clone();
        let validation_context = |guidance: &str| {
            state
                .validation_error
                .as_ref()
                .map(|error| format!("Try again: {error}"))
                .unwrap_or_else(|| guidance.to_string())
        };
        let view = match (state.maximum_hourly_microusd, state.maximum_total_microusd) {
            (None, None) => CustomPromptView::new(
                "GPU limits · 1 of 3 · Maximum hourly price".to_string(),
                "USD per hour, for example 10".to_string(),
                String::new(),
                Some(validation_context(
                    "Only offers at or below this hourly price are shown.",
                )),
                Box::new(move |value| match parse_positive_usd(value.as_str()) {
                    Ok(maximum_hourly_microusd) => {
                        tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                            recipe_id: recipe_id.clone(),
                            state: crate::app_event::GpuAuthorizationPromptState {
                                maximum_hourly_microusd: Some(maximum_hourly_microusd),
                                ..Default::default()
                            },
                        });
                    }
                    Err(validation_error) => {
                        tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                            recipe_id: recipe_id.clone(),
                            state: crate::app_event::GpuAuthorizationPromptState {
                                validation_error: Some(validation_error),
                                ..Default::default()
                            },
                        });
                    }
                }),
            ),
            (Some(maximum_hourly_microusd), None) => CustomPromptView::new(
                "GPU limits · 2 of 3 · Total spending cap".to_string(),
                "Total USD, for example 40".to_string(),
                String::new(),
                Some(validation_context(&format!(
                    "Hourly limit: ${:.2}. No offer has been accepted yet.",
                    maximum_hourly_microusd as f64 / 1_000_000.0
                ))),
                Box::new(move |value| match parse_positive_usd(value.as_str()) {
                    Ok(maximum_total_microusd) => {
                        tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                            recipe_id: recipe_id.clone(),
                            state: crate::app_event::GpuAuthorizationPromptState {
                                maximum_hourly_microusd: Some(maximum_hourly_microusd),
                                maximum_total_microusd: Some(maximum_total_microusd),
                                validation_error: None,
                            },
                        });
                    }
                    Err(validation_error) => {
                        tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                            recipe_id: recipe_id.clone(),
                            state: crate::app_event::GpuAuthorizationPromptState {
                                maximum_hourly_microusd: Some(maximum_hourly_microusd),
                                maximum_total_microusd: None,
                                validation_error: Some(validation_error),
                            },
                        });
                    }
                }),
            ),
            (Some(maximum_hourly_microusd), Some(maximum_total_microusd)) => CustomPromptView::new(
                "GPU rental duration · 3 of 3 · MINUTES".to_string(),
                "Enter minutes (not dollars), for example 120 = 2 hours".to_string(),
                String::new(),
                Some(validation_context(&format!(
                    "TIME LIMIT, NOT PRICE: setup and model download count against this duration. Price limits already set: ${:.2}/hour · ${:.2} total. Maximum duration: 10,080 minutes (7 days).",
                    maximum_hourly_microusd as f64 / 1_000_000.0,
                    maximum_total_microusd as f64 / 1_000_000.0
                ))),
                Box::new(move |value| match parse_ttl_minutes(value.as_str()) {
                    Ok(ttl_minutes) => tx.send(AppEvent::SearchGpuOffers {
                        recipe_id: recipe_id.clone(),
                        maximum_hourly_microusd,
                        maximum_total_microusd,
                        ttl_minutes,
                    }),
                    Err(validation_error) => {
                        tx.send(AppEvent::OpenGpuAuthorizationPrompt {
                            recipe_id: recipe_id.clone(),
                            state: crate::app_event::GpuAuthorizationPromptState {
                                maximum_hourly_microusd: Some(maximum_hourly_microusd),
                                maximum_total_microusd: Some(maximum_total_microusd),
                                validation_error: Some(validation_error),
                            },
                        });
                    }
                }),
            ),
            (None, Some(_)) => {
                self.add_error_message(
                    "GPU spending limits became inconsistent; no rental was created.".to_string(),
                );
                return;
            }
        };
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
                /*hint*/ None,
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
                        gpu_provider_display_name(offer.provider.as_str()),
                        offer.gpu_count,
                        offer.gpu_model,
                        offer.hourly_microusd as f64 / 1_000_000.0
                    ),
                    description: Some(format!(
                        "{} · {} · {} · topology {} · quote {}",
                        offer.offer_id,
                        offer.security_class,
                        offer.region,
                        if offer.high_bandwidth_interconnect {
                            "provider-attested"
                        } else if offer.runtime_topology_verification {
                            "verified on allocation before serving"
                        } else {
                            "not required"
                        },
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
                    // The confirmation view owns the next navigation step. Keeping the offer
                    // list underneath it resurfaces a stale billable action after confirmation
                    // succeeds and makes an already-created rental look repeatable.
                    dismiss_on_select: true,
                    ..Default::default()
                }
            })
            .collect();
        self.show_selection_view(SelectionViewParams {
            title: Some("Choose GPU provider and offer".to_string()),
            subtitle: Some(
                "Vast.ai and RunPod offers shown here passed the recipe's hard compatibility and security filters."
                    .to_string(),
            ),
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
        let remaining_minutes = remaining_authorization_minutes(
            authorization.terminate_at_ms,
            chrono::Utc::now().timestamp_millis(),
        );
        let confirm_recipe = recipe_id.clone();
        let confirm_authorization = authorization.clone();
        let confirm_offer = offer.clone();
        self.show_selection_view(SelectionViewParams {
            title: Some("Confirm billable GPU rental".to_string()),
            subtitle: Some(format!(
                "{} · {} · up to ${:.4}/hr · ${:.2} total · automatic stop in about {remaining_minutes} minutes; setup time is included",
                gpu_provider_display_name(offer.provider.as_str()),
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
            initial_selected_idx: Some(1),
            allow_number_shortcuts: false,
            ..Default::default()
        });
    }

    pub(crate) fn open_gpu_rental(&mut self, rental: GpuRental) {
        let rental_id = rental.rental_id.clone();
        let can_stop_serving = matches!(
            rental.observed_state,
            codex_state::GpuRentalState::Ready | codex_state::GpuRentalState::Degraded
        );
        let can_terminate = rental.may_be_billable();
        let stop_id = rental_id.clone();
        let terminate_id = rental_id.clone();
        let progress = gpu_progress_summary(&rental, chrono::Utc::now().timestamp_millis());
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
                "desired {} · observed {} · {progress} · estimated ${:.4} · max ${:.4}",
                rental.desired_state.as_str(),
                rental.observed_state.as_str(),
                rental.estimated_accrued_microusd as f64 / 1_000_000.0,
                rental.max_total_microusd as f64 / 1_000_000.0
            )),
            items,
            is_searchable: false,
            allow_number_shortcuts: false,
            ..Default::default()
        });
    }
}

pub(crate) fn gpu_progress_summary(rental: &GpuRental, now_ms: i64) -> String {
    if rental.observed_state == codex_state::GpuRentalState::Ready {
        return "available in model picker".to_string();
    }
    if rental.observed_state == codex_state::GpuRentalState::Terminating {
        return "terminating provider resource".to_string();
    }
    let elapsed = format_gpu_elapsed(now_ms.saturating_sub(rental.created_at_ms));
    rental
        .provision_step
        .as_deref()
        .and_then(gpu_provision_phase_label)
        .map_or_else(
            || format!("{elapsed} elapsed"),
            |phase| format!("{phase} · {elapsed} elapsed"),
        )
}

pub(crate) fn gpu_provision_phase_label(step: &str) -> Option<&'static str> {
    match step {
        "hardware_check" => Some("checking allocated hardware"),
        "runtime_setup" => Some("installing runtime dependencies"),
        "runtime_build" => Some("building inference runtime"),
        "model_download" => Some("downloading model weights"),
        "model_verification" => Some("verifying model artifacts"),
        "model_loading" => Some("loading model onto GPUs"),
        "endpoint_probing" | "02-readiness" => Some("qualifying inference endpoint"),
        _ => None,
    }
}

fn format_gpu_elapsed(elapsed_ms: i64) -> String {
    let total_minutes = elapsed_ms.max(0).saturating_div(60_000);
    if total_minutes < 1 {
        "<1m".to_string()
    } else if total_minutes < 60 {
        format!("{total_minutes}m")
    } else {
        format!("{}h {:02}m", total_minutes / 60, total_minutes % 60)
    }
}

fn remaining_authorization_minutes(terminate_at_ms: i64, now_ms: i64) -> i64 {
    terminate_at_ms
        .saturating_sub(now_ms)
        .max(0)
        .saturating_add(59_999)
        .saturating_div(60_000)
}

fn parse_ttl_minutes(value: &str) -> Result<i64, String> {
    value
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|minutes| (1..=10_080).contains(minutes))
        .ok_or_else(|| "Enter whole minutes from 1 to 10,080, for example 120.".to_string())
}

fn parse_positive_usd(value: &str) -> Result<i64, String> {
    let normalized = value
        .trim()
        .strip_prefix('$')
        .unwrap_or(value.trim())
        .replace(',', "");
    let parsed = normalized
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| {
            "Enter a USD amount greater than 0, for example 10 or $10.50.".to_string()
        })?;
    let microusd = parsed * 1_000_000.0;
    if microusd > i64::MAX as f64 {
        return Err("Dollar limit is too large.".to_string());
    }
    Ok(microusd.round() as i64)
}

fn gpu_provider_display_name(provider: &str) -> &str {
    match provider {
        "runpod" => "RunPod",
        "vast" => "Vast.ai",
        _ => "GPU marketplace",
    }
}

#[cfg(test)]
#[path = "gpu_menu_tests.rs"]
mod tests;
