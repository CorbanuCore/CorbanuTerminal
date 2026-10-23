use crate::policy_transforms::should_require_platform_sandbox;
use codex_install_context::InstallContext;
use codex_protocol::models::PermissionProfile;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::io::ErrorKind;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::fd::RawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const SYSTEM_BWRAP_PROGRAM: &str = "bwrap";
pub const APPARMOR_BWRAP_USERNS_PROFILE: &str = "rootlesskit";
const MISSING_BWRAP_WARNING: &str = concat!(
    "Corbanu Terminal could not find bubblewrap on PATH. ",
    "Install bubblewrap with your OS package manager. ",
    "See the sandbox prerequisites: ",
    "https://developers.openai.com/codex/concepts/sandboxing#prerequisites. ",
    "Corbanu Terminal will use the bundled bubblewrap in the meantime.",
);
/// Issue #563: neither a system nor a bundled bubblewrap, so every sandboxed
/// command is refused (see [`BWRAP_UNAVAILABLE_MESSAGE`]).
const NO_BWRAP_WARNING: &str = concat!(
    "Corbanu Terminal could not find bubblewrap on PATH, and no bundled copy came with it, ",
    "so it cannot run commands in its Linux sandbox. ",
    "Install bubblewrap with your OS package manager ",
    "(for example `sudo apt install bubblewrap` or `sudo dnf install bubblewrap`). ",
    "See the sandbox prerequisites: ",
    "https://developers.openai.com/codex/concepts/sandboxing#prerequisites.",
);
/// Issue #563: what the Linux sandbox helper prints, before exiting with
/// status 1, when it finds no usable bubblewrap. The command does not run.
/// It starts with the startup-failure marker the tool orchestrator matches
/// ("bubblewrap is unavailable") and names the sandbox, so a run is handled
/// exactly as it was when this was a panic.
pub const BWRAP_UNAVAILABLE_MESSAGE: &str = concat!(
    "bubblewrap is unavailable: Corbanu Terminal's Linux sandbox found no usable `bwrap` on ",
    "PATH and no bundled copy, so the command did not run. ",
    "Install or update bubblewrap with your OS package manager ",
    "(for example `sudo apt install bubblewrap` or `sudo dnf install bubblewrap`), ",
    "then try again.",
);
/// #581: no system bubblewrap, and the bundled copy cannot create user
/// namespaces here: AppArmor restricts them (the Ubuntu 24.04 default) and
/// the `rootlesskit` profile that would allow them through `aa-exec` is not
/// available. Every sandboxed command fails to start.
const BUNDLED_BWRAP_BLOCKED_WARNING: &str = concat!(
    "Corbanu Terminal could not find bubblewrap on PATH, and its bundled copy cannot start ",
    "the Linux sandbox on this host: AppArmor restricts unprivileged user namespaces and no ",
    "`rootlesskit` AppArmor profile is loaded to allow them, so sandboxed commands will not run. ",
    "Install bubblewrap with your OS package manager (for example `sudo apt install bubblewrap`) ",
    "and allow it to create user namespaces (on Ubuntu, the `apparmor` package's `rootlesskit` ",
    "profile). See the sandbox prerequisites: ",
    "https://developers.openai.com/codex/concepts/sandboxing#prerequisites.",
);
/// The sysctl through which AppArmor restricts unprivileged user namespaces.
const APPARMOR_RESTRICT_USERNS_SYSCTL: &str =
    "/proc/sys/kernel/apparmor_restrict_unprivileged_userns";
const AA_EXEC_CANDIDATES: [&str; 2] = ["/usr/bin/aa-exec", "/usr/sbin/aa-exec"];
const USER_NAMESPACE_WARNING: &str =
    "Corbanu Terminal's Linux sandbox uses bubblewrap and needs access to create user namespaces.";
