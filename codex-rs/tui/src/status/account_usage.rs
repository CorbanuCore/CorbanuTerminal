//! PF-84: the per-account token summary `/status` shows while named accounts
//! are on.
//!
//! Every thread of the session (the coordinator, its spawned agents and side
//! conversations) reports its provider-reported usage per (provider, account)
//! with each token count update. The summary adds the latest report of every
//! thread, so a spawned agent on another account counts toward that account.
//! Token counts only: no prices, no cost records, nothing stored.

use std::collections::BTreeMap;
use std::collections::HashMap;

use codex_app_server_protocol::ThreadAccountTokenUsage;
use codex_protocol::ThreadId;
use ratatui::prelude::*;
use ratatui::style::Stylize;

use crate::token_usage::TokenUsage;

/// The latest per-account usage each thread of this session reported.
#[derive(Debug, Clone, Default)]
pub(crate) struct SessionAccountUsage {
    by_thread: HashMap<ThreadId, Vec<ThreadAccountTokenUsage>>,
}

impl SessionAccountUsage {
    /// Replaces `thread_id`'s report: each one is that thread's running total.
    pub(crate) fn record(&mut self, thread_id: ThreadId, usage: Vec<ThreadAccountTokenUsage>) {
        self.by_thread.insert(thread_id, usage);
    }

    /// Whether `thread_id` has reported in this session.
    pub(crate) fn has_thread(&self, thread_id: ThreadId) -> bool {
        self.by_thread.contains_key(&thread_id)
    }

    pub(crate) fn clear(&mut self) {
        self.by_thread.clear();
    }

    /// The accounts of `provider_id` this session used: `active` first (also
    /// before it has usage), then the others by name.
    pub(crate) fn for_provider(&self, provider_id: &str, active: &str) -> StatusAccountUsage {
        let mut totals: BTreeMap<&str, TokenUsage> = BTreeMap::new();
        for entry in self
            .by_thread
            .values()
            .flatten()
            .filter(|entry| entry.provider_id == provider_id)
        {
            let total = totals.entry(entry.account.as_str()).or_default();
            total.input_tokens += entry.usage.input_tokens;
            total.cached_input_tokens += entry.usage.cached_input_tokens;
            total.output_tokens += entry.usage.output_tokens;
            total.reasoning_output_tokens += entry.usage.reasoning_output_tokens;
            total.total_tokens += entry.usage.total_tokens;
        }
        let active_usage = totals.remove(active).unwrap_or_default();
        let mut rows = vec![(active.to_string(), active_usage)];
        rows.extend(
            totals
                .into_iter()
                .map(|(account, usage)| (account.to_string(), usage)),
        );
        StatusAccountUsage {
            rows,
            named_account_in_home: None,
        }
    }
}

/// What `/status` shows for the active provider's accounts. The first row is
/// the session's account.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatusAccountUsage {
    pub(crate) rows: Vec<(String, TokenUsage)>,
    /// D86/#1193: whether the provider has a named account in this home, as
    /// the background check last found; `None` while unknown.
    pub(crate) named_account_in_home: Option<bool>,
}

impl StatusAccountUsage {
    /// D86/D125: whether `/status` names the account and shows these rows:
    /// the session is on (or used) a named account of the provider, or the
    /// home holds one. Unknown counts as holding one, so the account is
    /// shown rather than hidden. Never reads the vault.
    pub(crate) fn provider_has_named_account(&self) -> bool {
        self.rows
            .iter()
            .any(|(account, _)| account != codex_vault::DEFAULT_PROVIDER_ACCOUNT)
            || self.named_account_in_home.unwrap_or(true)
    }

    /// One line (wrapped by the caller) for a single account; a header and
    /// one aligned row per account, the active one marked, for several.
    pub(crate) fn lines(&self) -> Vec<Vec<Span<'static>>> {
        match self.rows.as_slice() {
            [] => Vec::new(),
            [(account, usage)] => vec![single_line(account, usage)],
            rows => table_lines(rows),
        }
    }
}

fn single_line(account: &str, usage: &TokenUsage) -> Vec<Span<'static>> {
    if usage.is_zero() {
        return vec![
            Span::from(account.to_string()),
            Span::from(" · no usage yet").dim(),
        ];
    }
    // Input is the uncached part, as in "Token usage"; reasoning is part of
    // the output. Cached input and reasoning appear only when reported.
    let mut detail = format!("{} input", grouped(usage.non_cached_input()));
    if usage.cached_input() > 0 {
        detail.push_str(&format!(
            " · {} cached input",
            grouped(usage.cached_input())
        ));
    }
    detail.push_str(&format!(" · {} output", grouped(usage.output_tokens)));
    if usage.reasoning_output_tokens > 0 {
        detail.push_str(&format!(
            " ({} reasoning)",
            grouped(usage.reasoning_output_tokens)
        ));
    }
    vec![
        Span::from(account.to_string()),
        Span::from(" · ").dim(),
        Span::from(detail),
    ]
}

fn table_lines(rows: &[(String, TokenUsage)]) -> Vec<Vec<Span<'static>>> {
    // Cached input and reasoning appear only where a provider reported them.
    let cached = rows.iter().any(|(_, usage)| usage.cached_input() > 0);
    let reasoning = rows
        .iter()
        .any(|(_, usage)| usage.reasoning_output_tokens > 0);
    let mut header = vec!["account", "input"];
    if cached {
        header.push("cached input");
    }
    header.push("output");
    if reasoning {
        header.push("reasoning");
    }
    let cells: Vec<Vec<String>> = rows
        .iter()
        .enumerate()
        .map(|(index, (account, usage))| {
            let mut row = vec![
                if index == 0 {
                    format!("{account} (active)")
                } else {
                    account.clone()
                },
                grouped(usage.non_cached_input()),
            ];
            if cached {
                row.push(grouped(usage.cached_input()));
            }
            row.push(grouped(usage.output_tokens));
            if reasoning {
                row.push(grouped(usage.reasoning_output_tokens));
            }
            row
        })
        .collect();
    let widths: Vec<usize> = (0..header.len())
        .map(|column| {
            cells
                .iter()
                .map(|row| crate::width::display_width(&row[column]))
                .chain([header[column].len()])
                .max()
                .unwrap_or(0)
        })
        .collect();
    let render = |row: Vec<String>| -> String {
        row.into_iter()
            .enumerate()
            .map(|(column, cell)| {
                let pad = " ".repeat(widths[column] - crate::width::display_width(&cell));
                // The account column is left-aligned; counts are right-aligned.
                if column == 0 {
                    format!("{cell}{pad}")
                } else {
                    format!("{pad}{cell}")
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_string()
    };
    let mut lines = vec![vec![
        Span::from(render(header.into_iter().map(str::to_string).collect())).dim(),
    ]];
    lines.extend(cells.into_iter().map(|row| vec![Span::from(render(row))]));
    lines
}

/// `12345` as `12,345`, independent of the locale so `/status` and `/cost`
/// read the same.
fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    if n < 0 { format!("-{out}") } else { out }
}

#[cfg(test)]
#[path = "account_usage_tests.rs"]
mod tests;
