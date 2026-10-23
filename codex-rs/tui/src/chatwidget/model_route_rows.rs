//! #653: `/model` offers each connection of a model served by several, e.g.
//! GLM 5.3 Flash on `zai` (OpenAI-style) and `zai-anthropic` (Claude-style).
//! Picking one switches the session to that provider and saves it like any
//! model choice.

use super::*;
use codex_model_provider_info::ModelRoute;
use codex_model_provider_info::model_route;
use codex_model_provider_info::model_routes;

impl ChatWidget {
    /// Replaces the rows of a model served over several connections with one
    /// row per connection configured in this session, the default
    /// (OpenAI-style) first, at the position of the model's first row.
    /// Connections without a usable credential are dropped afterwards by the
    /// picker's selectability filter, like any other row.
    pub(super) fn with_model_route_rows(&self, presets: Vec<ModelPreset>) -> Vec<ModelPreset> {
        let mut rows = Vec::with_capacity(presets.len());
        let mut expanded = HashSet::new();
        for preset in presets {
            let provider = preset
                .provider_id
                .clone()
                .or_else(|| Self::model_provider_for_selection(&preset.model));
            if provider
                .as_deref()
                .and_then(|provider| model_route(&preset.model, provider))
                .is_none()
            {
                rows.push(preset);
                continue;
            }
            // Later rows of the same model (a runtime entry for the session's
            // connection, say) are replaced by the generated ones.
            if !expanded.insert(preset.model.clone()) {
                continue;
            }
            let routes = model_routes(&preset.model)
                .iter()
                .copied()
                .filter(|route| self.config.model_providers.contains_key(route.provider_id))
                .collect::<Vec<_>>();
            if routes.len() < 2 {
                // A single connection is no choice; the row stays as it was.
                rows.push(preset);
                continue;
            }
            let show_accounts = self.route_accounts_differ(&routes);
            for (index, route) in routes.into_iter().enumerate() {
                let mut row = preset.clone();
                // Unique per connection, even when the first row was a runtime
                // entry carrying another connection's id.
                row.id = if index == 0 && preset.provider_id.is_none() {
                    preset.id.clone()
                } else {
                    format!("{}:{}", route.provider_id, preset.model)
                };
                if index > 0 {
                    row.is_default = false;
                    if let Some(note) = route.note {
                        row.description = note.to_string();
                    }
                }
                row.provider_id = Some(route.provider_id.to_string());
                row.display_name =
                    route.model_label(&Self::model_display_label_for_preset(&preset));
                if show_accounts {
                    row.display_name.push_str(" · ");
                    row.display_name.push_str(&self.route_account_label(route));
                }
                rows.push(row);
            }
        }
        rows
    }

    /// #665: whether another connection of a model served by several can be
    /// used when the preset's own cannot (`zai` on an account that is not
    /// enrolled, `zai-anthropic` on the default key). The picker then still
    /// offers the model, as that connection's row.
    pub(super) fn another_route_is_selectable(&self, preset: &ModelPreset) -> bool {
        let provider = preset
            .provider_id
            .clone()
            .or_else(|| Self::model_provider_for_selection(&preset.model));
        provider
            .as_deref()
            .and_then(|provider| model_route(&preset.model, provider))
            .is_some()
            && model_routes(&preset.model).iter().any(|route| {
                self.config.model_providers.contains_key(route.provider_id)
                    && self
                        .model_catalog
                        .provider_is_selectable(route.provider_id, &preset.model)
            })
    }

    /// PF-84 accounts are per provider, so two connections of one model can
    /// run on different accounts; the rows then name them.
    fn route_accounts_differ(&self, routes: &[ModelRoute]) -> bool {
        let account = |route: &ModelRoute| {
            self.config
                .model_providers
                .get(route.provider_id)
                .and_then(|provider| provider.account.as_ref())
                .map(|account| account.name.clone())
        };
        routes
            .split_first()
            .is_some_and(|(first, rest)| rest.iter().any(|route| account(route) != account(first)))
    }

    fn route_account_label(&self, route: ModelRoute) -> String {
        self.config
            .model_providers
            .get(route.provider_id)
            .and_then(|provider| provider.account.as_ref())
            .map_or_else(
                || "default account".to_string(),
                |account| format!("account {}", account.name),
            )
    }
}
