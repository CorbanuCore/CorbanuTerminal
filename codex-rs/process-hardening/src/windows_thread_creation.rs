//! PF-27-S07: threads protected from the moment they are created.
//!
//! A new thread's DACL comes from the creating token's default DACL, which
//! gives the user full access, so before PF-27-S07 a same-user process could
//! open a fresh thread between its creation and the TLS callback that
//! protects it, and keep that handle. Two mechanisms close that window:
//!
//! - In every hardened process, this image's `CreateThread` imports (every
//!   Rust thread: std, tokio; the C runtime is linked statically, so its
//!   `_beginthreadex` too) are redirected to a wrapper that passes the
//!   protected thread descriptor, so the thread object never exists with
//!   another DACL. Threads any other module starts (Windows thread pools,
//!   RPC, injected DLLs) still rely on the TLS callback, and the loader's own
//!   workers skip it.
//! - In a process that starts no children and creates no unnamed objects it
//!   reopens (the credential broker), the token's default DACL becomes the
//!   protected one, which covers every thread whoever starts it. It would
//!   break Core: pipes created without a descriptor (std's child pipes) take
//!   their DACL from it, and child processes inherit it.

use crate::windows_process_access::SecurityDescriptor;
use crate::windows_process_access::current_user_sid_string;
use crate::windows_process_access::thread_dacl_sddl;
use crate::windows_process_access::thread_descriptor;
use std::ffi::c_void;
use std::io;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Security::SetTokenInformation;
use windows_sys::Win32::Security::TOKEN_ADJUST_DEFAULT;
use windows_sys::Win32::Security::TOKEN_DEFAULT_DACL;
use windows_sys::Win32::Security::TokenDefaultDacl;
use windows_sys::Win32::System::Memory::PAGE_READWRITE;
use windows_sys::Win32::System::Memory::VirtualProtect;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::LPTHREAD_START_ROUTINE;
use windows_sys::Win32::System::Threading::OpenProcessToken;

type CreateThreadFn = unsafe extern "system" fn(
    *const SECURITY_ATTRIBUTES,
    usize,
    LPTHREAD_START_ROUTINE,
    *const c_void,
    u32,
    *mut u32,
) -> HANDLE;

/// The `CreateThread` the redirected imports pointed to; 0 before.
static ORIGINAL_CREATE_THREAD: AtomicUsize = AtomicUsize::new(0);
/// Serializes redirection (import pages are re-protected around writes).
static REDIRECT_LOCK: Mutex<()> = Mutex::new(());

/// Stands in for `CreateThread`: a thread created without its own
/// descriptor gets the protected one, once the process is hardened.
unsafe extern "system" fn create_protected_thread(
    attributes: *const SECURITY_ATTRIBUTES,
    stack_size: usize,
    start: LPTHREAD_START_ROUTINE,
    parameter: *const c_void,
    flags: u32,
    thread_id: *mut u32,
) -> HANDLE {
    let mut original = ORIGINAL_CREATE_THREAD.load(Ordering::Acquire);
    if original == 0 {
        // Stored before any import points here; a weakly ordered CPU could
        // still show the import first. Never call back through the import.
        original = resolve_create_thread();
        if original == 0 {
            // SAFETY: plain thread-local error state.
            unsafe { windows_sys::Win32::Foundation::SetLastError(ERROR_NOT_READY) };
            return 0;
        }
    }
    // SAFETY: `original` is `CreateThread` from kernel32.
    let original: CreateThreadFn = unsafe { std::mem::transmute(original) };
    let descriptor = thread_descriptor();
    // SAFETY: a non-null `attributes` points to a caller-owned structure.
    let explicit =
        !attributes.is_null() && unsafe { !(*attributes).lpSecurityDescriptor.is_null() };
    if descriptor.is_null() || explicit {
        // SAFETY: the caller's arguments, passed through unchanged.
        return unsafe { original(attributes, stack_size, start, parameter, flags, thread_id) };
    }
    let protected = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        // SAFETY: as above.
        bInheritHandle: if attributes.is_null() {
            0
        } else {
            unsafe { (*attributes).bInheritHandle }
        },
    };
    // SAFETY: `protected` outlives the call; the descriptor is never freed.
    unsafe { original(&protected, stack_size, start, parameter, flags, thread_id) }
}

