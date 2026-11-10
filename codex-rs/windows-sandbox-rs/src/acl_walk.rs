//! D311/D313 (#1542): the pieces of our own tree walk, shared by D261's
//! below-check and the #158 label walk:
//! - [`par_walk`]: several threads take folders from a shared queue;
//! - each folder stays open while it is walked: [`list`] reads its entries
//!   through its handle and [`open_relative`] opens each entry relative to
//!   that handle, never following a link. A folder swapped for a link (or
//!   moved) meanwhile can't redirect the walk, and paths past `MAX_PATH`
//!   need no special form, because only one name is ever opened at a time.

use crate::winutil::to_wide;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use std::ffi::c_void;
use std::path::Path;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::PoisonError;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_NO_MORE_FILES;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Storage::FileSystem::CreateFileW;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_LIST_DIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE;
use windows_sys::Win32::Storage::FileSystem::GetFileInformationByHandleEx;
use windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING;
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;

/// Most threads either walk uses (D312).
const MAX_THREADS: usize = 8;

fn cpus() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}

/// Threads for a read-mostly check (D312): one per CPU, at most 8.
pub(crate) fn check_threads() -> usize {
    cpus().clamp(1, MAX_THREADS)
}

/// Threads for the label walk (D312), which writes most objects: label
/// writes contend (hosted, 4 CPUs: 2 threads 1.2 s, 8 threads 3.0 s), so half
/// the CPUs, at most 8.
pub(crate) fn label_threads() -> usize {
    (cpus() / 2).clamp(1, MAX_THREADS)
}

/// Runs `visit` on every item, the seeds and those it adds to its second
/// argument, on up to `threads` threads (the calling thread is one of them).
/// Stops at the first error or panic, which it returns; the remaining items
/// are dropped unvisited.
pub(crate) fn par_walk<T: Send>(
    seed: Vec<T>,
    threads: usize,
    visit: impl Fn(T, &mut Vec<T>) -> Result<()> + Sync,
) -> Result<()> {
    struct State<T> {
        queue: Vec<T>,
        active: usize,
        error: Option<anyhow::Error>,
    }
    let state = Mutex::new(State {
        queue: seed,
        active: 0,
        error: None,
    });
    let ready = Condvar::new();
    let worker = || {
        loop {
            let item = {
                let mut st = state.lock().unwrap_or_else(PoisonError::into_inner);
                loop {
                    if st.error.is_some() || (st.queue.is_empty() && st.active == 0) {
                        ready.notify_all();
                        return;
                    }
                    if let Some(item) = st.queue.pop() {
                        st.active += 1;
                        break item;
                    }
                    st = ready.wait(st).unwrap_or_else(PoisonError::into_inner);
                }
            };
            let mut found = Vec::new();
            // A panic must not leave the others waiting on this item.
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| visit(item, &mut found)))
                    .unwrap_or_else(|_| Err(anyhow!("the walk panicked")));
            let mut st = state.lock().unwrap_or_else(PoisonError::into_inner);
            st.active -= 1;
            match result {
                Ok(()) => st.queue.append(&mut found),
                Err(err) => {
                    if st.error.is_none() {
                        st.error = Some(err);
                    }
                }
            }
            drop(st);
            ready.notify_all();
        }
    };
    std::thread::scope(|scope| {
        for _ in 1..threads.max(1) {
            // A thread that can't start just leaves more for the others.
            let _ = std::thread::Builder::new().spawn_scoped(scope, worker);
        }
        worker();
    });
    match state
        .into_inner()
        .unwrap_or_else(PoisonError::into_inner)
        .error
    {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// An open file or folder handle, closed on drop. `Send` so that folders can
/// move between the walk's threads.
pub(crate) struct Handle(pub(crate) HANDLE);

// SAFETY: a kernel handle may be used from any thread. Shared folder
// handles ([`Subfolder`]) are only used to open entries relative to them,
// which is safe concurrently; a folder is listed only by the one thread
// walking it.
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

/// Volume serial number and 128-bit file ID.
pub(crate) type FileId = (u64, [u8; 16]);

/// `IsReparseTagNameSurrogate`: the reparse point names another location.
const NAME_SURROGATE: u32 = 0x2000_0000;

/// What an object is to a walk. A link is a name-surrogate reparse point
/// (a junction, symbolic link or mount point): it is checked or labelled as
/// itself and never followed, as Windows' walk does. Other reparse points,
/// such as cloud-file (OneDrive) placeholders, are the object itself:
/// Windows' walk goes into such a folder, and so must ours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    File,
    Folder,
    Link,
}

