//! D283 (#1541): `.codex` and `.agents` folders below the top of a legacy
//! session's writable roots. Core loads them from every folder between a
//! project's root and its working directory, so a sandboxed command could
//! plant one deeper in a writable root, where D262 doesn't look.
//!
//! When a session starts, [`find`] lists the ones that exist; the session
//! protects them (and the folders above them) like the top-level ones and
//! records them with their file identity. When it ends, every one found then
//! that isn't in that list, unchanged, counts as planted ([`appeared`]), and
//! so does every folder the walk couldn't list. While it runs,
//! [`appeared_in_session`] tells Core the same about a single folder.
//!
//! #1896 item 3 (D425): an elevated session doesn't protect them, so it
//! also records what is inside each ([`with_contents`]); one whose contents
//! differ when the session ends counts as planted too.

use crate::acl::walk::Kind;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;

const PROJECT_METADATA_NAMES: [&str; 2] = [".codex", ".agents"];
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

/// A nested `.codex`/`.agents` entry and the file it was. `identity` is
/// `None` for a link (the sandbox may be able to point it elsewhere without
/// changing the link's identity) or when it can't be read; such an entry
/// never matches, so it counts as planted after every session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NestedMetadata {
    pub(crate) path: PathBuf,
    pub(crate) identity: Option<FileIdentity>,
    /// #1896 item 3: what was inside it ([`contents_of`]), for an elevated
    /// session (`None` otherwise: a legacy session keeps it unchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) contents: Option<crate::git_watch::Prints>,
}

impl NestedMetadata {
    /// Whether `path`, the file `identity` now, is still this entry: the
    /// same file, with the same contents when they were recorded (read
    /// within `budget`).
    fn unchanged(
        &self,
        path: &Path,
        identity: Option<FileIdentity>,
        budget: &ContentBudget,
    ) -> bool {
        same_path(&self.path, path)
            && self.identity == identity
            && self
                .contents
                .as_ref()
                .is_none_or(|before| *before == contents_of(path, identity, budget))
    }
}

/// Volume serial number and 128-bit file ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileIdentity {
    volume: u64,
    id: [u8; 16],
}

/// What a walk of the writable roots found.
#[derive(Default)]
pub(crate) struct Found {
    pub(crate) entries: Vec<NestedMetadata>,
    /// Folders the walk couldn't list (or entries it couldn't inspect): what
    /// lies below them is unknown.
    pub(crate) unlistable: Vec<PathBuf>,
    /// D420 (#1775): the repositories below the top of the roots: each
    /// nested `.git` (folder, file or link) and each folder that is itself a
    /// git folder (a bare repository), by git's rule.
    pub(crate) repos: Vec<PathBuf>,
    /// D420: folder links the walk found (it doesn't follow them).
    pub(crate) links: Vec<PathBuf>,
    /// D420: folders that count as changed after every session, and why: a
    /// folder holding an entry whose name isn't valid Unicode (it can't be
    /// recorded, so it isn't looked at; one note per folder), or a link to a
    /// folder too large to look through.
    pub(crate) notes: Vec<(PathBuf, String)>,
    /// The walk stopped at its budget.
    exhausted: bool,
}

impl Found {
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
            && self.unlistable.is_empty()
            && self.repos.is_empty()
            && self.links.is_empty()
            && self.notes.is_empty()
    }
}

