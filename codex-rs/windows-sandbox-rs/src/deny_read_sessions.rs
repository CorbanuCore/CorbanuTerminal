//! #323: a machine-wide registry of the live sessions that rely on the
//! elevated sandbox's deny-read entries.
//!
//! The entries are for the sandbox's users group, which is machine-wide, so
//! sessions on different `CODEX_HOME`s (or with different rules on one) rely
//! on the same entry: one adds it, and the others find it there. A sync
//! removes only entries its own `CODEX_HOME` added, once none of their rules
//! is configured (#304), and nothing while a contract on that home is armed
//! (#301). That still let one session remove an entry another live session's
//! commands relied on.
//!
//! So each Core process that runs sandboxed commands with deny-read rules,
//! or arms the secretless launch contract, registers itself here for its
//! lifetime: a session file it holds locked. A sync removes entries only
//! while no other session is registered, and holds [`SYNC_LOCK_FILE`] from
//! that check until its state is stored; registration takes the same lock,
//! so a session registering meanwhile waits, and its launch then adds back
//! whatever was removed. Anything that keeps the check from succeeding (no
//! registry, a lock that can't be had, a session file that can't be probed)
//! keeps the entries: they stay recorded and a later sync removes them.
//!
//! The registry is under `%ProgramData%`, which every user may create files
//! in. A session file is locked and open without delete sharing while its
//! session lives, so no one else can make it look gone; a process that holds
//! a stale one, or `SYNC_LOCK_FILE`, only keeps entries from being removed.

use crate::acl::ensure_explicit_deny_read_ace;
use crate::winutil::resolve_sid;
use anyhow::Context;
use anyhow::Result;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

/// Under `%ProgramData%`.
const REGISTRY_PARENT: &str = "CorbanuTerminalSandbox";
const REGISTRY_DIR: &str = "deny-read-sessions";
/// Held exclusively while a session registers, and while a sync checks for
/// other sessions and removes entries.
pub(crate) const SYNC_LOCK_FILE: &str = "sync.lock";
const SESSION_PREFIX: &str = "session-";
const SESSION_SUFFIX: &str = ".lock";
/// Each holder keeps [`SYNC_LOCK_FILE`] for milliseconds.
const SYNC_LOCK_WAIT: Duration = Duration::from_secs(10);
const SANDBOX_USERS_GROUP: &str = "CodexSandboxUsers";
const FILE_SHARE_READ: u32 = 0x1;
const FILE_SHARE_WRITE: u32 = 0x2;
const FILE_SHARE_DELETE: u32 = 0x4;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const DELETE: u32 = 0x0001_0000;

/// Where a sync looks for other sessions, and which one is the caller's.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenyReadSessions {
    /// `None`: no registry to check, so the sync removes nothing.
    #[serde(default)]
    pub registry: Option<PathBuf>,
    /// The caller's own session file, which the check skips.
    #[serde(default)]
    pub own: Option<String>,
}

impl DenyReadSessions {
    /// This process's view: the machine-wide registry, and this process's
    /// session in it, registered first when `register` (its launch carries
    /// deny-read rules).
    pub fn for_this_process(register: bool) -> Self {
        Self {
            registry: registry_dir(),
            own: if register {
                register_this_process().ok()
            } else {
                registered_session()
            },
        }
    }
}

/// `%ProgramData%\CorbanuTerminalSandbox\deny-read-sessions`.
pub fn registry_dir() -> Option<PathBuf> {
    program_data().map(|dir| dir.join(REGISTRY_PARENT).join(REGISTRY_DIR))
}

/// Creates `registry` (and its parent) if needed, labelled medium integrity
/// for everything created in them: an elevated Core's files would otherwise
/// be high integrity, which a normal session's Core may not write.
fn create_registry_dir(registry: &Path) -> Result<()> {
    use windows_sys::Win32::Foundation::HLOCAL;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::Storage::FileSystem::CreateDirectoryW;
    if registry.is_dir() {
        return Ok(());
    }
    let parent = registry.parent().context("registry has no parent")?;
    if !parent.is_dir() && parent.parent().is_some_and(Path::is_dir) {
        create_registry_dir(parent)?;
    }
    // The DACL is inherited; only the label is set.
    let sddl = crate::winutil::to_wide("S:(ML;OICI;NW;;;ME)");
    let mut descriptor: *mut std::ffi::c_void = std::ptr::null_mut();
    // SAFETY: parses `sddl` into a descriptor freed below.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error()).context("label descriptor");
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let path = crate::winutil::to_wide(registry);
    // SAFETY: valid path and attributes.
    let created = unsafe { CreateDirectoryW(path.as_ptr(), &attributes) } != 0;
    let error = std::io::Error::last_os_error();
    // SAFETY: allocated above.
    unsafe { LocalFree(descriptor as HLOCAL) };
    if created || registry.is_dir() {
        Ok(())
    } else {
        Err(error).with_context(|| format!("create {}", registry.display()))
    }
}

fn program_data() -> Option<PathBuf> {
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::FOLDERID_ProgramData;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;
    let mut path: *mut u16 = std::ptr::null_mut();
    // SAFETY: `path` receives a string the call allocates, freed below.
    let status = unsafe { SHGetKnownFolderPath(&FOLDERID_ProgramData, 0, 0, &mut path) };
    let result = (status == 0 && !path.is_null()).then(|| {
        // SAFETY: a NUL-terminated string from the call.
        let len = (0..).take_while(|&i| unsafe { *path.add(i) } != 0).count();
        // SAFETY: as above.
        PathBuf::from(String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts(path, len)
        }))
    });
    // SAFETY: allocated by SHGetKnownFolderPath (null is allowed).
    unsafe { CoTaskMemFree(path.cast()) };
    result
}

