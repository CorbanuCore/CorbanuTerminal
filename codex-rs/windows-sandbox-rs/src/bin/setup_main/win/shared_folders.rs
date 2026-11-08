//! #1791 / D400 (needs security sign-off): each full admin setup (first
//! run, upgrade, managed provisioning) scans the shared folders and denies
//! the sandbox users' group deleting (or, where Everyone may write, writing)
//! in them; see `acl_shared_folders.rs` in the library. A folder that can't
//! be changed is logged; a record that can't be kept fails the setup, since
//! the removal relies on it.

use super::log_line;
use anyhow::Result;
use codex_windows_sandbox::LocalSid;
use codex_windows_sandbox::apply_shared_folder_denies;
use codex_windows_sandbox::default_shared_folder_scan_roots;
use codex_windows_sandbox::remove_recorded_shared_folder_denies;
use codex_windows_sandbox::scan_shared_folders;
use codex_windows_sandbox::string_from_sid_bytes;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

pub(super) fn deny_shared_folders(
    codex_home: &Path,
    sandbox_group_sid: &[u8],
    log: &mut dyn Write,
) -> Result<()> {
    let group = string_from_sid_bytes(sandbox_group_sid).map_err(anyhow::Error::msg)?;
    let group_sid = LocalSid::from_string(&group)?;
    let roots = default_shared_folder_scan_roots();
    let scan = match scan_shared_folders(&roots) {
        Ok(scan) => scan,
        Err(err) => {
            log_line(log, &format!("shared-folder scan failed: {err:#}"))?;
            return Ok(());
        }
    };
    log_line(
        log,
        &format!(
            "shared-folder scan: {} recursive roots, {} single folders, visited={} unreadable={} findings={} in {:.1}s",
            roots.recursive.len(),
            roots.single.len(),
            scan.visited,
            scan.unreadable,
            scan.findings.len(),
            scan.seconds
        ),
    )?;
    let started = Instant::now();
    // SAFETY: `group_sid` is the SID `group` names and outlives the call.
    let report = unsafe {
        apply_shared_folder_denies(codex_home, &group, group_sid.as_ptr(), &scan.findings)
    }?;
    for finding in &report.added {
        log_line(
            log,
            &format!(
                "shared-folder deny added: {:?} on {}",
                finding.kind,
                finding.path.display()
            ),
        )?;
    }
    for error in &report.errors {
        log_line(log, &format!("shared-folder deny failed: {error}"))?;
    }
    log_line(
        log,
        &format!(
            "shared-folder denies: added={} already={} failed={} in {:.1}s",
            report.added.len(),
            report.already,
            report.errors.len(),
            started.elapsed().as_secs_f64()
        ),
    )
}

pub(super) fn remove_shared_folder_denies(codex_home: &Path, log: &mut dyn Write) -> Result<()> {
    let removal = remove_recorded_shared_folder_denies(codex_home)?;
    for error in &removal.errors {
        log_line(log, &format!("shared-folder deny not removed: {error}"))?;
    }
    log_line(
        log,
        &format!(
            "shared-folder denies removed={} absent={} failed={}",
            removal.removed,
            removal.absent,
            removal.errors.len()
        ),
    )?;
    if !removal.errors.is_empty() {
        anyhow::bail!(
            "{} shared-folder denies could not be removed; they stay recorded",
            removal.errors.len()
        );
    }
    Ok(())
}