/// `ERROR_NOT_READY`.
const ERROR_NOT_READY: u32 = 21;

/// `kernel32!CreateThread`, or 0.
fn resolve_create_thread() -> usize {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
    let name: Vec<u16> = "kernel32.dll".encode_utf16().chain([0]).collect();
    // SAFETY: kernel32 is always loaded; both strings are NUL-terminated.
    unsafe {
        let module = GetModuleHandleW(name.as_ptr());
        if module == 0 {
            return 0;
        }
        GetProcAddress(module, c"CreateThread".as_ptr().cast()).map_or(0, |f| f as usize)
    }
}

/// Points every `CreateThread` import of this image at
/// [`create_protected_thread`]. Fails, changing nothing, if there is none
/// (the linker would have bound thread creation some other way) or any
/// entry cannot be made writable. Idempotent.
pub(crate) fn redirect_thread_creation() -> io::Result<()> {
    let _guard = REDIRECT_LOCK
        .lock()
        .map_err(|_| io::Error::other("thread-creation redirect lock poisoned"))?;
    let wrapper = create_protected_thread as CreateThreadFn as usize;
    let slots: Vec<*mut usize> = create_thread_import_slots()
        .into_iter()
        // SAFETY: pointer-sized, aligned import address table entries of
        // this mapped image.
        .filter(|slot| unsafe { slot.read_volatile() } != wrapper)
        .collect();
    if slots.is_empty() {
        return if thread_creation_protected() {
            Ok(())
        } else {
            Err(io::Error::other(
                "this image imports no CreateThread to protect new threads with",
            ))
        };
    }
    // SAFETY: as above.
    let original = unsafe { slots[0].read_volatile() };
    let _ =
        ORIGINAL_CREATE_THREAD.compare_exchange(0, original, Ordering::AcqRel, Ordering::Acquire);
    let size = std::mem::size_of::<usize>();
    // Make every entry writable first, so a failure changes nothing.
    let mut writable = Vec::with_capacity(slots.len());
    for slot in &slots {
        let mut previous = 0_u32;
        // SAFETY: changes the protection of one table entry of this image.
        if unsafe { VirtualProtect(slot.cast(), size, PAGE_READWRITE, &mut previous) } == 0 {
            let error = io::Error::last_os_error();
            restore_protection(&writable, size);
            return Err(error);
        }
        writable.push((*slot, previous));
    }
    for slot in &slots {
        // SAFETY: the entry is writable now; an aligned pointer-sized store
        // is atomic, so a concurrent caller sees the old or new function.
        unsafe { slot.write_volatile(wrapper) };
    }
    restore_protection(&writable, size);
    Ok(())
}

fn restore_protection(entries: &[(*mut usize, u32)], size: usize) {
    for (slot, previous) in entries {
        let mut ignored = 0_u32;
        // SAFETY: restores the protection this entry had.
        unsafe { VirtualProtect(slot.cast(), size, *previous, &mut ignored) };
    }
}

/// True when this image imports `CreateThread` and every import is
/// redirected to [`create_protected_thread`].
pub fn thread_creation_protected() -> bool {
    let wrapper = create_protected_thread as CreateThreadFn as usize;
    let slots = create_thread_import_slots();
    // SAFETY: entries of this mapped image's import address table.
    !slots.is_empty()
        && slots
            .iter()
            .all(|slot| unsafe { slot.read_volatile() } == wrapper)
}

/// The import address table entries of this image bound to `CreateThread`
/// (from any DLL), found by name.
fn create_thread_import_slots() -> Vec<*mut usize> {
    let Some(base) = this_image_base() else {
        return Vec::new();
    };
    // SAFETY: `base` is the base of this mapped PE image.
    unsafe { import_slots(base, b"CreateThread") }
}

fn this_image_base() -> Option<*const u8> {
    use windows_sys::Win32::System::LibraryLoader::GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS;
    use windows_sys::Win32::System::LibraryLoader::GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleExW;
    let mut module = 0;
    // SAFETY: looks up the module containing this function; no reference taken.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            (create_protected_thread as CreateThreadFn as usize) as *const u16,
            &mut module,
        )
    };
    (found != 0).then_some(module as *const u8)
}

