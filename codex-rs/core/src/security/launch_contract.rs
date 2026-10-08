//! PF-27-S02 secretless agent launch contract (feature `secretless_agent_launch`).
//!
//! Once armed, every agent command launch must:
//! - run under an OS sandbox on a platform whose containment is verified here
//!   (macOS Seatbelt, Linux bubblewrap/seccomp, and on Windows the elevated
//!   sandbox, which runs commands as a separate user; PF-27-S06);
//! - deny reads of the vault store, sign-in files, wallet and broker runtime
//!   directory, and deny writes to `CODEX_HOME` (the policy store);
//! - carry no raw managed secret in its environment, argv or stdin, and not
//!   start a login shell that would re-load profile secrets.
//!
//! The environment allowlist itself lives in
//! [`codex_protocol::secretless_launch`] so every launcher that builds an
//! environment from the process applies it. Brokered credentials are added
//! back afterwards as dummies by the network proxy (it sources the raw values
//! from Core, not from the stripped child environment).

use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxKind;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_protocol::permissions::NetworkSandboxPolicy;
use codex_protocol::secretless_launch;
use codex_sandboxing::SandboxCommand;
use codex_sandboxing::SandboxType;
use codex_sandboxing::policy_transforms::effective_permission_profile;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;
use zeroize::Zeroizing;

/// The local group the elevated Windows sandbox's users belong to.
#[cfg(windows)]
const WINDOWS_SANDBOX_USERS_GROUP: &str = "CodexSandboxUsers";

/// Shorter values are too likely to collide with ordinary text.
const MIN_MANAGED_VALUE_BYTES: usize = 8;

/// Files and directories under `CODEX_HOME` that agent commands may not read:
/// credential stores, the broker runtime directory, and records that can hold
/// secrets from earlier sessions (shell snapshots, logs, transcripts, history).
const PROTECTED_CODEX_HOME_ENTRIES: &[&str] = &[
    "secrets",
    "auth.json",
    ".credentials.json",
    ".env",
    "provider_auth.json",
    "config.toml",
    "managed_config.toml",
    "wallet",
    "run",
    "shell_snapshots",
    "log",
    "sessions",
    "archived_sessions",
    "history.jsonl",
];

/// Credential files under `$HOME` that tools read for the user: CLI tokens
/// for GitHub, AWS, Docker, npm and git, `.netrc`, and Claude's sign-in.
const PROTECTED_HOME_ENTRIES: &[&str] = &[
    ".netrc",
    ".git-credentials",
    ".config/gh/hosts.yml",
    ".aws/credentials",
    ".docker/config.json",
    ".npmrc",
    ".claude/.credentials.json",
];

/// Glob (relative to `CODEX_HOME`) for Core's state and log databases.
const PROTECTED_CODEX_HOME_GLOB: &str = "*.sqlite*";

/// Set in protected launches so zsh does not source `~/.zshenv`, which can
/// export credentials; `/var/empty` holds no start-up files.
pub(crate) const PROTECTED_ZDOTDIR: &str = "/var/empty";

static ACTIVE: OnceLock<LaunchContract> = OnceLock::new();

/// Why a protected launch was refused. The text is shown to the user and model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LaunchDenied {
    UnsupportedPlatform,
    /// PF-27-S06: the unelevated Windows sandbox restricts writes only.
    WindowsUnelevatedSandbox,
    /// PF-27-S06: new files in `CODEX_HOME` could not be denied to the
    /// elevated sandbox's users (it is not set up yet, or the ACL failed).
    #[cfg_attr(not(windows), allow(dead_code))]
    WindowsSandboxNotSetUp,
    ProcessHardening,
    Unsandboxed,
    RemoteEnvironment,
    UnprotectableFileSystem,
    ProtectedPathReadable(String),
    PolicyStoreWritable,
    LoginShell,
    RawSecretInArgv,
    RawSecretInStdin,
}

