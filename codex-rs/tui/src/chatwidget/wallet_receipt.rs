use super::*;
use crate::app_event::WalletPlanPurchaseSummary;
use crate::app_event::WalletPlanReceiptSelectionPolicy;
use crate::chatwidget::wallet_http::gateway_client;
use crate::chatwidget::wallet_menu::WalletPlanStatus;
use crate::chatwidget::wallet_menu::title_case_plan;
use codex_wallet::BalanceClient;
use codex_wallet::WalletBalances;
use codex_wallet_daemon::WalletDaemonClient;
use zeroize::Zeroizing;

pub(super) const WALLET_PLAN_RECEIPT_VIEW_ID: &str = "wallet-plan-receipt";
const RECEIPT_BALANCE_ENRICHMENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Debug, Clone)]
pub(crate) struct WalletPlanReceipt {
    pub(crate) plan_id: String,
    pub(crate) price_usdc: Option<String>,
    pub(crate) transaction: Option<String>,
    pub(crate) starts_at: Option<String>,
    pub(crate) ends_at: Option<String>,
    pub(crate) active_plan_id: Option<String>,
    pub(crate) active_ends_at: Option<String>,
    pub(crate) remaining_usdc_atomic: Option<u64>,
    pub(crate) reconciliation_error: Option<String>,
    pub(crate) credential_error: Option<String>,
}

struct ReceiptSeed {
    plan_id: String,
    price_usdc: Option<String>,
    transaction: Option<String>,
    scheduled_start: Option<String>,
    reconciliation_error: Option<String>,
    credential_error: Option<String>,
}

pub(super) async fn reconcile_plan_receipt(
    home: std::path::PathBuf,
    api_key: Zeroizing<String>,
    plan_id: String,
    purchase: WalletPlanPurchaseSummary,
    credential_error: Option<String>,
) -> WalletPlanReceipt {
    let account = match gateway_client() {
        Ok(gateway) => {
            gateway
                .client
                .get(format!("{}/v1/account", gateway.origin))
                .bearer_auth(api_key.as_str())
                .send()
                .await
        }
        Err(error) => {
            return receipt_from_status(
                /*status*/ None,
                /*balances*/ None,
                ReceiptSeed {
                    plan_id,
                    price_usdc: Some(purchase.price_usdc),
                    transaction: purchase.transaction,
                    scheduled_start: purchase.scheduled_start,
                    reconciliation_error: Some(format!(
                        "account confirmation was unavailable: {error}"
                    )),
                    credential_error,
                },
            );
        }
    };
    tracing::debug!("received authoritative Corbanu Plan account response");
    let (status, reconciliation_error) = match account {
        Ok(response) if response.status().is_success() => {
            match response.json::<WalletPlanStatus>().await {
                Ok(status) => (Some(status), None),
                Err(error) => (
                    None,
                    Some(format!("account confirmation was malformed: {error}")),
                ),
            }
        }
        Ok(response) => (
            None,
            Some(format!(
                "account confirmation returned HTTP {}",
                response.status()
            )),
        ),
        Err(error) => (
            None,
            Some(format!("account confirmation was unavailable: {error}")),
        ),
    };
    let balances =
        optional_enrichment_with_timeout(RECEIPT_BALANCE_ENRICHMENT_TIMEOUT, async move {
            match WalletDaemonClient::new(home).status().await {
                Ok(daemon) => {
                    let (rpc, network) = match daemon.network.as_deref() {
                        Some("devnet") => (
                            "https://api.devnet.solana.com",
                            codex_wallet::Network::Devnet,
                        ),
                        _ => (
                            "https://api.mainnet-beta.solana.com",
                            codex_wallet::Network::Mainnet,
                        ),
                    };
                    match daemon.address.as_deref() {
                        Some(address) => match BalanceClient::new(rpc, network) {
                            Ok(client) => client.balances(address).await.ok(),
                            Err(_) => None,
                        },
                        None => None,
                    }
                }
                Err(_) => None,
            }
        })
        .await;
    tracing::debug!(
        balance_enrichment_present = balances.is_some(),
        "finished optional Corbanu Plan receipt enrichment"
    );
    receipt_from_status(
        status.as_ref(),
        balances,
        ReceiptSeed {
            plan_id,
            price_usdc: Some(purchase.price_usdc),
            transaction: purchase.transaction,
            scheduled_start: purchase.scheduled_start,
            reconciliation_error,
            credential_error,
        },
    )
}

