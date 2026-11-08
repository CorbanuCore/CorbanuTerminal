//! D414 (needs security sign-off with #1791 / D400): the shared folders are
//! scanned again when the CLI's version changes.
//!
//! Each admin setup that scans writes `.sandbox/shared_folder_scan.json`
//! for its home: the version, the folders whose deny the machine's record
//! keeps for this home, and the folders it found but couldn't change. The
//! admin setup is the only writer of those lists. The file has the setup
//! marker's protection: a command's write-restricted token can't write
//! `.sandbox` (the sandbox account's unrestricted runner token can, the
//! #1721 route #1800 closes; with it a command could at most suppress or
//! request a rescan, not remove a deny).
//!
//! Before an elevated command Core reads it ([`rescan_decision`]):
//! - none (a setup made before the denies, or after their removal): the
//!   admin setup runs again, as a `SETUP_VERSION` bump would (so the denies
//!   need none; #1800 takes 7);
//! - another version: Core scans as the signed-in user and runs the admin
//!   setup only if a folder lacks a deny this home relies on (a vendor folder
//!   installed or reinstalled since); otherwise it notes the new version.
//!
//! Core does this at most once per process for each home, so a setup that
//! fails or is declined doesn't prompt before every command, and a decline
//! leaves the denies as they were (owner decision pending; see #1798).
//! Folders the signed-in user can't list are rescanned only by the admin
//! setup.

use crate::acl::shared_folders::SharedFolderDenyKind;
use crate::acl::shared_folders::SharedFolderFinding;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;

const STAMP_FILE: &str = "shared_folder_scan.json";

/// A folder whose deny the machine's record keeps for a home.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RecordedSharedFolder {
    pub volume: u64,
    /// The folder's file ID, in hex.
    pub file_id: String,
    pub kind: SharedFolderDenyKind,
}

/// What the last scan for a home found.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFolderScanStamp {
    pub cli_version: String,
    #[serde(default)]
    pub recorded: Vec<RecordedSharedFolder>,
    /// Found, but the setup couldn't change them: a rescan doesn't ask for
    /// the setup again for them.
    #[serde(default)]
    pub unchanged: Vec<PathBuf>,
}

/// The version shipped with these binaries (Core and setup alike).
pub fn shared_folder_scan_cli_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn stamp_path(codex_home: &Path) -> PathBuf {
    crate::setup::sandbox_dir(codex_home).join(STAMP_FILE)
}

/// The home's stamp; `None` if missing or damaged.
pub fn read_shared_folder_scan_stamp(codex_home: &Path) -> Option<SharedFolderScanStamp> {
    std::fs::read(stamp_path(codex_home))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

pub fn write_shared_folder_scan_stamp(
    codex_home: &Path,
    stamp: &SharedFolderScanStamp,
) -> Result<()> {
    let path = stamp_path(codex_home);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    std::fs::write(&path, serde_json::to_vec_pretty(stamp)?)
        .with_context(|| format!("writing {}", path.display()))
}

/// D413: after the removal, so the next elevated command runs the setup
/// (and adds the denies) again.
pub fn remove_shared_folder_scan_stamp(codex_home: &Path) -> Result<()> {
    let path = stamp_path(codex_home);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("removing {}", path.display())),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RescanDecision {
    /// Scanned for this version already.
    NotDue,
    /// The user's scan found nothing to close: this stamp, for this version.
    Clean(SharedFolderScanStamp),
    /// The admin setup must run, for this reason.
    AdminSetup(String),
}

/// See the module docs. `scan` returns the folders lacking a deny the stamp
/// says this home relies on; its error is returned (nothing is noted, so a
/// later process tries again).
pub(crate) fn rescan_decision(
    stamp: Option<SharedFolderScanStamp>,
    version: &str,
    scan: impl FnOnce(&SharedFolderScanStamp) -> Result<Vec<SharedFolderFinding>>,
) -> Result<RescanDecision> {
    let Some(stamp) = stamp else {
        return Ok(RescanDecision::AdminSetup(
            "no shared-folder scan is recorded for this home".to_string(),
        ));
    };
    if stamp.cli_version == version {
        return Ok(RescanDecision::NotDue);
    }
    let missing = scan(&stamp)?;
    Ok(match missing.first() {
        None => RescanDecision::Clean(SharedFolderScanStamp {
            cli_version: version.to_string(),
            ..stamp
        }),
        Some(first) => RescanDecision::AdminSetup(format!(
            "CLI version {version}: {} shared folders lack the setup's deny (first: {:?} on {})",
            missing.len(),
            first.kind,
            first.path.display()
        )),
    })
}

#[cfg(test)]
#[path = "shared_folder_rescan_tests.rs"]
mod tests;
