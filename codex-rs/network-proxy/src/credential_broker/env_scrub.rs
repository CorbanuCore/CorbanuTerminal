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
//! - Windows: the C runtimes in the process copy the environment at start-up,
//!   keeping some copies and freeing others unwiped (later reused, partly
//!   overwritten), so every copy of the value in the process heaps is
//!   overwritten too. A copy in another form, or outside the heaps (a thread
//!   stack), is not found. A heap its owner created with
//!   `HEAP_NO_SERIALIZE` is not locked by `HeapLock`, so walking it races
//!   with that owner.

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
/// runtime's tables and in other copies on the heap. `None` when unset or empty
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
    windows_env::remove_from_c_runtime(&wide_name);
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
    // The C runtime made more copies of the launch environment at start-up
    // (some freed without being wiped, some kept); so may other code have.
    let started = std::time::Instant::now();
    // This function's own buffers keep their copies until they are dropped.
    let own = [
        windows_env::allocation(value.as_ptr().cast(), value.capacity() * 2),
        windows_env::allocation(wide.as_ptr(), wide.capacity()),
        windows_env::allocation(utf8.as_ptr(), utf8.capacity()),
    ];
    // An invalid value's UTF-8 holds only a decoded prefix, which must not
    // be swept for.
    let patterns: &[&[u8]] = if valid {
        &[wide.as_slice(), utf8.as_slice()]
    } else {
        &[wide.as_slice()]
    };
    windows_env::wipe_heap_copies(name, patterns, &own);
    // An environment block Windows replaced when the environment grew stays
    // in the process parameters' allocation, with its `NAME=value` entries.
    if let Some(base) = windows_env::process_parameters_allocation() {
        windows_env::wipe_entries_in_allocation(base, name, patterns);
    }
    tracing::debug!(
        "environment sweep for one handed-over variable took {} ms",
        started.elapsed().as_millis()
    );
    valid.then_some(utf8)
}