async fn optional_enrichment_with_timeout<T>(
    duration: std::time::Duration,
    enrichment: impl std::future::Future<Output = Option<T>>,
) -> Option<T> {
    tokio::time::timeout(duration, enrichment)
        .await
        .unwrap_or_default()
}

fn receipt_from_status(
    status: Option<&WalletPlanStatus>,
    balances: Option<WalletBalances>,
    seed: ReceiptSeed,
) -> WalletPlanReceipt {
    let queued = status.and_then(|account| {
        account.queued_periods.iter().rev().find(|period| {
            seed.transaction
                .as_deref()
                .is_some_and(|value| value == period.transaction)
                || period.plan_id == seed.plan_id
        })
    });
    let active = status.and_then(|account| {
        (account.period.plan_id == seed.plan_id
            || seed
                .transaction
                .as_deref()
                .is_some_and(|value| value == account.period.transaction))
        .then_some(&account.period)
    });
    WalletPlanReceipt {
        plan_id: seed.plan_id,
        price_usdc: seed.price_usdc,
        transaction: queued
            .map(|period| period.transaction.clone())
            .or_else(|| active.map(|period| period.transaction.clone()))
            .or(seed.transaction),
        starts_at: queued
            .map(|period| period.starts_at.clone())
            .or_else(|| active.map(|period| period.starts_at.clone()))
            .or(seed.scheduled_start),
        ends_at: queued
            .map(|period| period.ends_at.clone())
            .or_else(|| active.map(|period| period.ends_at.clone())),
        active_plan_id: status.map(|account| account.period.plan_id.clone()),
        active_ends_at: status.map(|account| account.period.ends_at.clone()),
        remaining_usdc_atomic: balances.map(|value| value.usdc_atomic),
        reconciliation_error: seed.reconciliation_error,
        credential_error: seed.credential_error,
    }
}

pub(super) fn latest_plan_receipt(
    status: &WalletPlanStatus,
    balances: Option<WalletBalances>,
    prices: &std::collections::BTreeMap<String, String>,
) -> WalletPlanReceipt {
    let (transaction, plan_id, starts_at, ends_at) = status
        .queued_periods
        .last()
        .map(|period| {
            (
                &period.transaction,
                &period.plan_id,
                &period.starts_at,
                &period.ends_at,
            )
        })
        .unwrap_or((
            &status.period.transaction,
            &status.period.plan_id,
            &status.period.starts_at,
            &status.period.ends_at,
        ));
    WalletPlanReceipt {
        plan_id: plan_id.clone(),
        price_usdc: prices.get(plan_id).cloned(),
        transaction: Some(transaction.clone()),
        starts_at: Some(starts_at.clone()),
        ends_at: Some(ends_at.clone()),
        active_plan_id: Some(status.period.plan_id.clone()),
        active_ends_at: Some(status.period.ends_at.clone()),
        remaining_usdc_atomic: balances.map(|value| value.usdc_atomic),
        reconciliation_error: None,
        credential_error: None,
    }
}

impl ChatWidget {
    pub(crate) fn on_wallet_plan_receipt_ready(
        &mut self,
        receipt: WalletPlanReceipt,
        selection_policy: WalletPlanReceiptSelectionPolicy,
    ) {
        for view in [
            WALLET_PLAN_RECEIPT_VIEW_ID,
            "wallet-plan-confirm",
            "wallet-plans",
            "wallet-menu",
        ] {
            while self.bottom_pane.dismiss_view_by_id(view) {}
        }
        if receipt.credential_error.is_none()
            && selection_policy == WalletPlanReceiptSelectionPolicy::SelectProviderOnSuccess
        {
            self.select_pfterminal_plan_provider();
        }
        self.add_info_message(receipt_history_message(&receipt), /*hint*/ None);
        self.open_wallet_plan_receipt(receipt);
    }

