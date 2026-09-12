//! The entire unsafe boundary. No Rust code runs between child creation and exec.
use std::ffi::CStr;
use std::io;
use std::mem::MaybeUninit;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::ptr;

type Spawn = unsafe extern "C" fn(
    *mut libc::c_int,
    *const libc::c_char,
    *const libc::posix_spawn_file_actions_t,
    *const libc::posix_spawnattr_t,
    *const *mut libc::c_char,
    *const *mut libc::c_char,
) -> libc::c_int;

fn check(code: libc::c_int) -> io::Result<()> {
    if code == 0 { Ok(()) } else { Err(io::Error::from_raw_os_error(code)) }
}

struct Actions(libc::posix_spawn_file_actions_t);
impl Drop for Actions {
    fn drop(&mut self) {
        // SAFETY: created successfully below; uniquely owned and destroyed once.
        unsafe { libc::posix_spawn_file_actions_destroy(&mut self.0); }
    }
}

struct Attributes(libc::posix_spawnattr_t);
impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: created successfully below; uniquely owned and destroyed once.
        unsafe { libc::posix_spawnattr_destroy(&mut self.0); }
    }
}

pub(super) fn launch(path: &CStr, image_fd: i32, argv: &[&CStr]) -> io::Result<OwnedFd> {
    // SAFETY: libc owns a static NUL-terminated version string. Limit this first
    // stage to the source-qualified libc series; missing support has no fallback.
    let version = unsafe { CStr::from_ptr(libc::gnu_get_libc_version()) };
    if version != c"2.43" {
        return Err(io::Error::new(io::ErrorKind::Unsupported, "unqualified libc version"));
    }
    // SAFETY: lookup only; no nullable address is invoked. Symbol ABI is the
    // qualified GNU bits/spawn_ext.h declaration, never pidfd_spawnp.
    let symbol = unsafe { libc::dlvsym(libc::RTLD_DEFAULT, c"pidfd_spawn".as_ptr(), c"GLIBC_2.39".as_ptr()) };
    if symbol.is_null() {
        return Err(io::Error::new(io::ErrorKind::Unsupported, "pidfd spawn unavailable"));
    }
    // SAFETY: non-null versioned libc function with the exact header ABI above.
    let spawn: Spawn = unsafe { std::mem::transmute(symbol) };
    let mut action_storage = MaybeUninit::uninit();
    let mut attr_storage = MaybeUninit::uninit();
    // SAFETY: correctly aligned writable storage, read only after successful init.
    check(unsafe { libc::posix_spawn_file_actions_init(action_storage.as_mut_ptr()) })?;
    let mut actions = Actions(unsafe { action_storage.assume_init() });
    // SAFETY: same storage/initialization contract as the file actions.
    check(unsafe { libc::posix_spawnattr_init(attr_storage.as_mut_ptr()) })?;
    let mut attrs = Attributes(unsafe { attr_storage.assume_init() });
    let mut disposition = MaybeUninit::<libc::sigaction>::uninit();
    // SAFETY: query only, with valid output storage; does not alter parent signals.
    if unsafe { libc::sigaction(libc::SIGCHLD, ptr::null(), disposition.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful sigaction query initializes the output structure.
    let disposition = unsafe { disposition.assume_init() };
    if disposition.sa_sigaction == libc::SIG_IGN || disposition.sa_flags & libc::SA_NOCLDWAIT != 0 {
        return Err(io::Error::new(io::ErrorKind::Unsupported, "child reaping unavailable"));
    }
    let mut empty = MaybeUninit::<libc::sigset_t>::uninit();
    let mut defaults = MaybeUninit::<libc::sigset_t>::uninit();
    // SAFETY: valid signal-set storage, then valid initialized attrs/actions.
    // The C functions copy their inputs; fixed C strings remain alive throughout.
    unsafe {
        if libc::sigemptyset(empty.as_mut_ptr()) != 0 || libc::sigfillset(defaults.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        check(libc::posix_spawnattr_setsigmask(&mut attrs.0, empty.as_ptr()))?;
        check(libc::posix_spawnattr_setsigdefault(&mut attrs.0, defaults.as_ptr()))?;
        check(libc::posix_spawnattr_setflags(&mut attrs.0,
            (libc::POSIX_SPAWN_SETSIGMASK | libc::POSIX_SPAWN_SETSIGDEF) as i16))?;
        check(libc::posix_spawn_file_actions_addchdir_np(&mut actions.0, c"/".as_ptr()))?;
        for fd in 0..3 {
            check(libc::posix_spawn_file_actions_addopen(&mut actions.0, fd, c"/dev/null".as_ptr(), libc::O_RDWR, 0))?;
        }
        for fd in 3..image_fd {
            check(libc::posix_spawn_file_actions_addclose(&mut actions.0, fd))?;
        }
        // Keep the image until exec, then its verified CLOEXEC flag closes it.
        check(libc::posix_spawn_file_actions_addclosefrom_np(&mut actions.0, image_fd + 1))?;
    }
    let mut args: Vec<_> = argv.iter().map(|arg| arg.as_ptr().cast_mut()).collect();
    args.push(ptr::null_mut());
    let env = [ptr::null_mut()];
    let mut pidfd = -1;
    // SAFETY: all pointers/arrays/strings and image fd remain live for the
    // synchronous call. libc atomically creates the owned pidfd, reaping setup/
    // exec failures under the documented exclusive-owner/platform preconditions.
    check(unsafe { spawn(&mut pidfd, path.as_ptr(), &actions.0, &attrs.0, args.as_ptr(), env.as_ptr()) })?;
    // SAFETY: successful pidfd_spawn returns one newly owned fd. Adopt before
    // any further fallible operation; no PID conversion or post-spawn lookup.
    Ok(unsafe { OwnedFd::from_raw_fd(pidfd) })
}
