//! Corbanu API account UI owned by the wallet surface.

use super::*;
use crate::chatwidget::wallet_http::gateway_client;
use crate::chatwidget::wallet_http::gateway_origin;
use crate::chatwidget::wallet_menu::item;
use codex_model_provider_info::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID;
use codex_wallet::CorbanuApiAccount;
use codex_wallet::CorbanuApiBalance;
use codex_wallet::CorbanuApiModel;
use codex_wallet::CorbanuApiOperation;
use codex_wallet::CorbanuApiOperationResult;
use codex_wallet::GatewayKey;
use codex_wallet_daemon::UnlockPolicy;
use codex_wallet_daemon::WalletDaemonClient;
use serde::Deserialize;
use zeroize::Zeroize;
use zeroize::Zeroizing;

const CORBANU_API_VIEW_ID: &str = "corbanu-api";

#[derive(Debug, Clone)]
pub(crate) struct CorbanuApiView {
    pub(crate) account: Option<CorbanuApiAccount>,
    pub(crate) key_summaries_loaded: bool,
    pub(crate) notice: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyAccountResponse {
    wallet_address: String,
    corbanu_api: CorbanuApiBalance,
}

#[derive(Deserialize)]
struct ModelListResponse {
    data: Vec<CorbanuApiModel>,
}

impl ChatWidget {
    pub(crate) fn open_corbanu_api(&mut self) {
        self.show_corbanu_api_loading();
        let home = self.config.codex_home.as_path().to_path_buf();
        let capability = self
            .wallet_capability
            .as_ref()
            .map(|value| Zeroizing::new(value.to_string()));
        let credential_store_mode = self.config.cli_auth_credentials_store_mode;
        let keyring_backend = self.config.auth_keyring_backend_kind();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result =
                if let Some(capability) = capability {
                    WalletDaemonClient::new(home)
                        .execute_corbanu_api_operation(
                            capability.to_string(),
                            gateway_origin(),
                            CorbanuApiOperation::Account,
                        )
                        .await
                        .map_err(|error| error.to_string())
                        .and_then(|result| match result {
                            CorbanuApiOperationResult::Account { account } => Ok(CorbanuApiView {
                                account: Some(account),
                                key_summaries_loaded: true,
                                notice: None,
                            }),
                            _ => Err("Corbanu API returned the wrong account operation result"
                                .to_string()),
                        })
                } else {
                    load_read_only_account(home, credential_store_mode, keyring_backend).await
                };
            tx.send(AppEvent::CorbanuApiLoaded { result });
        });
    }

    fn show_corbanu_api_loading(&mut self) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Corbanu API".bold()));
        header.push(Line::from("Loading balance and at-cost prices…".dim()));
        self.show_selection_view(SelectionViewParams {
            view_id: Some(CORBANU_API_VIEW_ID),
            header: Box::new(header),
            items: vec![SelectionItem {
                name: "Loading…".to_string(),
                is_disabled: true,
                ..Default::default()
            }],
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        });
    }

    pub(crate) fn on_corbanu_api_loaded(&mut self, result: Result<CorbanuApiView, String>) {
        self.bottom_pane
            .replace_selection_view_if_present(CORBANU_API_VIEW_ID, corbanu_api_params(result));
    }

    pub(crate) fn open_corbanu_api_top_up(&mut self) {
        let tx = self.app_event_tx.clone();
        self.bottom_pane.show_view(Box::new(CustomPromptView::new(
            "Top up Corbanu API".to_string(),
            "USDC amount, for example 10 or 25.50".to_string(),
            String::new(),
            Some("One canonical USDC adds exactly one dollar of API balance.".to_string()),
            Box::new(move |amount_usd| {
                tx.send(AppEvent::ConfirmCorbanuApiTopUp { amount_usd });
            }),
        )));
    }

    pub(crate) fn confirm_corbanu_api_top_up(&mut self, amount_usd: String) {
        let amount_micros = match parse_top_up_amount(&amount_usd) {
            Ok(amount) => amount,
            Err(error) => {
                self.add_error_message(error);
                self.open_corbanu_api_top_up();
                return;
            }
        };
        let available = self.wallet_balances.map(|balance| balance.usdc_atomic);
        let affordable = available.is_some_and(|balance| balance >= amount_micros);
        let canonical_amount = format_usd_micros(amount_micros);
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Confirm Corbanu API top-up".bold()));
        header.push(Line::from(format!(
            "Pay exactly {canonical_amount} canonical USDC for ${canonical_amount} of API balance."
        )));
        header.push(Line::from(
            "This is a one-time transfer. There is no tier, renewal, or expiring allowance.".dim(),
        ));
        match available {
            Some(balance) if affordable => header.push(Line::from(format!(
                "Wallet balance: {} USDC · after payment: {} USDC",
                format_usd_micros(balance),
                format_usd_micros(balance - amount_micros),
            ))),
            Some(balance) => header.push(Line::from(
                format!(
                    "Insufficient USDC: {} available, {} required",
                    format_usd_micros(balance),
                    canonical_amount,
                )
                .red(),
            )),
            None => header.push(Line::from(
                "Wallet balance unavailable; refresh /wallet before paying.".red(),
            )),
        }
        let operation = CorbanuApiOperation::TopUpIntent {
            amount_usd: canonical_amount.clone(),
        };
        self.show_selection_view(SelectionViewParams {
            view_id: Some("corbanu-api-top-up-confirm"),
            header: Box::new(header),
            items: vec![
                SelectionItem {
                    name: "Cancel".to_string(),
                    description: Some("Return without signing or sending USDC".to_string()),
                    dismiss_on_select: true,
                    ..Default::default()
                },
                SelectionItem {
                    name: format!("Pay {canonical_amount} USDC"),
                    description: Some(
                        "Unlock if needed, then sign only this exact transfer".to_string(),
                    ),
                    is_disabled: !affordable,
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::CorbanuApiOperationRequested {
                            operation: operation.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                },
            ],
            initial_selected_idx: Some(0),
            allow_number_shortcuts: false,
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        });
    }

    pub(crate) fn confirm_corbanu_api_key_revocation(
        &mut self,
        key_id: String,
        display_prefix: String,
    ) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Revoke Corbanu API key".bold()));
        header.push(Line::from(format!(
            "Revoke {display_prefix}? Requests using this key will stop immediately."
        )));
        let operation = CorbanuApiOperation::RevokeKey { key_id };
        self.show_selection_view(SelectionViewParams {
            view_id: Some("corbanu-api-revoke-confirm"),
            header: Box::new(header),
            items: vec![
                SelectionItem {
                    name: "Cancel".to_string(),
                    dismiss_on_select: true,
                    ..Default::default()
                },
                SelectionItem {
                    name: "Revoke key".to_string(),
                    description: Some("The shared wallet balance is not changed".to_string()),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::CorbanuApiOperationRequested {
                            operation: operation.clone(),
                        });
                    })],
                    dismiss_on_select: true,
                    ..Default::default()
                },
            ],
            initial_selected_idx: Some(0),
            allow_number_shortcuts: false,
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        });
    }

    pub(crate) fn request_corbanu_api_operation(&mut self, operation: CorbanuApiOperation) {
        let Some(capability) = self
            .wallet_capability
            .as_ref()
            .map(|value| Zeroizing::new(value.to_string()))
        else {
            self.app_event_tx.send(AppEvent::OpenWalletUnlock {
                policy: UnlockPolicy::OneAction,
                continuation: crate::app_event::WalletUnlockContinuation::CorbanuApiOperation {
                    operation,
                },
            });
            return;
        };
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        self.add_info_message(
            "Submitting the wallet-authorized Corbanu API operation…".to_string(),
            None,
        );
        tokio::spawn(async move {
            let result = WalletDaemonClient::new(home)
                .execute_corbanu_api_operation(capability.to_string(), gateway_origin(), operation)
                .await
                .map_err(|error| error.to_string());
            tx.send(AppEvent::CorbanuApiOperationFinished { result });
        });
    }

    pub(crate) fn on_corbanu_api_operation_finished(
        &mut self,
        result: Result<CorbanuApiOperationResult, String>,
    ) {
        match result {
            Ok(CorbanuApiOperationResult::Account { account }) => {
                self.on_corbanu_api_loaded(Ok(CorbanuApiView {
                    account: Some(account),
                    key_summaries_loaded: true,
                    notice: None,
                }));
            }
            Ok(CorbanuApiOperationResult::TopUp {
                balance,
                api_key,
                transaction,
            }) => {
                self.add_info_message(
                    format!(
                        "Corbanu API funded. Available balance: ${}.{}",
                        balance.available_usd,
                        transaction
                            .map_or_else(String::new, |value| format!(" Settlement: {value}")),
                    ),
                    None,
                );
                if let Some(api_key) = api_key {
                    self.store_and_reveal_corbanu_api_key(api_key);
                } else {
                    self.open_corbanu_api();
                }
            }
            Ok(CorbanuApiOperationResult::KeyCreated { api_key }) => {
                self.add_info_message(
                    "Corbanu API key created. Its plaintext is available only in the secure view."
                        .to_string(),
                    None,
                );
                self.store_and_reveal_corbanu_api_key(api_key);
            }
            Ok(CorbanuApiOperationResult::KeyRevoked { .. }) => {
                self.add_info_message(
                    "Corbanu API key revoked. The shared dollar balance was unchanged.".to_string(),
                    None,
                );
                self.open_corbanu_api();
            }
            Err(error) => {
                self.wallet_capability = None;
                self.add_error_message(format!(
                    "Corbanu API wallet operation failed: {error}. No automatic retry was sent."
                ));
                self.open_corbanu_api();
            }
        }
    }

    fn store_and_reveal_corbanu_api_key(&mut self, api_key: GatewayKey) {
        let mut plaintext = api_key.api_key;
        let stored = codex_login::login_with_provider_api_key(
            &self.config.codex_home,
            PFTERMINAL_PLAN_API_KEY_ENV_VAR,
            &plaintext,
            self.config.cli_auth_credentials_store_mode,
            self.config.auth_keyring_backend_kind(),
        );
        if let Err(error) = stored {
            self.add_error_message(format!(
                "The API key was created but could not be stored in the encrypted credential store: {error}"
            ));
        } else {
            self.select_corbanu_api_model("corbanu/glm-5.3-flash");
        }
        self.bottom_pane.show_view(Box::new(
            crate::bottom_pane::vault_secret_reveal::VaultSecretRevealView::new(
                format!("Corbanu API {} — shown once", api_key.display_prefix),
                std::mem::take(&mut plaintext),
            ),
        ));
        plaintext.zeroize();
    }

    fn select_corbanu_api_model(&self, model: &str) {
        let provider = if model == "corbanu/claude-fable-5" {
            PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID
        } else {
            PFTERMINAL_PLAN_PROVIDER_ID
        };
        self.app_event_tx.send(AppEvent::UpdateModelSelection {
            model: model.to_string(),
            provider: Some(provider.to_string()),
        });
        self.app_event_tx.send(AppEvent::PersistModelSelection {
            model: model.to_string(),
            provider: Some(provider.to_string()),
            effort: None,
        });
    }
}