impl std::fmt::Display for LaunchDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Self::UnsupportedPlatform => {
                "secretless agent launch is not available on this platform yet (PF-27-S06)".to_string()
            }
            Self::WindowsUnelevatedSandbox => {
                "the unelevated Windows sandbox cannot deny reads of the vault and sign-in files; set `sandbox = \"elevated\"` under `[windows]`"
                    .to_string()
            }
            Self::WindowsSandboxNotSetUp => {
                "the elevated Windows sandbox is not set up yet, so new files in Corbanu's configuration directory cannot be denied to it; set it up first"
                    .to_string()
            }
            Self::ProcessHardening => {
                "Corbanu could not protect its own process memory from agent commands".to_string()
            }
            Self::Unsandboxed => {
                "this command would run outside the OS sandbox, which protected launch does not allow"
                    .to_string()
            }
            Self::RemoteEnvironment => {
                "commands in a remote environment build their environment on the remote host, which this contract cannot check yet"
                    .to_string()
            }
            Self::UnprotectableFileSystem => {
                "the current permissions give commands unrestricted file access, so the vault and sign-in files cannot be protected"
                    .to_string()
            }
            Self::ProtectedPathReadable(path) => {
                format!("the sandbox would let this command read {path}")
            }
            Self::PolicyStoreWritable => {
                "the sandbox would let this command write Corbanu's configuration directory".to_string()
            }
            Self::LoginShell => {
                "login shells load profile files that can export secrets".to_string()
            }
            Self::RawSecretInArgv => "the command line contains a managed secret".to_string(),
            Self::RawSecretInStdin => "the input contains a managed secret".to_string(),
        };
        write!(f, "{LAUNCH_DENIED_PREFIX}{reason}.")
    }
}

const LAUNCH_DENIED_PREFIX: &str = "Protected launch refused: ";

/// True when an error message is a [`LaunchDenied`] refusal.
pub(crate) fn is_launch_denial(message: &str) -> bool {
    message.starts_with(LAUNCH_DENIED_PREFIX)
}

impl std::error::Error for LaunchDenied {}

/// Process-wide contract state captured when the feature is first armed.
pub(crate) struct LaunchContract {
    codex_home: AbsolutePathBuf,
    protected_read_paths: Vec<AbsolutePathBuf>,
    managed_values: Vec<Zeroizing<String>>,
    hardened: bool,
}

impl std::fmt::Debug for LaunchContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaunchContract")
            .field("codex_home", &self.codex_home)
            .field("protected_read_paths", &self.protected_read_paths)
            .field("managed_values", &self.managed_values.len())
            .field("hardened", &self.hardened)
            .finish()
    }
}

/// Arms the contract for this process: the environment allowlist, Core's own
/// process hardening, and launch checks. Idempotent and one-way.
pub(crate) fn arm(codex_home: &AbsolutePathBuf) {
    secretless_launch::arm();
    ACTIVE.get_or_init(|| {
        let hardened = harden_current_process();
        LaunchContract::capture(codex_home, std::env::vars_os(), hardened)
    });
}

/// Brokered variables (for example `GITHUB_TOKEN`) that the user's shell
/// environment policy would pass to agent commands, before the launch
/// allowlist runs and without the model-provider keys Core strips for the
/// shell tool (built-in ones and `provider_env_keys`). Only these may be
/// sourced from Core's environment as dummies.
pub(crate) fn policy_permitted_brokered_env_keys(
    policy: &codex_protocol::config_types::ShellEnvironmentPolicy,
    provider_env_keys: &[String],
) -> Vec<String> {
    permitted_brokered_env_keys(
        std::env::vars_os().filter_map(|(name, value)| {
            Some((name.into_string().ok()?, value.into_string().ok()?))
        }),
        policy,
        provider_env_keys,
    )
}

pub(crate) fn permitted_brokered_env_keys<I>(
    vars: I,
    policy: &codex_protocol::config_types::ShellEnvironmentPolicy,
    provider_env_keys: &[String],
) -> Vec<String>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut env = codex_protocol::shell_environment::create_env_from_vars(
        vars, policy, /*thread_id*/ None,
    );
    crate::exec_env::remove_provider_auth_env_vars(
        &mut env,
        provider_env_keys.iter().map(String::as_str),
    );
    codex_network_proxy::credential_broker_env_var_names()
        .into_iter()
        .filter(|name| env.contains_key(*name))
        .map(str::to_string)
        .collect()
}

/// The armed contract, if any.
pub(crate) fn active() -> Option<&'static LaunchContract> {
    ACTIVE.get()
}

