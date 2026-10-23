//! Issue #536 (Travis, 2026-10-12): a writable root must not follow a link
//! made after the session started, nor a link that leads outside the
//! writable roots holding it, at every level (Permissive included).
//!
//! The escape: a writable root that is configured but does not exist yet,
//! inside another writable root (`ws/target`), is created by a command as a
//! link (`ln -s /elsewhere ws/target`); from the next command on, write
//! access followed the link.
//!
//! Each writable root's real location (every link followed, see
//! [`real_location`]) is recorded the first time this process sees it,
//! normally when the session starts ([`record_writable_roots`]). Before each
//! sandboxed command ([`check_writable_roots`]) the command is refused, with
//! a message naming the link, when a root:
//! - leads through a link somewhere else than where it led when recorded;
//! - is held by another writable root and leads outside every root holding
//!   it (also a link left from an earlier session).
//!
//! A root first seen after the session started (added by a permission
//! request, or another working folder) is recorded then; it gets the second
//! check and the pinning, but no comparison with the session's start.
//!
//! The sandboxes then grant writes at the location checked here (the
//! [`PinnedWritableRoot`]s, and [`pinned_real_location`] for Seatbelt), not
//! wherever the root leads by the time they are set up, so a link swapped in
//! after the check grants nothing.
//!
//! A root that leads through no link any more (the person removed it, or a
//! real folder took its place) is recorded where it is and runs again. A
//! root's record lives as long as the process.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::MutexGuard;

use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::PinnedWritableRoot;
use codex_protocol::protocol::WritableRoot;

/// Each root's real location when first seen (or when it last led through
/// no link).
static LEDGER: LazyLock<Mutex<HashMap<PathBuf, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn ledger() -> MutexGuard<'static, HashMap<PathBuf, PathBuf>> {
    LEDGER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// #603: folders checked to be real at a location ([`record_checked_root`]).
/// A writable root below one, such as a grant at `worktrees/<name>`, is
/// refused while the folder leads elsewhere through a link.
static CHECKED: LazyLock<Mutex<HashMap<PathBuf, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn checked() -> MutexGuard<'static, HashMap<PathBuf, PathBuf>> {
    CHECKED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Why a command was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WritableRootRefusal {
    /// The root leads, through `link`, somewhere else than where it led when
    /// the session first saw it (`was`).
    Changed {
        root: PathBuf,
        link: PathBuf,
        was: PathBuf,
        now: PathBuf,
    },
    /// The root, held by another writable root, leads through `link`
    /// outside every root holding it.
    OutsideHolder {
        root: PathBuf,
        link: PathBuf,
        real: PathBuf,
        holder: PathBuf,
    },
}

impl fmt::Display for WritableRootRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Changed {
                root,
                link,
                was,
                now,
            } => write!(
                f,
                "Refused to run the command: the writable folder `{}` now leads to `{}` through \
                 the link `{}`, made or changed after this folder was first checked (it led to `{}`). Such a link \
                 cannot move write access, so commands are refused until it is removed: remove \
                 it yourself (for example `!rm {}`), or restart Corbanu if you made it on \
                 purpose.",
                root.display(),
                now.display(),
                link.display(),
                was.display(),
                shell_quoted(link),
            ),
            Self::OutsideHolder {
                root,
                link,
                real,
                holder,
            } => write!(
                f,
                "Refused to run the command: the writable folder `{}` leads through the link `{}` \
                 to `{}`, outside `{}`, the writable folder that holds it. A link cannot extend \
                 write access, so commands are refused until it is removed: remove it yourself \
                 (for example `!rm {}`), or configure the real folder instead.",
                root.display(),
                link.display(),
                real.display(),
                holder.display(),
                shell_quoted(link),
            ),
        }
    }
}

impl std::error::Error for WritableRootRefusal {}

/// `path` quoted for a shell command line.
fn shell_quoted(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', r"'\''"))
}

/// Records the real location of each writable root of `policy` this process
/// has not seen yet. Called when a session starts.
pub fn record_writable_roots(policy: &FileSystemSandboxPolicy, cwd: &Path) {
    let roots = policy.get_writable_roots_with_cwd(cwd);
    let mut ledger = ledger();
    for root in roots {
        let real = real_path(root.root.as_path());
        ledger.entry(root.root.into_path_buf()).or_insert(real);
    }
}

/// #603: records `real` as the location of the folder spelled `root`, which
/// the caller has just checked to be a real folder there (the Corbanu home's
/// `worktrees` carve-out). A link swapped in before the sandbox is set up is
/// then a change ([`check_writable_roots`] refuses the command), not the
/// root's first sight; so is one in place of the folder above a writable
/// root below it (a grant at `worktrees/<name>`).
pub fn record_checked_root(root: &Path, real: &Path) {
    ledger().insert(root.to_path_buf(), real.to_path_buf());
    checked().insert(root.to_path_buf(), real.to_path_buf());
}

