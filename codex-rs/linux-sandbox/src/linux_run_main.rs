use clap::Parser;
use std::ffi::CString;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use crate::bwrap::BwrapNetworkMode;
use crate::bwrap::BwrapOptions;
use crate::bwrap::create_bwrap_command_args;
use crate::landlock::apply_permission_profile_to_current_thread;
use crate::launcher::exec_bwrap;
use crate::launcher::preferred_bwrap_inherited_fd;
use crate::launcher::preferred_bwrap_supports_argv0;
use crate::proxy_routing::activate_proxy_routes_in_netns;
use crate::proxy_routing::prepare_host_proxy_route_spec;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::FileSystemAccessMode;
use codex_protocol::protocol::FileSystemPath;
use codex_protocol::protocol::FileSystemSandboxEntry;
use codex_protocol::protocol::FileSystemSandboxPolicy;
use codex_protocol::protocol::FileSystemSpecialPath;
use codex_protocol::protocol::NetworkSandboxPolicy;
use codex_sandboxing::landlock::CODEX_LINUX_SANDBOX_ARG0;

static BWRAP_CHILD_PID: AtomicI32 = AtomicI32::new(0);
static PENDING_FORWARDED_SIGNAL: AtomicI32 = AtomicI32::new(0);

const FORWARDED_SIGNALS: &[libc::c_int] =
    &[libc::SIGHUP, libc::SIGINT, libc::SIGQUIT, libc::SIGTERM];
const SYNTHETIC_MOUNT_MARKER_SYNTHETIC: &[u8] = b"synthetic\n";
const SYNTHETIC_MOUNT_MARKER_EXISTING: &[u8] = b"existing\n";
const PROTECTED_CREATE_MARKER: &[u8] = b"protected-create\n";
/// #582: in a marker folder, what the placeholder is and where (`file` or
/// `directory`, a newline, then the path), so a later run can remove it when
/// its owner is gone without cleaning up (killed with SIGKILL).
const SYNTHETIC_MOUNT_TARGET_RECORD: &str = "target";

#[derive(Debug)]
struct SyntheticMountTargetRegistration {
    target: crate::bwrap::SyntheticMountTarget,
    marker_file: PathBuf,
    marker_dir: PathBuf,
    /// #607: whether this run wrote the record (it makes the placeholder).
    wrote_record: bool,
}

#[derive(Debug)]
struct ProtectedCreateTargetRegistration {
    target: crate::bwrap::ProtectedCreateTarget,
    marker_file: PathBuf,
    marker_dir: PathBuf,
}

struct ProtectedCreateMonitor {
    stop: Arc<AtomicBool>,
    violation: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
}

struct ProtectedCreateWatcher {
    fd: libc::c_int,
    _watches: Vec<libc::c_int>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProtectedCreateRemoval {
    Directory,
    Other,
}

#[derive(Debug, Parser)]
/// CLI surface for the Linux sandbox helper.
///
/// The type name remains `LandlockCommand` for compatibility with existing
/// wiring, but bubblewrap is now the default filesystem sandbox and Landlock
/// is the legacy fallback.
pub struct LandlockCommand {
    /// It is possible that the cwd used in the context of the sandbox policy
    /// is different from the cwd of the process to spawn.
    #[arg(long = "sandbox-policy-cwd")]
    pub sandbox_policy_cwd: PathBuf,

    /// The logical working directory for the command being sandboxed.
    ///
    /// This can intentionally differ from `sandbox_policy_cwd` when the
    /// command runs from a symlinked alias of the policy workspace. Keep it
    /// explicit so bubblewrap can preserve the caller's logical cwd when that
    /// alias would otherwise disappear inside the sandbox namespace.
    #[arg(long = "command-cwd", hide = true)]
    pub command_cwd: Option<PathBuf>,

    /// Canonical runtime permissions for the command.
    #[arg(
        long = "permission-profile",
        hide = true,
        value_parser = parse_permission_profile
    )]
    pub permission_profile: Option<PermissionProfile>,

    /// Opt-in: use the legacy Landlock Linux sandbox fallback.
    ///
    /// When not set, the helper uses the default bubblewrap pipeline.
    #[arg(long = "use-legacy-landlock", hide = true, default_value_t = false)]
    pub use_legacy_landlock: bool,

    /// Internal: apply seccomp and `no_new_privs` in the already-sandboxed
    /// process, then exec the user command.
    ///
    /// This exists so we can run bubblewrap first (which may rely on setuid)
    /// and only tighten with seccomp after the filesystem view is established.
    #[arg(long = "apply-seccomp-then-exec", hide = true, default_value_t = false)]
    pub apply_seccomp_then_exec: bool,

    /// Internal compatibility flag.
    ///
    /// By default, restricted-network sandboxing uses isolated networking.
    /// If set, sandbox setup switches to proxy-only network mode with
    /// managed routing bridges.
    #[arg(long = "allow-network-for-proxy", hide = true, default_value_t = false)]
    pub allow_network_for_proxy: bool,

    /// Internal route spec used for managed proxy routing in bwrap mode.
    #[arg(long = "proxy-route-spec", hide = true)]
    pub proxy_route_spec: Option<String>,

    /// Internal (#536): the writable roots' real locations as the caller
    /// pinned them; each root is bound there, and the command is refused
    /// when a root no longer leads there.
    #[arg(
        long = "pinned-writable-roots",
        hide = true,
        value_parser = parse_pinned_writable_roots
    )]
    pub pinned_writable_roots: Option<PinnedWritableRoots>,

    /// Internal (#348): the folder (in the Corbanu home) where each run
    /// records the placeholders it makes, so one a run left behind is
    /// removed later even when the temporary folder's registry is lost.
    #[arg(long = "placeholder-record-dir", hide = true)]
    pub placeholder_record_dir: Option<PathBuf>,

    /// When set, skip mounting a fresh `/proc` even though PID isolation is
    /// still enabled. This is primarily intended for restrictive container
    /// environments that deny `--proc /proc`.
    #[arg(long = "no-proc", default_value_t = false)]
    pub no_proc: bool,

    /// Internal (#581): a descriptor the inner stage closes before running
    /// the command (the verified bundled bubblewrap that `aa-exec` ran).
    #[arg(long = "close-inherited-fd", hide = true)]
    pub close_inherited_fd: Option<i32>,

    /// Full command args to run under the Linux sandbox helper.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}

/// Entry point for the Linux sandbox helper.
///
/// The sequence is:
/// 1. When needed, wrap the command with bubblewrap to construct the
///    filesystem view.
/// 2. Apply in-process restrictions (no_new_privs + seccomp).
/// 3. `execvp` into the final command.
pub fn run_main() -> ! {
    let LandlockCommand {
        sandbox_policy_cwd,
        command_cwd,
        permission_profile,
        use_legacy_landlock,
        apply_seccomp_then_exec,
        allow_network_for_proxy,
        proxy_route_spec,
        pinned_writable_roots,
        placeholder_record_dir,
        no_proc,
        close_inherited_fd,
        command,
    } = LandlockCommand::parse();

    if command.is_empty() {
        panic!("No command specified to execute.");
    }
    ensure_inner_stage_mode_is_valid(apply_seccomp_then_exec, use_legacy_landlock);
    let EffectivePermissions {
        permission_profile,
        mut file_system_sandbox_policy,
        network_sandbox_policy,
    } = resolve_permission_profile(permission_profile).unwrap_or_else(|err| panic!("{err}"));
    ensure_legacy_landlock_mode_supports_policy(
        use_legacy_landlock,
        &file_system_sandbox_policy,
        network_sandbox_policy,
        &sandbox_policy_cwd,
    );

    // Inner stage: apply seccomp/no_new_privs after bubblewrap has already
    // established the filesystem view.
    if apply_seccomp_then_exec {
        // #581: first, so nothing started here (proxy bridges) inherits it.
        if let Some(fd) = close_inherited_fd.filter(|fd| *fd > libc::STDERR_FILENO) {
            // SAFETY: closes a descriptor this process does not use.
            unsafe {
                libc::close(fd);
            }
        }
        if allow_network_for_proxy {
            let spec = proxy_route_spec
                .as_deref()
                .unwrap_or_else(|| panic!("managed proxy mode requires --proxy-route-spec"));
            if let Err(err) = activate_proxy_routes_in_netns(spec) {
                panic!("error activating Linux proxy routing bridge: {err}");
            }
        }
        let proxy_routing_active = allow_network_for_proxy;
        if let Err(e) = apply_permission_profile_to_current_thread(
            &permission_profile,
            &sandbox_policy_cwd,
            /*apply_landlock_fs*/ false,
            allow_network_for_proxy,
            proxy_routing_active,
        ) {
            panic!("error applying Linux sandbox restrictions: {e:?}");
        }
        if file_system_sandbox_policy.denies_user_namespaces() {
            mark_user_namespaces_denied();
        }
        exec_or_panic(command);
    }

    if file_system_sandbox_policy.has_full_disk_write_access() && !allow_network_for_proxy {
        run_supervised_without_bwrap(|| {
            if let Err(e) = apply_permission_profile_to_current_thread(
                &permission_profile,
                &sandbox_policy_cwd,
                /*apply_landlock_fs*/ false,
                allow_network_for_proxy,
                /*proxy_routed_network*/ false,
            ) {
                panic!("error applying Linux sandbox restrictions: {e:?}");
            }
            if file_system_sandbox_policy.denies_user_namespaces() {
                mark_user_namespaces_denied();
            }
            exec_or_panic(command);
        });
    }

    if let Some(PinnedWritableRoots(pinned)) = pinned_writable_roots {
        // #536: bubblewrap and legacy Landlock grant each writable root on
        // the folder opened and checked at its pinned location.
        crate::pinned::set(pinned);
    }

    if !use_legacy_landlock {
        // #826: bubblewrap needs a user namespace, which a command run in a
        // sandbox that denies them cannot create; say so instead of leaving
        // it to bubblewrap's message.
        refuse_when_user_namespaces_denied();
        // #348: before any argument is built, so the record folder is there
        // (and masked where the policy denies it) when the command runs.
        if let Some(dir) = placeholder_record_dir.as_deref() {
            crate::placeholder_journal::open(dir);
        }
        // Outer stage: bubblewrap first, then re-enter this binary in the
        // sandboxed environment to apply seccomp. This path never falls back
        // to legacy Landlock on failure.
        let proxy_route_spec = if allow_network_for_proxy {
            let (proxy_route_spec, socket_dir) = prepare_host_proxy_route_spec()
                .unwrap_or_else(|err| panic!("failed to prepare host proxy routing bridge: {err}"));
            file_system_sandbox_policy = file_system_sandbox_policy.with_additional_readable_roots(
                &sandbox_policy_cwd,
                std::slice::from_ref(&socket_dir),
            );
            Some(proxy_route_spec)
        } else {
            None
        };
        let inner = build_inner_seccomp_command(InnerSeccompCommandArgs {
            sandbox_policy_cwd: &sandbox_policy_cwd,
            command_cwd: command_cwd.as_deref(),
            permission_profile: &permission_profile,
            allow_network_for_proxy,
            proxy_route_spec,
            close_inherited_fd: preferred_bwrap_inherited_fd(),
            command,
        });
        run_bwrap_with_proc_fallback(
            &sandbox_policy_cwd,
            command_cwd.as_deref(),
            &file_system_sandbox_policy,
            network_sandbox_policy,
            inner,
            !no_proc,
            allow_network_for_proxy,
        );
    }

    // Legacy path: Landlock enforcement only, when bwrap sandboxing is not enabled.
    run_supervised_without_bwrap(|| {
        if let Err(e) = apply_permission_profile_to_current_thread(
            &permission_profile,
            &sandbox_policy_cwd,
            /*apply_landlock_fs*/ true,
            allow_network_for_proxy,
            /*proxy_routed_network*/ false,
        ) {
            panic!("error applying legacy Linux sandbox restrictions: {e:?}");
        }
        if file_system_sandbox_policy.denies_user_namespaces() {
            mark_user_namespaces_denied();
        }
        exec_or_panic(command);
    });
}