/// The `.codex`/`.agents` entries below the top of `roots` (canonical), of
/// any kind (a link counts), recorded under the name Core opens. Links are
/// never followed; other reparse points, such as ProjFS folders or cloud
/// placeholders, are walked like plain folders (#1672). The walk doesn't descend into `.git` folders or into
/// the entries it reports: the sandbox can't write inside a protected one,
/// and Core doesn't run from inside `.git`. Names match the way NTFS
/// compares them (each character upper-cased), so `.agentſ` counts; such a
/// name never matches a snapshot.
///
/// #1877: the folders down to [`SPLIT_DEPTH`] are listed on several
/// threads, each deeper subtree by one thread as [`find_bounded`] walks it. Each part is put back in its place, so the result is the same,
/// in the same order, as one walk.
pub(crate) fn find(roots: &[PathBuf]) -> Found {
    // A folder's place: its root, then for each level its position counted
    // from the last folder listed (the walk takes the last one first).
    let parts = std::sync::Mutex::new(Vec::<(Vec<usize>, Found)>::new());
    let seed = roots
        .iter()
        .enumerate()
        .map(|(n, root)| (Pending::root(root), vec![n]))
        .collect();
    let walked = crate::acl::walk::par_walk(
        seed,
        crate::acl::walk::check_threads(),
        |(dir, place): (Pending, Vec<usize>), more| {
            let mut part = Found::default();
            let mut budget = usize::MAX;
            if place.len() <= SPLIT_DEPTH {
                let mut children = Vec::new();
                list(dir, place.len() == 1, &mut part, &mut budget, &mut children);
                let count = children.len();
                more.extend(children.into_iter().enumerate().map(|(n, child)| {
                    let mut place = place.clone();
                    place.push(count - 1 - n);
                    (child, place)
                }));
            } else {
                let mut stack = vec![dir];
                while let Some(dir) = stack.pop() {
                    list(dir, /*top*/ false, &mut part, &mut budget, &mut stack);
                }
            }
            if !part.is_empty() {
                parts
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((place, part));
            }
            Ok(())
        },
    );
    if walked.is_err() {
        // A panic: walk again on this thread, as before #1877.
        let mut budget = usize::MAX;
        return find_bounded(roots, &mut budget);
    }
    let mut parts = parts
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A folder's own part comes before those below it.
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    let mut found = Found::default();
    for (_, part) in parts {
        found.entries.extend(part.entries);
        found.unlistable.extend(part.unlistable);
        found.repos.extend(part.repos);
        found.links.extend(part.links);
        found.notes.extend(part.notes);
    }
    found
}

/// [`find`] lists each folder down to this depth (a root is 1) as its own
/// piece of work; each deeper subtree is walked whole by one thread.
const SPLIT_DEPTH: usize = 4;

/// [`find`], looking at no more than `budget` entries (it counts down).
fn find_bounded(roots: &[PathBuf], budget: &mut usize) -> Found {
    let mut found = Found::default();
    for root in roots {
        let mut stack = vec![Pending::root(root)];
        while let Some(dir) = stack.pop() {
            if *budget == 0 {
                found.exhausted = true;
                return found;
            }
            let top = dir.parent.is_none();
            list(dir, top, &mut found, budget, &mut stack);
            if found.exhausted {
                return found;
            }
        }
    }
    found
}

/// A folder a walk still has to list: its path, and the open folder it was
/// listed in (none for a root, which is opened by its path).
pub(crate) struct Pending {
    path: PathBuf,
    parent: Option<std::sync::Arc<crate::acl::walk::Handle>>,
    /// The file it must still be when opened, if one was recorded.
    expected: Option<FileIdentity>,
}

impl Pending {
    fn root(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            parent: None,
            expected: None,
        }
    }

    /// The folder `path` listed in the open folder `parent`.
    fn child(path: PathBuf, parent: &std::sync::Arc<crate::acl::walk::Handle>) -> Self {
        Self {
            path,
            parent: Some(std::sync::Arc::clone(parent)),
            expected: None,
        }
    }

    /// #1909: the folder open for listing. A root is opened by its path,
    /// following links as listing it by path did; any other folder through
    /// the folder it was found in, following no link: one swapped in since
    /// (a junction, say, perhaps to a network share) fails to open, and
    /// nothing behind it is reached.
    fn open(&self) -> std::io::Result<crate::acl::walk::Handle> {
        let handle = match &self.parent {
            None => open_by_path(&self.path, FOLDER_LIST),
            // SAFETY: the parent folder is open.
            Some(parent) => unsafe {
                crate::acl::walk::open_folder_unfollowed(
                    parent.0,
                    &wide_name(&self.path),
                    crate::acl::walk::FolderAccess::List,
                )
            },
        }?;
        if let Some(expected) = self.expected {
            // SAFETY: the handle is open.
            let (volume, id) = unsafe { handle.identity() }
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            if (FileIdentity { volume, id }) != expected {
                return Err(std::io::Error::other("not the folder recorded"));
            }
        }
        Ok(handle)
    }

    /// Whether the folder is gone (its parent no longer holds its name).
    fn is_gone(&self) -> bool {
        match &self.parent {
            None => std::fs::symlink_metadata(&self.path)
                .is_err_and(|err| err.kind() == std::io::ErrorKind::NotFound),
            // SAFETY: the parent folder is open.
            Some(parent) => unsafe {
                attributes_at(parent.0, &wide_name(&self.path))
                    .is_err_and(|err| err.kind() == std::io::ErrorKind::NotFound)
            },
        }
    }
}

