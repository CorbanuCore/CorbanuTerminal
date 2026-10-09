//! PF-27-S05: take a provider key out of this process's environment.
//!
//! Removing a variable is not enough: the original `NAME=value` bytes stay
//! where they were. On Unix that is the launch environment on the main stack
//! (which `ps -E` and `/proc/<pid>/environ` read) or a `setenv` heap string;
//! on Windows (PF-27-S09) it is the process environment block (which another
//! process reads as this process's environment) and the C runtime's copies of
//! the environment. The value bytes of every live entry are overwritten in
//! place before the variable is removed.
//!
//! The walk of `environ` cannot take Rust's private environment lock, so a
//! concurrent `setenv` could reallocate the array (or `unsetenv` free an
//! entry) under it and crash the process. Environment writes in this crate
//! therefore go through [`env_write_lock`], which the walk holds too.
//!
//! Known limits (recorded in the PF-27-S05 and PF-27-S09 sprint records):
//! - This runs once a session config enables the broker, when Core already
//!   has other threads. The walk of `environ` and `unsetenv` are not
//!   serialized with C-level `getenv` callers (resolver, TLS setup), or with
//!   a concurrent `setenv` from outside this crate that reallocates the
//!   array. Doing it before `main` (in `arg0`) needs the flag decided at
//!   process start.
//! - A launch-environment value that `.env` loading already replaced is no
//!   longer reachable through `environ`; its original bytes stay in the
//!   launch block.

#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt as _;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt as _;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
use zeroize::Zeroizing;

static ENV_WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Serializes environment writes in this crate with [`take_env_var`]'s walk
/// of `environ`. Not reentrant: release it before calling `take_env_var`.
pub(crate) fn env_write_lock() -> MutexGuard<'static, ()> {
    ENV_WRITE_LOCK
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Returns the value of `name` and removes it from the environment, with its
/// bytes overwritten. `None` when unset or empty (nothing is changed).
#[cfg(unix)]
pub(crate) fn take_env_var(name: &str) -> Option<Zeroizing<Vec<u8>>> {
    if name.is_empty() || name.contains(['=', '\0']) {
        return None;
    }
    let _writes = env_write_lock();
    let value = Zeroizing::new(std::env::var_os(name)?.into_vec());
    if value.is_empty() {
        return None;
    }
    overwrite_in_place(OsStr::new(name).as_bytes());
    // SAFETY: Rust's own environment accessors serialize with this call and
    // this crate's writers hold `env_write_lock`. A C-level reader or an
    // outside writer racing it is the known limit in the module docs.
    unsafe { std::env::remove_var(name) };
    Some(value)
}

/// Sets a variable for a test, serialized with [`take_env_var`].
#[cfg(test)]
pub(crate) fn set_env_var_for_test(name: &str, value: &str) {
    let _writes = env_write_lock();
    // SAFETY: a test-unique variable; `env_write_lock` keeps this `setenv`
    // from reallocating `environ` under `take_env_var`'s walk.
    unsafe { std::env::set_var(name, value) };
}