/// #1006 (Travis, 2026-10-20, D47 option B): a command run without
/// bubblewrap (full disk write access without managed-proxy routing, legacy
/// Landlock) cannot outlive this helper either. `exec_command` (restrict,
/// then exec) runs under #935's supervisor, in this helper's process group
/// and session (the terminal's foreground group, if it was), with the
/// original signal mask and `SIGCHLD` action. The supervisor kills every
/// process it leaves behind when the command exits, when this helper exits
/// or is killed, or when this helper's parent exits, and exits as the
/// command did; this helper then exits the same way.
///
/// Signals: one sent to the process group reaches the command directly, so
/// this helper keeps every signal blocked and ignores it, except the
/// forwarded ones, which the supervisor relays once the command has left
/// the group ([`crate::supervisor::Relay`]). A catchable signal sent to this
/// helper's pid alone no longer reaches the command, except a forwarded one
/// sent before this helper forks the supervisor or once the command has
/// left the group (Corbanu signals the group, or kills this helper with
/// `SIGKILL`, which stops the command).
fn run_supervised_without_bwrap(exec_command: impl FnOnce()) -> ! {
    // SAFETY: plain syscall on this process, still single-threaded.
    let group = unsafe { libc::getpgrp() };
    let original_mask = crate::supervisor::block_all_signals();
    // SAFETY: plain data, valid when zeroed.
    let mut original_sigchld: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: valid in and out pointers; the supervisor is waited for, and
    // an ignored SIGCHLD (inherited across exec) would reap it unseen.
    unsafe {
        let mut default: libc::sigaction = std::mem::zeroed();
        default.sa_sigaction = libc::SIG_DFL;
        libc::sigaction(libc::SIGCHLD, &default, &mut original_sigchld);
    }
    let mut watched = FORWARDED_SIGNALS.to_vec();
    watched.push(libc::SIGCHLD);
    let signals = crate::supervisor::SignalFd::new(&watched);
    // Without a pidfd, the parent's exit kills this helper (it blocks the
    // signals it could catch), which cuts the lifeline.
    let parent_watch = crate::supervisor::ParentWatch::open(libc::SIGKILL);
    let (lifeline_read, lifeline) = crate::supervisor::lifeline();
    let (mut relay, relay_requests) = crate::supervisor::Relay::pipe(group, FORWARDED_SIGNALS);
    // Received before the command exists: sent to it as it starts, and not
    // relayed later (read here, they are gone).
    while let Some((signal, _)) = signals.try_next() {
        relay.add_pending(signal);
    }
    // SAFETY: this process is single-threaded.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to fork the sandbox supervisor: {err}");
    }
    if pid == 0 {
        drop(lifeline);
        drop(relay_requests);
        drop(signals);
        crate::supervisor::run(lifeline_read, &[], &[], Some(relay), |supervisor_pid| {
            terminate_with_parent(supervisor_pid, libc::SIGKILL);
            // SAFETY: plain syscalls on this process, restoring what it
            // was started with.
            unsafe {
                libc::sigaction(libc::SIGCHLD, &original_sigchld, std::ptr::null_mut());
                libc::sigprocmask(libc::SIG_SETMASK, &original_mask, std::ptr::null_mut());
            }
            exec_command();
        });
    }
    drop(lifeline_read);
    drop(relay);
    parent_watch.cut_when_gone(Arc::new(lifeline));
    loop {
        let (signal, code) = signals.next();
        if signal != libc::SIGCHLD {
            // A terminal hangup signals the session leader alone.
            let hangup = signal == libc::SIGHUP && code == libc::SI_KERNEL;
            crate::supervisor::Relay::request(&relay_requests, signal, hangup);
            continue;
        }
        let mut status = 0;
        // SAFETY: valid out pointer; `pid` is this process's child.
        let res = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if res == pid {
            crate::supervisor::exit_like(status);
        }
        if res < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
            let err = std::io::Error::last_os_error();
            panic!("waitpid failed for the sandbox supervisor: {err}");
        }
    }
}

/// #826: set in the environment of a command whose sandbox denies user
/// namespaces. It only selects the message below; whether a namespace can
/// be created is always tried.
const USER_NAMESPACES_DENIED_ENV: &str = "CODEX_SANDBOX_USER_NAMESPACES_DENIED";

/// Shown when this helper runs inside a sandbox that denies user namespaces.
const USER_NAMESPACES_DENIED_MESSAGE: &str = "Corbanu's Linux sandbox cannot start here: this command runs inside an agent command's sandbox under security level Moderate or Aggressive, where creating user namespaces is denied (Operation not permitted), and the sandbox needs one. Run it outside the agent's sandbox (under Moderate a person may allow user namespaces for one command whose request to run outside the sandbox they approve when asked: before untrusted content any command, in practice the first command of a new session; after it only one plain command that runs unshare, bwrap, podman, buildah, flatpak or nix), or choose Permissive in /security.";

fn mark_user_namespaces_denied() {
    // SAFETY: this helper process is single-threaded here (the proxy bridges
    // run in forked processes), and the environment changes just before the
    // command is executed.
    unsafe {
        std::env::set_var(USER_NAMESPACES_DENIED_ENV, "1");
    }
}

/// #826: inside a sandbox that denies user namespaces, stops with
/// [`USER_NAMESPACES_DENIED_MESSAGE`] when one really cannot be created.
fn refuse_when_user_namespaces_denied() {
    if std::env::var_os(USER_NAMESPACES_DENIED_ENV).is_none() {
        return;
    }
    if user_namespace_creation_error() == Some(libc::EPERM) {
        eprintln!("{USER_NAMESPACES_DENIED_MESSAGE}");
        std::process::exit(1);
    }
}

/// The error creating a user namespace fails with in a child of this
/// process, `None` when it succeeds or cannot be tried.
fn user_namespace_creation_error() -> Option<i32> {
    // SAFETY: the child calls only async-signal-safe functions, then exits.
    let pid = unsafe { libc::fork() };
    match pid {
        -1 => None,
        0 => unsafe {
            let code = if libc::unshare(libc::CLONE_NEWUSER) == 0 {
                0
            } else {
                *libc::__errno_location()
            };
            libc::_exit(code.clamp(0, 255));
        },
        pid => {
            let mut status = 0;
            loop {
                // SAFETY: waits for the child forked above.
                let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
                if waited == pid {
                    break;
                }
                if waited == -1
                    && std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR)
                {
                    return None;
                }
            }
            (libc::WIFEXITED(status) && libc::WEXITSTATUS(status) != 0)
                .then(|| libc::WEXITSTATUS(status))
        }
    }
}

#[derive(Debug, Clone)]
struct EffectivePermissions {
    permission_profile: PermissionProfile,
    file_system_sandbox_policy: FileSystemSandboxPolicy,
    network_sandbox_policy: NetworkSandboxPolicy,
}

#[derive(Debug, PartialEq, Eq)]
enum ResolvePermissionProfileError {
    MissingConfiguration,
}

impl fmt::Display for ResolvePermissionProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfiguration => write!(f, "missing permission profile configuration"),
        }
    }
}

/// The `--pinned-writable-roots` value (#536).
#[derive(Debug, Clone)]
pub struct PinnedWritableRoots(pub Vec<codex_protocol::permissions::PinnedWritableRoot>);

fn parse_pinned_writable_roots(value: &str) -> std::result::Result<PinnedWritableRoots, String> {
    serde_json::from_str(value)
        .map(PinnedWritableRoots)
        .map_err(|err| format!("invalid pinned writable roots: {err}"))
}

fn parse_permission_profile(value: &str) -> std::result::Result<PermissionProfile, String> {
    serde_json::from_str(value).map_err(|err| format!("invalid permission profile JSON: {err}"))
}

fn resolve_permission_profile(
    permission_profile: Option<PermissionProfile>,
) -> Result<EffectivePermissions, ResolvePermissionProfileError> {
    let permission_profile =
        permission_profile.ok_or(ResolvePermissionProfileError::MissingConfiguration)?;
    let (file_system_sandbox_policy, network_sandbox_policy) =
        permission_profile.to_runtime_permissions();
    Ok(EffectivePermissions {
        permission_profile,
        file_system_sandbox_policy,
        network_sandbox_policy,
    })
}

fn ensure_inner_stage_mode_is_valid(apply_seccomp_then_exec: bool, use_legacy_landlock: bool) {
    if apply_seccomp_then_exec && use_legacy_landlock {
        panic!("--apply-seccomp-then-exec is incompatible with --use-legacy-landlock");
    }
}

fn ensure_legacy_landlock_mode_supports_policy(
    use_legacy_landlock: bool,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    network_sandbox_policy: NetworkSandboxPolicy,
    sandbox_policy_cwd: &Path,
) {
    if use_legacy_landlock
        && file_system_sandbox_policy
            .needs_direct_runtime_enforcement(network_sandbox_policy, sandbox_policy_cwd)
    {
        panic!(
            "permission profiles requiring direct runtime enforcement are incompatible with --use-legacy-landlock"
        );
    }
}