/// [`open_by_path`] access to list a folder (`FILE_LIST_DIRECTORY`).
const FOLDER_LIST: u32 = 0x1;
/// [`open_by_path`] access to open what is inside (`FILE_TRAVERSE`).
const FOLDER_TRAVERSE: u32 = 0x20;

/// The folder `path` open for `access` ([`FOLDER_LIST`] or
/// [`FOLDER_TRAVERSE`]), following links as a path open does.
fn open_by_path(path: &Path, access: u32) -> std::io::Result<crate::acl::walk::Handle> {
    use std::os::windows::fs::OpenOptionsExt as _;
    use std::os::windows::io::IntoRawHandle as _;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let file = std::fs::OpenOptions::new()
        .access_mode(if access == FOLDER_LIST {
            access | FILE_READ_ATTRIBUTES | SYNCHRONIZE
        } else {
            access | SYNCHRONIZE
        })
        .share_mode(0x7)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    Ok(crate::acl::walk::Handle(file.into_raw_handle() as _))
}

/// #1909: the folder holding `path` (absolute), reached one folder at a time
/// from the top of its volume, following no link, and the last name of
/// `path`; `None` if any folder on the way can't be opened that way (also a
/// folder a volume is mounted on: such a path counts as changed every time).
fn open_parent_unfollowed(path: &Path) -> Option<(crate::acl::walk::Handle, Vec<u16>)> {
    use std::os::windows::ffi::OsStrExt as _;
    use std::path::Component;
    let mut top = PathBuf::new();
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir if names.is_empty() => {
                top.push(component.as_os_str());
            }
            Component::Normal(name) => names.push(name.encode_wide().collect::<Vec<u16>>()),
            _ => return None,
        }
    }
    let last = names.pop()?;
    let mut dir = open_by_path(&top, FOLDER_TRAVERSE).ok()?;
    for name in names {
        // SAFETY: `dir` is open.
        dir = unsafe {
            crate::acl::walk::open_folder_unfollowed(
                dir.0,
                &name,
                crate::acl::walk::FolderAccess::Traverse,
            )
        }
        .ok()?;
    }
    Some((dir, last))
}

/// The last component of `path`, as the name to open in its folder.
fn wide_name(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt as _;
    path.file_name()
        .map(|name| name.encode_wide().collect())
        .unwrap_or_default()
}

/// The attributes and reparse tag of the entry `name` of the open folder
/// `dir` itself (not followed).
unsafe fn attributes_at(
    dir: windows_sys::Win32::Foundation::HANDLE,
    name: &[u16],
) -> std::io::Result<crate::acl::walk::Attributes> {
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    crate::acl::walk::open_itself(dir, name, FILE_READ_ATTRIBUTES)?
        .kind()
        .map_err(|err| std::io::Error::other(err.to_string()))
}

/// The identity of the entry `name` of the open folder `dir` itself.
unsafe fn identity_at(
    dir: windows_sys::Win32::Foundation::HANDLE,
    name: &[u16],
) -> Option<FileIdentity> {
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    let handle = crate::acl::walk::open_itself(dir, name, FILE_READ_ATTRIBUTES).ok()?;
    let (volume, id) = handle.identity().ok()?;
    Some(FileIdentity { volume, id })
}

/// An entry of a listed folder: its name and its kind.
type Entry = (Vec<u16>, std::io::Result<crate::acl::walk::Attributes>);

/// The entries of `dir` (open for listing), no more than `at_most` of them
/// (#1877 item 4: the flag says it holds more, and isn't listed past
/// that); a reparse point's attributes and tag are asked of the entry
/// itself, since a listing that shows one may not carry its tag (#1672).
fn entries_of(
    dir: &crate::acl::walk::Handle,
    at_most: usize,
) -> anyhow::Result<(Vec<Entry>, bool)> {
    // SAFETY: the folder is open for listing.
    let (listed, more) = unsafe { crate::acl::walk::list_at_most(dir.0, at_most) }?;
    let entries = listed
        .into_iter()
        .map(|(name, attributes)| {
            let attributes = if attributes.attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                // SAFETY: the folder is open.
                unsafe { attributes_at(dir.0, &name) }
            } else {
                Ok(attributes)
            };
            (name, attributes)
        })
        .collect();
    Ok((entries, more))
}

