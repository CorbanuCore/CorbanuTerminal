use super::*;
use crate::app_event::WalletCreatedResult;
use crate::app_event::WalletPersistenceOperation;
use crate::app_event::WalletPlanCredentialPersistenceError;
use crate::app_event::WalletPlanPersistenceAttemptId;
use crate::app_event::WalletPlanProvisionedResult;
use crate::app_event::WalletPlanProvisioningOperation;
use crate::app_event::WalletPlanPurchaseSummary;
use crate::app_event::WalletPlanReceiptSelectionPolicy;
use crate::app_event::WalletSecret;
use crate::chatwidget::wallet_http::gateway_client;
use crate::chatwidget::wallet_http::gateway_origin;
use crate::chatwidget::wallet_receipt::WalletPlanReceipt;
use crate::chatwidget::wallet_receipt::latest_plan_receipt;
use crate::chatwidget::wallet_receipt::reconcile_plan_receipt;
use crate::chatwidget::wallet_render::WalletTextStyle;
use crate::chatwidget::wallet_render::push_wallet_text;
use codex_model_provider_info::AMBIENT_DEFAULT_MODEL;
use codex_model_provider_info::PFTERMINAL_PLAN_API_KEY_ENV_VAR;
use codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID;
use codex_wallet::BalanceClient;
use codex_wallet::Network;
use codex_wallet::PlanPurchaseIntent;
use codex_wallet::Wallet;
use codex_wallet::WalletBalances;
use codex_wallet_daemon::DaemonStatus;
use codex_wallet_daemon::UnlockPolicy;
use codex_wallet_daemon::WalletDaemonClient;
use zeroize::Zeroize;
use zeroize::Zeroizing;