/// Issue #218: applies the armed contract to an external agent CLI that
/// Corbanu launches itself (contained Claude panes): the platform, process
/// hardening and sandbox checks, the argv and environment checks, and
/// `profile` with the protected reads denied and `CODEX_HOME` read-only.
/// Fails when the contract is not armed (`secretless_agent_launch` off).
pub fn protect_external_agent_launch(
    sandbox: SandboxType,
    argv: &[String],
    env: &mut HashMap<String, String>,
    profile: &PermissionProfile,
    cwd: &Path,
) -> Result<PermissionProfile, String> {
    let contract = active().ok_or_else(|| {
        "contained external agents need the secretless agent launch contract (feature `secretless_agent_launch`)"
            .to_string()
    })?;
    contract
        .protect_external_launch(sandbox, argv, env, profile, cwd)
        .map_err(|err| err.to_string())
}

/// True once the secretless launch contract is armed in this process.
pub fn external_agent_contract_armed() -> bool {
    active().is_some()
}

impl LaunchContract {
    pub(crate) fn capture<I>(codex_home: &AbsolutePathBuf, vars: I, hardened: bool) -> Self
    where
        I: IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
    {
        let mut managed_values: Vec<Zeroizing<String>> = Vec::new();
        for (name, value) in vars {
            let (Some(name), Some(value)) = (name.to_str(), value.to_str()) else {
                continue;
            };
            let value = value.trim();
            if secretless_launch::is_secret_looking_name(name)
                && value.len() >= MIN_MANAGED_VALUE_BYTES
                && !managed_values.iter().any(|known| known.as_str() == value)
            {
                managed_values.push(Zeroizing::new(value.to_string()));
            }
        }
        let mut protected_read_paths = PROTECTED_CODEX_HOME_ENTRIES
            .iter()
            .map(|entry| codex_home.join(entry))
            .collect::<Vec<_>>();
        // Windows has no HOME by default; its profile directory is USERPROFILE.
        let home = std::env::var_os("HOME").or_else(|| {
            cfg!(windows)
                .then(|| std::env::var_os("USERPROFILE"))
                .flatten()
        });
        if let Some(home) = home
            && let Ok(home) = AbsolutePathBuf::from_absolute_path(home)
        {
            protected_read_paths
                .extend(PROTECTED_HOME_ENTRIES.iter().map(|entry| home.join(entry)));
            if cfg!(target_os = "macos") {
                protected_read_paths.push(home.join("Library/Keychains"));
            }
            if cfg!(windows) {
                protected_read_paths.push(home.join("_netrc"));
            }
        }
        if cfg!(windows)
            && let Some(app_data) = std::env::var_os("APPDATA")
            && let Ok(app_data) = AbsolutePathBuf::from_absolute_path(app_data)
        {
            protected_read_paths.push(app_data.join("GitHub CLI").join("hosts.yml"));
            protected_read_paths.push(app_data.join("gcloud"));
        }
        if cfg!(windows)
            && let Some(cargo_home) = std::env::var_os("CARGO_HOME").or_else(|| {
                std::env::var_os("USERPROFILE").map(|home| {
                    std::path::PathBuf::from(home)
                        .join(".cargo")
                        .into_os_string()
                })
            })
            && let Ok(cargo_home) = AbsolutePathBuf::from_absolute_path(cargo_home)
        {
            protected_read_paths.push(cargo_home.join("credentials.toml"));
            protected_read_paths.push(cargo_home.join("credentials"));
        }
        if let Some(dir) = std::env::var_os("CLAUDE_CONFIG_DIR")
            && let Ok(dir) = AbsolutePathBuf::from_absolute_path(dir)
        {
            protected_read_paths.push(dir.join(".credentials.json"));
        }
        if let Some(dir) = codex_network_proxy::credential_broker_user_runtime_dir()
            && let Ok(dir) = AbsolutePathBuf::from_absolute_path(dir)
        {
            protected_read_paths.push(dir);
        }
        Self {
            codex_home: codex_home.clone(),
            protected_read_paths,
            managed_values,
            hardened,
        }
    }

