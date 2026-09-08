//! Recovery follows the effective credential source; never silently select a fallback.
use codex_provider_auth::CredentialControl;
use codex_provider_auth::ProviderCatalogEntry;
use codex_provider_auth::ProviderMethodState;
use codex_provider_auth::ProviderRecoveryReason;
use codex_provider_auth::ProviderSetupCapability;
use codex_provider_auth::ProviderStatusSnapshot;

use super::*;

impl ChatWidget {
    pub(crate) fn open_provider_recovery(
        &mut self,
        entry: &ProviderCatalogEntry,
        status: &ProviderStatusSnapshot,
    ) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from(format!("Recover {}", entry.display_name).bold()));
        header.push(
            Paragraph::new(
                "Keep the current provider and model. Esc cancels without changing credentials.",
            )
            .wrap(ratatui::widgets::Wrap { trim: false }),
        );
        let effective = status
            .methods
            .iter()
            .filter(|method| {
                matches!(
                    method.state,
                    ProviderMethodState::Configured { .. }
                        | ProviderMethodState::RecoveryRequired { .. }
                )
            })
            .collect::<Vec<_>>();
        let methods = if effective.is_empty() {
            status.methods.iter().collect()
        } else {
            effective
        };
        let mut items = Vec::new();
        for method in methods {
            let control = match method.state {
                ProviderMethodState::Configured { control, .. }
                | ProviderMethodState::RecoveryRequired {
                    reason: ProviderRecoveryReason::CredentialRejected { control, .. },
                } => Some(control),
                ProviderMethodState::RecoveryRequired {
                    reason: ProviderRecoveryReason::InvalidEnvironmentCredential,
                } => Some(CredentialControl::ExternalEnvironment),
                _ => None,
            };
            if control == Some(CredentialControl::ExternalEnvironment) {
                header.push(Paragraph::new("Environment credential: update it in the launching shell, then restart Corbanu. A vault key cannot override it.").wrap(ratatui::widgets::Wrap { trim: false }));
                continue;
            }
            let label = match &method.capability {
                ProviderSetupCapability::OpenAiAccount
                    if control != Some(CredentialControl::ExternalProvider) =>
                {
                    "Sign in to OpenAI again"
                }
                ProviderSetupCapability::ApiKey { .. } => "Replace the saved API key",
                ProviderSetupCapability::ClaudeAccount => "Recover the selected Claude credential",
                ProviderSetupCapability::CorbanuPlan => "Replace the saved Corbanu API key",
                ProviderSetupCapability::Local { .. } => {
                    header.push(Paragraph::new("This local provider has no login. Check that its model server is running.").wrap(ratatui::widgets::Wrap { trim: false }));
                    continue;
                }
                ProviderSetupCapability::OpenAiAccount
                | ProviderSetupCapability::CommandAuth { .. }
                | ProviderSetupCapability::StatusOnly { .. } => {
                    header.push(Paragraph::new("Externally managed credentials: renew the credential in its owning tool or AWS session, then restart Corbanu to validate it. Corbanu will not replace its auth source.").wrap(ratatui::widgets::Wrap { trim: false }));
                    continue;
                }
            };
            let provider_id = status.id.clone();
            let capability = method.capability.clone();
            items.push(SelectionItem {
                name: label.to_string(),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::ProviderManagerBeginAuthentication {
                        provider_id: provider_id.clone(),
                        capability: capability.clone(),
                    })
                })],
                dismiss_on_select: true,
                ..Default::default()
            });
        }
        self.show_selection_view(SelectionViewParams {
            header: Box::new(header),
            items,
            allow_number_shortcuts: false,
            ..Default::default()
        });
    }
}