fn run_bwrap_with_proc_fallback(
    sandbox_policy_cwd: &Path,
    command_cwd: Option<&Path>,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    network_sandbox_policy: NetworkSandboxPolicy,
    inner: Vec<String>,
    mount_proc: bool,
    allow_network_for_proxy: bool,
) -> ! {
    let network_mode = bwrap_network_mode(network_sandbox_policy, allow_network_for_proxy);
    let mut mount_proc = mount_proc;
    let command_cwd = command_cwd.unwrap_or(sandbox_policy_cwd);
    // #582: once, before any arguments are built, so a stale placeholder is
    // not taken for an existing path (or a file where a folder belongs).
    remove_stale_synthetic_mount_targets();
    // #348: then those whose runs' records in the temporary folder were lost
    // (a reboot), from the record kept in the Corbanu home.
    sweep_recorded_placeholders();

    if mount_proc
        && !preflight_proc_mount_support(network_mode)
            .unwrap_or_else(|err| exit_with_bwrap_build_error(err))
    {
        // Keep the retry silent so sandbox-internal diagnostics do not leak into the
        // child process stderr stream.
        mount_proc = false;
    }

    let options = BwrapOptions {
        mount_proc,
        network_mode,
        pins: crate::pinned::get().map(|roots| crate::bwrap::WritableRootPins {
            roots,
            bind_fd: crate::launcher::bind_fd_available(),
        }),
        ..Default::default()
    };
    // #442: watch before the existing credential files are found and hidden,
    // so none created in between is missed.
    let credential_watcher = crate::credential_watch::CredentialWatcher::for_policy(
        file_system_sandbox_policy,
        sandbox_policy_cwd,
    )
    .unwrap_or_else(|err| {
        eprintln!("{}", crate::credential_watch::refused_message(&err));
        std::process::exit(1);
    });
    let mut file_system_sandbox_policy = file_system_sandbox_policy.clone();
    if let Some(watcher) = &credential_watcher {
        for target in watcher.existing_files() {
            if let Ok(path) = codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path(target)
            {
                file_system_sandbox_policy
                    .entries
                    .push(FileSystemSandboxEntry::new(
                        FileSystemPath::Path { path },
                        FileSystemAccessMode::Deny,
                    ));
            }
        }
    }
    let mut bwrap_args = build_bwrap_argv(
        inner,
        &file_system_sandbox_policy,
        sandbox_policy_cwd,
        command_cwd,
        options,
    )
    .unwrap_or_else(|err| exit_with_bwrap_build_error(err));
    if file_system_sandbox_policy.denies_user_namespaces() {
        add_user_namespace_seccomp_fds(&mut bwrap_args).unwrap_or_else(|err| {
            eprintln!("error building bubblewrap command: {err}");
            std::process::exit(1);
        });
    }
    apply_inner_command_argv0(&mut bwrap_args.args);
    run_or_exec_bwrap(bwrap_args, credential_watcher);
}

/// #826: hands bubblewrap the user-namespace seccomp programs
/// (`--add-seccomp-fd`, bubblewrap 0.5+, as `--perms` that a system copy must
/// support), which it applies to every process in the sandbox before it runs
/// anything there. That includes its own PID 1, which is no descendant of the
/// command, so the inner stage cannot filter it, and which would otherwise
/// create namespaces for a command that takes it over (`ptrace`,
/// `/proc/1/mem`).
fn add_user_namespace_seccomp_fds(bwrap_args: &mut crate::bwrap::BwrapArgs) -> Result<(), String> {
    let programs = crate::landlock::user_namespace_seccomp_programs()
        .map_err(|err| format!("cannot build the user-namespace seccomp programs: {err}"))?;
    let mut args = Vec::new();
    for program in programs {
        let bytes: Vec<u8> = program
            .iter()
            .flat_map(|instruction| {
                let mut bytes = Vec::with_capacity(8);
                bytes.extend_from_slice(&instruction.code.to_ne_bytes());
                bytes.push(instruction.jt);
                bytes.push(instruction.jf);
                bytes.extend_from_slice(&instruction.k.to_ne_bytes());
                bytes
            })
            .collect();
        let file = seccomp_program_file(&bytes)
            .map_err(|err| format!("cannot pass a seccomp program to bubblewrap: {err}"))?;
        args.push("--add-seccomp-fd".to_string());
        args.push(file.as_raw_fd().to_string());
        bwrap_args.preserved_files.push(file);
    }
    // Right after the program name, before the command.
    bwrap_args.args.splice(1..1, args);
    Ok(())
}

