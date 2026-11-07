//! Bubblewrap-based filesystem sandboxing for Linux.
//!
//! This module mirrors the semantics used by the macOS Seatbelt sandbox:
//! - the filesystem is read-only by default,
//! - explicit writable roots are layered on top, and
//! - sensitive subpaths such as `.git`, `.agents`, and `.codex` remain
//!   read-only even when their parent root is writable.
//!
//! Restricted execution also hides WSLg's duplicate distro root so it cannot
//! expose filesystem contents outside the policy's path masks. Explicit grants and
//! command paths using that alias are rejected before constructing mounts.
//!
//! The overall Linux sandbox is composed of:
//! - seccomp + `PR_SET_NO_NEW_PRIVS` applied in-process, and
//! - bubblewrap used to construct the filesystem view before exec.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::fs::Metadata;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::linux_run_main::RegistryLock;
use crate::linux_run_main::synthetic_mount_registry_root;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;
use codex_protocol::permissions::FileSystemDenyGlob;
use codex_protocol::permissions::PinnedWritableRoot;
use codex_protocol::permissions::credential_glob_exempts_templates;
use codex_protocol::permissions::credential_rule_roots;
use codex_protocol::permissions::is_credential_template_name;
use codex_protocol::permissions::is_protected_metadata_name;
use codex_protocol::protocol::FileSystemAccessMode;
use codex_protocol::protocol::FileSystemPath;
use codex_protocol::protocol::FileSystemSandboxPolicy;
use codex_protocol::protocol::FileSystemSpecialPath;
use codex_protocol::protocol::WritableRoot;
use codex_sandboxing::find_executable_in_search_paths;
use codex_sandboxing::find_pre_sandbox_executable_in_path;
use codex_utils_absolute_path::AbsolutePathBuf;
use globset::GlobBuilder;
use globset::GlobSet;
use globset::GlobSetBuilder;

/// Linux "platform defaults" that keep common system binaries and dynamic
/// libraries readable when a split filesystem policy requests `:minimal`.
///
/// These are intentionally system-level paths only (plus Nix store roots) so
/// `include_platform_defaults` does not silently widen access to user data.
const LINUX_PLATFORM_DEFAULT_READ_ROOTS: &[&str] = &[
    "/bin",
    "/sbin",
    "/usr",
    "/etc",
    "/lib",
    "/lib64",
    "/nix/store",
    "/run/current-system/sw",
];

/// External scanner used by deny-glob expansion and its dependency inventory.
pub const GLOB_SCAN_PROGRAM: &str = "rg";

const MAX_UNREADABLE_GLOB_MATCHES: usize = 8192;
pub(crate) const WSL_INTEROP_DIR: &str = "/run/WSL";
pub(crate) const WSLG_DISTRO_ROOT: &str = "/mnt/wslg/distro";

/// #536: the writable roots the caller pinned (`--pinned-writable-roots`).
/// Every writable root is then bound from the folder this helper opened and
/// checked at its pinned location (`--bind-fd`), never by path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WritableRootPins {
    pub(crate) roots: &'static [PinnedWritableRoot],
    /// Whether the bubblewrap that runs supports `--bind-fd` (0.8+).
    pub(crate) bind_fd: bool,
}

/// Options that control how bubblewrap is invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BwrapOptions {
    /// Whether to mount a fresh `/proc`; disabled by `--no-proc` or mount fallback.
    pub mount_proc: bool,
    /// Explicitly reuse the caller's PID namespace instead of creating one.
    pub inherit_pid_namespace: bool,
    /// How networking should be configured inside the bubblewrap sandbox.
    pub network_mode: BwrapNetworkMode,
    /// Hide the WSL Windows interop socket from commands with restricted filesystem access.
    pub mask_wsl_interop: bool,
    /// Whether the host exposes WSLg's duplicate distro root, which must be hidden
    /// when constructing a restricted filesystem view.
    pub mask_wslg_distro: bool,
    /// Optional maximum depth for expanding unreadable glob patterns with ripgrep.
    ///
    /// Keep this uncapped by default so existing nested deny-read matches are
    /// masked before the sandboxed command starts.
    pub glob_scan_max_depth: Option<usize>,
    /// #536: the caller's pinned writable roots, for a session's command.
    pub pins: Option<WritableRootPins>,
}

impl Default for BwrapOptions {
    fn default() -> Self {
        Self {
            mount_proc: true,
            inherit_pid_namespace: false,
            network_mode: BwrapNetworkMode::FullAccess,
            mask_wsl_interop: false,
            mask_wslg_distro: false,
            glob_scan_max_depth: None,
            pins: None,
        }
    }
}

/// Network policy modes for bubblewrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum BwrapNetworkMode {
    /// Keep access to the host network namespace.
    #[default]
    FullAccess,
    /// Remove access to the host network namespace.
    Isolated,
    /// Intended proxy-only mode.
    ///
    /// Bubblewrap enforces this by unsharing the network namespace. The
    /// proxy-routing bridge is established by the helper process after startup.
    ProxyOnly,
}

impl BwrapNetworkMode {
    fn should_unshare_network(self) -> bool {
        !matches!(self, Self::FullAccess)
    }
}

