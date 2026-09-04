use super::*;
use crate::chatwidget::wallet_http::gateway_client;
use crate::chatwidget::wallet_menu::WalletPlanCatalog;
use crate::chatwidget::wallet_menu::WalletPlanStatus;
use crate::chatwidget::wallet_menu::title_case_plan;
use crate::chatwidget::wallet_render::WalletTextStyle;
use crate::chatwidget::wallet_render::push_wallet_text;
use zeroize::Zeroizing;

const WALLET_PLAN_USAGE_VIEW_ID: &str = "wallet-plan-usage";

#[derive(Debug)]
pub(crate) struct WalletPlanUsageOverview {
    status: WalletPlanStatus,
    plan_prices_usdc: std::collections::BTreeMap<String, String>,
    refreshed_at: String,
}

impl ChatWidget {
    pub(crate) fn open_wallet_plan_usage(&mut self) {
        self.show_selection_view(plan_usage_loading_params());
        let home = self.config.codex_home.as_path().to_path_buf();
        let key = super::wallet_http::corbanu_account_key(
            &home,
            self.config.cli_auth_credentials_store_mode,
            self.config.auth_keyring_backend_kind(),
        )
        .ok()
        .flatten();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = load_plan_usage(key).await;
            tx.send(AppEvent::WalletPlanUsageReady { result });
        });
    }

    pub(crate) fn on_wallet_plan_usage_ready(
        &mut self,
        result: Result<WalletPlanUsageOverview, String>,
    ) {
        let params = plan_usage_params(result);
        if self
            .bottom_pane
            .replace_selection_view_if_present(WALLET_PLAN_USAGE_VIEW_ID, params)
        {
            self.request_redraw();
        }
    }
}