pub(crate) const WSL1_BWRAP_WARNING: &str = concat!(
    "Corbanu Terminal's Linux sandbox uses bubblewrap, which is not supported on WSL1 ",
    "because WSL1 cannot create the required user namespaces. ",
    "Use WSL2 for sandboxed shell commands."
);
const USER_NAMESPACE_FAILURES: [&str; 4] = [
    "loopback: Failed RTM_NEWADDR",
    "loopback: Failed RTM_NEWLINK",
    "setting up uid map: Permission denied",
    "No permissions to create a new namespace",
];
const SYSTEM_BWRAP_PROBE_TIMEOUT: Duration = Duration::from_millis(500);
const SYSTEM_BWRAP_PROBE_POLL_INTERVAL: Duration = Duration::from_millis(50);
const SYSTEM_BWRAP_PROBE_STDERR_LIMIT_BYTES: u64 = 64 * 1024;

pub fn system_bwrap_warning(permission_profile: &PermissionProfile) -> Option<String> {
    if !should_warn_about_system_bwrap(permission_profile) {
        return None;
    }

    let system_bwrap_path = find_system_bwrap_in_path();
    system_bwrap_warning_for_path(
        system_bwrap_path.as_deref(),
        || find_bundled_bwrap().is_some(),
        bundled_bwrap_can_start,
    )
}

/// #581: whether a bubblewrap with no AppArmor profile of its own (the
/// bundled copy) can create user namespaces here, judged without running
/// it: AppArmor does not restrict them, or the `rootlesskit` profile is
/// loaded for `aa-exec`. The sandbox helper probes the verified copy itself
/// before each run.
fn bundled_bwrap_can_start() -> bool {
    let restricted = std::fs::read_to_string(APPARMOR_RESTRICT_USERNS_SYSCTL)
        .is_ok_and(|value| value.trim() == "1");
    !restricted
        || find_aa_exec().is_some_and(|aa_exec| {
            let mut command = Command::new(aa_exec);
            command.args(["-p", APPARMOR_BWRAP_USERNS_PROFILE, "--", "/bin/true"]);
            command_succeeds(&mut command, SYSTEM_BWRAP_PROBE_TIMEOUT)
        })
}

fn find_aa_exec() -> Option<PathBuf> {
    AA_EXEC_CANDIDATES
        .into_iter()
        .map(PathBuf::from)
        .find(|candidate| candidate.is_file())
}

fn should_warn_about_system_bwrap(permission_profile: &PermissionProfile) -> bool {
    let (file_system_policy, network_policy) = permission_profile.to_runtime_permissions();
    should_require_platform_sandbox(
        &file_system_policy,
        network_policy,
        /*has_managed_network_requirements*/ false,
    )
}

fn system_bwrap_warning_for_path(
    system_bwrap_path: Option<&Path>,
    has_bundled_bwrap: impl FnOnce() -> bool,
    bundled_bwrap_can_start: impl FnOnce() -> bool,
) -> Option<String> {
    if is_wsl1() {
        return Some(WSL1_BWRAP_WARNING.to_string());
    }

    let Some(system_bwrap_path) = system_bwrap_path else {
        let warning = if !has_bundled_bwrap() {
            NO_BWRAP_WARNING
        } else if bundled_bwrap_can_start() {
            MISSING_BWRAP_WARNING
        } else {
            BUNDLED_BWRAP_BLOCKED_WARNING
        };
        return Some(warning.to_string());
    };

    if !system_bwrap_has_user_namespace_access(system_bwrap_path, SYSTEM_BWRAP_PROBE_TIMEOUT)
        && find_apparmor_bwrap_launcher(system_bwrap_path).is_none()
    {
        return Some(USER_NAMESPACE_WARNING.to_string());
    }

    None
}

fn system_bwrap_has_user_namespace_access(system_bwrap_path: &Path, timeout: Duration) -> bool {
    let mut command = Command::new(system_bwrap_path);
    command.args(bwrap_user_namespace_probe_args());
    command_has_user_namespace_access(&mut command, timeout)
}

