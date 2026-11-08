//! #1791 / D400 (needs security sign-off): each full admin setup (first
//! run, managed provisioning, and after an upgrade when Core's own rescan
//! finds a folder without its deny, D414) scans the shared folders and denies
//! the sandbox users' group deleting (or, where Everyone may write, writing)
//! in them; see `acl_shared_folders.rs` in the library. A folder that can't
//! be changed is logged; a record that can't be kept fails the setup, since
//! the removal relies on it.

use super::log_line;
use anyhow::Result;
use codex_windows_sandbox::LocalSid;
use codex_windows_sandbox::SharedFolderDenyRemovalSummary;
use codex_windows_sandbox::SharedFolderScanStamp;
use codex_windows_sandbox::apply_shared_folder_denies;
use codex_windows_sandbox::default_shared_folder_scan_roots;
use codex_windows_sandbox::ensure_machine_shared_folder_record_dir;
use codex_windows_sandbox::recorded_shared_folders_for_home;
use codex_windows_sandbox::remove_shared_folder_denies_for_home;
use codex_windows_sandbox::remove_shared_folder_scan_stamp;
use codex_windows_sandbox::scan_shared_folders;
use codex_windows_sandbox::shared_folder_scan_cli_version;
use codex_windows_sandbox::string_from_sid_bytes;
use codex_windows_sandbox::write_shared_folder_removal_summary;
use codex_windows_sandbox::write_shared_folder_scan_stamp;
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
    let record_dir = ensure_machine_shared_folder_record_dir(&group)?;
    let roots = default_shared_folder_scan_roots();
    let scan = match scan_shared_folders(&roots) {
        Ok(scan) => scan,
        Err(err) => {
            // D414: no stamp, so Core asks for this setup again (once per
            // process once one has succeeded).
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
    for path in &scan.no_dacl {
        log_line(
            log,
            &format!(
                "shared-folder scan: {} has no DACL (open to everyone); not changed",
                path.display()
            ),
        )?;
    }
    let started = Instant::now();
    // SAFETY: `group_sid` is the SID `group` names and outlives the call.
    let report = unsafe {
        apply_shared_folder_denies(
            &record_dir,
            codex_home,
            &group,
            group_sid.as_ptr(),
            &scan.findings,
        )
    }?;
    for note in &report.notes {
        log_line(log, &format!("shared-folder record: {note}"))?;
    }
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
            "shared-folder denies: added={} already={} foreign={} failed={} in {:.1}s",
            report.added.len(),
            report.already,
            report.foreign,
            report.errors.len(),
            started.elapsed().as_secs_f64()
        ),
    )?;
    // D414: what Core's rescans compare against. Without it Core asks for
    // this setup again (once per process), so it isn't fatal.
    let stamp =
        recorded_shared_folders_for_home(&record_dir, codex_home, &group).and_then(|recorded| {
            write_shared_folder_scan_stamp(
                codex_home,
                &SharedFolderScanStamp {
                    cli_version: shared_folder_scan_cli_version().to_string(),
                    recorded,
                    foreign: report.foreign_folders.clone(),
                    unchanged: report.unchanged.clone(),
                    unopened: report.unopened.clone(),
                    clean_versions: Vec::new(),
                },
            )
        });
    if let Err(err) = stamp {
        log_line(log, &format!("shared-folder scan stamp: {err:#}"))?;
    }
    Ok(())
}

pub(super) fn remove_shared_folder_denies(
    codex_home: &Path,
    sandbox_group_sid: &[u8],
    log: &mut dyn Write,
) -> Result<()> {
    let group = string_from_sid_bytes(sandbox_group_sid).map_err(anyhow::Error::msg)?;
    let group_sid = LocalSid::from_string(&group)?;
    let record_dir = ensure_machine_shared_folder_record_dir(&group)?;
    // SAFETY: `group_sid` is the SID `group` names and outlives the call.
    let removal = unsafe {
        remove_shared_folder_denies_for_home(&record_dir, codex_home, &group, group_sid.as_ptr())
    }?;
    // D414: the next elevated command runs the setup (and adds them) again.
    remove_shared_folder_scan_stamp(codex_home)?;
    write_shared_folder_removal_summary(
        codex_home,
        &SharedFolderDenyRemovalSummary {
            removed: removal.removed,
            kept: removal.kept,
            absent: removal.absent,
            failed: removal.errors.len(),
        },
    )?;
    for error in &removal.errors {
        log_line(log, &format!("shared-folder deny not removed: {error}"))?;
    }
    log_line(
        log,
        &format!(
            "shared-folder denies removed={} kept-for-other-homes={} absent={} failed={}",
            removal.removed,
            removal.kept,
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
