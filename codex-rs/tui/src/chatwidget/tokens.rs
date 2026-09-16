//! Coordinates asynchronous `/usage` cards in the chat widget.
//!
//! The slash command builds a composite history cell immediately, but the widget
//! keeps that cell transient while the account request runs. The transient card is
//! rendered above the composer through [`ChatWidget::pending_token_activity_output`]
//! so loading never requires clearing or rewriting transcript history. When the
//! matching response arrives, [`TokenActivityHandle`] updates the shared card state
//! and [`ChatWidget::finish_token_activity_refresh`] moves the cell into a completed
//! slot. Event dispatch commits that completed cell into history only after active
//! output and stream consolidation no longer block insertion.
//!
//! Pure chart rendering and date bucketing live in [`chart`]. This module owns
//! request correlation, transient/completed card state, and integration with
//! `ChatWidget` history insertion.

mod chart;

use std::sync::Arc;
use std::sync::RwLock;

use chrono::NaiveDate;
use chrono::Utc;
use codex_app_server_protocol::GetAccountTokenUsageResponse;
use ratatui::style::Stylize;
use ratatui::text::Line;

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::history_cell::CompositeHistoryCell;
use crate::history_cell::HistoryCell;
use crate::history_cell::PlainHistoryCell;
use crate::history_cell::plain_lines;

pub(crate) use chart::TokenActivityView;

/// Tracks the renderable lifecycle of one token activity history cell.
#[derive(Debug)]
enum TokenActivityState {
    Loading,
    Loaded {
        response: GetAccountTokenUsageResponse,
        today: NaiveDate,
    },
    Error,
}

/// Completes an asynchronously rendered token activity history cell.
///
/// Clones share the same card state, allowing the background request path to
/// update a cell still owned by the widget's transient-output state. The widget
/// remains responsible for request-ID matching, redraws, and history insertion.
#[derive(Clone, Debug)]
pub(super) struct TokenActivityHandle {
    state: Arc<RwLock<TokenActivityState>>,
}

/// Holds the one transient token activity card waiting on its background response.
///
/// The request ID prevents late results from mutating a newer `/usage` card. The
/// cell stays out of transcript history until the matching response completes and
/// the widget confirms that active output no longer blocks insertion.
pub(super) struct PendingTokenActivityOutput {
    request_id: u64,
    cell: CompositeHistoryCell,
    handle: TokenActivityHandle,
}

impl TokenActivityHandle {
    /// Replaces the loading state with either fetched activity or an unavailable state.
    ///
    /// This method intentionally discards the error string because the TUI exposes
    /// one stable unavailable message. Calling it more than once replaces the prior
    /// terminal state, so request-ID matching should happen before completion.
    pub(super) fn finish(&self, result: Result<GetAccountTokenUsageResponse, String>) {
        self.finish_with_today(result, Utc::now().date_naive());
    }

    fn finish_with_today(
        &self,
        result: Result<GetAccountTokenUsageResponse, String>,
        today: NaiveDate,
    ) {
        let state = match result {
            Ok(response) => TokenActivityState::Loaded { response, today },
            Err(_) => TokenActivityState::Error,
        };
        #[expect(clippy::expect_used)]
        let mut current = self.state.write().expect("token activity state poisoned");
        *current = state;
    }
}

/// Renders one `/usage` card from shared asynchronous state.
#[derive(Debug)]
struct TokenActivityHistoryCell {
    view: TokenActivityView,
    state: Arc<RwLock<TokenActivityState>>,
}