pub(super) const WALLET_MENU_VIEW_ID: &str = "wallet-menu";
const WALLET_PLANS_VIEW_ID: &str = "wallet-plans";
const WALLET_PLAN_CONFIRM_VIEW_ID: &str = "wallet-plan-confirm";
const SHARED_PROVIDER_SETUP_VIEW_ID: &str = "shared-provider-setup";
const SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID: &str = "shared-provider-account-auth";
pub(super) const WALLET_DISCONNECT_PLAN_VIEW_ID: &str = "wallet-disconnect-plan";
pub(super) const WALLET_REMOVE_VIEW_ID: &str = "wallet-remove";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletPlanChoice {
    pub(crate) id: String,
    pub(crate) price_usdc: String,
    pub(crate) amount_atomic: String,
    pub(crate) weekly_token_limit: u64,
    pub(crate) monthly_token_limit: u64,
    #[serde(skip)]
    pub(crate) scheduled_start: Option<String>,
    #[serde(skip)]
    pub(crate) deferred_setup: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WalletPlanPurchaseMode {
    New,
    Onboarding {
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    },
    Upgrade {
        current_plan_id: String,
        starts_at: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletPaymentConfig {
    pub(crate) network: String,
    pub(crate) asset: String,
    pub(crate) pay_to: String,
    pub(crate) rpc_url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct WalletPlanCatalog {
    pub(crate) plans: Vec<WalletPlanChoice>,
    pub(crate) payment: WalletPaymentConfig,
}

#[derive(Debug, Clone)]
pub(crate) struct WalletOverview {
    pub(crate) daemon: DaemonStatus,
    pub(crate) balances: Option<WalletBalances>,
    pub(crate) balance_error: Option<String>,
    pub(crate) plan: Option<WalletPlanStatus>,
    pub(crate) linked_plan_for_other_wallet: Option<WalletPlanStatus>,
    pub(crate) plan_error: Option<String>,
    pub(crate) plan_credential_present: bool,
    pub(crate) plan_prices_usdc: std::collections::BTreeMap<String, String>,
}

fn wallet_balance_endpoint(
    wallet_network: &str,
    catalog: Option<&WalletPlanCatalog>,
) -> (String, codex_wallet::Network) {
    let (caip_network, fallback_rpc, network) = match wallet_network {
        "devnet" => (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
            "https://api.devnet.solana.com",
            codex_wallet::Network::Devnet,
        ),
        _ => (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            "https://api.mainnet-beta.solana.com",
            codex_wallet::Network::Mainnet,
        ),
    };
    let rpc = catalog
        .filter(|catalog| catalog.payment.network == caip_network)
        .map(|catalog| catalog.payment.rpc_url.clone())
        .unwrap_or_else(|| fallback_rpc.to_string());
    (rpc, network)
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletPlanStatus {
    pub(crate) wallet_address: String,
    pub(crate) period: WalletPlanPeriod,
    pub(crate) weekly: WalletUsageWindow,
    pub(crate) monthly_remaining_tokens: u64,
    pub(crate) weekly_remaining_tokens: u64,
    #[serde(default)]
    pub(crate) queued_periods: Vec<WalletQueuedPlanPeriod>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletQueuedPlanPeriod {
    pub(crate) transaction: String,
    pub(crate) plan_id: String,
    pub(crate) starts_at: String,
    pub(crate) ends_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletPlanPeriod {
    pub(crate) transaction: String,
    pub(crate) plan_id: String,
    pub(crate) starts_at: String,
    pub(crate) ends_at: String,
    pub(crate) monthly_limit_tokens: u64,
    pub(crate) monthly_used_tokens: u64,
    pub(crate) monthly_reserved_tokens: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletUsageWindow {
    pub(crate) ends_at: String,
    pub(crate) limit_tokens: u64,
    pub(crate) used_tokens: u64,
    pub(crate) reserved_tokens: u64,
}

impl ChatWidget {
    pub(crate) fn has_wallet_signing_capability(&self) -> bool {
        self.wallet_capability.is_some()
    }

    pub(crate) fn open_shared_provider_setup(
        &mut self,
        host: &crate::provider_status_host::ProviderStatusHost,
        queued_corbanu: bool,
        can_finish: bool,
    ) {
        let statuses = host.resolve();
        let mut items = Vec::new();
        for entry in host.catalog().entries() {
            let status = statuses.get(entry.id.as_str());
            let has_noninteractive_capability = entry.setup_capabilities.iter().any(|capability| {
                matches!(
                    capability,
                    codex_provider_auth::ProviderSetupCapability::Local { .. }
                        | codex_provider_auth::ProviderSetupCapability::CommandAuth { .. }
                        | codex_provider_auth::ProviderSetupCapability::StatusOnly { .. }
                )
            });
            for capability in entry.setup_capabilities.iter() {
                let provider_id = entry.id.clone();
                let selected = capability.clone();
                let method = shared_provider_method_label(capability);
                let mut item = SelectionItem {
                    name: format!("{} — {method}", entry.display_name),
                    description: Some(shared_provider_status_description(status)),
                    search_value: Some(format!("{} {method}", entry.display_name)),
                    ..Default::default()
                };
                match capability {
                    codex_provider_auth::ProviderSetupCapability::OpenAiAccount
                    | codex_provider_auth::ProviderSetupCapability::ApiKey { .. }
                    | codex_provider_auth::ProviderSetupCapability::ClaudeAccount => {
                        item.actions.push(Box::new(move |tx| {
                            tx.send(AppEvent::SharedProviderSetupBegin {
                                provider_id: provider_id.clone(),
                                capability: selected.clone(),
                            });
                        }));
                    }
                    codex_provider_auth::ProviderSetupCapability::CorbanuPlan => {
                        item.name = if queued_corbanu {
                            format!("{} — queued after Done", entry.display_name)
                        } else {
                            format!("{} — queue after Done", entry.display_name)
                        };
                        item.actions.push(Box::new(move |tx| {
                            tx.send(AppEvent::SharedProviderSetupQueueCorbanu(!queued_corbanu));
                        }));
                    }
                    codex_provider_auth::ProviderSetupCapability::Local { .. }
                    | codex_provider_auth::ProviderSetupCapability::CommandAuth { .. }
                    | codex_provider_auth::ProviderSetupCapability::StatusOnly { .. } => {
                        if status.is_some_and(
                            crate::onboarding::provider_setup::provider_is_explicitly_selectable,
                        ) && let Some(runtime_provider_id) =
                            entry.runtime_provider_ids.first().cloned()
                        {
                            item.description = Some(format!(
                                "{} · select as current provider",
                                shared_provider_status_description(status)
                            ));
                            item.actions.push(Box::new(move |tx| {
                                tx.send(AppEvent::SharedProviderSetupSelectExisting {
                                    provider_id: provider_id.clone(),
                                    runtime_provider_id: runtime_provider_id.clone(),
                                });
                            }));
                        } else {
                            item.is_disabled = true;
                        }
                    }
                }
                items.push(item);
            }
            if !has_noninteractive_capability
                && status.is_some_and(|status| {
                    crate::onboarding::provider_setup::provider_should_offer_existing_selection(
                        status,
                        has_noninteractive_capability,
                    )
                })
                && let Some(runtime_provider_id) = entry.runtime_provider_ids.first().cloned()
            {
                let provider_id = entry.id.clone();
                items.push(SelectionItem {
                    name: format!("{} — use existing configuration", entry.display_name),
                    description: Some(
                        "Select as current provider without re-authenticating".to_string(),
                    ),
                    search_value: Some(format!("{} use existing", entry.display_name)),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::SharedProviderSetupSelectExisting {
                            provider_id: provider_id.clone(),
                            runtime_provider_id: runtime_provider_id.clone(),
                        });
                    })],
                    ..Default::default()
                });
            }
        }
        items.push(SelectionItem {
            name: "Done".to_string(),
            description: Some(if can_finish {
                "Finish provider setup".to_string()
            } else {
                "Configure a usable provider or queue Corbanu Plan".to_string()
            }),
            is_disabled: !can_finish,
            actions: vec![Box::new(|tx| tx.send(AppEvent::SharedProviderSetupDone))],
            ..Default::default()
        });
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Set up providers".bold()));
        header.push(Line::from(
            "Configure more than one provider, then choose Done.".dim(),
        ));
        self.show_selection_view(SelectionViewParams {
            view_id: Some(SHARED_PROVIDER_SETUP_VIEW_ID),
            header: Box::new(header),
            items,
            is_searchable: true,
            search_placeholder: Some("Search providers".to_string()),
            footer_hint: Some(standard_popup_hint_line()),
            on_cancel: Some(Box::new(|tx| {
                tx.send(AppEvent::SharedProviderSetupCancelled);
            })),
            ..Default::default()
        });
    }

    pub(crate) fn open_shared_provider_api_key(
        &mut self,
        target: codex_provider_auth::ApiKeyAuthTarget,
        display_name: String,
    ) {
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            format!("provider:{}", target.provider_id),
            format!("Add {display_name}"),
            "API key — masked".to_string(),
            "Stored through the selected provider credential backend".to_string(),
            Box::new(move |_label, secret| {
                tx.send(AppEvent::SaveSharedProviderApiKey {
                    target: target.clone(),
                    api_key: crate::app_event::ProviderApiKeySecret::new(secret),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_shared_openai_challenge(
        &mut self,
        challenge: codex_provider_auth::OpenAiAccountChallenge,
    ) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("OpenAI account login".bold()));
        if let Some((verification_url, user_code)) = challenge.device_code_values() {
            header.push(Line::from(format!("Open: {verification_url}")));
            header.push(Line::from(format!("Code: {user_code}").cyan().bold()));
        } else if let Some(auth_url) = challenge.browser_auth_url() {
            header.push(Line::from(format!("Open: {auth_url}")));
        }
        self.show_shared_account_auth_selection(SelectionViewParams {
            header: Box::new(header),
            items: vec![SelectionItem {
                name: "Cancel login".to_string(),
                actions: vec![Box::new(|tx| {
                    tx.send(AppEvent::SharedProviderAuthAction(
                        codex_provider_auth::OpenAiAccountAction::Cancel.into(),
                    ))
                })],
                ..Default::default()
            }],
            on_cancel: Some(Box::new(|tx| {
                tx.send(AppEvent::SharedProviderAuthAction(
                    codex_provider_auth::OpenAiAccountAction::Cancel.into(),
                ));
            })),
            ..Default::default()
        });
    }

    pub(crate) fn open_shared_account_pending(
        &mut self,
        kind: crate::provider_account_auth_host::ProviderAccountCancelKind,
    ) {
        let cancel_action = move || kind.action();
        self.show_shared_account_auth_selection(SelectionViewParams {
            title: Some("Starting account authentication".to_string()),
            items: vec![SelectionItem {
                name: "Cancel".to_string(),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::SharedProviderAuthAction(cancel_action()))
                })],
                ..Default::default()
            }],
            on_cancel: Some(Box::new(move |tx| {
                let action = kind.action();
                tx.send(AppEvent::SharedProviderAuthAction(action));
            })),
            ..Default::default()
        });
    }

    pub(crate) fn open_shared_claude_method_choice(&mut self) {
        use codex_provider_auth::claude_account_flow::ClaudeAccountAction;
        use codex_provider_auth::claude_account_flow::ClaudeAccountMethod;
        let method_item = |name: &str, method| SelectionItem {
            name: name.to_string(),
            actions: vec![Box::new(move |tx: &AppEventSender| {
                tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::ChooseMethod(method).into(),
                ));
                if method == ClaudeAccountMethod::ClaudeCodeLogin {
                    tx.send(AppEvent::SharedProviderAuthAction(
                        ClaudeAccountAction::Submit.into(),
                    ));
                }
            })],
            ..Default::default()
        };
        self.show_shared_account_auth_selection(SelectionViewParams {
            title: Some("Claude account method".to_string()),
            items: vec![
                method_item(
                    "Managed subscription token",
                    ClaudeAccountMethod::ManagedToken,
                ),
                method_item("Claude Code login", ClaudeAccountMethod::ClaudeCodeLogin),
            ],
            on_cancel: Some(Box::new(|tx| {
                tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::Cancel.into(),
                ));
            })),
            ..Default::default()
        });
    }

    pub(crate) fn open_shared_claude_managed_token_entry(&mut self) {
        use codex_provider_auth::claude_account_flow::ClaudeAccountAction;
        self.bottom_pane
            .dismiss_view_by_id(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID);
        let tx = self.app_event_tx.clone();
        let cancel_tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret_with_cancel(
            "claude-managed-token".into(),
            "Claude managed subscription token".into(),
            "Token — masked".into(),
            "Paste the long-lived subscription token".into(),
            Box::new(move |_, secret| {
                tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::SetManagedToken(
                        codex_provider_auth::claude_account_flow::ClaudeManagedTokenSecret::new(secret),
                    ).into(),
                ));
                tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::Submit.into(),
                ));
            }),
            Box::new(move || {
                cancel_tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::Cancel.into(),
                ));
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_shared_claude_challenge(
        &mut self,
        challenge: codex_provider_auth::claude_account_flow::ClaudeCodeChallenge,
    ) {
        use codex_provider_auth::claude_account_flow::ClaudeAccountAction;
        self.bottom_pane
            .dismiss_view_by_id(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID);
        let tx = self.app_event_tx.clone();
        let cancel_tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret_with_cancel(
            "claude-authorization-code".into(),
            "Claude Code authorization".into(),
            format!("Open {}", challenge.verification_url()),
            "Paste the authorization code — masked".into(),
            Box::new(move |_, secret| {
                tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::SubmitAuthorizationCode(
                        codex_provider_auth::claude_account_flow::ClaudeAuthorizationCodeSecret::new(secret),
                    ).into(),
                ));
            }),
            Box::new(move || {
                cancel_tx.send(AppEvent::SharedProviderAuthAction(
                    ClaudeAccountAction::Cancel.into(),
                ));
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    fn show_shared_account_auth_selection(&mut self, mut params: SelectionViewParams) {
        params.view_id = Some(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID);
        if self.bottom_pane.active_view_id() == Some(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID) {
            let replaced = self
                .bottom_pane
                .replace_selection_view_if_active(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID, params);
            debug_assert!(replaced);
        } else {
            self.bottom_pane
                .dismiss_view_by_id(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID);
            self.show_selection_view(params);
        }
    }

    pub(crate) fn open_wallet_menu(&mut self) {
        let params = wallet_params(/*result*/ None, self.wallet_capability.is_some());
        if !self
            .bottom_pane
            .replace_selection_view_if_present(WALLET_MENU_VIEW_ID, params)
        {
            self.show_selection_view(wallet_params(
                /*result*/ None,
                self.wallet_capability.is_some(),
            ));
        }
        self.refresh_wallet_status();
    }

    pub(crate) fn refresh_wallet_status(&mut self) {
        self.wallet_status_generation = self.wallet_status_generation.wrapping_add(1);
        let generation = self.wallet_status_generation;
        let home = self.config.codex_home.as_path().to_path_buf();
        let plan_key = codex_login::provider_api_key_from_auth_storage(
            &home,
            PFTERMINAL_PLAN_API_KEY_ENV_VAR,
            self.config.cli_auth_credentials_store_mode,
            self.config.auth_keyring_backend_kind(),
        )
        .ok()
        .flatten()
        .map(Zeroizing::new);
        let plan_credential_present = plan_key.is_some();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = async {
                let daemon = WalletDaemonClient::new(home)
                    .status()
                    .await
                    .map_err(|error| error.to_string())?;
                let plan_request = async {
                    if let Some(key) = plan_key {
                        let gateway = match gateway_client() {
                            Ok(gateway) => gateway,
                            Err(error) => {
                                return (None, Some(format!("plan status unavailable: {error}")));
                            }
                        };
                        match gateway
                            .client
                            .get(format!("{}/v1/account", gateway.origin))
                            .bearer_auth(key.as_str())
                            .send()
                            .await
                        {
                            Ok(response) if response.status().is_success() => {
                                match response.json::<WalletPlanStatus>().await {
                                    Ok(status) => (Some(status), None),
                                    Err(error) => {
                                        (None, Some(format!("plan status was malformed: {error}")))
                                    }
                                }
                            }
                            Ok(response) => (
                                None,
                                Some(format!("plan status returned HTTP {}", response.status())),
                            ),
                            Err(error) => (None, Some(format!("plan status unavailable: {error}"))),
                        }
                    } else {
                        (None, None)
                    }
                };
                let catalog_request = async {
                    let Ok(gateway) = gateway_client() else {
                        return None;
                    };
                    match gateway
                        .client
                        .get(format!("{}/v1/plans", gateway.origin))
                        .send()
                        .await
                    {
                        Ok(response) if response.status().is_success() => {
                            response.json::<WalletPlanCatalog>().await.ok()
                        }
                        _ => None,
                    }
                };
                let ((plan, plan_error), catalog) = tokio::join!(plan_request, catalog_request);
                let (plan, linked_plan_for_other_wallet) =
                    separate_plan_for_local_wallet(plan, daemon.address.as_deref());
                let (balances, balance_error) = if let (Some(address), Some(network)) =
                    (daemon.address.as_deref(), daemon.network.as_deref())
                {
                    let (rpc, network) = wallet_balance_endpoint(network, catalog.as_ref());
                    match BalanceClient::new(rpc, network) {
                        Ok(client) => match client.balances(address).await {
                            Ok(value) => (Some(value), None),
                            Err(error) => (None, Some(error.to_string())),
                        },
                        Err(error) => (None, Some(error.to_string())),
                    }
                } else {
                    (None, None)
                };
                let plan_prices_usdc = catalog
                    .map(|catalog| {
                        catalog
                            .plans
                            .into_iter()
                            .map(|plan| (plan.id, plan.price_usdc))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(WalletOverview {
                    daemon,
                    balances,
                    balance_error,
                    plan,
                    linked_plan_for_other_wallet,
                    plan_error,
                    plan_credential_present,
                    plan_prices_usdc,
                })
            }
            .await;
            tx.send(AppEvent::WalletStatusReady { generation, result });
        });
    }

    pub(crate) fn on_wallet_status_ready(
        &mut self,
        generation: u64,
        result: Result<WalletOverview, String>,
    ) {
        if generation != self.wallet_status_generation {
            return;
        }
        if let Ok(overview) = &result {
            self.wallet_balances = overview.balances;
        }
        let selected = self
            .bottom_pane
            .selected_index_for_active_view(WALLET_MENU_VIEW_ID);
        let mut params = wallet_params(Some(result), self.wallet_capability.is_some());
        params.initial_selected_idx = selected;
        self.bottom_pane
            .replace_selection_view_if_present(WALLET_MENU_VIEW_ID, params);
    }

    pub(crate) fn open_wallet_create(&mut self) {
        self.open_wallet_create_for_deferred(/*deferred*/ None);
    }

    pub(crate) fn open_deferred_wallet_create(
        &mut self,
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    ) {
        self.open_wallet_create_for_deferred(Some(deferred));
    }

    fn open_wallet_create_for_deferred(
        &mut self,
        deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    ) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        let cancel_tx = self.app_event_tx.clone();
        let cancelled = deferred;
        let submit = Box::new(move |_label: String, mut passcode: String| {
            tokio::spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    let result = Wallet::new(home).create(&passcode, Network::Mainnet);
                    passcode.zeroize();
                    result
                        .map(|created| WalletCreatedResult {
                            address: created.manifest.address,
                            recovery: WalletSecret::new(created.recovery_material.to_string()),
                        })
                        .map_err(|error| error.to_string())
                })
                .await
                .map_err(|error| format!("wallet task failed: {error}"))
                .and_then(|value| value);
                tx.send(AppEvent::WalletCreateFinished {
                    operation: WalletPersistenceOperation::Create,
                    result,
                });
            });
        });
        let view = if let Some(deferred) = cancelled {
            crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_confirmed_secret_with_cancel(
                "wallet-passcode".to_string(),
                "Create Solana wallet".to_string(),
                "Wallet passcode — masked".to_string(),
                "Passcode (6+ characters; use 12+ for portable recovery)".to_string(),
                submit,
                Box::new(move || {
                    cancel_tx.send(AppEvent::DeferredCorbanuPlanCancelled { deferred });
                }),
            )
        } else {
            crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_confirmed_secret(
                "wallet-passcode".to_string(),
                "Create Solana wallet".to_string(),
                "Wallet passcode — masked".to_string(),
                "Passcode (6+ characters; use 12+ for portable recovery)".to_string(),
                submit,
            )
        };
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_wallet_restore(&mut self) {
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            "wallet-recovery".to_string(),
            "Restore Solana wallet".to_string(),
            "Recovery material — masked".to_string(),
            "Recovery material (masked — never stored in chat)".to_string(),
            Box::new(move |_label, recovery| {
                tx.send(AppEvent::OpenWalletRestorePasscode {
                    recovery: WalletSecret::new(recovery),
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn open_wallet_restore_passcode(&mut self, recovery: WalletSecret) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        let view =
            crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_confirmed_secret(
                "wallet-passcode".to_string(),
                "Protect restored wallet".to_string(),
                "New wallet passcode — masked".to_string(),
                "New passcode (6+ characters; use 12+ for portable recovery)".to_string(),
                Box::new(move |_label, mut passcode| {
                    let mut recovery = recovery.into_inner();
                    tokio::spawn(async move {
                        let result = tokio::task::spawn_blocking(move || {
                            let result =
                                Wallet::new(home).restore(&recovery, &passcode, Network::Mainnet);
                            recovery.zeroize();
                            passcode.zeroize();
                            result
                                .map(|created| WalletCreatedResult {
                                    address: created.manifest.address,
                                    recovery: WalletSecret::new(
                                        created.recovery_material.to_string(),
                                    ),
                                })
                                .map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| format!("wallet task failed: {error}"))
                        .and_then(|value| value);
                        tx.send(AppEvent::WalletCreateFinished {
                            operation: WalletPersistenceOperation::Restore,
                            result,
                        });
                    });
                }),
            );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn on_wallet_create_finished(
        &mut self,
        operation: WalletPersistenceOperation,
        result: Result<WalletCreatedResult, String>,
    ) {
        match result {
            Ok(created) => {
                self.add_info_message(
                    wallet_persistence_success_message(operation, &created.address),
                    /*hint*/ None,
                );
                self.bottom_pane.show_view(Box::new(
                    crate::bottom_pane::wallet_recovery::WalletRecoveryView::new(
                        created.address,
                        created.recovery.into_inner(),
                    ),
                ));
                // Replace the pre-create wallet snapshot underneath the recovery view. The
                // generation guard prevents an older in-flight read from restoring stale state.
                self.refresh_wallet_status();
            }
            Err(error) => self.add_error_message(format!(
                "{} failed: {error}",
                wallet_persistence_action_label(operation)
            )),
        }
    }

    pub(crate) fn on_deferred_wallet_create_finished(
        &mut self,
        operation: WalletPersistenceOperation,
        result: Result<WalletCreatedResult, String>,
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    ) {
        match result {
            Ok(created) => {
                self.add_info_message(
                    wallet_persistence_success_message(operation, &created.address),
                    /*hint*/ None,
                );
                let tx = self.app_event_tx.clone();
                let cancel_tx = self.app_event_tx.clone();
                let next = deferred.clone();
                let cancelled = deferred;
                self.bottom_pane.show_view(Box::new(
                    crate::bottom_pane::wallet_recovery::WalletRecoveryView::new(
                        created.address,
                        created.recovery.into_inner(),
                    )
                    .with_confirmation(Box::new(move || {
                        tx.send(AppEvent::OpenWalletUnlock {
                            policy: codex_wallet_daemon::UnlockPolicy::Timed {
                                duration_seconds: 300,
                            },
                            continuation: crate::app_event::WalletUnlockContinuation::OpenPlans {
                                mode: WalletPlanPurchaseMode::Onboarding { deferred: next },
                            },
                        });
                    }))
                    .with_cancellation(Box::new(move || {
                        cancel_tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                            deferred: cancelled,
                        });
                    })),
                ));
                self.refresh_wallet_status();
            }
            Err(error) => {
                self.add_error_message(format!(
                    "{} failed: {error}",
                    wallet_persistence_action_label(operation)
                ));
                self.show_deferred_wallet_preflight_retry(deferred);
            }
        }
    }

    pub(crate) fn show_deferred_wallet_preflight_retry(
        &mut self,
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    ) {
        let retry = deferred.clone();
        let cancel = deferred.clone();
        self.show_selection_view(SelectionViewParams {
            title: Some("Continue Corbanu Plan setup".to_string()),
            items: vec![
                SelectionItem {
                    name: "Retry wallet setup".to_string(),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::BeginDeferredCorbanuPlan {
                            deferred: retry.clone(),
                        })
                    })],
                    ..Default::default()
                },
                SelectionItem {
                    name: "Cancel".to_string(),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                            deferred: cancel.clone(),
                        })
                    })],
                    ..Default::default()
                },
            ],
            on_cancel: Some(Box::new(move |tx| {
                tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                    deferred: deferred.clone(),
                });
            })),
            ..Default::default()
        });
    }

    pub(crate) fn open_wallet_recovery_backup(&mut self) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            "wallet-passcode".to_string(),
            "Back up recovery material".to_string(),
            "Wallet passcode — masked".to_string(),
            "Fresh wallet passcode (masked)".to_string(),
            Box::new(move |_label, mut passcode| {
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        let result = Wallet::new(home).export_recovery(&passcode);
                        passcode.zeroize();
                        result
                            .map(|backup| WalletCreatedResult {
                                address: backup.manifest.address,
                                recovery: WalletSecret::new(backup.recovery_material.to_string()),
                            })
                            .map_err(|error| error.to_string())
                    })
                    .await
                    .map_err(|error| format!("wallet task failed: {error}"))
                    .and_then(|value| value);
                    tx.send(AppEvent::WalletRecoveryBackupFinished { result });
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    pub(crate) fn on_wallet_recovery_backup_finished(
        &mut self,
        result: Result<WalletCreatedResult, String>,
    ) {
        match result {
            Ok(backup) => {
                let tx = self.app_event_tx.clone();
                self.bottom_pane.show_view(Box::new(
                    crate::bottom_pane::wallet_recovery::WalletRecoveryView::new(
                        backup.address,
                        backup.recovery.into_inner(),
                    )
                    .with_confirmation(Box::new(move || {
                        tx.send(AppEvent::InsertHistoryCell(Box::new(
                            history_cell::new_info_event(
                                "Recovery backup acknowledged. The secure view was cleared."
                                    .to_string(),
                                /*hint*/ None,
                            ),
                        )));
                    })),
                ));
            }
            Err(error) => self.add_error_message(format!(
                "Recovery backup failed: {error}. The wallet and its signing state were unchanged."
            )),
        }
    }

    pub(crate) fn lock_wallet(&mut self) {
        self.wallet_capability = None;
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let client = WalletDaemonClient::new(home);
            let cell: Box<dyn HistoryCell> = match client.status().await {
                Ok(status) if !status.wallet_exists => Box::new(history_cell::new_info_event(
                    "No local wallet exists; there is nothing to lock.".to_string(),
                    /*hint*/ None,
                )),
                Ok(_) => match client.lock().await {
                    Ok(()) => Box::new(history_cell::new_info_event(
                        "Wallet locked in every Corbanu Terminal process.".to_string(),
                        /*hint*/ None,
                    )),
                    Err(error) => Box::new(history_cell::new_error_event(format!(
                        "Wallet lock failed: {error}"
                    ))),
                },
                Err(error) => Box::new(history_cell::new_error_event(format!(
                    "Wallet lock failed while checking wallet state: {error}"
                ))),
            };
            tx.send(AppEvent::InsertHistoryCell(cell));
            tx.send(AppEvent::OpenWallet);
        });
    }

    pub(crate) fn open_wallet_plans(&mut self, mode: WalletPlanPurchaseMode) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Corbanu Plans".bold()));
        let mut params = SelectionViewParams {
            view_id: Some(WALLET_PLANS_VIEW_ID),
            header: Box::new(header),
            items: vec![SelectionItem {
                name: "Loading plans…".to_string(),
                is_disabled: true,
                ..Default::default()
            }],
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        };
        if let WalletPlanPurchaseMode::Onboarding { deferred } = &mode {
            let deferred = deferred.clone();
            params.on_cancel = Some(Box::new(move |tx: &AppEventSender| {
                tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                    deferred: deferred.clone(),
                });
            }));
        }
        self.show_selection_view(params);
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = async {
                let gateway = gateway_client()?;
                let response = gateway
                    .client
                    .get(format!("{}/v1/plans", gateway.origin))
                    .send()
                    .await
                    .map_err(|error| error.to_string())?;
                if !response.status().is_success() {
                    return Err(format!("plan service returned HTTP {}", response.status()));
                }
                response
                    .json::<WalletPlanCatalog>()
                    .await
                    .map_err(|error| error.to_string())
            }
            .await;
            tx.send(AppEvent::WalletPlansReady { mode, result });
        });
    }

    pub(crate) fn on_wallet_plans_ready(
        &mut self,
        mode: WalletPlanPurchaseMode,
        result: Result<WalletPlanCatalog, String>,
    ) {
        match result {
            Ok(catalog) => {
                let mut header = ColumnRenderable::new();
                match &mode {
                    WalletPlanPurchaseMode::New | WalletPlanPurchaseMode::Onboarding { .. } => {
                        header.push(Line::from("Corbanu Plans".bold()));
                        header.push(Line::from(
                            "One month, paid once in Solana USDC. No recurring wallet charge."
                                .dim(),
                        ));
                    }
                    WalletPlanPurchaseMode::Upgrade {
                        current_plan_id,
                        starts_at,
                    } => {
                        header.push(Line::from("Upgrade Corbanu Plan".bold()));
                        push_wallet_line(
                            &mut header,
                            &format!(
                                "Choose a tier above {}. It starts {starts_at} after the paid period you already own.",
                                title_case_plan(current_plan_id)
                            ),
                            /*dimmed*/ false,
                        );
                        push_wallet_line(
                            &mut header,
                            "The existing period and its remaining tokens are preserved.",
                            /*dimmed*/ true,
                        );
                    }
                }
                let payment = catalog.payment;
                let plans = match &mode {
                    WalletPlanPurchaseMode::New | WalletPlanPurchaseMode::Onboarding { .. } => {
                        catalog.plans
                    }
                    WalletPlanPurchaseMode::Upgrade {
                        current_plan_id, ..
                    } => higher_tiers_from_catalog(catalog.plans, current_plan_id),
                };
                let available_usdc_atomic = self.wallet_balances.map(|value| value.usdc_atomic);
                let mut items = plans
                    .into_iter()
                    .map(|plan| {
                        let mut selected = plan.clone();
                        selected.deferred_setup = match &mode {
                            WalletPlanPurchaseMode::Onboarding { deferred } => {
                                Some(deferred.clone())
                            }
                            WalletPlanPurchaseMode::New
                            | WalletPlanPurchaseMode::Upgrade { .. } => None,
                        };
                        if let WalletPlanPurchaseMode::Upgrade { starts_at, .. } = &mode {
                            selected.scheduled_start = Some(starts_at.clone());
                        }
                        let required = plan.amount_atomic.parse::<u64>().ok();
                        let affordability = match (available_usdc_atomic, required) {
                            (Some(available), Some(required)) if available >= required => {
                                format!("affordable with {} USDC", format_usdc_atomic(available))
                            }
                            (Some(available), Some(required)) => format!(
                                "needs {} more USDC",
                                format_usdc_atomic(required.saturating_sub(available))
                            ),
                            _ => "balance unknown; refresh /wallet".to_string(),
                        };
                        SelectionItem {
                            name: format!(
                                "{} — {} USDC",
                                title_case_plan(&plan.id),
                                plan.price_usdc
                            ),
                            description: Some(format!(
                                "{} tokens/week · {} tokens/month · {affordability}",
                                format_token_count(plan.weekly_token_limit),
                                format_token_count(plan.monthly_token_limit),
                            )),
                            is_disabled: matches!(
                                (available_usdc_atomic, required),
                                (Some(available), Some(required)) if available < required
                            ),
                            actions: vec![Box::new(move |tx| {
                                tx.send(AppEvent::ConfirmWalletPlanPurchase {
                                    plan: selected.clone(),
                                })
                            })],
                            ..Default::default()
                        }
                    })
                    .collect::<Vec<_>>();
                if items.is_empty() {
                    items.push(SelectionItem {
                        name: "No higher tier available".to_string(),
                        description: Some(
                            "The current or already-scheduled plan is the highest tier."
                                .to_string(),
                        ),
                        is_disabled: true,
                        ..Default::default()
                    });
                }
                let deferred = match &mode {
                    WalletPlanPurchaseMode::Onboarding { deferred } => Some(deferred.clone()),
                    WalletPlanPurchaseMode::New | WalletPlanPurchaseMode::Upgrade { .. } => None,
                };
                self.bottom_pane.replace_selection_view_if_present(
                    WALLET_PLANS_VIEW_ID,
                    plan_params_with_cancel(
                        SelectionViewParams {
                            view_id: Some(WALLET_PLANS_VIEW_ID),
                            header: Box::new(header),
                            items,
                            footer_hint: Some(standard_popup_hint_line()),
                            ..Default::default()
                        },
                        deferred,
                    ),
                );
                self.wallet_payment_config = Some(payment);
            }
            Err(error) => {
                self.add_error_message(format!("Could not load plans: {error}"));
                let retry_mode = mode.clone();
                let deferred = match &mode {
                    WalletPlanPurchaseMode::Onboarding { deferred } => Some(deferred.clone()),
                    WalletPlanPurchaseMode::New | WalletPlanPurchaseMode::Upgrade { .. } => None,
                };
                self.bottom_pane.replace_selection_view_if_present(
                    WALLET_PLANS_VIEW_ID,
                    plan_params_with_cancel(
                        SelectionViewParams {
                            view_id: Some(WALLET_PLANS_VIEW_ID),
                            items: vec![SelectionItem {
                                name: "Retry loading plans".to_string(),
                                actions: vec![Box::new(move |tx| {
                                    tx.send(AppEvent::OpenWalletPlans {
                                        mode: retry_mode.clone(),
                                    })
                                })],
                                ..Default::default()
                            }],
                            footer_hint: Some(standard_popup_hint_line()),
                            ..Default::default()
                        },
                        deferred,
                    ),
                );
            }
        }
    }

    pub(crate) fn confirm_wallet_plan_purchase(&mut self, plan: WalletPlanChoice) {
        let selected = plan.clone();
        let cancel_deferred = plan.deferred_setup.clone();
        let amount_atomic = plan.amount_atomic.parse::<u64>().ok();
        let remaining_usdc = self.wallet_balances.and_then(|balance| {
            amount_atomic
                .map(|amount| (balance.usdc_atomic, balance.usdc_atomic.checked_sub(amount)))
        });
        let mut header = ColumnRenderable::new();
        header.push(Line::from(
            format!("Confirm {} plan", title_case_plan(&plan.id)).bold(),
        ));
        header.push(Line::from(format!(
            "Pay exactly {} USDC on Solana",
            plan.price_usdc
        )));
        push_wallet_line(
            &mut header,
            &format!(
                "Allowance: {} tokens/week and {} tokens/month for one month.",
                format_token_count(plan.weekly_token_limit),
                format_token_count(plan.monthly_token_limit),
            ),
            /*dimmed*/ false,
        );
        push_wallet_line(
            &mut header,
            &plan.scheduled_start.as_ref().map_or_else(
                || "This payment is final and does not recur automatically.".to_string(),
                |starts_at| {
                    format!(
                        "This upgrade begins {starts_at}; the current paid period remains active until then."
                    )
                },
            ),
            /*dimmed*/ true,
        );
        match remaining_usdc {
            Some((current, Some(remaining))) => header.push(Line::from(format!(
                "Balance: {:.2} USDC now · {:.2} USDC after payment",
                current as f64 / 1_000_000.0,
                remaining as f64 / 1_000_000.0,
            ))),
            Some((current, None)) => header.push(Line::from(
                format!(
                    "Insufficient balance: {:.2} USDC available",
                    current as f64 / 1_000_000.0,
                )
                .red(),
            )),
            None => header.push(Line::from(
                "Balance is unavailable; refresh /wallet before paying.".red(),
            )),
        }
        if let Some(balance) = self.wallet_balances {
            header.push(Line::from(format!(
                "SOL: {:.6} · x402 facilitator sponsors the transaction fee",
                balance.sol_lamports as f64 / 1_000_000_000.0,
            )));
        }
        self.show_selection_view(plan_params_with_cancel(
            SelectionViewParams {
                view_id: Some(WALLET_PLAN_CONFIRM_VIEW_ID),
                header: Box::new(header),
                items: vec![
                    SelectionItem {
                        name: "Cancel".to_string(),
                        description: Some("Return without signing or sending USDC".to_string()),
                        dismiss_on_select: true,
                        actions: cancel_deferred.map_or_else(Vec::new, |deferred| {
                            vec![Box::new(move |tx: &AppEventSender| {
                                tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                                    deferred: deferred.clone(),
                                });
                            }) as SelectionAction]
                        }),
                        ..Default::default()
                    },
                    SelectionItem {
                        name: format!("Pay {} USDC", plan.price_usdc),
                        description: Some(
                            "Sign the exact x402 USDC transfer and activate the provider"
                                .to_string(),
                        ),
                        is_disabled: remaining_usdc
                            .is_none_or(|(_, remaining)| remaining.is_none()),
                        actions: vec![Box::new(move |tx| {
                            tx.send(AppEvent::WalletPlanPurchaseRequested {
                                plan: selected.clone(),
                            })
                        })],
                        dismiss_on_select: true,
                        ..Default::default()
                    },
                ],
                initial_selected_idx: Some(0),
                allow_number_shortcuts: false,
                footer_hint: Some(standard_popup_hint_line()),
                ..Default::default()
            },
            plan.deferred_setup,
        ));
    }

    pub(crate) fn purchase_wallet_plan(&mut self, plan: WalletPlanChoice) {
        let deferred_setup = plan.deferred_setup.clone();
        let Some(capability) = wallet_capability_for_request(self.wallet_capability.as_ref())
        else {
            self.add_error_message(
                "Unlock the wallet from /wallet before confirming a purchase.".to_string(),
            );
            return;
        };
        let Some(payment) = self.wallet_payment_config.clone() else {
            self.add_error_message(
                "Plan payment configuration expired; reopen /wallet and reload plans.".to_string(),
            );
            return;
        };
        self.add_info_message(
            format!(
                "Submitting the exact {} USDC payment for the {} plan…",
                plan.price_usdc, plan.id
            ),
            /*hint*/ None,
        );
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let purchase = WalletPlanPurchaseSummary {
                price_usdc: plan.price_usdc.clone(),
                scheduled_start: plan.scheduled_start.clone(),
                transaction: None,
            };
            let intent = PlanPurchaseIntent {
                gateway_origin: gateway_origin(),
                plan_id: plan.id,
                network: payment.network,
                rpc_url: payment.rpc_url,
                asset: payment.asset,
                amount_atomic: plan.amount_atomic,
                pay_to: payment.pay_to,
            };
            let result = WalletDaemonClient::new(home)
                .provision_plan(capability.to_string(), intent)
                .await
                .map(|provisioned| WalletPlanProvisionedResult {
                    plan_id: provisioned.plan_id,
                    api_key: WalletSecret::new(provisioned.api_key),
                    purchase: Some(WalletPlanPurchaseSummary {
                        transaction: provisioned.transaction,
                        ..purchase
                    }),
                })
                .map_err(|error| error.to_string());
            tx.send(AppEvent::WalletPlanProvisioned {
                operation: WalletPlanProvisioningOperation::Purchase,
                result,
                deferred_setup,
            });
        });
    }

    pub(crate) fn dismiss_deferred_wallet_plan_views(&mut self) {
        for view_id in [
            crate::chatwidget::wallet_receipt::WALLET_PLAN_RECEIPT_VIEW_ID,
            WALLET_PLAN_CONFIRM_VIEW_ID,
            WALLET_PLANS_VIEW_ID,
            WALLET_MENU_VIEW_ID,
        ] {
            while self.bottom_pane.dismiss_view_by_id(view_id) {}
        }
    }

    fn present_provisional_plan_receipt(&mut self, receipt: WalletPlanReceipt) {
        for view_id in [
            WALLET_PLAN_CONFIRM_VIEW_ID,
            WALLET_PLANS_VIEW_ID,
            WALLET_MENU_VIEW_ID,
        ] {
            while self.bottom_pane.dismiss_view_by_id(view_id) {}
        }
        self.open_wallet_plan_receipt(receipt);
    }

    fn begin_wallet_plan_persistence_attempt(&mut self) -> WalletPlanPersistenceAttemptId {
        self.next_wallet_plan_persistence_attempt =
            self.next_wallet_plan_persistence_attempt.wrapping_add(1);
        if self.next_wallet_plan_persistence_attempt == 0 {
            self.next_wallet_plan_persistence_attempt = 1;
        }
        let attempt_id =
            WalletPlanPersistenceAttemptId::new(self.next_wallet_plan_persistence_attempt);
        self.current_wallet_plan_persistence_attempt = Some(attempt_id);
        attempt_id
    }

    fn is_current_wallet_plan_persistence_attempt(
        &self,
        attempt_id: WalletPlanPersistenceAttemptId,
    ) -> bool {
        self.current_wallet_plan_persistence_attempt == Some(attempt_id)
    }

    pub(crate) fn on_wallet_plan_provisioned(
        &mut self,
        operation: WalletPlanProvisioningOperation,
        result: Result<WalletPlanProvisionedResult, String>,
        deferred_setup: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
    ) {
        let provisioned = match result {
            Ok(provisioned) => provisioned,
            Err(error) => {
                self.add_error_message(wallet_plan_provisioning_error(operation, &error));
                return;
            }
        };
        let attempt_id = self.begin_wallet_plan_persistence_attempt();
        if let Some(purchase) = provisioned.purchase.as_ref() {
            self.add_info_message(
                "Payment settled. Verifying the plan schedule and preparing its receipt…"
                    .to_string(),
                /*hint*/ None,
            );
            self.present_provisional_plan_receipt(provisional_plan_receipt(
                &provisioned.plan_id,
                purchase,
                /*credential_error*/ None,
            ));
        }

        let home = self.config.codex_home.as_path().to_path_buf();
        let store_mode = self.config.cli_auth_credentials_store_mode;
        let keyring_backend = self.config.auth_keyring_backend_kind();
        let tx = self.app_event_tx.clone();
        let plan_id = provisioned.plan_id;
        let purchase = provisioned.purchase;
        let api_key = provisioned.api_key;
        tokio::spawn(async move {
            let persisted = tokio::task::spawn_blocking(move || {
                let result = codex_login::login_with_provider_api_key(
                    &home,
                    PFTERMINAL_PLAN_API_KEY_ENV_VAR,
                    api_key.expose(),
                    store_mode,
                    keyring_backend,
                )
                .map_err(|_| WalletPlanCredentialPersistenceError::StoreFailed);
                (api_key, result)
            })
            .await;
            let (api_key, result) = match persisted {
                Ok((api_key, result)) => (Some(api_key), result),
                Err(_) => (
                    None,
                    Err(WalletPlanCredentialPersistenceError::WorkerUnavailable),
                ),
            };
            tx.send(AppEvent::WalletPlanCredentialPersistenceFinished {
                attempt_id,
                operation,
                plan_id,
                purchase,
                deferred_setup,
                api_key,
                result,
            });
        });
    }

    pub(crate) fn on_wallet_plan_credential_persistence_finished(
        &mut self,
        attempt_id: WalletPlanPersistenceAttemptId,
        operation: WalletPlanProvisioningOperation,
        plan_id: String,
        purchase: Option<WalletPlanPurchaseSummary>,
        deferred_setup: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
        api_key: Option<WalletSecret>,
        result: Result<(), WalletPlanCredentialPersistenceError>,
    ) {
        if !self.is_current_wallet_plan_persistence_attempt(attempt_id) {
            return;
        }
        let purchase = match (operation, purchase) {
            (WalletPlanProvisioningOperation::Purchase, Some(purchase)) => Some(purchase),
            (WalletPlanProvisioningOperation::Recovery, None) => None,
            _ => return,
        };
        let selection_policy = if deferred_setup.is_some() {
            WalletPlanReceiptSelectionPolicy::PreserveCurrentProvider
        } else {
            WalletPlanReceiptSelectionPolicy::SelectProviderOnSuccess
        };
        if result.is_ok()
            && let Some(deferred) = deferred_setup
        {
            self.app_event_tx
                .send(AppEvent::DeferredCorbanuPlanConfigured { deferred });
        }
        if let Some(purchase) = purchase {
            let credential_error = result.err().map(|error| error.to_string());
            let Some(api_key) = api_key else {
                self.present_provisional_plan_receipt(provisional_plan_receipt(
                    &plan_id,
                    &purchase,
                    credential_error,
                ));
                return;
            };
            let home = self.config.codex_home.as_path().to_path_buf();
            let tx = self.app_event_tx.clone();
            tokio::spawn(async move {
                let receipt = reconcile_plan_receipt(
                    home,
                    Zeroizing::new(api_key.into_inner()),
                    plan_id,
                    purchase,
                    credential_error,
                )
                .await;
                tracing::debug!(?attempt_id, "dispatching reconciled Corbanu Plan receipt");
                tx.send(AppEvent::WalletPlanReceiptReady {
                    attempt_id,
                    selection_policy,
                    receipt,
                });
            });
        } else {
            match result {
                Ok(()) => {
                    self.add_info_message(
                        "Corbanu Plan access recovered. Credential stored securely.".to_string(),
                        /*hint*/ None,
                    );
                    self.select_pfterminal_plan_provider();
                    self.open_wallet_menu();
                }
                Err(error) => self.add_error_message(format!(
                    "Plan access was recovered, but storing its API key failed: {error}"
                )),
            }
        }
    }

    pub(crate) fn on_wallet_plan_receipt_reconciled(
        &mut self,
        attempt_id: WalletPlanPersistenceAttemptId,
        selection_policy: WalletPlanReceiptSelectionPolicy,
        receipt: WalletPlanReceipt,
    ) {
        let current = self.is_current_wallet_plan_persistence_attempt(attempt_id);
        tracing::debug!(
            ?attempt_id,
            current,
            "received reconciled Corbanu Plan receipt"
        );
        if current {
            self.on_wallet_plan_receipt_ready(receipt, selection_policy);
        }
    }

    pub(crate) fn recover_wallet_plan_access(&mut self) {
        let Some(capability) = wallet_capability_for_request(self.wallet_capability.as_ref())
        else {
            self.open_wallet_plan_recovery_unlock();
            return;
        };
        self.request_wallet_plan_recovery(capability);
    }

    fn open_wallet_plan_recovery_unlock(&mut self) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        let view = crate::bottom_pane::vault_secret_entry::VaultSecretEntryView::new_fixed_secret(
            "wallet-passcode".to_string(),
            "Recover Corbanu Plan".to_string(),
            "Wallet passcode — masked".to_string(),
            "Verify wallet ownership and restore plan access. No USDC will be sent.".to_string(),
            Box::new(move |_label, mut passcode| {
                tokio::spawn(async move {
                    let daemon = WalletDaemonClient::new(home);
                    let result = match daemon
                        .unlock(std::mem::take(&mut passcode), UnlockPolicy::OneAction)
                        .await
                    {
                        Ok((capability, _expires_in_seconds)) => daemon
                            .issue_gateway_key(capability, wallet_gateway_origin())
                            .await
                            .map(|key| WalletPlanProvisionedResult {
                                plan_id: "existing".to_string(),
                                api_key: WalletSecret::new(key.api_key),
                                purchase: None,
                            })
                            .map_err(|error| error.to_string()),
                        Err(error) => Err(error.to_string()),
                    };
                    passcode.zeroize();
                    tx.send(AppEvent::WalletPlanProvisioned {
                        operation: WalletPlanProvisioningOperation::Recovery,
                        result,
                        deferred_setup: None,
                    });
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }

    fn request_wallet_plan_recovery(&self, capability: Zeroizing<String>) {
        let home = self.config.codex_home.as_path().to_path_buf();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = WalletDaemonClient::new(home)
                .issue_gateway_key(capability.to_string(), wallet_gateway_origin())
                .await
                .map(|key| WalletPlanProvisionedResult {
                    plan_id: "existing".to_string(),
                    api_key: WalletSecret::new(key.api_key),
                    purchase: None,
                })
                .map_err(|error| error.to_string());
            tx.send(AppEvent::WalletPlanProvisioned {
                operation: WalletPlanProvisioningOperation::Recovery,
                result,
                deferred_setup: None,
            });
        });
    }

    pub(super) fn select_pfterminal_plan_provider(&self) {
        self.app_event_tx.send(AppEvent::UpdateModelSelection {
            model: AMBIENT_DEFAULT_MODEL.to_string(),
            provider: Some(PFTERMINAL_PLAN_PROVIDER_ID.to_string()),
        });
        self.app_event_tx.send(AppEvent::PersistModelSelection {
            model: AMBIENT_DEFAULT_MODEL.to_string(),
            provider: Some(PFTERMINAL_PLAN_PROVIDER_ID.to_string()),
            effort: None,
        });
    }

    pub(crate) fn on_deferred_corbanu_plan_configured(
        &mut self,
        deferred: &crate::onboarding::provider_setup::DeferredProviderSetup,
    ) -> Result<(), DeferredPlanActivationError> {
        let host = crate::provider_status_host::ProviderStatusHost::from_config(
            &self.config,
            crate::provider_status_host::ProviderAccountMetadata {
                corbanu: codex_provider_auth::CorbanuPlanMetadata::Configured {
                    source: codex_provider_auth::CorbanuCredentialSource::Managed,
                    availability: codex_provider_auth::ConfiguredAvailability::Ready,
                },
                ..Default::default()
            },
        );
        if !host.activate(codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID) {
            self.add_error_message(
                "Corbanu Plan was stored, but activation could not be persisted.".to_string(),
            );
            self.show_deferred_plan_activation_retry(deferred.clone());
            return Err(DeferredPlanActivationError::Persistence);
        }
        let reconciled = host
            .resolve_provider(codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID)
            .is_some_and(|status| {
                status.configuration == codex_provider_auth::ProviderConfigurationState::Configured
                    && status.eligibility == codex_provider_auth::ProviderEligibilityState::Active
                    && status.availability == codex_provider_auth::ProviderAvailabilityState::Ready
            });
        if !reconciled {
            self.add_error_message(
                "Corbanu Plan was stored, but status reconciliation is still incomplete."
                    .to_string(),
            );
            self.show_deferred_plan_activation_retry(deferred.clone());
            return Err(DeferredPlanActivationError::StatusNotReady);
        }
        if !deferred.has_usable_fallback() {
            self.select_pfterminal_plan_provider();
        }
        Ok(())
    }

    fn show_deferred_plan_activation_retry(
        &mut self,
        deferred: crate::onboarding::provider_setup::DeferredProviderSetup,
    ) {
        let retry = deferred.clone();
        let cancel = deferred.clone();
        self.show_selection_view(SelectionViewParams {
            title: Some("Finish Corbanu Plan setup".to_string()),
            items: vec![
                SelectionItem {
                    name: "Retry activation".to_string(),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::DeferredCorbanuPlanConfigured {
                            deferred: retry.clone(),
                        })
                    })],
                    ..Default::default()
                },
                SelectionItem {
                    name: "Cancel".to_string(),
                    actions: vec![Box::new(move |tx| {
                        tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                            deferred: cancel.clone(),
                        })
                    })],
                    ..Default::default()
                },
            ],
            on_cancel: Some(Box::new(move |tx| {
                tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                    deferred: deferred.clone(),
                });
            })),
            ..Default::default()
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeferredPlanActivationError {
    Persistence,
    StatusNotReady,
}