/// Records what `dir` (`top`: one of the roots) holds in `found`, and adds
/// the folders to walk next to `stack`, in the order listed. #1877 item 4:
/// a folder holding more entries than are left in `budget` isn't listed
/// past that, and the walk is exhausted.
fn list(dir: Pending, top: bool, found: &mut Found, budget: &mut usize, stack: &mut Vec<Pending>) {
    #[cfg(test)]
    tests::before_list(&dir.path);
    let entries = match dir.open() {
        Ok(handle) => {
            entries_of(&handle, *budget).map(|entries| (std::sync::Arc::new(handle), entries))
        }
        // Gone. A folder that is still there but says "not found"
        // (a ProjFS or Cloud Files provider can) is unlistable.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && dir.is_gone() => return,
        Err(err) => Err(err.into()),
    };
    // A listing that fails partway makes the whole folder unlistable
    // (#1909: it is listed in full before anything in it is recorded).
    let Ok((handle, (entries, more))) = entries else {
        found.unlistable.push(dir.path);
        return;
    };
    if more {
        found.exhausted = true;
        return;
    }
    let dir = dir.path;
    // D420: a `HEAD` or `commondir` makes this folder a candidate
    // bare repository.
    let mut git_candidate = false;
    // Whether this folder already has its note, and its unlistable
    // record, for a name that isn't valid Unicode.
    let (mut noted, mut unlisted) = (false, false);
    for (wide, metadata) in entries {
        *budget = budget.saturating_sub(1);
        // D420: a name that isn't valid Unicode can't be recorded
        // (the records are JSON), so what is below it isn't walked
        // and its folder counts as changed every time.
        let Ok(name) = String::from_utf16(&wide) else {
            if !noted {
                noted = true;
                found.notes.push((
                    dir.clone(),
                    "holds an entry whose name isn't valid Unicode, so it isn't checked"
                        .to_string(),
                ));
            }
            // D283: nested `.codex`/`.agents` folders below it
            // aren't seen either, so its folder (which can be
            // recorded) counts as unlistable. A plain file hides
            // nothing.
            if !unlisted && !metadata.is_ok_and(|kind| kind.kind() == Kind::File) {
                unlisted = true;
                found.unlistable.push(dir.clone());
            }
            continue;
        };
        let path = dir.join(&name);
        git_candidate |=
            name.eq_ignore_ascii_case("HEAD") || name.eq_ignore_ascii_case("commondir");
        // D420: a nested `.git` of any kind (the top-level one is
        // D332's).
        // Names fold as NTFS compares them (`.gıt` opens as `.git`).
        if !top && (name.eq_ignore_ascii_case(".git") || folds_to(&name, ".git")) {
            found.repos.push(dir.join(".git"));
        }
        // Not followed if it is a link.
        if let Some((metadata_name, exact)) = metadata_name(&name) {
            // The top-level ones are D262's.
            if top && exact {
                continue;
            }
            if !top {
                let identity = (exact && !is_link(metadata.as_ref()))
                    // SAFETY: the folder is open.
                    .then(|| unsafe { identity_at(handle.0, &wide) })
                    .flatten();
                found.entries.push(NestedMetadata {
                    path: dir.join(metadata_name),
                    identity,
                    contents: None,
                });
                if exact {
                    continue;
                }
            }
        }
        // D420: a folder link isn't walked, but git follows it: a
        // repository behind one (moved to `%TEMP%`, say) counts.
        if !name.eq_ignore_ascii_case(".git")
            && !folds_to(&name, ".git")
            && is_link(metadata.as_ref())
        {
            let link = path.clone();
            if std::fs::metadata(link.join(".git")).is_ok() {
                found.repos.push(link.join(".git"));
            } else if crate::git_watch::is_git_directory(&link) {
                found.repos.push(link.clone());
            }
            if std::fs::metadata(&link).is_ok_and(|metadata| metadata.is_dir()) {
                found.links.push(link);
            }
        }
        let Ok(metadata) = metadata else {
            found.unlistable.push(path);
            continue;
        };
        if metadata.kind() == Kind::Folder
            // ASCII only: skipping a folded name (`.gıt`) would
            // leave an ordinary folder unwatched.
            && !name.eq_ignore_ascii_case(".git")
        {
            stack.push(Pending::child(path, &handle));
        }
    }
    if !top && git_candidate && crate::git_watch::is_git_directory(&dir) {
        found.repos.push(dir);
    }
}

/// Which of `.codex`/`.agents` `name` is, and whether it is that name up to
/// ASCII case.
fn metadata_name(name: &str) -> Option<(&'static str, bool)> {
    PROJECT_METADATA_NAMES.into_iter().find_map(|metadata| {
        if name.eq_ignore_ascii_case(metadata) {
            Some((metadata, true))
        } else if folds_to(name, metadata) {
            Some((metadata, false))
        } else {
            None
        }
    })
}

