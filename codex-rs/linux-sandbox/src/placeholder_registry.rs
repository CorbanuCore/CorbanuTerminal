//! #607 (#582 follow-ups): the shared placeholder registry's checks that keep
//! one run from removing what is not a stale placeholder of its own user:
//! - a run in another PID namespace (a container sharing `TMPDIR`), whose
//!   pid `kill(pid, 0)` cannot see, is taken as running;
//! - a placeholder is removed without following a link in the folders above
//!   it, at the location they had when it was registered;
//! - a placeholder another run recorded whose status changed well after the
//!   record was written (the person removed it and made their own) is not
//!   removed.

use std::ffi::CString;
use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::SystemTime;

/// How long after its record a placeholder may still be made: bubblewrap
/// makes it right after the run registers it. Generous, for a host folder
/// whose file server's clock runs ahead, or a run stopped in between.
const PLACEHOLDER_MADE_WITHIN: Duration = Duration::from_secs(15 * 60);

/// The extension of the file beside a run's marker naming its PID namespace.
const PID_NAMESPACE_EXTENSION: &str = "pidns";

/// This process's PID namespace (the inode of `/proc/self/ns/pid`), when
/// known.
pub(crate) fn own_pid_namespace() -> Option<u64> {
    static NAMESPACE: OnceLock<Option<u64>> = OnceLock::new();
    *NAMESPACE.get_or_init(|| {
        fs::metadata("/proc/self/ns/pid")
            .ok()
            .map(|metadata| metadata.ino())
    })
}

/// The file beside `marker` (named for a run's pid) that names the run's PID
/// namespace.
pub(crate) fn pid_namespace_file(marker: &Path) -> PathBuf {
    marker.with_extension(PID_NAMESPACE_EXTENSION)
}

/// Writes a run's marker, after the file naming its PID namespace.
pub(crate) fn write_marker(marker: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(namespace) = own_pid_namespace() {
        fs::write(pid_namespace_file(marker), namespace.to_string())?;
    }
    fs::write(marker, contents)
}

/// Removes a run's marker and the file naming its PID namespace.
pub(crate) fn remove_marker(marker: &Path) -> io::Result<()> {
    match fs::remove_file(pid_namespace_file(marker)) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    fs::remove_file(marker)
}

/// Whether the run that wrote `marker` (pid `pid`) may still be running. A
/// run in another PID namespace is: `kill(pid, 0)` there names another
/// process, or none.
pub(crate) fn marker_run_is_active(marker: &Path, pid: libc::pid_t) -> bool {
    if let Ok(namespace) = fs::read_to_string(pid_namespace_file(marker))
        && namespace.trim().parse::<u64>().ok() != own_pid_namespace()
    {
        return true;
    }
    process_is_active(pid)
}

pub(crate) fn process_is_active(pid: libc::pid_t) -> bool {
    // SAFETY: `kill` with signal 0 only checks the pid.
    let result = unsafe { libc::kill(pid, 0) };
    if result == 0 {
        return true;
    }
    let err = io::Error::last_os_error();
    !matches!(err.raw_os_error(), Some(libc::ESRCH))
}

/// `path` with the folder it is in resolved (every link followed), as it is
/// when a run registers its placeholder, before the command runs. `None`
/// when the folder cannot be resolved.
pub(crate) fn real_placeholder_path(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?;
    let parent = fs::canonicalize(path.parent()?).ok()?;
    Some(parent.join(name))
}

/// What is at a placeholder's location, reached without following a link.
pub(crate) struct Found {
    folder: OwnedFd,
    name: CString,
    pub(crate) status: libc::stat,
}

impl Found {
    pub(crate) fn is_empty_file(&self) -> bool {
        self.status.st_mode & libc::S_IFMT == libc::S_IFREG && self.status.st_size == 0
    }

    pub(crate) fn is_dir(&self) -> bool {
        self.status.st_mode & libc::S_IFMT == libc::S_IFDIR
    }

    pub(crate) fn identity(&self) -> (u64, u64) {
        (self.status.st_dev, self.status.st_ino)
    }