#[derive(Debug)]
pub(crate) struct BwrapArgs {
    pub args: Vec<String>,
    pub preserved_files: Vec<File>,
    pub synthetic_mount_targets: Vec<SyntheticMountTarget>,
    /// #673: held from before the paths were looked at until the targets
    /// are registered.
    pub registry_lock: Option<RegistryLock>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileIdentity {
    dev: u64,
    ino: u64,
}

impl FileIdentity {
    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            dev: metadata.dev(),
            ino: metadata.ino(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SyntheticMountTargetKind {
    EmptyFile,
    EmptyDirectory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SyntheticMountTarget {
    path: PathBuf,
    kind: SyntheticMountTargetKind,
    // If an empty metadata path was already present, remember its inode so
    // cleanup does not delete a real pre-existing file or directory.
    pre_existing_path: Option<FileIdentity>,
    /// #607: where it is, with the folder above it resolved when the run
    /// registered it (before the command ran). It is removed there, without
    /// following a link, so a folder the command swaps for a link cannot
    /// redirect the removal.
    real_path: Option<PathBuf>,
}

impl SyntheticMountTarget {
    pub(crate) fn missing(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            kind: SyntheticMountTargetKind::EmptyFile,
            pre_existing_path: None,
            real_path: None,
        }
    }

    pub(crate) fn missing_empty_directory(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            kind: SyntheticMountTargetKind::EmptyDirectory,
            pre_existing_path: None,
            real_path: None,
        }
    }

    pub(crate) fn existing_empty_file(path: &Path, metadata: &Metadata) -> Self {
        Self {
            path: path.to_path_buf(),
            kind: SyntheticMountTargetKind::EmptyFile,
            pre_existing_path: Some(FileIdentity::from_metadata(metadata)),
            real_path: None,
        }
    }

    pub(crate) fn existing_empty_directory(path: &Path, metadata: &Metadata) -> Self {
        Self {
            path: path.to_path_buf(),
            kind: SyntheticMountTargetKind::EmptyDirectory,
            pre_existing_path: Some(FileIdentity::from_metadata(metadata)),
            real_path: None,
        }
    }

    /// #607: the same target, at `real_path` (see [`Self::removal_path`]).
    pub(crate) fn with_real_path(mut self, real_path: Option<PathBuf>) -> Self {
        self.real_path = real_path;
        self
    }

    /// #607: where it is removed: its location when it was registered, or
    /// its path when that could not be resolved.
    pub(crate) fn removal_path(&self) -> &Path {
        self.real_path.as_deref().unwrap_or(&self.path)
    }

    pub(crate) fn real_path(&self) -> Option<&Path> {
        self.real_path.as_deref()
    }

    pub(crate) fn preserves_pre_existing_path(&self) -> bool {
        self.pre_existing_path.is_some()
    }

    /// #562: whether the file or folder found when the arguments were built
    /// is still the one at the path. Another run's cleanup may have removed
    /// it before this run registered it; the mount target made then is this
    /// run's own placeholder. #607: once registered, a change is the
    /// person's (or a mount target bubblewrap made again after the path
    /// vanished in between, which is not told apart): left in place.
    pub(crate) fn pre_existing_path_is_unchanged(&self) -> bool {
        let Some(identity) = self.pre_existing_path else {
            return true;
        };
        fs::symlink_metadata(&self.path)
            .is_ok_and(|metadata| FileIdentity::from_metadata(&metadata) == identity)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn kind(&self) -> SyntheticMountTargetKind {
        self.kind
    }

    /// Whether what is at the path after bubblewrap ran is still this
    /// placeholder: an empty file, or a folder (`empty_file`, `dir`), and not
    /// the pre-existing path found when the arguments were built (`identity`
    /// is its `(dev, ino)`).
    pub(crate) fn should_remove(&self, empty_file: bool, dir: bool, identity: (u64, u64)) -> bool {
        match self.kind {
            SyntheticMountTargetKind::EmptyFile => {
                if !empty_file {
                    return false;
                }
            }
            SyntheticMountTargetKind::EmptyDirectory => {
                if !dir {
                    return false;
                }
            }
        }

        match self.pre_existing_path {
            Some(pre_existing_path) => {
                pre_existing_path
                    != FileIdentity {
                        dev: identity.0,
                        ino: identity.1,
                    }
            }
            None => true,
        }
    }
}

/// Wrap a command with bubblewrap so the filesystem is read-only by default,
/// with explicit writable roots and read-only subpaths layered afterward.
///
/// When the policy grants full disk write access and full network access, this
/// returns `command` unchanged so we avoid unnecessary sandboxing overhead.
/// If network isolation is requested, we still wrap with bubblewrap so network
/// namespace restrictions apply while preserving full filesystem access, except
/// that `/dev` is a fresh minimal device tree (#498).
pub(crate) fn create_bwrap_command_args(
    command: Vec<String>,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    sandbox_policy_cwd: &Path,
    command_cwd: &Path,
    options: BwrapOptions,
) -> Result<BwrapArgs> {
    let unreadable_globs =
        file_system_sandbox_policy.get_unreadable_glob_scans_with_cwd(sandbox_policy_cwd);
    // Full disk write normally skips bwrap, but unreadable glob patterns still
    // need concrete bwrap masks for the matches expanded below, an
    // unlistable directory (#506) its own mount, and a path that may not be
    // moved (#598) its pins.
    if file_system_sandbox_policy.has_full_disk_write_access()
        && unreadable_globs.is_empty()
        && file_system_sandbox_policy
            .get_unlistable_directories()
            .is_empty()
        && file_system_sandbox_policy.get_unmovable_paths().is_empty()
    {
        return if options.network_mode == BwrapNetworkMode::FullAccess {
            Ok(BwrapArgs {
                args: command,
                preserved_files: Vec::new(),
                synthetic_mount_targets: Vec::new(),
                registry_lock: None,
            })
        } else {
            Ok(create_bwrap_flags_full_filesystem(command, options))
        };
    }

    create_bwrap_flags(
        command,
        file_system_sandbox_policy,
        sandbox_policy_cwd,
        command_cwd,
        options,
    )
}

fn create_bwrap_flags_full_filesystem(command: Vec<String>, options: BwrapOptions) -> BwrapArgs {
    let mut args = vec![
        "--new-session".to_string(),
        "--die-with-parent".to_string(),
        "--bind".to_string(),
        "/".to_string(),
        "/".to_string(),
        // #498/#493: bubblewrap binds `nodev`, so the host's `/dev/null` cannot be
        // opened under `--bind / /`. Mount a fresh minimal `/dev`.
        "--dev".to_string(),
        "/dev".to_string(),
        // Restore shared memory without exposing other host devices.
        "--bind-try".to_string(),
        "/dev/shm".to_string(),
        "/dev/shm".to_string(),
        // Always enter a fresh user namespace so root inside a container does
        // not need ambient CAP_SYS_ADMIN to create the remaining namespaces.
        "--unshare-user".to_string(),
        "--unshare-ipc".to_string(),
    ];
    if !options.inherit_pid_namespace {
        args.push("--unshare-pid".to_string());
    }
    if options.network_mode.should_unshare_network() {
        args.push("--unshare-net".to_string());
    }
    if options.mount_proc {
        args.push("--proc".to_string());
        args.push("/proc".to_string());
    }
    args.push("--cap-drop".to_string());
    args.push("ALL".to_string());
    args.push("--".to_string());
    args.extend(command);
    BwrapArgs {
        args,
        preserved_files: Vec::new(),
        synthetic_mount_targets: Vec::new(),
        registry_lock: None,
    }
}

/// Build the bubblewrap flags (everything after `argv[0]`).
fn create_bwrap_flags(
    command: Vec<String>,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    sandbox_policy_cwd: &Path,
    command_cwd: &Path,
    options: BwrapOptions,
) -> Result<BwrapArgs> {
    if options.mask_wslg_distro {
        let granted_paths = file_system_sandbox_policy.entries.iter().flat_map(|entry| {
            FileSystemSandboxPolicy::restricted(vec![entry.clone()])
                .get_readable_roots_with_cwd(sandbox_policy_cwd)
        });
        let executable = command
            .first()
            .filter(|executable| executable.contains('/'))
            .map(|executable| AbsolutePathBuf::resolve_path_against_base(executable, command_cwd));
        for path in granted_paths
            .map(AbsolutePathBuf::into_path_buf)
            .chain([sandbox_policy_cwd.to_path_buf(), command_cwd.to_path_buf()])
            .chain(executable.map(AbsolutePathBuf::into_path_buf))
        {
            crate::wslg::ensure_supported_path(&path)?;
        }
    }
    let BwrapArgs {
        args: filesystem_args,
        preserved_files,
        synthetic_mount_targets,
        registry_lock,
    } = create_filesystem_args(file_system_sandbox_policy, sandbox_policy_cwd, options)?;
    let normalized_command_cwd = normalize_command_cwd_for_bwrap(command_cwd);
    let mut args = Vec::new();
    args.push("--new-session".to_string());
    args.push("--die-with-parent".to_string());
    args.extend(filesystem_args);
    if options.mask_wsl_interop {
        // WSL's Windows-process bridge uses Unix sockets under /run/WSL.
        // Mask them after all filesystem binds so network access cannot bypass
        // the Linux filesystem policy by launching an unrestricted Windows process.
        args.push("--tmpfs".to_string());
        args.push(WSL_INTEROP_DIR.to_string());
    }
    if (options.mask_wsl_interop || options.mask_wslg_distro) && !options.mount_proc {
        // Without a fresh procfs, the root bind retains the host procfs.
        // Hide it so host process roots cannot restore the masked WSL views.
        args.push("--tmpfs".to_string());
        args.push("/proc".to_string());
    }
    // Request a user namespace explicitly rather than relying on bubblewrap's
    // auto-enable behavior, which is skipped when the caller runs as uid 0.
    // This also blocks host procfs root/cwd/fd links through ptrace permission
    // checks, including when a container requires retaining the host procfs.
    args.push("--unshare-user".to_string());
    if !options.inherit_pid_namespace {
        args.push("--unshare-pid".to_string());
    }
    args.push("--unshare-ipc".to_string());
    if options.network_mode.should_unshare_network() {
        args.push("--unshare-net".to_string());
    }
    // Mount a fresh /proc unless the caller explicitly disables it.
    if options.mount_proc {
        args.push("--proc".to_string());
        args.push("/proc".to_string());
    }
    if normalized_command_cwd.as_path() != command_cwd {
        // Bubblewrap otherwise inherits the helper's logical cwd, which can be
        // a symlink alias that disappears once the sandbox only mounts
        // canonical roots. Enter the canonical command cwd explicitly so
        // relative paths stay aligned with the mounted filesystem view.
        args.push("--chdir".to_string());
        args.push(path_to_string(normalized_command_cwd.as_path()));
    }
    args.push("--cap-drop".to_string());
    args.push("ALL".to_string());
    args.push("--".to_string());
    args.extend(command);
    Ok(BwrapArgs {
        args,
        preserved_files,
        synthetic_mount_targets,
        registry_lock,
    })
}

/// Build the bubblewrap filesystem mounts for a given filesystem policy.
///
/// The mount order is important:
/// 1. Full-read policies, and restricted policies that explicitly read `/`,
///    use `--ro-bind / /`; other restricted-read policies start from
///    `--tmpfs /` and layer scoped `--ro-bind` mounts.
/// 2. `--dev /dev` mounts a minimal writable `/dev` with standard device nodes
///    (including `/dev/urandom`) even under a read-only root.
/// 3. Unreadable ancestors of writable roots are masked before their child
///    mounts are rebound so nested writable carveouts can be reopened safely.
/// 4. `--bind <root> <root>` re-enables writes for allowed roots, including
///    writable subpaths under `/dev` (for example, `/dev/shm`). Binding `/`
///    recreates the minimal `/dev` before applying any carveouts below it,
///    so the host's `nodev` `/dev` never covers it and `/dev/null` opens (#493).
///    For a session's command, `--bind-fd <checked folder> <root>` (#536).
/// 5. `--ro-bind <subpath> <subpath>` re-applies read-only protections under
///    those writable roots so protected subpaths win.
/// 6. Nested unreadable carveouts under a writable root are masked after that
///    root is bound, and unrelated unreadable roots are masked afterward.
fn create_filesystem_args(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    cwd: &Path,
    options: BwrapOptions,
) -> Result<BwrapArgs> {
    let daemon_directory = codex_uds::prepare_shared_daemon_socket_directory()?;
    let daemon_directories = crate::daemon_mounts::daemon_socket_mask_paths(
        &daemon_directory,
        options
            .mask_wslg_distro
            .then_some(Path::new(WSLG_DISTRO_ROOT)),
    )?;
    let pins = options.pins;
    let unreadable_globs = file_system_sandbox_policy.get_unreadable_glob_scans_with_cwd(cwd);
    // #536: for a session's command, no writable root held by another may
    // lead outside every root holding it, including the roots computed here.
    let checked = match pins {
        Some(_) => crate::pinned::check_held_links(file_system_sandbox_policy, cwd)?,
        None => Vec::new(),
    };
    // Bubblewrap requires bind mount targets to exist. Skip missing writable
    // roots so mixed-platform configs can keep harmless paths for other
    // environments without breaking Linux command startup.
    let mut writable_roots = file_system_sandbox_policy
        .get_writable_roots_with_cwd_inheriting_root_metadata(cwd)
        .into_iter()
        .filter(|writable_root| writable_root.root.as_path().exists())
        .collect::<Vec<_>>();
    if writable_roots.is_empty()
        && file_system_sandbox_policy.has_full_disk_write_access()
        && (!unreadable_globs.is_empty()
            || !file_system_sandbox_policy
                .get_unlistable_directories()
                .is_empty()
            || !file_system_sandbox_policy.get_unmovable_paths().is_empty())
    {
        writable_roots.push(WritableRoot {
            root: AbsolutePathBuf::from_absolute_path("/")?,
            read_only_subpaths: Vec::new(),
            protected_metadata_names: Vec::new(),
        });
    }
    let missing_auto_metadata_read_only_project_root_subpaths: HashSet<PathBuf> =
        file_system_sandbox_policy
            .entries
            .iter()
            .filter(|entry| entry.access == FileSystemAccessMode::Read)
            .filter_map(|entry| {
                let subpath = match &entry.path {
                    FileSystemPath::Special {
                        value:
                            FileSystemSpecialPath::ProjectRoots {
                                subpath: Some(subpath),
                            },
                    } => subpath,
                    _ => return None,
                };
                // Automatic repo-metadata read masks are skipped here so the
                // metadata handling below can apply the root-scoped
                // protection consistently for `.git`, `.agents`, and `.codex`.
                // User-authored `read` rules for other subpaths and `none`
                // rules should keep their normal bwrap behavior, which can mask
                // the first missing component to prevent creation under writable
                // roots.
                let project_subpath = Path::new(subpath);
                if project_subpath != Path::new(".git")
                    && project_subpath != Path::new(".agents")
                    && project_subpath != Path::new(".codex")
                {
                    return None;
                }
                let resolved = AbsolutePathBuf::resolve_path_against_base(subpath, cwd);
                (!resolved.as_path().exists()).then(|| resolved.into_path_buf())
            })
            .collect();
    let mut unreadable_roots = file_system_sandbox_policy
        .get_unreadable_roots_with_cwd_preserving_symlinks(cwd)
        .into_iter()
        .map(AbsolutePathBuf::into_path_buf)
        .collect::<Vec<_>>();
    // Bubblewrap can only mask concrete paths. Expand unreadable glob patterns
    // to the existing matches we can see before constructing the mount overlay;
    // core tool helpers still evaluate the original patterns directly at read time.
    unreadable_roots.extend(
        expand_unreadable_globs_with_scanner(
            &unreadable_globs,
            cwd,
            || {
                find_pre_sandbox_executable_in_path(
                    GLOB_SCAN_PROGRAM,
                    file_system_sandbox_policy,
                    cwd,
                )
            },
            options
                .glob_scan_max_depth
                .or(file_system_sandbox_policy.glob_scan_max_depth),
            |_| {},
        )?
        .into_iter()
        .map(AbsolutePathBuf::into_path_buf),
    );
    unreadable_roots.sort();
    unreadable_roots.dedup();
    // #673: from here on, whether a path exists and is another run's
    // placeholder decides how it is mounted; no placeholder is removed until
    // this run has registered those it found.
    let registry_lock = RegistryLock::acquire();

    // Read-only roots reached through a link and mounted at the link's path
    // (restricted reads), with their real targets (#501).
    let mut logical_link_roots: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut args = if file_system_sandbox_policy.has_full_disk_read_access() {
        // Read-only root, then mount a minimal device tree.
        // In bubblewrap (`bubblewrap.c`, `SETUP_MOUNT_DEV`), `--dev /dev`
        // creates the standard minimal nodes: null, zero, full, random,
        // urandom, and tty. `/dev` must be mounted before writable roots so
        // explicit `/dev/*` writable binds remain visible.
        vec![
            "--ro-bind".to_string(),
            "/".to_string(),
            "/".to_string(),
            "--dev".to_string(),
            "/dev".to_string(),
        ]
    } else {
        // Start from an empty filesystem and add only the approved readable
        // roots plus a minimal `/dev`.
        let mut args = vec![
            "--tmpfs".to_string(),
            "/".to_string(),
            "--dev".to_string(),
            "/dev".to_string(),
        ];

        let mut readable_roots: BTreeSet<PathBuf> = file_system_sandbox_policy
            .get_readable_roots_with_cwd(cwd)
            .into_iter()
            .map(PathBuf::from)
            .collect();
        if file_system_sandbox_policy.include_platform_defaults() {
            readable_roots.extend(
                LINUX_PLATFORM_DEFAULT_READ_ROOTS
                    .iter()
                    .map(|path| PathBuf::from(*path))
                    .filter(|path| path.exists()),
            );
            if options.inherit_pid_namespace {
                // Reuse the caller's procfs, including its submount masks. Bind it
                // with the other read roots so explicit deny rules still win.
                readable_roots.insert(PathBuf::from("/proc"));
            }
        }

        // A restricted policy can still explicitly request `/`, which is
        // the broad read baseline. Explicit unreadable carveouts are
        // re-applied later.
        if readable_roots.iter().any(|root| root == Path::new("/")) {
            args = vec![
                "--ro-bind".to_string(),
                "/".to_string(),
                "/".to_string(),
                "--dev".to_string(),
                "/dev".to_string(),
            ];
        } else {
            for root in readable_roots {
                if !root.exists() {
                    continue;
                }
                // Writable roots are rebound by real target below; mirror that
                // for their restricted-read bootstrap mount. Plain read-only
                // roots must stay logical because callers may execute those
                // paths inside bwrap, such as Bazel runfiles helper binaries.
                let mount_root = if writable_roots
                    .iter()
                    .any(|writable_root| root.starts_with(writable_root.root.as_path()))
                {
                    canonical_target_if_symlinked_path(&root).unwrap_or(root)
                } else {
                    if let Some(target) = canonical_target_if_symlinked_path(&root) {
                        logical_link_roots.push((root.clone(), target));
                    }
                    root
                };
                args.push("--ro-bind".to_string());
                args.push(path_to_string(&mount_root));
                args.push(path_to_string(&mount_root));
                append_daemon_socket_masks(&mut args, &mount_root, &daemon_directories)?;
            }
        }

        args
    };
    if args[0] == "--ro-bind" {
        append_daemon_socket_masks(&mut args, Path::new("/"), &daemon_directories)?;
    }
    let mut bwrap_args = BwrapArgs {
        args,
        preserved_files: Vec::new(),
        synthetic_mount_targets: Vec::new(),
        registry_lock,
    };
    let mut allowed_write_paths = Vec::with_capacity(writable_roots.len());
    for writable_root in &writable_roots {
        let root = writable_root.root.as_path();
        allowed_write_paths.push(root.to_path_buf());
        if let Some(target) = canonical_target_if_symlinked_path(root) {
            allowed_write_paths.push(target);
        }
    }
    let mut unreadable_paths: HashSet<PathBuf> = unreadable_roots.iter().cloned().collect();
    // #502: a denial spelled through a link the command cannot change is
    // masked where the link leads. Bubblewrap cannot create a mount point
    // through an absolute link (every command would fail), and inside the
    // sandbox the link leads to the same masked file. Links a command could
    // change still fail closed below.
    // #501: under restricted reads, a read-only root reached through a link
    // is mounted at the link's path, so a denial below its target is masked
    // there too.
    let unreadable_roots = {
        let mut resolved = Vec::with_capacity(unreadable_roots.len());
        for path in unreadable_roots {
            let path = if first_writable_symlink_component_in_path(&path, &allowed_write_paths)
                .is_none()
                && let Some(real) = real_path_through_links(&path, &allowed_write_paths)
            {
                real
            } else {
                path
            };
            for (root, target) in &logical_link_roots {
                if target.starts_with(&path) {
                    // The denial holds the whole linked root.
                    resolved.push(root.clone());
                } else if let Ok(relative) = path.strip_prefix(target) {
                    resolved.push(root.join(relative));
                }
            }
            resolved.push(path);
        }
        resolved.sort();
        resolved.dedup();
        // A denial inside a denied folder that is masked whole, with no
        // writable root between them, is already hidden; bubblewrap could
        // not create its mount point in the folder's read-only mask.
        let hidden: Vec<PathBuf> = resolved
            .iter()
            .filter(|path| {
                resolved.iter().any(|folder| {
                    *path != folder
                        && path.starts_with(folder)
                        && folder.is_dir()
                        && !allowed_write_paths
                            .iter()
                            .any(|root| root.starts_with(folder) && path.starts_with(root))
                })
            })
            .cloned()
            .collect();
        resolved.retain(|path| !hidden.contains(path));
        unreadable_paths.extend(resolved.iter().cloned());
        unreadable_paths.extend(hidden);
        resolved
    };
    // #503: order by the bound (canonical) path, so a symlink to `/` is bound
    // first like `/` itself instead of covering shallower roots' masks. The
    // target is resolved once, so the order and the binds agree.
    let mut sorted_writable_roots: Vec<(WritableRoot, Option<PathBuf>)> = writable_roots
        .into_iter()
        .map(|writable_root| {
            let symlink_target = canonical_target_if_symlinked_path(writable_root.root.as_path());
            (writable_root, symlink_target)
        })
        .collect();
    sorted_writable_roots.sort_by_key(|(writable_root, symlink_target)| {
        path_depth(
            symlink_target
                .as_deref()
                .unwrap_or(writable_root.root.as_path()),
        )
    });
    // #506: each unlistable directory is mounted in the pass that exposes it:
    // the last writable-root pass (in mount order) above it, or the read
    // baseline.
    let mount_roots: Vec<PathBuf> = sorted_writable_roots
        .iter()
        .map(|(writable_root, symlink_target)| {
            symlink_target
                .clone()
                .unwrap_or_else(|| writable_root.root.to_path_buf())
        })
        .collect();
    let binds_file_system_root = sorted_writable_roots
        .iter()
        .any(|(writable_root, _)| writable_root.root.as_path() == Path::new("/"));
    let unlistable_directories = unlistable_directories_by_pass(
        file_system_sandbox_policy,
        cwd,
        &mount_roots,
        &unreadable_roots,
    );
    let mut unlisted = Vec::new();
    for unlistable in &unlistable_directories {
        if unlistable.pass.is_none() {
            append_unlistable_directory_args(
                &mut bwrap_args.args,
                unlistable,
                /*writable*/ false,
            )?;
            unlisted.push(unlistable.target.clone());
        }
    }
    // Mask only the unreadable ancestors that sit outside every writable root.
    // Unreadable paths nested under a broader writable root are applied after
    // that broader root is bound, then reopened by any deeper writable child.
    let mut unreadable_ancestors_of_writable_roots: Vec<PathBuf> = unreadable_roots
        .iter()
        .filter(|path| {
            let unreadable_root = path.as_path();
            !allowed_write_paths
                .iter()
                .any(|root| unreadable_root.starts_with(root))
                && allowed_write_paths
                    .iter()
                    .any(|root| root.starts_with(unreadable_root))
        })
        .cloned()
        .collect();
    unreadable_ancestors_of_writable_roots.sort_by_key(|path| path_depth(path));

    for unreadable_root in &unreadable_ancestors_of_writable_roots {
        append_unreadable_root_args(&mut bwrap_args, unreadable_root, &allowed_write_paths)?;
    }

    // #587: once the policy denies reads (as Seatbelt's rename guard), the
    // folders above each protected path inside a writable root are pinned.
    let pin_protected_folders = !unreadable_roots.is_empty();
    // #594/#598: and, whatever else the policy holds, each path it says may
    // not be moved and the folders above it.
    let unmovable = unmovable_pin_targets(file_system_sandbox_policy);
    // `:root = write` (full access with denials), not a project at `/`.
    let full_write_root = file_system_sandbox_policy.entries.iter().any(|entry| {
        entry.access == FileSystemAccessMode::Write
            && matches!(
                entry.path,
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root
                }
            )
    });
    // #503: read-only paths applied by earlier passes, by bound path. A later
    // pass whose bind covers them (two roots bound at the same directory, or a
    // link whose target lies inside an earlier root) applies them again.
    let mut earlier_read_only_subpaths: Vec<PathBuf> = Vec::new();
    // #503: the same for denials, in the form an earlier pass masked them
    // (a denial spelled through a link is masked at the link's target).
    let mut earlier_unreadable_roots: Vec<PathBuf> = Vec::new();
    for (writable_root, symlink_target) in &sorted_writable_roots {
        let root = writable_root.root.as_path();
        let mount_root = symlink_target.as_deref().unwrap_or(root);
        // #473: placeholders are deduplicated within this pass only. Its
        // `--bind` puts the live host folder back over every mount an earlier
        // (broader) pass made below it.
        let earlier_synthetic_mount_targets =
            std::mem::take(&mut bwrap_args.synthetic_mount_targets);
        // If a denied ancestor was already masked, recreate any missing mount
        // target parents before binding the narrower writable descendant.
        if let Some(masking_root) = unreadable_roots
            .iter()
            .map(PathBuf::as_path)
            .filter(|unreadable_root| mount_root.starts_with(unreadable_root))
            .max_by_key(|unreadable_root| path_depth(unreadable_root))
        {
            append_mount_target_parent_dir_args(&mut bwrap_args.args, mount_root, masking_root);
        }

        // Rebinding a root alias would also undo masks already applied to `/`.
        // The physical root is already writable; only the alias's carveouts remain.
        let redundant_root_alias =
            binds_file_system_root && root != Path::new("/") && mount_root == Path::new("/");
        if !redundant_root_alias {
            // #536: for a session's command, every writable root is bound from
            // the folder opened and checked at its pinned location.
            match pins {
                Some(pins) => {
                    if !pins.bind_fd {
                        return Err(CodexErr::UnsupportedOperation(
                            "Refused to run the command: binding a checked writable folder needs \
                             bubblewrap 0.8 or newer (`--bind-fd`), and none was found."
                                .to_string(),
                        ));
                    }
                    let real =
                        crate::pinned::expected_location(pins.roots, &checked, &writable_root.root);
                    let file = crate::pinned::open_at(&[root, mount_root], &real)?;
                    bwrap_args.args.push("--bind-fd".to_string());
                    bwrap_args.args.push(file.as_raw_fd().to_string());
                    bwrap_args.args.push(path_to_string(mount_root));
                    bwrap_args.preserved_files.push(file);
                }
                None => {
                    bwrap_args.args.push("--bind".to_string());
                    bwrap_args.args.push(path_to_string(mount_root));
                    bwrap_args.args.push(path_to_string(mount_root));
                }
            }
            // #493: the root bind shadows the earlier device tree and applies
            // nodev. Restore the minimal `/dev` (step 2) before the deny masks
            // of this pass and before deeper `/dev/*` writable roots (sorted
            // later), with shared memory but no other host devices.
            let dev = Path::new("/dev");
            if dev.starts_with(mount_root) && mount_root != dev {
                bwrap_args.args.extend([
                    "--dev".to_string(),
                    "/dev".to_string(),
                    "--bind-try".to_string(),
                    "/dev/shm".to_string(),
                    "/dev/shm".to_string(),
                ]);
            }
            append_daemon_socket_masks(&mut bwrap_args.args, mount_root, &daemon_directories)?;
        }

        let mut read_only_subpaths: Vec<PathBuf> = writable_root
            .read_only_subpaths
            .iter()
            .map(|path| path.as_path().to_path_buf())
            .filter(|path| !unreadable_paths.contains(path))
            .filter(|path| !missing_auto_metadata_read_only_project_root_subpaths.contains(path))
            // The filesystem root of a full-write profile with denials (PF-23
            // protected paths under full access) is not a project root, and
            // an unprivileged bwrap cannot create a mount point there:
            // `mkdir /.git` or `/.codex` would fail every command.
            .filter(|path| {
                !(full_write_root && mount_root.parent().is_none() && path.parent() == Some(root))
                    || path.exists()
            })
            .collect();
        let protected_metadata_names = if mount_root.parent().is_none() && full_write_root {
            Vec::new()
        } else {
            writable_root.protected_metadata_names.clone()
        };
        append_metadata_path_masks_for_writable_root(
            &mut read_only_subpaths,
            root,
            &protected_metadata_names,
        );
        if let Some(target) = symlink_target {
            read_only_subpaths = remap_paths_for_symlink_target(read_only_subpaths, root, target);
        }
        // A link's own bound path stays read-only when an earlier root made
        // it read-only (for example a link to a workspace's `.git`).
        for path in &earlier_read_only_subpaths {
            if path.starts_with(mount_root)
                && (symlink_target.is_some() || path.as_path() != mount_root)
                && !read_only_subpaths.contains(path)
            {
                read_only_subpaths.push(path.clone());
            }
        }
        for path in &read_only_subpaths {
            if !earlier_read_only_subpaths.contains(path) {
                earlier_read_only_subpaths.push(path.clone());
            }
        }
        read_only_subpaths.sort_by_key(|path| path_depth(path));
        // A root reached through a link is bound at its target, so a denied
        // path spelled under the target is nested too (#488); the rootless
        // pass below skips it because the target is an allowed write path.
        let mut nested_unreadable_roots: Vec<PathBuf> = unreadable_roots
            .iter()
            .filter(|path| {
                path.starts_with(root)
                    || symlink_target
                        .as_ref()
                        .is_some_and(|target| path.starts_with(target))
            })
            .cloned()
            .collect();
        if let Some(target) = symlink_target {
            nested_unreadable_roots =
                remap_paths_for_symlink_target(nested_unreadable_roots, root, target);
        }
        nested_unreadable_roots.extend(
            earlier_unreadable_roots
                .iter()
                .filter(|path| path.starts_with(mount_root))
                .cloned(),
        );
        nested_unreadable_roots.sort();
        nested_unreadable_roots.dedup();
        for path in &nested_unreadable_roots {
            if !earlier_unreadable_roots.contains(path) {
                earlier_unreadable_roots.push(path.clone());
            }
        }
        nested_unreadable_roots.sort_by_key(|path| path_depth(path));
        // #473: denied paths missing below a read-only folder, by the folder
        // that gets a snapshot (see `append_read_only_snapshot_dir_args`).
        let mut snapshot_dirs: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
        let mut snapshot_covered: HashSet<PathBuf> = HashSet::new();
        for unreadable_root in &nested_unreadable_roots {
            if let Some(missing) = missing_component_below_read_only_path(
                unreadable_root,
                &read_only_subpaths,
                &allowed_write_paths,
            ) && let Some(dir) = missing.parent()
            {
                snapshot_dirs
                    .entry(dir.to_path_buf())
                    .or_default()
                    .push(missing.clone());
                snapshot_covered.insert(unreadable_root.clone());
            }
        }
        // #506: this pass's unlistable directories go among its read-only
        // mounts so that none of those binds the real folder back over them:
        // after every read-only folder at or above them and before the ones
        // below (deny masks come later still); after a snapshot folder above
        // them. A snapshot of the folder itself is made unlistable instead.
        // In writable passes an unlistable folder's target is its real
        // location, where the pass's own mounts are.
        let pass_unlistable: Vec<&Unlistable> = unlistable_directories
            .iter()
            .filter(|unlistable| unlistable.pass.as_deref() == Some(mount_root))
            .collect();
        let under_snapshot = |directory: &Path| {
            snapshot_dirs
                .keys()
                .any(|dir| directory.starts_with(dir) && directory != dir)
        };
        let mut pending_unlistable: Vec<(&Unlistable, bool)> = pass_unlistable
            .iter()
            .filter(|unlistable| {
                !snapshot_dirs.contains_key(&unlistable.target)
                    && !under_snapshot(&unlistable.target)
            })
            .map(|unlistable| {
                let read_only = read_only_subpaths
                    .iter()
                    .any(|read_only| unlistable.target.starts_with(read_only));
                (*unlistable, read_only)
            })
            .collect();
        if pin_protected_folders || !unmovable.is_empty() {
            let protected: Vec<&Path> = read_only_subpaths
                .iter()
                .chain(&nested_unreadable_roots)
                .map(PathBuf::as_path)
                .chain(
                    pass_unlistable
                        .iter()
                        .map(|unlistable| unlistable.target.as_path()),
                )
                .collect();
            append_protected_folder_pins(
                &mut bwrap_args,
                mount_root,
                &protected,
                pin_protected_folders,
                &unmovable,
                &allowed_write_paths,
            );
        }
        for subpath in &read_only_subpaths {
            let depth = path_depth(subpath);
            for (unlistable, read_only) in pending_unlistable
                .extract_if(.., |(unlistable, _)| depth > path_depth(&unlistable.target))
            {
                append_unlistable_directory_args(&mut bwrap_args.args, unlistable, !read_only)?;
                unlisted.push(unlistable.target.clone());
            }
            if missing_component_below_read_only_path(
                subpath,
                &read_only_subpaths,
                &allowed_write_paths,
            )
            .is_some()
            {
                // #473: the read-only folder above already stops the command
                // creating it, and bubblewrap cannot create a mount target
                // in a read-only mount.
                continue;
            }
            append_read_only_subpath_args(
                &mut bwrap_args,
                subpath,
                &allowed_write_paths,
                &daemon_directories,
            )?;
        }
        for (unlistable, read_only) in pending_unlistable {
            append_unlistable_directory_args(&mut bwrap_args.args, unlistable, !read_only)?;
            unlisted.push(unlistable.target.clone());
        }
        let after_snapshots: Vec<&Unlistable> = pass_unlistable
            .iter()
            .filter(|unlistable| under_snapshot(&unlistable.target))
            .copied()
            .collect();
        let mut snapshot_dirs: Vec<(PathBuf, Vec<PathBuf>)> = snapshot_dirs.into_iter().collect();
        snapshot_dirs.sort_by_key(|(dir, _)| path_depth(dir));
        for (dir, hidden) in &snapshot_dirs {
            let unlistable = pass_unlistable
                .iter()
                .any(|unlistable| unlistable.target == *dir);
            append_read_only_snapshot_dir_args(
                &mut bwrap_args,
                dir,
                hidden,
                if unlistable { "111" } else { "555" },
            )?;
            if unlistable {
                unlisted.push(dir.clone());
            }
        }
        // Inside a read-only snapshot: read-only too.
        for unlistable in after_snapshots {
            append_unlistable_directory_args(
                &mut bwrap_args.args,
                unlistable,
                /*writable*/ false,
            )?;
            unlisted.push(unlistable.target.clone());
        }
        // Protect the registry only where a writable bind exposes it. Apply
        // this before deny masks so a denied parent stays hidden, rather than
        // trying to recreate the registry beneath an already read-only tmpfs.
        let registry_root = synthetic_mount_registry_root();
        let registry_mount = if registry_root.starts_with(mount_root) {
            Some(registry_root.as_path())
        } else if mount_root.starts_with(&registry_root) {
            Some(mount_root)
        } else {
            None
        };
        if let Some(registry_mount) = registry_mount {
            fs::create_dir_all(&registry_root)?;
            bwrap_args.args.push("--ro-bind".to_string());
            bwrap_args.args.push(path_to_string(registry_mount));
            bwrap_args.args.push(path_to_string(registry_mount));
            append_daemon_socket_masks(&mut bwrap_args.args, registry_mount, &daemon_directories)?;
        }
        // #348: the same for the placeholder record kept in the Corbanu
        // home, so the command cannot forge a record a later run acts on.
        if let Some(record_mount) = crate::placeholder_journal::record_dir_mount(mount_root) {
            bwrap_args.args.push("--ro-bind".to_string());
            bwrap_args.args.push(path_to_string(&record_mount));
            bwrap_args.args.push(path_to_string(&record_mount));
        }
        for unreadable_root in nested_unreadable_roots {
            if snapshot_covered.contains(&unreadable_root) {
                // Hidden by the snapshot above, also if it is created later.
                continue;
            }
            append_unreadable_root_args(&mut bwrap_args, &unreadable_root, &allowed_write_paths)?;
        }
        let pass_synthetic_mount_targets = std::mem::replace(
            &mut bwrap_args.synthetic_mount_targets,
            earlier_synthetic_mount_targets,
        );
        bwrap_args
            .synthetic_mount_targets
            .extend(pass_synthetic_mount_targets);
    }

    let mut rootless_unreadable_roots: Vec<PathBuf> = unreadable_roots
        .iter()
        .filter(|path| {
            let unreadable_root = path.as_path();
            !allowed_write_paths
                .iter()
                .any(|root| unreadable_root.starts_with(root) || root.starts_with(unreadable_root))
        })
        .cloned()
        .collect();
    rootless_unreadable_roots.sort_by_key(|path| path_depth(path));
    for unreadable_root in rootless_unreadable_roots {
        append_unreadable_root_args(&mut bwrap_args, &unreadable_root, &allowed_write_paths)?;
    }
    // #506: last, once every mount target below them exists, so the command
    // cannot change their mode (or add entries) either.
    for directory in unlisted {
        bwrap_args.args.push("--remount-ro".to_string());
        bwrap_args.args.push(path_to_string(&directory));
    }

    if options.mask_wslg_distro
        && !unreadable_roots.iter().any(|denied| {
            let distro = Path::new(WSLG_DISTRO_ROOT);
            distro.starts_with(denied)
                && !allowed_write_paths.iter().any(|root| {
                    root.starts_with(denied)
                        && (distro.starts_with(root) || root.starts_with(distro))
                })
        })
    {
        // Use the expanded masks, including symlink targets from deny globs.
        // An already hidden ancestor cannot accommodate a new child mount.
        // Otherwise apply this after every grant to exclude the duplicate view.
        bwrap_args.args.extend([
            "--perms".to_string(),
            "000".to_string(),
            "--tmpfs".to_string(),
            WSLG_DISTRO_ROOT.to_string(),
            "--remount-ro".to_string(),
            WSLG_DISTRO_ROOT.to_string(),
        ]);
    }

    Ok(bwrap_args)
}

/// #587: binds each folder between `mount_root` and a protected path onto
/// itself, before the pass's masks. A mount point cannot be renamed or
/// removed (`EBUSY`), so a command cannot move a protected path's folder
/// aside (`mv ~/.corbanu x`, then plant a new `~/.corbanu`) and so leave the
/// rules behind: the new folder would have none, and the moved copy's
/// denied stores would be readable under its new name. Seatbelt denies the
/// same renames (`build_seatbelt_rename_guard_policy`). The bind is the live
/// folder, so what is in it is unchanged; a folder gone by then (another
/// run's placeholder) is skipped (`--bind-try`). Folders at or below a
/// protected path (masked or read-only already), writable roots (bound
/// already), links, folders at or below `/dev` (a bind would put the host's
/// back over the minimal one, #493) and folders in a parent the user
/// neither owns nor can write (they cannot be renamed, such as `/home`) are
/// left out.
///
/// `above_protected`: whether the folders above `protected` are pinned
/// (#587, once the policy denies reads); they are always left out. #594/#598:
/// the folders above each [`UnmovableTarget`] are pinned in any case, and
/// the target itself when it is a folder.
fn append_protected_folder_pins(
    bwrap_args: &mut BwrapArgs,
    mount_root: &Path,
    protected: &[&Path],
    above_protected: bool,
    unmovable: &[UnmovableTarget],
    allowed_write_paths: &[PathBuf],
) {
    let mut folders = BTreeSet::new();
    let sources = protected
        .iter()
        .filter(|_| above_protected)
        .map(|path| (*path, /*itself*/ false))
        .chain(
            unmovable
                .iter()
                .map(|target| (target.path.as_path(), target.pin_itself)),
        );
    for (path, itself) in sources {
        for folder in path.ancestors().skip(usize::from(!itself)) {
            if folder == mount_root || !folder.starts_with(mount_root) {
                break;
            }
            folders.insert(folder);
        }
    }
    let mut pins: Vec<&Path> = folders
        .into_iter()
        .filter(|folder| {
            !protected.iter().any(|path| folder.starts_with(path))
                && !folder.starts_with("/dev")
                && !allowed_write_paths.iter().any(|root| root == folder)
                && fs::symlink_metadata(folder).is_ok_and(|metadata| metadata.is_dir())
                && folder.parent().is_some_and(user_may_change)
        })
        .collect();
    pins.sort_by_key(|folder| path_depth(folder));
    for folder in pins {
        bwrap_args.args.push("--bind-try".to_string());
        bwrap_args.args.push(path_to_string(folder));
        bwrap_args.args.push(path_to_string(folder));
    }
}

/// #594/#598: a location the sandbox keeps in place for a path the policy
/// says may not be moved ([`FileSystemSandboxPolicy::get_unmovable_paths`]).
#[derive(Debug, PartialEq, Eq)]
struct UnmovableTarget {
    /// Real location: of the path, or of a link on its way (the link's own
    /// folder resolved).
    path: PathBuf,
    /// Whether the location itself is pinned (a folder), or only the folders
    /// above it (a link: a mount cannot sit on a link, so a command that can
    /// write the folder holding it can still replace it; Seatbelt keeps it).
    pin_itself: bool,
}

/// #594/#598: the real location of each unmovable path, and of each link on
/// its way (a Corbanu home reached through `~/.corbanu -> ~/dotfiles/x` keeps
/// `~/dotfiles/x`, `~/dotfiles` and the folders above `~/.corbanu`).
fn unmovable_pin_targets(policy: &FileSystemSandboxPolicy) -> Vec<UnmovableTarget> {
    let mut targets: Vec<UnmovableTarget> = Vec::new();
    let mut push = |path: PathBuf, pin_itself: bool| {
        if !targets
            .iter()
            .any(|target| target.path == path && target.pin_itself == pin_itself)
        {
            targets.push(UnmovableTarget { path, pin_itself });
        }
    };
    for path in policy.get_unmovable_paths() {
        let path = path.as_path();
        if let Ok(real) = fs::canonicalize(path) {
            push(real, /*pin_itself*/ true);
        }
        let mut spelled = PathBuf::new();
        for component in path.components() {
            spelled.push(component);
            if fs::symlink_metadata(&spelled)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
                && let (Some(parent), Some(name)) = (spelled.parent(), spelled.file_name())
                && let Ok(parent) = fs::canonicalize(parent)
            {
                push(parent.join(name), /*pin_itself*/ false);
            }
        }
    }
    targets
}

/// Whether the user could rename an entry of `folder`: it can write it, or
/// owns it (and so could make it writable first).
fn user_may_change(folder: &Path) -> bool {
    // SAFETY: `geteuid` has no preconditions.
    let euid = unsafe { libc::geteuid() };
    if fs::metadata(folder).is_ok_and(|metadata| metadata.uid() == euid) {
        return true;
    }
    let Ok(folder) = std::ffi::CString::new(folder.as_os_str().as_encoded_bytes()) else {
        return false;
    };
    // SAFETY: `folder` is a valid NUL-terminated string for the call.
    unsafe { libc::access(folder.as_ptr(), libc::W_OK) == 0 }
}

/// #506: one unlistable directory, as the sandbox shows it.
struct Unlistable {
    /// The mount root of the last writable-root pass (in mount order) that
    /// contains it; `None`: the read baseline exposes it.
    pass: Option<PathBuf>,
    /// Its real location on the host, where its entries are bound from.
    source: PathBuf,
    /// Where the command sees it: the real location, or below a read-only
    /// root bound at its own (logical) spelling.
    target: PathBuf,
}

/// #506: the policy's unlistable directories that exist and the command can
/// see. A directory hidden by a deny mask is left out.
fn unlistable_directories_by_pass(
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
    cwd: &Path,
    mount_roots: &[PathBuf],
    unreadable_roots: &[PathBuf],
) -> Vec<Unlistable> {
    // Restricted-read baselines bind readable roots at their own spelling.
    let readable_roots: Vec<(PathBuf, PathBuf)> = file_system_sandbox_policy
        .get_readable_roots_with_cwd(cwd)
        .into_iter()
        .map(|root| {
            let real = root
                .as_path()
                .canonicalize()
                .unwrap_or_else(|_| root.to_path_buf());
            (root.to_path_buf(), real)
        })
        .collect();
    let mut directories: Vec<Unlistable> = Vec::new();
    for directory in file_system_sandbox_policy.get_unlistable_directories() {
        let Ok(source) = directory.as_path().canonicalize() else {
            continue;
        };
        if !source.is_dir() || directories.iter().any(|known| known.source == source) {
            continue;
        }
        // Passes run in this order; a later one binds over an earlier one.
        let pass = mount_roots
            .iter()
            .rfind(|root| source.starts_with(root))
            .cloned();
        // Hidden by a deny mask no writable root below it reopens.
        let hidden = unreadable_roots.iter().any(|unreadable| {
            source.starts_with(unreadable)
                && !pass
                    .as_ref()
                    .is_some_and(|root| root.starts_with(unreadable))
        });
        if hidden {
            continue;
        }
        let target = if pass.is_some() || file_system_sandbox_policy.has_full_disk_read_access() {
            Some(source.clone())
        } else {
            readable_roots
                .iter()
                .filter(|(_, real)| source.starts_with(real))
                .max_by_key(|(_, real)| path_depth(real))
                .and_then(|(logical, real)| {
                    source.strip_prefix(real).ok().map(|rest| {
                        if rest.as_os_str().is_empty() {
                            logical.clone()
                        } else {
                            logical.join(rest)
                        }
                    })
                })
        };
        if let Some(target) = target {
            directories.push(Unlistable {
                pass,
                source,
                target,
            });
        }
    }
    directories
}

/// #506: replace the directory, where the command sees it, with an empty
/// folder only its owner can pass through (mode 0111: no listing), and put
/// each entry back as it was: links as links, everything else bound from the
/// host. The folder is made read-only at the end (see
/// [`create_filesystem_args`]). A directory that cannot be read fails the
/// command rather than staying listable.
fn append_unlistable_directory_args(
    args: &mut Vec<String>,
    unlistable: &Unlistable,
    writable: bool,
) -> Result<()> {
    let Unlistable { source, target, .. } = unlistable;
    let directory = source.as_path();
    let entries = fs::read_dir(directory)
        .and_then(Iterator::collect::<io::Result<Vec<_>>>)
        .map_err(|err| {
            CodexErr::Fatal(format!(
                "cannot keep {} from being listed: {err}",
                directory.display()
            ))
        })?;
    let mut children: Vec<(std::ffi::OsString, Option<PathBuf>)> = entries
        .into_iter()
        .map(|entry| {
            let link = entry
                .file_type()
                .is_ok_and(|kind| kind.is_symlink())
                .then(|| fs::read_link(entry.path()).ok())
                .flatten();
            (entry.file_name(), link)
        })
        .collect();
    children.sort();
    args.extend([
        "--perms".to_string(),
        "111".to_string(),
        "--tmpfs".to_string(),
        path_to_string(target),
    ]);
    for (name, link) in children {
        let at = path_to_string(&target.join(&name));
        match link {
            Some(link) => args.extend(["--symlink".to_string(), path_to_string(&link), at]),
            None => args.extend([
                if writable {
                    "--bind-try"
                } else {
                    "--ro-bind-try"
                }
                .to_string(),
                path_to_string(&source.join(&name)),
                at,
            ]),
        }
    }
    Ok(())
}

fn append_metadata_path_masks_for_writable_root(
    read_only_subpaths: &mut Vec<PathBuf>,
    root: &Path,
    protected_metadata_names: &[String],
) {
    for name in protected_metadata_names {
        let path = root.join(name);
        if !read_only_subpaths.iter().any(|subpath| subpath == &path) {
            read_only_subpaths.push(path);
        }
    }
}

// Mask immediately after every bind that exposes the directory, before policy
// deny masks can hide its parent. Use the mount destination for read-root aliases.
fn append_daemon_socket_masks(
    args: &mut Vec<String>,
    mount_root: &Path,
    daemon_directories: &BTreeSet<PathBuf>,
) -> Result<()> {
    let source = fs::canonicalize(mount_root)?;
    for daemon_directory in daemon_directories {
        if source.starts_with(daemon_directory) {
            return Err(CodexErr::Fatal(
                "app-server socket directory cannot be a sandbox mount root".to_string(),
            ));
        }
        if let Ok(relative) = daemon_directory.strip_prefix(&source) {
            let destination = mount_root.join(relative);
            // A lossy conversion could mask a different path and expose the socket.
            let destination = destination
                .to_str()
                .ok_or_else(|| {
                    CodexErr::Fatal("app-server socket mask path must be valid UTF-8".to_string())
                })?
                .to_string();
            args.extend([
                "--perms".to_string(),
                "000".to_string(),
                "--tmpfs".to_string(),
                destination.clone(),
                "--remount-ro".to_string(),
                destination,
            ]);
        }
    }
    Ok(())
}

/// Expand deny globs with the launcher's scanner in an explicit command environment.
/// The supplied environment replaces inheritance, including when it is empty.
pub fn expand_unreadable_globs_in_environment(
    patterns: &[String],
    cwd: &Path,
    file_system_policy: &FileSystemSandboxPolicy,
    max_depth: Option<usize>,
    env: &HashMap<String, String>,
    command_cwd: &Path,
) -> Result<Vec<AbsolutePathBuf>> {
    expand_unreadable_globs_with_ripgrep(
        patterns,
        cwd,
        || {
            find_executable_in_search_paths(
                GLOB_SCAN_PROGRAM,
                std::env::split_paths(env.get("PATH")?),
                command_cwd,
                file_system_policy,
                cwd,
            )
        },
        max_depth,
        |command| {
            command.env_clear().envs(env).current_dir(command_cwd);
        },
    )
}

/// Upstream's entry point for plain glob patterns (no #732 database scans):
/// resolves the scanner once and walks with the built-in globset walker when
/// none was found.
fn expand_unreadable_globs_with_ripgrep(
    patterns: &[String],
    cwd: &Path,
    resolve_scanner: impl FnOnce() -> Option<PathBuf>,
    max_depth: Option<usize>,
    configure_command: impl Fn(&mut Command),
) -> Result<Vec<AbsolutePathBuf>> {
    let patterns: Vec<FileSystemDenyGlob> = patterns
        .iter()
        .map(|pattern| FileSystemDenyGlob {
            pattern: pattern.clone(),
            scan: None,
        })
        .collect();
    expand_unreadable_globs_with_scanner(
        &patterns,
        cwd,
        resolve_scanner,
        max_depth,
        configure_command,
    )
}

/// Resolves the ripgrep scanner (outside every writable root, #51407) and
/// expands the policy's deny globs with it, or with the built-in walker when
/// none was found.
fn expand_unreadable_globs_with_scanner(
    patterns: &[FileSystemDenyGlob],
    cwd: &Path,
    resolve_scanner: impl FnOnce() -> Option<PathBuf>,
    max_depth: Option<usize>,
    configure_command: impl Fn(&mut Command),
) -> Result<Vec<AbsolutePathBuf>> {
    if patterns.is_empty() || max_depth == Some(0) {
        return Ok(Vec::new());
    }
    let rg_path = resolve_scanner();
    let scan = |search_root: &Path, globs: &[String], depth: Option<usize>| match &rg_path {
        Some(rg_path) => ripgrep_files(rg_path, search_root, globs, depth, &configure_command),
        None => glob_files(search_root, globs, depth),
    };
    expand_unreadable_globs(patterns, cwd, max_depth, &scan)
}

type GlobFileScanner<'a> =
    &'a dyn Fn(&Path, &[String], Option<usize>) -> Result<Vec<AbsolutePathBuf>>;
/// Search root, credential rule, depth, and whether it is a database-entry
/// scan (#732).
type ScanGroup = (AbsolutePathBuf, bool, Option<usize>, bool);

fn expand_unreadable_globs(
    patterns: &[FileSystemDenyGlob],
    cwd: &Path,
    max_depth: Option<usize>,
    scan_files: GlobFileScanner<'_>,
) -> Result<Vec<AbsolutePathBuf>> {
    if patterns.is_empty() || max_depth == Some(0) {
        return Ok(Vec::new());
    }

    // Group each pattern by the static path prefix before its first glob
    // metacharacter. That keeps scans narrow, avoids searching from `/`, and
    // lets one `rg --files` call handle all patterns under the same root.
    // The whole credential rule's globs are scanned apart: their matches
    // leave out the template names (#442, Travis 2026-10-12), which other
    // globs still cover.
    let plain_patterns: Vec<String> = patterns.iter().map(|glob| glob.pattern.clone()).collect();
    let rule_roots = credential_rule_roots(&plain_patterns);
    // #732: depth is part of the group identity. A generic glob sharing this
    // root (even the identical pattern text) retains its own recursive scan.
    // #732 (D226): a database-entry scan lists only its folder's own entries.
    let mut patterns_by_search_root: BTreeMap<ScanGroup, Vec<String>> = BTreeMap::new();
    for pattern in patterns {
        let database_directory = pattern
            .database_entries_directory()
            .map_err(|message| CodexErr::Fatal(message.to_string()))?;
        let (search_root, glob, depth, database) = match database_directory {
            Some(directory) => (directory.clone(), "*.sqlite*".to_string(), Some(1), true),
            None => {
                // Fail closed (#38026): a pattern with no non-root prefix
                // cannot be scanned, and skipping it would drop the deny.
                let Some((root, glob)) = split_pattern_for_ripgrep(&pattern.pattern, cwd) else {
                    return Err(CodexErr::Fatal(format!(
                        "unreadable glob `{}` cannot be safely expanded; use a pattern with a non-root directory prefix",
                        pattern.pattern
                    )));
                };
                (root, glob, max_depth, false)
            }
        };
        // Follow symlink roots, but never discard a deny because inspecting its
        // search root failed. Only absent or non-directory roots have no matches.
        match fs::metadata(search_root.as_path()) {
            Ok(metadata) if metadata.is_dir() => {
                let credential = credential_glob_exempts_templates(&pattern.pattern, &rule_roots);
                patterns_by_search_root
                    .entry((search_root, credential, depth, database))
                    .or_default()
                    .push(glob);
            }
            Ok(_) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) => {}
            Err(err) => {
                return Err(CodexErr::Fatal(format!(
                    "failed to inspect unreadable glob search root {}: {err}",
                    search_root.display()
                )));
            }
        }
    }

