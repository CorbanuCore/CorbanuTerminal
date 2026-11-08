//! D414 (needs security sign-off with #1791 / D400): the first elevated
//! command after the CLI's version changes scans the shared folders again,
//! as the signed-in user, and re-runs the admin setup only if a folder the
//! scan finds lacks the setup's deny (a vendor folder installed or
//! reinstalled since). So an upgrade prompts for administrator approval only
//! when there is something to close. It also brings the denies to setups
//! made before them, so they need no new `SETUP_VERSION` (#1800 takes 7).
//!
//! The version the last scan ran for is kept in `.sandbox`, which sandboxed
//! commands can't write (their restricted token holds no write grant there).

use crate::acl::shared_folders::SharedFolderFinding;
use crate::acl::shared_folders::default_shared_folder_scan_roots;
use crate::acl::shared_folders::shared_folders_missing_denies;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;

const STAMP_FILE: &str = "shared_folder_scan.json";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ScanStamp {
    cli_version: String,
}

/// The version shipped with these binaries (Core, setup and runner alike).
pub(crate) fn cli_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn stamp_path(codex_home: &Path) -> PathBuf {
    crate::setup::sandbox_dir(codex_home).join(STAMP_FILE)
}

/// Whether no scan has run for [`cli_version`] in this home.
pub(crate) fn rescan_due(codex_home: &Path) -> bool {
    std::fs::read(stamp_path(codex_home))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<ScanStamp>(&bytes).ok())
        .is_none_or(|stamp| stamp.cli_version != cli_version())
}

/// Notes that the shared folders were scanned (and closed) for
/// [`cli_version`].
pub(crate) fn record_rescan(codex_home: &Path) -> Result<()> {
    let path = stamp_path(codex_home);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let stamp = ScanStamp {
        cli_version: cli_version().to_string(),
    };
    std::fs::write(&path, serde_json::to_vec(&stamp)?)
        .with_context(|| format!("writing {}", path.display()))
}

/// The folders the admin setup must close; `None` if no rescan is due.
pub(crate) fn folders_needing_setup(codex_home: &Path) -> Result<Option<Vec<SharedFolderFinding>>> {
    if !rescan_due(codex_home) {
        return Ok(None);
    }
    shared_folders_missing_denies(&default_shared_folder_scan_roots()).map(Some)
}

#[cfg(test)]
#[path = "shared_folder_rescan_tests.rs"]
mod tests;