/// Whether `name` upper-cases, character by character as NTFS compares
/// names, to `target` (ASCII).
fn folds_to(name: &str, target: &str) -> bool {
    let mut folded = name.chars().map(|c| {
        let mut upper = c.to_uppercase();
        match (upper.next(), upper.next()) {
            (Some(single), None) => single,
            _ => c,
        }
    });
    target
        .chars()
        .all(|t| folded.next() == Some(t.to_ascii_uppercase()))
        && folded.next().is_none()
}

/// The entries of `found` the session protects like the top-level ones:
/// not links (their targets are the sandbox's choice), and identified (the
/// others count as planted anyway).
pub(crate) fn to_protect(found: &[NestedMetadata]) -> Vec<NestedMetadata> {
    found
        .iter()
        .filter(|entry| entry.identity.is_some())
        .cloned()
        .collect()
}

/// The entries of `now` that aren't in `before` as the same file.
pub(crate) fn appeared(before: &[NestedMetadata], now: Vec<NestedMetadata>) -> Vec<PathBuf> {
    let budget = ContentBudget::default();
    now.into_iter()
        .filter(|entry| {
            entry.identity.is_none()
                || !before
                    .iter()
                    .any(|old| old.unchanged(&entry.path, entry.identity, &budget))
        })
        .map(|entry| entry.path)
        .collect()
}

/// Whether `folder` (a `.codex`/`.agents` path, parent canonical) lies below
/// the top of one of `roots` and isn't in `before` as the file it is now
/// (one that doesn't exist counts: it may appear right after this check).
pub(crate) fn appeared_in_session(
    roots: &[PathBuf],
    before: &[NestedMetadata],
    folder: &Path,
) -> bool {
    let Some(parent) = folder.parent() else {
        return false;
    };
    let nested = roots
        .iter()
        .any(|root| !same_path(parent, root) && starts_with(parent, root));
    if !nested {
        return false;
    }
    let now = identity(folder);
    let budget = ContentBudget::default();
    now.is_none() || !before.iter().any(|old| old.unchanged(folder, now, &budget))
}

/// Entries [`contents_of`] looks at in one folder, all levels together; past
/// that it differs every time.
const MAX_CONTENT_ENTRIES: usize = 20_000;
/// Bytes of file content [`contents_of`] reads in one folder; past that it
/// differs every time.
const MAX_CONTENT_BYTES: u64 = 64 * 1024 * 1024;

/// #1909 item 4: entries [`contents_of`] looks at in all the folders of one
/// session's start or end together; past that, each folder still being read
/// differs every time.
const MAX_TOTAL_CONTENT_ENTRIES: usize = 100_000;
/// #1909 item 4: bytes of file content read in all the folders together;
/// past that, as for entries.
const MAX_TOTAL_CONTENT_BYTES: u64 = 256 * 1024 * 1024;

/// #1909 item 4: what [`contents_of`] may still look at across several folders
/// (read on several threads at once). Going over never stops a command: the
/// folders past it count as changed.
pub(crate) struct ContentBudget {
    entries: std::sync::atomic::AtomicUsize,
    bytes: std::sync::atomic::AtomicU64,
    max_entries: usize,
    max_bytes: u64,
}

impl Default for ContentBudget {
    fn default() -> Self {
        Self::with_limits(MAX_TOTAL_CONTENT_ENTRIES, MAX_TOTAL_CONTENT_BYTES)
    }
}

impl ContentBudget {
    fn with_limits(max_entries: usize, max_bytes: u64) -> Self {
        Self {
            entries: std::sync::atomic::AtomicUsize::new(0),
            bytes: std::sync::atomic::AtomicU64::new(0),
            max_entries,
            max_bytes,
        }
    }

    /// Counts one more entry; false once there have been too many.
    fn take_entry(&self) -> bool {
        self.entries
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(1)
            <= self.max_entries
    }

