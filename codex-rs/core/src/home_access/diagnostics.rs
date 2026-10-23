//! #616: a diagnostic record of a failed access to the Corbanu home.
//!
//! Only file-system facts are recorded (paths, modes, owners, flags, ACLs,
//! extended-attribute names, mounts, sandbox and responsible process), never
//! file contents, environment variables or credentials.

use std::fmt::Write as _;
use std::path::Path;

/// Whether this process's sandbox denies reading `path`; `None` when that
/// cannot be checked.
pub(super) fn sandbox_denies_read(path: &Path) -> Option<bool> {
    platform::sandbox_denies_read(path)
}

/// The record for one failed `operation` on `path`.
pub(super) fn record(operation: &str, path: &Path, error: &str, now: &str) -> String {
    let mut out = String::new();
    let argv0 = std::env::args_os()
        .next()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_default();
    let _ = writeln!(
        out,
        "=== {now} pid={} ppid={} argv0={argv0}\n\
         operation: {operation} {}\nerror: {error}",
        std::process::id(),
        parent_pid(),
        path.display(),
    );
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd.display().to_string(),
        Err(error) => format!("unavailable ({error})"),
    };
    let _ = writeln!(out, "cwd: {cwd}");
    platform::process_facts(&mut out, path);
    let _ = writeln!(out, "path and parents:");
    for ancestor in path.ancestors() {
        let _ = writeln!(out, "  {}: {}", ancestor.display(), entry_facts(ancestor));
    }
    out
}

fn parent_pid() -> u32 {
    #[cfg(unix)]
    {
        std::os::unix::process::parent_id()
    }
    #[cfg(not(unix))]
    {
        0
    }
}

