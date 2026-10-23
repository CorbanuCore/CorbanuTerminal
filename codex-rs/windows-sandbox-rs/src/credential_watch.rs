//! #442 (Travis, 2026-10-12): on Windows too, a credential file that appears
//! while a sandboxed command runs stops that command, as on Linux.
//!
//! The sandbox denies reading the credential files that exist when a command
//! starts (deny ACEs on the files the policy's credential globs match). A
//! file created, renamed or moved in later gets the ACL of its folder, which
//! the command can read. So under a policy that denies credential names
//! below a folder:
//! - the folder is watched (`ReadDirectoryChangesW`, recursive, names only)
//!   from before the sandbox is set up for the command until it ends;
//! - a credential file found at the start that the deny list does not hold
//!   refuses the command ("run it again": the next command denies it);
//! - a non-folder with a credential name created, renamed to or moved in
//!   (also inside a folder moved in), even at the place of a denied file (a
//!   new file has a new ACL), stops the command at once
//!   ([`CredentialWatch::found`]); later commands deny it;
//! - fail closed: a folder that cannot be watched (also a drive root, a
//!   network or WSL folder) refuses the command; when the system reports
//!   lost changes (a full buffer), everything is scanned again and a
//!   credential file that is new, or not the file that was denied, stops the
//!   command; any other watch failure stops it.
//!
//! Limits, as on Linux: the stop follows the event, so a process already
//! waiting for the file could read it in that moment; only the folders the
//! credential globs cover are watched; the file is left in place (its
//! creator is unknown); at most [`MAX_SCANNED_ENTRIES`] entries are checked
//! at the start. Which names count is taken from the policy: `.env.example`,
//! `.env.sample` and `.env.template` only where it denies the whole
//! credential rule. On other systems the same logic runs on `notify`, for
//! tests.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::mpsc;
use std::time::Duration;

use codex_protocol::permissions::CREDENTIAL_FILE_NAME_PATTERNS;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::credential_file_glob_root;
use codex_protocol::permissions::credential_file_names_shown;
use codex_protocol::permissions::credential_rule_roots;
use codex_protocol::permissions::is_credential_template_name;

/// Entries looked at by one scan, at most. Past it the command is refused.
pub const MAX_SCANNED_ENTRIES: usize = 1_000_000;

/// How long [`CredentialWatch::finish`] lets pending changes arrive after
/// the command ended.
const SETTLE: Duration = Duration::from_millis(100);

/// Called once, from the watch thread, when the command must stop.
pub type StopCommand = Box<dyn FnOnce() + Send>;

/// What a folder watch reports.
enum Change {
    /// A path was created or renamed to.
    Appeared(PathBuf),
    /// Changes were lost (a full buffer): scan everything again.
    Lost,
    /// The watch failed: stop.
    Failed(String),
}

/// Watches the credential folders while one command runs. Dropping it ends
/// the watch.
pub struct CredentialWatch {
    found: Arc<Mutex<Option<String>>>,
    _folders: Vec<backend::FolderWatch>,
}