async fn load_read_only_account(
    home: std::path::PathBuf,
    credential_store_mode: codex_config::types::AuthCredentialsStoreMode,
    keyring_backend: codex_config::types::AuthKeyringBackendKind,
) -> Result<CorbanuApiView, String> {
    let api_key = codex_login::provider_api_key_from_auth_storage(
        &home,
        PFTERMINAL_PLAN_API_KEY_ENV_VAR,
        credential_store_mode,
        keyring_backend,
    )
    .map_err(|error| error.to_string())?
    .map(Zeroizing::new);
    let Some(api_key) = api_key else {
        return Ok(CorbanuApiView {
            account: None,
            key_summaries_loaded: false,
            notice: Some("Top up with USDC to create the first API key.".to_string()),
        });
    };
    let gateway = gateway_client()?;
    let account_response = gateway
        .client
        .get(format!("{}/v1/account", gateway.origin))
        .bearer_auth(api_key.as_str())
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if account_response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Ok(CorbanuApiView {
            account: None,
            key_summaries_loaded: false,
            notice: Some(
                "The stored Corbanu API key is no longer valid. Unlock this wallet to load its balance or create a replacement key."
                    .to_string(),
            ),
        });
    }
    if !account_response.status().is_success() {
        return Err(format!(
            "Corbanu API account returned HTTP {}",
            account_response.status()
        ));
    }
    let account = account_response
        .json::<KeyAccountResponse>()
        .await
        .map_err(|error| format!("Corbanu API account response was malformed: {error}"))?;
    let daemon = WalletDaemonClient::new(home)
        .status()
        .await
        .map_err(|error| error.to_string())?;
    if daemon.address.as_deref() != Some(account.wallet_address.as_str()) {
        return Ok(CorbanuApiView {
            account: None,
            key_summaries_loaded: false,
            notice: Some(
                "The stored Corbanu API key belongs to a different wallet. Unlock this wallet to load its account."
                    .to_string(),
            ),
        });
    }
    let models_response = gateway
        .client
        .get(format!("{}/v1/models", gateway.origin))
        .bearer_auth(api_key.as_str())
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !models_response.status().is_success() {
        return Err(format!(
            "Corbanu API model catalog returned HTTP {}",
            models_response.status()
        ));
    }
    let models = models_response
        .json::<ModelListResponse>()
        .await
        .map_err(|error| format!("Corbanu API model catalog was malformed: {error}"))?
        .data;
    Ok(CorbanuApiView {
        account: Some(CorbanuApiAccount {
            balance: account.corbanu_api,
            keys: Vec::new(),
            models,
        }),
        key_summaries_loaded: false,
        notice: Some("Unlock the wallet to list, create, or revoke API keys.".to_string()),
    })
}