/// Creates the card contents and completion handle for one `/usage` invocation.
///
/// The composite cell includes the echoed slash command and a loading card from
/// the start. Callers must retain the returned handle and complete it when the
/// matching background response arrives; otherwise the transient card stays loading.
pub(super) fn new_token_activity_output(
    view: TokenActivityView,
) -> (CompositeHistoryCell, TokenActivityHandle) {
    let command = PlainHistoryCell::new(vec![
        format!("/usage {}", view.label().to_lowercase())
            .magenta()
            .into(),
    ]);
    let state = Arc::new(RwLock::new(TokenActivityState::Loading));
    let handle = TokenActivityHandle {
        state: Arc::clone(&state),
    };
    let card = TokenActivityHistoryCell { view, state };
    (
        CompositeHistoryCell::new(vec![Box::new(command), Box::new(card)]),
        handle,
    )
}

impl HistoryCell for TokenActivityHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        #[expect(clippy::expect_used)]
        let state = self.state.read().expect("token activity state poisoned");
        match &*state {
            TokenActivityState::Loading => {
                vec![
                    " Token activity".bold().into(),
                    "   Loading...".dim().into(),
                ]
            }
            TokenActivityState::Error => vec![
                " Token activity".bold().into(),
                "   Token activity unavailable".dim().into(),
            ],
            TokenActivityState::Loaded { response, today } => {
                chart::loaded_lines(self.view, response, *today, width)
            }
        }
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.display_lines(u16::MAX))
    }
}

impl ChatWidget {
    /// Starts a token activity refresh and replaces the current transient card.
    ///
    /// Each invocation receives a request ID so background responses update only
    /// their own card. The card remains outside transcript history until completion,
    /// which keeps loading visible without disturbing existing transcript content.
    pub(crate) fn add_token_activity_output(&mut self, view: TokenActivityView) {
        let request_id = self.next_token_activity_request_id;
        self.next_token_activity_request_id =
            self.next_token_activity_request_id.wrapping_add(/*rhs*/ 1);
        let (cell, handle) = new_token_activity_output(view);
        self.completed_token_activity_output = None;
        self.refreshing_token_activity_output = Some(PendingTokenActivityOutput {
            request_id,
            cell,
            handle,
        });
        self.bump_active_cell_revision();
        self.request_redraw();
        self.app_event_tx
            .send(AppEvent::RefreshTokenActivity { request_id });
    }

    /// Returns the transient token activity card that should render above the composer.
    ///
    /// A loading card takes precedence over a completed card waiting for history
    /// insertion. Callers should render the returned cell but leave ownership with
    /// the widget so completion and insertion can update it safely.
    pub(super) fn pending_token_activity_output(&self) -> Option<&dyn HistoryCell> {
        self.refreshing_token_activity_output
            .as_ref()
            .map(|output| &output.cell as &dyn HistoryCell)
            .or_else(|| {
                self.completed_token_activity_output
                    .as_ref()
                    .map(|cell| cell as &dyn HistoryCell)
            })
    }

    /// Applies a background token activity result to its matching transient card.
    ///
    /// Returns `true` when the pending request matched and moved into the completed
    /// slot. Late responses return `false`, including responses for cards replaced
    /// by a newer `/usage` invocation or cleared during transcript changes.
    pub(crate) fn finish_token_activity_refresh(
        &mut self,
        request_id: u64,
        result: Result<GetAccountTokenUsageResponse, String>,
    ) -> bool {
        let Some(output) = self.refreshing_token_activity_output.take() else {
            return false;
        };
        if output.request_id != request_id {
            self.refreshing_token_activity_output = Some(output);
            return false;
        }
        output.handle.finish(result);
        self.completed_token_activity_output = Some(output.cell);
        self.bump_active_cell_revision();
        self.request_redraw();
        true
    }

    /// Reports whether completed asynchronous usage output must wait before insertion.
    ///
    /// Inserting while a stream, queued consolidation, or active transcript cell is
    /// present can reorder output relative to visible work, so callers retry once
    /// these barriers clear.
    pub(crate) fn usage_history_insertion_blocked(&self) -> bool {
        self.stream_controller.is_some()
            || self.plan_stream_controller.is_some()
            || self.pending_stream_consolidations > 0
            || self.transcript.active_cell.is_some()
            || self.active_hook_cell.is_some()
    }

