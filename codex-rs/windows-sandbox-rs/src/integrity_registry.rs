//! D598 (#1953): refuse records owned by an account the host does not trust.
//! An inherited deny does not constrain an owner's implicit WRITE_DAC right.

use crate::deny_read_sessions::current_user_sid;
use crate::deny_read_sessions::owner_sid;
use anyhow::Result;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::windows::fs::MetadataExt as _;
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;

#[derive(Debug)]
pub(crate) struct UntrustedRecords {
    path: PathBuf,
    reason: String,
}

impl std::fmt::Display for UntrustedRecords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the Windows sandbox refuses untrusted records at {}: {}. Stop all Corbanu sessions and sandbox commands, preserve these records for inspection, and follow the Windows sandbox records recovery instructions in docs/sandbox.md#untrusted-windows-sandbox-records; do not change their owner to make them trusted",
            self.path.display(),
            self.reason
        )
    }
}

impl std::error::Error for UntrustedRecords {}

pub(crate) fn refusal(path: &Path, reason: impl std::fmt::Display) -> anyhow::Error {
    UntrustedRecords {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
    .into()
}

/// Check by handle without following a reparse point. READ_CONTROL is not a
/// read-data request, so the registry's exclusive lock can still be inspected.
/// Administrators and SYSTEM already have authority over the host's records;
/// the sandbox accounts are neither and must never become trusted owners.
pub(crate) fn check_path(path: &Path) -> Result<()> {
    let file = open_path(path).map_err(|err| refusal(path, err))?;
    check_file(path, &file)
}

fn open_path(path: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .access_mode(0x0002_0000 | 0x80) // READ_CONTROL | FILE_READ_ATTRIBUTES
        .share_mode(0x1 | 0x2 | 0x4)
        .custom_flags(0x0200_0000 | 0x0020_0000) // BACKUP_SEMANTICS | OPEN_REPARSE_POINT
        .open(path)
}

fn check_file(path: &Path, file: &File) -> Result<()> {
    let user = current_user_sid().map_err(|err| refusal(path, err))?;
    let metadata = file.metadata().map_err(|err| refusal(path, err))?;
    if metadata.file_attributes() & 0x400 != 0 {
        return Err(refusal(path, "a records entry is a reparse point"));
    }
    let owner = owner_sid(file).map_err(|err| refusal(path, err))?;
    if owner != user && owner != "S-1-5-32-544" && owner != "S-1-5-18" {
        return Err(refusal(
            path,
            format!("owner {owner} is not the host user, Administrators or SYSTEM"),
        ));
    }
    Ok(())
}

/// Inspect the registry without creating or relabelling it, for readers that
/// do not pass through the launch-time ensure_registry path.
pub(crate) fn check_registry(registry: &Path) -> Result<()> {
    let parent = registry
        .parent()
        .ok_or_else(|| refusal(registry, "missing parent"))?;
    check_path(parent)?;
    check_path(registry)?;
    check_entries(registry)
}

/// Inspect every existing entry, including lock/session/temp files. Unknown
/// entries cannot be ignored: their owner could remove an inherited deny too.
pub(crate) fn check_entries(registry: &Path) -> Result<()> {
    for entry in std::fs::read_dir(registry).map_err(|err| refusal(registry, err))? {
        let entry = entry.map_err(|err| refusal(registry, err))?;
        check_directory_entry(entry)?;
    }
    Ok(())
}

fn check_directory_entry(entry: std::fs::DirEntry) -> Result<()> {
    let path = entry.path();
    match open_path(&path) {
        Ok(file) => check_file(&path, &file),
        // The entry itself vanished. Do not ignore a missing parent directory,
        // access denial, or any other inability to verify an existing entry.
        Err(err) if err.raw_os_error() == Some(ERROR_FILE_NOT_FOUND as i32) => Ok(()),
        Err(err) => Err(refusal(&path, err)),
    }
}

#[cfg(test)]
#[path = "integrity_registry_tests.rs"]
mod tests;