/// Lets `command`'s child inherit `fd` (cleared close-on-exec in the child
/// only, so no other process started meanwhile gets it).
fn inherit_fd(command: &mut Command, fd: RawFd) {
    // SAFETY: the closure only calls `fcntl`, which is async-signal-safe.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

/// Ubuntu's AppArmor user-namespace restriction can reject an otherwise valid unprivileged
/// bubblewrap invocation. Ubuntu ships a narrowly named, unconfined `rootlesskit` profile whose
/// purpose is to authorize user namespaces. When that profile is loaded, execute bubblewrap
/// through `aa-exec` instead of failing every Workspace/Read-only command and asking the user to
/// escape the sandbox.
///
/// The returned launcher is accepted only after it successfully runs the same user+network
/// namespace probe as direct bubblewrap. Callers must still pass the absolute bubblewrap path
/// after `aa-exec --`; this function does not authorize a PATH-resolved replacement binary.
pub fn find_apparmor_bwrap_launcher(system_bwrap_path: &Path) -> Option<PathBuf> {
    if system_bwrap_has_user_namespace_access(system_bwrap_path, SYSTEM_BWRAP_PROBE_TIMEOUT) {
        return None;
    }

    let launcher = find_aa_exec()?;
    let mut command = Command::new(&launcher);
    command
        .args(["-p", APPARMOR_BWRAP_USERNS_PROFILE, "--"])
        .arg(system_bwrap_path)
        .args(bwrap_user_namespace_probe_args());
    command_has_user_namespace_access(&mut command, SYSTEM_BWRAP_PROBE_TIMEOUT).then_some(launcher)
}

/// #581: [`find_apparmor_bwrap_launcher`] for the bundled bubblewrap, run as
/// `/proc/self/fd/<fd>` from its digest-verified descriptor `fd` (which
/// each probe inherits), so no unverified copy is ever run. `aa-exec` is
/// used only when the copy cannot create user namespaces by itself and
/// does through it: here the probe must succeed, since `aa-exec` failing to
/// start the copy says nothing about namespaces.
pub fn find_apparmor_bwrap_launcher_for_fd(fd: RawFd) -> Option<PathBuf> {
    let bwrap = PathBuf::from(format!("/proc/self/fd/{fd}"));
    let mut direct = Command::new(&bwrap);
    direct.args(bwrap_user_namespace_probe_args());
    inherit_fd(&mut direct, fd);
    if command_has_user_namespace_access(&mut direct, SYSTEM_BWRAP_PROBE_TIMEOUT) {
        return None;
    }
    let launcher = find_aa_exec()?;
    let mut command = Command::new(&launcher);
    command
        .args(["-p", APPARMOR_BWRAP_USERNS_PROFILE, "--"])
        .arg(&bwrap)
        .args(bwrap_user_namespace_probe_args());
    inherit_fd(&mut command, fd);
    command_succeeds(&mut command, SYSTEM_BWRAP_PROBE_TIMEOUT).then_some(launcher)
}

/// Whether `command` exits successfully within `timeout` (killed after it).
fn command_succeeds(command: &mut Command, timeout: Duration) -> bool {
    let Ok(mut child) = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if Instant::now() < deadline => {
                thread::sleep(SYSTEM_BWRAP_PROBE_POLL_INTERVAL);
            }
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

fn bwrap_user_namespace_probe_args() -> [&'static str; 7] {
    [
        "--unshare-user",
        "--unshare-net",
        "--ro-bind",
        "/",
        "/",
        "--",
        "/bin/true",
    ]
}

fn command_has_user_namespace_access(command: &mut Command, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    command.stdout(Stdio::null()).stderr(Stdio::piped());
    let mut child = loop {
        match command.spawn() {
            Ok(child) => break child,
            // #635: Linux refuses to run a file that any process still holds
            // open for writing (a child forked by another thread keeps a copy
            // of the writer's descriptor until it starts its own program), and
            // that ends quickly: retry until the deadline instead of reading
            // the spawn error as "has access".
            Err(err)
                if err.kind() == ErrorKind::ExecutableFileBusy && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(5));
            }
            Err(_) => return true,
        }
    };
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = child.stderr.take().map_or_else(Vec::new, |stderr| {
                    let fd = stderr.as_raw_fd();
                    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
                    if flags < 0
                        || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
                    {
                        return Vec::new();
                    }

                    let mut bytes = Vec::new();
                    let mut stderr = stderr.take(SYSTEM_BWRAP_PROBE_STDERR_LIMIT_BYTES);
                    if let Err(err) = stderr.read_to_end(&mut bytes)
                        && err.kind() != ErrorKind::WouldBlock
                    {
                        return bytes;
                    }
                    bytes
                });
                let output = Output {
                    status,
                    stdout: Vec::new(),
                    stderr,
                };
                return output.status.success() || !is_user_namespace_failure(&output);
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return true;
                }
                thread::sleep(SYSTEM_BWRAP_PROBE_POLL_INTERVAL);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return true;
            }
        }
    }
}