fn entry_facts(path: &Path) -> String {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return format!("stat failed ({error})"),
    };
    let mut facts = String::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let _ = write!(
            facts,
            "mode={:o} uid={} gid={} dev={} ino={}",
            metadata.mode(),
            metadata.uid(),
            metadata.gid(),
            metadata.dev(),
            metadata.ino()
        );
    }
    #[cfg(not(unix))]
    {
        let _ = write!(facts, "readonly={}", metadata.permissions().readonly());
    }
    platform::entry_facts(&mut facts, path, &metadata);
    facts
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::fmt::Write as _;
    use std::os::raw::c_char;
    use std::os::raw::c_int;
    use std::os::raw::c_void;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    const ACL_TYPE_EXTENDED: c_int = 0x0000_0100;
    const SANDBOX_FILTER_PATH: c_int = 1;

    unsafe extern "C" {
        fn acl_get_link_np(path: *const c_char, acl_type: c_int) -> *mut c_void;
        fn acl_to_text(acl: *mut c_void, len: *mut isize) -> *mut c_char;
        fn acl_free(obj: *mut c_void) -> c_int;
    }

    type SandboxCheck = unsafe extern "C" fn(libc::pid_t, *const c_char, c_int, ...) -> c_int;
    type ResponsiblePid = unsafe extern "C" fn(libc::pid_t) -> libc::pid_t;

    /// A private libSystem symbol, looked up at run time so a missing one
    /// never stops the binary from loading.
    fn symbol(name: &CStr) -> Option<*mut c_void> {
        // SAFETY: `name` is NUL-terminated; RTLD_DEFAULT searches loaded images.
        let address = unsafe { libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr()) };
        (!address.is_null()).then_some(address)
    }

    pub(super) fn process_facts(out: &mut String, path: &Path) {
        // SAFETY: getpid has no preconditions.
        let pid = unsafe { libc::getpid() };
        let responsible = symbol(c"responsibility_get_pid_responsible_for_pid").map(|address| {
            // SAFETY: the symbol has this signature in every macOS that exports it.
            let responsible: ResponsiblePid = unsafe { std::mem::transmute(address) };
            // SAFETY: takes a pid and returns one.
            let responsible_pid = unsafe { responsible(pid) };
            format!(
                "pid={responsible_pid} path={}",
                process_path(responsible_pid)
            )
        });
        let _ = writeln!(
            out,
            "responsible process: {}",
            responsible.as_deref().unwrap_or("unavailable")
        );
        let sandbox = match sandbox_denies_read(path) {
            Some(true) => "denied",
            Some(false) => "allowed",
            None => "unavailable",
        };
        let _ = writeln!(out, "sandbox_check(file-read-data): {sandbox}");
        let _ = writeln!(out, "mounted volume: {}", mount_facts(path));
    }

    pub(super) fn sandbox_denies_read(path: &Path) -> Option<bool> {
        let check = symbol(c"sandbox_check")?;
        let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
        // Without it a denied check is reported as a sandbox violation.
        let no_report = symbol(c"SANDBOX_CHECK_NO_REPORT").map_or(0, |address| {
            // SAFETY: the symbol is a `const enum sandbox_filter_type` (an int).
            unsafe { *address.cast::<c_int>() }
        });
        // SAFETY: sandbox_check(pid, operation, SANDBOX_FILTER_PATH, path).
        let check: SandboxCheck = unsafe { std::mem::transmute(check) };
        // SAFETY: both strings are NUL-terminated and outlive the call.
        let result = unsafe {
            check(
                libc::getpid(),
                c"file-read-data".as_ptr(),
                SANDBOX_FILTER_PATH | no_report,
                c_path.as_ptr(),
            )
        };
        match result {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn process_path(pid: libc::pid_t) -> String {
        let mut buffer = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
        // SAFETY: the buffer is writable for its full length.
        let len =
            unsafe { libc::proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
        if len <= 0 {
            return "unknown".to_string();
        }
        buffer.truncate(len as usize);
        String::from_utf8_lossy(&buffer).into_owned()
    }

    /// The mount holding `path`, from the kernel's mount table (no lookup of
    /// `path` itself, which may be the thing that fails).
    fn mount_facts(path: &Path) -> String {
        // SAFETY: a null buffer only asks for the number of mounts.
        let count = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
        let Ok(count) = usize::try_from(count) else {
            return "mount table unavailable".to_string();
        };
        // Room for mounts added since the first call.
        // SAFETY: statfs is plain data; all-zero is a valid value.
        let mut mounts = vec![unsafe { std::mem::zeroed::<libc::statfs>() }; count + 8];
        let Ok(size) = c_int::try_from(mounts.len() * std::mem::size_of::<libc::statfs>()) else {
            return "mount table unavailable".to_string();
        };
        // SAFETY: the buffer holds `mounts.len()` entries, `size` bytes.
        let filled = unsafe { libc::getfsstat(mounts.as_mut_ptr(), size, libc::MNT_NOWAIT) };
        let Ok(filled) = usize::try_from(filled) else {
            return "mount table unavailable".to_string();
        };
        mounts.truncate(filled);
        let field = |chars: &[c_char]| {
            // SAFETY: statfs name fields are NUL-terminated within their arrays.
            unsafe { CStr::from_ptr(chars.as_ptr()) }
                .to_string_lossy()
                .into_owned()
        };
        mounts
            .iter()
            .map(|mount| (field(&mount.f_mntonname), mount))
            .filter(|(on, _)| path.starts_with(on))
            .max_by_key(|(on, _)| on.len())
            .map_or_else(
                || "none".to_string(),
                |(on, mount)| {
                    format!(
                        "{on} from {} type {} flags={:#x}",
                        field(&mount.f_mntfromname),
                        field(&mount.f_fstypename),
                        mount.f_flags
                    )
                },
            )
    }

    pub(super) fn entry_facts(out: &mut String, path: &Path, metadata: &std::fs::Metadata) {
        use std::os::macos::fs::MetadataExt;
        let _ = write!(out, " flags={:#x}", metadata.st_flags());
        let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
            return;
        };
        let mut names = vec![0u8; 4096];
        // SAFETY: the buffer is writable for its full length.
        let len = unsafe {
            libc::listxattr(
                c_path.as_ptr(),
                names.as_mut_ptr().cast(),
                names.len(),
                libc::XATTR_NOFOLLOW,
            )
        };
        let xattrs = if len < 0 {
            format!("unavailable ({})", std::io::Error::last_os_error())
        } else {
            names.truncate(len as usize);
            names
                .split(|byte| *byte == 0)
                .filter(|name| !name.is_empty())
                .map(String::from_utf8_lossy)
                .collect::<Vec<_>>()
                .join(",")
        };
        let _ = write!(out, " xattrs=[{xattrs}]");
        // SAFETY: c_path is NUL-terminated; a non-null ACL is freed below.
        let acl = unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_EXTENDED) };
        if acl.is_null() {
            let _ = write!(out, " acl=none");
            return;
        }
        // SAFETY: acl is a valid ACL; the text is freed with acl_free.
        let text = unsafe { acl_to_text(acl, std::ptr::null_mut()) };
        if !text.is_null() {
            // SAFETY: acl_to_text returns a NUL-terminated string.
            let acl_text = unsafe { CStr::from_ptr(text) }.to_string_lossy();
            let _ = write!(out, " acl=[{}]", acl_text.trim().replace('\n', "; "));
            // SAFETY: allocated by acl_to_text.
            unsafe { acl_free(text.cast()) };
        }
        // SAFETY: allocated by acl_get_link_np.
        unsafe { acl_free(acl) };
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use std::path::Path;

    pub(super) fn process_facts(_out: &mut String, _path: &Path) {}

    pub(super) fn sandbox_denies_read(_path: &Path) -> Option<bool> {
        None
    }

    pub(super) fn entry_facts(_out: &mut String, _path: &Path, _metadata: &std::fs::Metadata) {}
}