    /// Counts `bytes` more of file content; false once there have been
    /// too many.
    fn take_bytes(&self, bytes: u64) -> bool {
        self.bytes
            .fetch_add(bytes, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(bytes)
            <= self.max_bytes
    }
}

/// #1896 item 3 (D425): `entries` (from [`find`]) with what is inside each
/// identified one ([`contents_of`]), several at a time, all within one
/// [`ContentBudget`].
pub(crate) fn with_contents(entries: Vec<NestedMetadata>) -> Vec<NestedMetadata> {
    with_contents_within(entries, &ContentBudget::default())
}

fn with_contents_within(
    entries: Vec<NestedMetadata>,
    budget: &ContentBudget,
) -> Vec<NestedMetadata> {
    crate::par_map::par_map(&entries, |entry| NestedMetadata {
        contents: entry
            .identity
            .map(|_| contents_of(&entry.path, entry.identity, budget)),
        ..entry.clone()
    })
}

/// What is inside the `.codex`/`.agents` folder `folder` (as
/// [`contents_of`] the file it is now), within a budget of its own.
#[cfg(test)]
pub(crate) fn contents(folder: &Path) -> crate::git_watch::Prints {
    contents_of(folder, identity(folder), &ContentBudget::default())
}

/// [`contents_of`] the file `folder` is now, within `budget`.
#[cfg(test)]
fn contents_within(folder: &Path, budget: &ContentBudget) -> crate::git_watch::Prints {
    contents_of(folder, identity(folder), budget)
}

/// What is inside the `.codex`/`.agents` folder `folder`, the file
/// `expected`: each entry at any depth by its path from `folder`, a file by
/// its content ([`crate::git_watch::unfollowed_print_at`], which marks hard
/// links and reads only the file itself), a link by its target only (never
/// followed), a folder as `dir`. A `folder` that is a file is its content
/// (read the same way). A folder that can't be listed, a name that isn't
/// valid Unicode, more than [`MAX_CONTENT_ENTRIES`] entries or
/// [`MAX_CONTENT_BYTES`] of files, or going over `budget` (shared with the
/// other folders, #1909 item 4) make it differ every time.
///
/// #1909: nothing is looked up by path below the top of the volume:
/// `folder` is reached one folder at a time from there, following no link
/// ([`open_parent_unfollowed`]), and must still be the file `expected`;
/// each folder below it is opened through the folder it was listed in.
/// Otherwise (a folder on the way swapped for a link, say) it differs
/// every time.
fn contents_of(
    folder: &Path,
    expected: Option<FileIdentity>,
    budget: &ContentBudget,
) -> crate::git_watch::Prints {
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    let mut prints = crate::git_watch::Prints::new();
    let reached = open_parent_unfollowed(folder).and_then(|(parent, name)| {
        // SAFETY: the parent folder is open.
        let itself =
            unsafe { crate::acl::walk::open_itself(parent.0, &name, FILE_READ_ATTRIBUTES) }.ok()?;
        // SAFETY: the handle is open.
        let kind = unsafe { itself.kind() }.ok()?.kind();
        // SAFETY: the handle is open.
        let (volume, id) = unsafe { itself.identity() }.ok()?;
        (expected == Some(FileIdentity { volume, id })).then_some((parent, name, kind))
    });
    let Some((parent, name, kind)) = reached else {
        prints.insert(String::new(), crate::git_watch::truncated());
        return prints;
    };
    // A `.codex`/`.agents` that is a file: its content.
    if kind != Kind::Folder {
        let print = crate::git_watch::unfollowed_print_at(parent.0, &name, folder, |size| {
            size <= MAX_CONTENT_BYTES && budget.take_bytes(size)
        });
        prints.insert(String::new(), print);
        return prints;
    }
    let (mut seen, mut bytes) = (0_usize, 0_u64);
    // Checked again on the handle it is listed through.
    let top = Pending {
        path: folder.to_path_buf(),
        parent: Some(std::sync::Arc::new(parent)),
        expected,
    };
    let mut stack = vec![(top, String::new())];
    while let Some((dir, prefix)) = stack.pop() {
        #[cfg(test)]
        tests::before_list(&dir.path);
        let listed = dir.open().map_err(anyhow::Error::from).and_then(|handle| {
            entries_of(&handle, usize::MAX)
                .map(|(entries, _)| (std::sync::Arc::new(handle), entries))
        });
        let Ok((handle, entries)) = listed else {
            prints.insert(prefix, crate::git_watch::truncated());
            continue;
        };
        for (wide, metadata) in entries {
            seen += 1;
            if seen > MAX_CONTENT_ENTRIES || !budget.take_entry() {
                prints.insert(String::new(), crate::git_watch::truncated());
                return prints;
            }
            let Ok(name) = String::from_utf16(&wide) else {
                prints.insert(prefix.clone(), crate::git_watch::truncated());
                continue;
            };
            let path = dir.path.join(&name);
            let child = format!("{prefix}{name}");
            match metadata.map(crate::acl::walk::Attributes::kind) {
                Ok(Kind::Folder) => {
                    prints.insert(child.clone(), "dir".to_string());
                    stack.push((Pending::child(path, &handle), format!("{child}\\")));
                }
                // A link by its target, never read through: it may name a
                // network share (reading it would send the person's
                // credentials there) or a device.
                Ok(Kind::Link) => {
                    // SAFETY: the folder is open.
                    let target = unsafe { crate::acl::walk::link_target_at(handle.0, &wide) }
                        .unwrap_or_else(|| "?".to_string());
                    prints.insert(child, format!("link to {target}"));
                }
                // Counted at the size the open file has (the listing's may
                // be stale).
                Ok(Kind::File) => {
                    let print =
                        crate::git_watch::unfollowed_print_at(handle.0, &wide, &path, |size| {
                            bytes = bytes.saturating_add(size);
                            bytes <= MAX_CONTENT_BYTES && budget.take_bytes(size)
                        });
                    prints.insert(child, print);
                }
                Err(_) => {
                    prints.insert(child, crate::git_watch::truncated());
                }
            }
        }
    }
    prints
}

/// D332 (#1682): the top-level `.git` of each of `roots` (canonical) that
/// exists, of any kind, with the file it is. A link (Google's `repo` tool
/// makes one; a sandboxed command could point it elsewhere) or one that
/// can't be inspected has no identity, so it counts as changed after every
/// session. Git runs a `.git`'s hooks and config outside the sandbox, so one
/// that appears or changes while a session runs counts as planted, as new
/// `.codex`/`.agents` folders do; a missing one may still be made
/// (`git init` must work).
pub(crate) fn top_level_git(roots: &[PathBuf]) -> Vec<NestedMetadata> {
    roots
        .iter()
        .filter_map(|root| {
            let path = root.join(".git");
            let identity = match std::fs::symlink_metadata(&path) {
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
                Ok(metadata) if !metadata.file_type().is_symlink() => identity(&path),
                _ => None,
            };
            Some(NestedMetadata {
                path,
                identity,
                contents: None,
            })
        })
        .collect()
}

/// D332: whether `folder` is the top-level `.git` of one of `roots` and
/// isn't in `before` (from [`top_level_git`]) as the file it is now (one
/// that doesn't exist counts: it may appear right after this check).
pub(crate) fn git_appeared_in_session(
    roots: &[PathBuf],
    before: &[NestedMetadata],
    folder: &Path,
) -> bool {
    let top_level = folder
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case(".git"))
        && folder
            .parent()
            .is_some_and(|parent| roots.iter().any(|root| same_path(parent, root)));
    if !top_level {
        return false;
    }
    let now = identity(folder).filter(|_| !crate::workspace_acl::is_link(folder));
    now.is_none()
        || !before
            .iter()
            .any(|old| same_path(&old.path, folder) && old.identity == now)
}

