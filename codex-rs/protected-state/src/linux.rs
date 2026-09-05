use crate::RootError;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub(crate) const MAX_BYTES: usize = 16 * 1024;

#[derive(Debug)]
pub(crate) struct Directory {
    file: File,
    #[cfg(test)]
    pub(crate) fault: std::cell::Cell<Option<Fault>>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fault {
    NoSpace,
    ShortWrite,
    FileSync,
    DirectorySync,
    AfterDurable,
}

#[repr(C)]
struct OpenHow {
    flags: u64,
    mode: u64,
    resolve: u64,
}

fn open_at(fd: i32, name: &str, flags: i32, mode: u64) -> Result<File, RootError> {
    let name = CString::new(name).map_err(|_| RootError::Invalid)?;
    let how = OpenHow {
        flags: u64::try_from(flags | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .map_err(|_| RootError::Invalid)?,
        mode,
        // Never follow a symlink, including /proc magic links. Relative opens
        // additionally cannot escape the trusted directory or cross a mount.
        resolve: 0x04 | 0x02 | if fd == libc::AT_FDCWD { 0 } else { 0x08 | 0x01 },
    };
    // SAFETY: both input pointers are valid for the duration of openat2.
    let raw = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            fd,
            name.as_ptr(),
            &how,
            size_of::<OpenHow>(),
        )
    };
    if raw < 0 {
        return Err(RootError::Unavailable);
    }
    // SAFETY: openat2 returned one newly owned file descriptor.
    Ok(unsafe { File::from_raw_fd(i32::try_from(raw).map_err(|_| RootError::Unavailable)?) })
}

fn private(file: &File, directory: bool) -> Result<(), RootError> {
    let metadata = file.metadata().map_err(|_| RootError::Unavailable)?;
    // SAFETY: geteuid has no pointer or precondition.
    let uid = unsafe { libc::geteuid() };
    if metadata.uid() != uid
        || metadata.mode() & 0o077 != 0
        || if directory {
            !metadata.is_dir()
        } else {
            !metadata.is_file() || metadata.nlink() != 1
        }
    {
        return Err(RootError::Invalid);
    }
    Ok(())
}

impl Directory {
    /// Production paths have no writable ancestor outside root's authority.
    /// Walk opened descriptors so checking one ancestor cannot be raced by
    /// changing a later pathname. Test directories deliberately do not use it.
    pub(crate) fn verify_system_path(path: &Path) -> Result<(), RootError> {
        let mut parent = open_at(libc::AT_FDCWD, "/", libc::O_RDONLY | libc::O_DIRECTORY, 0)?;
        for component in path.components() {
            match component {
                std::path::Component::RootDir => {}
                std::path::Component::Normal(name) => {
                    parent = open_at(
                        parent.as_raw_fd(),
                        name.to_str().ok_or(RootError::Invalid)?,
                        libc::O_RDONLY | libc::O_DIRECTORY,
                        0,
                    )?;
                    let metadata = parent.metadata().map_err(|_| RootError::Unavailable)?;
                    if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
                        return Err(RootError::Invalid);
                    }
                }
                _ => return Err(RootError::Invalid),
            }
        }
        Ok(())
    }

    pub(crate) fn open(path: &Path) -> Result<Self, RootError> {
        if !path.is_absolute() {
            return Err(RootError::Invalid);
        }
        let file = open_at(
            libc::AT_FDCWD,
            path.to_str().ok_or(RootError::Invalid)?,
            libc::O_RDONLY | libc::O_DIRECTORY,
            0,
        )?;
        private(&file, true)?;
        let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: stat points to writable storage; fstatfs initializes it on success.
        if unsafe { libc::fstatfs(file.as_raw_fd(), stat.as_mut_ptr()) } != 0 {
            return Err(RootError::Unavailable);
        }
        // SAFETY: fstatfs returned success.
        let filesystem = unsafe { stat.assume_init() }.f_type;
        // Initial selected implementation supports local ext-family and XFS
        // semantics only. tmpfs/network/overlay/unknown filesystems deny.
        if filesystem != 0xef53 && filesystem != 0x58465342 {
            return Err(RootError::Unsupported);
        }
        Ok(Self {
            file,
            #[cfg(test)]
            fault: std::cell::Cell::new(None),
        })
    }

    pub(crate) fn read(&self, name: &str) -> Result<Vec<u8>, RootError> {
        private(&self.file, true)?;
        let file = open_at(self.file.as_raw_fd(), name, libc::O_RDONLY, 0)?;
        private(&file, false)?;
        let mut bytes = Vec::new();
        file.take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| RootError::Unavailable)?;
        if bytes.len() > MAX_BYTES {
            return Err(RootError::Invalid);
        }
        Ok(bytes)
    }

    pub(crate) fn create(&self, name: &str, bytes: &[u8]) -> Result<(), RootError> {
        private(&self.file, true)?;
        if bytes.len() > MAX_BYTES {
            return Err(RootError::Invalid);
        }
        #[cfg(test)]
        if self.fault.get() == Some(Fault::NoSpace) {
            return Err(RootError::Unavailable);
        }
        let mut file = open_at(
            self.file.as_raw_fd(),
            name,
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        #[cfg(test)]
        if self.fault.get() == Some(Fault::ShortWrite) {
            file.write_all(&bytes[..bytes.len() / 2])
                .map_err(|_| RootError::Unavailable)?;
            return Err(RootError::Unavailable);
        }
        file.write_all(bytes).map_err(|_| RootError::Unavailable)?;
        #[cfg(test)]
        if self.fault.get() == Some(Fault::FileSync) {
            return Err(RootError::Unavailable);
        }
        file.sync_all().map_err(|_| RootError::Unavailable)?;
        self.sync()
    }

    pub(crate) fn lock(&self) -> Result<File, RootError> {
        private(&self.file, true)?;
        // The inode is enrolled once and never replaced or recreated by open.
        let file = open_at(self.file.as_raw_fd(), "lock", libc::O_RDWR, 0)?;
        private(&file, false)?;
        // SAFETY: flock acts only on the owned descriptor.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(RootError::Conflict);
        }
        Ok(file)
    }

    pub(crate) fn publish(&self) -> Result<(), RootError> {
        private(&self.file, true)?;
        // SAFETY: static NUL-terminated names and live directory descriptor.
        if unsafe {
            libc::renameat(
                self.file.as_raw_fd(),
                c"pending".as_ptr(),
                self.file.as_raw_fd(),
                c"head".as_ptr(),
            )
        } != 0
        {
            return Err(RootError::Unavailable);
        }
        #[cfg(test)]
        if self.fault.get() == Some(Fault::DirectorySync) {
            return Err(RootError::Ambiguous);
        }
        self.sync().map_err(|_| RootError::Ambiguous)?;
        #[cfg(test)]
        if self.fault.get() == Some(Fault::AfterDurable) {
            return Err(RootError::Ambiguous);
        }
        Ok(())
    }

    pub(crate) fn has_pending(&self) -> Result<bool, RootError> {
        private(&self.file, true)?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: valid descriptor, static name and writable stat storage.
        let result = unsafe {
            libc::fstatat(
                self.file.as_raw_fd(),
                c"pending".as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            return Ok(true);
        }
        if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            Ok(false)
        } else {
            Err(RootError::Unavailable)
        }
    }

    fn sync(&self) -> Result<(), RootError> {
        self.file.sync_all().map_err(|_| RootError::Unavailable)
    }
}
