use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_wallet::Wallet;
use zeroize::Zeroizing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PfTerminalPlanStatus {
    Checking,
    Active {
        plan_id: String,
        wallet_address: String,
    },
    WalletOnly {
        wallet_address: String,
    },
    ConnectedStatusUnavailable {
        wallet_address: Option<String>,
    },
    NotConnected,
    Unavailable,
}

pub(crate) async fn load(
    codex_home: std::path::PathBuf,
    auth_store_mode: codex_login::AuthCredentialsStoreMode,
    keyring_backend_kind: codex_login::AuthKeyringBackendKind,
) -> PfTerminalPlanStatus {
    let context_home = codex_home.clone();
    let context = tokio::task::spawn_blocking(move || {
        let key = codex_login::provider_api_key_from_auth_storage(
            &context_home,
            PFTERMINAL_PLAN_API_KEY_ENV_VAR,
            auth_store_mode,
            keyring_backend_kind,
        )
        .map(|value| value.map(Zeroizing::new));
        let wallet_address = Wallet::new(context_home)
            .manifest()
            .ok()
            .map(|manifest| manifest.address);
        (key, wallet_address)
    })
    .await;
    let Ok((Ok(key), wallet_address)) = context else {
        return PfTerminalPlanStatus::Unavailable;
    };
    let Some(key) = key else {
        return wallet_address.map_or(PfTerminalPlanStatus::NotConnected, |wallet_address| {
            PfTerminalPlanStatus::WalletOnly { wallet_address }
        });
    };
    let Ok(gateway) = super::wallet_http::gateway_client() else {
        return PfTerminalPlanStatus::ConnectedStatusUnavailable { wallet_address };
    };
    let response = gateway
        .client
        .get(format!("{}/v1/account", gateway.origin))
        .bearer_auth(key.as_str())
        .send()
        .await;
    let Ok(response) = response else {
        return PfTerminalPlanStatus::ConnectedStatusUnavailable { wallet_address };
    };
    if !response.status().is_success() {
        return PfTerminalPlanStatus::ConnectedStatusUnavailable { wallet_address };
    }
    match response
        .json::<super::wallet_menu::WalletPlanStatus>()
        .await
    {
        Ok(status) => PfTerminalPlanStatus::Active {
            plan_id: status.period.plan_id,
            wallet_address: status.wallet_address,
        },
        Err(_) => PfTerminalPlanStatus::ConnectedStatusUnavailable { wallet_address },
    }
}

pub(crate) fn description(status: &PfTerminalPlanStatus) -> String {
    match status {
        PfTerminalPlanStatus::Checking => "Checking plan...".to_string(),
        PfTerminalPlanStatus::Active {
            plan_id,
            wallet_address,
        } => format!(
            "Active · {} plan · {}",
            super::wallet_menu::title_case_plan(plan_id),
            super::wallet_menu::short_address(wallet_address)
        ),
        PfTerminalPlanStatus::WalletOnly { wallet_address } => format!(
            "Wallet connected · no plan credential · {}",
            super::wallet_menu::short_address(wallet_address)
        ),
        PfTerminalPlanStatus::ConnectedStatusUnavailable { wallet_address } => wallet_address
            .as_deref()
            .map(super::wallet_menu::short_address)
            .map_or_else(
                || "Connected · plan status unavailable".to_string(),
                |address| format!("Connected · status unavailable · {address}"),
            ),
        PfTerminalPlanStatus::NotConnected => "Not connected".to_string(),
        PfTerminalPlanStatus::Unavailable => "Status unavailable".to_string(),
    }
}