pub(crate) fn is_wsl1() -> bool {
    std::fs::read_to_string("/proc/version")
        .is_ok_and(|proc_version| proc_version_indicates_wsl1(&proc_version))
}

fn proc_version_indicates_wsl1(proc_version: &str) -> bool {
    let proc_version = proc_version.to_ascii_lowercase();
    let mut remaining = proc_version.as_str();
    while let Some(marker) = remaining.find("wsl") {
        let version_start = marker + "wsl".len();
        let version_digits: String = remaining[version_start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(version) = version_digits.parse::<u32>() {
            return version == 1;
        }
        remaining = &remaining[version_start..];
    }

    proc_version.contains("microsoft") && !proc_version.contains("microsoft-standard")
}

fn is_user_namespace_failure(output: &Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr);
    USER_NAMESPACE_FAILURES
        .iter()
        .any(|failure| stderr.contains(failure))
}

pub fn find_system_bwrap_in_path() -> Option<PathBuf> {
    let search_path = std::env::var_os("PATH")?;
    let cwd = std::env::current_dir().ok()?;
    find_system_bwrap_in_search_paths(std::env::split_paths(&search_path), &cwd)
}

fn find_system_bwrap_in_search_paths(
    search_paths: impl IntoIterator<Item = PathBuf>,
    cwd: &Path,
) -> Option<PathBuf> {
    let search_path = std::env::join_paths(search_paths).ok()?;
    let cwd = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let cwd_is_root = cwd.parent().is_none();
    which::which_in_all(SYSTEM_BWRAP_PROGRAM, Some(search_path), &cwd)
        .ok()?
        .find_map(|path| {
            let path = std::fs::canonicalize(path).ok()?;
            if !cwd_is_root && path.starts_with(&cwd) {
                None
            } else {
                Some(path)
            }
        })
}

/// The bubblewrap that came with Corbanu Terminal: in the install's
/// resources, or next to the executable (older package layouts and
/// development builds).
pub fn find_bundled_bwrap() -> Option<AbsolutePathBuf> {
    let current_exe = std::env::current_exe().ok()?;
    find_bundled_bwrap_for_install_context(InstallContext::current())
        .or_else(|| find_legacy_bundled_bwrap_for_exe(&current_exe))
}

fn find_bundled_bwrap_for_install_context(context: &InstallContext) -> Option<AbsolutePathBuf> {
    context
        .bundled_resource("bwrap")
        .filter(|path| is_executable_file(path))
}

fn find_legacy_bundled_bwrap_for_exe(exe: &Path) -> Option<AbsolutePathBuf> {
    legacy_bundled_bwrap_candidates_for_exe(exe)
        .into_iter()
        .find(|candidate| is_executable_file(candidate))
        .and_then(|path| AbsolutePathBuf::from_absolute_path(&path).ok())
}

fn legacy_bundled_bwrap_candidates_for_exe(exe: &Path) -> Vec<PathBuf> {
    let Some(exe_dir) = exe.parent() else {
        return Vec::new();
    };

    let mut candidates = vec![exe_dir.join("codex-resources").join("bwrap")];
    if let Some(package_target_dir) = exe_dir.parent() {
        candidates.push(package_target_dir.join("codex-resources").join("bwrap"));
    }
    candidates.push(exe_dir.join("bwrap"));
    candidates
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(test)]
#[path = "bwrap_tests.rs"]
mod tests;
