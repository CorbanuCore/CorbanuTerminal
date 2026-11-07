//! In-process Linux sandbox primitives: `no_new_privs` and seccomp.
//!
//! Filesystem restrictions are enforced by bubblewrap in `linux_run_main`.
//! The legacy Landlock filesystem implementation is retained here, but the
//! helper rejects legacy mode for filesystem-restricted policies.
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use codex_network_proxy::ManagedNetworkSandboxContext;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;
use codex_protocol::error::SandboxErr;
use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::NetworkSandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;

use landlock::ABI;
#[allow(unused_imports)]
use landlock::Access;
use landlock::AccessFs;
use landlock::CompatLevel;
use landlock::Compatible;
use landlock::Ruleset;
use landlock::RulesetAttr;
use landlock::RulesetCreatedAttr;
use seccompiler::BpfProgram;
use seccompiler::SeccompAction;
use seccompiler::SeccompCmpArgLen;
use seccompiler::SeccompCmpOp;
use seccompiler::SeccompCondition;
use seccompiler::SeccompFilter;
use seccompiler::SeccompRule;
use seccompiler::TargetArch;
use seccompiler::apply_filter;

/// Apply sandbox policies inside this thread so only the child inherits
/// them, not the entire CLI process.
///
/// This function is responsible for:
<<<<<<< HEAD:codex-rs/linux-sandbox/src/landlock.rs
/// - enabling `PR_SET_NO_NEW_PRIVS` when restrictions apply,
/// - installing the network seccomp filter when network access is disabled,
///   and
/// - installing the user-namespace seccomp filter when the policy denies
///   user namespaces (#826).
||||||| 413492cd6c:codex-rs/linux-sandbox/src/landlock.rs
/// - enabling `PR_SET_NO_NEW_PRIVS` when restrictions apply, and
/// - installing the network seccomp filter when network access is disabled.
=======
/// - enabling `PR_SET_NO_NEW_PRIVS` when restrictions apply, and
/// - installing seccomp restrictions for network isolation and VM sockets.
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a:codex-rs/linux-sandbox/src/seccomp.rs
///
/// Filesystem restrictions are normally handled by bubblewrap. The legacy
/// Landlock implementation is only selected by `apply_landlock_fs`.
pub(crate) fn apply_permission_profile_to_current_thread(
    permission_profile: &PermissionProfile,
    cwd: &Path,
    apply_landlock_fs: bool,
    managed_network: Option<&ManagedNetworkSandboxContext>,
    proxy_routing_active: bool,
) -> Result<()> {
    let (file_system_sandbox_policy, network_sandbox_policy) =
        permission_profile.to_runtime_permissions();
    let network_seccomp_mode = network_seccomp_mode(
        network_sandbox_policy,
        managed_network.is_some(),
        proxy_routing_active,
    )
    .or_else(|| {
        // VM sockets can reach host services outside the filesystem sandbox.
        // In WSL2 they also allow Windows process launch through an alias of
        // the interop socket, even when /run/WSL is masked. Keep ordinary
        // network access while denying that host bridge.
        (!file_system_sandbox_policy.has_full_disk_write_access())
            .then_some(NetworkSeccompMode::VmSocketRestricted)
    });

    let deny_user_namespaces = file_system_sandbox_policy.denies_user_namespaces();

    // `PR_SET_NO_NEW_PRIVS` is required for seccomp, but it also prevents
    // setuid privilege elevation. Many `bwrap` deployments rely on setuid, so
    // we avoid this unless we need seccomp or we are explicitly using the
    // legacy Landlock filesystem pipeline.
    if network_seccomp_mode.is_some()
        || deny_user_namespaces
        || (apply_landlock_fs && !file_system_sandbox_policy.has_full_disk_write_access())
    {
        set_no_new_privs()?;
    }

    if let Some(mode) = network_seccomp_mode {
        install_network_seccomp_filter_on_current_thread(mode, managed_network)?;
    }

    if deny_user_namespaces {
        install_user_namespace_seccomp_filters_on_current_thread()?;
    }

    if apply_landlock_fs && !file_system_sandbox_policy.has_full_disk_write_access() {
        if !file_system_sandbox_policy.has_full_disk_read_access() {
            return Err(CodexErr::UnsupportedOperation(
                "Restricted read-only access is not supported by the legacy Linux Landlock filesystem backend."
                    .to_string(),
            ));
        }

        let writable_roots: Vec<AbsolutePathBuf> = file_system_sandbox_policy
            .get_writable_roots_with_cwd(cwd)
            .into_iter()
            .map(|writable_root| writable_root.root)
            .collect();
        match crate::pinned::get() {
            // #536: rules on the folders opened and checked at their pinned
            // locations (Landlock rules hold the folder, not its path).
            Some(pinned) => {
                let checked = crate::pinned::check_held_links(&file_system_sandbox_policy, cwd)?;
                let folders = writable_roots
                    .iter()
                    .filter(|root| root.as_path().exists())
                    .map(|root| {
                        let real = crate::pinned::expected_location(pinned, &checked, root);
                        crate::pinned::open_at(&[root.as_path()], &real)
                    })
                    .collect::<Result<Vec<File>>>()?;
                install_filesystem_landlock_rules_on_current_thread(Vec::new(), folders)?;
            }
            None => {
                install_filesystem_landlock_rules_on_current_thread(writable_roots, Vec::new())?
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NetworkSeccompMode {
    Restricted,
    ProxyRouted,
    VmSocketRestricted,
}

fn should_install_network_seccomp(
    network_sandbox_policy: NetworkSandboxPolicy,
    allow_network_for_proxy: bool,
) -> bool {
    // Managed-network sessions should remain fail-closed even for policies that
    // would normally grant full network access (for example, DangerFullAccess).
    !network_sandbox_policy.is_enabled() || allow_network_for_proxy
}

fn network_seccomp_mode(
    network_sandbox_policy: NetworkSandboxPolicy,
    allow_network_for_proxy: bool,
    proxy_routed_network: bool,
) -> Option<NetworkSeccompMode> {
    if !should_install_network_seccomp(network_sandbox_policy, allow_network_for_proxy) {
        None
    } else if proxy_routed_network {
        Some(NetworkSeccompMode::ProxyRouted)
    } else {
        Some(NetworkSeccompMode::Restricted)
    }
}

/// Enable `PR_SET_NO_NEW_PRIVS` so seccomp can be applied safely.
fn set_no_new_privs() -> Result<()> {
    let result = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

/// Installs Landlock file-system rules on the current thread allowing read
/// access to the entire file-system while restricting write access to
/// `/dev/null` and the provided list of `writable_roots`.
///
/// # Errors
/// Returns [`CodexErr::Sandbox`] variants when the ruleset fails to apply.
///
/// Retained for the legacy pipeline. The helper currently rejects legacy mode
/// for filesystem-restricted policies, so filesystem sandboxing uses bubblewrap.
fn install_filesystem_landlock_rules_on_current_thread(
    writable_roots: Vec<AbsolutePathBuf>,
    writable_folders: Vec<File>,
) -> Result<()> {
    let abi = ABI::V5;
    let access_rw = AccessFs::from_all(abi);
    let access_ro = AccessFs::from_read(abi);

    let mut ruleset = Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(access_rw)?
        .create()?
        .add_rules(landlock::path_beneath_rules(&["/"], access_ro))?
        .add_rules(landlock::path_beneath_rules(&["/dev/null"], access_rw))?
        .set_no_new_privs(true);

    if !writable_roots.is_empty() {
        ruleset = ruleset.add_rules(landlock::path_beneath_rules(&writable_roots, access_rw))?;
    }
    for folder in writable_folders {
        ruleset = ruleset.add_rule(landlock::PathBeneath::new(folder, access_rw))?;
    }

    let status = ruleset.restrict_self()?;

    if status.ruleset == landlock::RulesetStatus::NotEnforced {
        return Err(CodexErr::Sandbox(SandboxErr::LandlockRestrict));
    }

    Ok(())
}

/// Installs a seccomp filter for Linux network sandboxing.
///
/// The filter is applied to the current thread so only the sandboxed child
/// inherits it.
fn install_network_seccomp_filter_on_current_thread(
    mode: NetworkSeccompMode,
    managed_network: Option<&ManagedNetworkSandboxContext>,
) -> std::result::Result<(), SandboxErr> {
    fn deny_syscall(rules: &mut BTreeMap<i64, Vec<SeccompRule>>, nr: i64) {
        rules.insert(nr, vec![]); // empty rule vec = unconditional match
    }

    // Build rule map.
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    if mode != NetworkSeccompMode::VmSocketRestricted {
        deny_syscall(&mut rules, libc::SYS_ptrace);
        deny_syscall(&mut rules, libc::SYS_process_vm_readv);
        deny_syscall(&mut rules, libc::SYS_process_vm_writev);
    }
    // io_uring can create AF_VSOCK sockets without a socket() syscall, so
    // keep it unavailable in every mode with socket-family restrictions.
    deny_syscall(&mut rules, libc::SYS_io_uring_setup);
    deny_syscall(&mut rules, libc::SYS_io_uring_enter);
    deny_syscall(&mut rules, libc::SYS_io_uring_register);

    match mode {
        NetworkSeccompMode::Restricted => {
            deny_syscall(&mut rules, libc::SYS_connect);
            deny_syscall(&mut rules, libc::SYS_accept);
            deny_syscall(&mut rules, libc::SYS_accept4);
            deny_syscall(&mut rules, libc::SYS_bind);
            deny_syscall(&mut rules, libc::SYS_listen);
            deny_syscall(&mut rules, libc::SYS_getpeername);
            deny_syscall(&mut rules, libc::SYS_getsockname);
            deny_syscall(&mut rules, libc::SYS_shutdown);
            deny_syscall(&mut rules, libc::SYS_sendto);
            deny_syscall(&mut rules, libc::SYS_sendmmsg);
            // NOTE: allowing recvfrom allows some tools like: `cargo clippy`
            // to run with their socketpair + child processes for sub-proc
            // management.
            // deny_syscall(&mut rules, libc::SYS_recvfrom);
            deny_syscall(&mut rules, libc::SYS_recvmmsg);
            deny_syscall(&mut rules, libc::SYS_getsockopt);
            deny_syscall(&mut rules, libc::SYS_setsockopt);

            // For `socket` we allow AF_UNIX (arg0 == AF_UNIX) and deny
            // everything else.
            let unix_only_rule = SeccompRule::new(vec![SeccompCondition::new(
                0, // first argument (domain)
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::Ne,
                libc::AF_UNIX as u64,
            )?])?;

            rules.insert(libc::SYS_socket, vec![unix_only_rule.clone()]);
            rules.insert(libc::SYS_socketpair, vec![unix_only_rule]);
        }
        NetworkSeccompMode::ProxyRouted => {
            // In proxy-routed mode we allow IP sockets in the isolated
            // namespace (used to reach the local TCP bridge). Standalone Unix
            // sockets require an explicit managed-policy grant; all other
            // socket families remain denied.
            let mut denied_socket_conditions = vec![
                SeccompCondition::new(
                    0,
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::Ne,
                    libc::AF_INET as u64,
                )?,
                SeccompCondition::new(
                    0,
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::Ne,
                    libc::AF_INET6 as u64,
                )?,
            ];
            if managed_network.is_some_and(|context| context.dangerously_allow_all_unix_sockets) {
                denied_socket_conditions.push(SeccompCondition::new(
                    0,
                    SeccompCmpArgLen::Dword,
                    SeccompCmpOp::Ne,
                    libc::AF_UNIX as u64,
                )?);
            }
            let deny_non_ip_socket = SeccompRule::new(denied_socket_conditions)?;
            let deny_non_unix_socketpair = SeccompRule::new(vec![SeccompCondition::new(
                0,
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::Ne,
                libc::AF_UNIX as u64,
            )?])?;
            rules.insert(libc::SYS_socket, vec![deny_non_ip_socket]);
            rules.insert(libc::SYS_socketpair, vec![deny_non_unix_socketpair]);
        }
        NetworkSeccompMode::VmSocketRestricted => {
            let deny_vsock = SeccompRule::new(vec![SeccompCondition::new(
                0,
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::Eq,
                libc::AF_VSOCK as u64,
            )?])?;
            rules.insert(libc::SYS_socket, vec![deny_vsock.clone()]);
            rules.insert(libc::SYS_socketpair, vec![deny_vsock]);
        }
    }

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,                     // default – allow
        SeccompAction::Errno(libc::EPERM as u32), // when rule matches – return EPERM
        seccomp_target_arch(),
    )?;

    let prog: BpfProgram = filter.try_into()?;

    apply_filter(&prog)?;

    Ok(())
}

fn seccomp_target_arch() -> TargetArch {
    if cfg!(target_arch = "x86_64") {
        TargetArch::x86_64
    } else if cfg!(target_arch = "aarch64") {
        TargetArch::aarch64
    } else {
        unimplemented!("unsupported architecture for seccomp filter");
    }
}

/// The x32 ABI's syscall numbers are the x86_64 ones with this bit set, under
/// the same audit architecture, so a filter on the x86_64 numbers alone
/// misses them where the kernel has x32 enabled.
#[cfg(target_arch = "x86_64")]
const X32_SYSCALL_BIT: i64 = 0x4000_0000;

/// `nr` and, on x86_64, its x32 alias.
fn with_x32_alias(nr: i64) -> Vec<i64> {
    #[cfg(target_arch = "x86_64")]
    {
        vec![nr, nr | X32_SYSCALL_BIT]
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        vec![nr]
    }
}

/// #826: the seccomp programs that stop a process creating user namespaces,
/// so a command cannot become root over the files it owns (the placeholders
/// bubblewrap hides denied files with) or mount over paths in a namespace of
/// its own:
///
/// - `unshare` and `clone` with `CLONE_NEWUSER` fail with `EPERM`;
/// - `clone3`, whose flags are in memory a filter cannot read, fails with
///   `ENOSYS`, so the C libraries and language runtimes fall back to `clone`
///   (as under Docker's and Flatpak's filters).
///
/// Bubblewrap applies them to every process in the sandbox, its own PID 1
/// included (`--add-seccomp-fd`, see `linux_run_main`), and the inner stage
/// installs them again on the command. They are inherited by every child and
/// cannot be removed. Joining an existing user namespace (`setns`) needs
/// `CAP_SYS_ADMIN` in it: inside bubblewrap's namespace a process has that
/// only in namespaces created below it, which these programs refuse; on the
/// paths without bubblewrap (legacy Landlock, full write access) the command
/// could join one its user made elsewhere, where Landlock and seccomp still
/// apply to it.
pub(crate) fn user_namespace_seccomp_programs() -> std::result::Result<Vec<BpfProgram>, SandboxErr>
{
    let new_user = libc::CLONE_NEWUSER as u64;
    let with_new_user = SeccompRule::new(vec![SeccompCondition::new(
        0, // flags, the first argument of both calls on x86_64 and aarch64
        SeccompCmpArgLen::Dword,
        SeccompCmpOp::MaskedEq(new_user),
        new_user,
    )?])?;
    let mut denied: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
    for nr in [libc::SYS_unshare, libc::SYS_clone]
        .into_iter()
        .flat_map(with_x32_alias)
    {
        denied.insert(nr, vec![with_new_user.clone()]);
    }
    let mut unavailable: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
    for nr in with_x32_alias(libc::SYS_clone3) {
        unavailable.insert(nr, vec![]); // empty rule vec = unconditional match
    }
    [(denied, libc::EPERM), (unavailable, libc::ENOSYS)]
        .into_iter()
        .map(|(rules, errno)| {
            let filter = SeccompFilter::new(
                rules,
                SeccompAction::Allow,
                SeccompAction::Errno(errno as u32),
                seccomp_target_arch(),
            )?;
            Ok(filter.try_into()?)
        })
        .collect()
}

/// Installs [`user_namespace_seccomp_programs`] on this thread.
fn install_user_namespace_seccomp_filters_on_current_thread() -> std::result::Result<(), SandboxErr>
{
    for prog in user_namespace_seccomp_programs()? {
        apply_filter(&prog)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "seccomp_tests.rs"]
mod tests;