/// An object's attributes and reparse tag (0 unless it is a reparse point).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Attributes {
    pub(crate) attributes: u32,
    pub(crate) tag: u32,
}

impl Attributes {
    pub(crate) fn kind(self) -> Kind {
        if self.attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 && self.tag & NAME_SURROGATE != 0 {
            Kind::Link
        } else if self.is_dir() {
            Kind::Folder
        } else {
            Kind::File
        }
    }

    /// A directory object (for inheritance), link or not.
    pub(crate) fn is_dir(self) -> bool {
        self.attributes & FILE_ATTRIBUTE_DIRECTORY != 0
    }

    /// The same kind of object: what a check of the listing still holds for.
    pub(crate) fn same_kind(self, other: Self) -> bool {
        self.kind() == other.kind() && self.is_dir() == other.is_dir()
    }
}

impl Handle {
    /// The handle, now the caller's to close.
    pub(crate) fn into_raw(self) -> HANDLE {
        let handle = self.0;
        std::mem::forget(self);
        handle
    }

    /// Opens `path` with `access`, the link itself if it is one.
    pub(crate) unsafe fn open(path: &Path, access: u32) -> Result<Self> {
        Self::open_with(
            path,
            access,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
        )
    }

    pub(crate) unsafe fn open_with(path: &Path, access: u32, flags: u32) -> Result<Self> {
        let handle = CreateFileW(
            to_wide(path).as_ptr(),
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            flags,
            std::ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            let err = std::io::Error::last_os_error();
            bail!("open {}: {err}", path.display());
        }
        Ok(Self(handle))
    }

    /// Opens the root of a walk with `access`, following links above it
    /// (and other reparse points) as before, but refuses (D352, #1722) a
    /// root that is itself a link: Windows' walk writes such a link itself,
    /// not what it points at, so the caller falls back to that walk. The
    /// handle must be the object checked to be no link, so a link swapped in
    /// meanwhile is refused too.
    pub(crate) unsafe fn open_root(path: &Path, access: u32) -> Result<Self> {
        let itself = Self::open(path, FILE_READ_ATTRIBUTES)?;
        if itself.kind()?.kind() == Kind::Link {
            bail!("{} is a link", path.display());
        }
        #[cfg(test)]
        BETWEEN_ROOT_OPENS.with(|hook| {
            if let Some(hook) = hook.borrow_mut().as_mut() {
                hook(path);
            }
        });
        let root = Self::open_with(path, access, FILE_FLAG_BACKUP_SEMANTICS)?;
        if root.identity()? != itself.identity()? {
            bail!("{} changed while being opened", path.display());
        }
        Ok(root)
    }

    pub(crate) unsafe fn identity(&self) -> Result<FileId> {
        super::file_identity(self.0).map_err(|err| anyhow!("reading a file ID failed: {err}"))
    }

    /// Its attributes and reparse tag.
    pub(crate) unsafe fn kind(&self) -> Result<Attributes> {
        // `FILE_ATTRIBUTE_TAG_INFO`.
        const FILE_ATTRIBUTE_TAG_INFO: i32 = 9;
        let mut info = [0_u32; 2];
        if GetFileInformationByHandleEx(
            self.0,
            FILE_ATTRIBUTE_TAG_INFO,
            info.as_mut_ptr() as *mut c_void,
            std::mem::size_of_val(&info) as u32,
        ) == 0
        {
            bail!("reading the attributes failed: {}", GetLastError());
        }
        Ok(Attributes {
            attributes: info[0],
            tag: if info[0] & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                info[1]
            } else {
                0
            },
        })
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: a handle this type opened.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

/// Run by [`Handle::open_root`] between its two opens (a swap there).
#[cfg(test)]
pub(crate) type RootOpenHook = Option<Box<dyn FnMut(&Path)>>;

#[cfg(test)]
thread_local! {
    pub(crate) static BETWEEN_ROOT_OPENS: std::cell::RefCell<RootOpenHook> =
        const { std::cell::RefCell::new(None) };
}

/// A subfolder (not a link) a walk found and still has to walk, not held open (a
/// folder with many subfolders would otherwise hold a handle for each): the
/// folder it was listed in (held, shared), its name and its identity.
pub(crate) struct Subfolder {
    parent: Arc<Handle>,
    name: Vec<u16>,
    identity: FileId,
}

impl Subfolder {
    /// `name` in `parent`, as `opened` (its handle) is now.
    pub(crate) unsafe fn new(
        parent: &Arc<Handle>,
        name: Vec<u16>,
        opened: &Handle,
    ) -> Result<Self> {
        Ok(Self {
            parent: Arc::clone(parent),
            name,
            identity: opened.identity()?,
        })
    }