fn corbanu_api_params(result: Result<CorbanuApiView, String>) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Corbanu API".bold()));
    let mut items = Vec::new();
    match result {
        Err(error) => {
            header.push(Line::from(format!("Unavailable: {error}").red()));
        }
        Ok(view) => {
            if let Some(account) = view.account {
                header.push(Line::from(
                    format!("${} available", account.balance.available_usd)
                        .cyan()
                        .bold(),
                ));
                if account.balance.reserved_microusd != "0" {
                    header.push(Line::from(format!(
                        "${} reserved by in-flight requests",
                        account.balance.reserved_usd
                    )));
                }
                header.push(Line::from(
                    "Prices are exact upstream cost with zero markup, per million tokens.".dim(),
                ));
                for model in account.models {
                    let recommended = if model.recommended {
                        " · Recommended"
                    } else {
                        ""
                    };
                    let faster = if model.balance_rate == "faster" {
                        " · uses balance faster"
                    } else {
                        ""
                    };
                    let privacy = if model.privacy == "corbanu-controlled" {
                        "Corbanu-controlled"
                    } else {
                        "Third-party inference"
                    };
                    let model_id = model.id.clone();
                    items.push(SelectionItem {
                        name: format!("{}{}{}", model.display_name, recommended, faster),
                        description: Some(format!(
                            "${} input · ${} cached input · ${} output · {privacy}",
                            model.pricing.input_usd,
                            model.pricing.cache_read_usd,
                            model.pricing.output_usd,
                        )),
                        actions: vec![Box::new(move |tx| {
                            let provider = if model_id == "corbanu/claude-fable-5" {
                                PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID
                            } else {
                                PFTERMINAL_PLAN_PROVIDER_ID
                            };
                            tx.send(AppEvent::UpdateModelSelection {
                                model: model_id.clone(),
                                provider: Some(provider.to_string()),
                            });
                            tx.send(AppEvent::PersistModelSelection {
                                model: model_id.clone(),
                                provider: Some(provider.to_string()),
                                effort: None,
                            });
                        })],
                        ..Default::default()
                    });
                }
                if view.key_summaries_loaded {
                    for key in account
                        .keys
                        .into_iter()
                        .filter(|key| key.revoked_at.is_none())
                    {
                        let key_id = key.id.clone();
                        let prefix = key.display_prefix.clone();
                        items.push(SelectionItem {
                            name: format!("API key {}", key.display_prefix),
                            description: Some(match key.last_used_at {
                                Some(last_used) => {
                                    format!("Last used {last_used} · select to revoke")
                                }
                                None => "Never used · select to revoke".to_string(),
                            }),
                            actions: vec![Box::new(move |tx| {
                                tx.send(AppEvent::ConfirmCorbanuApiKeyRevocation {
                                    key_id: key_id.clone(),
                                    display_prefix: prefix.clone(),
                                });
                            })],
                            ..Default::default()
                        });
                    }
                }
            } else {
                header.push(Line::from("$0 available".cyan().bold()));
                header.push(Line::from(
                    "No plan tier is required. Add any positive canonical-USDC amount.".dim(),
                ));
            }
            if let Some(notice) = view.notice {
                header.push(Line::from(notice.dim()));
            }
        }
    }
    items.insert(
        0,
        item(
            "Top up balance",
            "Choose any positive canonical-USDC amount",
            || AppEvent::OpenCorbanuApiTopUp,
        ),
    );
    items.insert(
        1,
        item(
            "Manage API keys",
            "Unlock to load key summaries and balances",
            || AppEvent::CorbanuApiOperationRequested {
                operation: CorbanuApiOperation::Account,
            },
        ),
    );
    items.insert(
        2,
        item(
            "Create API key",
            "Unlock and reveal a new key exactly once",
            || AppEvent::CorbanuApiOperationRequested {
                operation: CorbanuApiOperation::CreateKey,
            },
        ),
    );
    items.push(item("Refresh", "Reload balance and model prices", || {
        AppEvent::OpenCorbanuApi
    }));
    SelectionViewParams {
        view_id: Some(CORBANU_API_VIEW_ID),
        header: Box::new(header),
        items,
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

fn parse_top_up_amount(value: &str) -> Result<u64, String> {
    let trimmed = value.trim();
    let value = trimmed.strip_prefix('$').unwrap_or(trimmed);
    let (whole, fractional) = value
        .split_once('.')
        .map_or((value, ""), |(whole, fractional)| (whole, fractional));
    if whole.is_empty()
        || (value.contains('.') && fractional.is_empty())
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fractional.len() > 6
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("Enter a positive USDC amount with at most 6 decimal places.".to_string());
    }
    let whole = whole
        .parse::<u64>()
        .map_err(|_| "The USDC amount is too large.".to_string())?;
    let fractional = format!("{fractional:0<6}")
        .parse::<u64>()
        .map_err(|_| "The USDC amount is invalid.".to_string())?;
    let amount = whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fractional))
        .ok_or_else(|| "The USDC amount is too large.".to_string())?;
    if amount == 0 {
        return Err("The USDC amount must be greater than zero.".to_string());
    }
    Ok(amount)
}

fn format_usd_micros(value: u64) -> String {
    let whole = value / 1_000_000;
    let fraction = value % 1_000_000;
    if fraction == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{fraction:06}")
            .trim_end_matches('0')
            .to_string()
    }
}

#[cfg(test)]
#[path = "wallet_api_tests.rs"]
mod tests;
