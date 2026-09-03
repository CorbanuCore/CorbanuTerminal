use super::*;
use crate::chatwidget::wallet_render::WalletTextStyle;
use crate::chatwidget::wallet_render::push_wallet_text;
use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID;
use codex_wallet_daemon::WalletDaemonClient;

impl ChatWidget {
    pub(crate) fn confirm_wallet_plan_disconnect(&mut self) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Disconnect Corbanu API".bold()));
        push_wallet_text(
            &mut header,
            "This removes the stored API credential from this device. Your wallet and dollar balance remain unchanged.",
            WalletTextStyle::Dimmed,
        );
        self.show_selection_view(SelectionViewParams {
            view_id: Some(super::wallet_menu::WALLET_DISCONNECT_PLAN_VIEW_ID),
            header: Box::new(header),
            items: vec![
                confirmation_item("Cancel", "Keep this device connected", || {
                    AppEvent::OpenWallet
                }),
                confirmation_item(
                    "Disconnect API",
                    "Remove only the stored API credential",
                    || AppEvent::WalletPlanDisconnectRequested,
                ),
            ],
            initial_selected_idx: Some(0),
            allow_number_shortcuts: false,
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        });
    }

    pub(crate) fn disconnect_wallet_plan(&mut self) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::task::spawn_blocking(move || {
            let result =
                codex_login::delete_provider_api_key(&home, PFTERMINAL_PLAN_API_KEY_ENV_VAR)
                    .map_err(|error| error.to_string());
            tx.send(AppEvent::WalletPlanDisconnected { result });
        });
    }

    pub(crate) fn on_wallet_plan_disconnected(&mut self, result: Result<bool, String>) {
        self.bottom_pane
            .dismiss_view_by_id(super::wallet_menu::WALLET_DISCONNECT_PLAN_VIEW_ID);
        match result {
            Ok(true) => self.add_info_message(
                "Corbanu API disconnected from this device. The wallet and dollar balance were not changed."
                    .to_string(),
                /*hint*/ None,
            ),
            Ok(false) => self.add_info_message(
                "Corbanu API was already disconnected on this device.".to_string(),
                /*hint*/ None,
            ),
            Err(error) => {
                self.add_error_message(format!("Unable to disconnect Corbanu API: {error}"));
                self.open_wallet_menu();
                return;
            }
        }
        self.choose_replacement_provider_or_reopen_wallet();
    }

    pub(crate) fn confirm_wallet_removal(&mut self, address: String) {
        let short = super::wallet_menu::short_address(&address);
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Remove wallet from this device".bold().red()));
        header.push(Line::from(format!("Wallet: {short}")));
        push_wallet_text(
            &mut header,
            "Funds stay on Solana. Corbanu Terminal cannot recover them without your recovery material.",
            WalletTextStyle::Danger,
        );
        push_wallet_text(
            &mut header,
            "This also removes the stored Corbanu API credential. Your on-chain funds and dollar balance remain unchanged.",
            WalletTextStyle::Dimmed,
        );
        self.show_selection_view(SelectionViewParams {
            view_id: Some(super::wallet_menu::WALLET_REMOVE_VIEW_ID),
            header: Box::new(header),
            items: vec![
                confirmation_item("Cancel", "Keep the wallet on this device", || {
                    AppEvent::OpenWallet
                }),
                confirmation_item(
                    "Remove wallet",
                    "I have saved the recovery material",
                    move || AppEvent::WalletRemoveRequested {
                        address: address.clone(),
                    },
                ),
            ],
            initial_selected_idx: Some(0),
            allow_number_shortcuts: false,
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        });
    }

    pub(crate) fn remove_wallet_from_device(&mut self, address: String) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = WalletDaemonClient::new(home.clone())
                .remove_wallet(address)
                .await
                .map_err(|error| error.to_string());
            let result = match result {
                Ok(()) => {
                    let suppress_home = home.clone();
                    tokio::task::spawn_blocking(move || {
                        codex_login::suppress_provider_api_key(
                            &suppress_home,
                            PFTERMINAL_PLAN_API_KEY_ENV_VAR,
                        )
                        .map(|_| ())
                        .map_err(|error| {
                            format!(
                                "wallet was removed, but its Corbanu API credential could not be disabled: {error}"
                            )
                        })
                    })
                    .await
                    .map_err(|error| format!("credential suppression task failed: {error}"))
                    .and_then(|value| value)
                }
                Err(error) => Err(error),
            };
            let removal_committed = result.is_ok();
            tx.send(AppEvent::WalletRemoved { result });
            if removal_committed {
                let cleanup_result = tokio::task::spawn_blocking(move || {
                    codex_login::delete_provider_api_key(&home, PFTERMINAL_PLAN_API_KEY_ENV_VAR)
                })
                .await;
                match cleanup_result {
                    Ok(Ok(_)) => {}
                    Ok(Err(error)) => tracing::warn!(
                        %error,
                        "wallet was removed and its Corbanu API credential suppressed, but vault cleanup failed"
                    ),
                    Err(error) => tracing::warn!(
                        %error,
                        "wallet was removed and its Corbanu API credential suppressed, but vault cleanup task failed"
                    ),
                }
            }
        });
    }

    pub(crate) fn on_wallet_removed(&mut self, result: Result<(), String>) {
        self.bottom_pane
            .dismiss_view_by_id(super::wallet_menu::WALLET_REMOVE_VIEW_ID);
        self.wallet_capability = None;
        self.wallet_capability_policy = None;
        self.wallet_balances = None;
        match result {
            Ok(()) => self.add_info_message(
                "Wallet and stored Corbanu API credential removed from this device. On-chain funds and the dollar balance were not changed."
                    .to_string(),
                /*hint*/ None,
            ),
            Err(error) => {
                self.add_error_message(format!("Unable to remove wallet cleanly: {error}"));
                self.open_wallet_menu();
                return;
            }
        }
        self.choose_replacement_provider_or_reopen_wallet();
    }

    fn choose_replacement_provider_or_reopen_wallet(&mut self) {
        self.open_wallet_menu();
        if self.config.model_provider_id == PFTERMINAL_PLAN_PROVIDER_ID {
            self.add_info_message(
                "Choose another provider before starting the next turn.".to_string(),
                /*hint*/ None,
            );
            self.open_model_popup();
        }
    }
}

fn confirmation_item<F>(name: &str, description: &str, event: F) -> SelectionItem
where
    F: Fn() -> AppEvent + Send + Sync + 'static,
{
    let mut item = super::wallet_menu::item(name, description, event);
    item.dismiss_on_select = true;
    item
}