    /// Records a stream consolidation barrier that delays token card insertion.
    ///
    /// Each queued consolidation should eventually call
    /// [`ChatWidget::note_stream_consolidation_completed`].
    pub(crate) fn note_stream_consolidation_queued(&mut self) {
        self.pending_stream_consolidations =
            self.pending_stream_consolidations.saturating_add(/*rhs*/ 1);
    }

    /// Releases one queued stream consolidation barrier.
    ///
    /// The counter saturates at zero so an unmatched completion does not underflow,
    /// but paired queue/completion calls are still the intended contract.
    pub(crate) fn note_stream_consolidation_completed(&mut self) {
        self.pending_stream_consolidations =
            self.pending_stream_consolidations.saturating_sub(/*rhs*/ 1);
    }

    /// Transfers the completed token activity card into the history insertion path.
    ///
    /// Callers should use this only after
    /// [`ChatWidget::usage_history_insertion_blocked`] returns `false`;
    /// taking the card removes it from the transient render area.
    pub(crate) fn take_completed_token_activity_output(&mut self) -> Option<CompositeHistoryCell> {
        let output = self.completed_token_activity_output.take()?;
        self.bump_active_cell_revision();
        Some(output)
    }

    /// Requests another insertion attempt when completed usage output is waiting.
    ///
    /// This is used after stream or history lifecycle events that may have cleared
    /// the insertion barriers without directly owning the completed output.
    pub(crate) fn request_pending_usage_output_insertion(&self) {
        if self.completed_token_activity_output.is_some()
            || self.pending_rate_limit_reset_hint().is_some()
        {
            self.app_event_tx.send(AppEvent::CommitPendingUsageOutput);
        }
    }

    pub(crate) fn request_pending_usage_output_insertion_after_stream_shutdown(&self) {
        if self.completed_token_activity_output.is_some()
            || self.pending_rate_limit_reset_hint().is_some()
        {
            self.app_event_tx
                .send(AppEvent::CommitPendingUsageOutputAfterStreamShutdown);
        }
    }

    /// Drops transient and completed token cards that must no longer update.
    ///
    /// Late background responses cannot mutate cards after a transcript reset,
    /// backtrack, or replacement flow clears this widget-owned state.
    pub(crate) fn clear_pending_token_activity_refreshes(&mut self) {
        self.accounting_inspector = None;
        self.bottom_pane.dismiss_view_by_id(INSPECTOR_VIEW);
        let cleared_refresh = self.refreshing_token_activity_output.take().is_some();
        let cleared_completed = self.completed_token_activity_output.take().is_some();
        if cleared_refresh || cleared_completed {
            self.bump_active_cell_revision();
            self.request_redraw();
        }
    }
}

// Inspector pages contain local evidence only; never insert them into history.
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use codex_protocol::ThreadId;
use codex_state::accounting::BucketQuote;
use codex_state::accounting::Decimal;
use codex_state::accounting::InspectionDay;
use codex_state::accounting::ObservationQuote;
use uuid::Uuid;

const INSPECTOR_VIEW: &str = "recorded-requests";

pub(super) struct Inspector {
    generation: Uuid,
    thread: Option<ThreadId>,
    day: i64,
    pages: Vec<InspectorPage>,
    page: usize,
    alive: Arc<std::sync::atomic::AtomicBool>,
}

struct InspectorPage {
    title: String,
    text: Vec<String>,
    links: Vec<(String, usize)>,
    parent: Option<usize>,
    selected: Arc<std::sync::atomic::AtomicUsize>,
}

fn money(value: Decimal) -> String {
    let display = value.display();
    format!(
        "${}{}{}",
        display.text,
        if display.rounded { " (rounded)" } else { "" },
        if display.nonzero_sub_micro {
            " (nonzero, less than $0.000001)"
        } else {
            ""
        }
    )
}

fn exact(value: Decimal) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unavailable".to_string())
}