fn shared_provider_method_label(
    capability: &codex_provider_auth::ProviderSetupCapability,
) -> &'static str {
    match capability {
        codex_provider_auth::ProviderSetupCapability::OpenAiAccount => "account",
        codex_provider_auth::ProviderSetupCapability::ApiKey { .. } => "API key",
        codex_provider_auth::ProviderSetupCapability::ClaudeAccount => "account",
        codex_provider_auth::ProviderSetupCapability::CorbanuPlan => "Plan",
        codex_provider_auth::ProviderSetupCapability::Local { .. } => "local runtime",
        codex_provider_auth::ProviderSetupCapability::CommandAuth { .. } => "external command",
        codex_provider_auth::ProviderSetupCapability::StatusOnly { .. } => "status only",
    }
}

fn shared_provider_status_description(
    status: Option<&codex_provider_auth::ProviderStatusSnapshot>,
) -> String {
    let Some(status) = status else {
        return "Status unavailable".to_string();
    };
    use codex_provider_auth::ProviderAvailabilityState as Availability;
    use codex_provider_auth::ProviderConfigurationState as Configuration;
    use codex_provider_auth::ProviderEligibilityState as Eligibility;
    match (
        status.configuration,
        status.eligibility,
        status.availability,
    ) {
        (Configuration::Configured, Eligibility::Active, Availability::Ready) => {
            "Configured · active · ready".to_string()
        }
        (Configuration::Configured, Eligibility::Inactive, _) => {
            "Configured · inactive".to_string()
        }
        (Configuration::Configured, Eligibility::Active, _) => {
            "Configured · active · unavailable".to_string()
        }
        (Configuration::RecoveryRequired, _, _) => "Recovery required".to_string(),
        (Configuration::NotConfigured, _, _) => "Not configured".to_string(),
        (Configuration::Checking, _, _) => "Checking".to_string(),
        (Configuration::Unavailable, _, _) => "Status unavailable".to_string(),
        (Configuration::Configured, _, _) => "Configured · status unavailable".to_string(),
    }
}