    // Record both the logical match and any canonical symlink target. The bwrap
    // overlay needs the resolved target to prevent a readable symlink path from
    // bypassing an unreadable glob match.
    let mut expanded_paths = BTreeSet::new();
    for ((search_root, credential, depth, database), globs) in patterns_by_search_root {
        // #732 (D226): ripgrep still opens each subfolder at `--max-depth 1`
        // (ripgrep 14.1), so an unreadable one failed the scan; listing the
        // folder itself never opens a subfolder.
        let paths = if database {
            glob_files(search_root.as_path(), &globs, depth)?
        } else {
            scan_files(search_root.as_path(), &globs, depth)?
        };
        for path in paths {
            if credential
                && path
                    .as_path()
                    .file_name()
                    .is_some_and(is_credential_template_name)
            {
                continue;
            }
            if let Some(target) = canonical_target_if_symlinked_path(path.as_path()) {
                expanded_paths.insert(AbsolutePathBuf::from_absolute_path_checked(target)?);
            }
            expanded_paths.insert(path);
            if expanded_paths.len() > MAX_UNREADABLE_GLOB_MATCHES {
                return Err(CodexErr::Fatal(format!(
                    "unreadable glob expansion for {} matched more than {MAX_UNREADABLE_GLOB_MATCHES} paths",
                    search_root.display()
                )));
            }
        }
    }