fn estimate(known: Decimal, unknown: i64, attempts: i64) -> Vec<String> {
    if attempts == 0 {
        return vec!["No recorded attempts in this day; collection coverage unknown.".into()];
    }
    if unknown == 0 {
        return vec![format!(
            "Estimated token cost for recorded attempts: {}",
            money(known)
        )];
    }
    let mut lines = vec![
        format!(
            "Known estimated token cost: {} + unknown costs",
            money(known)
        ),
        format!(
            "Full recorded estimate: unavailable ({unknown} of {attempts} attempts incomplete)"
        ),
    ];
    if known == Decimal::default() {
        lines.insert(0, "Estimated token cost: unknown".into());
    }
    lines
}

const METRICS: [&str; 7] = [
    "Input",
    "Noncached input (derived for inclusive input)",
    "Cache read",
    "Cache write",
    "Output",
    "Reasoning (subset, not separately billed)",
    "Total (not separately billed)",
];
const BUCKETS: [&str; 4] = ["Noncached input", "Cache read", "Cache write", "Output"];

fn attempt_text(q: &ObservationQuote) -> Vec<String> {
    let a = &q.attempt;
    let mut lines = vec![
        format!("Request: {}", a.request_id),
        format!("Attempt: {}", a.attempt_id),
        format!("Thread: {}", a.thread_id),
        format!("Turn: {}", a.turn),
        format!("Provider: {}", a.provider),
        format!("Model: {}", a.model),
        format!("Opaque scope: {}", a.scope),
        format!("Arithmetic dialect: {:?}", a.dialect),
        format!(
            "Admission time: {} ms since Unix epoch (UTC)",
            i64::from(a.dispatched_at_ms)
        ),
        format!(
            "Retry predecessor: {}",
            a.retry_of.map_or("none".into(), |v| v.to_string())
        ),
        "Completion/billing status: not recorded. Literal wire/endpoint and usage observation wall time: unavailable"
            .into(),
    ];
    lines.extend(estimate(
        q.known_subtotal,
        i64::from(q.all_buckets_priced.is_none()),
        1,
    ));
    let u = &q.usage;
    for (index, (label, value)) in METRICS
        .iter()
        .zip([
            u.input,
            u.noncached,
            u.read,
            u.write,
            u.output,
            u.reasoning,
            u.total,
        ])
        .enumerate()
    {
        let derived = match a.dialect {
            codex_state::accounting::Dialect::Inclusive => index == 1,
            codex_state::accounting::Dialect::NativeAnthropic => index == 0 || index == 6,
            codex_state::accounting::Dialect::UnknownCompatible => false,
        };
        lines.push(format!(
            "{label}: {}",
            value.map_or("unknown — no retained numeric evidence".into(), |n| {
                if n == 0 {
                    if derived {
                        "0 (derived)"
                    } else {
                        "0 (reported)"
                    }
                    .into()
                } else {
                    format!("{n}{}", if derived { " (derived)" } else { "" })
                }
            })
        ));
    }
    for (label, bucket) in BUCKETS.iter().zip(q.buckets) {
        lines.push(format!(
            "{label} cost: {}",
            match bucket {
                BucketQuote::Priced(v) => format!("{}; exact USD {}", money(v), exact(v)),
                BucketQuote::MissingUsage => "unknown — no retained numeric evidence".into(),
                BucketQuote::MissingRate => "unknown — rate unavailable".into(),
            }
        ));
    }
    lines.push(format!(
        "Known subtotal exact USD: {}",
        exact(q.known_subtotal)
    ));
    if let Some(s) = &q.snapshot {
        lines.extend([
            format!("Price ID: {}", s.id),
            format!(
                "Price source: {:?}; reference {}",
                s.source_kind, s.source_reference
            ),
            format!("Price currency/unit: {:?} / {:?}", s.currency, s.unit),
            format!(
                "Price observed/approved: {} / {} ms UTC",
                i64::from(s.observed_at_ms),
                i64::from(s.approved_at_ms)
            ),
            format!(
                "Price effective interval: [{}, {}) ms UTC",
                i64::from(s.effective_from_ms),
                s.effective_end_ms
                    .map_or("unbounded".into(), |n| i64::from(n).to_string())
            ),
        ]);
        for (label, rate) in BUCKETS.iter().zip([
            s.rates.noncached,
            s.rates.read,
            s.rates.write,
            s.rates.output,
        ]) {
            lines.push(rate.map_or_else(
                || format!("Rate unavailable for {label}"),
                |r| format!("{label} rate: {} USD per million tokens", exact(r)),
            ));
        }
    } else {
        lines.push("Price: unavailable — no dispatch-time price snapshot".into());
    }
    for o in &q.observations {
        lines.push(format!(
            "Evidence revision {}; source {}; sequence {}; presence patch {}",
            i64::from(o.revision),
            o.source,
            i64::from(o.sequence),
            serde_json::to_string(&o.patch).unwrap_or_default()
        ));
    }
    lines
}