impl CredentialWatch {
    /// Starts watching; call it before the sandbox is set up for the
    /// command. `None` when the policy denies no credential names below a
    /// folder. `known` are the files the deny list holds. `Err` is the
    /// refusal shown instead of running it.
    pub fn start(
        policy: &FileSystemSandboxPolicy,
        cwd: &Path,
        known: &[PathBuf],
        stop: StopCommand,
    ) -> Result<Option<Self>, String> {
        let names = Arc::new(Names::for_policy(policy, cwd)?);
        if names.roots.is_empty() {
            return Ok(None);
        }
        let max_depth = policy.glob_scan_max_depth;
        let (tx, rx) = mpsc::channel::<Change>();
        let mut folders = Vec::new();
        for root in &names.roots {
            folders.push(
                backend::FolderWatch::start(root, tx.clone()).map_err(|reason| {
                    refused_message(&format!(
                        "cannot watch {} for credential files: {reason}",
                        root.display()
                    ))
                })?,
            );
        }
        drop(tx);
        // After the watch: anything created from here on is reported. The
        // files the deny list holds are recorded by identity, so one saved
        // over them later is told apart.
        let known: Arc<HashMap<PathBuf, Option<FileId>>> = Arc::new(
            known
                .iter()
                .map(|path| (key(&real_spelling(path)), file_id(path)))
                .collect(),
        );
        for root in &names.roots {
            match scan(root, &known, &names, max_depth) {
                Ok(None) => {}
                Ok(Some(found)) => {
                    return Err(refused_message(&format!(
                        "the credential file {found} appeared just before the command started; \
                         run it again"
                    )));
                }
                Err(reason) => return Err(refused_message(&reason)),
            }
        }
        let found = Arc::new(Mutex::new(None));
        let thread_found = Arc::clone(&found);
        std::thread::Builder::new()
            .name("corbanu-credential-watch".to_string())
            .spawn(move || {
                // Ends when every folder watch is dropped (the channel closes).
                while let Ok(first) = rx.recv() {
                    // Everything queued is taken at once: new paths first,
                    // then one rescan for any number of lost changes.
                    let mut batch = vec![first];
                    batch.extend(rx.try_iter());
                    let lost = batch.iter().any(|change| matches!(change, Change::Lost));
                    let found = batch
                        .into_iter()
                        .filter(|change| !matches!(change, Change::Lost))
                        .find_map(|change| must_stop(change, &known, &names, max_depth))
                        .or_else(|| {
                            lost.then(|| must_stop(Change::Lost, &known, &names, max_depth))
                                .flatten()
                        });
                    if let Some(found) = found {
                        *thread_found.lock().unwrap_or_else(PoisonError::into_inner) = Some(found);
                        stop();
                        return;
                    }
                }
            })
            .map_err(|err| refused_message(&format!("cannot watch for credential files: {err}")))?;
        Ok(Some(Self {
            found,
            _folders: folders,
        }))
    }