/// Overwrites the value bytes of every `name=` entry with `0` characters.
#[cfg(unix)]
fn overwrite_in_place(name: &[u8]) {
    // SAFETY: `environ` is a null-terminated array of NUL-terminated strings
    // owned by the C library. Only bytes inside an entry's existing value
    // (before its terminating NUL) are written, so every entry stays a valid
    // C string of the same length.
    unsafe {
        let mut entry = environ();
        if entry.is_null() {
            return;
        }
        while !(*entry).is_null() {
            let bytes = (*entry).cast::<u8>();
            let len = libc::strlen(*entry);
            if len > name.len()
                && std::slice::from_raw_parts(bytes, name.len()) == name
                && *bytes.add(name.len()) == b'='
            {
                for offset in name.len() + 1..len {
                    std::ptr::write_volatile(bytes.add(offset), b'0');
                }
            }
            entry = entry.add(1);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn environ() -> *mut *mut libc::c_char {
    // SAFETY: `_NSGetEnviron` always returns a valid pointer to `environ`.
    unsafe { *libc::_NSGetEnviron() }
}

#[cfg(all(unix, not(target_os = "macos")))]
unsafe fn environ() -> *mut *mut libc::c_char {
    unsafe extern "C" {
        static mut environ: *mut *mut libc::c_char;
    }
    // SAFETY: reads the C library's `environ` pointer.
    unsafe { environ }
}

/// Returns the value of `name` (UTF-8) and removes it from the environment,
/// with its bytes overwritten in the process environment block, in the C
/// runtime's copies and in freed heap blocks. `None` when unset or empty
/// (nothing is changed), or when the value is not valid Unicode (it is still
/// removed and overwritten).
#[cfg(windows)]
pub(crate) fn take_env_var(name: &str) -> Option<Zeroizing<Vec<u8>>> {
    if name.is_empty() || name.contains(['=', '\0']) {
        return None;
    }
    let _writes = env_write_lock();
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let value = windows_env::read(&wide_name)?;
    if value.is_empty() {
        return None;
    }
    windows_env::overwrite_in_place(name, &wide_name, value.len());
    // SAFETY: Rust's own environment accessors serialize with this call and
    // this crate's writers hold `env_write_lock`.
    unsafe { std::env::remove_var(name) };
    let mut utf8 = Zeroizing::new(Vec::with_capacity(value.len() * 3));
    let mut valid = true;
    for unit in char::decode_utf16(value.iter().copied()) {
        let Ok(unit) = unit else {
            valid = false;
            break;
        };
        let mut buffer = [0_u8; 4];
        utf8.extend_from_slice(unit.encode_utf8(&mut buffer).as_bytes());
        buffer.fill(0);
    }
    let mut wide = Zeroizing::new(Vec::with_capacity(value.len() * 2));
    for unit in value.iter() {
        wide.extend_from_slice(&unit.to_le_bytes());
    }
    // The C runtime and the loader copied the launch environment at start-up
    // and freed the copies without wiping them.
    windows_env::wipe_freed_heap_copies(&[wide.as_slice(), utf8.as_slice()]);
    valid.then_some(utf8)
}

/// PF-27-S09: the Windows environment, read and overwritten without copies
/// that outlive the call.
#[cfg(windows)]
mod windows_env {
    use std::ffi::c_char;
    use windows_sys::Win32::System::Environment::GetEnvironmentVariableW;
    use windows_sys::Win32::System::Environment::SetEnvironmentVariableW;
    use zeroize::Zeroizing;

    unsafe extern "C" {
        /// The C runtime's narrow environment table (`_environ`).
        fn __p__environ() -> *mut *mut *mut c_char;
        /// The C runtime's wide environment table (`_wenviron`).
        fn __p__wenviron() -> *mut *mut *mut u16;
    }

    /// The value of the NUL-terminated `name`, read into a buffer that is
    /// wiped on drop (`std::env::var_os` would leave copies behind).
    pub(super) fn read(name: &[u16]) -> Option<Zeroizing<Vec<u16>>> {
        let mut buffer = Zeroizing::new(Vec::<u16>::new());
        loop {
            let capacity = u32::try_from(buffer.len()).ok()?;
            // SAFETY: `name` is NUL-terminated; `buffer` holds `capacity`
            // units (a null pointer with 0 asks for the size).
            let needed = unsafe {
                GetEnvironmentVariableW(
                    name.as_ptr(),
                    if capacity == 0 {
                        std::ptr::null_mut()
                    } else {
                        buffer.as_mut_ptr()
                    },
                    capacity,
                )
            } as usize;
            if needed == 0 {
                // Unset (or empty: nothing to take either way).
                return None;
            }
            if needed < buffer.len() {
                buffer.truncate(needed);
                return Some(buffer);
            }
            // Too small: `needed` includes the terminating NUL. Grow into a
            // fresh buffer so no unwiped copy is left by a reallocation.
            buffer = Zeroizing::new(vec![0_u16; needed]);
        }
    }

    /// Replaces the value with as many `0` characters as it has UTF-16
    /// units, which Windows writes over the old value inside the process
    /// environment block (same size: nothing moves), then overwrites every
    /// `name=` entry in the C runtime's environment tables.
    pub(super) fn overwrite_in_place(name: &str, wide_name: &[u16], len: usize) {
        let zeros: Vec<u16> = std::iter::repeat_n(u16::from(b'0'), len)
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: both strings are NUL-terminated.
        unsafe { SetEnvironmentVariableW(wide_name.as_ptr(), zeros.as_ptr()) };
        // SAFETY: the C runtime returns pointers to its table pointers; a
        // table that was never created is null. Each entry is NUL-terminated
        // and only bytes before its terminator are written, so every entry
        // stays a valid string of the same length.
        unsafe {
            overwrite_table(*__p__environ(), name.as_bytes(), |entry: *mut c_char| {
                entry.cast::<u8>()
            });
            let wide: Vec<u16> = name.encode_utf16().collect();
            overwrite_table(*__p__wenviron(), &wide, |entry: *mut u16| entry);
        }
    }

    /// Overwrites every occurrence of a needle (at least 8 bytes) inside the
    /// free blocks of this process's heaps: copies freed without being wiped
    /// (the C runtime's start-up copies of the environment, for one). Each
    /// heap is locked while it is walked. Only bytes inside a free block's
    /// data that equal a whole needle are written, which no heap metadata can
    /// be, so the heap stays consistent.
    pub(super) fn wipe_freed_heap_copies(needles: &[&[u8]]) {
        use windows_sys::Win32::System::Memory::GetProcessHeaps;
        use windows_sys::Win32::System::Memory::HeapLock;
        use windows_sys::Win32::System::Memory::HeapUnlock;
        use windows_sys::Win32::System::Memory::HeapWalk;
        use windows_sys::Win32::System::Memory::PROCESS_HEAP_ENTRY;
        // `PROCESS_HEAP_*` entry flags (`winbase.h`).
        const PROCESS_HEAP_REGION: u32 = 0x1;
        const PROCESS_HEAP_UNCOMMITTED_RANGE: u32 = 0x2;
        const PROCESS_HEAP_ENTRY_BUSY: u32 = 0x4;
        let needles: Vec<&[u8]> = needles
            .iter()
            .copied()
            .filter(|needle| needle.len() >= 8)
            .collect();
        if needles.is_empty() {
            return;
        }
        // Allocated before any heap is locked: nothing is allocated (and no
        // heap changes shape) during a walk.
        // SAFETY: a size query, then a fill of at most `heaps.len()` handles.
        let mut heaps = vec![0; unsafe { GetProcessHeaps(0, std::ptr::null_mut()) } as usize + 8];
        let count = unsafe { GetProcessHeaps(heaps.len() as u32, heaps.as_mut_ptr()) } as usize;
        heaps.truncate(count.min(heaps.len()));
        let skip = PROCESS_HEAP_ENTRY_BUSY | PROCESS_HEAP_REGION | PROCESS_HEAP_UNCOMMITTED_RANGE;
        let mut committed = CommittedPages::default();
        for heap in heaps {
            // SAFETY: a heap of this process; unlocked below.
            if unsafe { HeapLock(heap) } == 0 {
                continue;
            }
            // SAFETY: zeroed POD; a null `lpData` starts the walk.
            let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
            // SAFETY: the heap is locked, so the walk and the free blocks it
            // reports stay valid until `HeapUnlock`.
            while unsafe { HeapWalk(heap, &mut entry) } != 0 {
                if u32::from(entry.wFlags) & skip != 0 || entry.lpData.is_null() {
                    continue;
                }
                // Large free blocks may be partly decommitted: only the
                // committed, writable pages are read.
                let start = entry.lpData as usize;
                let end = start.saturating_add(entry.cbData as usize);
                // Nothing is allocated during the walk.
                committed.for_each_writable(start, end, |from, to| {
                    // SAFETY: committed, writable bytes of a free block.
                    let data =
                        unsafe { std::slice::from_raw_parts_mut(from as *mut u8, to - from) };
                    for needle in &needles {
                        wipe(data, needle);
                    }
                });
            }
            // SAFETY: locked above.
            unsafe { HeapUnlock(heap) };
        }
    }

    /// The last region `VirtualQuery` reported, so blocks in one region
    /// cost one query.
    #[derive(Default)]
    struct CommittedPages {
        start: usize,
        end: usize,
        writable: bool,
    }

    impl CommittedPages {
        /// Calls `visit` with each part of `[start, end)` that is committed
        /// and writable.
        fn for_each_writable(
            &mut self,
            mut start: usize,
            end: usize,
            mut visit: impl FnMut(usize, usize),
        ) {
            use windows_sys::Win32::System::Memory::MEM_COMMIT;
            use windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
            use windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE;
            use windows_sys::Win32::System::Memory::PAGE_GUARD;
            use windows_sys::Win32::System::Memory::PAGE_READWRITE;
            use windows_sys::Win32::System::Memory::VirtualQuery;
            while start < end {
                if !(self.start..self.end).contains(&start) {
                    // SAFETY: zeroed POD out-structure.
                    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
                    // SAFETY: queries this process's address space.
                    let written = unsafe {
                        VirtualQuery(
                            start as *const std::ffi::c_void,
                            &mut info,
                            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                        )
                    };
                    let region_end = (info.BaseAddress as usize).saturating_add(info.RegionSize);
                    if written == 0 || region_end <= start {
                        break;
                    }
                    *self = Self {
                        start: info.BaseAddress as usize,
                        end: region_end,
                        writable: info.State == MEM_COMMIT
                            && info.Protect & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE) != 0
                            && info.Protect & PAGE_GUARD == 0,
                    };
                }
                let to = end.min(self.end);
                if self.writable {
                    visit(start, to);
                }
                start = to;
            }
        }
    }

    /// Zeroes every occurrence of `needle` in `data`.
    fn wipe(data: &mut [u8], needle: &[u8]) {
        let mut start = 0;
        while start + needle.len() <= data.len() {
            if data[start..start + needle.len()] == *needle {
                for byte in &mut data[start..start + needle.len()] {
                    // SAFETY: a valid byte of the block.
                    unsafe { std::ptr::write_volatile(byte, 0) };
                }
                start += needle.len();
            } else {
                start += 1;
            }
        }
    }

    /// Overwrites the value of each `name=` entry (names compared ASCII
    /// case-insensitively, as Windows does) in a null-terminated table.
    unsafe fn overwrite_table<C, U>(table: *mut *mut C, name: &[U], unit: impl Fn(*mut C) -> *mut U)
    where
        U: Copy + Into<u32> + From<u8>,
    {
        if table.is_null() {
            return;
        }
        let fold = |value: u32| {
            if (u32::from(b'a')..=u32::from(b'z')).contains(&value) {
                value - 32
            } else {
                value
            }
        };
        // SAFETY: (caller) a null-terminated table of NUL-terminated strings.
        unsafe {
            let mut entry = table;
            while !(*entry).is_null() {
                let start = unit(*entry);
                let matches = name.iter().enumerate().all(|(index, wanted)| {
                    let have: u32 = (*start.add(index)).into();
                    have != 0 && fold(have) == fold((*wanted).into())
                }) && (*start.add(name.len())).into() == u32::from(b'=');
                if matches {
                    let mut offset = name.len() + 1;
                    while (*start.add(offset)).into() != 0 {
                        std::ptr::write_volatile(start.add(offset), U::from(b'0'));
                        offset += 1;
                    }
                }
                entry = entry.add(1);
            }
        }
    }
}

#[cfg(test)]
#[path = "env_scrub_tests.rs"]
mod tests;
