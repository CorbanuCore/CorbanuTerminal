use super::*;
use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID;
use codex_wallet_daemon::WalletDaemonClient;

impl ChatWidget {
    pub(crate) fn confirm_wallet_plan_disconnect(&mut self) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Disconnect PfTerminal Plan".bold()));
        for line in super::wallet_menu::wallet_wrapped_lines(
            "This removes the plan credential from this device. Your paid period and wallet remain unchanged.",
        ) {
            header.push(Line::from(line.dim()));
        }
        self.show_selection_view(SelectionViewParams {
            view_id: Some(super::wallet_menu::WALLET_DISCONNECT_PLAN_VIEW_ID),
            header: Box::new(header),
            items: vec![
                confirmation_item("Cancel", "Keep this device connected", || {
                    AppEvent::OpenWallet
                }),
                confirmation_item(
                    "Disconnect plan",
                    "Remove only the metered-inference credential",
                    || AppEvent::WalletPlanDisconnectRequested,
                ),
            ],
            initial_selected_idx: Some(0),
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
                "PfTerminal Plan disconnected from this device. The paid period and wallet were not changed."
                    .to_string(),
                None,
            ),
            Ok(false) => self.add_info_message(
                "PfTerminal Plan was already disconnected on this device.".to_string(),
                None,
            ),
            Err(error) => {
                self.add_error_message(format!("Unable to disconnect PfTerminal Plan: {error}"));
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
        for line in super::wallet_menu::wallet_wrapped_lines(
            "Funds stay on Solana. PfTerminal cannot recover them without your recovery material.",
        ) {
            header.push(Line::from(line.red()));
        }
        for line in super::wallet_menu::wallet_wrapped_lines(
            "This also disconnects the local PfTerminal Plan credential. It does not cancel or refund the paid period.",
        ) {
            header.push(Line::from(line.dim()));
        }
        self.show_selection_view(SelectionViewParams {
            view_id: Some(super::wallet_menu::WALLET_REMOVE_VIEW_ID),
            header: Box::new(header),
            items: vec![
                confirmation_item("Cancel", "Keep the wallet on this device", || {
                    AppEvent::OpenWallet
                }),
                {
                    let address = address;
                    confirmation_item(
                        "Remove wallet",
                        "I have saved the recovery material",
                        move || AppEvent::WalletRemoveRequested {
                            address: address.clone(),
                        },
                    )
                },
            ],
            initial_selected_idx: Some(0),
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
                Ok(()) => tokio::task::spawn_blocking(move || {
                    codex_login::delete_provider_api_key(
                        &home,
                        PFTERMINAL_PLAN_API_KEY_ENV_VAR,
                    )
                    .map(|_| ())
                    .map_err(|error| {
                        format!(
                            "wallet was removed, but its plan credential could not be disconnected: {error}"
                        )
                    })
                })
                .await
                .map_err(|error| format!("credential cleanup task failed: {error}"))
                .and_then(|value| value),
                Err(error) => Err(error),
            };
            tx.send(AppEvent::WalletRemoved { result });
        });
    }

    pub(crate) fn on_wallet_removed(&mut self, result: Result<(), String>) {
        self.bottom_pane
            .dismiss_view_by_id(super::wallet_menu::WALLET_REMOVE_VIEW_ID);
        self.wallet_capability = None;
        self.wallet_balances = None;
        match result {
            Ok(()) => self.add_info_message(
                "Wallet and PfTerminal Plan credential removed from this device. On-chain funds and the paid period were not changed."
                    .to_string(),
                None,
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
                None,
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
