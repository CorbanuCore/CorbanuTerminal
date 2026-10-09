//! PF-27-S05 test support: a best-effort scan of this process's writable
//! memory (heap, stacks including the launch environment, anonymous maps)
//! for a needle that is never materialized in plain form here.
//!
//! The needle is held masked (`byte ^ MASK`) and unmasked only in registers
//! during comparison, so the scanner cannot find its own copy. The chunk
//! buffer is zeroed after each comparison for the same reason.

pub(crate) const MASK: u8 = 0x5a;
const CHUNK: usize = 1 << 20;

pub(crate) fn mask(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().map(|byte| byte ^ MASK).collect()
}

/// Number of places the unmasked needle occurs in writable memory.
pub(crate) fn count_in_writable_memory(masked: &[u8]) -> usize {
    assert!(masked.len() >= 8, "needle too short to be meaningful");
    let mut buffer = vec![0_u8; CHUNK + masked.len()];
    let mut hits = 0;
    for (start, len) in writable_regions() {
        let mut offset = 0;
        while offset < len {
            // Overlap chunks by the needle length so no match is split.
            let want = (len - offset).min(CHUNK + masked.len() - 1);
            let read = read_own_memory(start + offset, &mut buffer[..want]);
            if read >= masked.len() {
                hits += count(&buffer[..read], masked);
            }
            buffer.iter_mut().for_each(|byte| *byte = 0);
            if want <= CHUNK {
                break;
            }
            offset += CHUNK;
        }
    }
    hits
}

/// PF-27-S09 diagnostics: where the needle occurs (address, and on Windows
/// the region and heap block holding it), for a failing scan.
#[cfg(windows)]
pub(crate) fn locate_in_writable_memory(masked: &[u8]) -> Vec<String> {
    let mut buffer = vec![0_u8; CHUNK + masked.len()];
    let mut found = Vec::new();
    for (start, len) in writable_regions() {
        let mut offset = 0;
        while offset < len {
            let want = (len - offset).min(CHUNK + masked.len() - 1);
            let read = read_own_memory(start + offset, &mut buffer[..want]);
            if read >= masked.len() {
                for (index, window) in buffer[..read].windows(masked.len()).enumerate() {
                    if window
                        .iter()
                        .zip(masked)
                        .all(|(byte, masked)| byte ^ MASK == *masked)
                    {
                        found.push(start + offset + index);
                    }
                }
            }
            buffer.iter_mut().for_each(|byte| *byte = 0);
            if want <= CHUNK {
                break;
            }
            offset += CHUNK;
        }
    }
    found.into_iter().map(describe).collect()
}

#[cfg(windows)]
fn describe(address: usize) -> String {
    use windows_sys::Win32::System::Memory::GetProcessHeaps;
    use windows_sys::Win32::System::Memory::HeapWalk;
    use windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
    use windows_sys::Win32::System::Memory::PROCESS_HEAP_ENTRY;
    use windows_sys::Win32::System::Memory::VirtualQuery;
    // SAFETY: zeroed POD out-structure; queries this process.
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    unsafe {
        VirtualQuery(
            address as *const std::ffi::c_void,
            &mut info,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    let mut heaps = vec![0; 64];
    // SAFETY: fills at most 64 handles.
    let count = unsafe { GetProcessHeaps(64, heaps.as_mut_ptr()) } as usize;
    let mut block = String::from("no heap block");
    for (index, heap) in heaps.into_iter().take(count).enumerate() {
        // SAFETY: zeroed POD; walks a heap of this process (unlocked: a
        // diagnostic only).
        let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
        while unsafe { HeapWalk(heap, &mut entry) } != 0 {
            let start = entry.lpData as usize;
            if (start..start + entry.cbData as usize).contains(&address) {
                block = format!(
                    "heap {index} block +{:#x} of {} bytes, flags {:#x}",
                    address - start,
                    entry.cbData,
                    entry.wFlags
                );
            }
        }
    }
    // The 24 bytes before the match (a variable name, if it is an
    // environment copy), printable ASCII only.
    let mut before = [0_u8; 24];
    read_own_memory(address.saturating_sub(24), &mut before);
    let before: String = before
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() {
                *byte as char
            } else {
                '.'
            }
        })
        .collect();
    format!(
        "{address:#x} after {before:?}: allocation {:#x} region {:#x}+{:#x} type {:#x} protect {:#x}; {block}",
        info.AllocationBase as usize,
        info.BaseAddress as usize,
        info.RegionSize,
        info.Type,
        info.Protect
    )
}

fn count(haystack: &[u8], masked: &[u8]) -> usize {
    haystack
        .windows(masked.len())
        .filter(|window| {
            window
                .iter()
                .zip(masked)
                .all(|(byte, masked)| byte ^ MASK == *masked)
        })
        .count()
}