/// An in-memory file holding `bytes`, read from the start, above
/// the standard streams (so it never becomes the command's stdin, stdout or
/// stderr where one of them was closed).
fn seccomp_program_file(bytes: &[u8]) -> std::io::Result<File> {
    use std::io::Seek;
    use std::io::Write;
    // SAFETY: creates a new descriptor from a valid C string.
    let fd = unsafe { libc::memfd_create(c"codex-userns-seccomp".as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `fd` was just created and is owned here.
    let created = unsafe { File::from_raw_fd(fd) };
    // SAFETY: duplicates a descriptor owned here.
    let raised = unsafe { libc::fcntl(created.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
    if raised < 0 {
        return Err(std::io::Error::last_os_error());
    }
    drop(created);
    // SAFETY: `raised` was just created and is owned here.
    let mut file = unsafe { File::from_raw_fd(raised) };
    file.write_all(bytes)?;
    file.seek(std::io::SeekFrom::Start(0))?;
    Ok(file)
}

fn bwrap_network_mode(
    network_sandbox_policy: NetworkSandboxPolicy,
    allow_network_for_proxy: bool,
) -> BwrapNetworkMode {
    if allow_network_for_proxy {
        BwrapNetworkMode::ProxyOnly
    } else if network_sandbox_policy.is_enabled() {
        BwrapNetworkMode::FullAccess
    } else {
        BwrapNetworkMode::Isolated
    }
}

fn build_bwrap_argv(
    inner: Vec<String>,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    sandbox_policy_cwd: &Path,
    command_cwd: &Path,
    options: BwrapOptions,
) -> CodexResult<crate::bwrap::BwrapArgs> {
    let bwrap_args = create_bwrap_command_args(
        inner,
        file_system_sandbox_policy,
        sandbox_policy_cwd,
        command_cwd,
        options,
    )?;

    let mut argv = vec!["bwrap".to_string()];
    argv.extend(bwrap_args.args);
    Ok(crate::bwrap::BwrapArgs {
        args: argv,
        preserved_files: bwrap_args.preserved_files,
        synthetic_mount_targets: bwrap_args.synthetic_mount_targets,
        protected_create_targets: bwrap_args.protected_create_targets,
        registry_lock: bwrap_args.registry_lock,
    })
}

fn exit_with_bwrap_build_error(err: codex_protocol::error::CodexErr) -> ! {
    eprintln!("error building bubblewrap command: {err}");
    std::process::exit(1);
}

fn apply_inner_command_argv0(argv: &mut Vec<String>) {
    apply_inner_command_argv0_for_launcher(
        argv,
        preferred_bwrap_supports_argv0(),
        current_process_argv0(),
    );
}

fn apply_inner_command_argv0_for_launcher(
    argv: &mut Vec<String>,
    supports_argv0: bool,
    argv0_fallback_command: String,
) {
    let command_separator_index = argv
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or_else(|| panic!("bubblewrap argv is missing command separator '--'"));

    if supports_argv0 {
        argv.splice(
            command_separator_index..command_separator_index,
            ["--argv0".to_string(), CODEX_LINUX_SANDBOX_ARG0.to_string()],
        );
        return;
    }

    let command_index = command_separator_index + 1;
    let Some(command) = argv.get_mut(command_index) else {
        panic!("bubblewrap argv is missing inner command after '--'");
    };
    *command = argv0_fallback_command;
}

fn current_process_argv0() -> String {
    match std::env::args_os().next() {
        Some(argv0) => argv0.to_string_lossy().into_owned(),
        None => panic!("failed to resolve current process argv[0]"),
    }
}

fn preflight_proc_mount_support(network_mode: BwrapNetworkMode) -> CodexResult<bool> {
    let preflight_argv = build_preflight_bwrap_argv(network_mode)?;
    let stderr = run_bwrap_in_child_capture_stderr(preflight_argv);
    Ok(!is_proc_mount_failure(stderr.as_str()))
}

fn build_preflight_bwrap_argv(
    network_mode: BwrapNetworkMode,
) -> CodexResult<crate::bwrap::BwrapArgs> {
    let file_system_sandbox_policy =
        FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::Minimal,
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        }]);
    let preflight_command = vec![resolve_true_command()];
    build_bwrap_argv(
        preflight_command,
        &file_system_sandbox_policy,
        Path::new("/"),
        Path::new("/"),
        BwrapOptions {
            mount_proc: true,
            network_mode,
            ..Default::default()
        },
    )
}

fn resolve_true_command() -> String {
    for candidate in ["/usr/bin/true", "/bin/true"] {
        if Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "true".to_string()
}

/// #935: bubblewrap always runs under the supervisor (`crate::supervisor`),
/// so the sandbox cannot outlive this helper; also where there is nothing
/// to clean up afterwards (this used to exec bubblewrap directly then).
fn run_or_exec_bwrap(
    bwrap_args: crate::bwrap::BwrapArgs,
    credential_watcher: Option<crate::credential_watch::CredentialWatcher>,
) -> ! {
    run_bwrap_in_child_with_synthetic_mount_cleanup(bwrap_args, credential_watcher);
}

fn run_bwrap_in_child_with_synthetic_mount_cleanup(
    bwrap_args: crate::bwrap::BwrapArgs,
    credential_watcher: Option<crate::credential_watch::CredentialWatcher>,
) -> ! {
    let crate::bwrap::BwrapArgs {
        args,
        preserved_files,
        synthetic_mount_targets,
        protected_create_targets,
        registry_lock,
    } = bwrap_args;
    // #935: the supervisor is waited for; an ignored SIGCHLD (inherited
    // across exec) would reap it unseen.
    // SAFETY: plain syscall on this process.
    unsafe {
        libc::signal(libc::SIGCHLD, libc::SIG_DFL);
    }
    let setup_signal_mask = ForwardedSignalMask::block();
    let (synthetic_mount_registrations, protected_create_registrations) = register_mount_targets(
        &synthetic_mount_targets,
        &protected_create_targets,
        registry_lock,
        &args,
    );
    let exec_start_pipe = create_exec_start_pipe(
        !protected_create_targets.is_empty() || credential_watcher.is_some(),
    );
    let placeholders = synthetic_mount_targets
        .iter()
        .map(|target| target.path().to_path_buf())
        .collect::<Vec<_>>();
    // #935: the supervisor stops the sandbox when this helper's parent
    // exits, when this helper exits or is killed, or on a credential stop.
    let parent_watch = crate::supervisor::ParentWatch::open(libc::SIGTERM);
    let (lifeline_read, lifeline) = crate::supervisor::lifeline();
    // `pid` is the supervisor, which runs bubblewrap and exits as it did.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to fork for bubblewrap: {err}");
    }

    if pid == 0 {
        drop(lifeline);
        reset_forwarded_signal_handlers_to_default();
        // Bubblewrap shares the supervisor's process group, where forwarded
        // signals are sent; they stay blocked in the supervisor itself.
        let setpgid_res = unsafe { libc::setpgid(0, 0) };
        if setpgid_res < 0 {
            let err = std::io::Error::last_os_error();
            panic!("failed to place bubblewrap child in its own process group: {err}");
        }
        crate::supervisor::run(
            lifeline_read,
            FORWARDED_SIGNALS,
            &exec_start_pipe,
            /*relay*/ None,
            |supervisor_pid| {
                setup_signal_mask.restore();
                terminate_with_parent(supervisor_pid, libc::SIGTERM);
                wait_for_parent_exec_start(exec_start_pipe[0], exec_start_pipe[1]);
                exec_bwrap(args, preserved_files);
            },
        );
    }

    // Also here, so the group exists before any signal is forwarded to it.
    // SAFETY: plain syscall on this process's child, which never execs.
    unsafe {
        libc::setpgid(pid, pid);
    }
    drop(lifeline_read);
    let lifeline = Arc::new(lifeline);
    parent_watch.cut_when_gone(Arc::clone(&lifeline));
    close_child_exec_start_read(exec_start_pipe[0]);
    let credential_monitor = credential_watcher.map(|watcher| {
        let lifeline = Arc::clone(&lifeline);
        watcher.start(move || lifeline.cut(), &placeholders)
    });
    let protected_create_monitor = ProtectedCreateMonitor::start(&protected_create_targets);
    let signal_forwarders = install_bwrap_signal_forwarders(pid);
    release_child_exec_start(exec_start_pipe[1]);
    setup_signal_mask.restore();
    // Stop the credential watcher while the child is still unreaped, so a
    // late stop can never signal a reused pid.
    let credential_stop = credential_monitor.and_then(|monitor| {
        wait_for_bwrap_child_exit_without_reaping(pid);
        monitor.stop()
    });
    // Stop forwarding before the pid can be reused (forwarded signals are
    // handled on this thread only: the others started with them blocked).
    wait_for_bwrap_child_exit_without_reaping(pid);
    BWRAP_CHILD_PID.store(0, Ordering::SeqCst);
    let status = wait_for_bwrap_child(pid);
    let cleanup_signal_mask = ForwardedSignalMask::block();
    let protected_create_monitor_violation = protected_create_monitor
        .map(ProtectedCreateMonitor::stop)
        .unwrap_or(false);
    // #348: this run no longer uses anything; then its cleanup.
    crate::placeholder_journal::leave();
    cleanup_synthetic_mount_targets(&synthetic_mount_registrations);
    let protected_create_violation = protected_create_monitor_violation
        || cleanup_protected_create_targets(&protected_create_registrations);
    // #348: what this run made and another live run still holds stays
    // recorded (for a later sweep); the rest of its record goes.
    crate::placeholder_journal::finish();
    signal_forwarders.restore();
    cleanup_signal_mask.restore();
    if let Some(found) = credential_stop {
        eprintln!("{}", crate::credential_watch::stopped_message(&found));
        std::process::exit(1);
    }
    exit_with_wait_status_or_policy_violation(status, protected_create_violation);
}

impl ProtectedCreateMonitor {
    fn start(targets: &[crate::bwrap::ProtectedCreateTarget]) -> Option<Self> {
        if targets.is_empty() {
            return None;
        }

        let targets = targets.to_vec();
        let stop = Arc::new(AtomicBool::new(false));
        let violation = Arc::new(AtomicBool::new(false));
        let monitor_stop = Arc::clone(&stop);
        let monitor_violation = Arc::clone(&violation);
        let handle = thread::spawn(move || {
            let watcher = ProtectedCreateWatcher::new(&targets);
            while !monitor_stop.load(Ordering::SeqCst) {
                for target in &targets {
                    if remove_protected_create_target_best_effort(target).is_some() {
                        monitor_violation.store(true, Ordering::SeqCst);
                    }
                }
                if let Some(watcher) = &watcher {
                    watcher.wait_for_create_event(&monitor_stop);
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        Some(Self {
            stop,
            violation,
            handle,
        })
    }

    fn stop(self) -> bool {
        self.stop.store(true, Ordering::SeqCst);
        self.handle
            .join()
            .unwrap_or_else(|_| panic!("protected create monitor thread panicked"));
        self.violation.load(Ordering::SeqCst)
    }
}

impl ProtectedCreateWatcher {
    fn new(targets: &[crate::bwrap::ProtectedCreateTarget]) -> Option<Self> {
        let fd = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
        if fd < 0 {
            return None;
        }

        let mut watched_parents = Vec::<PathBuf>::new();
        let mut watches = Vec::new();
        for target in targets {
            let Some(parent) = target.path().parent() else {
                continue;
            };
            if watched_parents.iter().any(|watched| watched == parent) {
                continue;
            }
            watched_parents.push(parent.to_path_buf());
            let Ok(parent_cstr) = CString::new(parent.as_os_str().as_bytes()) else {
                continue;
            };
            let mask =
                libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_DELETE_SELF | libc::IN_MOVE_SELF;
            let watch = unsafe { libc::inotify_add_watch(fd, parent_cstr.as_ptr(), mask) };
            if watch >= 0 {
                watches.push(watch);
            }
        }

        if watches.is_empty() {
            unsafe {
                libc::close(fd);
            }
            return None;
        }

        Some(Self {
            fd,
            _watches: watches,
        })
    }

    fn wait_for_create_event(&self, stop: &AtomicBool) {
        let mut poll_fd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };
        while !stop.load(Ordering::SeqCst) {
            let res = unsafe { libc::poll(&mut poll_fd, 1, 10) };
            if res > 0 {
                self.drain_events();
                return;
            }
            if res == 0 {
                return;
            }
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return;
        }
    }

    fn drain_events(&self) {
        let mut buf = [0_u8; 4096];
        loop {
            let read = unsafe { libc::read(self.fd, buf.as_mut_ptr().cast(), buf.len()) };
            if read > 0 {
                continue;
            }
            if read == 0 {
                return;
            }
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return;
        }
    }
}

impl Drop for ProtectedCreateWatcher {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

fn create_exec_start_pipe(enabled: bool) -> [libc::c_int; 2] {
    if !enabled {
        return [-1, -1];
    }
    let mut pipe = [-1, -1];
    if unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to create bubblewrap exec start pipe: {err}");
    }
    pipe
}

fn wait_for_parent_exec_start(read_fd: libc::c_int, write_fd: libc::c_int) {
    if write_fd >= 0 {
        unsafe {
            libc::close(write_fd);
        }
    }
    if read_fd < 0 {
        return;
    }

    let mut byte = [0_u8; 1];
    let started = loop {
        let read = unsafe { libc::read(read_fd, byte.as_mut_ptr().cast(), byte.len()) };
        if read >= 0 {
            break read == 1;
        }
        let err = std::io::Error::last_os_error();
        if err.kind() != std::io::ErrorKind::Interrupted {
            break false;
        }
    };
    unsafe {
        libc::close(read_fd);
    }
    if !started {
        // #935: the helper died before its watchers ran; never start the
        // command unwatched.
        // SAFETY: ends this forked child without unwinding.
        unsafe { libc::_exit(127) }
    }
}

fn close_child_exec_start_read(read_fd: libc::c_int) {
    if read_fd >= 0 {
        unsafe {
            libc::close(read_fd);
        }
    }
}

fn release_child_exec_start(write_fd: libc::c_int) {
    if write_fd < 0 {
        return;
    }
    let byte = [0_u8; 1];
    unsafe {
        libc::write(write_fd, byte.as_ptr().cast(), byte.len());
        libc::close(write_fd);
    }
}

struct ForwardedSignalMask {
    previous: libc::sigset_t,
}

struct ForwardedSignalHandlers {
    previous: Vec<(libc::c_int, libc::sigaction)>,
}

impl ForwardedSignalMask {
    fn block() -> Self {
        let mut blocked: libc::sigset_t = unsafe { std::mem::zeroed() };
        let mut previous: libc::sigset_t = unsafe { std::mem::zeroed() };
        unsafe {
            libc::sigemptyset(&mut blocked);
            for signal in FORWARDED_SIGNALS {
                libc::sigaddset(&mut blocked, *signal);
            }
            if libc::sigprocmask(libc::SIG_BLOCK, &blocked, &mut previous) < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to block bubblewrap forwarded signals: {err}");
            }
        }
        Self { previous }
    }

    fn restore(&self) {
        let mut restored = self.previous;
        unsafe {
            for signal in FORWARDED_SIGNALS {
                libc::sigdelset(&mut restored, *signal);
            }
            if libc::sigprocmask(libc::SIG_SETMASK, &restored, std::ptr::null_mut()) < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to restore bubblewrap forwarded signals: {err}");
            }
        }
    }
}

fn terminate_with_parent(parent_pid: libc::pid_t, signal: libc::c_int) {
    let res = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, signal) };
    if res < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to set the sandboxed child's parent-death signal: {err}");
    }
    if unsafe { libc::getppid() } != parent_pid {
        unsafe {
            libc::raise(signal);
        }
    }
}

impl ForwardedSignalHandlers {
    fn restore(self) {
        BWRAP_CHILD_PID.store(0, Ordering::SeqCst);
        PENDING_FORWARDED_SIGNAL.store(0, Ordering::SeqCst);
        for (signal, previous_action) in self.previous {
            unsafe {
                if libc::sigaction(signal, &previous_action, std::ptr::null_mut()) < 0 {
                    let err = std::io::Error::last_os_error();
                    panic!("failed to restore bubblewrap signal handler for {signal}: {err}");
                }
            }
        }
    }
}

fn install_bwrap_signal_forwarders(pid: libc::pid_t) -> ForwardedSignalHandlers {
    BWRAP_CHILD_PID.store(pid, Ordering::SeqCst);
    let mut previous = Vec::with_capacity(FORWARDED_SIGNALS.len());
    for signal in FORWARDED_SIGNALS {
        let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
        let mut previous_action: libc::sigaction = unsafe { std::mem::zeroed() };
        action.sa_sigaction = forward_signal_to_bwrap_child as *const () as libc::sighandler_t;
        unsafe {
            libc::sigemptyset(&mut action.sa_mask);
            if libc::sigaction(*signal, &action, &mut previous_action) < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to install bubblewrap signal forwarder for {signal}: {err}");
            }
        }
        previous.push((*signal, previous_action));
    }
    replay_pending_forwarded_signal(pid);
    ForwardedSignalHandlers { previous }
}

