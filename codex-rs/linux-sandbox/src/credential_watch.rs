//! #442 (Travis, 2026-10-11): credential files created after a session
//! starts must stay unreadable too.
//!
//! Bubblewrap can only hide paths that exist when a command starts, and
//! neither seccomp nor Landlock can match a file name. So under a policy that
//! denies the credential names (`CREDENTIAL_FILE_NAME_PATTERNS`, templates aside) below a
//! folder:
//! - every command hides the matches that exist when it starts (the policy's
//!   glob expansion, plus every letter case);
//! - a credential name that is not a plain file (a link, a pipe or device, a
//!   file with other hard links) refuses the command: whatever the person
//!   writes to it could land somewhere the command can read;
//! - this watcher covers the rest: it watches every folder below that folder
//!   (inotify) from before that expansion until the command ends, and a
//!   non-folder with such a name created, linked or moved in meanwhile stops
//!   the command at once (SIGKILL). Later commands hide the file.
//! - Fail closed: a folder that cannot be watched, or one on a filesystem
//!   whose changes from elsewhere inotify does not see (network, FUSE, 9p),
//!   refuses the command; after lost events everything is scanned again.
//!
//! Limits: the stop follows the create event, so a process already waiting
//! for the file could read it in that moment; a folder reached through a
//! link below the watched folder is not watched (the glob expansion does not
//! follow links either); the file itself is left in place (its creator is
//! unknown, so it may be the person's own).

use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::CString;
use std::ffi::OsStr;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;

use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::credential_file_glob_root;
use codex_protocol::permissions::credential_file_names_shown;
use codex_protocol::permissions::credential_rule_roots;
use codex_protocol::permissions::is_credential_file_name;
use codex_protocol::permissions::is_credential_template_name;
use codex_protocol::permissions::unsafe_credential_entry_message;

/// Folders watched for one command, at most. Past it the command is refused.
const MAX_WATCHED_FOLDERS: usize = 100_000;

const WATCH_MASK: u32 =
    libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_ONLYDIR | libc::IN_DONT_FOLLOW;

/// Filesystems where a change made elsewhere (the server, the host, another
/// client) raises no inotify event here. Kernel magic numbers are 32-bit.
const UNWATCHABLE_FILESYSTEMS: &[(u32, &str)] = &[
    (0x6969, "NFS"),
    (0x517B, "SMB"),
    (0xFF53_4D42, "CIFS"),
    (0xFE53_4D42, "SMB2"),
    (0x0102_1997, "9p"),
    (0x6573_5546, "FUSE"),
    (0x564C, "NCP"),
    (0x00C3_6400, "Ceph"),
    (0x0BD0_0BD0, "Lustre"),
    (0x5346_414F, "AFS"),
    (0x6B41_4653, "AFS"),
    (0x7375_7245, "Coda"),
    (0x7461_636F, "OCFS2"),
    (0x0116_1970, "GFS2"),
    (0x786F_4256, "vboxsf"),
];

/// The folders whose credential names the policy denies, existing, by their
/// real path, outermost only. The filesystem root is left out: its glob is
/// not expanded either.
pub(crate) fn credential_watch_roots(policy: &FileSystemSandboxPolicy, cwd: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = policy
        .get_unreadable_globs_with_cwd(cwd)
        .iter()
        .filter_map(|pattern| credential_file_glob_root(pattern))
        .filter(|root| root != Path::new("/"))
        .filter_map(|root| std::fs::canonicalize(root).ok())
        .filter(|root| root.is_dir())
        .collect();
    roots.sort();
    roots.dedup();
    roots
        .iter()
        .filter(|root| {
            !roots
                .iter()
                .any(|other| other != *root && root.starts_with(other))
        })
        .cloned()
        .collect()
}

/// How [`CredentialWatcher::watch_tree`] treats what it finds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Scan {
    /// Before the command: existing credential files are hidden, unsafe ones
    /// refuse it.
    Start,
    /// While it runs: any credential file not seen at the start stops it.
    Running,
}

