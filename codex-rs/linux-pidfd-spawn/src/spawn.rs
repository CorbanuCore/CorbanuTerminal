use rustix::fs::SealFlags;
use rustix::fs::fcntl_get_seals;
use rustix::fs::statfs;
use rustix::io::FdFlags;
use rustix::io::fcntl_getfd;
use rustix::process::Signal;
use rustix::process::WaitId;
use rustix::process::WaitIdOptions;
use rustix::process::WaitIdStatus;
use rustix::process::pidfd_send_signal;
use rustix::process::waitid;
use std::ffi::CStr;
use std::ffi::CString;
use std::io;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::OwnedFd;
use std::time::Duration;

/// Fixed non-root probe recipes only; no production identity or credential input.
pub enum SyntheticRole {
    Journal,
    Policy,
    Worker,
}

/// Sole stable child owner. Errors retain ownership; Drop kills and reaps before
/// releasing it. Drop has no deadline and creates no worker. Use only inside a
/// dedicated owner whose quarantine remains held until cleanup is observed.
#[must_use]
pub struct OwnedChild {
    pidfd: OwnedFd,
    status: Option<WaitIdStatus>,
}

impl OwnedChild {
    /// Observe/reap only this child; absence of an exit does not prove readiness.
    pub fn try_wait(&mut self) -> io::Result<Option<WaitIdStatus>> {
        if self.status.is_none() {
            self.status = waitid(
                WaitId::PidFd(self.pidfd.as_fd()),
                WaitIdOptions::EXITED | WaitIdOptions::NOHANG,
            )?;
        }
        Ok(self.status)
    }

    /// Requests termination by stable pidfd, not a reused numeric PID.
    pub fn terminate(&self) -> io::Result<()> {
        if self.status.is_some() {
            return Ok(());
        }
        match pidfd_send_signal(&self.pidfd, Signal::KILL) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        while self.status.is_none() {
            let _ = self.terminate();
            match self.try_wait() {
                Ok(Some(_)) => break,
                // A competing reaper violates the owner precondition; no child
                // remains waitable by this process. Never signal a replacement PID.
                Err(err) if err.raw_os_error() == Some(libc::ECHILD) => break,
                Ok(None) | Err(_) => std::thread::sleep(Duration::from_millis(20)),
            }
        }
    }
}

/// Consumes a sealed image into a fixed synthetic probe invocation. The caller
/// must separately validate its static ELF profile and provenance. Seals prove
/// immutability, not executable trust. Production activation is not exposed.
pub fn spawn_synthetic_probe(image: OwnedFd, role: SyntheticRole) -> io::Result<OwnedChild> {
    let args: &[&CStr] = match role {
        SyntheticRole::Journal => &[
            c"codex-protected-root-probe",
            c"--prepare-synthetic-child",
            c"journal",
            c"101",
            c"201",
            c"204",
        ],
        SyntheticRole::Policy => &[
            c"codex-protected-root-probe",
            c"--prepare-synthetic-child",
            c"policy",
            c"102",
            c"202",
            c"204",
        ],
        SyntheticRole::Worker => &[
            c"codex-protected-root-probe",
            c"--prepare-synthetic-child",
            c"worker",
            c"103",
            c"203",
            c"none",
        ],
    };
    if rustix::process::getuid().is_root() || rustix::process::geteuid().is_root() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "synthetic launch requires non-root",
        ));
    }
    launch(image, args)
}

fn launch(image: OwnedFd, args: &[&CStr]) -> io::Result<OwnedChild> {
    let fd = image.as_raw_fd();
    let required = SealFlags::WRITE | SealFlags::SHRINK | SealFlags::GROW | SealFlags::SEAL;
    if !(3..=256).contains(&fd)
        || !fcntl_get_seals(&image)?.contains(required)
        || !fcntl_getfd(&image)?.contains(FdFlags::CLOEXEC)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid sealed image descriptor",
        ));
    }
    if statfs(c"/proc/self/fd")?.f_type != rustix::fs::PROC_SUPER_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "trusted procfs unavailable",
        ));
    }
    let path = CString::new(format!("/proc/self/fd/{fd}")).map_err(io::Error::other)?;
    let pidfd = super::ffi::launch(&path, fd, args)?;
    Ok(OwnedChild {
        pidfd,
        status: None,
    })
}

#[cfg(test)]
#[path = "spawn_tests.rs"]
mod tests;