fn separate_plan_for_local_wallet(
    plan: Option<WalletPlanStatus>,
    local_wallet_address: Option<&str>,
) -> (Option<WalletPlanStatus>, Option<WalletPlanStatus>) {
    match plan {
        Some(plan)
            if local_wallet_address.is_some_and(|address| address == plan.wallet_address) =>
        {
            (Some(plan), None)
        }
        Some(plan) => (None, Some(plan)),
        None => (None, None),
    }
}

fn wallet_plan_provisioning_error(
    operation: WalletPlanProvisioningOperation,
    error: &str,
) -> String {
    match operation {
        WalletPlanProvisioningOperation::Purchase => format!(
            "Plan purchase failed: {error}. If payment may have settled, use Recover plan access; do not submit another payment."
        ),
        WalletPlanProvisioningOperation::Recovery => {
            format!("Plan access recovery failed: {error}. No USDC was sent.")
        }
    }
}

fn wallet_persistence_action_label(operation: WalletPersistenceOperation) -> &'static str {
    match operation {
        WalletPersistenceOperation::Create => "Wallet creation",
        WalletPersistenceOperation::Restore => "Wallet restoration",
    }
}

fn wallet_persistence_success_message(
    operation: WalletPersistenceOperation,
    address: &str,
) -> String {
    let verb = match operation {
        WalletPersistenceOperation::Create => "Created",
        WalletPersistenceOperation::Restore => "Restored",
    };
    format!(
        "{verb} Solana wallet {address}. The recovery material is shown only in the secure view."
    )
}