#[cfg(target_os = "linux")]
fn writable_regions() -> Vec<(usize, usize)> {
    // An unreadable map yields no regions; the caller's positive control
    // (the key must be found before hand-over) then fails the test.
    let maps = std::fs::read_to_string("/proc/self/maps").unwrap_or_default();
    maps.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let range = fields.next()?;
            let perms = fields.next()?;
            if !perms.starts_with("rw") || line.ends_with("[vvar]") {
                return None;
            }
            let (start, end) = range.split_once('-')?;
            let start = usize::from_str_radix(start, 16).ok()?;
            let end = usize::from_str_radix(end, 16).ok()?;
            Some((start, end - start))
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn read_own_memory(address: usize, buffer: &mut [u8]) -> usize {
    use std::os::unix::fs::FileExt as _;
    thread_local! {
        static MEM: Option<std::fs::File> = std::fs::File::open("/proc/self/mem").ok();
    }
    MEM.with(|mem| {
        mem.as_ref()
            .and_then(|mem| mem.read_at(buffer, address as u64).ok())
            .unwrap_or(0)
    })
}

#[cfg(target_os = "macos")]
mod mach {
    pub(super) const VM_REGION_BASIC_INFO_64: i32 = 9;
    pub(super) const VM_REGION_BASIC_INFO_COUNT_64: u32 = 9;
    pub(super) const VM_PROT_READ: i32 = 1;
    pub(super) const VM_PROT_WRITE: i32 = 2;
    pub(super) const KERN_SUCCESS: i32 = 0;

    pub(super) type Port = u32;

    unsafe extern "C" {
        static mach_task_self_: Port;
        pub(super) fn mach_vm_region(
            task: Port,
            address: *mut u64,
            size: *mut u64,
            flavor: i32,
            info: *mut i32,
            count: *mut u32,
            object_name: *mut Port,
        ) -> i32;
        pub(super) fn mach_vm_read_overwrite(
            task: Port,
            address: u64,
            size: u64,
            data: u64,
            out_size: *mut u64,
        ) -> i32;
    }

    pub(super) fn task_self() -> Port {
        // SAFETY: the C library initializes this before `main`.
        unsafe { mach_task_self_ }
    }
}

#[cfg(target_os = "macos")]
fn writable_regions() -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut address = 0_u64;
    loop {
        let mut size = 0_u64;
        // `vm_region_basic_info_64`: protection is the first field.
        let mut info = [0_i32; mach::VM_REGION_BASIC_INFO_COUNT_64 as usize];
        let mut count = mach::VM_REGION_BASIC_INFO_COUNT_64;
        let mut object = 0;
        // SAFETY: every out-pointer is valid for the duration of the call and
        // `info` holds `count` natural_t words.
        let result = unsafe {
            mach::mach_vm_region(
                mach::task_self(),
                &mut address,
                &mut size,
                mach::VM_REGION_BASIC_INFO_64,
                info.as_mut_ptr(),
                &mut count,
                &mut object,
            )
        };
        if result != mach::KERN_SUCCESS {
            break;
        }
        let protection = info[0];
        if protection & (mach::VM_PROT_READ | mach::VM_PROT_WRITE)
            == mach::VM_PROT_READ | mach::VM_PROT_WRITE
        {
            regions.push((address as usize, size as usize));
        }
        address = address.saturating_add(size);
    }
    regions
}

#[cfg(target_os = "macos")]
fn read_own_memory(address: usize, buffer: &mut [u8]) -> usize {
    let mut read = 0_u64;
    // SAFETY: the kernel copies at most `buffer.len()` bytes into `buffer`
    // and reports failure instead of faulting on unmapped pages.
    let result = unsafe {
        mach::mach_vm_read_overwrite(
            mach::task_self(),
            address as u64,
            buffer.len() as u64,
            buffer.as_mut_ptr() as u64,
            &mut read,
        )
    };
    if result == mach::KERN_SUCCESS {
        read as usize
    } else {
        0
    }
}

#[cfg(windows)]
fn writable_regions() -> Vec<(usize, usize)> {
    use windows_sys::Win32::System::Memory::MEM_COMMIT;
    use windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
    use windows_sys::Win32::System::Memory::PAGE_EXECUTE_READWRITE;
    use windows_sys::Win32::System::Memory::PAGE_EXECUTE_WRITECOPY;
    use windows_sys::Win32::System::Memory::PAGE_GUARD;
    use windows_sys::Win32::System::Memory::PAGE_READWRITE;
    use windows_sys::Win32::System::Memory::PAGE_WRITECOPY;
    use windows_sys::Win32::System::Memory::VirtualQuery;
    let writable =
        PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
    let mut regions = Vec::new();
    let mut address = 0_usize;
    loop {
        // SAFETY: zeroed POD out-structure.
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: queries this process's address space; `info` is valid.
        let written = unsafe {
            VirtualQuery(
                address as *const std::ffi::c_void,
                &mut info,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if written == 0 || info.RegionSize == 0 {
            break;
        }
        if info.State == MEM_COMMIT
            && info.Protect & writable != 0
            && info.Protect & PAGE_GUARD == 0
        {
            regions.push((info.BaseAddress as usize, info.RegionSize));
        }
        match (info.BaseAddress as usize).checked_add(info.RegionSize) {
            Some(next) if next > address => address = next,
            _ => break,
        }
    }
    regions
}

#[cfg(windows)]
fn read_own_memory(address: usize, buffer: &mut [u8]) -> usize {
    use windows_sys::Win32::System::Diagnostics::Debug::ReadProcessMemory;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut read = 0_usize;
    // SAFETY: the kernel copies at most `buffer.len()` bytes into `buffer`
    // and reports failure instead of faulting on pages that changed.
    let ok = unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            address as *const std::ffi::c_void,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut read,
        )
    };
    // A partial copy (a page decommitted meanwhile) still reports its bytes.
    if ok != 0 || read > 0 { read } else { 0 }
}
