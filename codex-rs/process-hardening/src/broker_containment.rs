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
    contain_credential_broker_with_files(writable_dir, &[])
}

/// Like [`contain_credential_broker`], also leaving each existing file in
/// `writable_files` writable (PF-27-S05: the vault's lock file).
pub fn contain_credential_broker_with_files(
    writable_dir: &Path,
    writable_files: &[std::path::PathBuf],
) -> BrokerContainment {
    #[cfg(target_os = "macos")]
    {
        macos::contain(writable_dir, writable_files)
    }
    #[cfg(target_os = "linux")]
    {
        linux::contain(writable_dir, writable_files)
    }
    #[cfg(windows)]
    {
        let _ = (writable_dir, writable_files);
        windows::contain()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
    {
        let _ = (writable_dir, writable_files);
        BrokerContainment::none()
    }
}

/// The Seatbelt profile used on macOS, exposed for tests.
pub fn broker_seatbelt_profile(writable_dir: &Path) -> Option<String> {
    broker_seatbelt_profile_with_files(writable_dir, &[])
}

/// The Seatbelt profile with extra writable files.
pub fn broker_seatbelt_profile_with_files(
    writable_dir: &Path,
    writable_files: &[std::path::PathBuf],
) -> Option<String> {
    let (dir, canonical) = quotable(writable_dir)?;
    let mut extra = String::new();
    for file in writable_files {
        let (file, canonical) = quotable(file)?;
        extra.push_str(&format!(
            "\n    (require-not (literal \"{file}\"))\n    (require-not (literal \"{canonical}\"))"
        ));
    }
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
    (require-not (literal "/private/var/run/syslog")){extra}))
"#
    ))
}

/// `path` and its canonical form, when both can be quoted in a profile.
fn quotable(path: &Path) -> Option<(String, String)> {
    let raw = path.to_str()?;
    if raw.contains(['"', '\\']) || !path.is_absolute() {
        return None;
    }
    // Seatbelt matches canonical paths (`/tmp` is `/private/tmp`). A file that
    // does not exist yet is matched through its canonical parent.
    let canonical = std::fs::canonicalize(path)
        .ok()
        .or_else(|| {
            let parent = std::fs::canonicalize(path.parent()?).ok()?;
            Some(parent.join(path.file_name()?))
        })
        .and_then(|path| path.to_str().map(str::to_string))
        .filter(|path| !path.contains(['"', '\\']))
        .unwrap_or_else(|| raw.to_string());
    Some((raw.to_string(), canonical))
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

    pub(super) fn contain(
        writable_dir: &Path,
        writable_files: &[std::path::PathBuf],
    ) -> BrokerContainment {
        let Some(profile) = super::broker_seatbelt_profile_with_files(writable_dir, writable_files)
        else {
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

    pub(super) fn contain(
        writable_dir: &Path,
        writable_files: &[std::path::PathBuf],
    ) -> BrokerContainment {
        // SAFETY: prctl only changes flags of this process.
        if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
            return BrokerContainment::none();
        }
        let mut mechanisms = Vec::new();
        if landlock_writes(writable_dir, writable_files) {
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

    fn landlock_writes(writable_dir: &Path, writable_files: &[std::path::PathBuf]) -> bool {
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
            })
            .and_then(|ruleset| {
                // File rules keep only the file-compatible rights; a rule
                // needs the file to exist.
                let existing = writable_files.iter().filter(|file| file.is_file());
                ruleset.add_rules(landlock::path_beneath_rules(existing, access_rw))
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

/// PF-27-S06: Windows has no per-process file sandbox the broker can apply
/// to itself, so it applies what it can: a process and thread DACL that keeps
/// other processes out of its memory (`dacl`), and a job (`job`) that allows
/// no child processes and denies the desktop, clipboard, global atoms, other
/// processes' USER handles and system settings. Not confined on Windows: file
/// writes, opening other processes of the user, and asking another process
/// to run something (WMI, Task Scheduler, out-of-process COM). A restricted
/// or AppContainer token for the broker is the follow-up.
#[cfg(windows)]
mod windows {
    use super::BrokerContainment;
    use std::io;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
    use windows_sys::Win32::System::JobObjects::CreateJobObjectW;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_DESKTOP;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_DISPLAYSETTINGS;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_EXITWINDOWS;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_GLOBALATOMS;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_HANDLES;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_READCLIPBOARD;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS;
    use windows_sys::Win32::System::JobObjects::JOB_OBJECT_UILIMIT_WRITECLIPBOARD;
    use windows_sys::Win32::System::JobObjects::JOBOBJECT_BASIC_UI_RESTRICTIONS;
    use windows_sys::Win32::System::JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
    use windows_sys::Win32::System::JobObjects::JobObjectBasicUIRestrictions;
    use windows_sys::Win32::System::JobObjects::JobObjectExtendedLimitInformation;
    use windows_sys::Win32::System::JobObjects::SetInformationJobObject;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    pub(super) fn contain() -> BrokerContainment {
        let mut mechanisms = Vec::new();
        if crate::restrict_current_process_access().is_ok() {
            mechanisms.push("dacl");
        }
        if forbid_child_processes().is_ok() {
            mechanisms.push("job");
        }
        if mechanisms.is_empty() {
            return BrokerContainment::none();
        }
        BrokerContainment {
            mechanism: mechanisms.join("+"),
        }
    }

    /// Puts this process in a job that allows one active process: itself.
    /// The job handle stays open for the life of the process.
    fn forbid_child_processes() -> io::Result<()> {
        // SAFETY: an anonymous job with default security.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: zeroed POD; the fields set below are the only limits.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
        limits.BasicLimitInformation.ActiveProcessLimit = 1;
        let ui = JOBOBJECT_BASIC_UI_RESTRICTIONS {
            UIRestrictionsClass: JOB_OBJECT_UILIMIT_DESKTOP
                | JOB_OBJECT_UILIMIT_HANDLES
                | JOB_OBJECT_UILIMIT_READCLIPBOARD
                | JOB_OBJECT_UILIMIT_WRITECLIPBOARD
                | JOB_OBJECT_UILIMIT_GLOBALATOMS
                | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS
                | JOB_OBJECT_UILIMIT_DISPLAYSETTINGS
                | JOB_OBJECT_UILIMIT_EXITWINDOWS,
        };
        // SAFETY: `limits` and `ui` are valid for the sizes passed; `job` is open.
        let applied = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) != 0
                && SetInformationJobObject(
                    job,
                    JobObjectBasicUIRestrictions,
                    (&ui as *const JOBOBJECT_BASIC_UI_RESTRICTIONS).cast(),
                    std::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
                ) != 0
                && AssignProcessToJobObject(job, GetCurrentProcess()) != 0
        };
        if applied {
            Ok(())
        } else {
            let error = io::Error::last_os_error();
            // SAFETY: opened above; not assigned, so closing it is safe.
            unsafe { CloseHandle(job) };
            Err(error)
        }
    }
}

#[cfg(test)]
#[path = "broker_containment_tests.rs"]
mod tests;