fn plan_params_with_cancel(
    mut params: SelectionViewParams,
    deferred: Option<crate::onboarding::provider_setup::DeferredProviderSetup>,
) -> SelectionViewParams {
    if let Some(deferred) = deferred {
        params.on_cancel = Some(Box::new(move |tx: &AppEventSender| {
            tx.send(AppEvent::DeferredCorbanuPlanCancelled {
                deferred: deferred.clone(),
            });
        }));
    }
    params
}

fn wallet_params(
    result: Option<Result<WalletOverview, String>>,
    client_can_sign: bool,
) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Wallet".bold()));
    let items = match result {
        None => {
            header.push(Line::from(
                "Loading wallet state and Solana balances…".dim(),
            ));
            vec![SelectionItem {
                name: "Loading…".to_string(),
                is_disabled: true,
                ..Default::default()
            }]
        }
        Some(Err(error)) => {
            header.push(Line::from(format!("Unavailable: {error}").red()));
            vec![SelectionItem {
                name: "Retry".to_string(),
                actions: vec![Box::new(|tx| tx.send(AppEvent::OpenWallet))],
                ..Default::default()
            }]
        }
        Some(Ok(overview)) => wallet_items(&mut header, overview, client_can_sign),
    };
    SelectionViewParams {
        view_id: Some(WALLET_MENU_VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        items,
        header: Box::new(header),
        ..Default::default()
    }
}