    /// The credential file whose appearance stopped the command, if any.
    pub fn found(&self) -> Option<String> {
        self.found
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// [`Self::found`] once the command has ended, after changes still on
    /// their way arrived.
    pub fn finish(&self) -> Option<String> {
        std::thread::sleep(SETTLE);
        self.found()
    }
}

/// The folders whose credential names the policy denies, existing, outermost
/// only.
pub fn credential_watch_roots(policy: &FileSystemSandboxPolicy, cwd: &Path) -> Vec<PathBuf> {
    Names::for_policy(policy, cwd)
        .map(|names| names.roots)
        .unwrap_or_default()
}

/// Which file names count as credential files where, from the policy.
struct Names {
    /// The folders to watch (outermost, real spelling).
    roots: Vec<PathBuf>,
    /// Each folder a credential glob covers (compared spelling) and its name
    /// pattern; a pattern applies below its own folder.
    covered: Vec<(PathBuf, &'static str)>,
    /// Folders where the whole rule applies: their `.env.*` leaves the
    /// templates out.
    template_roots: Vec<PathBuf>,
}

impl Names {
    fn for_policy(policy: &FileSystemSandboxPolicy, cwd: &Path) -> Result<Self, String> {
        let globs = policy.get_unreadable_globs_with_cwd(cwd);
        let mut covered: Vec<(PathBuf, &'static str)> = Vec::new();
        let mut real_folders: Vec<PathBuf> = Vec::new();
        for glob in &globs {
            let Some(root) = credential_file_glob_root(glob) else {
                continue;
            };
            let Some(name) = CREDENTIAL_FILE_NAME_PATTERNS
                .iter()
                .find(|name| glob.ends_with(**name))
            else {
                continue;
            };
            if !root.is_dir() {
                continue;
            }
            if root.parent().is_none() || root.parent() == Some(Path::new("")) {
                return Err(refused_message(&format!(
                    "{} is a drive root, which is too broad to watch; start from a folder",
                    root.display()
                )));
            }
            let real = dunce::canonicalize(&root).map_err(|err| {
                refused_message(&format!("cannot read {}: {err}", root.display()))
            })?;
            if let Some(kind) = unwatchable(&real) {
                return Err(refused_message(&format!(
                    "{} is on a {kind}, where credential files created from elsewhere cannot be seen",
                    real.display()
                )));
            }
            covered.push((key(&real), name));
            real_folders.push(real);
        }
        real_folders.sort();
        real_folders.dedup_by(|a, b| key(a) == key(b));
        let roots = real_folders
            .iter()
            .filter(|folder| {
                !real_folders
                    .iter()
                    .any(|other| key(other) != key(folder) && key(folder).starts_with(key(other)))
            })
            .cloned()
            .collect();
        let template_roots = credential_rule_roots(&globs)
            .into_iter()
            .filter_map(|root| dunce::canonicalize(root).ok())
            .map(|root| key(&root))
            .collect();
        Ok(Self {
            roots,
            covered,
            template_roots,
        })
    }

    fn is_credential(&self, path: &Path) -> bool {
        self.is_credential_within(path, /*max_depth*/ None)
    }

    /// How many levels below `folder` the deepest covered folder is.
    fn covered_depth_below(&self, folder: &Path) -> usize {
        let folder = key(folder);
        self.covered
            .iter()
            .filter_map(|(covered, _)| covered.strip_prefix(&folder).ok())
            .map(|below| below.components().count())
            .max()
            .unwrap_or(0)
    }

    /// As [`Self::is_credential`], counting only covered folders the path
    /// is at most `max_depth` levels below (the deny list's scan depth).
    fn is_credential_within(&self, path: &Path, max_depth: Option<usize>) -> bool {
        let Some(name) = path.file_name() else {
            return false;
        };
        let name = name.to_string_lossy();
        // A `name:stream` alternate data stream belongs to the file `name`.
        let name = if cfg!(windows) {
            name.split(':').next().unwrap_or_default().to_string()
        } else {
            name.into_owned()
        };
        let lower = name.to_ascii_lowercase();
        let template = is_credential_template_name(std::ffi::OsStr::new(&name));
        let path = key(path);
        self.covered.iter().any(|(folder, pattern)| {
            path.starts_with(folder)
                && max_depth.is_none_or(|max_depth| {
                    path.strip_prefix(folder)
                        .is_ok_and(|below| below.components().count() <= max_depth)
                })
                && match *pattern {
                    // Under the whole rule the templates are not credentials.
                    ".env.*" => {
                        lower.starts_with(".env.")
                            && !(template && self.template_roots.contains(folder))
                    }
                    exact => lower == exact,
                }
        })
    }
}

/// What to stop for, if anything, after one change.
fn must_stop(
    change: Change,
    known: &HashMap<PathBuf, Option<FileId>>,
    names: &Names,
    max_depth: Option<usize>,
) -> Option<String> {
    let path = match change {
        Change::Appeared(path) => path,
        Change::Lost => {
            for root in &names.roots {
                match scan(root, known, names, max_depth) {
                    Ok(None) => {}
                    Ok(Some(found)) => return Some(found),
                    Err(reason) => {
                        return Some(format!("a folder that could not be checked ({reason})"));
                    }
                }
            }
            return None;
        }
        Change::Failed(reason) => {
            return Some(format!("an unwatched change (the watch failed: {reason})"));
        }
    };
    match std::fs::symlink_metadata(&path) {
        // A folder created or moved in: anything inside it is new.
        Ok(metadata) if metadata.is_dir() && !is_link(&metadata) => {
            match scan(&path, &HashMap::new(), names, /*max_depth*/ None) {
                Ok(found) => found,
                Err(reason) => Some(format!("a folder that could not be checked ({reason})")),
            }
        }
        // A new file, even at the place of a denied one, has a new ACL.
        // One already gone may have been read. (Short names in the folder
        // part are resolved first.)
        _ if names.is_credential(&path)
            || (credential_like(&path) && names.is_credential(&real_spelling(&path))) =>
        {
            Some(path.display().to_string())
        }
        _ => None,
    }
}

/// A credential file below `folder` (links and junctions are not followed)
/// that is not a file `known` holds. `Err` when the folder cannot be
/// checked.
fn scan(
    folder: &Path,
    known: &HashMap<PathBuf, Option<FileId>>,
    names: &Names,
    max_depth: Option<usize>,
) -> Result<Option<String>, String> {
    // Deep enough for the deepest covered folder below `folder` plus the
    // deny list's depth.
    let limit = max_depth.map(|max_depth| max_depth + names.covered_depth_below(folder));
    let mut pending = vec![(folder.to_path_buf(), 0_usize)];
    let mut seen = 0_usize;
    while let Some((folder, depth)) = pending.pop() {
        let entries = match std::fs::read_dir(&folder) {
            Ok(entries) => entries,
            // Removed meanwhile (a temporary build folder).
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(format!("cannot list {}: {err}", folder.display())),
        };
        for entry in entries {
            let entry = entry.map_err(|err| format!("cannot list {}: {err}", folder.display()))?;
            seen += 1;
            if seen > MAX_SCANNED_ENTRIES {
                return Err(format!(
                    "more than {MAX_SCANNED_ENTRIES} files to check for credential files below {}; \
                     start from a smaller folder",
                    folder.display()
                ));
            }
            let path = entry.path();
            // From the listing itself: no handle per entry.
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => return Err(format!("cannot read {}: {err}", path.display())),
            };
            // `is_symlink` covers symbolic links, junctions and mount points
            // (name surrogates); other reparse points (cloud files) are
            // folders like any other.
            let is_dir = kind.is_dir() && !kind.is_symlink();
            if is_dir {
                if limit.is_none_or(|limit| depth + 1 < limit) {
                    pending.push((path, depth + 1));
                }
                continue;
            }
            if kind.is_dir() || !names.is_credential_within(&path, max_depth) {
                continue;
            }
            // Only the very file that was denied is fine: a file at a denied
            // path that did not exist then, or was replaced since, is new.
            match known.get(&key(&path)) {
                Some(Some(id)) if file_id(&path) == Some(*id) => {}
                _ => return Ok(Some(path.display().to_string())),
            }
        }
    }
    Ok(None)
}

/// A file's identity (volume and file number): a file saved over another by
/// rename has a new one.
type FileId = (u64, u64);

#[cfg(unix)]
fn file_id(path: &Path) -> Option<FileId> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path).ok()?;
    Some((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
fn file_id(path: &Path) -> Option<FileId> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION;
    use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandle;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const SHARE_ALL: u32 = 0x7;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let file = std::fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(SHARE_ALL)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .ok()?;
    // SAFETY: zeroed out-structure; the handle is open for the call.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, &mut info) } == 0 {
        return None;
    }
    Some((
        u64::from(info.dwVolumeSerialNumber),
        (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    ))
}

/// Symbolic links, junctions and mount points (`std` counts all name
/// surrogates); they are not followed.
fn is_link(metadata: &std::fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

/// Where a folder is that changes made elsewhere are not reported from.
#[cfg(windows)]
fn unwatchable(folder: &Path) -> Option<&'static str> {
    use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
    const DRIVE_REMOTE: u32 = 4;
    let text = folder.to_string_lossy();
    let local = match text.strip_prefix(r"\\?\") {
        Some(rest)
            if rest
                .get(..4)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("UNC\\")) =>
        {
            return Some("network or WSL share");
        }
        Some(rest) => rest.to_string(),
        None if text.starts_with(r"\\") => return Some("network or WSL share"),
        None => text.to_string(),
    };
    let drive: String = local.chars().take(2).collect();
    let root = crate::winutil::to_wide(format!("{drive}\\"));
    // SAFETY: a NUL-terminated wide string.
    (unsafe { GetDriveTypeW(root.as_ptr()) } == DRIVE_REMOTE).then_some("network drive")
}

#[cfg(not(windows))]
fn unwatchable(_folder: &Path) -> Option<&'static str> {
    None
}