    Ok(expanded_paths.into_iter().collect())
}

fn split_pattern_for_ripgrep(pattern: &str, cwd: &Path) -> Option<(AbsolutePathBuf, String)> {
    // Resolve relative patterns once, then split at the first glob
    // metacharacter. The prefix becomes the search root and the suffix stays as
    // the ripgrep glob. Root-level glob scans are intentionally skipped because
    // they are too broad for startup-time sandbox construction.
    let absolute_pattern = AbsolutePathBuf::resolve_path_against_base(pattern, cwd);
    let pattern = absolute_pattern.to_string_lossy();
    let first_glob_index = pattern
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '*' | '?' | '[' | ']').then_some(index))?;
    let static_prefix = &pattern[..first_glob_index];
    if static_prefix.is_empty() || static_prefix == "/" {
        return None;
    }
    let search_root_end = if static_prefix.ends_with('/') {
        static_prefix.len() - 1
    } else {
        static_prefix.rfind('/').unwrap_or(0)
    };
    let search_root = if search_root_end == 0 {
        PathBuf::from("/")
    } else {
        PathBuf::from(&pattern[..search_root_end])
    };
    let search_root = AbsolutePathBuf::from_absolute_path_checked(search_root).ok()?;
    let glob = escape_unclosed_glob_classes(&pattern[search_root_end + 1..]);
    (!glob.is_empty()).then_some((search_root, glob))
}