fn wallet_items(
    header: &mut ColumnRenderable,
    overview: WalletOverview,
    client_can_sign: bool,
) -> Vec<SelectionItem> {
    if !overview.daemon.wallet_exists {
        header.push(Line::from(
            "No local wallet. Secrets stay outside chat and model context.".dim(),
        ));
        return vec![
            item(
                "Create wallet",
                "Create a new Solana mainnet wallet",
                || AppEvent::OpenWalletCreate,
            ),
            item("Restore wallet", "Restore from recovery material", || {
                AppEvent::OpenWalletRestore
            }),
        ];
    }
    let address = overview
        .daemon
        .address
        .clone()
        .unwrap_or_else(|| "unavailable".to_string());
    let latest_receipt = overview
        .plan
        .as_ref()
        .map(|status| latest_plan_receipt(status, overview.balances, &overview.plan_prices_usdc));
    let can_sign = client_can_sign && !overview.daemon.locked && !overview.daemon.busy;
    let lock = if overview.daemon.busy {
        "signing operation in progress"
    } else if overview.daemon.locked {
        "locked"
    } else if can_sign {
        "ready to sign in this TUI"
    } else {
        "unlocked elsewhere; passcode required here"
    };
    header.push(Line::from(
        format!("{} · {lock}", short_address(&address)).cyan(),
    ));
    let network = match overview.daemon.network.as_deref() {
        Some("devnet") => "Solana devnet",
        _ => "Solana mainnet",
    };
    push_wallet_line(header, network, /*dimmed*/ true);
    if let Some(balance) = overview.balances {
        header.push(Line::from(format!(
            "{:.6} SOL · {:.2} USDC",
            balance.sol_lamports as f64 / 1_000_000_000.0,
            balance.usdc_atomic as f64 / 1_000_000.0
        )));
    }
    if let Some(error) = overview.balance_error {
        header.push(Line::from(format!("Balance unavailable: {error}").red()));
    }
    if let Some(linked_plan) = &overview.linked_plan_for_other_wallet {
        push_wallet_line(
            header,
            &format!(
                "Corbanu Plan · {} linked to another wallet",
                title_case_plan(&linked_plan.period.plan_id),
            ),
            /*dimmed*/ false,
        );
        push_wallet_line(
            header,
            &linked_plan_owner_description(linked_plan),
            /*dimmed*/ false,
        );
    }
    if !overview.plan_credential_present {
        push_wallet_line(
            header,
            "Corbanu Plan · not connected",
            /*dimmed*/ false,
        );
    }
    let upgrade_mode = overview.plan.as_ref().map(|plan| {
        let (current_plan_id, starts_at) = plan
            .queued_periods
            .last()
            .map(|queued| (queued.plan_id.clone(), queued.ends_at.clone()))
            .unwrap_or_else(|| (plan.period.plan_id.clone(), plan.period.ends_at.clone()));
        WalletPlanPurchaseMode::Upgrade {
            current_plan_id,
            starts_at,
        }
    });
    if let Some(plan) = &overview.plan {
        push_wallet_line(header, &wallet_plan_summary(plan), /*dimmed*/ false);
    }
    if let Some(error) = overview.plan_error {
        header.push(Line::from(format!("Plan status: {error}").red()));
    }
    let receive_address = address.clone();
    let mut items = vec![SelectionItem {
        name: "Receive".to_string(),
        description: Some(address.clone()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::InsertHistoryCell(Box::new(
                history_cell::new_info_event(
                    format!("Solana receive address: {receive_address}"),
                    /*hint*/ None,
                ),
            )))
        })],
        ..Default::default()
    }];
    if overview.plan_credential_present {
        items.push(item(
            "Plan details",
            "View prepaid spend, token usage, limits, reset dates, and queued periods",
            || AppEvent::OpenWalletPlanUsage,
        ));
    }
    if let Some(receipt) = latest_receipt {
        let receipt_for_action = receipt.clone();
        items.push(item(
            "View latest plan receipt",
            &format!(
                "{} plan · Solana payment confirmation",
                title_case_plan(&receipt.plan_id)
            ),
            move || AppEvent::OpenWalletPlanReceipt {
                receipt: receipt_for_action.clone(),
            },
        ));
    }
    if !overview.plan_credential_present && !overview.daemon.busy {
        if can_sign {
            items.push(item(
                "Buy Corbanu Plan",
                "Choose a plan and confirm the exact USDC payment",
                || AppEvent::OpenWalletPlans {
                    mode: WalletPlanPurchaseMode::New,
                },
            ));
        } else {
            items.push(item(
                "Buy Corbanu Plan",
                "Unlock, choose a plan, and confirm the exact USDC payment",
                || AppEvent::OpenWalletUnlock {
                    policy: UnlockPolicy::Timed {
                        duration_seconds: 300,
                    },
                    continuation: crate::app_event::WalletUnlockContinuation::OpenPlans {
                        mode: WalletPlanPurchaseMode::New,
                    },
                },
            ));
        }
        items.push(item(
            "Recover existing plan",
            "Only for a wallet that previously purchased a plan; sends no USDC",
            || AppEvent::WalletRecoverPlanRequested,
        ));
    }
    if !can_sign && !overview.daemon.busy {
        for (name, policy) in [
            ("Unlock for one signing action", UnlockPolicy::OneAction),
            (
                "Unlock for 5 minutes",
                UnlockPolicy::Timed {
                    duration_seconds: 300,
                },
            ),
            (
                "Unlock for 15 minutes",
                UnlockPolicy::Timed {
                    duration_seconds: 900,
                },
            ),
            (
                "Unlock for 1 hour",
                UnlockPolicy::Timed {
                    duration_seconds: 3_600,
                },
            ),
        ] {
            items.push(SelectionItem {
                name: name.to_string(),
                description: Some("Signing capability remains only in this TUI".to_string()),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::OpenWalletUnlock {
                        policy,
                        continuation: crate::app_event::WalletUnlockContinuation::WalletMenu,
                    })
                })],
                ..Default::default()
            });
        }
        items.push(item(
            "Unlock for a custom duration",
            "Choose 1 minute through 8 hours; access remains only in this TUI",
            || AppEvent::OpenWalletCustomUnlock {
                validation_error: None,
                continuation: crate::app_event::WalletUnlockContinuation::WalletMenu,
            },
        ));
    }
    if !overview.daemon.locked {
        items.push(item(
            "Lock wallet",
            "Revoke signing from every Corbanu Terminal process",
            || AppEvent::WalletLockRequested,
        ));
    }
    if can_sign {
        if let Some(mode) = upgrade_mode {
            let starts_at = match &mode {
                WalletPlanPurchaseMode::Upgrade { starts_at, .. } => starts_at.clone(),
                WalletPlanPurchaseMode::New | WalletPlanPurchaseMode::Onboarding { .. } => {
                    unreachable!()
                }
            };
            items.push(item(
                "Upgrade Corbanu Plan",
                &format!("Choose a higher tier for the period starting {starts_at}"),
                move || AppEvent::OpenWalletPlans { mode: mode.clone() },
            ));
        } else if overview.plan_credential_present {
            items.push(item(
                "Buy a Corbanu Plan",
                "Pay once with USDC and activate metered inference",
                || AppEvent::OpenWalletPlans {
                    mode: WalletPlanPurchaseMode::New,
                },
            ));
        }
        if overview.plan_credential_present {
            items.push(item(
                "Recover plan access",
                "Issue a replacement key for an already-paid wallet without another payment",
                || AppEvent::WalletRecoverPlanRequested,
            ));
        }
    } else if !overview.daemon.busy
        && let Some(mode) = upgrade_mode
    {
        items.push(item(
            "Upgrade Corbanu Plan",
            "Unlock for 5 minutes, then choose a higher tier",
            move || AppEvent::OpenWalletUnlock {
                policy: UnlockPolicy::Timed {
                    duration_seconds: 300,
                },
                continuation: crate::app_event::WalletUnlockContinuation::OpenPlans {
                    mode: mode.clone(),
                },
            },
        ));
    }
    if overview.plan_credential_present {
        items.push(item(
            "Disconnect Corbanu Plan",
            "Remove the plan credential; keep the wallet and paid period",
            || AppEvent::ConfirmWalletPlanDisconnect,
        ));
    }
    items.push(item(
        "Back up recovery material",
        "Requires the fresh wallet passcode; opens only in the secure view",
        || AppEvent::OpenWalletRecoveryBackup,
    ));
    let removal_address = address;
    items.push(item(
        "Remove wallet from this device",
        "Requires saved recovery material; does not move funds",
        move || AppEvent::ConfirmWalletRemoval {
            address: removal_address.clone(),
        },
    ));
    items.push(item("Refresh", "Refresh daemon state and balances", || {
        AppEvent::OpenWallet
    }));
    items
}

fn linked_plan_owner_description(plan: &WalletPlanStatus) -> String {
    format!(
        "Purchased by {} · not this local wallet",
        short_address(&plan.wallet_address)
    )
}

fn wallet_plan_summary(plan: &WalletPlanStatus) -> String {
    let mut summary = format!(
        "Corbanu Plan · {} active",
        title_case_plan(&plan.period.plan_id)
    );
    if let Some(next) = plan.queued_periods.first() {
        summary.push_str(&format!(" · {} next", title_case_plan(&next.plan_id)));
    }
    summary
}

pub(super) fn item<F>(name: &str, description: &str, event: F) -> SelectionItem
where
    F: Fn() -> AppEvent + Send + Sync + 'static,
{
    SelectionItem {
        name: name.to_string(),
        description: Some(description.to_string()),
        actions: vec![Box::new(move |tx| {
            tx.send(event());
        })],
        ..Default::default()
    }
}