/// A name that could be a credential file's (any rule): only those are worth
/// resolving the folder of.
fn credential_like(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        name.to_string_lossy()
            .to_ascii_lowercase()
            .starts_with(".env")
    })
}

/// `path` with its folder resolved, the way the scan spells paths.
fn real_spelling(path: &Path) -> PathBuf {
    match (
        path.parent()
            .and_then(|parent| dunce::canonicalize(parent).ok()),
        path.file_name(),
    ) {
        (Some(parent), Some(name)) => parent.join(name),
        _ => path.to_path_buf(),
    }
}

/// How a path is compared: Windows paths match in any letter case.
fn key(path: &Path) -> PathBuf {
    let path = dunce::simplified(path);
    if cfg!(windows) {
        // Long paths keep the `\\?\` prefix after canonicalizing; compare
        // them as the plain drive path the scan builds.
        let text = path.to_string_lossy().to_lowercase();
        let plain = text
            .strip_prefix(r"\\?\")
            .map(|rest| match rest.strip_prefix("unc\\") {
                Some(share) => format!(r"\\{share}"),
                None => rest.to_string(),
            });
        PathBuf::from(plain.unwrap_or(text))
    } else {
        path.to_path_buf()
    }
}

/// The output of a command the watch stopped: `stderr` gains the stop
/// message, and a success exit code becomes a failure.
pub fn stopped_output(found: Option<String>, exit_code: i32, stderr: &mut Vec<u8>) -> i32 {
    let Some(found) = found else {
        return exit_code;
    };
    if !stderr.is_empty() && !stderr.ends_with(b"\n") {
        stderr.push(b'\n');
    }
    stderr.extend_from_slice(stopped_message(&found).as_bytes());
    stderr.push(b'\n');
    if exit_code == 0 { 1 } else { exit_code }
}

/// What the person and the model see when a command is refused.
pub fn refused_message(reason: &str) -> String {
    format!(
        "Operation not permitted: the sandbox refuses to run this command because it cannot keep \
         credential files ({}) hidden from it: {reason}.",
        credential_file_names_shown()
    )
}

/// What the person and the model see when a command was stopped.
pub fn stopped_message(found: &str) -> String {
    format!(
        "Operation not permitted: the sandbox stopped this command because a credential file \
         ({found}) appeared while it ran. Files named {} are hidden from agent commands; one \
         created after a command starts cannot be hidden from it, so the command is stopped. \
         Later commands cannot read it.",
        credential_file_names_shown()
    )
}

/// Windows: `ReadDirectoryChangesW` on the folder, names only, with lost
/// changes and failures reported (the `notify` backend drops both).
#[cfg(windows)]
mod backend {
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::mpsc::Sender;
    use std::thread::JoinHandle;

    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::ERROR_NOTIFY_ENUM_DIR;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
    use windows_sys::Win32::Storage::FileSystem::CreateFileW;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;
    use windows_sys::Win32::Storage::FileSystem::FILE_LIST_DIRECTORY;
    use windows_sys::Win32::Storage::FileSystem::FILE_NOTIFY_CHANGE_DIR_NAME;
    use windows_sys::Win32::Storage::FileSystem::FILE_NOTIFY_CHANGE_FILE_NAME;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
    use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
    use windows_sys::Win32::Storage::FileSystem::ReadDirectoryChangesW;
    use windows_sys::Win32::System::IO::CancelIoEx;
    use windows_sys::Win32::System::IO::GetOverlappedResult;
    use windows_sys::Win32::System::IO::OVERLAPPED;
    use windows_sys::Win32::System::Threading::CreateEventW;
    use windows_sys::Win32::System::Threading::INFINITE;
    use windows_sys::Win32::System::Threading::ResetEvent;
    use windows_sys::Win32::System::Threading::SetEvent;
    use windows_sys::Win32::System::Threading::WaitForMultipleObjects;

    use super::Change;

    /// Buffer for one read, in 8-byte words (DWORD-aligned): 64 KiB.
    const BUFFER_WORDS: usize = 8 * 1024;
    const FILE_ACTION_ADDED: u32 = 1;
    const FILE_ACTION_RENAMED_NEW_NAME: u32 = 5;

    /// One watched folder; dropping it ends the watch.
    pub(super) struct FolderWatch {
        stop: HANDLE,
        thread: Option<JoinHandle<()>>,
    }

    /// One folder's handle, event, buffer and OVERLAPPED. Always boxed: the
    /// system writes to the OVERLAPPED and buffer addresses of a pending
    /// read, so they never move while one is pending.
    struct Read {
        dir: HANDLE,
        event: HANDLE,
        overlapped: OVERLAPPED,
        buffer: Vec<u64>,
        pending: bool,
    }

    // SAFETY: plain kernel handles and owned memory, used by one thread at a
    // time (the starting thread, then the watch thread).
    unsafe impl Send for Read {}

    impl Read {
        /// Ask for the next changes; `false` when the read cannot be issued.
        fn issue(&mut self) -> bool {
            // SAFETY: the event is valid; `self` is boxed, so the buffer and
            // OVERLAPPED stay at these addresses until the read completes or
            // `drop` has cancelled it and waited.
            let issued = unsafe {
                ResetEvent(self.event);
                self.overlapped = std::mem::zeroed();
                self.overlapped.hEvent = self.event;
                ReadDirectoryChangesW(
                    self.dir,
                    self.buffer.as_mut_ptr().cast(),
                    u32::try_from(self.buffer.len() * 8).unwrap_or(u32::MAX),
                    1,
                    FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME,
                    std::ptr::null_mut(),
                    &mut self.overlapped,
                    None,
                ) != 0
            };
            self.pending = issued;
            issued
        }
    }

    impl Drop for Read {
        fn drop(&mut self) {
            // SAFETY: a pending read was issued on this handle with this very
            // OVERLAPPED; it is cancelled and waited for before the memory and
            // handles go away. The handles are owned and closed once.
            unsafe {
                if self.pending {
                    CancelIoEx(self.dir, &self.overlapped);
                    let mut ignored = 0_u32;
                    GetOverlappedResult(self.dir, &self.overlapped, &mut ignored, 1);
                }
                if self.event != 0 {
                    CloseHandle(self.event);
                }
                CloseHandle(self.dir);
            }
        }
    }

    impl FolderWatch {
        pub(super) fn start(root: &Path, tx: Sender<Change>) -> Result<Self, String> {
            let wide = crate::winutil::to_wide(root);
            // SAFETY: a NUL-terminated wide path; the handle is owned below.
            let dir = unsafe {
                CreateFileW(
                    wide.as_ptr(),
                    FILE_LIST_DIRECTORY,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    std::ptr::null(),
                    OPEN_EXISTING,
                    FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
                    0,
                )
            };
            if dir == INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error().to_string());
            }
            // SAFETY: plain calls; null names and attributes.
            let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
            let stop = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
            let mut read = Box::new(Read {
                dir,
                event,
                // SAFETY: plain data, set before use.
                overlapped: unsafe { std::mem::zeroed() },
                buffer: vec![0; BUFFER_WORDS],
                pending: false,
            });
            if event == 0 || stop == 0 {
                // SAFETY: closes what was created.
                unsafe {
                    if stop != 0 {
                        CloseHandle(stop);
                    }
                }
                return Err(std::io::Error::last_os_error().to_string());
            }
            // The first read is issued before this returns: changes from now
            // on are buffered by the system.
            if !read.issue() {
                let err = std::io::Error::last_os_error();
                // SAFETY: closes what was created.
                unsafe { CloseHandle(stop) };
                return Err(err.to_string());
            }
            let root = root.to_path_buf();
            // On failure the closure, and the boxed read with it, is dropped:
            // the pending read is cancelled and waited for.
            let thread = match std::thread::Builder::new()
                .name("corbanu-credential-folder".to_string())
                .spawn(move || run(read, stop, &root, &tx))
            {
                Ok(thread) => thread,
                Err(err) => {
                    // SAFETY: owned, closed once.
                    unsafe { CloseHandle(stop) };
                    return Err(err.to_string());
                }
            };
            Ok(Self {
                stop,
                thread: Some(thread),
            })
        }
    }

    impl Drop for FolderWatch {
        fn drop(&mut self) {
            // SAFETY: the event is owned here and stays open until the
            // thread has ended.
            unsafe { SetEvent(self.stop) };
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
            // SAFETY: owned, closed once.
            unsafe { CloseHandle(self.stop) };
        }
    }

    // The read was issued before the move here: the system writes into its
    // `OVERLAPPED` and buffer, so it must stay boxed (unmoved).
    #[allow(clippy::boxed_local)]
    fn run(mut read: Box<Read>, stop: HANDLE, root: &Path, tx: &Sender<Change>) {
        loop {
            let handles = [read.event, stop];
            // SAFETY: two valid event handles.
            let woke = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) };
            if woke != WAIT_OBJECT_0 {
                // Stopped (or the wait failed); dropping `read` cancels the
                // pending read and waits for it.
                if woke != WAIT_OBJECT_0 + 1 {
                    let _ = tx.send(Change::Failed("waiting for changes failed".to_string()));
                }
                return;
            }
            let mut bytes = 0_u32;
            // SAFETY: the read completed (its event is set).
            let ok = unsafe { GetOverlappedResult(read.dir, &read.overlapped, &mut bytes, 0) };
            read.pending = false;
            let change_sent = if ok == 0 {
                // SAFETY: plain call.
                let code = unsafe { GetLastError() };
                if code == ERROR_NOTIFY_ENUM_DIR {
                    tx.send(Change::Lost).is_ok()
                } else {
                    let _ = tx.send(Change::Failed(format!("error {code}")));
                    return;
                }
            } else if bytes == 0 {
                // The buffer overflowed: the changes were lost.
                tx.send(Change::Lost).is_ok()
            } else {
                send_entries(&read.buffer, usize::try_from(bytes).unwrap_or(0), root, tx)
            };
            if !change_sent {
                return;
            }
            if !read.issue() {
                let _ = tx.send(Change::Failed(std::io::Error::last_os_error().to_string()));
                return;
            }
        }
    }

    /// Sends the created and renamed-to paths of one completed read.
    fn send_entries(buffer: &[u64], len: usize, root: &Path, tx: &Sender<Change>) -> bool {
        // SAFETY: viewing the initialized words as bytes.
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), buffer.len() * 8) };
        let Some(entries) = super::parse_notify_entries(&bytes[..len.min(bytes.len())]) else {
            return tx.send(Change::Lost).is_ok();
        };
        for (action, name) in entries {
            if (action == FILE_ACTION_ADDED || action == FILE_ACTION_RENAMED_NEW_NAME)
                && tx
                    .send(Change::Appeared(root.join(PathBuf::from(name))))
                    .is_err()
            {
                return false;
            }
        }
        true
    }
}