fn inspection_pages(result: Result<InspectionDay, String>) -> Vec<InspectorPage> {
    let mut pages = vec![InspectorPage { title: "Recorded requests — this thread only".into(), text: vec![
        "Collection coverage: unknown; recorded attempts only. Descendants excluded.".into(),
        "Billed cost: unavailable — no settlement evidence".into(),
        "Logical requests may have attempts on other days; this UTC day is not their complete lifetime.".into(),
    ], links: Vec::new(), parent: None, selected: Arc::default() }];
    let ready = match result {
        Ok(InspectionDay::Ready(view)) => view,
        other => {
            pages[0].text.push(match other {
                Ok(InspectionDay::Absent) => "Unavailable — accounting ledger not installed. Collection remains off.".into(),
                Ok(InspectionDay::MissingThread) => "Unavailable — native thread no longer exists.".into(),
                Ok(InspectionDay::CheckpointLag) => "Snapshot is not current; newer activity is unverified".into(),
                Ok(InspectionDay::NeedsRefresh) => "Recorded totals unavailable — stored contributions need refresh. Retry rereads only; no repair performed.".into(),
                Ok(InspectionDay::TooLarge) => "Range too large for this inspector. No total shown.".into(),
                Ok(InspectionDay::DetailUnavailable { coverage, read_at_ms, compact }) => format!(
                    "Request detail unavailable for this whole UTC day — {}. No total shown. Store checkpoint: {}; aggregate day floor: {}; oldest recorded day: {:?}; 90-day wall-clock detail cutoff: {:?} ms UTC.",
                    if compact { "compacted history lost request/provider attribution" } else { "day touches expired detail or aggregate history" },
                    coverage.completed_as_of_ms, coverage.aggregate_day_floor, coverage.oldest_recorded_day, read_at_ms.checked_sub(90 * 86_400_000).filter(|n| *n >= 0)),
                Err(message) => message,
                Ok(InspectionDay::Ready(_)) => unreachable!(),
            });
            return pages;
        }
    };
    if ready.read_at_ms > ready.coverage.completed_as_of_ms {
        pages[0]
            .text
            .push("Snapshot is not current; newer activity is unverified".into());
    }
    let t = &ready.totals;
    pages[0]
        .text
        .splice(0..0, estimate(t.known_usd, t.unknown_estimates, t.attempts));
    pages[0].text.extend([
        format!("UTC admission interval: [{}, {}) ms since Unix epoch", ready.utc_day * 86_400_000, (ready.utc_day + 1) * 86_400_000),
        format!("Read at: {} ms UTC; store checkpoint: {} ms UTC; maintenance lag: {} ms",
            ready.read_at_ms, ready.coverage.completed_as_of_ms, ready.read_at_ms - ready.coverage.completed_as_of_ms),
        format!("90-day wall-clock detail cutoff: {:?}; aggregate day floor at checkpoint: {}; oldest recorded day: {:?}",
            ready.read_at_ms.checked_sub(90 * 86_400_000).filter(|n| *n >= 0), ready.coverage.aggregate_day_floor, ready.coverage.oldest_recorded_day),
        format!("Known subtotal exact USD: {}", exact(t.known_usd)),
    ]);
    for (label, m) in METRICS.iter().zip(&t.measured) {
        pages[0].text.push(format!(
            "{label}: {} known + unknown in {} attempts",
            m.known, m.unknown
        ));
    }
    for (request, quotes) in ready.requests {
        let request_page = pages.len();
        let number = pages[0].links.len() + 1;
        pages[0]
            .links
            .push((format!("Request {number}"), request_page));
        pages.push(InspectorPage {
            title: "Logical request".into(),
            text: vec![format!("Request: {request}")],
            links: Vec::new(),
            parent: Some(0),
            selected: Arc::default(),
        });
        for (index, quote) in quotes.iter().enumerate() {
            let target = pages.len();
            pages[request_page]
                .links
                .push((format!("Attempt {}", index + 1), target));
            pages.push(InspectorPage {
                title: "Attempt, components and original price".into(),
                text: attempt_text(quote),
                links: Vec::new(),
                parent: Some(request_page),
                selected: Arc::default(),
            });
        }
    }
    pages
}

