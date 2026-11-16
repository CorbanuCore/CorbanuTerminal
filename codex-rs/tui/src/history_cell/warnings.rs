//! Retain warning details for the transcript while exposing stable identities to the footer.
//! Message identities deduplicate replay; MCP identities count affected servers, not summary rows.
//! Quota warnings remain visible in both transcript modes.

use super::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum WarningId {
    Message(String),
    McpServer(String),
}

// Warning notifications and startup config warnings carry text, not a category. Keep
// Corbanu's security/account notices visible without promoting ordinary diagnostics.
pub(super) fn is_security_or_account_warning(message: &str) -> bool {
    message.starts_with("Security level ")
        || message == crate::legacy_core::security_revocation::KILL_SWITCH_STOPPED
        || message.starts_with("`[provider_accounts]` is ignored because")
        || message.starts_with("Moderate needs ")
        || message.starts_with("No provider is allowed for spawned agents:")
        || message == codex_sandboxing::LEGACY_WINDOWS_SANDBOX_WARNING
        || message == codex_login::OPENAI_API_KEY_ENV_FALLBACK_NOTICE
        || message.contains(" credential was rejected. ")
        || (message.contains(") account `") && message.contains("` was rejected. "))
}

/// A retained diagnostic. Identity is independent of wrapping and duplicate delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WarningEntry {
    pub(crate) id: WarningId,
    pub(crate) source: String,
    pub(crate) details: String,
}

pub(crate) fn warning_entries(cells: &[Arc<dyn HistoryCell>]) -> Vec<WarningEntry> {
    let mut entries: Vec<WarningEntry> = Vec::new();
    let mut indices = BTreeMap::new();
    for entry in cells.iter().flat_map(|cell| cell.warning_entries()) {
        if let Some(&index) = indices.get(&entry.id) {
            let existing: &mut WarningEntry = &mut entries[index];
            if !existing.details.contains(&entry.details) {
                existing.details.push_str("\n\n");
                existing.details.push_str(&entry.details);
            }
        } else {
            indices.insert(entry.id.clone(), entries.len());
            entries.push(entry);
        }
    }
    entries
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum WarningKey<'a> {
    Message(&'a str),
    McpServer(&'a str),
}

#[derive(Debug)]
pub(crate) struct WarningHistoryCell {
    pub(crate) server_version_notice: bool,
    pub(super) visible_in_transcript: bool,
    pub(super) key: String,
    pub(super) diagnostic: String,
    pub(super) details: PrefixedWrappedHistoryCell,
}

impl HistoryCell for WarningHistoryCell {
    fn live_raw_lines(&self) -> Vec<Line<'static>> {
        if self.visible_in_transcript {
            self.details.raw_lines()
        } else {
            Vec::new()
        }
    }

    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        if self.visible_in_transcript {
            self.details.display_lines(width)
        } else {
            Vec::new()
        }
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.details.transcript_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.details.raw_lines()
    }

    fn warning_keys(&self) -> Vec<WarningKey<'_>> {
        vec![WarningKey::Message(&self.key)]
    }

    fn warning_entries(&self) -> Vec<WarningEntry> {
        vec![WarningEntry {
            id: WarningId::Message(self.key.clone()),
            source: "Warning".into(),
            details: self.diagnostic.clone(),
        }]
    }
}

pub(crate) fn warning_count(cells: &[Arc<dyn HistoryCell>]) -> usize {
    cells
        .iter()
        .flat_map(|cell| cell.warning_keys())
        .collect::<BTreeSet<_>>()
        .len()
}

#[cfg(test)]
#[path = "warnings_tests.rs"]
mod tests;
