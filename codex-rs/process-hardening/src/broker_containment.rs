//! PF-27-S02 OS containment for the isolated credential broker.
//!
//! The broker holds raw provider credentials and talks to the network, so it
//! is confined before it reads anything secret: it may not start programs,
//! debug or read other processes, or write files outside its runtime
//! directory. Call this while the process is still single-threaded.

use std::path::Path;

/// What containment was applied; reported to the controller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerContainment {
    /// Short machine-readable description, for example `seatbelt` or
    /// `landlock+seccomp`. `none` means nothing could be applied.
    pub mechanism: String,
}

impl BrokerContainment {
    fn none() -> Self {
        Self {
            mechanism: "none".to_string(),
        }
    }
}

/// Confines the current process. Writes stay possible only beneath
/// `writable_dir` (and `/dev/null`); process creation and cross-process
/// inspection are denied.
pub fn contain_credential_broker(writable_dir: &Path) -> BrokerContainment {
    #[cfg(target_os = "macos")]
    {
        macos::contain(writable_dir)
    }
    #[cfg(target_os = "linux")]
    {
        linux::contain(writable_dir)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = writable_dir;
        BrokerContainment::none()
    }
}

/// The Seatbelt profile used on macOS, exposed for tests.
pub fn broker_seatbelt_profile(writable_dir: &Path) -> Option<String> {
    let dir = writable_dir.to_str()?;
    if dir.contains(['"', '\\']) || !writable_dir.is_absolute() {
        return None;
    }
    // Seatbelt matches canonical paths (`/tmp` is `/private/tmp`).
    let canonical = std::fs::canonicalize(writable_dir)
        .ok()
        .and_then(|path| path.to_str().map(str::to_string))
        .filter(|path| !path.contains(['"', '\\']))
        .unwrap_or_else(|| dir.to_string());
    Some(format!(
        r#"(version 1)
(allow default)
(deny process-exec*)
(deny process-fork)
(deny signal (target others))
(deny process-info* (target others))
(deny file-write*
  (require-all
    (require-not (subpath "{dir}"))
    (require-not (subpath "{canonical}"))
    (require-not (literal "/dev/null"))
    (require-not (literal "/private/var/run/syslog"))))
"#
    ))
}

#[cfg(target_os = "macos")]
mod macos {
    use super::BrokerContainment;
    use std::ffi::CString;
    use std::os::raw::c_char;
    use std::os::raw::c_int;
    use std::path::Path;

    unsafe extern "C" {
        fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> c_int;
        fn sandbox_free_error(errorbuf: *mut c_char);
    }

    pub(super) fn contain(writable_dir: &Path) -> BrokerContainment {
        let Some(profile) = super::broker_seatbelt_profile(writable_dir) else {
            return BrokerContainment::none();
        };
        let Ok(profile) = CString::new(profile) else {
            return BrokerContainment::none();
        };
        let mut error: *mut c_char = std::ptr::null_mut();
        // SAFETY: `profile` is a valid NUL-terminated string; `error` receives
        // an allocation we free below.
        let result = unsafe { sandbox_init(profile.as_ptr(), 0, &mut error) };
        if !error.is_null() {
            // SAFETY: `error` was allocated by sandbox_init.
            unsafe { sandbox_free_error(error) };
        }
        if result == 0 {
            BrokerContainment {
                mechanism: "seatbelt".to_string(),
            }
        } else {
            BrokerContainment::none()
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::BrokerContainment;
    use landlock::ABI;
    use landlock::Access;
    use landlock::AccessFs;
    use landlock::CompatLevel;
    use landlock::Compatible;
    use landlock::Ruleset;
    use landlock::RulesetAttr;
    use landlock::RulesetCreatedAttr;
    use landlock::RulesetStatus;
    use seccompiler::BpfProgram;
    use seccompiler::SeccompAction;
    use seccompiler::SeccompCmpArgLen;
    use seccompiler::SeccompCmpOp;
    use seccompiler::SeccompCondition;
    use seccompiler::SeccompFilter;
    use seccompiler::SeccompRule;
    use seccompiler::TargetArch;
    use std::collections::BTreeMap;
    use std::path::Path;

    pub(super) fn contain(writable_dir: &Path) -> BrokerContainment {
        // SAFETY: prctl only changes flags of this process.
        if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
            return BrokerContainment::none();
        }
        let mut mechanisms = Vec::new();
        if landlock_writes(writable_dir) {
            mechanisms.push("landlock");
        }
        if seccomp_no_exec().is_ok() {
            mechanisms.push("seccomp");
        }
        if mechanisms.is_empty() {
            return BrokerContainment::none();
        }
        BrokerContainment {
            mechanism: mechanisms.join("+"),
        }
    }

    fn landlock_writes(writable_dir: &Path) -> bool {
        let abi = ABI::V5;
        let access_rw = AccessFs::from_all(abi);
        let access_ro = AccessFs::from_read(abi);
        let ruleset = Ruleset::default()
            .set_compatibility(CompatLevel::BestEffort)
            .handle_access(access_rw)
            .and_then(Ruleset::create)
            .and_then(|ruleset| ruleset.add_rules(landlock::path_beneath_rules(["/"], access_ro)))
            .and_then(|ruleset| {
                ruleset.add_rules(landlock::path_beneath_rules(["/dev/null"], access_rw))
            })
            .and_then(|ruleset| {
                ruleset.add_rules(landlock::path_beneath_rules([writable_dir], access_rw))
            });
        match ruleset.and_then(landlock::RulesetCreated::restrict_self) {
            Ok(status) => status.ruleset != RulesetStatus::NotEnforced,
            Err(_) => false,
        }
    }

    /// Denies program execution, new processes, and cross-process memory
    /// access. Threads (`clone` with `CLONE_THREAD`) stay allowed; `clone3`
    /// reports ENOSYS so the C library falls back to `clone`.
    fn seccomp_no_exec() -> Result<(), Box<dyn std::error::Error>> {
        let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
        for syscall in [
            libc::SYS_execve,
            libc::SYS_execveat,
            libc::SYS_ptrace,
            libc::SYS_process_vm_readv,
            libc::SYS_process_vm_writev,
            libc::SYS_pidfd_getfd,
        ] {
            rules.insert(syscall, Vec::new());
        }
        #[cfg(target_arch = "x86_64")]
        for syscall in [libc::SYS_fork, libc::SYS_vfork] {
            rules.insert(syscall, Vec::new());
        }
        rules.insert(
            libc::SYS_clone,
            vec![SeccompRule::new(vec![SeccompCondition::new(
                0,
                SeccompCmpArgLen::Qword,
                SeccompCmpOp::MaskedEq(libc::CLONE_THREAD as u64),
                0,
            )?])?],
        );
        let arch = if cfg!(target_arch = "x86_64") {
            TargetArch::x86_64
        } else if cfg!(target_arch = "aarch64") {
            TargetArch::aarch64
        } else {
            return Err("unsupported architecture".into());
        };
        let filter = SeccompFilter::new(
            rules,
            SeccompAction::Allow,
            SeccompAction::Errno(libc::EPERM as u32),
            arch,
        )?;
        let program: BpfProgram = filter.try_into()?;
        let clone3 = SeccompFilter::new(
            BTreeMap::from([(libc::SYS_clone3, Vec::new())]),
            SeccompAction::Allow,
            SeccompAction::Errno(libc::ENOSYS as u32),
            arch,
        )?;
        let clone3: BpfProgram = clone3.try_into()?;
        seccompiler::apply_filter_all_threads(&clone3)?;
        seccompiler::apply_filter_all_threads(&program)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "broker_containment_tests.rs"]
mod tests;
