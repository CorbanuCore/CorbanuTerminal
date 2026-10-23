//! #519: a Codex Apps upload read that runs outside the sandbox (profiles
//! with full write access) under the protected-path rules. Checking a path
//! and then reading it is a race: a link swapped in between makes the read
//! open a denied file. The file is opened once, without following a link at
//! its last component, what was opened is checked where the kernel says it
//! is, and the bytes come from that same handle.

use super::read_denials::readable_under;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

/// The contents of the file at `path` (at most `limit` bytes), when it is
/// readable under `policy` both before it is opened and as opened.
pub(crate) fn read_checked(
    policy: &FileSystemSandboxPolicy,
    path: &Path,
    cwd: &Path,
    limit: u64,
) -> io::Result<Vec<u8>> {
    if !readable_under(policy, path, cwd) {
        return Err(blocked());
    }
    read_opened(policy, &std::fs::canonicalize(path)?, cwd, limit)
}

/// Opens `canonical` (which a swapped folder may now redirect) and reads it
/// when what was opened is readable.
fn read_opened(
    policy: &FileSystemSandboxPolicy,
    canonical: &Path,
    cwd: &Path,
    limit: u64,
) -> io::Result<Vec<u8>> {
    let path = canonical;
    let mut file = open_no_follow(canonical)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path `{}` is not a file", path.display()),
        ));
    }
    let opened = opened_path(&file)?;
    if !readable_under(policy, &opened, cwd) {
        return Err(blocked());
    }
    if metadata.len() > limit {
        return Err(too_large(path, metadata.len(), limit));
    }
    let mut contents = Vec::new();
    (&mut file).take(limit + 1).read_to_end(&mut contents)?;
    if contents.len() as u64 > limit {
        return Err(too_large(path, contents.len() as u64, limit));
    }
    Ok(contents)
}

fn blocked() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        "reading this file is blocked by the protected-path rules",
    )
}

fn too_large(path: &Path, size: u64, limit: u64) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "file `{}` is too large: {size} bytes exceeds the limit of {limit} bytes",
            path.display()
        ),
    )
}

#[cfg(unix)]
fn open_no_follow(path: &Path) -> io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        // A FIFO swapped in would block the open; a terminal must not
        // become the process's controlling terminal.
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOCTTY)
        .open(path)
}

#[cfg(not(unix))]
fn open_no_follow(path: &Path) -> io::Result<std::fs::File> {
    std::fs::File::open(path)
}

/// Where the opened file is, as the kernel reports it. Platforms without
/// such a report refuse the read (fail closed).
fn opened_path(file: &std::fs::File) -> io::Result<PathBuf> {
    kernel_path(file)
}

#[cfg(target_os = "macos")]
fn kernel_path(file: &std::fs::File) -> io::Result<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;
    let mut buffer = vec![0u8; libc::PATH_MAX as usize];
    // SAFETY: `F_GETPATH` writes a NUL-terminated path of at most
    // `PATH_MAX` bytes into `buffer`, which holds that many.
    let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETPATH, buffer.as_mut_ptr()) };
    if result == -1 {
        return Err(io::Error::last_os_error());
    }
    let path = std::ffi::CStr::from_bytes_until_nul(&buffer).map_err(io::Error::other)?;
    Ok(PathBuf::from(std::ffi::OsStr::from_bytes(path.to_bytes())))
}

#[cfg(target_os = "linux")]
fn kernel_path(file: &std::fs::File) -> io::Result<PathBuf> {
    use std::os::unix::io::AsRawFd;
    std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn kernel_path(_file: &std::fs::File) -> io::Result<PathBuf> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "this platform cannot check which file was opened",
    ))
}

#[cfg(test)]
#[path = "upload_read_tests.rs"]
mod tests;
