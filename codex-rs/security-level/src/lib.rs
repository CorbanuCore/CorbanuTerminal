//! The human-chosen `/security` level stored in `$CODEX_HOME/security_level.toml`
//! (PF-24-S03), and detection of agents started by agent commands while
//! Aggressive is enforced (nested launches).
//!
//! Kept out of `codex-tui` so every Corbanu binary that can start an agent
//! (`corbanu`, and the standalone `codex-exec`, `codex-tui`, `codex-app-server`
//! and `codex-mcp-server`) runs the same nested-launch check.

pub mod level;
pub mod nested;