fn escape_unclosed_glob_classes(glob: &str) -> String {
    // The filesystem policy accepts an unclosed `[` as a literal. Ripgrep treats
    // that as invalid glob syntax, so escape only the unclosed class opener.
    let mut escaped = String::with_capacity(glob.len());
    let mut chars = glob.chars();

    while let Some(ch) = chars.next() {
        if ch != '[' {
            escaped.push(ch);
            continue;
        }

        let mut class = String::new();
        let mut closed = false;
        for class_ch in chars.by_ref() {
            if class_ch == ']' {
                closed = true;
                break;
            }
            class.push(class_ch);
        }

        if closed {
            escaped.push('[');
            escaped.push_str(&class);
            escaped.push(']');
        } else {
            escaped.push_str(r"\[");
            escaped.push_str(&class);
        }
    }

    escaped
}

fn ripgrep_files(
    rg_path: &Path,
    search_root: &Path,
    globs: &[String],
    max_depth: Option<usize>,
    configure_command: &impl Fn(&mut Command),
) -> Result<Vec<AbsolutePathBuf>> {
    let mut command = Command::new(rg_path);
    configure_command(&mut command);
    // Use `rg --files` rather than shell expansion so dotfiles and ignored files
    // are still considered. A status 1 with no stderr is ripgrep's "no matches"
    // case, not a sandbox construction error.
    command
        .arg("--no-config")
        .arg("--files")
        .arg("--hidden")
        .arg("--no-ignore")
        .arg("--null");
    if let Some(max_depth) = max_depth {
        command.arg("--max-depth").arg(max_depth.to_string());
    }
    for glob in globs {
        command.arg("--glob").arg(glob);
    }
    command.arg("--").arg(search_root);

    /*
     * Prefer ripgrep for unreadable glob expansion because it is fast and
     * already implements the file-walking semantics we want here: include
     * dotfiles, ignore ignore files, and do not recurse through symlinked
     * directories. If the selected `rg` is unavailable at execution time, fall
     * back to the internal globset walker so sandbox construction still masks
     * matching paths. Other ripgrep failures stay fatal so deny-read does not
     * silently weaken.
     */
    let output = match command.output() {
        Ok(output) => output,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return glob_files(search_root, globs, max_depth);
        }
        Err(err) => return Err(err.into()),
    };
    if !output.status.success() {
        if output.status.code() == Some(1) && output.stderr.is_empty() {
            return Ok(Vec::new());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CodexErr::Fatal(format!(
            "ripgrep unreadable glob scan failed for {}: {stderr}",
            search_root.display()
        )));
    }

    let paths = output
        .stdout
        .split(|byte| *byte == b'\0')
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = PathBuf::from(OsString::from_vec(path.to_vec()));
            if path.is_absolute() {
                path
            } else {
                search_root.join(path)
            }
        })
        .map(AbsolutePathBuf::from_absolute_path_checked)
        .collect::<io::Result<Vec<_>>>()?;
    Ok(paths)
}

