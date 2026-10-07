//! PF-27-S05: take a provider key out of this process's environment.
//!
//! Removing a variable is not enough: the C library keeps the original
//! `NAME=value` bytes where they were (the launch environment on the main
//! stack, which `ps -E` and `/proc/<pid>/environ` read, or a `setenv` heap
//! string). The value bytes of every live entry are overwritten in place
//! before the variable is removed.
//!
//! Known limits (recorded in the PF-27-S05 sprint record):
//! - This runs once a session config enables the broker, when Core already
//!   has other threads. The walk of `environ` and `unsetenv` are not
//!   serialized with C-level `getenv` callers (resolver, TLS setup), or with
//!   a concurrent `setenv` that reallocates the array. Doing it before
//!   `main` (in `arg0`) needs the flag decided at process start.
//! - A launch-environment value that `.env` loading already replaced is no
//!   longer reachable through `environ`; its original bytes stay in the
//!   launch block.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::ffi::OsStringExt as _;
use zeroize::Zeroizing;

/// Returns the value of `name` and removes it from the environment, with its
/// bytes overwritten. `None` when unset or empty (nothing is changed).
pub(crate) fn take_env_var(name: &str) -> Option<Zeroizing<Vec<u8>>> {
    if name.is_empty() || name.contains(['=', '\0']) {
        return None;
    }
    let value = Zeroizing::new(std::env::var_os(name)?.into_vec());
    if value.is_empty() {
        return None;
    }
    overwrite_in_place(OsStr::new(name).as_bytes());
    // SAFETY: Rust's own environment accessors serialize with this call. A
    // C-level reader racing it is the known limit in the module docs.
    unsafe { std::env::remove_var(name) };
    Some(value)
}

/// Overwrites the value bytes of every `name=` entry with `0` characters.
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

#[cfg(not(target_os = "macos"))]
unsafe fn environ() -> *mut *mut libc::c_char {
    unsafe extern "C" {
        static mut environ: *mut *mut libc::c_char;
    }
    // SAFETY: reads the C library's `environ` pointer.
    unsafe { environ }
}

#[cfg(test)]
#[path = "env_scrub_tests.rs"]
mod tests;