static THIS_PROCESS: Mutex<Option<Registration>> = Mutex::new(None);

/// Registers this process once (retrying after a failure) and keeps the
/// registration until it exits. Returns its session file's name.
pub fn register_this_process() -> Result<String> {
    let mut registration = THIS_PROCESS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(registered) = registration.as_ref() {
        return Ok(registered.name.clone());
    }
    let registry = registry_dir().context("no ProgramData folder")?;
    let registered = Registration::register(&registry)?;
    let name = registered.name.clone();
    *registration = Some(registered);
    Ok(name)
}

fn registered_session() -> Option<String> {
    THIS_PROCESS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .map(|registered| registered.name.clone())
}

/// A session's file, locked until dropped or the process exits (then
/// deleted).
#[derive(Debug)]
pub struct Registration {
    file: Option<File>,
    path: PathBuf,
    name: String,
}

impl Registration {
    /// Creates and locks a new session file in `registry`, under
    /// [`SYNC_LOCK_FILE`] so no sync is between its check and its removals.
    pub fn register(registry: &Path) -> Result<Self> {
        create_registry_dir(registry)?;
        let _sync = SyncLock::acquire(registry, SYNC_LOCK_WAIT)?
            .context("another process holds the deny-read sync lock")?;
        let mut rng = SmallRng::from_entropy();
        let name = format!(
            "{SESSION_PREFIX}{}-{:032x}{SESSION_SUFFIX}",
            std::process::id(),
            rng.r#gen::<u128>()
        );
        let path = registry.join(&name);
        // Shared for reading only (no one else may write, delete or rename
        // it while it is held), and gone with the last handle, however this
        // process exits.
        let file = std::fs::OpenOptions::new()
            .access_mode(GENERIC_READ | GENERIC_WRITE | DELETE)
            .create_new(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_DELETE_ON_CLOSE)
            .open(&path)
            .with_context(|| format!("create {}", path.display()))?;
        file.try_lock()
            .map_err(|err| anyhow::anyhow!("lock {}: {err}", path.display()))?;
        deny_to_sandbox_users(&path);
        Ok(Self {
            file: Some(file),
            path,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Best effort: a sandboxed command that held a session or sync lock could
/// only keep entries from being removed.
fn deny_to_sandbox_users(path: &Path) {
    if let Ok(mut group) = resolve_sid(SANDBOX_USERS_GROUP) {
        // SAFETY: `group` is a valid SID for the call.
        let _ = unsafe { ensure_explicit_deny_read_ace(path, group.as_mut_ptr().cast()) };
    }
}

/// [`SYNC_LOCK_FILE`], held exclusively.
struct SyncLock {
    _file: File,
}

impl SyncLock {
    /// `None` if it is still held by another process after `wait`.
    fn acquire(registry: &Path, wait: Duration) -> Result<Option<Self>> {
        let path = registry.join(SYNC_LOCK_FILE);
        let open = |write: bool| {
            std::fs::OpenOptions::new()
                .read(true)
                .write(write)
                .create(write)
                .truncate(false)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)
        };
        // Another user's (read-only here) does as well: locking needs reads.
        let file = open(true)
            .or_else(|_| open(false))
            .with_context(|| format!("open {}", path.display()))?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            anyhow::bail!("{} is not a regular file", path.display());
        }
        deny_to_sandbox_users(&path);
        let deadline = Instant::now() + wait;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Some(Self { _file: file })),
                Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(std::fs::TryLockError::WouldBlock) => return Ok(None),
                Err(std::fs::TryLockError::Error(err)) => {
                    return Err(err).with_context(|| format!("lock {}", path.display()));
                }
            }
        }
    }
}

/// Held by a sync from its check for other sessions until its state is
/// stored: no other session is registered, and none can register meanwhile.
pub(crate) struct NoOtherSessions {
    _sync: SyncLock,
}

/// `Some` when no session other than `sessions.own` is registered; `None`
/// when one is, or it can't be told (then nothing may be removed).
pub(crate) fn lock_out_other_sessions(sessions: &DenyReadSessions) -> Option<NoOtherSessions> {
    let registry = sessions.registry.as_deref()?;
    create_registry_dir(registry).ok()?;
    let sync = SyncLock::acquire(registry, SYNC_LOCK_WAIT).ok()??;
    for entry in std::fs::read_dir(registry).ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_str()?;
        if !(name.starts_with(SESSION_PREFIX) && name.ends_with(SESSION_SUFFIX)) {
            continue;
        }
        if sessions.own.as_deref() == Some(name) {
            continue;
        }
        if session_is_live(&entry.path())? {
            return None;
        }
    }
    Some(NoOtherSessions { _sync: sync })
}

/// Whether a session still holds `path`; `None` if that can't be told.
/// A dead session's file is deleted when this user may.
fn session_is_live(path: &Path) -> Option<bool> {
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
    {
        Ok(file) => file,
        // Deleted since it was listed: that session is gone.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Some(false),
        // Being deleted (its holder just closed it): gone.
        Err(err) if err.raw_os_error() == Some(5) && !path.exists() => return Some(false),
        Err(_) => return None,
    };
    match file.try_lock() {
        Ok(()) => {
            drop(file);
            let _ = std::fs::remove_file(path);
            Some(false)
        }
        Err(std::fs::TryLockError::WouldBlock) => Some(true),
        Err(std::fs::TryLockError::Error(_)) => None,
    }
}

#[cfg(test)]
#[path = "deny_read_sessions_tests.rs"]
mod tests;