/// D420 (#1775): the repositories deeper behind `links` (folder links a walk
/// found) whose target lies in one of `unwalked` (writable roots the walk
/// doesn't cover, such as `%TEMP%`): a command could build one there and
/// link it in. Each such link is walked by its own path, its links in turn,
/// up to 20,000 entries in all; past that the link is a note (it counts as
/// changed after every session). #1877 item 3: a link whose target lies in
/// one of `walked` (the roots the walk covers, which may lie inside an
/// unwalked one: a project in `%TEMP%`) isn't walked again.
pub(crate) fn repos_behind_links(
    links: &[PathBuf],
    unwalked: &[PathBuf],
    walked: &[PathBuf],
) -> (Vec<PathBuf>, Vec<(PathBuf, String)>) {
    let mut budget = 20_000_usize;
    let mut repos = Vec::new();
    let mut notes = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut queue: Vec<PathBuf> = links.to_vec();
    while let Some(link) = queue.pop() {
        let Ok(target) = dunce::canonicalize(&link) else {
            continue;
        };
        if !unwalked.iter().any(|root| starts_with(&target, root))
            || walked.iter().any(|root| starts_with(&target, root))
            || !visited.insert(target)
        {
            continue;
        }
        let found = find_bounded(std::slice::from_ref(&link), &mut budget);
        if found.exhausted {
            notes.push((
                link,
                "links to a folder too large to check for repositories".to_string(),
            ));
            break;
        }
        repos.extend(found.repos);
        notes.extend(found.notes);
        queue.extend(found.links);
    }
    (repos, notes)
}

