//! Stable pidfs identity only; the owner separately checks child liveness.
use rustix::fs::fstat;
use rustix::fs::fstatfs;
use std::io;
use std::os::fd::BorrowedFd;

pub(crate) fn same_process(owner: BorrowedFd<'_>, peer: BorrowedFd<'_>) -> io::Result<bool> {
    // Linux pidfs uses a boot-unique inode on 64-bit kernels. Anonymous-inode
    // pidfds do not have that guarantee; 32-bit kernels need another protocol.
    if !cfg!(target_pointer_width = "64") {
        return Err(io::ErrorKind::Unsupported.into());
    }
    for fd in [owner, peer] {
        // include/uapi/linux/magic.h: PID_FS_MAGIC; not exported by rustix1.1.4.
        if fstatfs(fd)?.f_type != 0x5049_4446 {
            return Err(io::ErrorKind::Unsupported.into());
        }
    }
    let own = fstat(owner)?;
    let other = fstat(peer)?;
    Ok((own.st_dev, own.st_ino) == (other.st_dev, other.st_ino))
}

#[cfg(test)]
#[path = "peer_tests.rs"]
mod tests;