    pub(crate) fn open_wallet_plan_receipt(&mut self, receipt: WalletPlanReceipt) {
        let mut header = ColumnRenderable::new();
        header.push(Line::from("Payment confirmed".green().bold()));
        let plan = title_case_plan(&receipt.plan_id);
        header.push(Line::from(receipt.price_usdc.as_ref().map_or_else(
            || format!("{plan} plan payment"),
            |price| format!("{price} USDC paid · {plan} plan"),
        )));
        if let Some(transaction) = &receipt.transaction {
            header.push(Line::from("Solana transaction".dim()));
            for line in textwrap::wrap(transaction, 64) {
                header.push(Line::from(line.into_owned()));
            }
        } else {
            header.push(Line::from(
                "Settlement confirmed; transaction signature was not returned.".dim(),
            ));
        }
        let queued = is_queued(&receipt);
        if queued {
            for line in textwrap::wrap(&schedule_text(&receipt), 64) {
                header.push(Line::from(line.into_owned().green()));
            }
            if let (Some(active), Some(end)) = (&receipt.active_plan_id, &receipt.active_ends_at) {
                let text = format!("{} remains active through {end}", title_case_plan(active));
                for line in textwrap::wrap(&text, 64) {
                    header.push(Line::from(line.into_owned().dim()));
                }
            }
        } else if let (Some(start), Some(end)) = (&receipt.starts_at, &receipt.ends_at) {
            header.push(Line::from(format!("Active {start} through {end}").green()));
        }
        if let Some(remaining) = receipt.remaining_usdc_atomic {
            header.push(Line::from(format!(
                "Wallet balance after payment: {:.2} USDC",
                remaining as f64 / 1_000_000.0
            )));
        }
        if let Some(error) = &receipt.reconciliation_error {
            let text = format!(
                "Payment settled, but the plan schedule could not be refreshed: {error}. Use Recover plan access; do not pay again."
            );
            for line in textwrap::wrap(&text, 64) {
                header.push(Line::from(line.into_owned().red()));
            }
        }
        if let Some(error) = &receipt.credential_error {
            let text = format!(
                "Payment settled, but storing the plan credential failed: {error}. Use Recover plan access; do not pay again."
            );
            for line in textwrap::wrap(&text, 64) {
                header.push(Line::from(line.into_owned().red()));
            }
        }
        self.show_selection_view(SelectionViewParams {
            view_id: Some(WALLET_PLAN_RECEIPT_VIEW_ID),
            header: Box::new(header),
            items: vec![wallet_menu::item(
                "Done",
                "Return to the authoritative wallet and plan status",
                || AppEvent::CloseWalletPlanReceipt,
            )],
            footer_hint: Some(standard_popup_hint_line()),
            ..Default::default()
        });
    }

    pub(crate) fn close_wallet_plan_receipt(&mut self) {
        self.bottom_pane
            .dismiss_view_by_id(WALLET_PLAN_RECEIPT_VIEW_ID);
        self.open_wallet_menu();
    }
}

fn is_queued(receipt: &WalletPlanReceipt) -> bool {
    receipt
        .active_plan_id
        .as_deref()
        .is_some_and(|active| active != receipt.plan_id)
}

fn schedule_text(receipt: &WalletPlanReceipt) -> String {
    match (&receipt.starts_at, &receipt.ends_at) {
        (Some(start), Some(end)) => format!("Scheduled {start} through {end}"),
        (Some(start), None) => format!("Scheduled to start {start}"),
        _ => "Scheduled after the current paid period".to_string(),
    }
}

