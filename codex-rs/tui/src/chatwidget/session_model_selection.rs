//! Session-only model selections leave durable defaults untouched. Explicit Astra selections retain
//! their originating task and are checked against its effective model when the app applies them.

use super::*;
use crate::bottom_pane::SelectionSecondaryAction;
use crate::keymap::ListAction;
use crate::model_catalog::LUNA_RESERVE_MODEL;

/// A final Astra selection whose model transition must be checked while the app applies it.
#[derive(Debug)]
pub(crate) enum AstraModelPickerAction {
    // Sync S4b decision (ledger, chatwidget/model_popups.rs): an explicit pick in main's
    // provider-tabbed picker sends main's provider-aware events, so these two upstream picker
    // actions are not sent there; the handlers stay for the follow-up that reconciles them.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "main's picker sends provider-aware events (S4b)")
    )]
    UpdateModel,
    #[expect(dead_code, reason = "main's picker sends provider-aware events (S4b)")]
    ApplyModelSelection {
        effort: Option<ReasoningEffortConfig>,
    },
    ApplyAdvancedReasoning {
        effort: ReasoningEffortConfig,
    },
    /// Main's provider-aware pick in its provider-tabbed picker (S4b): the
    /// model on its route's provider. The effort and the saved default follow
    /// as separate events, as on main.
    UpdateModelSelection {
        provider: Option<String>,
    },
    SelectSessionModel {
        effort: Option<ReasoningEffortConfig>,
        service_tier: Option<String>,
    },
}

impl AstraModelPickerAction {
    pub(super) fn into_picker_event(self, thread_id: Option<ThreadId>, model: String) -> AppEvent {
        match thread_id {
            Some(thread_id) => AppEvent::AstraSelectedFromModelPicker {
                thread_id,
                model,
                action: self,
            },
            None => self.into_app_event(model),
        }
    }

    pub(crate) fn into_app_event(self, model: String) -> AppEvent {
        match self {
            Self::UpdateModel => AppEvent::UpdateModel(model),
            Self::ApplyModelSelection { effort } => AppEvent::ApplyModelSelection { model, effort },
            Self::ApplyAdvancedReasoning { effort } => {
                AppEvent::ApplyAdvancedReasoning { model, effort }
            }
            Self::UpdateModelSelection { provider } => {
                AppEvent::UpdateModelSelection { model, provider }
            }
            Self::SelectSessionModel {
                effort,
                service_tier,
            } => AppEvent::SelectSessionModel {
                model,
                effort,
                service_tier,
            },
        }
    }
}

impl ChatWidget {
    pub(super) fn session_model_selection_action(
        &self,
        model: String,
        effort: Option<ReasoningEffortConfig>,
    ) -> Option<SelectionSecondaryAction> {
        // Reserve selections are already session-only.
        if model == LUNA_RESERVE_MODEL {
            return None;
        }
        let key = key_hint::plain(KeyCode::Char('s'));
        let keymap = self.bottom_pane.list_keymap();
        let mut hints = Vec::new();
        if let Some(accept) = keymap.primary_hint(ListAction::Accept) {
            let label = if effort == Some(ReasoningEffortConfig::Ultra) {
                " apply · "
            } else {
                " default · "
            };
            hints.extend([accept.into(), label.dim()]);
        }
        hints.extend([key.into(), " session".dim()]);
        if let Some(cancel) = keymap.primary_hint(ListAction::Cancel) {
            hints.extend([" · ".dim(), cancel.into(), " back".dim()]);
        }
        let warning = effort
            .as_ref()
            .and_then(|effort| self.ultra_reasoning_concurrency_warning(effort));
        let sparkle_thread = self.sparkle_thread_for_picker_action(&model);
        Some(SelectionSecondaryAction {
            key,
            footer_hint: hints.into(),
            action: Box::new(move |tx| {
                tx.send(
                    AstraModelPickerAction::SelectSessionModel {
                        effort: effort.clone(),
                        service_tier: None,
                    }
                    .into_picker_event(sparkle_thread, model.clone()),
                );
                if let Some(warning) = warning.clone() {
                    tx.send(AppEvent::InsertHistoryCell(Box::new(
                        history_cell::new_warning_event(warning),
                    )));
                }
            }),
        })
    }
}