extern "C" fn forward_signal_to_bwrap_child(signal: libc::c_int) {
    PENDING_FORWARDED_SIGNAL.store(signal, Ordering::SeqCst);
    let pid = BWRAP_CHILD_PID.load(Ordering::SeqCst);
    if pid > 0 {
        send_signal_to_bwrap_child(pid, signal);
    }
}

fn replay_pending_forwarded_signal(pid: libc::pid_t) {
    let signal = PENDING_FORWARDED_SIGNAL.swap(0, Ordering::SeqCst);
    if signal > 0 {
        send_signal_to_bwrap_child(pid, signal);
    }
}

fn send_signal_to_bwrap_child(pid: libc::pid_t, signal: libc::c_int) {
    unsafe {
        libc::kill(-pid, signal);
        libc::kill(pid, signal);
    }
}

fn reset_forwarded_signal_handlers_to_default() {
    for signal in FORWARDED_SIGNALS {
        unsafe {
            if libc::signal(*signal, libc::SIG_DFL) == libc::SIG_ERR {
                let err = std::io::Error::last_os_error();
                panic!("failed to reset bubblewrap signal handler for {signal}: {err}");
            }
        }
    }
}

/// Returns once `pid` has exited, leaving it to be reaped.
fn wait_for_bwrap_child_exit_without_reaping(pid: libc::pid_t) {
    let Ok(id) = libc::id_t::try_from(pid) else {
        return;
    };
    loop {
        // SAFETY: `info` is a valid out pointer for `waitid`.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: plain syscall on this process's own child.
        let res =
            unsafe { libc::waitid(libc::P_PID, id, &mut info, libc::WEXITED | libc::WNOWAIT) };
        if res == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
            return;
        }
    }
}

fn wait_for_bwrap_child(pid: libc::pid_t) -> libc::c_int {
    loop {
        let mut status: libc::c_int = 0;
        let wait_res = unsafe { libc::waitpid(pid, &mut status as *mut libc::c_int, 0) };
        if wait_res >= 0 {
            return status;
        }
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        panic!("waitpid failed for bubblewrap child: {err}");
    }
}

/// Registers a run's placeholders and protected-create targets, then
/// releases the registry lock its arguments were built under (#673,
/// [`RegistryLock`]).
fn register_mount_targets(
    synthetic_mount_targets: &[crate::bwrap::SyntheticMountTarget],
    protected_create_targets: &[crate::bwrap::ProtectedCreateTarget],
    registry_lock: Option<RegistryLock>,
    args: &[String],
) -> (
    Vec<SyntheticMountTargetRegistration>,
    Vec<ProtectedCreateTargetRegistration>,
) {
    // #673: with nothing to register, no registry lock either.
    let (synthetic, protected) =
        if synthetic_mount_targets.is_empty() && protected_create_targets.is_empty() {
            (Vec::new(), Vec::new())
        } else {
            (
                register_synthetic_mount_targets(synthetic_mount_targets),
                register_protected_create_targets(protected_create_targets),
            )
        };
    record_mount_targets(&synthetic, args);
    drop(registry_lock);
    (synthetic, protected)
}

/// #348: records in the Corbanu home the placeholders this run makes (and
/// makes them; bubblewrap then mounts over them) and the paths it uses: its
/// placeholders, and the empty files and folders it mounts at that another
/// run recorded making (a placeholder a sweep must leave while this run
/// mounts over it). Still under the lock the arguments were built under,
/// which keeps out the record's sweeps ([`RegistryLock`]).
fn record_mount_targets(registrations: &[SyntheticMountTargetRegistration], args: &[String]) {
    let planned = registrations
        .iter()
        .map(|registration| crate::placeholder_journal::Planned {
            kind: match registration.target.kind() {
                crate::bwrap::SyntheticMountTargetKind::EmptyFile => {
                    crate::placeholder_journal::Kind::File
                }
                crate::bwrap::SyntheticMountTargetKind::EmptyDirectory => {
                    crate::placeholder_journal::Kind::Directory
                }
            },
            path: registration.target.path(),
            real: registration.target.removal_path(),
            makes: registration.wrote_record,
        })
        .collect::<Vec<_>>();
    crate::placeholder_journal::register(&planned, &empty_mount_destinations(args));
}

/// The empty files and folders named in bubblewrap's arguments (before the
/// command), with the folders above them resolved. Any path named is
/// included, not only mount targets: one too many only keeps a placeholder
/// a little longer.
fn empty_mount_destinations(args: &[String]) -> Vec<(PathBuf, PathBuf)> {
    let mut found: Vec<(PathBuf, PathBuf)> = Vec::new();
    for arg in args.iter().take_while(|arg| arg.as_str() != "--") {
        let path = Path::new(arg);
        if !path.is_absolute() || found.iter().any(|(seen, _)| seen == path) {
            continue;
        }
        let empty = fs::symlink_metadata(path).is_ok_and(|metadata| {
            (metadata.is_file() && metadata.len() == 0)
                || (metadata.is_dir()
                    && fs::read_dir(path).is_ok_and(|mut entries| entries.next().is_none()))
        });
        if empty {
            let real = crate::placeholder_registry::real_placeholder_path(path)
                .unwrap_or_else(|| path.to_path_buf());
            found.push((path.to_path_buf(), real));
        }
    }
    found
}

fn register_synthetic_mount_targets(
    targets: &[crate::bwrap::SyntheticMountTarget],
) -> Vec<SyntheticMountTargetRegistration> {
    with_synthetic_mount_registry_lock(|| {
        targets
            .iter()
            .map(|target| {
                let marker_dir = synthetic_mount_marker_dir(target.path());
                fs::create_dir_all(&marker_dir).unwrap_or_else(|err| {
                    panic!(
                        "failed to create synthetic bubblewrap mount marker directory {}: {err}",
                        marker_dir.display()
                    )
                });
                // #562: another live run made it, or it has gone since the
                // arguments were built (where this run could not take the
                // registry lock for them, #673, or the person removed it):
                // then the mount target bubblewrap makes is this run's own
                // placeholder, and its marker must say so for later runs.
                let target = if target.preserves_pre_existing_path()
                    && (synthetic_mount_marker_dir_has_active_synthetic_owner(&marker_dir)
                        || !target.pre_existing_path_is_unchanged())
                {
                    match target.kind() {
                        crate::bwrap::SyntheticMountTargetKind::EmptyFile => {
                            crate::bwrap::SyntheticMountTarget::missing(target.path())
                        }
                        crate::bwrap::SyntheticMountTargetKind::EmptyDirectory => {
                            crate::bwrap::SyntheticMountTarget::missing_empty_directory(
                                target.path(),
                            )
                        }
                    }
                } else {
                    target.clone()
                };
                // #607: its location now, before the command runs, where it
                // is removed (without following a link).
                let real_path = crate::placeholder_registry::real_placeholder_path(target.path());
                let target = target.with_real_path(real_path);
                // #607: a placeholder another run made (still there) keeps
                // its record, whose time is when it was made.
                let made_by_another_run = fs::symlink_metadata(target.path()).is_ok()
                    && read_synthetic_mount_target_record(&marker_dir)
                        .is_some_and(|made| made.path() == target.path());
                let wrote_record = !target.preserves_pre_existing_path() && !made_by_another_run;
                if wrote_record {
                    write_synthetic_mount_target_record(&marker_dir, &target);
                }
                let marker_file = marker_dir.join(std::process::id().to_string());
                crate::placeholder_registry::write_marker(
                    &marker_file,
                    synthetic_mount_marker_contents(&target),
                )
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to register synthetic bubblewrap mount target {}: {err}",
                        target.path().display()
                    )
                });
                SyntheticMountTargetRegistration {
                    target,
                    marker_file,
                    marker_dir,
                    wrote_record,
                }
            })
            .collect()
    })
}

fn register_protected_create_targets(
    targets: &[crate::bwrap::ProtectedCreateTarget],
) -> Vec<ProtectedCreateTargetRegistration> {
    with_synthetic_mount_registry_lock(|| {
        targets
            .iter()
            .map(|target| {
                let marker_dir = synthetic_mount_marker_dir(target.path());
                fs::create_dir_all(&marker_dir).unwrap_or_else(|err| {
                    panic!(
                        "failed to create protected create marker directory {}: {err}",
                        marker_dir.display()
                    )
                });
                let marker_file = marker_dir.join(std::process::id().to_string());
                crate::placeholder_registry::write_marker(&marker_file, PROTECTED_CREATE_MARKER)
                    .unwrap_or_else(|err| {
                        panic!(
                            "failed to register protected create target {}: {err}",
                            target.path().display()
                        )
                    });
                ProtectedCreateTargetRegistration {
                    target: target.clone(),
                    marker_file,
                    marker_dir,
                }
            })
            .collect()
    })
}

fn synthetic_mount_marker_contents(target: &crate::bwrap::SyntheticMountTarget) -> &'static [u8] {
    if target.preserves_pre_existing_path() {
        SYNTHETIC_MOUNT_MARKER_EXISTING
    } else {
        SYNTHETIC_MOUNT_MARKER_SYNTHETIC
    }
}

fn synthetic_mount_marker_dir_has_active_synthetic_owner(marker_dir: &Path) -> bool {
    synthetic_mount_marker_dir_has_active_process_matching(marker_dir, |path| {
        match fs::read(path) {
            Ok(contents) => contents == SYNTHETIC_MOUNT_MARKER_SYNTHETIC,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => false,
            Err(err) => panic!(
                "failed to read synthetic bubblewrap mount marker {}: {err}",
                path.display()
            ),
        }
    })
}

fn synthetic_mount_marker_dir_has_active_process(marker_dir: &Path) -> bool {
    synthetic_mount_marker_dir_has_active_process_matching(marker_dir, |_| true)
}

fn synthetic_mount_marker_dir_has_active_process_matching(
    marker_dir: &Path,
    matches_marker: impl Fn(&Path) -> bool,
) -> bool {
    let entries = match fs::read_dir(marker_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return false,
        Err(err) => panic!(
            "failed to read synthetic bubblewrap mount marker directory {}: {err}",
            marker_dir.display()
        ),
    };
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| {
            panic!(
                "failed to read synthetic bubblewrap mount marker in {}: {err}",
                marker_dir.display()
            )
        });
        let path = entry.path();
        let Some(pid) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.parse::<libc::pid_t>().ok())
        else {
            continue;
        };
        if !crate::placeholder_registry::marker_run_is_active(&path, pid) {
            match crate::placeholder_registry::remove_marker(&path) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => panic!(
                    "failed to remove stale synthetic bubblewrap mount marker {}: {err}",
                    path.display()
                ),
            }
            continue;
        }
        let matches_marker = matches_marker(&path);
        if matches_marker {
            return true;
        }
    }
    false
}