/// The address of this process's parameters (diagnostics for a failing
/// memory scan).
#[cfg(all(test, windows))]
pub(crate) fn process_parameters_allocation_for_test() -> Option<usize> {
    windows_env::process_parameters_allocation()
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
        /// Sets (an empty value: removes) a variable in the C runtime's
        /// tables and the process environment.
        fn _wputenv_s(name: *const u16, value: *const u16) -> i32;
    }

    /// Drops the (already overwritten) entry from the C runtime's tables, so
    /// C `getenv` and the C runtime's spawn functions no longer see it.
    pub(super) fn remove_from_c_runtime(wide_name: &[u16]) {
        let empty = [0_u16];
        // SAFETY: both strings are NUL-terminated.
        let error = unsafe { _wputenv_s(wide_name.as_ptr(), empty.as_ptr()) };
        if error != 0 {
            // The entry stays, overwritten with `0` characters.
            tracing::debug!("C runtime environment entry not removed (errno {error})");
        }
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

    /// The allocation holding this process's parameters (`PEB`
    /// `ProcessParameters`). The launch environment block is part of it, so
    /// when the environment grows and Windows moves it, the old block (with
    /// every launch variable) is never freed.
    pub(super) fn process_parameters_allocation() -> Option<usize> {
        use windows_sys::Wdk::System::Threading::NtQueryInformationProcess;
        use windows_sys::Wdk::System::Threading::ProcessBasicInformation;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        use windows_sys::Win32::System::Threading::PROCESS_BASIC_INFORMATION;
        // SAFETY: zeroed POD out-structure of the size passed.
        let mut info: PROCESS_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let mut len = 0_u32;
        // SAFETY: queries this process into `info`.
        let status = unsafe {
            NtQueryInformationProcess(
                GetCurrentProcess(),
                ProcessBasicInformation,
                (&mut info as *mut PROCESS_BASIC_INFORMATION).cast(),
                std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
                &mut len,
            )
        };
        if status != 0 || info.PebBaseAddress.is_null() {
            return None;
        }
        // SAFETY: this process's PEB, which lives as long as the process.
        let parameters = unsafe { (*info.PebBaseAddress).ProcessParameters } as usize;
        (parameters != 0).then_some(parameters)
    }

    /// Gives every `name=value` entry (narrow or UTF-16, the name compared
    /// ASCII case-insensitively) in the committed,
    /// writable regions of the allocation containing `address` `0`
    /// characters for its value. Memory is read with `ReadProcessMemory` and
    /// written with `WriteProcessMemory`, which fail instead of faulting if a
    /// region changes meanwhile; only whole entries are written.
    pub(super) fn wipe_entries_in_allocation(address: usize, name: &str, values: &[&[u8]]) {
        use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
        use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
        use windows_sys::Win32::System::Memory::MEM_COMMIT;
        use windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
        use windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE;
        use windows_sys::Win32::System::Memory::PAGE_GUARD;
        use windows_sys::Win32::System::Memory::PAGE_READWRITE;
        use windows_sys::Win32::System::Memory::VirtualQuery;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        const CHUNK: usize = 1 << 20;
        let entries: Vec<(Zeroizing<Vec<u8>>, Vec<u8>, usize)> = values
            .iter()
            .filter(|value| value.len() >= 8)
            .map(|value| {
                let wide = value.len() > 1 && value[1] == 0;
                let prefix: Vec<u8> = if wide {
                    format!("{name}=")
                        .encode_utf16()
                        .flat_map(u16::to_le_bytes)
                        .collect()
                } else {
                    format!("{name}=").into_bytes()
                };
                let zeros: Vec<u8> = if wide {
                    std::iter::repeat_n([b'0', 0], value.len() / 2)
                        .flatten()
                        .collect()
                } else {
                    vec![b'0'; value.len()]
                };
                let mut entry = Zeroizing::new(prefix.clone());
                entry.extend_from_slice(value);
                (entry, zeros, prefix.len())
            })
            .collect();
        let Some(longest) = entries.iter().map(|(entry, _, _)| entry.len()).max() else {
            return;
        };
        let query = |at: usize| {
            // SAFETY: zeroed POD out-structure; queries this process.
            let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
            let written = unsafe {
                VirtualQuery(
                    at as *const std::ffi::c_void,
                    &mut info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            (written != 0).then_some(info)
        };
        let Some(first) = query(address) else {
            return;
        };
        // The committed, writable regions of the allocation.
        let allocation_base = first.AllocationBase as usize;
        let mut regions = Vec::new();
        let mut address = allocation_base;
        while let Some(info) = query(address) {
            let base = info.BaseAddress as usize;
            let Some(end) = base.checked_add(info.RegionSize) else {
                break;
            };
            if info.AllocationBase as usize != allocation_base || end <= address {
                break;
            }
            if info.State == MEM_COMMIT
                && info.Protect & (PAGE_READWRITE | PAGE_EXECUTE_READWRITE) != 0
                && info.Protect & PAGE_GUARD == 0
            {
                regions.push((base, end));
            }
            address = end;
        }
        let Some(largest) = regions.iter().map(|(start, end)| end - start).max() else {
            return;
        };
        let chunk = largest.min(CHUNK);
        let mut buffer = Zeroizing::new(vec![0_u8; chunk + longest]);
        // The search buffers hold entries themselves.
        let mut own: Vec<(usize, usize)> = entries
            .iter()
            .map(|(entry, _, _)| allocation(entry.as_ptr(), entry.capacity()))
            .collect();
        own.push(allocation(buffer.as_ptr(), buffer.capacity()));
        // SAFETY: a pseudo-handle for this process.
        let process = unsafe { GetCurrentProcess() };
        for (base, end) in regions {
            let mut offset = base;
            while offset < end {
                // Chunks overlap by the longest entry so none is split.
                let want = (end - offset).min(buffer.len());
                let mut read = 0_usize;
                // SAFETY: reads into `buffer`, which holds `want` bytes;
                // fails instead of faulting on memory released meanwhile.
                let ok = unsafe {
                    ReadProcessMemory(
                        process,
                        offset as *const std::ffi::c_void,
                        buffer.as_mut_ptr().cast(),
                        want,
                        &mut read,
                    )
                };
                if ok == 0 && read == 0 {
                    break;
                }
                for (entry, zeros, prefix) in &entries {
                    let mut at = 0;
                    while at + entry.len() <= read {
                        // The name compared ASCII case-insensitively, as
                        // Windows does; the value exactly.
                        let matches = buffer[at].eq_ignore_ascii_case(&entry[0])
                            && buffer[at..at + prefix].eq_ignore_ascii_case(&entry[..*prefix])
                            && buffer[at + prefix..at + entry.len()] == entry[*prefix..];
                        if !matches {
                            at += 1;
                            continue;
                        }
                        let target = offset + at + prefix;
                        if own
                            .iter()
                            .any(|(start, end)| target < *end && target + zeros.len() > *start)
                        {
                            at += entry.len();
                            continue;
                        }
                        // SAFETY: writes `zeros` over the value at `target`,
                        // just read; fails on memory released meanwhile.
                        unsafe {
                            WriteProcessMemory(
                                process,
                                target as *const std::ffi::c_void,
                                zeros.as_ptr().cast(),
                                zeros.len(),
                                std::ptr::null_mut(),
                            )
                        };
                        at += entry.len();
                    }
                }
                buffer[..read].fill(0);
                if want < buffer.len() {
                    break;
                }
                offset += chunk;
            }
        }
    }

    /// Shortest value (in characters) whose bare copies are overwritten in
    /// blocks in use; real provider keys are much longer.
    pub(super) const MIN_BARE_CHARS: usize = 20;

    /// The address range `[start, start + len)` of an allocation.
    pub(super) fn allocation(start: *const u8, len: usize) -> (usize, usize) {
        (start as usize, start as usize + len)
    }

    /// Overwrites copies of a handed-over value in this process's heaps,
    /// except inside the `keep` ranges (the caller's own buffers):
    /// - a `name=value` environment entry (narrow or UTF-16, the name
    ///   compared ASCII case-insensitively), and any occurrence in a free
    ///   block, from 8 characters;
    /// - any other occurrence in a block in use (a stale copy in reused
    ///   memory, such as the C runtime's freed start-up copies, or a copy Core
    ///   must not keep), from [`MIN_BARE_CHARS`] characters, so placeholder
    ///   values (`dummy-key`) are not hunted through the process.
    ///
    /// Every match gets `0` characters (UTF-16: `0` units), never NUL, so a
    /// live string stays the same length, valid and terminated where it was.
    /// `values` holds the value's encodings (UTF-16LE, UTF-8). Each heap is locked while it is walked and nothing
    /// is allocated meanwhile. Only bytes that equal a whole value are
    /// written, which no heap metadata can be, so the heap stays consistent.
    pub(super) fn wipe_heap_copies(name: &str, values: &[&[u8]], keep: &[(usize, usize)]) {
        use windows_sys::Win32::System::Memory::GetProcessHeaps;
        use windows_sys::Win32::System::Memory::HeapLock;
        use windows_sys::Win32::System::Memory::HeapUnlock;
        use windows_sys::Win32::System::Memory::HeapWalk;
        use windows_sys::Win32::System::Memory::PROCESS_HEAP_ENTRY;
        // `PROCESS_HEAP_*` entry flags (`winbase.h`).
        const PROCESS_HEAP_REGION: u32 = 0x1;
        const PROCESS_HEAP_UNCOMMITTED_RANGE: u32 = 0x2;
        const PROCESS_HEAP_ENTRY_BUSY: u32 = 0x4;
        // Allocated before any heap is locked: nothing is allocated (and no
        // heap changes shape) during a walk.
        let narrow_name: Vec<u8> = format!("{name}=").into_bytes();
        let wide_name: Vec<u8> = format!("{name}=")
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let patterns: Vec<Pattern<'_>> = values
            .iter()
            .filter(|value| value.len() >= 8)
            .map(|value| Pattern {
                // UTF-16LE values have a zero high byte for ASCII.
                prefix: if value.len() > 1 && value[1] == 0 {
                    &wide_name
                } else {
                    &narrow_name
                },
                value,
            })
            .collect();
        if patterns.is_empty() {
            return;
        }
        // SAFETY: a size query, then a fill of at most `heaps.len()` handles.
        let mut heaps = vec![0; unsafe { GetProcessHeaps(0, std::ptr::null_mut()) } as usize + 8];
        let count = unsafe { GetProcessHeaps(heaps.len() as u32, heaps.as_mut_ptr()) } as usize;
        heaps.truncate(count.min(heaps.len()));
        let mut committed = CommittedPages::default();
        for heap in heaps {
            // SAFETY: a heap of this process; unlocked below.
            if unsafe { HeapLock(heap) } == 0 {
                continue;
            }
            // SAFETY: zeroed POD; a null `lpData` starts the walk.
            let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
            // SAFETY: the heap is locked, so the walk and the blocks it
            // reports stay valid until `HeapUnlock`.
            while unsafe { HeapWalk(heap, &mut entry) } != 0 {
                let flags = u32::from(entry.wFlags);
                if flags & (PROCESS_HEAP_REGION | PROCESS_HEAP_UNCOMMITTED_RANGE) != 0
                    || entry.lpData.is_null()
                {
                    continue;
                }
                let free = flags & PROCESS_HEAP_ENTRY_BUSY == 0;
                let start = entry.lpData as usize;
                let end = start.saturating_add(entry.cbData as usize);
                // Large free blocks may be partly decommitted: only the
                // committed, writable pages are read.
                committed.for_each_writable(start, end, |from, to| {
                    for pattern in &patterns {
                        // SAFETY: committed, writable bytes of a heap block.
                        unsafe { pattern.wipe(from as *mut u8, to - from, free, keep) };
                    }
                });
            }
            // SAFETY: locked above.
            unsafe { HeapUnlock(heap) };
        }
    }

    /// One encoding of a value and of the `name=` that precedes it in an
    /// environment entry.
    struct Pattern<'a> {
        prefix: &'a [u8],
        value: &'a [u8],
    }

    impl Pattern<'_> {
        /// Gives every `prefix value` entry and, in a free block or when the
        /// value is long enough, every other occurrence of the value `0`
        /// characters, in the `len` bytes at `data`, outside the `keep` ranges. Other threads may use a busy block
        /// meanwhile, so the bytes are only ever read and written through
        /// volatile raw-pointer accesses, never as a Rust slice.
        ///
        /// # Safety
        ///
        /// `data..data + len` must be committed, writable memory.
        unsafe fn wipe(&self, data: *mut u8, len: usize, free: bool, keep: &[(usize, usize)]) {
            let wide = self.value.len() > 1 && self.value[1] == 0;
            let characters = if wide {
                self.value.len() / 2
            } else {
                self.value.len()
            };
            let bare = free || characters >= MIN_BARE_CHARS;
            // SAFETY: (caller) `index < len` is committed and readable.
            let byte = |index: usize| unsafe { std::ptr::read_volatile(data.add(index)) };
            let equal = |at: usize, wanted: &[u8], fold: bool| {
                wanted.iter().enumerate().all(|(offset, want)| {
                    let have = byte(at + offset);
                    if fold {
                        have.eq_ignore_ascii_case(want)
                    } else {
                        have == *want
                    }
                })
            };
            let mut at = 0;
            while at + self.value.len() <= len {
                if byte(at) != self.value[0] || !equal(at, self.value, false) {
                    at += 1;
                    continue;
                }
                let address = data as usize + at;
                if !keep
                    .iter()
                    .any(|(start, end)| (*start..*end).contains(&address))
                {
                    let entry =
                        at >= self.prefix.len() && equal(at - self.prefix.len(), self.prefix, true);
                    if entry || bare {
                        for index in 0..self.value.len() {
                            let replacement = if wide && index % 2 == 1 { 0 } else { b'0' };
                            // SAFETY: (caller) within the committed, writable range.
                            unsafe { std::ptr::write_volatile(data.add(at + index), replacement) };
                        }
                    }
                }
                at += self.value.len();
            }
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