/// `FILE_NOTIFY_INFORMATION` entries (next offset, action, name length in
/// bytes, then the UTF-16 name relative to the folder) as (action, name);
/// `None` when the buffer is malformed (treated as lost changes).
#[cfg(any(windows, test))]
fn parse_notify_entries(bytes: &[u8]) -> Option<Vec<(u32, String)>> {
    let word = |at: usize| -> Option<u32> {
        Some(u32::from_le_bytes(
            bytes.get(at..at.checked_add(4)?)?.try_into().ok()?,
        ))
    };
    let mut entries = Vec::new();
    let mut at = 0_usize;
    loop {
        let next = word(at)?;
        let action = word(at.checked_add(4)?)?;
        let name_len = usize::try_from(word(at.checked_add(8)?)?).ok()?;
        let start = at.checked_add(12)?;
        let name = bytes.get(start..start.checked_add(name_len)?)?;
        let units: Vec<u16> = name
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        entries.push((action, String::from_utf16_lossy(&units)));
        if next == 0 {
            return Some(entries);
        }
        at = at.checked_add(usize::try_from(next).ok()?)?;
        if at >= bytes.len() {
            return None;
        }
    }
}

/// Other systems (tests): `notify`; a rescan request is a lost change, an
/// error a failed watch.
#[cfg(not(windows))]
mod backend {
    use std::path::Path;
    use std::sync::mpsc::Sender;

