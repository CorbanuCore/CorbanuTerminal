use std::fs;
#[cfg(not(windows))]
use std::fs::File;

use codex_utils_absolute_path::AbsolutePathBuf;

use crate::AuthorityIdentity;
use crate::DispatchResolution;
use crate::journal_types::JournalConfig;
use crate::journal_types::JournalError;
use crate::recovery::RecoveryBlocker;

pub(crate) const MAX_RECORD_BYTES: usize = 32 * 1024;

pub(crate) fn validate_resolution(
    authority: &AuthorityIdentity,
    resolution: &DispatchResolution,
) -> Result<(), JournalError> {
    match (authority, resolution) {
        (
            AuthorityIdentity::Mandate { mandate_id },
            DispatchResolution::Completed {
                outcome,
                mandate_receipt: Some(receipt),
            },
        ) if &receipt.mandate_id == mandate_id && receipt.outcome == *outcome => receipt
            .validate()
            .map_err(|_| JournalError::InvalidResolution),
        (
            AuthorityIdentity::Grant { .. },
            DispatchResolution::Completed {
                mandate_receipt: None,
                ..
            },
        )
        | (_, DispatchResolution::Unknown { .. }) => Ok(()),
        _ => Err(JournalError::InvalidResolution),
    }
}

pub(crate) fn ensure_directory(path: &std::path::Path) -> std::io::Result<()> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(std::io::Error::other(
                "audit directory is not a real directory",
            ));
        }
    } else {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn sync_directory(path: &AbsolutePathBuf) -> Result<(), JournalError> {
    File::open(path.as_path())
        .and_then(|directory| directory.sync_all())
        .map_err(|_| JournalError::StorageUnavailable)
}

// Windows does not support opening a directory with std::fs::File. The PF-20
// protected root remains authoritative: loss of an unsynced local entry is
// detected as truncation against that root and fails closed on recovery.
#[cfg(windows)]
pub(crate) fn sync_directory(_path: &AbsolutePathBuf) -> Result<(), JournalError> {
    Ok(())
}

pub(crate) fn segment_number_for_path(
    path: &std::path::Path,
    config: JournalConfig,
) -> Result<u64, RecoveryBlocker> {
    let parent = path.parent().ok_or(RecoveryBlocker::InvalidRecord)?;
    let parsed = parent
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(parse_segment_name)
        .ok_or(RecoveryBlocker::InvalidRecord)?;
    let sequence = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(parse_record_name)
        .ok_or(RecoveryBlocker::InvalidRecord)?;
    let expected = (sequence - 1)
        / u64::try_from(config.records_per_segment).map_err(|_| RecoveryBlocker::InvalidRecord)?
        + 1;
    if parsed != expected {
        return Err(RecoveryBlocker::InvalidRecord);
    }
    Ok(parsed)
}

pub(crate) fn parse_segment_name(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("segment-")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let number = digits.parse::<u64>().ok()?;
    (number != 0).then_some(number)
}

pub(crate) fn parse_temp_name(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("record-")?.strip_suffix(".json.tmp")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let sequence = digits.parse::<u64>().ok()?;
    (sequence != 0).then_some(sequence)
}

pub(crate) fn parse_record_name(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("record-")?.strip_suffix(".json")?;
    if digits.len() != 20 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let sequence = digits.parse::<u64>().ok()?;
    (sequence != 0).then_some(sequence)
}

pub(crate) struct WriterLock {
    pub(crate) _lock: fslock::LockFile,
}
