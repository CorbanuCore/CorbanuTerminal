//! Retain hidden reasoning and metadata between activity calls without breaking their compact group.
//!
//! Positions count preceding calls. Prepending history shifts those positions while sharing the
//! original immutable detail cells, so expanded output keeps its chronological source content.
//! Raw projection follows each cell's contract, retaining terminal input while hiding reasoning.

use super::HistoryCell;
use super::HistoryRenderMode;
use crate::terminal_hyperlinks::HyperlinkLine;
use crate::terminal_hyperlinks::plain_hyperlink_lines;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct ActivityDetail {
    after_calls: usize,
    cell: Arc<dyn HistoryCell>,
    offers_disclosure: bool,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ActivityDetails {
    entries: Vec<ActivityDetail>,
}

impl ActivityDetails {
    pub(crate) fn push(&mut self, after_calls: usize, cell: Arc<dyn HistoryCell>) {
        self.entries.push(ActivityDetail {
            after_calls,
            cell,
            offers_disclosure: true,
        });
    }

    /// Keep one job record per call, before any reasoning, without adding a disclosure control.
    pub(crate) fn set_metadata(&mut self, after_calls: usize, cell: Arc<dyn HistoryCell>) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.after_calls == after_calls && !entry.offers_disclosure)
        {
            entry.cell = cell;
        } else {
            self.entries.insert(
                /*index*/ 0,
                ActivityDetail {
                    after_calls,
                    cell,
                    offers_disclosure: false,
                },
            );
        }
    }

    pub(crate) fn has_disclosure_after(&self, after_calls: usize, width: u16) -> bool {
        self.entries.iter().any(|entry| {
            entry.after_calls == after_calls
                && entry.offers_disclosure
                && !entry.cell.transcript_hyperlink_lines(width).is_empty()
        })
    }

    pub(crate) fn prepend(&mut self, mut older: Self, older_calls: usize) {
        for entry in &mut self.entries {
            entry.after_calls += older_calls;
        }
        older.entries.append(&mut self.entries);
        *self = older;
    }

    pub(crate) fn lines_after(
        &self,
        after_calls: usize,
        width: u16,
        mode: HistoryRenderMode,
    ) -> Vec<HyperlinkLine> {
        self.entries
            .iter()
            .filter(|entry| entry.after_calls == after_calls)
            .flat_map(|entry| {
                let mut lines = match mode {
                    HistoryRenderMode::Rich => entry.cell.transcript_hyperlink_lines(width),
                    HistoryRenderMode::Raw => plain_hyperlink_lines(entry.cell.raw_lines()),
                };
                if mode == HistoryRenderMode::Rich && !lines.is_empty() {
                    lines.insert(/*index*/ 0, HyperlinkLine::from(""));
                }
                lines
            })
            .collect()
    }
}