/// Watches set up before the command's view is built.
pub(crate) struct CredentialWatcher {
    fd: OwnedFd,
    roots: Vec<PathBuf>,
    /// Below these the whole credential rule applies, so the templates are
    /// not credential files (#442, Travis 2026-10-12); below a folder only a
    /// person's own `**/.env.*` covers, they are.
    template_roots: Vec<PathBuf>,
    folders: HashMap<libc::c_int, PathBuf>,
    /// Credential files present at the start (hidden by the glob
    /// expansion), with their inode.
    known: HashMap<PathBuf, u64>,
    /// Mount points bubblewrap itself creates (empty placeholders).
    placeholders: HashSet<PathBuf>,
    /// Existing credential files, in any letter case, to hide.
    existing: Vec<PathBuf>,
}

/// The watcher running beside one command.
pub(crate) struct CredentialMonitor {
    stop: Arc<AtomicBool>,
    found: Arc<Mutex<Option<String>>>,
    handle: thread::JoinHandle<()>,
}

impl CredentialWatcher {
    /// `None` when the policy denies no credential names below a folder;
    /// `Err` (the command must not run) explains why it cannot be protected
    /// ([`refused_message`]).
    pub(crate) fn for_policy(
        policy: &FileSystemSandboxPolicy,
        cwd: &Path,
    ) -> Result<Option<Self>, String> {
        let roots = credential_watch_roots(policy, cwd);
        if roots.is_empty() {
            return Ok(None);
        }
        // SAFETY: plain syscall; the returned descriptor is owned below.
        let fd = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
        if fd < 0 {
            return Err(format!(
                "cannot watch for credential files created while the command runs: {}",
                std::io::Error::last_os_error()
            ));
        }
        let template_roots = credential_rule_roots(&policy.get_unreadable_globs_with_cwd(cwd))
            .into_iter()
            .filter_map(|root| std::fs::canonicalize(root).ok())
            .collect();
        let mut watcher = Self {
            // SAFETY: `fd` is a fresh descriptor nothing else owns.
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
            roots: roots.clone(),
            template_roots,
            folders: HashMap::new(),
            known: HashMap::new(),
            placeholders: HashSet::new(),
            existing: Vec::new(),
        };
        for root in &roots {
            if let Some(refusal) = watcher.watch_tree(root, Scan::Start) {
                return Err(refusal);
            }
        }
        Ok(Some(watcher))
    }

    /// The credential files that exist now, in any letter case: the command
    /// must not read them (the glob expansion matches the exact case only).
    pub(crate) fn existing_files(&self) -> &[PathBuf] {
        &self.existing
    }