    /// Applies the whole contract to one launch before its sandbox transform:
    /// platform and sandbox checks, argv and environment checks, and the
    /// returned profile with protected paths denied (verified again after the
    /// command's own additional permissions are merged).
    /// `windows_elevated`: the command will run under the elevated Windows
    /// sandbox backend (ignored elsewhere).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn protect_launch(
        &self,
        sandbox: SandboxType,
        sandbox_requested: bool,
        exec_server: bool,
        windows_elevated: bool,
        command: &mut SandboxCommand,
        permissions: &PermissionProfile,
        cwd: &Path,
    ) -> Result<PermissionProfile, LaunchDenied> {
        let mut protect = || -> Result<PermissionProfile, LaunchDenied> {
            self.check_sandbox(sandbox, sandbox_requested, exec_server, windows_elevated)?;
            let mut argv = vec![command.program.to_string_lossy().into_owned()];
            argv.extend(command.args.iter().cloned());
            self.check_command(&argv, &mut command.env)?;
            let protected = self.protect_permissions(permissions, cwd)?;
            let merged =
                effective_permission_profile(&protected, command.additional_permissions.as_ref());
            self.verify_permissions(&merged, cwd)?;
            Ok(protected)
        };
        protect().inspect_err(super::inspection::record_launch_denial)
    }

    /// [`protect_external_agent_launch`] with this contract.
    pub(crate) fn protect_external_launch(
        &self,
        sandbox: SandboxType,
        argv: &[String],
        env: &mut HashMap<String, String>,
        profile: &PermissionProfile,
        cwd: &Path,
    ) -> Result<PermissionProfile, LaunchDenied> {
        let mut protect = || -> Result<PermissionProfile, LaunchDenied> {
            // External agents are not launched through the Windows sandbox.
            self.check_sandbox(
                sandbox, /*sandbox_requested*/ true, /*exec_server*/ false,
                /*windows_elevated*/ false,
            )?;
            self.check_command(argv, env)?;
            self.protect_permissions(profile, cwd)
        };
        protect().inspect_err(super::inspection::record_launch_denial)
    }

    /// PF-27-S06: a protected file replaced by rename (a token refresh) is a
    /// new file and would not carry its per-file deny, so every file created
    /// directly in `CODEX_HOME` inherits a read deny for the elevated
    /// sandbox's users (subdirectories such as `skills` are unaffected). The
    /// entry stays on the directory. Needs the sandbox's users group, which
    /// its setup creates; until then protected launches are refused.
    #[cfg(windows)]
    pub(crate) fn protect_new_codex_home_files(&self) -> Result<(), LaunchDenied> {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;
        static APPLIED: AtomicBool = AtomicBool::new(false);
        if APPLIED.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut group = codex_windows_sandbox::resolve_sid(WINDOWS_SANDBOX_USERS_GROUP)
            .map_err(|_| LaunchDenied::WindowsSandboxNotSetUp)?;
        // SAFETY: `group` holds a valid SID for the duration of the call.
        let present = unsafe {
            codex_windows_sandbox::add_deny_read_ace_for_new_files(
                self.codex_home.as_path(),
                group.as_mut_ptr().cast(),
            )
        };
        if !matches!(present, Ok(true)) {
            return Err(LaunchDenied::WindowsSandboxNotSetUp);
        }
        APPLIED.store(true, Ordering::Release);
        Ok(())
    }

    /// Whether Core's own process hardening succeeded when it was armed.
    pub(crate) fn hardened(&self) -> bool {
        self.hardened
    }

    /// Platform, process-hardening and sandbox checks for one launch attempt.
    /// `sandbox_requested` is the policy decision; `sandbox` is the concrete
    /// wrapper (`None` for exec-server launches, which apply it remotely).
    /// On Windows only the elevated backend (`windows_elevated`) can deny
    /// reads, so the unelevated one is refused (PF-27-S06). Exec-server
    /// launches are refused on every platform before that.
    pub(crate) fn check_sandbox(
        &self,
        sandbox: SandboxType,
        sandbox_requested: bool,
        exec_server: bool,
        windows_elevated: bool,
    ) -> Result<(), LaunchDenied> {
        if !cfg!(any(target_os = "macos", target_os = "linux", windows)) {
            return Err(LaunchDenied::UnsupportedPlatform);
        }
        if !self.hardened {
            return Err(LaunchDenied::ProcessHardening);
        }
        if exec_server {
            return Err(LaunchDenied::RemoteEnvironment);
        }
        if !sandbox_requested || sandbox == SandboxType::None {
            return Err(LaunchDenied::Unsandboxed);
        }
        if cfg!(windows) && !windows_elevated {
            return Err(LaunchDenied::WindowsUnelevatedSandbox);
        }
        Ok(())
    }

    /// Returns `profile` with read denials for the protected paths added, then
    /// verifies the result denies those reads and every write to `CODEX_HOME`.
    pub(crate) fn protect_permissions(
        &self,
        profile: &PermissionProfile,
        cwd: &Path,
    ) -> Result<PermissionProfile, LaunchDenied> {
        let PermissionProfile::Managed { .. } = profile else {
            return Err(LaunchDenied::Unsandboxed);
        };
        let (mut file_system, network) = profile.to_runtime_permissions();
        if file_system.kind != FileSystemSandboxKind::Restricted {
            return Err(LaunchDenied::UnprotectableFileSystem);
        }
        // A writable root above CODEX_HOME (for example `$TMPDIR`) becomes
        // read-only there; this only narrows access. A workspace inside
        // CODEX_HOME stays writable and is refused below.
        if file_system.can_write_path_with_cwd(self.codex_home.as_path(), cwd) {
            file_system.entries.push(FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: self.codex_home.clone(),
                },
                FileSystemAccessMode::Read,
            ));
        }
        let glob = FileSystemSandboxEntry::new(
            FileSystemPath::GlobPattern {
                pattern: self
                    .codex_home
                    .join(PROTECTED_CODEX_HOME_GLOB)
                    .to_string_lossy()
                    .into_owned(),
            },
            FileSystemAccessMode::Deny,
        );
        for entry in self
            .protected_read_paths
            .iter()
            .map(|path| {
                let path_entry = FileSystemPath::Path { path: path.clone() };
                // PF-27-S06: the Windows sandbox drops every skip-if-missing
                // entry, so a path that exists is denied outright there.
                if cfg!(windows) && path.as_path().exists() {
                    FileSystemSandboxEntry::new(path_entry, FileSystemAccessMode::Deny)
                } else {
                    FileSystemSandboxEntry::skip_missing_path(
                        path_entry,
                        FileSystemAccessMode::Deny,
                    )
                }
            })
            .chain(std::iter::once(glob))
        {
            if !file_system.entries.contains(&entry) {
                file_system.entries.push(entry);
            }
        }
        let protected = PermissionProfile::from_runtime_permissions(&file_system, network);
        self.verify_permissions(&protected, cwd)?;
        Ok(protected)
    }

    /// Verifies that `profile` denies reading every protected path and writing
    /// `CODEX_HOME`. Run again after per-command permissions are merged.
    pub(crate) fn verify_permissions(
        &self,
        profile: &PermissionProfile,
        cwd: &Path,
    ) -> Result<(), LaunchDenied> {
        let PermissionProfile::Managed { .. } = profile else {
            return Err(LaunchDenied::Unsandboxed);
        };
        let file_system = profile.file_system_sandbox_policy();
        if file_system.kind != FileSystemSandboxKind::Restricted {
            return Err(LaunchDenied::UnprotectableFileSystem);
        }
        for path in &self.protected_read_paths {
            if file_system.can_read_path_with_cwd(path.as_path(), cwd) {
                return Err(LaunchDenied::ProtectedPathReadable(
                    path.display().to_string(),
                ));
            }
        }
        let canonical_home = std::fs::canonicalize(self.codex_home.as_path()).ok();
        let writable_inside_home =
            file_system
                .get_writable_roots_with_cwd(cwd)
                .iter()
                .any(|root| {
                    let root = root.root.as_path();
                    let canonical_root = std::fs::canonicalize(root).ok();
                    [Some(root), canonical_root.as_deref()]
                        .into_iter()
                        .flatten()
                        .any(|root| {
                            root.starts_with(self.codex_home.as_path())
                                || canonical_home
                                    .as_deref()
                                    .is_some_and(|home| root.starts_with(home))
                        })
                });
        if writable_inside_home
            || file_system.can_write_path_with_cwd(self.codex_home.as_path(), cwd)
            || file_system.can_write_path_with_cwd(&self.codex_home.join("config.toml"), cwd)
        {
            return Err(LaunchDenied::PolicyStoreWritable);
        }
        Ok(())
    }

    /// Permissions for in-process file tools (apply_patch, structured edits,
    /// image viewing): the protected profile, or a deny-everything profile
    /// when the launch could not be protected, so these tools fail closed.
    pub(crate) fn file_tool_permissions(
        &self,
        profile: &PermissionProfile,
        sandboxed: bool,
        cwd: &Path,
    ) -> PermissionProfile {
        let protected = sandboxed
            .then(|| self.protect_permissions(profile, cwd).ok())
            .flatten();
        protected.unwrap_or_else(|| {
            PermissionProfile::from_runtime_permissions(
                &FileSystemSandboxPolicy::restricted(Vec::new()),
                NetworkSandboxPolicy::Restricted,
            )
        })
    }

    /// Refuses login shells and managed secrets in argv, then removes any
    /// remaining environment value that embeds a managed secret or a URL
    /// password. Broker dummies differ from the raw values and are kept.
    pub(crate) fn check_command(
        &self,
        argv: &[String],
        env: &mut HashMap<String, String>,
    ) -> Result<(), LaunchDenied> {
        if is_login_shell(argv) {
            return Err(LaunchDenied::LoginShell);
        }
        if argv.iter().any(|arg| self.contains_managed_value(arg)) {
            return Err(LaunchDenied::RawSecretInArgv);
        }
        env.retain(|_, value| {
            !self.contains_managed_value(value) && !secretless_launch::value_has_url_password(value)
        });
        if cfg!(unix) {
            env.insert("ZDOTDIR".to_string(), PROTECTED_ZDOTDIR.to_string());
        }
        Ok(())
    }

    /// Refuses input that carries a managed secret to a running process.
    pub(crate) fn check_stdin(&self, input: &[u8]) -> Result<(), LaunchDenied> {
        let input = String::from_utf8_lossy(input);
        if self.contains_managed_value(&input) {
            super::inspection::record_launch_denial(&LaunchDenied::RawSecretInStdin);
            return Err(LaunchDenied::RawSecretInStdin);
        }
        Ok(())
    }

    fn contains_managed_value(&self, text: &str) -> bool {
        self.managed_values
            .iter()
            .any(|value| text.contains(value.as_str()))
    }
}