/// Import address table entries for functions imported by `name`.
///
/// # Safety
/// `base` must be the base address of a mapped 64-bit PE image.
unsafe fn import_slots(base: *const u8, name: &[u8]) -> Vec<*mut usize> {
    const IMPORT_DIRECTORY_INDEX: usize = 1;
    const DESCRIPTOR_SIZE: usize = 20;
    const ORDINAL_FLAG: usize = 1 << (usize::BITS - 1);
    let read_u32 = |offset: usize| unsafe { base.add(offset).cast::<u32>().read_unaligned() };
    let nt = read_u32(0x3c) as usize;
    if read_u32(nt) != 0x0000_4550 {
        return Vec::new();
    }
    let optional = nt + 4 + 20;
    // PE32+ only: 64-bit Windows is the only target.
    if unsafe { base.add(optional).cast::<u16>().read_unaligned() } != 0x20b
        || (read_u32(optional + 108) as usize) <= IMPORT_DIRECTORY_INDEX
    {
        return Vec::new();
    }
    let imports = read_u32(optional + 112 + IMPORT_DIRECTORY_INDEX * 8) as usize;
    if imports == 0 {
        return Vec::new();
    }
    let mut slots = Vec::new();
    let mut descriptor = imports;
    loop {
        let names = read_u32(descriptor) as usize;
        let dll = read_u32(descriptor + 12);
        let table = read_u32(descriptor + 16) as usize;
        if dll == 0 && table == 0 {
            break;
        }
        // Without the name table the entries cannot be told apart.
        if names != 0 {
            let mut index = 0;
            loop {
                let entry_offset = index * std::mem::size_of::<usize>();
                // SAFETY: within the image's import name table.
                let entry = unsafe {
                    base.add(names + entry_offset)
                        .cast::<usize>()
                        .read_unaligned()
                };
                if entry == 0 {
                    break;
                }
                if entry & ORDINAL_FLAG == 0 {
                    // IMAGE_IMPORT_BY_NAME: a 2-byte hint, then the name.
                    let symbol = unsafe { base.add((entry & 0x7fff_ffff) + 2) };
                    // SAFETY: a NUL-terminated name inside the image.
                    let symbol = unsafe { std::ffi::CStr::from_ptr(symbol.cast()) };
                    if symbol.to_bytes() == name {
                        slots.push(unsafe { base.add(table + entry_offset) } as *mut usize);
                    }
                }
                index += 1;
            }
        }
        descriptor += DESCRIPTOR_SIZE;
    }
    slots
}

/// Makes the protected thread DACL this process token's default DACL, so
/// every thread (and every other object created without a descriptor) is
/// protected from creation, whoever starts it. Only for processes that
/// start no children and never reopen such objects (the credential broker).
pub fn protect_new_objects_by_default() -> io::Result<()> {
    let mut token: HANDLE = 0;
    // SAFETY: the pseudo-handle is valid and `token` is a valid out pointer.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_DEFAULT, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let result = set_protected_default_dacl(token);
    // SAFETY: opened above.
    unsafe { CloseHandle(token) };
    result
}

/// Sets the protected thread DACL as `token`'s default DACL. `token` needs
/// `TOKEN_ADJUST_DEFAULT` and must belong to the current user.
fn set_protected_default_dacl(token: HANDLE) -> io::Result<()> {
    let descriptor = SecurityDescriptor::from_sddl(&thread_dacl_sddl(&current_user_sid_string()?))?;
    let dacl: *const ACL = descriptor.dacl()?;
    let value = TOKEN_DEFAULT_DACL {
        DefaultDacl: dacl.cast_mut(),
    };
    // SAFETY: `value` points into `descriptor`, which outlives the call.
    let ok = unsafe {
        SetTokenInformation(
            token,
            TokenDefaultDacl,
            (&value as *const TOKEN_DEFAULT_DACL).cast(),
            std::mem::size_of::<TOKEN_DEFAULT_DACL>() as u32,
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// The redirect has something to redirect: Rust threads are created
    /// through this image's `CreateThread` import.
    #[test]
    fn pf_27_s07_this_image_imports_create_thread() {
        assert!(!super::create_thread_import_slots().is_empty());
    }
}