fn glob_files(
    search_root: &Path,
    globs: &[String],
    max_depth: Option<usize>,
) -> Result<Vec<AbsolutePathBuf>> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        let glob = GlobBuilder::new(glob)
            .literal_separator(true)
            .allow_unclosed_class(true)
            .build()
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "unreadable glob pattern is invalid for {}: {err}",
                    search_root.display()
                ))
            })?;
        builder.add(glob);
    }
    let glob_set = builder.build().map_err(|err| {
        CodexErr::Fatal(format!(
            "unreadable glob matcher failed for {}: {err}",
            search_root.display()
        ))
    })?;

    let mut paths = Vec::new();
    collect_glob_files(search_root, search_root, &glob_set, max_depth, &mut paths)?;
    Ok(paths)
}

fn collect_glob_files(
    search_root: &Path,
    dir: &Path,
    glob_set: &GlobSet,
    remaining_depth: Option<usize>,
    paths: &mut Vec<AbsolutePathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let relative = path.strip_prefix(search_root).unwrap_or(path.as_path());

        if (file_type.is_file() || file_type.is_symlink()) && glob_set.is_match(relative) {
            paths.push(AbsolutePathBuf::from_absolute_path_checked(&path)?);
        }

        if !file_type.is_dir() {
            continue;
        }
        let remaining_depth = match remaining_depth {
            Some(0 | 1) => continue,
            Some(depth) => Some(depth - 1),
            None => None,
        };
        collect_glob_files(search_root, &path, glob_set, remaining_depth, paths)?;
    }
    Ok(())
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn path_depth(path: &Path) -> usize {
    path.components().count()
}

fn canonical_target_if_symlinked_path(path: &Path) -> Option<PathBuf> {
    // Return the fully resolved target only when some path component is a
    // symlink. Callers use this to bind/mask the real filesystem location while
    // leaving ordinary paths in their logical form.
    let mut current = PathBuf::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::RootDir => {
                current.push(Path::new("/"));
                continue;
            }
            Component::CurDir => continue,
            Component::ParentDir => {
                current.pop();
                continue;
            }
            Component::Normal(part) => current.push(part),
            Component::Prefix(_) => continue,
        }

        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(_) => return None,
        };
        if metadata.file_type().is_symlink() {
            let target = fs::canonicalize(path).ok()?;
            if target.as_path() == path {
                return None;
            }
            return Some(target);
        }
    }
    None
}