impl Inspector {
    fn params(&self) -> SelectionViewParams {
        let page = &self.pages[self.page];
        let generation = self.generation;
        // Small selectable fragments keep every long field reachable at 40 columns.
        let mut items: Vec<SelectionItem> = page
            .text
            .iter()
            .flat_map(|text| {
                let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                textwrap::wrap(&clean, 26)
                    .into_iter()
                    .map(|part| SelectionItem {
                        name: part.into_owned(),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        for (label, target) in &page.links {
            let page = *target;
            items.push(SelectionItem {
                name: label.clone(),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::NavigateAccountingInspector { generation, page })
                })],
                ..Default::default()
            });
        }
        if let Some(page) = page.parent {
            items.push(SelectionItem {
                name: "Back".into(),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::NavigateAccountingInspector { generation, page })
                })],
                ..Default::default()
            });
        }
        items.push(SelectionItem {
            name: "Refresh".into(),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::RefreshAccountingInspector { generation })
            })],
            ..Default::default()
        });
        let close_alive = self.alive.clone();
        items.push(SelectionItem {
            name: "Close".into(),
            dismiss_on_select: true,
            actions: vec![Box::new(move |tx| {
                close_alive.store(false, std::sync::atomic::Ordering::Release);
                tx.send(AppEvent::CloseAccountingInspector { generation });
            })],
            ..Default::default()
        });
        let alive = self.alive.clone();
        let parent = page.parent;
        let selected = page.selected.clone();
        SelectionViewParams {
            initial_selected_idx: Some(selected.load(std::sync::atomic::Ordering::Relaxed)),
            on_selection_changed: Some(Box::new(move |index, _| {
                selected.store(index, std::sync::atomic::Ordering::Relaxed);
            })),
            view_id: Some(INSPECTOR_VIEW),
            title: Some(page.title.clone()),
            items,
            allow_number_shortcuts: false,
            footer_hint: Some("↑↓ scroll · Enter open · Esc back/close".into()),
            on_cancel: Some(Box::new(move |tx| {
                if let Some(page) = parent {
                    tx.send(AppEvent::NavigateAccountingInspector { generation, page });
                } else {
                    alive.store(false, std::sync::atomic::Ordering::Release);
                    tx.send(AppEvent::CloseAccountingInspector { generation });
                }
            })),
            ..Default::default()
        }
    }
}

impl ChatWidget {
    pub(super) fn invalidate_accounting_inspector_for_thread(&mut self) {
        if self
            .accounting_inspector
            .as_ref()
            .is_some_and(|v| v.thread != self.thread_id())
        {
            self.accounting_inspector = None;
            self.bottom_pane.dismiss_view_by_id(INSPECTOR_VIEW);
        }
    }