    /// The status of the folder it is in.
    pub(crate) fn folder_status(&self) -> io::Result<libc::stat> {
        // SAFETY: `stat` is plain data, filled in by `fstat` below.
        let mut status: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: the folder descriptor is open and `status` writable.
        if unsafe { libc::fstat(self.folder.as_raw_fd(), &mut status) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(status)
    }

    /// When its status last changed (made, linked or renamed).
    pub(crate) fn changed_at(&self) -> SystemTime {
        let seconds = u64::try_from(self.status.st_ctime).unwrap_or(0);
        let nanos = u32::try_from(self.status.st_ctime_nsec).unwrap_or(0);
        SystemTime::UNIX_EPOCH + Duration::new(seconds, nanos)
    }

    /// Removes it (`unlinkat` in the folder found, so a link swapped in
    /// above it since cannot redirect the removal).
    pub(crate) fn remove(&self) -> io::Result<()> {
        let flags = if self.is_dir() { libc::AT_REMOVEDIR } else { 0 };
        // SAFETY: the folder descriptor is open and the name a C string.
        if unsafe { libc::unlinkat(self.folder.as_raw_fd(), self.name.as_ptr(), flags) } < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

/// What is at `path` (absolute), reached from `/` without following any
/// link: `Ok(None)` when it is missing, or when a folder above it is a link
/// or not a folder (it is then not the placeholder this run registered).
pub(crate) fn find_without_links(path: &Path) -> io::Result<Option<Found>> {
    let (folder, name) = match open_parent_without_links(path) {
        Ok(found) => found,
        Err(err)
            if err.kind() == io::ErrorKind::InvalidInput
                || matches!(
                    err.raw_os_error(),
                    Some(libc::ELOOP | libc::ENOTDIR | libc::ENOENT)
                ) =>
        {
            return Ok(None);
        }
        Err(err) => return Err(err),
    };
    // SAFETY: `stat` is plain data, filled in by `fstatat` below.
    let mut status: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: the folder descriptor is open, the name a C string and
    // `status` writable.
    let result = unsafe {
        libc::fstatat(
            folder.as_raw_fd(),
            name.as_ptr(),
            &mut status,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result < 0 {
        let err = io::Error::last_os_error();
        return if err.kind() == io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(err)
        };
    }
    Ok(Some(Found {
        folder,
        name,
        status,
    }))
}

/// The folder `path` (absolute and normalized) is in, reached from `/`
/// without following any link, and its name there. An error when a folder
/// above it is a link (`ELOOP`), not a folder (`ENOTDIR`) or missing
/// (`ENOENT`), or the path is not absolute and normalized (`InvalidInput`).
pub(crate) fn open_parent_without_links(path: &Path) -> io::Result<(OwnedFd, CString)> {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    };
    let mut folder = open_folder(libc::AT_FDCWD, Path::new("/"))?;
    for component in parent.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(part) => folder = open_folder(folder.as_raw_fd(), Path::new(part))?,
            // A placeholder path is absolute and normalized.
            _ => return Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }
    let name = CString::new(name.as_bytes()).map_err(io::Error::other)?;
    Ok((folder, name))
}

/// Opens the folder `name` in `folder` without following a link.
fn open_folder(folder: libc::c_int, name: &Path) -> io::Result<OwnedFd> {
    let name = CString::new(name.as_os_str().as_bytes()).map_err(io::Error::other)?;
    // SAFETY: `name` is a C string; the result is checked below.
    let fd = unsafe {
        libc::openat(
            folder,
            name.as_ptr(),
            libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `fd` was just opened and is owned here.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

/// Whether a placeholder whose status last changed at `changed_at` may be
/// the one made for a record written at `recorded_at`: bubblewrap makes it
/// just after. One changed long after (the person removed the placeholder
/// and made their own empty file or folder there) is not. (No lower bound:
/// a file's status changes when it is made or moved there, and a file
/// server's clock may run behind.)
pub(crate) fn made_for_record(changed_at: SystemTime, recorded_at: SystemTime) -> bool {
    recorded_at
        .checked_add(PLACEHOLDER_MADE_WITHIN)
        .is_none_or(|latest| changed_at <= latest)
}

#[cfg(test)]
#[path = "placeholder_registry_tests.rs"]
mod tests;