    /// Opens it again through its folder for listing (plus `access`),
    /// failing unless it is still the same plain folder.
    pub(crate) unsafe fn reopen(&self, access: u32) -> Result<Handle> {
        let handle = open_relative(self.parent.0, &self.name, access | FILE_LIST_DIRECTORY)?;
        if handle.kind()?.kind() != Kind::Folder || handle.identity()? != self.identity {
            bail!(
                "{} changed while being walked",
                String::from_utf16_lossy(&self.name)
            );
        }
        Ok(handle)
    }
}

#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root_directory: HANDLE,
    object_name: *const UnicodeString,
    attributes: u32,
    security_descriptor: *const c_void,
    security_quality_of_service: *const c_void,
}

#[repr(C)]
struct IoStatusBlock {
    status: usize,
    information: usize,
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtOpenFile(
        file_handle: *mut HANDLE,
        desired_access: u32,
        object_attributes: *const ObjectAttributes,
        io_status_block: *mut IoStatusBlock,
        share_access: u32,
        open_options: u32,
    ) -> i32;
    fn RtlNtStatusToDosError(status: i32) -> u32;
}

const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
const FILE_DIRECTORY_FILE: u32 = 0x1;
const FILE_TRAVERSE: u32 = 0x20;
/// `OBJ_DONT_REPARSE`: the open fails at any reparse point the I/O manager
/// would follow (a junction or symbolic link); one a filter serves (a
/// cloud-file or ProjFS folder) still opens.
const OBJ_DONT_REPARSE: u32 = 0x1000;
const FILE_OPEN_FOR_BACKUP_INTENT: u32 = 0x4000;
const FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_FULL_DIRECTORY_INFO: i32 = 14;
const FILE_FULL_DIRECTORY_RESTART_INFO: i32 = 15;

/// Opens the entry `name` of the open folder `dir` with `access` (plus
/// attribute reads and synchronous I/O), the entry itself if it is a link.
/// The name is matched exactly, as [`list`] returned it.
pub(crate) unsafe fn open_relative(dir: HANDLE, name: &[u16], access: u32) -> Result<Handle> {
    plain_name(name)?;
    nt_open(
        dir,
        name,
        access | FILE_READ_ATTRIBUTES,
        /*object_attributes*/ 0,
        FILE_OPEN_REPARSE_POINT,
    )
    .map_err(|status| {
        anyhow!(
            "opening {} failed: {status:#x}",
            String::from_utf16_lossy(name)
        )
    })
}

/// What [`open_folder_unfollowed`] opens a folder for.
#[derive(Clone, Copy)]
pub(crate) enum FolderAccess {
    /// Listing it.
    List,
    /// Only opening what is inside it (a folder on the way).
    Traverse,
}

/// #1909: opens the folder `name` of the open folder `dir` for `access`
/// (with synchronous I/O), the way a path open would but following no link:
/// a junction or symbolic link (perhaps swapped in after the folder was
/// checked) fails with `ERROR_REPARSE_POINT_ENCOUNTERED`, so nothing behind
/// it, such as a network share, is ever opened; a folder a filter serves (a
/// cloud-file or ProjFS placeholder) opens as that filter presents it.
pub(crate) unsafe fn open_folder_unfollowed(
    dir: HANDLE,
    name: &[u16],
    access: FolderAccess,
) -> std::io::Result<Handle> {
    plain_name(name)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;
    // A folder on the way gets no attribute reads either: nothing reads
    // them, and a folder may grant traverse alone.
    let access = match access {
        FolderAccess::List => FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES,
        FolderAccess::Traverse => FILE_TRAVERSE,
    };
    nt_open(dir, name, access, OBJ_DONT_REPARSE, FILE_DIRECTORY_FILE)
        .map_err(|status| std::io::Error::from_raw_os_error(RtlNtStatusToDosError(status) as i32))
}