/// `sh -l`, `bash -lc`, `zsh --login`, and similar.
fn is_login_shell(argv: &[String]) -> bool {
    let Some(program) = argv.first() else {
        return false;
    };
    let name = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.starts_with('-') {
        return true;
    }
    if !matches!(name, "sh" | "bash" | "zsh" | "dash" | "ksh" | "fish") {
        return false;
    }
    argv.iter()
        .skip(1)
        .take_while(|arg| arg.starts_with('-'))
        .any(|arg| {
            arg == "--login"
                || (!arg.starts_with("--") && arg.trim_start_matches('-').contains('l'))
        })
}

/// Keeps same-user processes outside the sandbox from reading Core's memory:
/// non-dumpable on Linux (also hides `/proc/<pid>/environ` and `mem`), no
/// debugger attach on macOS, a process DACL that grants no memory or handle
/// access on Windows (PF-27-S06). Returns false when the platform call failed.
fn harden_current_process() -> bool {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: prctl with these arguments only changes this process's flags.
        let set = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) } == 0;
        // SAFETY: reading the flag has no side effects.
        set && unsafe { libc::prctl(libc::PR_GET_DUMPABLE, 0, 0, 0, 0) } == 0
    }
    #[cfg(target_os = "macos")]
    {
        // SAFETY: PT_DENY_ATTACH takes no address or data.
        unsafe { libc::ptrace(libc::PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0) == 0 }
    }
    #[cfg(windows)]
    {
        codex_process_hardening::restrict_current_process_access().is_ok()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        false
    }
}

#[cfg(test)]
#[path = "launch_contract_tests.rs"]
mod tests;

#[cfg(test)]
#[cfg(windows)]
#[path = "launch_contract_windows_tests.rs"]
mod windows_tests;