/// `path` with links resolved up to its nearest existing component (the
/// rest appended as written), when that differs from `path`. `None` too
/// when a link on the way lies inside a writable root (by its real
/// location): the command could change it, so the path keeps today's
/// handling.
fn real_path_through_links(path: &Path, allowed_write_paths: &[PathBuf]) -> Option<PathBuf> {
    use std::path::Component;
    let mut real = PathBuf::from("/");
    let mut components = path.components();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            if component == Component::ParentDir {
                real.pop();
            }
            continue;
        };
        let candidate = real.join(name);
        let Ok(metadata) = fs::symlink_metadata(&candidate) else {
            real = candidate;
            real.extend(components);
            break;
        };
        if !metadata.file_type().is_symlink() {
            real = candidate;
            continue;
        }
        if allowed_write_paths
            .iter()
            .any(|root| candidate.starts_with(root) && candidate != *root)
        {
            return None;
        }
        // A dangling link keeps today's handling too.
        real = fs::canonicalize(&candidate).ok()?;
    }
    (real != path).then_some(real)
}

fn remap_paths_for_symlink_target(paths: Vec<PathBuf>, root: &Path, target: &Path) -> Vec<PathBuf> {
    paths
        .into_iter()
        .map(|path| {
            if let Ok(relative) = path.strip_prefix(root) {
                target.join(relative)
            } else {
                path
            }
        })
        .collect()
}

fn normalize_command_cwd_for_bwrap(command_cwd: &Path) -> PathBuf {
    command_cwd
        .canonicalize()
        .unwrap_or_else(|_| command_cwd.to_path_buf())
}

fn append_mount_target_parent_dir_args(args: &mut Vec<String>, mount_target: &Path, anchor: &Path) {
    let mount_target_dir = if mount_target.is_dir() {
        mount_target
    } else if let Some(parent) = mount_target.parent() {
        parent
    } else {
        return;
    };
    let mut mount_target_dirs: Vec<PathBuf> = mount_target_dir
        .ancestors()
        .take_while(|path| *path != anchor)
        .map(Path::to_path_buf)
        .collect();
    mount_target_dirs.reverse();
    for mount_target_dir in mount_target_dirs {
        args.push("--dir".to_string());
        args.push(path_to_string(&mount_target_dir));
    }
}

/// Append mounts for a read-only path, after writable-symlink validation.
fn append_read_only_subpath_args(
    bwrap_args: &mut BwrapArgs,
    subpath: &Path,
    allowed_write_paths: &[PathBuf],
    daemon_directories: &BTreeSet<PathBuf>,
) -> Result<()> {
    if let Some(symlink) = first_writable_symlink_component_in_path(subpath, allowed_write_paths) {
        /*
         * A read-only carveout under a writable symlink cannot be made reliable
         * with bwrap path arguments. Binding the symlink's current target would
         * only protect a startup-time snapshot; the sandboxed process could
         * replace the writable symlink before it reads through the logical path.
         */
        return Err(CodexErr::Fatal(format!(
            "cannot enforce sandbox read-only path {} because it crosses writable symlink {}",
            subpath.display(),
            symlink.display()
        )));
    }

    if let Some(metadata) = transient_empty_metadata_path(subpath)
        && is_within_allowed_write_paths(subpath, allowed_write_paths)
    {
        // Another concurrent bwrap setup can leave an empty mount target at
        // a missing path (#562: any name, not only metadata). Treat it like
        // the missing case instead of binding that transient host path as
        // the stable source, so its owner's cleanup cannot remove it from
        // under this run.
        match metadata {
            EmptyProtectedMetadataPath::File(metadata) => {
                append_existing_empty_file_bind_data_args(bwrap_args, subpath, &metadata)?;
            }
            EmptyProtectedMetadataPath::Directory(metadata) => {
                append_existing_empty_directory_args(bwrap_args, subpath, &metadata);
            }
        }
        return Ok(());
    }

    if !subpath.exists() {
        if let Some(first_missing_component) = find_first_non_existent_component(subpath)
            && is_within_allowed_write_paths(&first_missing_component, allowed_write_paths)
            && !hold_file_above_missing_path(
                bwrap_args,
                &first_missing_component,
                allowed_write_paths,
            )?
        {
            append_missing_read_only_subpath_args(bwrap_args, &first_missing_component)?;
        }
        return Ok(());
    }

    if is_within_allowed_write_paths(subpath, allowed_write_paths) {
        bwrap_args.args.push("--ro-bind".to_string());
        bwrap_args.args.push(path_to_string(subpath));
        bwrap_args.args.push(path_to_string(subpath));
        append_daemon_socket_masks(&mut bwrap_args.args, subpath, daemon_directories)?;
    }
    Ok(())
}

fn append_empty_file_bind_data_args(bwrap_args: &mut BwrapArgs, path: &Path) -> Result<()> {
    // bubblewrap reads a `--ro-bind-data` descriptor to its end and then
    // closes it, so every mask needs its own. A shared one left the second
    // mask reading a closed descriptor ("Can't write data to file ...: Bad
    // file descriptor") unless bubblewrap happened to reuse its number (#636).
    let null = File::open("/dev/null")?;
    bwrap_args.args.push("--ro-bind-data".to_string());
    bwrap_args.args.push(null.as_raw_fd().to_string());
    bwrap_args.args.push(path_to_string(path));
    bwrap_args.preserved_files.push(null);
    Ok(())
}

fn append_empty_directory_args(bwrap_args: &mut BwrapArgs, path: &Path) {
    bwrap_args.args.push("--perms".to_string());
    bwrap_args.args.push("555".to_string());
    bwrap_args.args.push("--tmpfs".to_string());
    bwrap_args.args.push(path_to_string(path));
    bwrap_args.args.push("--remount-ro".to_string());
    bwrap_args.args.push(path_to_string(path));
}

fn append_missing_read_only_subpath_args(bwrap_args: &mut BwrapArgs, path: &Path) -> Result<()> {
    if has_synthetic_mount_target(bwrap_args, path) {
        return Ok(());
    }
    if path.file_name().is_some_and(is_protected_metadata_name) {
        append_empty_directory_args(bwrap_args, path);
        bwrap_args
            .synthetic_mount_targets
            .push(SyntheticMountTarget::missing_empty_directory(path));
        return Ok(());
    }

    append_missing_empty_file_bind_data_args(bwrap_args, path)
}

fn append_missing_empty_file_bind_data_args(bwrap_args: &mut BwrapArgs, path: &Path) -> Result<()> {
    if has_synthetic_mount_target(bwrap_args, path) {
        return Ok(());
    }
    append_empty_file_bind_data_args(bwrap_args, path)?;
    bwrap_args
        .synthetic_mount_targets
        .push(SyntheticMountTarget::missing(path));
    Ok(())
}

/// #473: several denied paths can share their first missing component
/// (`~/.config` for `~/.config/gh` and `~/.config/gcloud`). One empty
/// placeholder there already stops the whole tree being created, and
/// bubblewrap fails to mount a second one over the first. Only this
/// writable-root pass's targets are listed (see `create_filesystem_args`).
fn has_synthetic_mount_target(bwrap_args: &BwrapArgs, path: &Path) -> bool {
    bwrap_args
        .synthetic_mount_targets
        .iter()
        .any(|target| target.path() == path)
}

/// #473: the first missing component of `path` when the folder it would be
/// created in sits at or below an existing read-only path. That folder is
/// bound read-only, so the command cannot create it, and neither can
/// bubblewrap create a mount target there. `None` when `path` exists, is not
/// below a writable root, or crosses a writable symlink (masking it then
/// fails closed as before).
fn missing_component_below_read_only_path(
    path: &Path,
    read_only_subpaths: &[PathBuf],
    allowed_write_paths: &[PathBuf],
) -> Option<PathBuf> {
    if first_writable_symlink_component_in_path(path, allowed_write_paths).is_some() {
        return None;
    }
    let missing = find_first_non_existent_component(path)?;
    if !is_within_allowed_write_paths(&missing, allowed_write_paths) {
        return None;
    }
    let parent = missing.parent()?;
    let parent_is_dir = fs::symlink_metadata(parent).is_ok_and(|metadata| metadata.is_dir());
    (parent_is_dir
        && read_only_subpaths
            .iter()
            .any(|read_only| parent.starts_with(read_only)))
    .then_some(missing)
}

/// #473: hides denied paths that are missing in `dir`, a read-only folder,
/// for the whole run. A placeholder cannot be created there, and a live
/// read-only bind would still show a credential file the host writes while
/// the command runs. So `dir` becomes a read-only tmpfs that holds what it
/// has at launch (folders and files bound read-only, links recreated),
/// without `hidden`: nothing created after launch appears inside. Entries
/// that vanish before bubblewrap runs are skipped; names that are not UTF-8
/// are left out.
fn append_read_only_snapshot_dir_args(
    bwrap_args: &mut BwrapArgs,
    dir: &Path,
    hidden: &[PathBuf],
    perms: &str,
) -> Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    bwrap_args.args.push("--perms".to_string());
    bwrap_args.args.push(perms.to_string());
    bwrap_args.args.push("--tmpfs".to_string());
    bwrap_args.args.push(path_to_string(dir));
    for entry in entries {
        let path = entry.path();
        if hidden.contains(&path) || path.to_str().is_none() {
            continue;
        }
        if entry
            .file_type()
            .is_ok_and(|file_type| file_type.is_symlink())
        {
            let Ok(target) = fs::read_link(&path) else {
                continue;
            };
            let Some(target) = target.to_str() else {
                continue;
            };
            bwrap_args.args.push("--symlink".to_string());
            bwrap_args.args.push(target.to_string());
        } else {
            bwrap_args.args.push("--ro-bind-try".to_string());
            bwrap_args.args.push(path_to_string(&path));
        }
        bwrap_args.args.push(path_to_string(&path));
    }
    bwrap_args.args.push("--remount-ro".to_string());
    bwrap_args.args.push(path_to_string(dir));
    Ok(())
}

fn append_existing_empty_file_bind_data_args(
    bwrap_args: &mut BwrapArgs,
    path: &Path,
    metadata: &Metadata,
) -> Result<()> {
    append_empty_file_bind_data_args(bwrap_args, path)?;
    bwrap_args
        .synthetic_mount_targets
        .push(SyntheticMountTarget::existing_empty_file(path, metadata));
    Ok(())
}

fn append_existing_empty_directory_args(
    bwrap_args: &mut BwrapArgs,
    path: &Path,
    metadata: &Metadata,
) {
    append_empty_directory_args(bwrap_args, path);
    bwrap_args
        .synthetic_mount_targets
        .push(SyntheticMountTarget::existing_empty_directory(
            path, metadata,
        ));
}