/// Checks every writable root of `policy` before a sandboxed command (a root
/// seen for the first time is recorded now) and returns them with the real
/// locations the sandbox must grant.
pub fn check_writable_roots(
    policy: &FileSystemSandboxPolicy,
    cwd: &Path,
) -> Result<Vec<PinnedWritableRoot>, WritableRootRefusal> {
    let roots = policy.get_writable_roots_with_cwd(cwd);
    let reals: Vec<PathBuf> = roots
        .iter()
        .map(|root| real_path(root.root.as_path()))
        .collect();
    let checked = checked().clone();
    let mut ledger = ledger();
    let mut pinned = Vec::with_capacity(roots.len());
    for (index, root) in roots.iter().enumerate() {
        // #603: a root below a checked folder that now leads elsewhere.
        for (folder, was) in &checked {
            if root.root.as_path() == folder.as_path() || !root.root.as_path().starts_with(folder) {
                continue;
            }
            let now = real_path(folder);
            if now != *was && !links_on(folder).is_empty() {
                return Err(WritableRootRefusal::Changed {
                    root: folder.clone(),
                    link: link_to_name(&roots, folder, was),
                    was: was.clone(),
                    now,
                });
            }
        }
        let now = &reals[index];
        let was = ledger
            .entry(root.root.to_path_buf())
            .or_insert_with(|| now.clone());
        if *was != *now {
            // A root that leads through no link any more is where it is
            // spelled: record it there, so a link made later is refused.
            if !links_on(root.root.as_path()).is_empty() {
                return Err(WritableRootRefusal::Changed {
                    root: root.root.to_path_buf(),
                    link: link_to_name(&roots, root.root.as_path(), was),
                    was: was.clone(),
                    now: now.clone(),
                });
            }
            *was = now.clone();
        }
        if let Some(refusal) = outside_holders(&roots, &reals, index) {
            return Err(refusal);
        }
        pinned.push(PinnedWritableRoot {
            root: root.root.clone(),
            real: now.clone(),
        });
    }
    Ok(pinned)
}

/// The check of [`check_writable_roots`] that needs no record: no writable
/// root held by another leads outside every root holding it. The Linux
/// helper applies it to the roots it computes itself (a `:tmpdir` root
/// follows the command's `TMPDIR`, which the caller may not share).
pub fn check_held_links(
    policy: &FileSystemSandboxPolicy,
    cwd: &Path,
) -> Result<Vec<PinnedWritableRoot>, WritableRootRefusal> {
    let roots = policy.get_writable_roots_with_cwd(cwd);
    let reals: Vec<PathBuf> = roots
        .iter()
        .map(|root| real_path(root.root.as_path()))
        .collect();
    if let Some(refusal) = (0..roots.len()).find_map(|index| outside_holders(&roots, &reals, index))
    {
        return Err(refusal);
    }
    Ok(roots
        .into_iter()
        .zip(reals)
        .map(|(root, real)| PinnedWritableRoot {
            root: root.root,
            real,
        })
        .collect())
}

/// The refusal for `roots[index]` when it is held by other roots and leads
/// outside all of them (`reals` are the roots' real locations).
fn outside_holders(
    roots: &[WritableRoot],
    reals: &[PathBuf],
    index: usize,
) -> Option<WritableRootRefusal> {
    let root = &roots[index];
    let holders: Vec<usize> = roots
        .iter()
        .enumerate()
        .filter(|(other, holder)| *other != index && holds(holder, &reals[*other], root))
        .map(|(other, _)| other)
        .collect();
    let first = holders.first()?;
    if holders
        .iter()
        .any(|holder| within(&reals[index], &reals[*holder]))
    {
        return None;
    }
    // Leading outside its holder, the root goes through a link on its path
    // (or, failing that, is spelled through one above the holder).
    let link = links_on(root.root.as_path())
        .into_iter()
        .find(|link| link.starts_with(roots[*first].root.as_path()))
        .or_else(|| links_on(root.root.as_path()).into_iter().next())
        .unwrap_or_else(|| root.root.to_path_buf());
    Some(WritableRootRefusal::OutsideHolder {
        root: root.root.to_path_buf(),
        link,
        real: reals[index].clone(),
        holder: roots[*first].root.to_path_buf(),
    })
}

/// The link to name in a refusal for `root`, which led to `was`: the first
/// link on its path whose folder is inside a writable root (only those can
/// have been made by a command), else the one [`changed_link`] finds.
fn link_to_name(roots: &[WritableRoot], root: &Path, was: &Path) -> PathBuf {
    links_on(root)
        .into_iter()
        .find(|link| {
            link.parent().is_some_and(|folder| {
                roots
                    .iter()
                    .any(|writable| folder.starts_with(writable.root.as_path()))
            })
        })
        .unwrap_or_else(|| changed_link(root, was))
}

/// The real location recorded for `root`, if this process has seen it.
pub fn pinned_real_location(root: &Path) -> Option<PathBuf> {
    ledger().get(root).cloned()
}

/// Whether `holder` (whose real location is `holder_real`) holds `root`:
/// `root` is spelled inside it, or its folder really is inside it.
fn holds(holder: &WritableRoot, holder_real: &Path, root: &WritableRoot) -> bool {
    root.root != holder.root
        && (root.root.as_path().starts_with(holder.root.as_path())
            || root
                .root
                .as_path()
                .parent()
                .is_some_and(|parent| within(&real_path(parent), holder_real)))
}

#[cfg(unix)]
fn changed_link(path: &Path, was: &Path) -> PathBuf {
    codex_protocol::permissions::changed_link(path, was)
}

#[cfg(unix)]
fn links_on(path: &Path) -> Vec<PathBuf> {
    codex_protocol::permissions::nested_links(path)
}

#[cfg(unix)]
fn real_path(path: &Path) -> PathBuf {
    codex_protocol::permissions::real_location(path)
}

#[cfg(unix)]
fn within(path: &Path, prefix: &Path) -> bool {
    codex_protocol::permissions::location_within(path, prefix)
}

#[cfg(not(unix))]
fn changed_link(path: &Path, _was: &Path) -> PathBuf {
    path.to_path_buf()
}

#[cfg(not(unix))]
fn links_on(_path: &Path) -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(unix))]
fn real_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

#[cfg(not(unix))]
fn within(path: &Path, prefix: &Path) -> bool {
    path.starts_with(prefix)
}

// The tests exercise links and real paths, which are unix-only here. A
// separate `cfg(test)` keeps clippy treating the module as test code.
#[cfg(test)]
#[cfg(unix)]
#[path = "writable_root_ledger_tests.rs"]
mod tests;