    pub(super) fn open_accounting_command(&mut self, args: &str, today: NaiveDate) {
        let parts: Vec<_> = args.split_whitespace().collect();
        let date = match parts.as_slice() {
            ["requests"] => Some(today),
            ["requests", date] if date.len() == 10 => NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .ok()
                .filter(|d| d.to_string() == *date),
            _ => None,
        };
        if let Some(date) = date.filter(|d| *d <= today)
            && let Some(time) = date.and_hms_opt(0, 0, 0)
            && time.and_utc().timestamp() >= 0
        {
            self.open_accounting_inspector(time.and_utc().timestamp() / 86_400);
        } else {
            self.add_error_message(
                "Usage: /usage requests [YYYY-MM-DD] (UTC, no future dates)".into(),
            );
        }
    }

    pub(crate) fn open_accounting_inspector(&mut self, day: i64) {
        let generation = Uuid::new_v4();
        let thread = self.thread_id();
        let inspector = Inspector {
            generation,
            thread,
            day,
            pages: vec![InspectorPage {
                title: "Recorded requests".into(),
                text: vec!["Loading recorded requests…".into()],
                links: Vec::new(),
                parent: None,
                selected: Arc::default(),
            }],
            page: 0,
            alive: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        };
        let params = inspector.params();
        if !self
            .bottom_pane
            .replace_selection_view_if_present(INSPECTOR_VIEW, inspector.params())
        {
            self.bottom_pane.show_selection_view(params);
        }
        self.accounting_inspector = Some(inspector);
        self.app_event_tx.send(AppEvent::LoadAccountingInspector {
            generation,
            thread,
            day,
        });
        self.request_redraw();
    }

    pub(crate) fn finish_accounting_inspector(
        &mut self,
        generation: Uuid,
        thread: Option<ThreadId>,
        day: i64,
        result: Result<InspectionDay, String>,
    ) {
        let current_thread = self.thread_id();
        let Some(view) = self.accounting_inspector.as_mut().filter(|v| {
            v.generation == generation
                && v.thread == thread
                && v.day == day
                && v.thread == current_thread
                && v.alive.load(std::sync::atomic::Ordering::Acquire)
        }) else {
            return;
        };
        view.pages = inspection_pages(result);
        view.page = 0;
        if !self
            .bottom_pane
            .replace_selection_view_if_present(INSPECTOR_VIEW, view.params())
        {
            self.accounting_inspector = None;
        }
        self.request_redraw();
    }

    pub(crate) fn navigate_accounting_inspector(&mut self, generation: Uuid, page: usize) {
        let thread = self.thread_id();
        let Some(view) = self.accounting_inspector.as_mut().filter(|v| {
            v.thread == thread
                && v.generation == generation
                && v.alive.load(std::sync::atomic::Ordering::Acquire)
        }) else {
            return;
        };
        if page >= view.pages.len() {
            return;
        }
        view.page = page;
        if !self
            .bottom_pane
            .replace_selection_view_if_present(INSPECTOR_VIEW, view.params())
        {
            self.bottom_pane.show_selection_view(view.params());
        }
        self.request_redraw();
    }

    pub(crate) fn close_accounting_inspector(&mut self, generation: Uuid) {
        if self
            .accounting_inspector
            .as_ref()
            .is_some_and(|v| v.generation == generation)
        {
            self.accounting_inspector = None;
            self.bottom_pane.dismiss_view_by_id(INSPECTOR_VIEW);
        }
    }

    pub(crate) fn refresh_accounting_inspector(&mut self, generation: Uuid) {
        if let Some(view) = &self.accounting_inspector
            && view.generation == generation
            && view.alive.load(std::sync::atomic::Ordering::Acquire)
        {
            self.open_accounting_inspector(view.day);
        }
    }
}

#[cfg(test)]
#[path = "tokens_tests.rs"]
mod tests;
