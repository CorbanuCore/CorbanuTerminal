//! Issue #536: the writable roots' real locations, pinned by the caller
//! when the session first saw them (`--pinned-writable-roots`). This helper
//! process runs one command, so they are set once.
//!
//! Each writable root is opened here at the location it must have (with
//! `O_PATH`) and the kernel's own name for the opened folder is checked. The
//! sandbox then grants writes on that open folder (bubblewrap `--bind-fd`,
//! a Landlock rule from the descriptor), never by path again, so a link
//! swapped in after the check grants nothing.

use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::OnceLock;

use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;
use codex_protocol::permissions::PinnedWritableRoot;
use codex_protocol::permissions::real_location;

static PINNED: OnceLock<Vec<PinnedWritableRoot>> = OnceLock::new();

pub(crate) fn set(pinned: Vec<PinnedWritableRoot>) {
    let _ = PINNED.set(pinned);
}

/// The pinned roots, when the caller pinned them (a session's command).
pub(crate) fn get() -> Option<&'static [PinnedWritableRoot]> {
    PINNED.get().map(Vec::as_slice)
}

/// Where the writable root `root` must be: the location the caller pinned,
/// or, for a root only this helper computes (a `:tmpdir` root follows the
/// command's `TMPDIR`), where the held-link check found it (`checked`).
pub(crate) fn expected_location(
    pinned: &[PinnedWritableRoot],
    checked: &[PinnedWritableRoot],
    root: &codex_utils_absolute_path::AbsolutePathBuf,
) -> std::path::PathBuf {
    pinned
        .iter()
        .chain(checked)
        .find(|pinned| &pinned.root == root)
        .map(|pinned| pinned.real.clone())
        .unwrap_or_else(|| real_location(root.as_path()))
}

/// The held-link check (#536) for the roots this helper computes.
pub(crate) fn check_held_links(
    policy: &codex_protocol::permissions::FileSystemSandboxPolicy,
    cwd: &Path,
) -> Result<Vec<PinnedWritableRoot>> {
    codex_sandboxing::writable_root_ledger::check_held_links(policy, cwd)
        .map_err(|refusal| CodexErr::UnsupportedOperation(refusal.to_string()))
}

/// Opens the folder at `real`, the location the writable root spelled
/// `spelled` must have, after checking that each spelling still leads there,
/// and checks that the kernel opened it there.
pub(crate) fn open_at(spelled: &[&Path], real: &Path) -> Result<File> {
    for path in spelled {
        let now = real_location(path);
        if now != real {
            return Err(refusal(path, &now, real));
        }
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH)
        .open(real)?;
    let opened = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    if opened != real {
        return Err(refusal(
            spelled.first().copied().unwrap_or(real),
            &opened,
            real,
        ));
    }
    Ok(file)
}

fn refusal(root: &Path, now: &Path, was: &Path) -> CodexErr {
    let link = codex_protocol::permissions::changed_link(root, was);
    CodexErr::UnsupportedOperation(format!(
        "Refused to run the command: the writable folder `{}` now leads to `{}` through the \
         link `{}`, made or changed after it was checked (it led to `{}`). Such a link cannot move write \
         access: remove it (for example `!rm '{}'`).",
        root.display(),
        now.display(),
        link.display(),
        was.display(),
        link.to_string_lossy().replace('\'', r"'\''"),
    ))
}

#[cfg(test)]
#[path = "pinned_tests.rs"]
mod tests;