fn receipt_history_message(receipt: &WalletPlanReceipt) -> String {
    let amount = receipt
        .price_usdc
        .as_deref()
        .map(|price| format!("{price} USDC "))
        .unwrap_or_default();
    let state = if is_queued(receipt) {
        match (&receipt.starts_at, &receipt.ends_at) {
            (Some(start), Some(end)) => format!("scheduled {start} through {end}"),
            (Some(start), None) => format!("scheduled to start {start}"),
            _ => "scheduled after the current paid period".to_string(),
        }
    } else {
        "active".to_string()
    };
    let transaction = receipt
        .transaction
        .as_deref()
        .map(|value| format!(" Solana transaction: {value}."))
        .unwrap_or_default();
    format!(
        "Payment confirmed: {amount}for the {} plan; {state}.{transaction}",
        title_case_plan(&receipt.plan_id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_payment_confirmation_copy_snapshot() {
        let receipt = WalletPlanReceipt {
            plan_id: "basic".to_string(),
            price_usdc: Some("20".to_string()),
            transaction: Some("basic-settlement-signature".to_string()),
            starts_at: Some("2026-08-19T00:35:20Z".to_string()),
            ends_at: Some("2026-09-19T00:35:20Z".to_string()),
            active_plan_id: Some("starter".to_string()),
            active_ends_at: Some("2026-08-19T00:35:20Z".to_string()),
            remaining_usdc_atomic: Some(4_000_000),
            reconciliation_error: None,
            credential_error: None,
        };
        let history = receipt_history_message(&receipt);
        insta::assert_snapshot!(
            [
                "Payment confirmed",
                "20 USDC paid · Basic plan",
                "Solana transaction",
                "basic-settlement-signature",
                "Scheduled 2026-08-19T00:35:20Z through 2026-09-19T00:35:20Z",
                "Starter remains active through 2026-08-19T00:35:20Z",
                "Wallet balance after payment: 4.00 USDC",
                "Done — Return to the authoritative wallet and plan status",
                &history,
            ]
            .join("\n")
        );
    }

    #[tokio::test]
    async fn optional_balance_enrichment_cannot_delay_authoritative_receipt() {
        let enrichment = std::future::pending::<Option<WalletBalances>>();

        let balances =
            optional_enrichment_with_timeout(std::time::Duration::from_millis(1), enrichment).await;

        assert_eq!(balances, None);
    }

    #[tokio::test]
    async fn deferred_fallback_receipt_preserves_current_provider_but_wallet_purchase_selects() {
        let (mut chat, _sender, mut events, _ops) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        while events.try_recv().is_ok() {}
        let receipt = WalletPlanReceipt {
            plan_id: "starter".to_string(),
            price_usdc: Some("1".to_string()),
            transaction: Some("settlement".to_string()),
            starts_at: None,
            ends_at: None,
            active_plan_id: Some("starter".to_string()),
            active_ends_at: None,
            remaining_usdc_atomic: None,
            reconciliation_error: None,
            credential_error: None,
        };

        chat.on_wallet_plan_receipt_ready(
            receipt.clone(),
            WalletPlanReceiptSelectionPolicy::PreserveCurrentProvider,
        );

        assert_eq!(
            chat.bottom_pane.active_view_id(),
            Some(WALLET_PLAN_RECEIPT_VIEW_ID)
        );
        assert!(
            !std::iter::from_fn(|| events.try_recv().ok()).any(|event| matches!(
                event,
                AppEvent::UpdateModelSelection { .. } | AppEvent::PersistModelSelection { .. }
            )),
            "deferred fallback reconciliation must not select the Plan provider"
        );

        chat.on_wallet_plan_receipt_ready(
            receipt,
            WalletPlanReceiptSelectionPolicy::SelectProviderOnSuccess,
        );
        assert!(
            std::iter::from_fn(|| events.try_recv().ok())
                .any(|event| matches!(event, AppEvent::UpdateModelSelection { .. }))
        );
    }

    #[tokio::test]
    async fn reconciled_receipt_replaces_provisional_receipt_without_stale_stack() {
        let (mut chat, _sender, _events, _ops) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        let receipt = WalletPlanReceipt {
            plan_id: "starter".to_string(),
            price_usdc: Some("1".to_string()),
            transaction: Some("settlement".to_string()),
            starts_at: None,
            ends_at: None,
            active_plan_id: Some("starter".to_string()),
            active_ends_at: None,
            remaining_usdc_atomic: None,
            reconciliation_error: None,
            credential_error: None,
        };
        chat.open_wallet_plan_receipt(receipt.clone());

        chat.on_wallet_plan_receipt_ready(
            receipt,
            WalletPlanReceiptSelectionPolicy::PreserveCurrentProvider,
        );
        chat.close_wallet_plan_receipt();

        assert_eq!(chat.bottom_pane.active_view_id(), Some("wallet-menu"));
        assert!(
            !chat
                .bottom_pane
                .dismiss_view_by_id(WALLET_PLAN_RECEIPT_VIEW_ID),
            "the provisional receipt must not remain below the reconciled receipt"
        );
    }

    #[tokio::test]
    async fn receipt_done_dismisses_the_receipt_before_opening_wallet() {
        let (mut chat, _sender, _events, _ops) =
            crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
        chat.open_wallet_plan_receipt(WalletPlanReceipt {
            plan_id: "basic".to_string(),
            price_usdc: Some("20".to_string()),
            transaction: Some("settlement".to_string()),
            starts_at: None,
            ends_at: None,
            active_plan_id: Some("starter".to_string()),
            active_ends_at: None,
            remaining_usdc_atomic: None,
            reconciliation_error: None,
            credential_error: None,
        });
        assert!(
            chat.bottom_pane
                .selected_index_for_active_view(WALLET_PLAN_RECEIPT_VIEW_ID)
                .is_some()
        );

        chat.close_wallet_plan_receipt();

        assert!(
            chat.bottom_pane
                .selected_index_for_active_view(WALLET_PLAN_RECEIPT_VIEW_ID)
                .is_none()
        );
    }
}
