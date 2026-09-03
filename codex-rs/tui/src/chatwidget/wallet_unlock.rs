use super::*;
use crate::app_event::WalletSecret;
use crate::app_event::WalletUnlockContinuation;
use crate::app_event::WalletUnlockedResult;
use codex_wallet_daemon::UnlockPolicy;
use codex_wallet_daemon::WalletDaemonClient;
use zeroize::Zeroize;
use zeroize::Zeroizing;

impl ChatWidget {
    pub(crate) fn open_wallet_unlock(
        &mut self,
        policy: UnlockPolicy,
        continuation: WalletUnlockContinuation,
    ) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = WalletDaemonClient::new(home)
                .status()
                .await
                .map(|status| status.wallet_exists)
                .map_err(|error| error.to_string());
            tx.send(AppEvent::WalletUnlockPreflightFinished {
                policy,
                continuation,
                result,
            });
        });
    }

    fn show_wallet_unlock_prompt(
        &mut self,
        policy: UnlockPolicy,
        continuation: WalletUnlockContinuation,
    ) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        let cancel_tx = self.app_event_tx.clone();
        let cancelled_deferred = wallet_unlock_deferred_setup(&continuation).cloned();
        let submit = Box::new(move |_label: String, mut passcode: String| {
            tokio::spawn(async move {
                let result = WalletDaemonClient::new(home)
                    .unlock(std::mem::take(&mut passcode), policy)
                    .await
                    .map(|(capability, expires_in_seconds)| WalletUnlockedResult {
                        capability: WalletSecret::new(capability),
                        expires_in_seconds,
                    })
                    .map_err(|error| error.to_string());
                passcode.zeroize();
                tx.send(AppEvent::WalletUnlockFinished {
                    policy,
                    continuation,
                    result,
                });
            });
        });
        let view = if let Some(deferred) = cancelled_deferred {
            crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret_with_cancel(
                "wallet-passcode".to_string(),
                "Unlock wallet".to_string(),
                "Wallet passcode — masked".to_string(),
                "Passcode (masked)".to_string(),
                submit,
                Box::new(move || {
                    cancel_tx.send(AppEvent::DeferredCorbanuPlanCancelled { deferred });
                }),
            )
        } else {
            crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
                "wallet-passcode".to_string(),
                "Unlock wallet".to_string(),
                "Wallet passcode — masked".to_string(),
                "Passcode (masked)".to_string(),
                submit,
            )
        };
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn on_wallet_unlock_preflight_finished(
        &mut self,
        policy: UnlockPolicy,
        continuation: WalletUnlockContinuation,
        result: Result<bool, String>,
    ) {
        match result {
            Ok(true) => self.show_wallet_unlock_prompt(policy, continuation),
            Ok(false) => {
                self.add_error_message(
                    "Cannot unlock wallet: no local wallet exists. Use /wallet create or /wallet restore."
                        .to_string(),
                );
                if let Some(deferred) = wallet_unlock_deferred_setup(&continuation).cloned() {
                    self.app_event_tx
                        .send(AppEvent::DeferredCorbanuPlanCancelled { deferred });
                } else {
                    self.open_wallet_menu();
                }
            }
            Err(error) => {
                self.add_error_message(format!(
                    "Cannot check wallet state before unlock: {error}. No passcode was requested."
                ));
                if let Some(deferred) = wallet_unlock_deferred_setup(&continuation).cloned() {
                    self.app_event_tx
                        .send(AppEvent::DeferredCorbanuPlanCancelled { deferred });
                }
            }
        }
    }

    pub(crate) fn open_wallet_custom_unlock(
        &mut self,
        validation_error: Option<String>,
        continuation: WalletUnlockContinuation,
    ) {
        let tx = self.app_event_tx.clone();
        let guidance = validation_error
            .map(|error| format!("Try again: {error}"))
            .unwrap_or_else(|| {
                "Signing access stays in this TUI and expires on schedule.".to_string()
            });
        let view = CustomPromptView::new(
            "Custom wallet unlock".to_string(),
            "Whole minutes from 1 to 480, for example 30".to_string(),
            String::new(),
            Some(guidance),
            Box::new(move |value| match parse_unlock_minutes(&value) {
                Ok(policy) => tx.send(AppEvent::OpenWalletUnlock {
                    policy,
                    continuation: continuation.clone(),
                }),
                Err(validation_error) => tx.send(AppEvent::OpenWalletCustomUnlock {
                    validation_error: Some(validation_error),
                    continuation: continuation.clone(),
                }),
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn on_wallet_unlock_finished(
        &mut self,
        policy: UnlockPolicy,
        continuation: WalletUnlockContinuation,
        result: Result<WalletUnlockedResult, String>,
    ) {
        match result {
            Ok(unlocked) => {
                self.wallet_capability = Some(Zeroizing::new(unlocked.capability.into_inner()));
                self.wallet_capability_policy = Some(policy);
                self.add_info_message(
                    unlock_confirmation(policy, unlocked.expires_in_seconds),
                    /*hint*/ None,
                );
                match continuation {
                    WalletUnlockContinuation::WalletMenu => self.open_wallet_menu(),
                    WalletUnlockContinuation::OpenCorbanuApi { deferred } => {
                        if let Some(deferred) = deferred {
                            self.open_corbanu_api_for_deferred(deferred);
                        } else {
                            self.open_corbanu_api();
                        }
                    }
                    WalletUnlockContinuation::OpenPlans { mode } => self.open_wallet_plans(mode),
                    WalletUnlockContinuation::CorbanuApiOperation {
                        operation,
                        deferred,
                    } => {
                        self.request_corbanu_api_operation(operation, deferred);
                    }
                }
            }
            Err(error) => {
                self.add_error_message(format!("Wallet unlock failed: {error}. Try again."));
                self.show_wallet_unlock_prompt(policy, continuation);
            }
        }
    }
}

fn wallet_unlock_deferred_setup(
    continuation: &WalletUnlockContinuation,
) -> Option<&crate::onboarding::provider_setup::DeferredProviderSetup> {
    match continuation {
        WalletUnlockContinuation::OpenCorbanuApi { deferred }
        | WalletUnlockContinuation::CorbanuApiOperation { deferred, .. } => deferred.as_ref(),
        WalletUnlockContinuation::WalletMenu | WalletUnlockContinuation::OpenPlans { .. } => None,
    }
}

pub(super) fn wallet_capability_for_request(
    capability: &mut Option<Zeroizing<String>>,
    policy: Option<UnlockPolicy>,
) -> Option<Zeroizing<String>> {
    if matches!(policy, Some(UnlockPolicy::OneAction)) {
        capability.take()
    } else {
        capability
            .as_ref()
            .map(|value| Zeroizing::new(value.to_string()))
    }
}

fn parse_unlock_minutes(value: &str) -> Result<UnlockPolicy, String> {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|minutes| (1..=480).contains(minutes))
        .map(|minutes| UnlockPolicy::Timed {
            duration_seconds: minutes * 60,
        })
        .ok_or_else(|| "Enter whole minutes from 1 through 480.".to_string())
}

fn unlock_confirmation(policy: UnlockPolicy, expires_in_seconds: u64) -> String {
    match policy {
        UnlockPolicy::OneAction => format!(
            "Wallet unlocked for one signing action; it locks after that attempt or in {} minute(s) if unused.",
            expires_in_seconds / 60
        ),
        UnlockPolicy::Timed { .. } => {
            format!("Wallet unlocked for {} minute(s).", expires_in_seconds / 60)
        }
    }
}

#[cfg(test)]
#[path = "wallet_unlock_tests.rs"]
mod tests;