/// D420 (#1775): a repository and what git would run from it, when a
/// session started or ended. `identity` is `None` for a link or an entry that
/// can't be inspected: such a repository counts as changed after every
/// session, as a top-level `.git` link does (D332).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RepoState {
    pub(crate) path: PathBuf,
    pub(crate) identity: Option<FileIdentity>,
    #[serde(default)]
    pub(crate) link: bool,
    pub(crate) prints: crate::git_watch::Prints,
}

/// D420/gap 2 (#1775): those of `roots` (canonical) that are themselves git
/// folders (bare repositories), which git uses where it would use
/// `<root>\.git`.
pub(crate) fn bare_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .filter(|root| crate::git_watch::is_git_directory(root))
        .cloned()
        .collect()
}

/// D420: the state of each repository in `paths`; folders a `gitdir:` line
/// or `commondir` names are looked into when they lie in `roots`. #1877:
/// several repositories at a time.
pub(crate) fn repo_states(paths: &[PathBuf], roots: &[PathBuf]) -> Vec<RepoState> {
    let folders = crate::git_watch::FolderPrints::default();
    crate::par_map::par_map(paths, |path| {
        let link =
            std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink());
        RepoState {
            path: path.clone(),
            identity: if link { None } else { identity(path) },
            link,
            prints: crate::git_watch::fingerprint_with(path, roots, &folders),
        }
    })
}

/// D420/D421 (#1775): what changed in each repository of `now` since
/// `before`, for the person. A replaced one counts, and so does a new one
/// unless it has exactly the shape `git init` or `git clone` leave (D421); a
/// link, or one that can't be inspected, counts every time.
pub(crate) fn repo_changes(
    before: &[RepoState],
    now: Vec<RepoState>,
) -> Vec<(PathBuf, Vec<String>)> {
    // #1877: `before` by path as [`same_path`] compares them, in order,
    // looked up rather than compared with each.
    let mut by_path: std::collections::HashMap<String, Vec<&RepoState>> =
        std::collections::HashMap::new();
    for old in before {
        by_path.entry(path_text(&old.path)).or_default().push(old);
    }
    now.into_iter()
        .filter_map(|state| {
            let changes = if state.link {
                vec!["a link (checked after every command)".to_string()]
            } else if state.identity.is_none() {
                vec!["can't be inspected".to_string()]
            } else {
                let same = by_path
                    .get(&path_text(&state.path))
                    .map_or(&[][..], Vec::as_slice);
                match same.iter().find(|old| old.identity == state.identity) {
                    Some(old) => crate::git_watch::changes(&old.prints, &state.prints),
                    // Only a new one may pass as plain (D421), not one that
                    // replaced a repository.
                    None if !same.is_empty() => vec!["replaced".to_string()],
                    None if crate::git_watch::is_plain(&state.path) => Vec::new(),
                    None => vec!["new repository".to_string()],
                }
            };
            (!changes.is_empty()).then_some((state.path, changes))
        })
        .collect()
}

/// The file `path` is (not followed if it is a link).
pub(crate) fn identity(path: &Path) -> Option<FileIdentity> {
    use std::os::windows::fs::OpenOptionsExt as _;
    use std::os::windows::io::AsRawHandle as _;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    let file = std::fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(0x7)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .ok()?;
    // SAFETY: the handle is open for the call.
    let (volume, id) = unsafe { crate::acl::file_identity(file.as_raw_handle() as _) }.ok()?;
    Some(FileIdentity { volume, id })
}

/// A link (a name-surrogate reparse point); unknown counts as one. Other
/// reparse points, such as ProjFS folders or cloud placeholders, are the
/// folder itself (#1672).
fn is_link(metadata: Result<&crate::acl::walk::Attributes, &std::io::Error>) -> bool {
    metadata.map_or(true, |metadata| metadata.kind() == Kind::Link)
}

/// `path` as [`same_path`] compares it: two paths are the same exactly when
/// these are equal.
fn path_text(path: &Path) -> String {
    path.as_os_str().to_string_lossy().to_ascii_lowercase()
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
}

/// `path` is `base` or below it, ignoring case.
pub(crate) fn starts_with(path: &Path, base: &Path) -> bool {
    let mut path = path.components();
    base.components().all(|part| {
        path.next().is_some_and(|other| {
            other
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&part.as_os_str().to_string_lossy())
        })
    })
}

#[cfg(test)]
#[path = "nested_metadata_tests.rs"]
mod tests;