fn append_unreadable_root_args(
    bwrap_args: &mut BwrapArgs,
    unreadable_root: &Path,
    allowed_write_paths: &[PathBuf],
) -> Result<()> {
    if let Some(symlink) =
        first_writable_symlink_component_in_path(unreadable_root, allowed_write_paths)
    {
        /*
         * Deny-read masks must fail closed when the protected path crosses a
         * symlink that remains writable to the sandboxed process. Resolving and
         * masking the symlink's current target is a TOCTTOU snapshot: bwrap would
         * protect the old target while the logical path could later point
         * somewhere else.
         */
        return Err(CodexErr::Fatal(format!(
            "cannot enforce sandbox deny-read path {} because it crosses writable symlink {}",
            unreadable_root.display(),
            symlink.display()
        )));
    }

    if !unreadable_root.exists() {
        if let Some(first_missing_component) = find_first_non_existent_component(unreadable_root)
            && is_within_allowed_write_paths(&first_missing_component, allowed_write_paths)
            && !hold_file_above_missing_path(
                bwrap_args,
                &first_missing_component,
                allowed_write_paths,
            )?
        {
            append_missing_empty_file_bind_data_args(bwrap_args, &first_missing_component)?;
        }
        return Ok(());
    }

    // #562: another live run's placeholder (made in that run's writable
    // root, which need not be one of this run's): mask it as before, and
    // register it so that run's cleanup cannot remove it from under this one.
    if let Some(metadata) = live_placeholder_file(unreadable_root) {
        if has_synthetic_mount_target(bwrap_args, unreadable_root) {
            return Ok(());
        }
        bwrap_args.args.push("--perms".to_string());
        bwrap_args.args.push("000".to_string());
        return append_existing_empty_file_bind_data_args(bwrap_args, unreadable_root, &metadata);
    }
    // #607: the same for another live run's placeholder folder (a protected
    // metadata name it holds read-only): masked as a denied folder and
    // registered, so its maker's cleanup (or a sweep after that run was
    // killed) cannot remove it, and the mask with it, from under this run.
    if let Some(metadata) = live_placeholder_directory(unreadable_root) {
        append_existing_unreadable_path_args(bwrap_args, unreadable_root, allowed_write_paths)?;
        if !has_synthetic_mount_target(bwrap_args, unreadable_root) {
            bwrap_args.synthetic_mount_targets.push(
                SyntheticMountTarget::existing_empty_directory(unreadable_root, &metadata),
            );
        }
        return Ok(());
    }

    append_existing_unreadable_path_args(bwrap_args, unreadable_root, allowed_write_paths)
}

/// #562: the folder `missing` would go in is a regular file, so `missing`
/// needs no placeholder of its own (bubblewrap would fail the run: `Can't
/// mkdir parents … Not a directory`). The file stays in place instead, so the
/// command cannot remove it and create the path: another live run's
/// placeholder (for a path below it, such as `~/Library` for
/// `~/Library/Keychains`) is held as one (masked and registered, so that
/// run's cleanup cannot remove it from under this one); any other file in a
/// writable root is bound read-only where it is (a mount point cannot be
/// removed or renamed). #745: a folder there that is another run's empty
/// placeholder is held too ([`hold_placeholder_folder_above_missing_path`]).
/// Returns whether `missing` needs nothing more.
fn hold_file_above_missing_path(
    bwrap_args: &mut BwrapArgs,
    missing: &Path,
    allowed_write_paths: &[PathBuf],
) -> Result<bool> {
    let Some(parent) = missing.parent() else {
        return Ok(false);
    };
    let Ok(metadata) = fs::symlink_metadata(parent) else {
        return Ok(false);
    };
    if metadata.file_type().is_dir() {
        return Ok(hold_placeholder_folder_above_missing_path(
            bwrap_args,
            parent,
            &metadata,
            allowed_write_paths,
        ));
    }
    if !metadata.file_type().is_file() {
        return Ok(false);
    }
    if has_synthetic_mount_target(bwrap_args, parent) || has_read_only_bind(bwrap_args, parent) {
        return Ok(true);
    }
    if let Some(metadata) = live_placeholder_file(parent) {
        append_existing_empty_file_bind_data_args(bwrap_args, parent, &metadata)?;
    } else if is_within_allowed_write_paths(parent, allowed_write_paths) {
        bwrap_args.args.push("--ro-bind".to_string());
        bwrap_args.args.push(path_to_string(parent));
        bwrap_args.args.push(path_to_string(parent));
    }
    Ok(true)
}

/// #745: the folder `missing` would go in (`folder`) is another run's empty
/// placeholder folder (a protected `.git` it made, which this run's policy
/// took for a repository's). A placeholder made in it would change it, so
/// that its maker could not remove it, and it was left behind for good. It
/// is held read-only instead, as when it is a protected path itself: the
/// command cannot create `missing` (or anything) in it, as it could not
/// create the folder without the other run. Whether `missing` needs nothing
/// more.
fn hold_placeholder_folder_above_missing_path(
    bwrap_args: &mut BwrapArgs,
    folder: &Path,
    metadata: &Metadata,
    allowed_write_paths: &[PathBuf],
) -> bool {
    // Held already in this pass: its read-only mount covers `missing`.
    if has_synthetic_mount_target(bwrap_args, folder) {
        return true;
    }
    if !is_within_allowed_write_paths(folder, allowed_write_paths)
        || allowed_write_paths.iter().any(|root| root == folder)
        || !directory_is_empty(folder)
        || !(crate::linux_run_main::is_live_placeholder(folder)
            || crate::placeholder_journal::recorded_by_another_run(folder))
    {
        return false;
    }
    append_existing_empty_directory_args(bwrap_args, folder, metadata);
    true
}

fn has_read_only_bind(bwrap_args: &BwrapArgs, path: &Path) -> bool {
    let path = path_to_string(path);
    bwrap_args
        .args
        .windows(3)
        .any(|window| window[0] == "--ro-bind" && window[1] == path && window[2] == path)
}

/// #562: `path` when it is an empty file another live run registered as its
/// placeholder.
fn live_placeholder_file(path: &Path) -> Option<Metadata> {
    let metadata = empty_file_metadata(path)?;
    crate::linux_run_main::is_live_placeholder(path).then_some(metadata)
}

/// #607: `path` when it is an empty folder another live run registered as
/// its placeholder.
fn live_placeholder_directory(path: &Path) -> Option<Metadata> {
    let metadata = fs::symlink_metadata(path).ok()?;
    (metadata.file_type().is_dir() && directory_is_empty(path))
        .then_some(metadata)
        .filter(|_| crate::linux_run_main::is_live_placeholder(path))
}

fn empty_file_metadata(path: &Path) -> Option<Metadata> {
    let metadata = fs::symlink_metadata(path).ok()?;
    (metadata.file_type().is_file() && metadata.len() == 0).then_some(metadata)
}

fn append_existing_unreadable_path_args(
    bwrap_args: &mut BwrapArgs,
    unreadable_root: &Path,
    allowed_write_paths: &[PathBuf],
) -> Result<()> {
    if unreadable_root.is_dir() {
        let mut writable_descendants: Vec<&Path> = allowed_write_paths
            .iter()
            .map(PathBuf::as_path)
            .filter(|path| *path != unreadable_root && path.starts_with(unreadable_root))
            .collect();
        bwrap_args.args.push("--perms".to_string());
        // Execute-only perms let the process traverse into explicitly
        // re-opened writable descendants while still hiding the denied
        // directory contents. Plain denied directories with no writable child
        // mounts stay at `000`.
        bwrap_args.args.push(if writable_descendants.is_empty() {
            "000".to_string()
        } else {
            "111".to_string()
        });
        bwrap_args.args.push("--tmpfs".to_string());
        bwrap_args.args.push(path_to_string(unreadable_root));
        // Recreate any writable descendants inside the tmpfs before remounting
        // the denied parent read-only. Otherwise bubblewrap cannot mkdir the
        // nested mount targets after the parent has been frozen.
        writable_descendants.sort_by_key(|path| path_depth(path));
        for writable_descendant in writable_descendants {
            append_mount_target_parent_dir_args(
                &mut bwrap_args.args,
                writable_descendant,
                unreadable_root,
            );
        }
        bwrap_args.args.push("--remount-ro".to_string());
        bwrap_args.args.push(path_to_string(unreadable_root));
        return Ok(());
    }

    bwrap_args.args.push("--perms".to_string());
    bwrap_args.args.push("000".to_string());
    append_empty_file_bind_data_args(bwrap_args, unreadable_root)
}

/// Returns true when `path` is under any allowed writable root.
fn is_within_allowed_write_paths(path: &Path, allowed_write_paths: &[PathBuf]) -> bool {
    allowed_write_paths
        .iter()
        .any(|root| path.starts_with(root))
}

enum EmptyProtectedMetadataPath {
    File(Metadata),
    Directory(Metadata),
}

fn transient_empty_metadata_path(path: &Path) -> Option<EmptyProtectedMetadataPath> {
    if !path.file_name().is_some_and(is_protected_metadata_name) {
        // #562: another live run's placeholder, whatever its name.
        return live_placeholder_file(path).map(EmptyProtectedMetadataPath::File);
    }

    let metadata = fs::symlink_metadata(path).ok()?;
    if metadata.file_type().is_file() && metadata.len() == 0 {
        return Some(EmptyProtectedMetadataPath::File(metadata));
    }

    if metadata.file_type().is_dir() && directory_is_empty(path) {
        return Some(EmptyProtectedMetadataPath::Directory(metadata));
    }

    None
}

fn directory_is_empty(path: &Path) -> bool {
    let Ok(mut entries) = fs::read_dir(path) else {
        return false;
    };
    entries.next().is_none()
}

fn first_writable_symlink_component_in_path(
    target_path: &Path,
    allowed_write_paths: &[PathBuf],
) -> Option<PathBuf> {
    /*
     * Walk the logical path and report the first symlink component that lives
     * under a writable root. These symlinks are mutable from inside the sandbox,
     * so any mount or mask based on their resolved target would be racing a path
     * the sandboxed process can change.
     */
    let mut current = PathBuf::new();

    for component in target_path.components() {
        use std::path::Component;
        match component {
            Component::RootDir => {
                current.push(Path::new("/"));
                continue;
            }
            Component::CurDir => continue,
            Component::ParentDir => {
                current.pop();
                continue;
            }
            Component::Normal(part) => current.push(part),
            Component::Prefix(_) => continue,
        }

        let metadata = match std::fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(_) => break,
        };

        if metadata.file_type().is_symlink()
            && is_within_allowed_write_paths(&current, allowed_write_paths)
        {
            return Some(current);
        }
    }

    None
}

/// Find the first missing path component while walking `target_path`.
///
/// Mounting `/dev/null` on the first missing component prevents the sandboxed
/// process from creating the protected path hierarchy.
fn find_first_non_existent_component(target_path: &Path) -> Option<PathBuf> {
    let mut current = PathBuf::new();

    for component in target_path.components() {
        use std::path::Component;
        match component {
            Component::RootDir => {
                current.push(Path::new("/"));
                continue;
            }
            Component::CurDir => continue,
            Component::ParentDir => {
                current.pop();
                continue;
            }
            Component::Normal(part) => current.push(part),
            Component::Prefix(_) => continue,
        }

        if !current.exists() {
            return Some(current);
        }
    }

    None
}

/// The `rg` on `PATH`, for tests of the ripgrep engine (#732).
#[cfg(test)]
fn path_ripgrep_files(
    search_root: &Path,
    globs: &[String],
    max_depth: Option<usize>,
) -> Result<Vec<AbsolutePathBuf>> {
    ripgrep_files(
        Path::new(GLOB_SCAN_PROGRAM),
        search_root,
        globs,
        max_depth,
        &|_| {},
    )
}

#[cfg(test)]
#[path = "bwrap_wslg_tests.rs"]
mod wslg_tests;

#[cfg(test)]
#[path = "bwrap_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "database_glob_tests.rs"]
mod database_glob_tests;
