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