/// Fails unless `name` is one plain name: never empty, a path or a stream
/// (`name:stream`).
fn plain_name(name: &[u16]) -> Result<()> {
    let separators = [u16::from(b'\\'), u16::from(b'/'), u16::from(b':')];
    if name.is_empty() || name.iter().any(|c| *c == 0 || separators.contains(c)) {
        bail!("{:?} is not a plain name", String::from_utf16_lossy(name));
    }
    u16::try_from(std::mem::size_of_val(name))
        .map_err(|_| anyhow!("a name of {} characters is too long", name.len()))?;
    Ok(())
}

/// #1909: [`open_relative`] with the error Windows gives (so a missing
/// entry reads as `NotFound`).
pub(crate) unsafe fn open_itself(
    dir: HANDLE,
    name: &[u16],
    access: u32,
) -> std::io::Result<Handle> {
    plain_name(name)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;
    nt_open(
        dir,
        name,
        access | FILE_READ_ATTRIBUTES,
        /*object_attributes*/ 0,
        FILE_OPEN_REPARSE_POINT,
    )
    .map_err(|status| std::io::Error::from_raw_os_error(RtlNtStatusToDosError(status) as i32))
}

/// #1909: the target of the link `name` in the open folder `dir`, as
/// `std::fs::read_link` gives it (an absolute target as `\\?\...`), read
/// from the link itself opened relative to `dir`; `None` for anything but a
/// symbolic link or junction.
pub(crate) unsafe fn link_target_at(dir: HANDLE, name: &[u16]) -> Option<String> {
    use windows_sys::Win32::System::IO::DeviceIoControl;
    const FSCTL_GET_REPARSE_POINT: u32 = 0x0009_00A8;
    const TAG_SYMLINK: u32 = 0xA000_000C;
    const TAG_MOUNT_POINT: u32 = 0xA000_0003;
    const SYMLINK_FLAG_RELATIVE: u32 = 1;
    let link = open_itself(dir, name, /*access*/ 0).ok()?;
    let mut buffer = vec![0_u8; 16 * 1024];
    let mut returned = 0_u32;
    if DeviceIoControl(
        link.0,
        FSCTL_GET_REPARSE_POINT,
        std::ptr::null(),
        0,
        buffer.as_mut_ptr().cast(),
        buffer.len() as u32,
        &mut returned,
        std::ptr::null_mut(),
    ) == 0
    {
        return None;
    }
    let buffer = buffer.get(..returned as usize)?;
    let u16_at = |at: usize| Some(u16::from_le_bytes([*buffer.get(at)?, *buffer.get(at + 1)?]));
    let tag = u32::from_le_bytes(buffer.get(0..4)?.try_into().ok()?);
    let (path_buffer, relative) = match tag {
        TAG_SYMLINK => {
            let flags = u32::from_le_bytes(buffer.get(16..20)?.try_into().ok()?);
            (20, flags & SYMLINK_FLAG_RELATIVE != 0)
        }
        TAG_MOUNT_POINT => (16, false),
        _ => return None,
    };
    let (offset, length) = (usize::from(u16_at(8)?), usize::from(u16_at(10)?));
    let bytes = buffer.get(path_buffer + offset..path_buffer + offset + length)?;
    let mut target: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    // As `read_link`: an absolute target's `\??\` becomes `\\?\`.
    let nt_prefix = [
        u16::from(b'\\'),
        u16::from(b'?'),
        u16::from(b'?'),
        u16::from(b'\\'),
    ];
    if !relative && target.starts_with(&nt_prefix) {
        target[1] = u16::from(b'\\');
    }
    Some(String::from_utf16_lossy(&target))
}