fn cleanup_synthetic_mount_targets(targets: &[SyntheticMountTargetRegistration]) {
    if targets.is_empty() {
        return;
    }
    with_synthetic_mount_registry_lock(|| {
        for target in targets.iter().rev() {
            match crate::placeholder_registry::remove_marker(&target.marker_file) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => panic!(
                    "failed to unregister synthetic bubblewrap mount target {}: {err}",
                    target.target.path().display()
                ),
            }
        }

        for target in targets.iter().rev() {
            if synthetic_mount_marker_dir_has_active_process(&target.marker_dir) {
                continue;
            }
            // #348: a live run of another temporary folder mounts over it:
            // left (with its record here, and in this run's record kept in
            // the Corbanu home) for a later sweep.
            if crate::placeholder_journal::used_by_live_run(
                target.target.path(),
                target.target.removal_path(),
            ) {
                continue;
            }
            // #582: a placeholder another run made (and was killed before
            // removing) goes as its record says, also when this run only
            // held it as an existing path.
            // #607: one another run made is removed only as the placeholder
            // made for its record (that run may have been killed long ago);
            // this run's own as before. A record without its location (an
            // older run's) is removed at this run's, resolved when it
            // registered.
            match read_synthetic_mount_target_record(&target.marker_dir) {
                Some(made) if target.target.pre_existing_path_is_unchanged() => {
                    let made = if made.real_path().is_none() && made.path() == target.target.path()
                    {
                        made.with_real_path(target.target.real_path().map(Path::to_path_buf))
                    } else {
                        made
                    };
                    let recorded_at = if target.wrote_record {
                        None
                    } else {
                        synthetic_mount_target_record_time(&target.marker_dir)
                    };
                    remove_synthetic_mount_target(&made, recorded_at);
                }
                // #607: a path this run only held, which is no longer the
                // one it found (the person replaced it while the command
                // ran): theirs, left in place. (Before, it was removed when
                // empty, as if bubblewrap had made it for this run.)
                _ if target.target.preserves_pre_existing_path() => {}
                _ => remove_synthetic_mount_target(&target.target, /*recorded_at*/ None),
            }
            remove_synthetic_mount_target_record(&target.marker_dir);
            match fs::remove_dir(&target.marker_dir) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
                Err(err) => panic!(
                    "failed to remove synthetic bubblewrap mount marker directory {}: {err}",
                    target.marker_dir.display()
                ),
            }
        }
    });
}

fn cleanup_protected_create_targets(targets: &[ProtectedCreateTargetRegistration]) -> bool {
    if targets.is_empty() {
        return false;
    }
    with_synthetic_mount_registry_lock(|| {
        for target in targets.iter().rev() {
            match crate::placeholder_registry::remove_marker(&target.marker_file) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => panic!(
                    "failed to unregister protected create target {}: {err}",
                    target.target.path().display()
                ),
            }
        }

        let mut violation = false;
        for target in targets.iter().rev() {
            violation |= remove_protected_create_target(&target.target);
            match fs::remove_dir(&target.marker_dir) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
                Err(err) => panic!(
                    "failed to remove protected create marker directory {}: {err}",
                    target.marker_dir.display()
                ),
            }
        }
        violation
    })
}

fn remove_protected_create_target(target: &crate::bwrap::ProtectedCreateTarget) -> bool {
    for attempt in 0..100 {
        match try_remove_protected_create_target(target) {
            Ok(removal) => return removal.is_some(),
            Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty && attempt < 99 => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(err) => {
                panic!(
                    "failed to remove protected create target {}: {err}",
                    target.path().display()
                );
            }
        }
    }
    unreachable!("protected create removal retry loop should return or panic")
}

fn remove_protected_create_target_best_effort(
    target: &crate::bwrap::ProtectedCreateTarget,
) -> Option<ProtectedCreateRemoval> {
    for _ in 0..100 {
        match try_remove_protected_create_target(target) {
            Ok(removal) => return removal,
            Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(_) => return Some(ProtectedCreateRemoval::Other),
        }
    }
    Some(ProtectedCreateRemoval::Other)
}

fn try_remove_protected_create_target(
    target: &crate::bwrap::ProtectedCreateTarget,
) -> std::io::Result<Option<ProtectedCreateRemoval>> {
    let path = target.path();
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };

    let removal = if metadata.is_dir() {
        ProtectedCreateRemoval::Directory
    } else {
        ProtectedCreateRemoval::Other
    };
    let result = if removal == ProtectedCreateRemoval::Directory {
        match fs::remove_dir_all(path) {
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                make_directory_tree_writable(path)?;
                fs::remove_dir_all(path)
            }
            result => result,
        }
    } else {
        fs::remove_file(path)
    };
    match result {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    }
    eprintln!(
        "sandbox blocked creation of protected workspace metadata path {}",
        path.display()
    );
    Ok(Some(removal))
}

fn make_directory_tree_writable(path: &Path) -> std::io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    if !metadata.is_dir() {
        return Ok(());
    }

    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(path)?;
    let directory_path = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
    fs::set_permissions(&directory_path, fs::Permissions::from_mode(0o700))?;
    for entry in fs::read_dir(directory_path)? {
        make_directory_tree_writable(&entry?.path())?;
    }
    Ok(())
}

/// Removes `target` where it was registered, without following a link in
/// the folders above it (#607): an empty file, or an empty folder, that it
/// still is. With `recorded_at` (the time of the record of a placeholder
/// another run made), only when it was made for that record: not the
/// person's own empty file made there later.
fn remove_synthetic_mount_target(
    target: &crate::bwrap::SyntheticMountTarget,
    recorded_at: Option<std::time::SystemTime>,
) {
    let path = target.removal_path();
    let found = match crate::placeholder_registry::find_without_links(path) {
        Ok(Some(found)) => found,
        Ok(None) => return,
        Err(err) => panic!(
            "failed to inspect synthetic bubblewrap mount target {}: {err}",
            path.display()
        ),
    };
    if !target.should_remove(found.is_empty_file(), found.is_dir(), found.identity()) {
        return;
    }
    // #607: not when the record kept in the Corbanu home says a placeholder
    // was made there and this is none of those (the person replaced it); one
    // another run made must still be exactly as made.
    if crate::placeholder_journal::recorded_otherwise(target.path(), path, &found.status) {
        return;
    }
    if !target.preserves_pre_existing_path()
        && recorded_at.is_some_and(|recorded_at| {
            !crate::placeholder_registry::made_for_record(found.changed_at(), recorded_at)
        })
    {
        return;
    }
    match found.remove() {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
        Err(err) => panic!(
            "failed to remove synthetic bubblewrap mount target {}: {err}",
            path.display()
        ),
    }
}

/// When the record in `marker_dir` was written, if there is one.
fn synthetic_mount_target_record_time(marker_dir: &Path) -> Option<std::time::SystemTime> {
    fs::symlink_metadata(marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD))
        .and_then(|metadata| metadata.modified())
        .ok()
}

/// #582: records the placeholder this run makes, for
/// [`remove_stale_synthetic_mount_targets`]. Kept until the last run using
/// the placeholder ends.
fn write_synthetic_mount_target_record(
    marker_dir: &Path,
    target: &crate::bwrap::SyntheticMountTarget,
) {
    let kind = match target.kind() {
        crate::bwrap::SyntheticMountTargetKind::EmptyFile => "file",
        crate::bwrap::SyntheticMountTargetKind::EmptyDirectory => "directory",
    };
    let mut record = format!("{kind}\n").into_bytes();
    record.extend_from_slice(target.path().as_os_str().as_bytes());
    // #607: then where it is removed (see `SyntheticMountTarget::real_path`).
    // A path holds no NUL byte; an older run reads such a record as naming
    // another path and leaves it alone.
    if let Some(real_path) = target.real_path() {
        record.push(0);
        record.extend_from_slice(real_path.as_os_str().as_bytes());
    }
    let path = marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD);
    // Only this user may change it (the registry folder may be group
    // writable).
    let written = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
        .and_then(|mut file| {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            std::io::Write::write_all(&mut file, &record)
        });
    written.unwrap_or_else(|err| {
        panic!(
            "failed to record synthetic bubblewrap mount target {}: {err}",
            path.display()
        )
    });
}

fn remove_synthetic_mount_target_record(marker_dir: &Path) {
    let path = marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD);
    match fs::remove_file(&path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => panic!(
            "failed to remove synthetic bubblewrap mount target record {}: {err}",
            path.display()
        ),
    }
}

/// The placeholder recorded in `marker_dir`, if the record is ours and
/// names the path its folder is for.
fn read_synthetic_mount_target_record(
    marker_dir: &Path,
) -> Option<crate::bwrap::SyntheticMountTarget> {
    let path = marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD);
    if !owned_by_this_user(&path, /*file*/ true) {
        return None;
    }
    let record = fs::read(path).ok()?;
    let newline = record.iter().position(|byte| *byte == b'\n')?;
    let rest = &record[newline + 1..];
    // #607: the path, then (after a NUL byte) where it is removed.
    let (path, real_path) = match rest.iter().position(|byte| *byte == 0) {
        Some(nul) => (&rest[..nul], Some(&rest[nul + 1..])),
        None => (rest, None),
    };
    let path = Path::new(std::ffi::OsStr::from_bytes(path));
    if !path.is_absolute() || synthetic_mount_marker_dir(path) != marker_dir {
        return None;
    }
    let real_path = match real_path {
        Some(real_path) => {
            let real_path = Path::new(std::ffi::OsStr::from_bytes(real_path));
            if !real_path.is_absolute() || real_path.file_name() != path.file_name() {
                return None;
            }
            Some(real_path.to_path_buf())
        }
        None => None,
    };
    let target = match &record[..newline] {
        b"file" => crate::bwrap::SyntheticMountTarget::missing(path),
        b"directory" => crate::bwrap::SyntheticMountTarget::missing_empty_directory(path),
        _ => return None,
    };
    Some(target.with_real_path(real_path))
}

/// Whether `path` is a record (or the registry folder) of this user that no
/// other user may write: one who made the shared temporary folder's
/// registry first, or shares the user's group, must not plant records. The
/// folder may be group writable (umask 002 is Ubuntu's default, with
/// per-user groups); a record never is.
fn owned_by_this_user(path: &Path, file: bool) -> bool {
    // Records are made 0600; the registry folder follows the umask.
    let not_writable_by = if file { 0o022 } else { 0o002 };
    // SAFETY: `geteuid` has no preconditions.
    let euid = unsafe { libc::geteuid() };
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        use std::os::unix::fs::MetadataExt;
        let kind = if file {
            metadata.file_type().is_file()
        } else {
            metadata.file_type().is_dir()
        };
        kind && metadata.uid() == euid && metadata.mode() & not_writable_by == 0
    })
}