async fn load_plan_usage(
    key: Option<Zeroizing<String>>,
) -> Result<WalletPlanUsageOverview, String> {
    let key = key.ok_or_else(|| {
        "Corbanu Plan is disconnected. Reconnect it from /wallet to view usage.".to_string()
    })?;
    let gateway = gateway_client()?;
    let account_request = gateway
        .client
        .get(format!("{}/v1/account", gateway.origin))
        .bearer_auth(key.as_str())
        .send();
    let catalog_request = gateway
        .client
        .get(format!("{}/v1/plans", gateway.origin))
        .send();
    let (account, catalog) = tokio::join!(account_request, catalog_request);
    let account = account.map_err(|error| format!("plan usage is unavailable: {error}"))?;
    if !account.status().is_success() {
        return Err(format!("plan usage returned HTTP {}", account.status()));
    }
    let status = account
        .json::<WalletPlanStatus>()
        .await
        .map_err(|error| format!("plan usage was malformed: {error}"))?;
    let plan_prices_usdc = match catalog {
        Ok(response) if response.status().is_success() => response
            .json::<WalletPlanCatalog>()
            .await
            .ok()
            .map(|catalog| {
                catalog
                    .plans
                    .into_iter()
                    .map(|plan| (plan.id, plan.price_usdc))
                    .collect()
            })
            .unwrap_or_default(),
        _ => std::collections::BTreeMap::new(),
    };
    Ok(WalletPlanUsageOverview {
        status,
        plan_prices_usdc,
        refreshed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}

fn plan_usage_loading_params() -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Corbanu Plan usage".bold()));
    header.push(Line::from("Loading authoritative usage…".dim()));
    SelectionViewParams {
        view_id: Some(WALLET_PLAN_USAGE_VIEW_ID),
        header: Box::new(header),
        items: vec![SelectionItem {
            name: "Loading…".to_string(),
            is_disabled: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn plan_usage_params(result: Result<WalletPlanUsageOverview, String>) -> SelectionViewParams {
    let mut header = ColumnRenderable::new();
    header.push(Line::from("Corbanu Plan usage".bold()));
    let items = match result {
        Ok(overview) => {
            push_usage_summary(&mut header, &overview);
            vec![
                crate::chatwidget::wallet_menu::item(
                    "Refresh usage",
                    "Fetch current settled and in-flight token usage",
                    || AppEvent::OpenWalletPlanUsage,
                ),
                crate::chatwidget::wallet_menu::item(
                    "Open wallet",
                    "Manage funds, receipts, plan upgrades, or disconnection",
                    || AppEvent::OpenWallet,
                ),
            ]
        }
        Err(error) => {
            header.push(Line::from(format!("Unavailable: {error}").red()));
            vec![crate::chatwidget::wallet_menu::item(
                "Retry",
                "Fetch Corbanu Plan usage again",
                || AppEvent::OpenWalletPlanUsage,
            )]
        }
    };
    SelectionViewParams {
        view_id: Some(WALLET_PLAN_USAGE_VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        header: Box::new(header),
        items,
        ..Default::default()
    }
}

fn push_usage_summary(header: &mut ColumnRenderable, overview: &WalletPlanUsageOverview) {
    let status = &overview.status;
    let now = chrono::Utc::now();
    let plan = title_case_plan(&status.period.plan_id);
    header.push(Line::from(format!("{plan} plan").green().bold()));
    if let Some(price) = overview.plan_prices_usdc.get(&status.period.plan_id) {
        push_wrapped(
            header,
            &format!("Spend: {price} USDC prepaid for this period · no recurring wallet charge"),
            /*dimmed*/ false,
        );
    }
    push_wrapped(
        header,
        &format!(
            "Active {} to {}",
            status.period.starts_at, status.period.ends_at
        ),
        /*dimmed*/ true,
    );
    push_window(
        header,
        "Weekly",
        status.weekly.used_tokens,
        status.weekly.reserved_tokens,
        status.weekly.limit_tokens,
        status.weekly_remaining_tokens,
        &format!(
            "resets {} · {}",
            countdown_at(&status.weekly.ends_at, now),
            status.weekly.ends_at
        ),
    );
    push_window(
        header,
        "Monthly",
        status.period.monthly_used_tokens,
        status.period.monthly_reserved_tokens,
        status.period.monthly_limit_tokens,
        status.monthly_remaining_tokens,
        &format!(
            "period ends {} · {}",
            countdown_at(&status.period.ends_at, now),
            status.period.ends_at
        ),
    );
    if let Some(next) = status.queued_periods.first() {
        let next_price = overview
            .plan_prices_usdc
            .get(&next.plan_id)
            .map_or_else(String::new, |price| format!(" · {price} USDC prepaid"));
        push_wrapped(
            header,
            &format!(
                "Next: {} plan{next_price} · {} to {}",
                title_case_plan(&next.plan_id),
                next.starts_at,
                next.ends_at
            ),
            /*dimmed*/ true,
        );
    }
    push_wrapped(
        header,
        &format!("Refreshed {}", overview.refreshed_at),
        /*dimmed*/ true,
    );
}

fn countdown_at(value: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    let Ok(end) = chrono::DateTime::parse_from_rfc3339(value) else {
        return "at the timestamp shown".to_string();
    };
    let seconds = (end.with_timezone(&chrono::Utc) - now).num_seconds();
    if seconds <= 0 {
        return "already ended".to_string();
    }
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("in {days}d {hours}h")
    } else if hours > 0 {
        format!("in {hours}h {minutes}m")
    } else {
        format!("in {}m", minutes.max(1))
    }
}

fn push_window(
    header: &mut ColumnRenderable,
    label: &str,
    used: u64,
    reserved: u64,
    limit: u64,
    remaining: u64,
    reset: &str,
) {
    let (usage_line, remaining_line) = window_lines(label, used, reserved, limit, remaining, reset);
    push_wrapped(header, &usage_line, /*dimmed*/ false);
    push_wrapped(header, &remaining_line, /*dimmed*/ true);
}

fn push_wrapped(header: &mut ColumnRenderable, text: &str, dimmed: bool) {
    let style = if dimmed {
        WalletTextStyle::Dimmed
    } else {
        WalletTextStyle::Normal
    };
    push_wallet_text(header, text, style);
}

fn window_lines(
    label: &str,
    used: u64,
    reserved: u64,
    limit: u64,
    remaining: u64,
    reset: &str,
) -> (String, String) {
    let percent = if limit == 0 {
        0.0
    } else {
        (used.saturating_add(reserved) as f64 / limit as f64) * 100.0
    };
    (
        format!(
            "{label}: {} used + {} in flight of {} tokens ({percent:.1}%)",
            commas(used),
            commas(reserved),
            commas(limit)
        ),
        format!("{} remaining · {reset}", commas(remaining)),
    )
}

fn commas(value: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chatwidget::wallet_menu::WalletPlanPeriod;
    use crate::chatwidget::wallet_menu::WalletQueuedPlanPeriod;
    use crate::chatwidget::wallet_menu::WalletUsageWindow;

    fn overview() -> WalletPlanUsageOverview {
        WalletPlanUsageOverview {
            status: WalletPlanStatus {
                wallet_address: "wallet".to_string(),
                period: WalletPlanPeriod {
                    transaction: "transaction".to_string(),
                    plan_id: "starter".to_string(),
                    starts_at: "2026-07-19T00:35:20Z".to_string(),
                    ends_at: "2026-08-19T00:35:20Z".to_string(),
                    monthly_limit_tokens: 1_000_000,
                    monthly_used_tokens: 20_996,
                    monthly_reserved_tokens: 500,
                },
                weekly: WalletUsageWindow {
                    ends_at: "2026-07-26T00:35:20Z".to_string(),
                    limit_tokens: 250_000,
                    used_tokens: 20_996,
                    reserved_tokens: 500,
                },
                monthly_remaining_tokens: 978_504,
                weekly_remaining_tokens: 228_504,
                queued_periods: vec![WalletQueuedPlanPeriod {
                    transaction: "queued".to_string(),
                    plan_id: "basic".to_string(),
                    starts_at: "2026-08-19T00:35:20Z".to_string(),
                    ends_at: "2026-09-19T00:35:20Z".to_string(),
                }],
            },
            plan_prices_usdc: std::collections::BTreeMap::from([
                ("starter".to_string(), "1".to_string()),
                ("basic".to_string(), "20".to_string()),
            ]),
            refreshed_at: "2026-07-19T04:00:00Z".to_string(),
        }
    }

    #[test]
    fn usage_summary_exposes_spend_used_reserved_limits_and_resets() {
        let value = overview();
        assert_eq!(
            value.plan_prices_usdc.get("starter").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            value.plan_prices_usdc.get("basic").map(String::as_str),
            Some("20")
        );
        assert_eq!(value.status.queued_periods[0].plan_id, "basic");
        let weekly = window_lines(
            "Weekly",
            value.status.weekly.used_tokens,
            value.status.weekly.reserved_tokens,
            value.status.weekly.limit_tokens,
            value.status.weekly_remaining_tokens,
            "window 2026-07-19T00:35:20Z to 2026-07-26T00:35:20Z",
        );
        assert_eq!(
            weekly.0,
            "Weekly: 20,996 used + 500 in flight of 250,000 tokens (8.6%)"
        );
        assert_eq!(
            weekly.1,
            "228,504 remaining · window 2026-07-19T00:35:20Z to 2026-07-26T00:35:20Z"
        );
        let monthly = window_lines(
            "Monthly",
            value.status.period.monthly_used_tokens,
            value.status.period.monthly_reserved_tokens,
            value.status.period.monthly_limit_tokens,
            value.status.monthly_remaining_tokens,
            "period ends 2026-08-19T00:35:20Z",
        );
        assert_eq!(
            monthly.0,
            "Monthly: 20,996 used + 500 in flight of 1,000,000 tokens (2.1%)"
        );
    }

    #[test]
    fn comma_format_is_exact() {
        assert_eq!(commas(/*value*/ 0), "0");
        assert_eq!(commas(/*value*/ 999), "999");
        assert_eq!(commas(/*value*/ 1_000), "1,000");
        assert_eq!(commas(/*value*/ 12_345_678), "12,345,678");
    }

    #[test]
    fn reset_countdown_is_human_readable_and_keeps_exact_time_separate() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-07-19T00:35:20Z")
            .expect("now")
            .with_timezone(&chrono::Utc);
        assert_eq!(countdown_at("2026-07-26T03:05:20Z", now), "in 7d 2h");
        assert_eq!(countdown_at("2026-07-18T00:00:00Z", now), "already ended");
    }
}
