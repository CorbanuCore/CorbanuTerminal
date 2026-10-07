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
    let maps = std::fs::read_to_string("/proc/self/maps").expect("read /proc/self/maps");
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
        static MEM: std::fs::File = std::fs::File::open("/proc/self/mem").expect("open /proc/self/mem");
    }
    MEM.with(|mem| mem.read_at(buffer, address as u64).unwrap_or(0))
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
