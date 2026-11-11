//! D52 (#1001, Travis 2026-10-20): MCP "always approve" under security
//! level Aggressive, like remembered command approvals (#626).
//!
//! An MCP tool whose approval mode resolves to `approve` (a person's "Allow
//! and don't ask me again", or a setting in a config file) runs unasked. While
//! Aggressive is in force that is suspended, not deleted: the tool is treated
//! as if it had no such setting (asked as usual) and the question says why,
//! unless a person chose "Allow and don't ask me again" for it while
//! Aggressive was in force. Those tools are recorded in the Corbanu home's
//! [`FILE`], the only place trusted to say so (under Aggressive the Corbanu
//! home is read-only to agent commands). Moderate and Permissive are
//! unchanged.

use std::path::Path;

use super::McpToolApprovalKey;

/// The Corbanu home's record of tools approved for good under Aggressive.
pub(super) const FILE: &str = "mcp-approvals-aggressive.json";

/// What an MCP approval question says when an "always approve" setting was
/// not applied.
pub(super) const SUSPENDED_NOTE: &str = "This tool is set to always be allowed, but that \
     setting was not applied because security level Aggressive is on: settings saved outside \
     Aggressive are suspended while it is active, so you are asked instead. They are kept and \
     apply again when you return to a less strict level.";

/// What an MCP approval question offering "always allow" says while
/// Aggressive is on, for a tool outside the record's scope: the answer is
/// saved (and applies at less strict levels) but not recorded, so under
/// Aggressive it lasts only while the session's approval does.
pub(super) const SESSION_ONLY_NOTE: &str = "Always allowing this tool saves that choice, but \
     under security level Aggressive it applies only until this session ends or the level \
     changes, because Aggressive keeps it only for connectors and for servers defined only in \
     your own config. At less strict levels the saved choice applies.";

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct Recorded {
    tools: Vec<McpToolApprovalKey>,
}

/// Serializes read-modify-write of [`FILE`] within this process. Two
/// processes on one home may still race; a lost update only means the person
/// is asked again under Aggressive.
static WRITE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The record, or an error when it exists but cannot be read.
async fn read(codex_home: &Path) -> std::io::Result<Recorded> {
    match tokio::fs::read(codex_home.join(FILE)).await {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(std::io::Error::other),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Recorded::default()),
        Err(err) => Err(err),
    }
}

/// Whether a person approved `key` for good while Aggressive was in force.
/// A missing or unreadable record says no.
pub(super) async fn approved(codex_home: &Path, key: &McpToolApprovalKey) -> bool {
    match read(codex_home).await {
        Ok(recorded) => recorded.tools.contains(key),
        Err(err) => {
            tracing::warn!("MCP approvals saved under Aggressive not read: {err}");
            false
        }
    }
}

/// Records that a person approved `key` for good while Aggressive was in
/// force. A record that exists but cannot be read is left alone (an error).
#[expect(
    clippy::await_holding_invalid_type,
    reason = "the record's read-modify-write must not interleave within this process"
)]
pub(super) async fn record(codex_home: &Path, key: McpToolApprovalKey) -> std::io::Result<()> {
    let _guard = WRITE_LOCK.lock().await;
    let mut recorded = read(codex_home).await?;
    if recorded.tools.contains(&key) {
        return Ok(());
    }
    recorded.tools.push(key);
    let bytes = serde_json::to_vec_pretty(&recorded).map_err(std::io::Error::other)?;
    let codex_home = codex_home.to_path_buf();
    tokio::task::spawn_blocking(move || {
        use std::io::Write;
        // A temporary file of its own in the same folder, synced, then
        // renamed into place: a reader sees the old record or the new one.
        let mut temporary = tempfile::NamedTempFile::new_in(&codex_home)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(codex_home.join(FILE))
            .map(drop)
            .map_err(|err| err.error)
    })
    .await
    .map_err(std::io::Error::other)?
}

#[cfg(test)]
#[path = "mcp_tool_call_aggressive_tests.rs"]
mod tests;