    use notify::EventKind;
    use notify::RecursiveMode;
    use notify::Watcher;
    use notify::event::ModifyKind;
    use notify::event::RenameMode;

    use super::Change;

    pub(super) struct FolderWatch {
        _watcher: notify::RecommendedWatcher,
    }

    impl FolderWatch {
        pub(super) fn start(root: &Path, tx: Sender<Change>) -> Result<Self, String> {
            let mut watcher =
                notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                    let event = match event {
                        Ok(event) if event.need_rescan() => {
                            let _ = tx.send(Change::Lost);
                            return;
                        }
                        Ok(event) => event,
                        Err(err) => {
                            let _ = tx.send(Change::Failed(err.to_string()));
                            return;
                        }
                    };
                    let appeared = matches!(
                        event.kind,
                        EventKind::Create(_)
                            | EventKind::Any
                            | EventKind::Modify(ModifyKind::Name(
                                RenameMode::To | RenameMode::Both | RenameMode::Any
                            ))
                    );
                    if appeared {
                        for path in event.paths {
                            let _ = tx.send(Change::Appeared(path));
                        }
                    }
                })
                .map_err(|err| err.to_string())?;
            watcher
                .watch(root, RecursiveMode::Recursive)
                .map_err(|err| err.to_string())?;
            Ok(Self { _watcher: watcher })
        }
    }
}

#[cfg(test)]
#[path = "credential_watch_tests.rs"]
mod tests;