/// #582: removes the placeholders whose runs all ended without cleaning up
/// (the helper killed with SIGKILL, for example on a timeout), as their own
/// cleanup would have: an empty file, or an empty folder, at the recorded
/// path. Before, such a placeholder stayed: later runs took it for an
/// existing path, and one where a folder belongs (an empty read-only
/// `~/Library` for `~/Library/Keychains`) failed every protected run until
/// #579 held it in place.
///
/// A run that meets a recorded placeholder registers itself for it (under
/// the [`RegistryLock`] it took before finding it, #673), so one with any
/// live marker is left alone, as is a marker folder of another kind (a
/// protected-create target). This never fails a run: without a usable
/// registry it does nothing, and what cannot be removed keeps its record for
/// a later run.
fn remove_stale_synthetic_mount_targets() {
    // A run without placeholders must not fail here.
    let Some(registry_root) = unchecked_synthetic_mount_registry_root() else {
        return;
    };
    if !owned_by_this_user(&registry_root, /*file*/ false) {
        return;
    }
    let Ok(entries) = fs::read_dir(&registry_root) else {
        return;
    };
    // Without the lock first: most runs find nothing recorded.
    let recorded: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|marker_dir| marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD).is_file())
        .collect();
    if recorded.is_empty() {
        return;
    }
    // A registry this run cannot lock (read-only in a nested sandbox) is
    // left as it is. (#673: never while this thread holds the lock, which a
    // second descriptor would wait for forever.)
    if REGISTRY_LOCK_HELD.get() {
        return;
    }
    let Ok(lock_file) = File::open(registry_root.join("lock")) else {
        return;
    };
    // SAFETY: locks a descriptor this function owns.
    if unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX) } < 0 {
        return;
    }
    for marker_dir in recorded {
        let Some(target) = read_synthetic_mount_target_record(&marker_dir) else {
            continue;
        };
        if !only_ended_runs(&marker_dir) {
            continue;
        }
        // #348: nor while a live run of another temporary folder mounts
        // over it.
        if crate::placeholder_journal::used_by_live_run(target.path(), target.removal_path()) {
            continue;
        }
        let recorded_at = synthetic_mount_target_record_time(&marker_dir);
        // A record without its location (an older run's): resolved now, as
        // before.
        let target = if target.real_path().is_none() {
            let real_path = crate::placeholder_registry::real_placeholder_path(target.path());
            target.with_real_path(real_path)
        } else {
            target
        };
        // #607: reached without following a link above it, and only the
        // placeholder made for this record (not the person's own empty file
        // or folder made there later); anything else is no longer one.
        let removed = match crate::placeholder_registry::find_without_links(target.removal_path()) {
            Err(_) => false,
            Ok(None) => true,
            Ok(Some(found))
                if !target.should_remove(
                    found.is_empty_file(),
                    found.is_dir(),
                    found.identity(),
                ) || recorded_at.is_some_and(|recorded_at| {
                    !crate::placeholder_registry::made_for_record(found.changed_at(), recorded_at)
                }) || crate::placeholder_journal::recorded_otherwise(
                    target.path(),
                    target.removal_path(),
                    &found.status,
                ) =>
            {
                // #607: nor one the record kept in the Corbanu home says is
                // not exactly the placeholder made there (the person's own
                // empty file or folder, made within the window above).
                true
            }
            Ok(Some(found)) => match found.remove() {
                Ok(()) => true,
                // A folder that is no longer empty is no longer a placeholder.
                Err(err) => {
                    err.kind() == std::io::ErrorKind::NotFound
                        || err.kind() == std::io::ErrorKind::DirectoryNotEmpty
                }
            },
        };
        if removed {
            remove_ended_markers(&marker_dir);
            let _ = fs::remove_file(marker_dir.join(SYNTHETIC_MOUNT_TARGET_RECORD));
            let _ = fs::remove_dir(&marker_dir);
        }
    }
    // The lock is released when `lock_file` closes.
}

/// Whether every marker in `marker_dir` is a run (that made or held the
/// placeholder) which has ended.
fn only_ended_runs(marker_dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(marker_dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(pid) = marker_pid(&path) else {
            continue;
        };
        if crate::placeholder_registry::marker_run_is_active(&path, pid) {
            return false;
        }
        match fs::read(&path) {
            Ok(contents)
                if contents == SYNTHETIC_MOUNT_MARKER_SYNTHETIC
                    || contents == SYNTHETIC_MOUNT_MARKER_EXISTING => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            _ => return false,
        }
    }
    true
}

fn remove_ended_markers(marker_dir: &Path) {
    let Ok(entries) = fs::read_dir(marker_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if marker_pid(&path).is_some() {
            let _ = crate::placeholder_registry::remove_marker(&path);
        } else if marker_pid(&path.with_extension("")).is_some() {
            // #607: a PID namespace file whose marker is gone.
            let _ = fs::remove_file(path);
        }
    }
}

/// #348: sweeps the record kept in the Corbanu home
/// ([`crate::placeholder_journal`]). Under this temporary folder's registry
/// lock (taken first, as a registering run takes them), so no run of it
/// registers a path between the check that no live run uses it and its
/// removal; a run of another temporary folder records its paths there too.
fn sweep_recorded_placeholders() {
    let registry_lock = RegistryLock::acquire();
    let registry_root = unchecked_synthetic_mount_registry_root()
        .filter(|root| owned_by_this_user(root, /*file*/ false));
    let used_by_registry = |path: &Path, real: &Path| {
        registry_root.as_ref().is_some_and(|root| {
            [path, real].iter().any(|path| {
                marker_dir_has_live_run(&root.join(format!("{:016x}", hash_path(path))))
            })
        })
    };
    crate::placeholder_journal::sweep(&used_by_registry);
    drop(registry_lock);
}

/// Whether a live run has a marker in `marker_dir` (when it cannot be read,
/// taken as yes).
fn marker_dir_has_live_run(marker_dir: &Path) -> bool {
    let entries = match fs::read_dir(marker_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return false,
        Err(_) => return true,
    };
    for entry in entries {
        let Ok(entry) = entry else {
            return true;
        };
        let path = entry.path();
        if let Some(pid) = marker_pid(&path)
            && crate::placeholder_registry::marker_run_is_active(&path, pid)
        {
            return true;
        }
    }
    false
}

fn marker_pid(path: &Path) -> Option<libc::pid_t> {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.parse::<libc::pid_t>().ok())
}

thread_local! {
    /// #673: whether this thread holds a [`RegistryLock`].
    static REGISTRY_LOCK_HELD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// #673: the placeholder registry's lock, held by a run from just before it
/// looks at the paths it mounts until it has registered the placeholders it
/// found or makes ([`register_mount_targets`]). The registry's runs remove a
/// placeholder only under this lock (its makers' and holders' cleanup, the
/// sweep), so none is removed between this run finding it and registering
/// it.
///
/// Before, the arguments were built without it and each check locked on its
/// own: a placeholder found (`~/.aws`, or `~/.claude` above a missing
/// `.credentials.json`) could be removed by its maker's cleanup before the
/// check that it was a live placeholder. The run then took it for a path of
/// the person's own and mounted it unregistered, and bubblewrap failed the
/// command (`Can't create file at ~/.aws: Read-only file system`, `Can't find
/// source path ~/.claude`).
#[derive(Debug)]
pub(crate) struct RegistryLock {
    file: File,
    /// Whether it holds the Corbanu home record's lock too (#348).
    journal: bool,
    /// Released on the thread that took it (`REGISTRY_LOCK_HELD`).
    _not_send: std::marker::PhantomData<*const ()>,
}

impl RegistryLock {
    /// `None` where the registry cannot be made or locked (read-only, as in
    /// a nested sandbox, or another user's), or when this thread holds it
    /// already; each step then locks on its own, as before.
    pub(crate) fn acquire() -> Option<Self> {
        if REGISTRY_LOCK_HELD.get() {
            return None;
        }
        // Without the panicking checks of `synthetic_mount_registry_root`
        // (a run with nothing to register must not fail here), then the
        // same folder the markers are in.
        if unchecked_synthetic_mount_registry_root()?.is_symlink() {
            return None;
        }
        let registry_root = synthetic_mount_registry_root();
        let file = open_synthetic_mount_registry_lock(&registry_root).ok()?;
        if !owned_by_this_user(&registry_root, /*file*/ false) {
            return None;
        }
        // SAFETY: locks a descriptor this function owns.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } < 0 {
            return None;
        }
        REGISTRY_LOCK_HELD.set(true);
        // #348: and the Corbanu home record's, taken second, as every run
        // takes them: a sweep of a run in another temporary folder cannot
        // remove a placeholder this run found until it has recorded it.
        let journal = crate::placeholder_journal::hold();
        Some(Self {
            file,
            journal,
            _not_send: std::marker::PhantomData,
        })
    }
}

impl Drop for RegistryLock {
    fn drop(&mut self) {
        if self.journal {
            crate::placeholder_journal::release();
        }
        REGISTRY_LOCK_HELD.set(false);
        // Closing the descriptor releases it too.
        // SAFETY: unlocks a descriptor this lock owns.
        unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

/// The registry folder, without the panicking checks of
/// [`synthetic_mount_registry_root`].
fn unchecked_synthetic_mount_registry_root() -> Option<PathBuf> {
    let temp_dir = std::env::temp_dir().canonicalize().ok()?;
    // SAFETY: `geteuid` has no preconditions.
    let euid = unsafe { libc::geteuid() };
    Some(temp_dir.join(format!("codex-bwrap-synthetic-mount-targets-{euid}")))
}

/// The registry's lock file, opened (and the registry made) for locking.
fn open_synthetic_mount_registry_lock(registry_root: &Path) -> std::io::Result<File> {
    fs::create_dir_all(registry_root)?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(registry_root.join("lock"))
}

fn with_synthetic_mount_registry_lock<T>(f: impl FnOnce() -> T) -> T {
    // #673: part of this run's lock for its arguments.
    if REGISTRY_LOCK_HELD.get() {
        return f();
    }
    let registry_root = synthetic_mount_registry_root();
    let lock_path = registry_root.join("lock");
    let lock_file = open_synthetic_mount_registry_lock(&registry_root).unwrap_or_else(|err| {
        panic!(
            "failed to open synthetic bubblewrap mount registry lock {}: {err}",
            lock_path.display()
        )
    });
    if unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX) } < 0 {
        let err = std::io::Error::last_os_error();
        panic!(
            "failed to lock synthetic bubblewrap mount registry {}: {err}",
            lock_path.display()
        );
    }
    let result = f();
    if unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_UN) } < 0 {
        let err = std::io::Error::last_os_error();
        panic!(
            "failed to unlock synthetic bubblewrap mount registry {}: {err}",
            lock_path.display()
        );
    }
    result
}