/// `NtOpenFile` of the plain name `name` ([`plain_name`]) in the open folder
/// `dir` with `access` and synchronous I/O; the NTSTATUS when it fails.
unsafe fn nt_open(
    dir: HANDLE,
    name: &[u16],
    access: u32,
    object_attributes: u32,
    options: u32,
) -> std::result::Result<Handle, i32> {
    // STATUS_OBJECT_NAME_INVALID.
    const NAME_INVALID: i32 = 0xC000_0033_u32 as i32;
    let Ok(bytes) = u16::try_from(std::mem::size_of_val(name)) else {
        return Err(NAME_INVALID);
    };
    let mut name_buf = name.to_vec();
    let unicode = UnicodeString {
        length: bytes,
        maximum_length: bytes,
        buffer: name_buf.as_mut_ptr(),
    };
    let attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root_directory: dir,
        object_name: &unicode,
        attributes: object_attributes,
        security_descriptor: std::ptr::null(),
        security_quality_of_service: std::ptr::null(),
    };
    let mut io = IoStatusBlock {
        status: 0,
        information: 0,
    };
    let mut handle: HANDLE = std::ptr::null_mut();
    let status = NtOpenFile(
        &mut handle,
        access | SYNCHRONIZE,
        &attributes,
        &mut io,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_FOR_BACKUP_INTENT | options,
    );
    if status != 0 || handle.is_null() {
        if !handle.is_null() {
            CloseHandle(handle);
        }
        return Err(if status != 0 { status } else { NAME_INVALID });
    }
    Ok(Handle(handle))
}

/// The entries of the folder open as `dir` (with list access and
/// synchronous I/O) as (name, attributes and reparse tag), without `.` and
/// `..`. A link's attributes are its own.
pub(crate) unsafe fn list(dir: HANDLE) -> Result<Vec<Listed>> {
    list_at_most(dir, usize::MAX).map(|(entries, _)| entries)
}

/// An entry [`list`] found: its name, attributes and reparse tag.
pub(crate) type Listed = (Vec<u16>, Attributes);

/// #1877 item 4: [`list`], no more than `at_most` entries; the flag says
/// the folder holds more (listing stops there).
pub(crate) unsafe fn list_at_most(dir: HANDLE, at_most: usize) -> Result<(Vec<Listed>, bool)> {
    // `FILE_FULL_DIR_INFO`'s offsets; `EaSize` holds a reparse point's tag.
    const NEXT: usize = 0;
    const ATTRIBUTES: usize = 56;
    const NAME_BYTES: usize = 60;
    const EA_SIZE: usize = 64;
    const NAME: usize = 68;
    let mut out = Vec::new();
    let mut buffer = vec![0_u64; 8192];
    let mut class = FILE_FULL_DIRECTORY_RESTART_INFO;
    loop {
        if GetFileInformationByHandleEx(
            dir,
            class,
            buffer.as_mut_ptr() as *mut c_void,
            (buffer.len() * 8) as u32,
        ) == 0
        {
            let err = GetLastError();
            if err == ERROR_NO_MORE_FILES {
                return Ok((out, false));
            }
            bail!("listing a folder failed: {err}");
        }
        class = FILE_FULL_DIRECTORY_INFO;
        let base = buffer.as_ptr() as *const u8;
        let mut offset = 0_usize;
        loop {
            let entry = base.add(offset);
            let next = std::ptr::read_unaligned(entry.add(NEXT) as *const u32) as usize;
            let attributes = std::ptr::read_unaligned(entry.add(ATTRIBUTES) as *const u32);
            let name_len =
                std::ptr::read_unaligned(entry.add(NAME_BYTES) as *const u32) as usize / 2;
            let name: Vec<u16> = (0..name_len)
                .map(|i| std::ptr::read_unaligned(entry.add(NAME + 2 * i) as *const u16))
                .collect();
            let dot = u16::from(b'.');
            let tag = if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                std::ptr::read_unaligned(entry.add(EA_SIZE) as *const u32)
            } else {
                0
            };
            if name != [dot] && name != [dot, dot] {
                if out.len() == at_most {
                    return Ok((out, true));
                }
                out.push((name, Attributes { attributes, tag }));
            }
            if next == 0 {
                break;
            }
            offset += next;
        }
    }
}

#[cfg(test)]
#[path = "acl_walk_tests.rs"]
mod tests;