fn higher_tiers_from_catalog(
    plans: Vec<WalletPlanChoice>,
    current_plan_id: &str,
) -> Vec<WalletPlanChoice> {
    let Some(current_index) = plans.iter().position(|plan| plan.id == current_plan_id) else {
        return Vec::new();
    };
    plans.into_iter().skip(current_index + 1).collect()
}
pub(crate) fn short_address(address: &str) -> String {
    if address.len() > 14 {
        format!("{}…{}", &address[..7], &address[address.len() - 6..])
    } else {
        address.to_string()
    }
}
pub(crate) fn wallet_gateway_origin() -> String {
    gateway_origin()
}
pub(crate) fn title_case_plan(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
fn push_wallet_line(header: &mut ColumnRenderable, text: &str, dimmed: bool) {
    let style = if dimmed {
        WalletTextStyle::Dimmed
    } else {
        WalletTextStyle::Normal
    };
    push_wallet_text(header, text, style);
}

fn provisional_plan_receipt(
    plan_id: &str,
    purchase: &WalletPlanPurchaseSummary,
    credential_error: Option<String>,
) -> WalletPlanReceipt {
    WalletPlanReceipt {
        plan_id: plan_id.to_string(),
        price_usdc: Some(purchase.price_usdc.clone()),
        transaction: purchase.transaction.clone(),
        starts_at: purchase.scheduled_start.clone(),
        ends_at: None,
        active_plan_id: None,
        active_ends_at: None,
        remaining_usdc_atomic: None,
        reconciliation_error: None,
        credential_error,
    }
}

fn wallet_capability_for_request(
    capability: Option<&Zeroizing<String>>,
) -> Option<Zeroizing<String>> {
    capability.map(|value| Zeroizing::new(value.to_string()))
}

fn format_token_count(value: u64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}

fn format_usdc_atomic(value: u64) -> String {
    let whole = value / 1_000_000;
    let fraction = value % 1_000_000;
    if fraction == 0 {
        return whole.to_string();
    }
    let fraction = format!("{fraction:06}").trim_end_matches('0').to_string();
    format!("{whole}.{fraction}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shared_provider_setup_can_select_a_noninteractive_runtime() {
        let (mut chat, mut rx, _op_rx) =
            crate::chatwidget::tests::helpers::make_chatwidget_manual(None).await;
        while rx.try_recv().is_ok() {}
        let host = crate::provider_status_host::ProviderStatusHost::from_config(
            &chat.config,
            crate::provider_status_host::ProviderAccountMetadata::default(),
        );

        chat.open_shared_provider_setup(&host, false, false);
        for character in "ollama".chars() {
            chat.handle_key_event(crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char(character),
                crossterm::event::KeyModifiers::NONE,
            ));
        }
        chat.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));

        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::SharedProviderSetupSelectExisting {
                runtime_provider_id,
                ..
            }) if runtime_provider_id.as_str()
                == codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID
        ));
    }

    #[tokio::test]
    async fn shared_account_auth_phases_replace_one_stack_slot() {
        let (mut chat, _rx, _op_rx) =
            crate::chatwidget::tests::helpers::make_chatwidget_manual(None).await;

        chat.open_shared_account_pending(
            crate::provider_account_auth_host::ProviderAccountCancelKind::OpenAi,
        );
        chat.open_shared_openai_challenge(
            codex_provider_auth::OpenAiAccountChallenge::device_code(
                "https://example.test/device",
                "TEST-CODE",
            ),
        );
        assert_eq!(
            chat.bottom_pane.active_view_id(),
            Some(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID)
        );
        assert!(
            chat.bottom_pane
                .dismiss_view_by_id(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID)
        );
        assert_eq!(chat.bottom_pane.active_view_id(), None);

        chat.open_shared_account_pending(
            crate::provider_account_auth_host::ProviderAccountCancelKind::Claude,
        );
        chat.open_shared_claude_method_choice();
        chat.open_shared_claude_managed_token_entry();
        assert_ne!(
            chat.bottom_pane.active_view_id(),
            Some(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID)
        );
        assert!(
            !chat
                .bottom_pane
                .dismiss_view_by_id(SHARED_PROVIDER_ACCOUNT_AUTH_VIEW_ID)
        );
    }

    #[test]
    fn balance_reads_use_the_gateway_rpc_only_for_the_wallet_network() {
        let catalog = WalletPlanCatalog {
            plans: Vec::new(),
            payment: WalletPaymentConfig {
                network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
                asset: codex_wallet::SOLANA_MAINNET_USDC_MINT.to_string(),
                pay_to: "receiver".to_string(),
                rpc_url: "https://rpc.example.test".to_string(),
            },
        };

        assert_eq!(
            wallet_balance_endpoint("mainnet", Some(&catalog)),
            (
                "https://rpc.example.test".to_string(),
                codex_wallet::Network::Mainnet,
            )
        );
        assert_eq!(
            wallet_balance_endpoint("devnet", Some(&catalog)),
            (
                "https://api.devnet.solana.com".to_string(),
                codex_wallet::Network::Devnet,
            )
        );
    }

    #[test]
    fn recovery_failure_never_uses_purchase_or_settlement_language() {
        let message = wallet_plan_provisioning_error(
            WalletPlanProvisioningOperation::Recovery,
            "wallet passcode was incorrect",
        );
        assert_eq!(
            message,
            "Plan access recovery failed: wallet passcode was incorrect. No USDC was sent."
        );
        assert!(!message.contains("purchase"));
        assert!(!message.contains("settled"));
    }

    #[test]
    fn purchase_failure_retains_ambiguous_settlement_guidance() {
        let message = wallet_plan_provisioning_error(
            WalletPlanProvisioningOperation::Purchase,
            "connection closed",
        );
        assert!(message.contains("Plan purchase failed"));
        assert!(message.contains("payment may have settled"));
    }

    #[test]
    fn provisional_receipt_freezes_authoritative_settlement_without_inventing_schedule() {
        let receipt = provisional_plan_receipt(
            "starter",
            &WalletPlanPurchaseSummary {
                price_usdc: "1.00".to_string(),
                scheduled_start: None,
                transaction: Some("settlement-signature".to_string()),
            },
            Some("credential storage unavailable".to_string()),
        );

        assert_eq!(receipt.plan_id, "starter");
        assert_eq!(receipt.price_usdc.as_deref(), Some("1.00"));
        assert_eq!(receipt.transaction.as_deref(), Some("settlement-signature"));
        assert_eq!(receipt.starts_at, None);
        assert_eq!(receipt.ends_at, None);
        assert_eq!(receipt.active_plan_id, None);
        assert_eq!(receipt.active_ends_at, None);
        assert_eq!(receipt.remaining_usdc_atomic, None);
        assert_eq!(receipt.reconciliation_error, None);
        assert_eq!(
            receipt.credential_error.as_deref(),
            Some("credential storage unavailable")
        );
    }

    #[tokio::test]
    async fn provisional_receipt_replaces_duplicate_purchase_view_stack() {
        let (mut chat, _rx, _op_rx) =
            crate::chatwidget::tests::helpers::make_chatwidget_manual(None).await;
        for view_id in [
            WALLET_MENU_VIEW_ID,
            WALLET_PLANS_VIEW_ID,
            WALLET_PLANS_VIEW_ID,
            WALLET_PLAN_CONFIRM_VIEW_ID,
        ] {
            chat.show_selection_view(SelectionViewParams {
                view_id: Some(view_id),
                ..Default::default()
            });
        }

        chat.present_provisional_plan_receipt(provisional_plan_receipt(
            "starter",
            &WalletPlanPurchaseSummary {
                price_usdc: "1.00".to_string(),
                scheduled_start: None,
                transaction: Some("settlement-signature".to_string()),
            },
            None,
        ));

        assert_eq!(
            chat.bottom_pane.active_view_id(),
            Some(crate::chatwidget::wallet_receipt::WALLET_PLAN_RECEIPT_VIEW_ID)
        );
        assert!(
            chat.bottom_pane
                .dismiss_view_by_id(crate::chatwidget::wallet_receipt::WALLET_PLAN_RECEIPT_VIEW_ID)
        );
        assert_eq!(chat.bottom_pane.active_view_id(), None);
    }

    #[tokio::test]
    async fn deferred_cancellation_removes_every_wallet_plan_view() {
        let (mut chat, _rx, _op_rx) =
            crate::chatwidget::tests::helpers::make_chatwidget_manual(None).await;
        for view_id in [
            WALLET_MENU_VIEW_ID,
            WALLET_PLANS_VIEW_ID,
            WALLET_PLANS_VIEW_ID,
            WALLET_PLAN_CONFIRM_VIEW_ID,
        ] {
            chat.show_selection_view(SelectionViewParams {
                view_id: Some(view_id),
                ..Default::default()
            });
        }

        chat.dismiss_deferred_wallet_plan_views();

        assert_eq!(chat.bottom_pane.active_view_id(), None);
    }

    #[tokio::test]
    async fn provisioned_purchase_presents_receipt_before_persistence_completion() {
        let (mut chat, mut rx, _op_rx) =
            crate::chatwidget::tests::helpers::make_chatwidget_manual(None).await;
        let credential_home = tempfile::tempdir().expect("temporary credential home");
        let credential_home_root = credential_home.path().to_path_buf();
        let credential_home_path = credential_home.path().join("not-a-directory");
        std::fs::write(&credential_home_path, b"fixture").expect("create blocked credential home");
        chat.config.codex_home =
            codex_utils_absolute_path::AbsolutePathBuf::try_from(credential_home_path)
                .expect("absolute credential home");
        chat.config.cli_auth_credentials_store_mode =
            codex_config::types::AuthCredentialsStoreMode::File;
        let secret_canary = "pf53-persistence-secret-canary";

        chat.on_wallet_plan_provisioned(
            WalletPlanProvisioningOperation::Purchase,
            Ok(WalletPlanProvisionedResult {
                plan_id: "starter".to_string(),
                api_key: WalletSecret::new(secret_canary.to_string()),
                purchase: Some(WalletPlanPurchaseSummary {
                    price_usdc: "1.00".to_string(),
                    scheduled_start: None,
                    transaction: Some("settlement-signature".to_string()),
                }),
            }),
            None,
        );

        let pending_attempt = chat
            .current_wallet_plan_persistence_attempt
            .expect("persistence attempt must be registered synchronously");
        assert_eq!(
            chat.bottom_pane.active_view_id(),
            Some(crate::chatwidget::wallet_receipt::WALLET_PLAN_RECEIPT_VIEW_ID)
        );

        let completion = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let event = rx.recv().await.expect("persistence completion event");
                if matches!(
                    event,
                    AppEvent::WalletPlanCredentialPersistenceFinished { .. }
                ) {
                    break event;
                }
            }
        })
        .await
        .expect("credential persistence completed");
        assert!(
            !format!("{completion:?}").contains(secret_canary),
            "completion Debug output must redact the API key"
        );
        let AppEvent::WalletPlanCredentialPersistenceFinished {
            attempt_id,
            api_key,
            result,
            ..
        } = completion
        else {
            unreachable!("filtered to persistence completion")
        };
        assert_eq!(attempt_id, pending_attempt);
        assert_eq!(
            result,
            Err(WalletPlanCredentialPersistenceError::StoreFailed)
        );
        drop(api_key);
        drop(chat);
        drop(credential_home);
        assert!(
            !credential_home_root.exists(),
            "temporary credential storage containing the canary must be removed"
        );
    }

    #[tokio::test]
    async fn stale_plan_persistence_and_receipt_completions_are_inert() {
        let (mut chat, mut rx, _op_rx) =
            crate::chatwidget::tests::helpers::make_chatwidget_manual(None).await;
        while rx.try_recv().is_ok() {}
        const MARKER_VIEW_ID: &str = "wallet-plan-persistence-test-marker";
        chat.show_selection_view(SelectionViewParams {
            view_id: Some(MARKER_VIEW_ID),
            ..Default::default()
        });
        let stale_attempt = chat.begin_wallet_plan_persistence_attempt();
        let current_attempt = chat.begin_wallet_plan_persistence_attempt();

        chat.on_wallet_plan_credential_persistence_finished(
            stale_attempt,
            WalletPlanProvisioningOperation::Recovery,
            "existing".to_string(),
            None,
            None,
            Some(WalletSecret::new("stale-recovery-key".to_string())),
            Ok(()),
        );
        chat.on_wallet_plan_receipt_reconciled(
            stale_attempt,
            WalletPlanReceiptSelectionPolicy::SelectProviderOnSuccess,
            provisional_plan_receipt(
                "stale-plan",
                &WalletPlanPurchaseSummary {
                    price_usdc: "99".to_string(),
                    scheduled_start: None,
                    transaction: Some("stale-transaction".to_string()),
                },
                None,
            ),
        );

        assert_eq!(chat.bottom_pane.active_view_id(), Some(MARKER_VIEW_ID));
        assert_eq!(
            chat.current_wallet_plan_persistence_attempt,
            Some(current_attempt)
        );
        assert!(
            rx.try_recv().is_err(),
            "stale persistence must not emit provider activation events"
        );

        chat.on_wallet_plan_credential_persistence_finished(
            current_attempt,
            WalletPlanProvisioningOperation::Recovery,
            "existing".to_string(),
            None,
            None,
            Some(WalletSecret::new("current-recovery-key".to_string())),
            Ok(()),
        );
        assert_eq!(chat.bottom_pane.active_view_id(), Some(WALLET_MENU_VIEW_ID));
        assert!(
            std::iter::from_fn(|| rx.try_recv().ok()).any(|event| matches!(
                event,
                AppEvent::UpdateModelSelection {
                    provider: Some(provider),
                    ..
                } if provider == PFTERMINAL_PLAN_PROVIDER_ID
            ))
        );

        chat.on_wallet_plan_receipt_reconciled(
            current_attempt,
            WalletPlanReceiptSelectionPolicy::SelectProviderOnSuccess,
            provisional_plan_receipt(
                "current-plan",
                &WalletPlanPurchaseSummary {
                    price_usdc: "1".to_string(),
                    scheduled_start: None,
                    transaction: Some("current-transaction".to_string()),
                },
                None,
            ),
        );
        assert_eq!(
            chat.bottom_pane.active_view_id(),
            Some(crate::chatwidget::wallet_receipt::WALLET_PLAN_RECEIPT_VIEW_ID)
        );
    }

    fn overview(locked: bool) -> WalletOverview {
        WalletOverview {
            daemon: DaemonStatus {
                wallet_exists: true,
                address: Some("EpUYgzi88BYbsGoyiNghPppd3J9ASbARq7UjBCCUnk2i".to_string()),
                network: Some("mainnet".to_string()),
                locked,
                busy: false,
                expires_in_seconds: (!locked).then_some(300),
            },
            balances: Some(WalletBalances {
                sol_lamports: 100_000_000,
                usdc_atomic: 5_000_000,
            }),
            balance_error: None,
            plan: None,
            linked_plan_for_other_wallet: None,
            plan_error: None,
            plan_credential_present: false,
            plan_prices_usdc: std::collections::BTreeMap::new(),
        }
    }

    fn names(locked: bool, client_can_sign: bool) -> Vec<String> {
        let mut header = ColumnRenderable::new();
        wallet_items(&mut header, overview(locked), client_can_sign)
            .into_iter()
            .map(|item| item.name)
            .collect()
    }

    fn starter_plan() -> WalletPlanStatus {
        WalletPlanStatus {
            wallet_address: "EpUYgzi88BYbsGoyiNghPppd3J9ASbARq7UjBCCUnk2i".to_string(),
            period: WalletPlanPeriod {
                transaction: "starter-transaction".to_string(),
                plan_id: "starter".to_string(),
                starts_at: "2026-07-19T00:35:20Z".to_string(),
                ends_at: "2026-08-19T00:35:20Z".to_string(),
                monthly_limit_tokens: 1_000_000,
                monthly_used_tokens: 20_996,
                monthly_reserved_tokens: 0,
            },
            weekly: WalletUsageWindow {
                ends_at: "2026-07-26T00:35:20Z".to_string(),
                limit_tokens: 250_000,
                used_tokens: 20_996,
                reserved_tokens: 0,
            },
            monthly_remaining_tokens: 979_004,
            weekly_remaining_tokens: 229_004,
            queued_periods: Vec::new(),
        }
    }

    #[test]
    fn wallet_summary_keeps_usage_and_dates_out_of_the_header() {
        let mut plan = starter_plan();
        plan.queued_periods.push(WalletQueuedPlanPeriod {
            transaction: "basic-transaction".to_string(),
            plan_id: "basic".to_string(),
            starts_at: "2026-08-19T00:35:20Z".to_string(),
            ends_at: "2026-09-19T00:35:20Z".to_string(),
        });

        let summary = wallet_plan_summary(&plan);
        assert_eq!(summary, "Corbanu Plan · Starter active · Basic next");
        assert!(!summary.contains("token"));
        assert!(!summary.contains("2026-"));
        assert!(!summary.contains("USDC"));
    }

    #[test]
    fn connected_plan_exposes_dedicated_details_view() {
        let mut active = overview(/*locked*/ true);
        active.plan = Some(starter_plan());
        active.plan_credential_present = true;
        let mut header = ColumnRenderable::new();
        let items = wallet_items(&mut header, active, /*client_can_sign*/ false);
        let details = items
            .iter()
            .position(|item| item.name == "Plan details")
            .expect("plan details action");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);
        (items[details].actions[0])(&sender);
        assert!(matches!(rx.try_recv(), Ok(AppEvent::OpenWalletPlanUsage)));
    }

    #[test]
    fn linked_plan_for_another_wallet_is_not_presented_as_local_plan() {
        let local_address = "EpUYgzi88BYbsGoyiNghPppd3J9ASbARq7UjBCCUnk2i";
        let mut linked = starter_plan();
        linked.wallet_address = "2YYwro8tH3LzkwqCyHqZvBZt9KBsQwgtu9E6b1dBhbB5".to_string();

        let (local, other) =
            separate_plan_for_local_wallet(Some(linked.clone()), Some(local_address));
        assert!(local.is_none());
        assert_eq!(
            other.as_ref().map(|plan| &plan.wallet_address),
            Some(&linked.wallet_address)
        );

        let mut overview = overview(/*locked*/ false);
        overview.linked_plan_for_other_wallet = other;
        overview.plan_credential_present = true;
        let mut header = ColumnRenderable::new();
        let names = wallet_items(&mut header, overview, /*client_can_sign*/ true)
            .into_iter()
            .map(|item| item.name)
            .collect::<Vec<_>>();
        assert!(names.iter().any(|name| name == "Buy a Corbanu Plan"));
        assert!(names.iter().any(|name| name == "Disconnect Corbanu Plan"));
        assert!(!names.iter().any(|name| name == "Upgrade Corbanu Plan"));
        assert!(!names.iter().any(|name| name == "View latest plan receipt"));
        assert_eq!(
            linked_plan_owner_description(&linked),
            "Purchased by 2YYwro8…dBhbB5 · not this local wallet"
        );
    }

    #[test]
    fn matching_plan_wallet_remains_the_local_upgrade_account() {
        let linked = starter_plan();
        let address = linked.wallet_address.clone();

        let (local, other) = separate_plan_for_local_wallet(Some(linked), Some(&address));

        assert_eq!(
            local.as_ref().map(|plan| plan.period.plan_id.as_str()),
            Some("starter")
        );
        assert!(other.is_none());
    }

    #[test]
    fn unlocked_daemon_without_this_tui_capability_requires_unlock_again() {
        let items = names(/*locked*/ false, /*client_can_sign*/ false);
        assert!(items.iter().any(|name| name == "Buy Corbanu Plan"));
        assert!(items.iter().any(|name| name == "Recover existing plan"));
        assert!(
            items
                .iter()
                .any(|name| name == "Unlock for one signing action")
        );
        assert!(items.iter().any(|name| name == "Unlock for 15 minutes"));
        assert!(
            items
                .iter()
                .any(|name| name == "Unlock for a custom duration")
        );
        assert!(!items.iter().any(|name| name == "Buy a Corbanu Plan"));
        insta::assert_snapshot!("wallet_locked_action_names", items.join("\n"));
    }

    #[test]
    fn scoped_capability_enables_spending_actions_only_in_owning_tui() {
        let items = names(/*locked*/ false, /*client_can_sign*/ true);
        assert!(items.iter().any(|name| name == "Buy Corbanu Plan"));
        assert!(items.iter().any(|name| name == "Recover existing plan"));
        assert!(!items.iter().any(|name| name.starts_with("Unlock for")));
    }

    #[test]
    fn fresh_locked_wallet_leads_with_purchase_and_keeps_recovery_secondary() {
        let mut header = ColumnRenderable::new();
        let items = wallet_items(
            &mut header,
            overview(/*locked*/ true),
            /*client_can_sign*/ false,
        );
        let purchase = items
            .iter()
            .position(|item| item.name == "Buy Corbanu Plan")
            .expect("fresh wallet purchase action");
        let recovery = items
            .iter()
            .position(|item| item.name == "Recover existing plan")
            .expect("existing-plan recovery action");
        assert_eq!(purchase, 1, "purchase should follow Receive");
        assert_eq!(recovery, 2, "recovery should follow purchase");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);
        (items[purchase].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenWalletUnlock {
                policy: UnlockPolicy::Timed {
                    duration_seconds: 300
                },
                continuation: crate::app_event::WalletUnlockContinuation::OpenPlans {
                    mode: WalletPlanPurchaseMode::New
                }
            })
        ));
        (items[recovery].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::WalletRecoverPlanRequested)
        ));
    }

    #[test]
    fn active_signing_operation_is_busy_without_offering_conflicting_actions() {
        let mut busy = overview(/*locked*/ false);
        busy.daemon.busy = true;
        let mut header = ColumnRenderable::new();
        let items = wallet_items(&mut header, busy, /*client_can_sign*/ true);
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"Lock wallet"));
        assert!(!names.iter().any(|name| name.starts_with("Unlock for")));
        assert!(!names.contains(&"Buy a Corbanu Plan"));
        assert!(!names.contains(&"Buy Corbanu Plan"));
        assert!(!names.contains(&"Recover plan access"));
        assert!(!names.contains(&"Recover existing plan"));
    }

    #[test]
    fn active_plan_exposes_upgrade_before_and_after_unlock() {
        let mut locked_overview = overview(/*locked*/ true);
        locked_overview.plan = Some(starter_plan());
        let mut header = ColumnRenderable::new();
        let locked_items =
            wallet_items(&mut header, locked_overview, /*client_can_sign*/ false);
        let locked_upgrade = locked_items
            .iter()
            .position(|item| item.name == "Upgrade Corbanu Plan")
            .expect("locked upgrade action");
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);
        (locked_items[locked_upgrade].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenWalletUnlock {
                policy: UnlockPolicy::Timed {
                    duration_seconds: 300
                },
                continuation: crate::app_event::WalletUnlockContinuation::OpenPlans {
                    mode: WalletPlanPurchaseMode::Upgrade { .. }
                }
            })
        ));

        let mut unlocked_overview = overview(/*locked*/ false);
        unlocked_overview.plan = Some(starter_plan());
        let mut header = ColumnRenderable::new();
        let unlocked_items = wallet_items(
            &mut header,
            unlocked_overview,
            /*client_can_sign*/ true,
        );
        let unlocked_upgrade = unlocked_items
            .iter()
            .position(|item| item.name == "Upgrade Corbanu Plan")
            .expect("unlocked upgrade action");
        (unlocked_items[unlocked_upgrade].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenWalletPlans {
                mode: WalletPlanPurchaseMode::Upgrade {
                    current_plan_id,
                    starts_at,
                }
            }) if current_plan_id == "starter" && starts_at == "2026-08-19T00:35:20Z"
        ));
    }

    #[test]
    fn queued_plan_becomes_the_upgrade_floor_and_schedule_boundary() {
        let mut status = starter_plan();
        status.queued_periods.push(WalletQueuedPlanPeriod {
            transaction: "basic-transaction".to_string(),
            plan_id: "basic".to_string(),
            starts_at: "2026-08-19T00:35:20Z".to_string(),
            ends_at: "2026-09-19T00:35:20Z".to_string(),
        });
        let mut active = overview(/*locked*/ false);
        active.plan = Some(status);
        let mut header = ColumnRenderable::new();
        let items = wallet_items(&mut header, active, /*client_can_sign*/ true);
        let upgrade = items
            .iter()
            .position(|item| item.name == "Upgrade Corbanu Plan")
            .expect("upgrade action");
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);
        (items[upgrade].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenWalletPlans {
                mode: WalletPlanPurchaseMode::Upgrade {
                    current_plan_id,
                    starts_at,
                }
            }) if current_plan_id == "basic" && starts_at == "2026-09-19T00:35:20Z"
        ));
        let plans = ["entry", "daily", "studio"]
            .into_iter()
            .map(|id| WalletPlanChoice {
                id: id.to_string(),
                price_usdc: "1".to_string(),
                amount_atomic: "1000000".to_string(),
                weekly_token_limit: 1,
                monthly_token_limit: 1,
                scheduled_start: None,
                deferred_setup: None,
            })
            .collect();
        assert_eq!(
            higher_tiers_from_catalog(plans, "daily")
                .into_iter()
                .map(|plan| plan.id)
                .collect::<Vec<_>>(),
            vec!["studio"]
        );
    }

    #[test]
    fn queued_payment_exposes_a_durable_authoritative_receipt() {
        let mut status = starter_plan();
        status.queued_periods.push(WalletQueuedPlanPeriod {
            transaction: "basic-settlement-signature".to_string(),
            plan_id: "basic".to_string(),
            starts_at: "2026-08-19T00:35:20Z".to_string(),
            ends_at: "2026-09-19T00:35:20Z".to_string(),
        });
        let mut active = overview(/*locked*/ true);
        active.plan = Some(status);
        active
            .plan_prices_usdc
            .insert("basic".to_string(), "20".to_string());
        let mut header = ColumnRenderable::new();
        let items = wallet_items(&mut header, active, /*client_can_sign*/ false);
        let receipt = items
            .iter()
            .position(|item| item.name == "View latest plan receipt")
            .expect("receipt action");
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);

        (items[receipt].actions[0])(&sender);

        let AppEvent::OpenWalletPlanReceipt { receipt } = rx.try_recv().expect("receipt event")
        else {
            panic!("unexpected event");
        };
        assert_eq!(receipt.plan_id, "basic");
        assert_eq!(receipt.price_usdc.as_deref(), Some("20"));
        assert_eq!(
            receipt.transaction.as_deref(),
            Some("basic-settlement-signature")
        );
        assert_eq!(receipt.active_plan_id.as_deref(), Some("starter"));
    }

    #[test]
    fn plan_disconnect_and_wallet_removal_are_separate_actions() {
        let mut overview = overview(/*locked*/ true);
        overview.plan_credential_present = true;
        let mut header = ColumnRenderable::new();
        let items = wallet_items(&mut header, overview, /*client_can_sign*/ false);
        let names = items
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"Disconnect Corbanu Plan"));
        assert!(names.contains(&"Remove wallet from this device"));

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);
        let disconnect = items
            .iter()
            .position(|item| item.name == "Disconnect Corbanu Plan")
            .expect("disconnect action");
        (items[disconnect].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::ConfirmWalletPlanDisconnect)
        ));
        (items[disconnect].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::ConfirmWalletPlanDisconnect)
        ));
        let remove = items
            .iter()
            .position(|item| item.name == "Remove wallet from this device")
            .expect("remove action");
        (items[remove].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::ConfirmWalletRemoval { address })
                if address == "EpUYgzi88BYbsGoyiNghPppd3J9ASbARq7UjBCCUnk2i"
        ));
        (items[remove].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::ConfirmWalletRemoval { address })
                if address == "EpUYgzi88BYbsGoyiNghPppd3J9ASbARq7UjBCCUnk2i"
        ));
    }

    #[test]
    fn recovery_backup_is_available_while_locked_and_requires_fresh_passcode_flow() {
        let overview = overview(/*locked*/ true);
        let mut header = ColumnRenderable::new();
        let items = wallet_items(&mut header, overview, /*client_can_sign*/ false);
        let backup = items
            .iter()
            .position(|item| item.name == "Back up recovery material")
            .expect("backup action");
        assert!(
            items[backup]
                .description
                .as_deref()
                .is_some_and(|text| text.contains("fresh wallet passcode"))
        );
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = crate::app_event_sender::AppEventSender::new(tx);
        (items[backup].actions[0])(&sender);
        assert!(matches!(
            rx.try_recv(),
            Ok(AppEvent::OpenWalletRecoveryBackup)
        ));
    }

    #[test]
    fn signing_request_does_not_consume_the_tui_unlock_capability() {
        let held = Some(Zeroizing::new("test-wallet-capability".to_string()));
        let request = wallet_capability_for_request(held.as_ref()).expect("request capability");
        assert_eq!(request.as_str(), "test-wallet-capability");
        assert_eq!(
            held.as_deref().map(String::as_str),
            Some("test-wallet-capability")
        );
    }

    #[test]
    fn wallet_counts_format_for_status_and_allowance_copy() {
        assert_eq!(format_token_count(/*value*/ 1_000_000), "1,000,000");
        assert_eq!(format_usdc_atomic(/*value*/ 4_250_000), "4.25");
    }

    #[test]
    fn wallet_removal_confirmation_copy_snapshot() {
        let rendered = [
            "Remove wallet from this device",
            "Wallet: 3speRmS…JRwV5r",
            "Funds stay on Solana. Corbanu Terminal cannot recover them without your recovery material.",
            "This also disconnects the local Corbanu Plan credential. It does not cancel or refund the paid period.",
            "Cancel — Keep the wallet on this device",
            "Remove wallet — I have saved the recovery material",
        ]
        .join("\n");
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn wallet_upgrade_flow_copy_snapshot() {
        let upgrade_intro = "Choose a tier above Starter. It starts 2026-08-19T00:35:20Z after the paid period you already own.";
        let rendered = [
            "Locked wallet: Upgrade Corbanu Plan — Unlock for 5 minutes, then choose a higher tier",
            "Unlocked wallet: Upgrade Corbanu Plan — Choose a higher tier for the period starting 2026-08-19T00:35:20Z",
            "Upgrade Corbanu Plan",
            upgrade_intro,
            "The existing period and its remaining tokens are preserved.",
            "Confirmation: This upgrade begins 2026-08-19T00:35:20Z; the current paid period remains active until then.",
        ]
        .join("\n");
        insta::assert_snapshot!(rendered);
    }

    #[tokio::test]
    async fn successful_create_invalidates_the_pre_create_wallet_snapshot() {
        let (mut chat, _sender, _events, _ops) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        assert_eq!(chat.wallet_status_generation, 0);

        chat.on_wallet_create_finished(
            WalletPersistenceOperation::Create,
            Ok(WalletCreatedResult {
                address: "new-wallet-address".to_string(),
                recovery: WalletSecret::new("one-time-recovery".to_string()),
            }),
        );

        assert_eq!(chat.wallet_status_generation, 1);
    }

    #[test]
    fn wallet_persistence_messages_preserve_create_and_restore_identity() {
        assert_eq!(
            wallet_persistence_success_message(WalletPersistenceOperation::Create, "address"),
            "Created Solana wallet address. The recovery material is shown only in the secure view."
        );
        assert_eq!(
            wallet_persistence_success_message(WalletPersistenceOperation::Restore, "address"),
            "Restored Solana wallet address. The recovery material is shown only in the secure view."
        );
        assert_eq!(
            wallet_persistence_action_label(WalletPersistenceOperation::Restore),
            "Wallet restoration"
        );
    }

    #[tokio::test]
    async fn stale_wallet_status_cannot_overwrite_a_newer_refresh() {
        let (mut chat, _sender, _events, _ops) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        chat.wallet_status_generation = 2;
        chat.wallet_balances = Some(WalletBalances {
            sol_lamports: 7,
            usdc_atomic: 11,
        });

        chat.on_wallet_status_ready(/*generation*/ 1, Ok(overview(/*locked*/ true)));

        assert_eq!(
            chat.wallet_balances,
            Some(WalletBalances {
                sol_lamports: 7,
                usdc_atomic: 11,
            })
        );
    }
}