/// #562: whether a live run registered a placeholder it made at `path`
/// (an empty file standing in for a missing protected path). A run that
/// finds one must register it too, so the owner's cleanup leaves it in
/// place until the last run using it ends; until then (#673) the run holds
/// the [`RegistryLock`] it took before finding it.
pub(crate) fn is_live_placeholder(path: &Path) -> bool {
    let marker_dir = synthetic_mount_marker_dir(path);
    // A run registers its marker before bubblewrap makes the placeholder, so
    // without a marker folder there is no placeholder to share (and no need
    // to take the lock or create the registry).
    if !marker_dir.is_dir() {
        return false;
    }
    with_synthetic_mount_registry_lock(|| {
        synthetic_mount_marker_dir_has_active_synthetic_owner(&marker_dir)
            // #582: one whose maker was killed before removing it: held as
            // a placeholder too, so a sweep leaves it while this run uses it,
            // and the last run out removes it.
            || read_synthetic_mount_target_record(&marker_dir)
                .is_some_and(|made| made.path() == path)
    })
}

fn synthetic_mount_marker_dir(path: &Path) -> PathBuf {
    synthetic_mount_registry_root().join(format!("{:016x}", hash_path(path)))
}

pub(crate) fn synthetic_mount_registry_root() -> PathBuf {
    static REGISTRY_ROOT: OnceLock<PathBuf> = OnceLock::new();

    REGISTRY_ROOT
        .get_or_init(|| {
            let effective_uid = unsafe { libc::geteuid() };
            let temp_dir = std::env::temp_dir();
            let temp_dir = temp_dir.canonicalize().unwrap_or_else(|err| {
                panic!(
                    "failed to resolve synthetic mount registry temp directory {}: {err}",
                    temp_dir.display()
                )
            });
            let registry_root = temp_dir.join(format!(
                "codex-bwrap-synthetic-mount-targets-{effective_uid}"
            ));
            // A registry symlink can redirect bookkeeping into a writable root
            // that does not overlap TMPDIR, bypassing its read-only mount.
            assert!(
                !registry_root.is_symlink(),
                "synthetic mount registry must not be a symlink: {}",
                registry_root.display()
            );
            registry_root
        })
        .clone()
}

fn hash_path(path: &Path) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in path.as_os_str().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn exit_with_wait_status(status: libc::c_int) -> ! {
    if libc::WIFEXITED(status) {
        std::process::exit(libc::WEXITSTATUS(status));
    }

    if libc::WIFSIGNALED(status) {
        let signal = libc::WTERMSIG(status);
        unsafe {
            libc::signal(signal, libc::SIG_DFL);
            libc::kill(libc::getpid(), signal);
        }
        std::process::exit(128 + signal);
    }

    std::process::exit(1);
}

fn exit_with_wait_status_or_policy_violation(
    status: libc::c_int,
    protected_create_violation: bool,
) -> ! {
    if protected_create_violation && libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0 {
        std::process::exit(1);
    }

    exit_with_wait_status(status);
}

/// Run a short-lived bubblewrap preflight in a child process and capture stderr.
///
/// Strategy:
/// - This is used only by `preflight_proc_mount_support`, which runs `/bin/true`
///   under bubblewrap with `--proc /proc`.
/// - The goal is to detect environments where mounting `/proc` fails (for
///   example, restricted containers), so we can retry the real run with
///   `--no-proc`.
/// - We capture stderr from that preflight to match known mount-failure text.
///   We do not stream it because this is a one-shot probe with a trivial
///   command, and reads are bounded to a fixed max size.
fn run_bwrap_in_child_capture_stderr(bwrap_args: crate::bwrap::BwrapArgs) -> String {
    const MAX_PREFLIGHT_STDERR_BYTES: u64 = 64 * 1024;
    let crate::bwrap::BwrapArgs {
        args,
        preserved_files,
        synthetic_mount_targets,
        protected_create_targets,
        registry_lock,
    } = bwrap_args;
    let setup_signal_mask = ForwardedSignalMask::block();
    let (synthetic_mount_registrations, protected_create_registrations) = register_mount_targets(
        &synthetic_mount_targets,
        &protected_create_targets,
        registry_lock,
        &args,
    );

    let mut pipe_fds = [0; 2];
    let pipe_res = unsafe { libc::pipe2(pipe_fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if pipe_res < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to create stderr pipe for bubblewrap: {err}");
    }
    let read_fd = pipe_fds[0];
    let write_fd = pipe_fds[1];

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        panic!("failed to fork for bubblewrap: {err}");
    }

    if pid == 0 {
        reset_forwarded_signal_handlers_to_default();
        setup_signal_mask.restore();
        // Child: redirect stderr to the pipe, then run bubblewrap.
        unsafe {
            close_fd_or_panic(read_fd, "close read end in bubblewrap child");
            if libc::dup2(write_fd, libc::STDERR_FILENO) < 0 {
                let err = std::io::Error::last_os_error();
                panic!("failed to redirect stderr for bubblewrap: {err}");
            }
            close_fd_or_panic(write_fd, "close write end in bubblewrap child");
        }

        exec_bwrap(args, preserved_files);
    }

    let signal_forwarders = install_bwrap_signal_forwarders(pid);
    setup_signal_mask.restore();
    // Parent: close the write end and read stderr while the child runs.
    close_fd_or_panic(write_fd, "close write end in bubblewrap parent");

    // SAFETY: `read_fd` is a valid owned fd in the parent.
    let mut read_file = unsafe { File::from_raw_fd(read_fd) };
    let mut stderr_bytes = Vec::new();
    let mut limited_reader = (&mut read_file).take(MAX_PREFLIGHT_STDERR_BYTES);
    if let Err(err) = limited_reader.read_to_end(&mut stderr_bytes) {
        panic!("failed to read bubblewrap stderr: {err}");
    }

    let status = wait_for_bwrap_child(pid);
    let cleanup_signal_mask = ForwardedSignalMask::block();
    BWRAP_CHILD_PID.store(0, Ordering::SeqCst);
    crate::placeholder_journal::leave();
    cleanup_synthetic_mount_targets(&synthetic_mount_registrations);
    cleanup_protected_create_targets(&protected_create_registrations);
    crate::placeholder_journal::finish();
    signal_forwarders.restore();
    cleanup_signal_mask.restore();
    if libc::WIFSIGNALED(status) {
        exit_with_wait_status(status);
    }

    String::from_utf8_lossy(&stderr_bytes).into_owned()
}

/// Close an owned file descriptor and panic with context on failure.
///
/// We use explicit close() checks here (instead of ignoring return codes)
/// because this code runs in low-level sandbox setup paths where fd leaks or
/// close errors can mask the root cause of later failures.
fn close_fd_or_panic(fd: libc::c_int, context: &str) {
    let close_res = unsafe { libc::close(fd) };
    if close_res < 0 {
        let err = std::io::Error::last_os_error();
        panic!("{context}: {err}");
    }
}

fn is_proc_mount_failure(stderr: &str) -> bool {
    stderr.contains("Can't mount proc")
        && stderr.contains("/newroot/proc")
        && (stderr.contains("Invalid argument")
            || stderr.contains("Operation not permitted")
            || stderr.contains("Permission denied"))
}

struct InnerSeccompCommandArgs<'a> {
    sandbox_policy_cwd: &'a Path,
    command_cwd: Option<&'a Path>,
    permission_profile: &'a PermissionProfile,
    allow_network_for_proxy: bool,
    proxy_route_spec: Option<String>,
    close_inherited_fd: Option<i32>,
    command: Vec<String>,
}

/// Build the inner command that applies seccomp after bubblewrap.
fn build_inner_seccomp_command(args: InnerSeccompCommandArgs<'_>) -> Vec<String> {
    let InnerSeccompCommandArgs {
        sandbox_policy_cwd,
        command_cwd,
        permission_profile,
        allow_network_for_proxy,
        proxy_route_spec,
        close_inherited_fd,
        command,
    } = args;
    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => panic!("failed to resolve current executable path: {err}"),
    };
    let permission_profile_json = match serde_json::to_string(permission_profile) {
        Ok(json) => json,
        Err(err) => panic!("failed to serialize permission profile: {err}"),
    };

    let mut inner = vec![
        current_exe.to_string_lossy().to_string(),
        "--sandbox-policy-cwd".to_string(),
        sandbox_policy_cwd.to_string_lossy().to_string(),
    ];
    if let Some(command_cwd) = command_cwd {
        inner.push("--command-cwd".to_string());
        inner.push(command_cwd.to_string_lossy().to_string());
    }
    inner.extend([
        "--permission-profile".to_string(),
        permission_profile_json,
        "--apply-seccomp-then-exec".to_string(),
    ]);
    if allow_network_for_proxy {
        inner.push("--allow-network-for-proxy".to_string());
        let proxy_route_spec = proxy_route_spec
            .unwrap_or_else(|| panic!("managed proxy mode requires a proxy route spec"));
        inner.push("--proxy-route-spec".to_string());
        inner.push(proxy_route_spec);
    }
    if let Some(fd) = close_inherited_fd {
        inner.push("--close-inherited-fd".to_string());
        inner.push(fd.to_string());
    }
    inner.push("--".to_string());
    inner.extend(command);
    inner
}

/// Exec the provided argv, panicking with context if it fails.
fn exec_or_panic(command: Vec<String>) -> ! {
    #[expect(clippy::expect_used)]
    let c_command =
        CString::new(command[0].as_str()).expect("Failed to convert command to CString");
    #[expect(clippy::expect_used)]
    let c_args: Vec<CString> = command
        .iter()
        .map(|arg| CString::new(arg.as_str()).expect("Failed to convert arg to CString"))
        .collect();

    let mut c_args_ptrs: Vec<*const libc::c_char> = c_args.iter().map(|arg| arg.as_ptr()).collect();
    c_args_ptrs.push(std::ptr::null());

    unsafe {
        libc::execvp(c_command.as_ptr(), c_args_ptrs.as_ptr());
    }

    // If execvp returns, there was an error.
    let err = std::io::Error::last_os_error();
    panic!("Failed to execvp {}: {err}", command[0].as_str());
}

#[cfg(test)]
#[path = "linux_run_main_tests.rs"]
mod tests;