    /// Watches `root` and every folder below it (each before it is listed,
    /// so nothing created meanwhile is missed). Returns why the command must
    /// not run ([`Scan::Start`]) or must stop ([`Scan::Running`]).
    fn watch_tree(&mut self, root: &Path, scan: Scan) -> Option<String> {
        // The starting folders are always checked for their filesystem;
        // below them, only where the device changes.
        let parent_device = if self.roots.iter().any(|start| start == root) {
            None
        } else {
            root.parent()
                .and_then(|parent| std::fs::metadata(parent).ok())
                .map(|metadata| metadata.dev())
        };
        let mut pending = vec![(root.to_path_buf(), parent_device)];
        while let Some((folder, parent_device)) = pending.pop() {
            // Only the starting folders must exist; the others may be gone
            // (temporary build folders) or renamed (their new name raises its
            // own event).
            let required = scan == Scan::Start && self.roots.contains(&folder);
            let device = match std::fs::symlink_metadata(&folder) {
                Ok(metadata) if metadata.is_dir() => metadata.dev(),
                Ok(_) | Err(_) if !required => continue,
                Ok(_) => return Some(format!("{} is not a folder", folder.display())),
                Err(err) => return Some(format!("cannot read {}: {err}", folder.display())),
            };
            if parent_device != Some(device)
                && let Some(kind) = unwatchable_filesystem(&folder)
            {
                return Some(format!(
                    "{} is on a {kind} filesystem, where credential files created from elsewhere \
                     cannot be seen",
                    folder.display()
                ));
            }
            let Ok(path) = CString::new(folder.as_os_str().as_bytes()) else {
                return Some(format!("cannot watch {}", folder.display()));
            };
            // SAFETY: `path` is a valid NUL-terminated string and the
            // descriptor is open.
            let wd =
                unsafe { libc::inotify_add_watch(self.fd.as_raw_fd(), path.as_ptr(), WATCH_MASK) };
            if wd < 0 {
                let err = std::io::Error::last_os_error();
                if !required && matches!(err.raw_os_error(), Some(libc::ENOENT | libc::ENOTDIR)) {
                    continue;
                }
                return Some(format!(
                    "cannot watch {} for credential files: {err}",
                    folder.display()
                ));
            }
            // The kernel keys watches by folder: a folder seen again (or
            // renamed) keeps its watch and counts once.
            if !self.folders.contains_key(&wd) && self.folders.len() >= MAX_WATCHED_FOLDERS {
                return Some(format!(
                    "more than {MAX_WATCHED_FOLDERS} folders to watch for credential files below {}",
                    root.display()
                ));
            }
            self.folders.insert(wd, folder.clone());
            let entries = match std::fs::read_dir(&folder) {
                Ok(entries) => entries,
                Err(err) if !required && err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => return Some(format!("cannot list {}: {err}", folder.display())),
            };
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => return Some(format!("cannot list {}: {err}", folder.display())),
                };
                let name = entry.file_name();
                let path = folder.join(&name);
                // Links are not followed, as in the glob expansion.
                let metadata = match std::fs::symlink_metadata(&path) {
                    Ok(metadata) => metadata,
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                    // An entry that cannot be examined could be a folder that
                    // is never watched: fail closed.
                    Err(err) => return Some(format!("cannot read {}: {err}", path.display())),
                };
                if metadata.is_dir() {
                    pending.push((path, Some(device)));
                    continue;
                }
                if !self.is_credential(&path, &name) || self.placeholders.contains(&path) {
                    continue;
                }
                match scan {
                    Scan::Start => {
                        if let Some(message) = unsafe_credential_entry_message(&path, &metadata) {
                            return Some(message);
                        }
                        if metadata.is_file() {
                            self.existing.push(path.clone());
                        }
                        self.known.insert(path, metadata.ino());
                    }
                    Scan::Running if self.known.get(&path) != Some(&metadata.ino()) => {
                        return Some(path.display().to_string());
                    }
                    Scan::Running => {}
                }
            }
        }
        None
    }

    /// Whether `name` (of `path`) is a credential file name here.
    fn is_credential(&self, path: &Path, name: &OsStr) -> bool {
        is_credential_file_name(name)
            || (is_credential_template_name(name)
                && !self
                    .template_roots
                    .iter()
                    .any(|root| path.starts_with(root)))
    }

    /// Starts watching beside the command, which `stop_command` stops as
    /// soon as a credential name appears (#935: by cutting the supervisor's
    /// lifeline). `placeholders` are the empty mount points bubblewrap
    /// creates for it; they are not credential files.
    pub(crate) fn start(
        mut self,
        stop_command: impl FnOnce() + Send + 'static,
        placeholders: &[PathBuf],
    ) -> CredentialMonitor {
        self.placeholders = placeholders
            .iter()
            .map(|path| real_spelling(path))
            .collect();
        let stop = Arc::new(AtomicBool::new(false));
        let found = Arc::new(Mutex::new(None));
        let (thread_stop, thread_found) = (Arc::clone(&stop), Arc::clone(&found));
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                let Some(event) = self.next_match() else {
                    continue;
                };
                stop_command();
                *thread_found
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(event);
                return;
            }
        });
        CredentialMonitor {
            stop,
            found,
            handle,
        }
    }

    /// Waits briefly for events; `Some` with what to report when one must
    /// stop the command.
    fn next_match(&mut self) -> Option<String> {
        let mut poll_fd = libc::pollfd {
            fd: self.fd.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one valid pollfd.
        if unsafe { libc::poll(&mut poll_fd, 1, 20) } <= 0 {
            return None;
        }
        let mut buffer = [0_u8; 64 * 1024];
        // SAFETY: reads at most `buffer.len()` bytes into `buffer`.
        let read = unsafe {
            libc::read(
                self.fd.as_raw_fd(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
            )
        };
        let Ok(read) = usize::try_from(read) else {
            let err = std::io::Error::last_os_error();
            return match err.raw_os_error() {
                Some(libc::EAGAIN | libc::EINTR) => None,
                _ => Some(format!("watching for credential files failed: {err}")),
            };
        };
        self.matches_in(&buffer[..read])
    }

    fn matches_in(&mut self, mut events: &[u8]) -> Option<String> {
        const HEADER: usize = std::mem::size_of::<libc::inotify_event>();
        while events.len() >= HEADER {
            // SAFETY: at least `HEADER` bytes remain; the read is unaligned.
            let event: libc::inotify_event =
                unsafe { std::ptr::read_unaligned(events.as_ptr().cast()) };
            let end = HEADER.saturating_add(usize::try_from(event.len).unwrap_or(usize::MAX));
            let Some(name) = events.get(HEADER..end) else {
                return Some("a truncated credential file watch event".to_string());
            };
            events = &events[end..];
            if event.mask & libc::IN_IGNORED != 0 {
                // The folder is gone: it no longer counts toward the limit.
                self.folders.remove(&event.wd);
                continue;
            }
            if event.mask & libc::IN_Q_OVERFLOW != 0 {
                // Events were lost: look at everything again.
                for root in self.roots.clone() {
                    if let Some(found) = self.watch_tree(&root, Scan::Running) {
                        return Some(found);
                    }
                }
                continue;
            }
            let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let name = OsStr::from_bytes(name);
            let Some(folder) = self.folders.get(&event.wd).cloned() else {
                continue;
            };
            let path = folder.join(name);
            if event.mask & libc::IN_ISDIR != 0 {
                // A folder created or moved in: watch it, and anything
                // already inside it counts as created now.
                if let Some(found) = self.watch_tree(&path, Scan::Running) {
                    return Some(found);
                }
            } else if self.is_credential(&path, name) && !self.placeholders.contains(&path) {
                return Some(path.display().to_string());
            }
        }
        None
    }
}

impl CredentialMonitor {
    /// What stopped the command, if anything did. Call it before the
    /// command is reaped.
    pub(crate) fn stop(self) -> Option<String> {
        self.stop.store(true, Ordering::SeqCst);
        if self.handle.join().is_err() {
            return Some("the credential file watcher failed".to_string());
        }
        self.found
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

/// `path` with its folder resolved, the way the watcher spells paths.
fn real_spelling(path: &Path) -> PathBuf {
    match (
        path.parent()
            .and_then(|parent| std::fs::canonicalize(parent).ok()),
        path.file_name(),
    ) {
        (Some(parent), Some(name)) => parent.join(name),
        _ => path.to_path_buf(),
    }
}

/// The kind of filesystem `folder` is on, when inotify there misses changes
/// made from elsewhere.
fn unwatchable_filesystem(folder: &Path) -> Option<&'static str> {
    let path = CString::new(folder.as_os_str().as_bytes()).ok()?;
    // SAFETY: `statfs` writes into the zeroed struct.
    let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: valid path and out pointer.
    if unsafe { libc::statfs(path.as_ptr(), &mut stat) } != 0 {
        return None;
    }
    // `f_type` is signed on glibc and unsigned on musl; the magic is 32-bit.
    #[allow(clippy::unnecessary_cast, clippy::cast_possible_truncation)]
    let kind = stat.f_type as u32;
    UNWATCHABLE_FILESYSTEMS
        .iter()
        .find(|(magic, _)| *magic == kind)
        .map(|(_, name)| *name)
}

/// What the person and the model see when a command is refused.
pub(crate) fn refused_message(reason: &str) -> String {
    format!(
        "Operation not permitted: the sandbox refuses to run this command because it cannot keep \
         credential files ({}) hidden from it: {reason}.",
        credential_file_names_shown()
    )
}

/// What the person and the model see when a command was stopped.
pub(crate) fn stopped_message(found: &str) -> String {
    format!(
        "Operation not permitted: the sandbox stopped this command because a credential file \
         ({found}) appeared while it ran. Files named {} are hidden from agent commands; one \
         created after a command starts cannot be hidden from it, so the command is stopped. \
         Later commands cannot read it.",
        credential_file_names_shown()
    )
}

#[cfg(test)]
#[path = "credential_watch_tests.rs"]
mod tests;
