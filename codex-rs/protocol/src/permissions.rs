use std::borrow::Cow;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_absolute_path::canonicalize_preserving_symlinks;
use codex_utils_path_uri::LegacyAppPathString;
use codex_utils_path_uri::PathConvention;
use codex_utils_path_uri::PathUri;
use globset::Candidate;
use globset::GlobBuilder;
use globset::GlobMatcher;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::Display;
use tracing::error;
use ts_rs::TS;

use crate::protocol::NetworkAccess;
use crate::protocol::SandboxPolicy;
use crate::protocol::WritableRoot;

<<<<<<< HEAD
mod glob_scan;
pub use glob_scan::FileSystemDenyGlob;
pub use glob_scan::FileSystemGlobScan;

mod effective_reads;
pub use effective_reads::EffectiveReads;
mod effective_writes;
pub use effective_writes::EffectiveWrites;

#[cfg(test)]
#[path = "permissions/effective_reads_tests.rs"]
mod effective_reads_tests;
||||||| 413492cd6c
=======
mod deny_read_validator;
mod local_aliases;
mod target;
mod windows_glob;

use local_aliases::LocalPolicyContext;

pub use deny_read_validator::DenyReadValidator;
pub use deny_read_validator::DenyReadViolation;
pub use local_aliases::LocalFileSystemPolicyMatcher;
pub use windows_glob::WindowsDenyReadGlobScan;
pub use windows_glob::windows_deny_read_glob_scan;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a

const PROTECTED_METADATA_GIT_PATH_NAME: &str = ".git";
const PROTECTED_METADATA_AGENTS_PATH_NAME: &str = ".agents";
const PROTECTED_METADATA_CODEX_PATH_NAME: &str = ".codex";
const PROTECTED_METADATA_AWS_PATH_NAME: &str = ".aws";

/// Top-level workspace metadata paths that stay protected under writable roots.
pub const PROTECTED_METADATA_PATH_NAMES: &[&str] = &[
    PROTECTED_METADATA_GIT_PATH_NAME,
    PROTECTED_METADATA_AGENTS_PATH_NAME,
    PROTECTED_METADATA_CODEX_PATH_NAME,
    PROTECTED_METADATA_AWS_PATH_NAME,
];

/// Returns true when a path basename is one of the protected workspace metadata names.
pub fn is_protected_metadata_name(name: &OsStr) -> bool {
    PROTECTED_METADATA_PATH_NAMES
        .iter()
        .any(|metadata_name| name == OsStr::new(metadata_name))
}

/// #442 (Travis, 2026-10-11): credential file names a protected level denies
/// by name, so a file created after the session started is covered too. Each
/// pattern is one path component in the filesystem policy's glob syntax.
pub const CREDENTIAL_FILE_NAME_PATTERNS: &[&str] = &[".env", ".env.*", ".envrc"];

/// #442 (Travis, 2026-10-12): the template names a project keeps beside its
/// `.env` (they hold no credentials, and checkouts and installs write them).
/// [`CREDENTIAL_FILE_NAME_PATTERNS`] never covers them, in any case.
pub const CREDENTIAL_FILE_TEMPLATE_NAMES: &[&str] =
    &[".env.example", ".env.sample", ".env.template"];

/// The credential names as people read them: ".env, .env.* (except
/// .env.example, .env.sample and .env.template) and .envrc".
pub fn credential_file_names_shown() -> String {
    format!(
        ".env, .env.* (except {}) and .envrc",
        match CREDENTIAL_FILE_TEMPLATE_NAMES {
            [first, second, last] => format!("{first}, {second} and {last}"),
            names => names.join(", "),
        }
    )
}

/// Whether `name` is one of [`CREDENTIAL_FILE_TEMPLATE_NAMES`], in any case.
pub fn is_credential_template_name(name: &OsStr) -> bool {
    let name = name.as_encoded_bytes().to_ascii_lowercase();
    CREDENTIAL_FILE_TEMPLATE_NAMES
        .iter()
        .any(|template| name == template.as_bytes())
}

/// Whether `name` matches one of [`CREDENTIAL_FILE_NAME_PATTERNS`], in any
/// case (on a case-insensitive filesystem `.ENV` is the same file as `.env`),
/// and is not a template ([`CREDENTIAL_FILE_TEMPLATE_NAMES`]).
pub fn is_credential_file_name(name: &OsStr) -> bool {
    let lower = name.as_encoded_bytes().to_ascii_lowercase();
    (lower == b".env" || lower == b".envrc" || lower.starts_with(b".env."))
        && !is_credential_template_name(name)
}

/// Why an existing credential-named entry (not a folder) at `path` is unsafe:
/// whatever the person writes to it would land where agent commands could
/// read it. Only a plain file with no other hard links is safe; any link is
/// refused, since it can be re-pointed (or a folder along it swapped) after
/// it was checked.
pub fn credential_entry_problem(metadata: &std::fs::Metadata) -> Option<&'static str> {
    let kind = metadata.file_type();
    if kind.is_symlink() {
        return Some("is a link");
    }
    if !kind.is_file() {
        return Some("is not a regular file (a pipe, socket or device)");
    }
    (hard_links(metadata) > 1).then_some(HARD_LINKED)
}

/// #442 (Travis, 2026-10-12): why the existing credential entry at `path`
/// cannot be protected, naming it, and exactly how to fix it; `None` when it
/// is a plain file ([`credential_entry_problem`]). Shown when Aggressive
/// starts and, on Linux, for each refused command.
pub fn unsafe_credential_entry_message(
    path: &Path,
    metadata: &std::fs::Metadata,
) -> Option<String> {
    let problem = credential_entry_problem(metadata)?;
    let shown = path.display();
    let away = "or move it out of the workspace";
    Some(if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(path).ok().map(|target| {
            lexically_normal(
                &path
                    .parent()
                    .map_or(target.clone(), |parent| parent.join(&target)),
            )
        });
        let (to, exposed) = match &target {
            Some(target) => (
                format!(" to {}", target.display()),
                format!(
                    " What was written to it is in {}: unless that is a credential file too, \
                     agent commands may have read it, so delete that file too (moving it does not \
                     hide it) and change any secret in it.",
                    target.display()
                ),
            ),
            None => (String::new(), String::new()),
        };
        format!(
            "the credential file {shown} is a link{to}. A credential file must be a regular file, \
             or what is written to it lands where agent commands could read it under another \
             name. To fix it, replace the link with a regular file (delete the link, then create \
             the file in its place), {away}.{exposed}"
        )
    } else if problem == HARD_LINKED {
        format!(
            "the credential file {shown} has other hard links: the same file under other names \
             agent commands could read. To fix it, delete the other names (`find` with \
             `-samefile` lists them) or replace the file with a regular copy, {away}; any secret \
             in it may have been read, so change it."
        )
    } else {
        format!(
            "the credential file {shown} {problem}. To fix it, replace it with a regular file, \
             {away}."
        )
    })
}

/// `path` with `.` and `name/..` removed, without touching the filesystem.
fn lexically_normal(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
                if matches!(
                    normal.components().next_back(),
                    Some(std::path::Component::Normal(_))
                ) =>
            {
                normal.pop();
            }
            other => normal.push(other),
        }
    }
    normal
}

const HARD_LINKED: &str = "has other hard links";

#[cfg(unix)]
fn hard_links(metadata: &std::fs::Metadata) -> u64 {
    std::os::unix::fs::MetadataExt::nlink(metadata)
}

#[cfg(not(unix))]
fn hard_links(_metadata: &std::fs::Metadata) -> u64 {
    1
}

/// The deny globs that cover every credential file name below `root`, now
/// and created later, spelled as the filesystem policy resolves them (with
/// `\\` on Windows).
pub fn credential_file_globs(root: &str) -> Vec<String> {
    let separator = std::path::MAIN_SEPARATOR;
    let root = root.trim_end_matches(['/', separator]);
    CREDENTIAL_FILE_NAME_PATTERNS
        .iter()
        .map(|name| format!("{root}{separator}**{separator}{name}"))
        .collect()
}

/// `/`, and on Windows also `\\` (policy globs are resolved with it there).
fn is_glob_separator(ch: char) -> bool {
    ch == '/' || (cfg!(windows) && ch == '\\')
}

/// The folder and credential name pattern of a glob from
/// [`credential_file_globs`], with either separator.
fn credential_glob_parts(pattern: &str) -> Option<(PathBuf, &'static str)> {
    CREDENTIAL_FILE_NAME_PATTERNS.iter().find_map(|name| {
        let rest = pattern.strip_suffix(name)?;
        let rest = rest.strip_suffix(is_glob_separator)?.strip_suffix("**")?;
        let root = rest.strip_suffix(is_glob_separator)?;
        Some((
            PathBuf::from(if root.is_empty() { "/" } else { root }),
            *name,
        ))
    })
}

/// #442 (Travis, 2026-10-12): the folders whose credential names `globs`
/// deny with the whole rule, every pattern of [`credential_file_globs`] for
/// the folder, as the Aggressive level writes it. Only there are the
/// templates ([`CREDENTIAL_FILE_TEMPLATE_NAMES`]) exempt: a person's own
/// `**/.env.*` glob alone keeps covering them.
pub fn credential_rule_roots<S: AsRef<str>>(globs: &[S]) -> Vec<PathBuf> {
    let parts: Vec<(PathBuf, &str)> = globs
        .iter()
        .filter_map(|glob| credential_glob_parts(glob.as_ref()))
        .collect();
    let mut roots: Vec<PathBuf> = parts.iter().map(|(root, _)| root.clone()).collect();
    roots.sort();
    roots.dedup();
    roots.retain(|root| {
        CREDENTIAL_FILE_NAME_PATTERNS.iter().all(|wanted| {
            parts
                .iter()
                .any(|(other, name)| other == root && name == wanted)
        })
    });
    roots
}

/// Whether the deny glob `pattern` leaves the templates out: a credential
/// glob of a folder in `rule_roots` ([`credential_rule_roots`]).
pub fn credential_glob_exempts_templates(pattern: &str, rule_roots: &[PathBuf]) -> bool {
    credential_file_glob_root(pattern).is_some_and(|root| rule_roots.contains(&root))
}

/// The folder a deny glob from [`credential_file_globs`] covers, so a sandbox
/// that can only hide existing files can also watch for new ones.
pub fn credential_file_glob_root(pattern: &str) -> Option<PathBuf> {
    credential_glob_parts(pattern).map(|(root, _)| root)
}

/// Returns the protected workspace metadata name when an agent write to `path`
/// should be blocked before execution.
pub fn forbidden_agent_metadata_write(
    path: &Path,
    cwd: &Path,
    file_system_sandbox_policy: &FileSystemSandboxPolicy,
) -> Option<&'static str> {
    if !matches!(
        file_system_sandbox_policy.kind,
        FileSystemSandboxKind::Restricted
    ) {
        return None;
    }

    with_local_policy_context(path, cwd, |path, context| {
        file_system_sandbox_policy
            .metadata_write_denial(path, context)
            .filter(|_| !file_system_sandbox_policy.can_write_path(path, context))
    })?
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, Default, JsonSchema, TS,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NetworkSandboxPolicy {
    #[default]
    Restricted,
    Enabled,
}

impl NetworkSandboxPolicy {
    pub fn is_enabled(self) -> bool {
        matches!(self, NetworkSandboxPolicy::Enabled)
    }
}

/// Access mode for a filesystem entry.
///
/// When two equally specific entries target the same path, we compare these by
/// conflict precedence rather than by capability breadth: `deny` beats
/// `write`, and `write` beats `read`.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Display,
    JsonSchema,
    TS,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FileSystemAccessMode {
    Read,
    Write,
    /// `none` is a legacy input alias retained temporarily for compatibility.
    #[serde(alias = "none")]
    Deny,
}

impl FileSystemAccessMode {
    pub fn can_read(self) -> bool {
        !matches!(self, FileSystemAccessMode::Deny)
    }

    pub fn can_write(self) -> bool {
        matches!(self, FileSystemAccessMode::Write)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(tag = "kind")]
pub enum FileSystemSpecialPath {
    Root,
    Minimal,
    #[serde(alias = "current_working_directory")]
    ProjectRoots {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        subpath: Option<String>,
    },
    Tmpdir,
    SlashTmp,
    /// WARNING: `:special_path` tokens are part of config compatibility.
    /// Do not make older runtimes reject newly introduced tokens.
    /// New parser support should be additive, while unknown values must stay
    /// representable so config from a newer Codex degrades to warn-and-ignore
    /// instead of failing to load. Codex 0.112.0 rejected unknown values here,
    /// which broke forward compatibility for newer config.
    /// Preserves future special-path tokens so older runtimes can ignore them
    /// without rejecting config authored by a newer release.
    Unknown {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        subpath: Option<String>,
    },
}

impl FileSystemSpecialPath {
    pub fn project_roots(subpath: Option<String>) -> Self {
        Self::ProjectRoots { subpath }
    }

    pub fn unknown(path: impl Into<String>, subpath: Option<String>) -> Self {
        Self::Unknown {
            path: path.into(),
            subpath,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileSystemSandboxEntry {
    pub path: FileSystemPath,
    pub access: FileSystemAccessMode,
    pub missing_path_behavior: Option<FileSystemSandboxEntryMissingPathBehavior>,
}

/// Serialized filesystem entry used at legacy string-based seams.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[schemars(rename = "FileSystemSandboxEntry")]
#[ts(rename = "FileSystemSandboxEntry")]
pub struct RawFileSystemSandboxEntry {
    pub path: RawFileSystemPath,
    pub access: FileSystemAccessMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub missing_path_behavior: Option<FileSystemSandboxEntryMissingPathBehavior>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum FileSystemSandboxEntryMissingPathBehavior {
    Skip,
}

impl FileSystemSandboxEntry {
    pub fn new(path: FileSystemPath, access: FileSystemAccessMode) -> Self {
        Self {
            path,
            access,
            missing_path_behavior: None,
        }
    }

    pub fn skip_missing_path(path: FileSystemPath, access: FileSystemAccessMode) -> Self {
        Self {
            path,
            access,
            missing_path_behavior: Some(FileSystemSandboxEntryMissingPathBehavior::Skip),
        }
    }

    pub fn skips_missing_path(&self) -> bool {
        self.missing_path_behavior == Some(FileSystemSandboxEntryMissingPathBehavior::Skip)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, Default, JsonSchema, TS,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum FileSystemSandboxKind {
    #[default]
    Restricted,
    Unrestricted,
    ExternalSandbox,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemSandboxPolicy {
    pub kind: FileSystemSandboxKind,
    pub glob_scan_max_depth: Option<usize>,
    pub entries: Vec<FileSystemSandboxEntry>,
}

#[derive(Clone, Copy)]
enum WritableRootPathResolution {
    Effective,
    PreserveMutableComponents,
}

#[derive(Clone, Copy)]
enum RootMetadataWriteMounts {
    Separate,
    InheritWritableRoot,
}

impl WritableRootPathResolution {
    fn resolve(self, path: AbsolutePathBuf) -> AbsolutePathBuf {
        match self {
            Self::Effective => normalize_effective_absolute_path(path),
            Self::PreserveMutableComponents => path.normalize_system_aliases().unwrap_or(path),
        }
    }
}

/// Serialized filesystem policy used at legacy string-based seams.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[schemars(rename = "FileSystemSandboxPolicy")]
#[ts(rename = "FileSystemSandboxPolicy")]
pub struct RawFileSystemSandboxPolicy {
    pub kind: FileSystemSandboxKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub glob_scan_max_depth: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<RawFileSystemSandboxEntry>,
}

/// Executor-owned paths needed to interpret filesystem sandbox policy entries.
///
/// Orchestrator callers keep these as `PathUri` values until execution crosses
/// into the executor that owns them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSystemSandboxPolicyContext<'a> {
    pub cwd: &'a PathUri,
    pub workspace_roots: &'a [PathUri],
    pub user_home_dir: Option<&'a PathUri>,
    pub temporary_directories: Option<&'a [PathUri]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedFileSystemEntry {
    path: AbsolutePathBuf,
    access: FileSystemAccessMode,
}

struct PreparedFileSystemEntry<'a> {
    entry: &'a ResolvedFileSystemEntry,
    effective_path: AbsolutePathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSystemSemanticSignature {
    has_full_disk_read_access: bool,
    has_full_disk_write_access: bool,
    include_platform_defaults: bool,
    readable_roots: Vec<AbsolutePathBuf>,
    writable_roots: Vec<WritableRoot>,
    unreadable_roots: Vec<AbsolutePathBuf>,
    unreadable_globs: Vec<FileSystemDenyGlob>,
    unlistable_directories: Vec<AbsolutePathBuf>,
    unmovable_paths: Vec<AbsolutePathBuf>,
    keychain_services_denied: bool,
    user_namespaces_denied: bool,
}

/// #652: the writable roots of a policy, each as written, normalized and at
/// its real location, where Seatbelt and bubblewrap grant it (a root reached
/// through a nested link, `~/ws -> /Volumes/x/ws`, at its target).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WritableRootLinks {
    roots: Vec<(AbsolutePathBuf, AbsolutePathBuf, PathBuf)>,
}

impl WritableRootLinks {
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// `path` spelled through each root that holds its real location while
    /// `path` is not below the root as written or normalized (a root reached
    /// through a nested link, or `path` reached through one). The sandboxes
    /// grant such a root at its real location, so `path` is writable
    /// through it, and a read-only entry for `path` must be spelled through
    /// the root for them to honor it.
    pub fn spellings(&self, path: &Path, cwd: &Path) -> Vec<AbsolutePathBuf> {
        #[cfg(unix)]
        {
            if self.roots.is_empty() {
                return Vec::new();
            }
            let Some(path) = resolve_candidate_path(path, cwd) else {
                return Vec::new();
            };
            let real = real_location(path.as_path());
            let normalized = normalize_effective_absolute_path(path.clone());
            let mut spellings: Vec<AbsolutePathBuf> = Vec::new();
            for (root, root_normalized, root_real) in &self.roots {
                if path.as_path().starts_with(root.as_path())
                    || normalized.as_path().starts_with(root_normalized.as_path())
                {
                    continue;
                }
                let Ok(suffix) = real.strip_prefix(root_real) else {
                    continue;
                };
                let spelled = root.join(suffix);
                if !spellings.contains(&spelled) {
                    spellings.push(spelled);
                }
            }
            spellings
        }
        #[cfg(not(unix))]
        {
            let _ = (path, cwd);
            Vec::new()
        }
    }
}

/// Runtime matcher for read-deny entries in a filesystem sandbox policy.
pub struct ReadDenyMatcher {
<<<<<<< HEAD
    denied_candidates: Vec<Vec<PathBuf>>,
    /// Each glob, and whether its matches leave out the template names
    /// ([`credential_glob_exempts_templates`]).
    deny_read_matchers: Vec<(GlobMatcher, bool)>,
||||||| 413492cd6c
    denied_candidates: Vec<Vec<PathBuf>>,
    deny_read_matchers: Vec<GlobMatcher>,
=======
    native_cwd: Option<AbsolutePathBuf>,
    user_home_dir: Option<PathUri>,
    temporary_directories: Vec<PathUri>,
    prepared: PreparedReadDenyMatcher,
}

/// Prepared PathUri deny roots and globs for repeated executor-owned read checks.
struct PreparedReadDenyMatcher {
    denied_roots: Vec<PathUri>,
    deny_read_matchers: Vec<GlobMatcher>,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    invalid_pattern: bool,
    /// Directories that may not be listed (issue #506), by every spelling.
    unlistable_candidates: Vec<Vec<PathBuf>>,
}

impl ReadDenyMatcher {
    /// Builds a matcher for executor-owned URI paths without host projection.
    pub fn from_context(
        file_system_sandbox_policy: &FileSystemSandboxPolicy,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> Option<Self> {
        file_system_sandbox_policy
            .entries
            .iter()
            .any(|entry| entry.access == FileSystemAccessMode::Deny)
            .then(|| {
                file_system_sandbox_policy
                    .prepare_deny_read_matcher(context, InvalidDenyReadGlobBehavior::FailClosed)
                    .map(|prepared| Self {
                        native_cwd: None,
                        user_home_dir: None,
                        temporary_directories: Vec::new(),
                        prepared,
                    })
                    .ok()
            })
            .flatten()
    }

    /// Builds a local-path matcher for callers that must reject malformed glob patterns.
    ///
    /// Runtime read checks intentionally fail closed on malformed deny patterns.
    /// Host-side expansion work should use this constructor instead so a typo
    /// cannot broaden the set of paths it mutates before execution starts.
    pub fn try_new_for_local_paths(
        file_system_sandbox_policy: &FileSystemSandboxPolicy,
        cwd: &Path,
    ) -> Result<Option<Self>, String> {
        Self::build(
            file_system_sandbox_policy,
            cwd,
            InvalidDenyReadGlobBehavior::ReturnError,
        )
    }

    fn build(
        file_system_sandbox_policy: &FileSystemSandboxPolicy,
        cwd: &Path,
        invalid_glob_behavior: InvalidDenyReadGlobBehavior,
    ) -> Result<Option<Self>, String> {
        if !file_system_sandbox_policy.has_denied_read_restrictions() {
            return Ok(None);
        }
<<<<<<< HEAD

        // Exact roots are stored as all meaningful path spellings we can derive
        // cheaply. This lets direct tool checks catch both a symlink path and
        // its canonical target without changing the policy entries themselves.
        let denied_candidates = file_system_sandbox_policy
            .get_unreadable_roots_with_cwd(cwd)
            .into_iter()
            .map(|path| normalized_and_canonical_candidates(path.as_path()))
            .collect();
        // Pattern entries stay as policy-level globs. They are matched at read
        // time here instead of being snapshotted to startup filesystem state.
        let mut invalid_pattern = false;
        let mut deny_read_matchers = Vec::new();
        let globs = file_system_sandbox_policy.get_unreadable_globs_with_cwd(cwd);
        let rule_roots = credential_rule_roots(&globs);
        for pattern in globs {
            // Credential globs match in any letter case (`.ENV` on a
            // case-insensitive filesystem is `.env`); this also covers their
            // folder part, which only ever denies more.
            let case_insensitive = credential_file_glob_root(&pattern).is_some();
            let exempts_templates = credential_glob_exempts_templates(&pattern, &rule_roots);
            match build_glob_matcher(&pattern, case_insensitive) {
                Ok(matcher) => deny_read_matchers.push((matcher, exempts_templates)),
                Err(err) => match invalid_glob_behavior {
                    InvalidDenyReadGlobBehavior::FailClosed => invalid_pattern = true,
                    InvalidDenyReadGlobBehavior::ReturnError => {
                        return Err(format!("invalid deny-read glob pattern `{pattern}`: {err}"));
                    }
                },
            }
        }
        let unlistable_candidates = file_system_sandbox_policy
            .get_unlistable_directories()
            .into_iter()
            .map(|path| normalized_and_canonical_candidates(path.as_path()))
            .collect();
||||||| 413492cd6c

        // Exact roots are stored as all meaningful path spellings we can derive
        // cheaply. This lets direct tool checks catch both a symlink path and
        // its canonical target without changing the policy entries themselves.
        let denied_candidates = file_system_sandbox_policy
            .get_unreadable_roots_with_cwd(cwd)
            .into_iter()
            .map(|path| normalized_and_canonical_candidates(path.as_path()))
            .collect();
        // Pattern entries stay as policy-level globs. They are matched at read
        // time here instead of being snapshotted to startup filesystem state.
        let mut invalid_pattern = false;
        let mut deny_read_matchers = Vec::new();
        for pattern in file_system_sandbox_policy.get_unreadable_globs_with_cwd(cwd) {
            match build_glob_matcher(&pattern) {
                Ok(matcher) => deny_read_matchers.push(matcher),
                Err(err) => match invalid_glob_behavior {
                    InvalidDenyReadGlobBehavior::FailClosed => invalid_pattern = true,
                    InvalidDenyReadGlobBehavior::ReturnError => {
                        return Err(format!("invalid deny-read glob pattern `{pattern}`: {err}"));
                    }
                },
            }
        }
=======
        let cwd = AbsolutePathBuf::from_absolute_path(cwd)
            .map_err(|err| format!("invalid read-deny cwd: {err}"))?;
        let cwd_uri = PathUri::from_abs_path(&cwd);
        let user_home_dir = PathUri::from_host_native_path("~").ok();
        let temporary_directories = local_temporary_directories();
        let context = FileSystemSandboxPolicyContext {
            cwd: &cwd_uri,
            workspace_roots: std::slice::from_ref(&cwd_uri),
            user_home_dir: user_home_dir.as_ref(),
            temporary_directories: Some(&temporary_directories),
        };
        let prepared = file_system_sandbox_policy
            .prepare_deny_read_matcher(&context, invalid_glob_behavior)?;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        Ok(Some(Self {
<<<<<<< HEAD
            denied_candidates,
            deny_read_matchers,
            invalid_pattern,
            unlistable_candidates,
||||||| 413492cd6c
            denied_candidates,
            deny_read_matchers,
            invalid_pattern,
=======
            native_cwd: Some(cwd),
            user_home_dir,
            temporary_directories,
            prepared,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        }))
    }

    /// Returns whether a local native `path` is denied by this matcher.
    pub fn is_local_path_read_denied(&self, path: &Path) -> bool {
        let Some(cwd) = self.native_cwd.as_ref() else {
            return true;
        };
        let Some(path) = resolve_candidate_path(path, cwd.as_path()) else {
            return true;
        };
        let path = PathUri::from(path);
        let cwd = PathUri::from_abs_path(cwd);
        let context = FileSystemSandboxPolicyContext {
            cwd: &cwd,
            workspace_roots: std::slice::from_ref(&cwd),
            user_home_dir: self.user_home_dir.as_ref(),
            temporary_directories: Some(&self.temporary_directories),
        };
        self.is_read_denied_uri(&path, &context)
    }

<<<<<<< HEAD
        self.deny_read_matchers.iter().any(|(matcher, credential)| {
            path_candidates.iter().any(|candidate| {
                matcher.is_match(candidate)
                    && !(*credential
                        && candidate
                            .file_name()
                            .is_some_and(is_credential_template_name))
            })
        })
    }

    /// Returns whether listing the directory `path` is denied: its contents
    /// are unreadable, or it is a directory the policy may not list (#506).
    pub fn is_listing_denied(&self, path: &Path) -> bool {
        if self.is_read_denied(path) {
            return true;
        }
        let path_candidates = normalized_and_canonical_candidates(path);
        self.unlistable_candidates.iter().any(|unlistable| {
            path_candidates
                .iter()
                .any(|candidate| unlistable.contains(candidate))
        })
||||||| 413492cd6c
        self.deny_read_matchers.iter().any(|matcher| {
            path_candidates
                .iter()
                .any(|candidate| matcher.is_match(candidate))
        })
=======
    /// Returns whether an executor-owned URI is denied under its matching path context.
    pub fn is_read_denied_uri(
        &self,
        path: &PathUri,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> bool {
        FileSystemSandboxPolicy::matches_prepared_read_deny(path, context, &self.prepared)
    }

    /// Checks an enumerated path using a canonical location already resolved by
    /// the caller, without reopening the file to canonicalize it.
    ///
    /// Bulk filesystem walkers may derive this location from a freshly resolved
    /// parent and a non-symlink directory entry. Symlinks and Windows junctions
    /// must be resolved separately. Do not reuse these locations across walks:
    /// a later operation must observe newly created files and changed links.
    pub fn is_local_path_read_denied_with_canonical_path(
        &self,
        path: &Path,
        canonical_path: &Path,
    ) -> bool {
        self.is_local_path_read_denied(path) || self.is_local_path_read_denied(canonical_path)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }
}

#[derive(Clone, Copy)]
enum InvalidDenyReadGlobBehavior {
    FailClosed,
    ReturnError,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileSystemPath {
    Path {
        path: PathUri,
    },
    /// A git-style glob pattern. Pattern entries currently support
    /// FileSystemAccessMode::Deny only.
    GlobPattern {
        pattern: String,
        /// Linux expansion provenance for generated database-entry denials.
        /// Omitted for ordinary user and credential globs.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        scan: Option<FileSystemGlobScan>,
    },
    Special {
        value: FileSystemSpecialPath,
    },
}

/// Serialized filesystem path whose literal path variant preserves the raw
/// legacy string until an explicit seam conversion selects its meaning.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(tag = "type")]
#[schemars(rename = "FileSystemPath")]
#[ts(rename = "FileSystemPath")]
pub enum RawFileSystemPath {
    Path {
        #[schemars(with = "AbsolutePathBuf")]
        #[ts(type = "string")]
        path: LegacyAppPathString,
    },
    GlobPattern {
        pattern: String,
    },
    Special {
        value: FileSystemSpecialPath,
    },
}

impl From<AbsolutePathBuf> for FileSystemPath {
    fn from(path: AbsolutePathBuf) -> Self {
        Self::Path { path: path.into() }
    }
}

impl From<PathUri> for FileSystemPath {
    fn from(path: PathUri) -> Self {
        Self::Path { path }
    }
}

fn path_uri_from_raw(path: LegacyAppPathString) -> Result<PathUri, String> {
    let native_path =
        serde::de::value::StrDeserializer::<serde::de::value::Error>::new(path.as_str());
    if let Ok(path) = AbsolutePathBuf::deserialize(native_path) {
        return Ok(PathUri::from(path));
    }

    PathUri::try_from(path).map_err(|err| err.to_string())
}

fn raw_path_from_uri(path: PathUri) -> Result<LegacyAppPathString, String> {
    let raw_path = LegacyAppPathString::from(path.clone());
    if path_uri_from_raw(raw_path.clone()).as_ref() == Ok(&path) {
        Ok(raw_path)
    } else {
        Err("permission path cannot be represented losslessly".to_string())
    }
}

impl TryFrom<RawFileSystemPath> for FileSystemPath {
    type Error = String;

    fn try_from(path: RawFileSystemPath) -> Result<Self, Self::Error> {
        Ok(match path {
            RawFileSystemPath::Path { path } => Self::Path {
                path: path_uri_from_raw(path)?,
            },
            RawFileSystemPath::GlobPattern { pattern } => Self::GlobPattern { pattern },
            RawFileSystemPath::Special { value } => Self::Special { value },
        })
    }
}

impl TryFrom<FileSystemPath> for RawFileSystemPath {
    type Error = String;

    fn try_from(path: FileSystemPath) -> Result<Self, Self::Error> {
        Ok(match path {
            FileSystemPath::Path { path } => Self::Path {
                path: raw_path_from_uri(path)?,
            },
            FileSystemPath::GlobPattern { pattern } => Self::GlobPattern { pattern },
            FileSystemPath::Special { value } => Self::Special { value },
        })
    }
}

impl TryFrom<RawFileSystemSandboxEntry> for FileSystemSandboxEntry {
    type Error = String;

    fn try_from(entry: RawFileSystemSandboxEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            path: entry.path.try_into()?,
            access: entry.access,
            missing_path_behavior: entry.missing_path_behavior,
        })
    }
}

impl TryFrom<FileSystemSandboxEntry> for RawFileSystemSandboxEntry {
    type Error = String;

    fn try_from(entry: FileSystemSandboxEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            path: entry.path.try_into()?,
            access: entry.access,
            missing_path_behavior: entry.missing_path_behavior,
        })
    }
}

impl TryFrom<RawFileSystemSandboxPolicy> for FileSystemSandboxPolicy {
    type Error = String;

    fn try_from(policy: RawFileSystemSandboxPolicy) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: policy.kind,
            glob_scan_max_depth: policy.glob_scan_max_depth,
            entries: policy
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<FileSystemSandboxPolicy> for RawFileSystemSandboxPolicy {
    type Error = String;

    fn try_from(policy: FileSystemSandboxPolicy) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: policy.kind,
            glob_scan_max_depth: policy.glob_scan_max_depth,
            entries: policy
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

const PROJECT_ROOTS_GLOB_PATTERN_PREFIX: &str = "codex-project-roots://";

pub fn project_roots_glob_pattern(subpath: &Path) -> String {
    format!("{PROJECT_ROOTS_GLOB_PATTERN_PREFIX}{}", subpath.display())
}

/// Special-path token of a `deny` entry that stops agent commands listing one
/// directory (issue #506): the directory's entries cannot be enumerated, while
/// the directory itself and every path below it keep the access the rest of
/// the policy gives them. The directory is the entry's absolute `subpath`.
///
/// It is carried as [`FileSystemSpecialPath::Unknown`] so that a runtime that
/// predates it ignores it, as unknown tokens are designed to be: only the
/// sandboxes and matchers that know the token enforce it.
pub const DIRECTORY_LISTING_SPECIAL_PATH: &str = ":directory_listing";

impl FileSystemSandboxEntry {
    /// A `deny` entry that stops agent commands listing `directory`.
    pub fn deny_directory_listing(directory: &AbsolutePathBuf) -> Self {
        Self::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::unknown(
                    DIRECTORY_LISTING_SPECIAL_PATH,
                    Some(directory.to_string_lossy().into_owned()),
                ),
            },
            FileSystemAccessMode::Deny,
        )
    }

    /// The directory this entry stops agent commands listing, if it is a
    /// [`Self::deny_directory_listing`] entry.
    pub fn unlistable_directory(&self) -> Option<AbsolutePathBuf> {
        if self.access != FileSystemAccessMode::Deny {
            return None;
        }
        match &self.path {
            FileSystemPath::Special {
                value:
                    FileSystemSpecialPath::Unknown {
                        path,
                        subpath: Some(subpath),
                    },
            } if path == DIRECTORY_LISTING_SPECIAL_PATH => {
                AbsolutePathBuf::from_absolute_path(Path::new(subpath)).ok()
            }
            _ => None,
        }
    }
}

/// Special-path token of an entry that stops agent commands moving one path
/// aside (issues #594, #598): the path and every folder above it may not be
/// renamed or removed where the sandbox would otherwise let the command do
/// so; a link at the path keeps pointing where it does (Seatbelt). What is
/// in those folders keeps the access the rest of the policy gives it. The
/// path is the entry's absolute `subpath`. Carried as
/// [`FileSystemSpecialPath::Unknown`], like [`DIRECTORY_LISTING_SPECIAL_PATH`].
pub const NO_MOVE_SPECIAL_PATH: &str = ":no_move";

impl FileSystemSandboxEntry {
    /// An entry that stops agent commands moving `path` or a folder above it.
    pub fn deny_moving(path: &AbsolutePathBuf) -> Self {
        Self::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::unknown(
                    NO_MOVE_SPECIAL_PATH,
                    Some(path.to_string_lossy().into_owned()),
                ),
            },
            FileSystemAccessMode::Deny,
        )
    }

    /// The path this entry stops agent commands moving, if it is a
    /// [`Self::deny_moving`] entry.
    pub fn unmovable_path(&self) -> Option<AbsolutePathBuf> {
        if self.access != FileSystemAccessMode::Deny {
            return None;
        }
        match &self.path {
            FileSystemPath::Special {
                value:
                    FileSystemSpecialPath::Unknown {
                        path,
                        subpath: Some(subpath),
                    },
            } if path == NO_MOVE_SPECIAL_PATH => {
                AbsolutePathBuf::from_absolute_path(Path::new(subpath)).ok()
            }
            _ => None,
        }
    }
}

/// Special-path token of a `deny` entry that keeps agent commands away from
/// the macOS keychain services (issue #758): Seatbelt denies looking them up
/// after every other rule, so neither its network rules (a managed network
/// proxy, allowed Unix sockets) nor its platform defaults reopen them. It
/// changes no file access. Carried as [`FileSystemSpecialPath::Unknown`],
/// like [`DIRECTORY_LISTING_SPECIAL_PATH`]; other sandboxes ignore it.
pub const KEYCHAIN_SERVICES_SPECIAL_PATH: &str = ":keychain_services";

impl FileSystemSandboxEntry {
    /// A `deny` entry that keeps agent commands away from the macOS
    /// keychain services.
    pub fn deny_keychain_services() -> Self {
        Self::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::unknown(
                    KEYCHAIN_SERVICES_SPECIAL_PATH,
                    /*subpath*/ None,
                ),
            },
            FileSystemAccessMode::Deny,
        )
    }

    /// Whether this is a [`Self::deny_keychain_services`] entry.
    pub fn denies_keychain_services(&self) -> bool {
        self.access == FileSystemAccessMode::Deny
            && matches!(
                &self.path,
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Unknown { path, subpath: None },
                } if path == KEYCHAIN_SERVICES_SPECIAL_PATH
            )
    }
}

/// Special-path token of a `deny` entry that stops agent commands creating
/// user namespaces (issue #826): the Linux sandbox refuses `unshare` and
/// `clone` with `CLONE_NEWUSER` (`EPERM`) and `clone3` (`ENOSYS`, so C
/// libraries fall back to `clone`). Inside a user namespace of its own a
/// command could open the placeholders bubblewrap hides denied files with
/// and mount over paths. It changes no file access. Carried as
/// [`FileSystemSpecialPath::Unknown`], like
/// [`KEYCHAIN_SERVICES_SPECIAL_PATH`]; other sandboxes ignore it.
pub const USER_NAMESPACES_SPECIAL_PATH: &str = ":user_namespaces";

impl FileSystemSandboxEntry {
    /// A `deny` entry that stops agent commands creating user namespaces.
    pub fn deny_user_namespaces() -> Self {
        Self::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::unknown(
                    USER_NAMESPACES_SPECIAL_PATH,
                    /*subpath*/ None,
                ),
            },
            FileSystemAccessMode::Deny,
        )
    }

    /// Whether this is a [`Self::deny_user_namespaces`] entry.
    pub fn denies_user_namespaces(&self) -> bool {
        self.access == FileSystemAccessMode::Deny
            && matches!(
                &self.path,
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Unknown { path, subpath: None },
                } if path == USER_NAMESPACES_SPECIAL_PATH
            )
    }
}

fn read_only_file_system_entries() -> Vec<FileSystemSandboxEntry> {
    vec![FileSystemSandboxEntry::new(
        FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        },
        FileSystemAccessMode::Read,
    )]
}

impl Default for FileSystemSandboxPolicy {
    fn default() -> Self {
        Self::read_only()
    }
}

impl FileSystemSandboxPolicy {
    pub fn read_only() -> Self {
        Self::restricted(read_only_file_system_entries())
    }

    pub fn unrestricted() -> Self {
        Self {
            kind: FileSystemSandboxKind::Unrestricted,
            glob_scan_max_depth: None,
            entries: Vec::new(),
        }
    }

    pub fn external_sandbox() -> Self {
        Self {
            kind: FileSystemSandboxKind::ExternalSandbox,
            glob_scan_max_depth: None,
            entries: Vec::new(),
        }
    }

    pub fn restricted(entries: Vec<FileSystemSandboxEntry>) -> Self {
        Self {
            kind: FileSystemSandboxKind::Restricted,
            glob_scan_max_depth: None,
            entries,
        }
    }

    /// Removes entries that should be skipped when their paths are missing.
    ///
    /// Callers that materialize filesystem ACL targets should not turn these
    /// entries into newly-created sentinel paths.
    pub fn remove_skip_missing_path_entries(&mut self) {
        self.entries.retain(|entry| !entry.skips_missing_path());
    }

    /// Native-path compatibility adapter for local executor boundaries.
    /// Selected-executor orchestrators must keep paths as [`PathUri`] and use
    /// context-aware policy APIs instead.
    pub fn has_explicit_non_write_entry_for_local_path_with_cwd(
        &self,
        path: &Path,
        cwd: &Path,
    ) -> bool {
        let Some(path) = resolve_candidate_path(path, cwd) else {
            return false;
        };
        let cwd = AbsolutePathBuf::from_absolute_path(cwd).ok();
        self.entries.iter().any(|entry| {
            !entry.skips_missing_path()
                && !entry.access.can_write()
                && resolve_entry_path(&entry.path, cwd.as_ref()).as_ref() == Some(&path)
        })
    }

    /// As [`Self::has_explicit_non_write_entry_for_path_with_cwd`], but an
    /// entry spelled differently also matches when both resolve (links
    /// followed) to the same existing location: a carve-out under a writable
    /// root reached through a link is listed against the root's resolved path
    /// (#1542 item 2, D293).
    pub fn has_explicit_non_write_entry_resolving_to(&self, path: &Path, cwd: &Path) -> bool {
        if self.has_explicit_non_write_entry_for_path_with_cwd(path, cwd) {
            return true;
        }
        let Some(real) = resolve_candidate_path(path, cwd)
            .and_then(|path| std::fs::canonicalize(path.as_path()).ok())
        else {
            return false;
        };
        let cwd = AbsolutePathBuf::from_absolute_path(cwd).ok();
        self.entries.iter().any(|entry| {
            !entry.skips_missing_path()
                && !entry.access.can_write()
                && resolve_entry_path(&entry.path, cwd.as_ref())
                    .and_then(|entry| std::fs::canonicalize(entry.as_path()).ok())
                    .is_some_and(|entry| entry == real)
        })
    }

    fn has_root_access(&self, predicate: impl Fn(FileSystemAccessMode) -> bool) -> bool {
        matches!(self.kind, FileSystemSandboxKind::Restricted)
            && self.entries.iter().any(|entry| {
                matches!(
                    &entry.path,
                    FileSystemPath::Special { value }
                        if matches!(value, FileSystemSpecialPath::Root) && predicate(entry.access)
                )
            })
    }

    pub fn has_denied_read_restrictions(&self) -> bool {
        // TODO(anp) Migrate callers to select the executor's path convention explicitly.
        self.has_denied_read_restrictions_for_convention(Some(PathConvention::native()))
    }

    fn has_denied_read_restrictions_for_convention(
        &self,
        convention: Option<PathConvention>,
    ) -> bool {
        matches!(self.kind, FileSystemSandboxKind::Restricted)
            && self.entries.iter().any(|entry| {
                // #598: a no-move entry denies no read; a run a person
                // approves outside the sandbox may drop it.
                entry.access == FileSystemAccessMode::Deny
                    && entry.unmovable_path().is_none()
                    // #758: neither does a keychain-services entry.
                    && !entry.denies_keychain_services()
                    // #826: nor a user-namespaces entry.
                    && !entry.denies_user_namespaces()
                    && !matches!(
                        &entry.path,
                        FileSystemPath::Special {
                            value: FileSystemSpecialPath::SlashTmp,
                        } if convention == Some(PathConvention::Windows)
                    )
            })
    }

    pub fn from_legacy_sandbox_policy_preserving_deny_entries(
        sandbox_policy: &SandboxPolicy,
        cwd: &Path,
        existing: &Self,
    ) -> Self {
        let mut rebuilt = Self::from_legacy_sandbox_policy_for_cwd(sandbox_policy, cwd);
        if !matches!(rebuilt.kind, FileSystemSandboxKind::Restricted) {
            return rebuilt;
        }
        rebuilt.glob_scan_max_depth = existing.glob_scan_max_depth;

        for deny_entry in existing
            .entries
            .iter()
            .filter(|entry| entry.access == FileSystemAccessMode::Deny)
        {
            if !rebuilt.entries.iter().any(|entry| entry == deny_entry) {
                rebuilt.entries.push(deny_entry.clone());
            }
        }

        rebuilt
    }

    /// Preserve explicit read-deny rules from `existing` when a caller
    /// replaces the allow side of a policy.
    pub fn preserve_deny_read_restrictions_from(&mut self, existing: &Self) {
        let has_deny_read_entries = existing
            .entries
            .iter()
            .any(|entry| entry.access == FileSystemAccessMode::Deny);
        if matches!(self.kind, FileSystemSandboxKind::Unrestricted) && has_deny_read_entries {
            *self = Self::restricted(vec![FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Write,
            )]);
        }

        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return;
        }

        if self.glob_scan_max_depth.is_none() {
            self.glob_scan_max_depth = existing.glob_scan_max_depth;
        }

        for deny_entry in existing
            .entries
            .iter()
            .filter(|entry| entry.access == FileSystemAccessMode::Deny)
        {
            if !self.entries.iter().any(|entry| entry == deny_entry) {
                self.entries.push(deny_entry.clone());
            }
        }
    }

    /// Returns true when a restricted policy contains any entry that really
    /// reduces a broader `:root = write` grant.
    ///
    /// Raw entry presence is not enough here: an equally specific `write`
    /// entry for the same target wins under the normal precedence rules, so a
    /// shadowed `read` entry must not downgrade the policy out of full-disk
    /// write mode.
    fn has_write_narrowing_entries(&self, convention: PathConvention) -> bool {
        matches!(self.kind, FileSystemSandboxKind::Restricted)
            && self.entries.iter().any(|entry| {
                if entry.access.can_write() {
                    return false;
                }

                match &entry.path {
                    FileSystemPath::Path { .. } => {
                        !self.has_same_target_write_override(entry, convention)
                    }
                    FileSystemPath::GlobPattern { .. } => true,
                    FileSystemPath::Special { value } => match value {
                        FileSystemSpecialPath::Root => entry.access == FileSystemAccessMode::Deny,
                        FileSystemSpecialPath::SlashTmp
                            if convention == PathConvention::Windows =>
                        {
                            false
                        }
                        FileSystemSpecialPath::Minimal | FileSystemSpecialPath::Unknown { .. } => {
                            false
                        }
                        _ => !self.has_same_target_write_override(entry, convention),
                    },
                }
            })
    }

    /// Returns true when a higher-priority `write` entry targets the same
    /// location as `entry`, so `entry` cannot narrow effective write access.
    fn has_same_target_write_override(
        &self,
        entry: &FileSystemSandboxEntry,
        convention: PathConvention,
    ) -> bool {
        self.entries.iter().any(|candidate| {
            candidate.access.can_write()
                && candidate.access > entry.access
                && file_system_paths_share_target(&candidate.path, &entry.path, convention)
        })
    }

    /// Filesystem policy matching `WorkspaceWrite` semantics without requiring
    /// callers to construct a legacy [`SandboxPolicy`] first.
    pub fn workspace_write(
        writable_roots: &[AbsolutePathBuf],
        exclude_tmpdir_env_var: bool,
        exclude_slash_tmp: bool,
    ) -> Self {
        let mut entries = vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Read,
        )];

        entries.push(FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            FileSystemAccessMode::Write,
        ));
        if !exclude_slash_tmp {
            entries.push(FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::SlashTmp,
                },
                FileSystemAccessMode::Write,
            ));
        }
        if !exclude_tmpdir_env_var {
            entries.push(FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Tmpdir,
                },
                FileSystemAccessMode::Write,
            ));
        }
        entries.extend(
            writable_roots
                .iter()
                .cloned()
                .map(|path| FileSystemSandboxEntry::new(path.into(), FileSystemAccessMode::Write)),
        );

        append_default_read_only_project_root_subpath_if_no_explicit_rule(&mut entries, ".git");
        append_default_read_only_project_root_subpath_if_no_explicit_rule(&mut entries, ".agents");
        append_default_read_only_project_root_subpath_if_no_explicit_rule(&mut entries, ".codex");
        append_default_read_only_project_root_subpath_if_no_explicit_rule(&mut entries, ".aws");
        for writable_root in writable_roots {
            for protected_path in default_read_only_subpaths_for_writable_root(
                writable_root,
                /*protect_missing_dot_codex*/ false,
            ) {
                append_default_read_only_path_if_no_explicit_rule(&mut entries, protected_path);
            }
        }

        FileSystemSandboxPolicy::restricted(entries)
    }

    /// Converts a legacy sandbox policy into an equivalent filesystem policy
    /// after resolving cwd-sensitive legacy defaults for the provided cwd.
    ///
    /// Legacy `WorkspaceWrite` policies may list readable roots that live
    /// under an already-writable root. Those paths were redundant in the
    /// legacy model and should not become read-only carveouts when projected
    /// into split filesystem policy.
    pub fn from_legacy_sandbox_policy_for_cwd(sandbox_policy: &SandboxPolicy, cwd: &Path) -> Self {
        let mut file_system_policy = Self::from(sandbox_policy);
        if let SandboxPolicy::WorkspaceWrite { writable_roots, .. } = sandbox_policy {
            if let Ok(cwd_root) = AbsolutePathBuf::from_absolute_path(cwd) {
                for protected_path in default_read_only_subpaths_for_writable_root(
                    &cwd_root, /*protect_missing_dot_codex*/ true,
                ) {
                    append_default_read_only_path_if_no_explicit_rule(
                        &mut file_system_policy.entries,
                        protected_path,
                    );
                }
            }
            for writable_root in writable_roots {
                for protected_path in default_read_only_subpaths_for_writable_root(
                    writable_root,
                    /*protect_missing_dot_codex*/ false,
                ) {
                    append_default_read_only_path_if_no_explicit_rule(
                        &mut file_system_policy.entries,
                        protected_path,
                    );
                }
            }
        }

        file_system_policy
    }

    /// Returns true when filesystem reads are unrestricted on this host.
    pub fn has_full_disk_read_access(&self) -> bool {
        // TODO(anp) Migrate callers to select the executor's path convention explicitly.
        self.has_full_disk_read_access_for_convention(Some(PathConvention::native()))
    }

    /// Returns true when filesystem reads are unrestricted on the selected executor.
    /// If the convention is unknown, a `:slash_tmp` denial is still treated as a restriction.
    pub fn has_full_disk_read_access_for_convention(
        &self,
        convention: Option<PathConvention>,
    ) -> bool {
        match self.kind {
            FileSystemSandboxKind::Unrestricted | FileSystemSandboxKind::ExternalSandbox => true,
            FileSystemSandboxKind::Restricted => {
                self.has_root_access(FileSystemAccessMode::can_read)
                    && !self.has_denied_read_restrictions_for_convention(convention)
            }
        }
    }

    /// Returns true when filesystem writes are unrestricted on this host.
    pub fn has_full_disk_write_access(&self) -> bool {
        // TODO(anp) Migrate callers to select the executor's path convention explicitly.
        self.has_full_disk_write_access_for_convention(Some(PathConvention::native()))
    }

    /// Returns true when filesystem writes are unrestricted for the selected executor.
    pub fn has_full_disk_write_access_with_context(
        &self,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> bool {
        self.has_full_disk_write_access_for_convention(context.cwd.infer_path_convention())
    }

    /// Returns true when filesystem writes are unrestricted for the selected path convention.
    pub fn has_full_disk_write_access_for_convention(
        &self,
        convention: Option<PathConvention>,
    ) -> bool {
        match self.kind {
            FileSystemSandboxKind::Unrestricted | FileSystemSandboxKind::ExternalSandbox => true,
            FileSystemSandboxKind::Restricted => convention.is_some_and(|convention| {
                self.has_root_access(FileSystemAccessMode::can_write)
                    && !self.has_write_narrowing_entries(convention)
            }),
        }
    }

    /// Returns true when platform-default readable roots should be included.
    pub fn include_platform_defaults(&self) -> bool {
        !self.has_full_disk_read_access()
            && matches!(self.kind, FileSystemSandboxKind::Restricted)
            && self.entries.iter().any(|entry| {
                matches!(
                    &entry.path,
                    FileSystemPath::Special { value }
                        if matches!(value, FileSystemSpecialPath::Minimal)
                            && entry.access.can_read()
                )
            })
    }

    /// Native-path compatibility adapter for local executor boundaries.
    /// Selected-executor orchestrators must use [`Self::resolve_access`] with
    /// a [`PathUri`] and [`FileSystemSandboxPolicyContext`] instead.
    pub fn resolve_access_for_local_path_with_cwd(
        &self,
        path: &Path,
        cwd: &Path,
    ) -> FileSystemAccessMode {
        with_local_policy_context(path, cwd, |path, context| {
            self.prepare_local_matching(context)
                .map(|matching| matching.resolve_access(path))
                .unwrap_or(FileSystemAccessMode::Deny)
        })
        .unwrap_or(FileSystemAccessMode::Deny)
    }

    /// Native-path compatibility adapter for local executor boundaries.
    pub fn can_read_local_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        self.resolve_access_for_local_path_with_cwd(path, cwd)
            .can_read()
    }

    /// Native-path compatibility adapter for local executor boundaries.
    pub fn can_write_local_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        with_local_policy_context(path, cwd, |path, context| {
            self.prepare_local_matching(context)
                .and_then(|matching| matching.can_write_path(path))
                .unwrap_or(false)
        })
        .unwrap_or(false)
    }

    pub fn resolve_access(
        &self,
        path: &PathUri,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> FileSystemAccessMode {
        match self.kind {
            FileSystemSandboxKind::Unrestricted | FileSystemSandboxKind::ExternalSandbox => {
                return FileSystemAccessMode::Write;
            }
            FileSystemSandboxKind::Restricted => {}
        }

        let Some(convention) = context.cwd.infer_path_convention() else {
            return FileSystemAccessMode::Deny;
        };
        if path.infer_path_convention() != Some(convention)
            || path.lexical_depth().is_none()
            || context.cwd.lexical_depth().is_none()
        {
            return FileSystemAccessMode::Deny;
        }

        let Some(entries) = self.resolved_entries(context).into_iter().try_fold(
            Vec::new(),
            |mut entries, (root, access)| {
                match root.lexical_depth() {
                    Some(depth) => entries.push((root, access, depth)),
                    None if root.is_opaque() => {}
                    None => return None,
                }
                Some(entries)
            },
        ) else {
            return FileSystemAccessMode::Deny;
        };

        entries
            .into_iter()
            .filter(|(root, _, _)| path.starts_with(root))
            .max_by_key(|(_, access, depth)| (*depth, *access))
            .map(|(_, access, _)| access)
            .unwrap_or(FileSystemAccessMode::Deny)
    }

<<<<<<< HEAD
    pub fn can_read_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        self.resolve_access_with_cwd(path, cwd).can_read()
    }

    /// Like [`Self::can_read_path_with_cwd`], with the policy's paths and
    /// `path` normalized alike, the way the sandboxes normalize their roots
    /// before matching denials to them: top-level aliases resolve (`/tmp`
    /// and `/private/tmp` on macOS name one file), nested links stay as
    /// written, so a path spelled through a nested link does not match a
    /// rule spelled without it.
    ///
    /// This does not model a sandbox's own clauses; on macOS use
    /// `codex_sandboxing::seatbelt::seatbelt_allows_file_read` (#488).
    pub fn can_read_effective_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        self.effective_reads_with_cwd(cwd)
            .can_read_effective_path(path)
    }

    /// Issue918: effective-read checks for one set of paths, with the resolved
    /// entries normalized once on first use. Build a fresh view for each set;
    /// it is not a persistent cache or an atomic filesystem snapshot.
    pub fn effective_reads_with_cwd<'a>(&'a self, cwd: &'a Path) -> EffectiveReads<'a> {
        EffectiveReads::new(self, cwd)
    }

    /// Like [`Self::can_write_path_with_cwd`], with the policy's paths and
    /// `path` normalized alike, as [`Self::can_read_effective_path_with_cwd`]
    /// does: Seatbelt normalizes a writable root spelled `/tmp` to
    /// `/private/tmp`, so `/private/tmp/x` is writable through it (#619).
    /// Everything else, protected metadata names included, is decided as
    /// [`Self::can_write_path_with_cwd`] decides it for the normalized
    /// policy. #652: also writable through a writable root reached through a
    /// nested link ([`Self::writable_root_spellings_with_cwd`]), which the
    /// sandboxes grant at its real location. For deciding where a path
    /// becomes read-only; a write carve-out uses
    /// [`Self::can_grant_write_effective_path_with_cwd`].
    pub fn can_write_effective_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        self.effective_writes_with_cwd(cwd)
            .can_write_effective_path(path)
    }

    /// #652: [`Self::can_write_effective_path_with_cwd`] for adding a write
    /// carve-out, which must hold under every reading of the policy: where
    /// it names the same folder in two spellings with different access
    /// (`/tmp/p` writable, `/private/tmp/p` read-only), the read-only entry
    /// wins, and roots reached through a nested link are not followed.
    pub fn can_grant_write_effective_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        self.effective_writes_with_cwd(cwd)
            .can_grant_write_effective_path(path)
    }

    /// This policy with each entry naming a folder by its normalized path
    /// (one filesystem walk per entry); the others (the filesystem root,
    /// patterns) as they are.
    fn with_normalized_entries(&self, cwd: &Path) -> Self {
        let cwd_absolute = AbsolutePathBuf::from_absolute_path(cwd).ok();
        let entries = self
            .entries
            .iter()
            .map(|entry| match &entry.path {
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                } => entry.clone(),
                path => match resolve_entry_path(path, cwd_absolute.as_ref()) {
                    Some(path) => FileSystemSandboxEntry {
                        path: FileSystemPath::Path {
                            path: normalize_effective_absolute_path(path),
                        },
                        ..entry.clone()
                    },
                    None => entry.clone(),
                },
            })
            .collect();
        Self {
            kind: self.kind,
            glob_scan_max_depth: self.glob_scan_max_depth,
            entries,
        }
    }

    /// A writable entry tied with a read-only one for the same path becomes
    /// read-only (a denial wins already); in memory only.
    fn with_read_only_ties(mut self) -> Self {
        let read_only: Vec<AbsolutePathBuf> = self
            .entries
            .iter()
            .filter(|entry| entry.access == FileSystemAccessMode::Read)
            .filter_map(|entry| match &entry.path {
                FileSystemPath::Path { path } => Some(path.clone()),
                _ => None,
            })
            .collect();
        for entry in &mut self.entries {
            if entry.access == FileSystemAccessMode::Write
                && matches!(&entry.path, FileSystemPath::Path { path } if read_only.contains(path))
            {
                entry.access = FileSystemAccessMode::Read;
            }
        }
        self
    }

    /// #907: [`Self::can_write_effective_path_with_cwd`] and
    /// [`Self::can_grant_write_effective_path_with_cwd`] for many paths in
    /// `cwd`, with the entries normalized and the roots reached through a
    /// nested link computed once instead of once per path.
    pub fn effective_writes_with_cwd(&self, cwd: &Path) -> EffectiveWrites<'_> {
        EffectiveWrites::new(self, cwd)
    }

    /// #652: the writable roots with their normalized spellings and real
    /// locations, where the sandboxes grant them (a root reached through a
    /// nested link, `~/ws -> /Volumes/x/ws`, at its target), computed once
    /// for many paths.
    pub fn writable_root_links_with_cwd(&self, cwd: &Path) -> WritableRootLinks {
        #[cfg(unix)]
        {
            if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
                return WritableRootLinks::default();
            }
            let roots = self
                .get_writable_roots_with_cwd(cwd)
                .into_iter()
                .map(|root| {
                    let real = real_location(root.root.as_path());
                    let normalized = normalize_effective_absolute_path(root.root.clone());
                    (root.root, normalized, real)
                })
                .collect();
            WritableRootLinks { roots }
        }
        #[cfg(not(unix))]
        {
            let _ = cwd;
            WritableRootLinks::default()
        }
    }

    /// #652: `path` spelled through each writable root reached through a
    /// nested link ([`Self::writable_root_links_with_cwd`]).
    pub fn writable_root_spellings_with_cwd(
        &self,
        path: &Path,
        cwd: &Path,
    ) -> Vec<AbsolutePathBuf> {
        self.writable_root_links_with_cwd(cwd).spellings(path, cwd)
    }

    pub fn can_write_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        if !self.resolve_access_with_cwd(path, cwd).can_write() {
||||||| 413492cd6c
    pub fn can_read_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        self.resolve_access_with_cwd(path, cwd).can_read()
    }

    pub fn can_write_path_with_cwd(&self, path: &Path, cwd: &Path) -> bool {
        if !self.resolve_access_with_cwd(path, cwd).can_write() {
=======
    pub fn can_write_path(
        &self,
        path: &PathUri,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> bool {
        if !self.resolve_access(path, context).can_write() {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
            return false;
        }
        self.has_full_disk_write_access_with_context(context)
            || self.metadata_write_denial(path, context).is_none()
    }

    fn metadata_write_denial(
        &self,
        path: &PathUri,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> Option<&'static str> {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return None;
        }
        let entries = self.resolved_entries(context);
        let (protected, metadata_name) = entries
            .iter()
            .filter(|(_, access)| access.can_write())
            .find_map(|(root, _)| {
                PROTECTED_METADATA_PATH_NAMES
                    .iter()
                    .find_map(|metadata_name| {
                        let protected = root.join_descendant(metadata_name).ok()?;
                        path.starts_with(&protected)
                            .then_some((protected, *metadata_name))
                    })
            })?;
        (!entries.iter().any(|(root, access)| {
            access.can_write() && path.starts_with(root) && root.starts_with(&protected)
        }))
        .then_some(metadata_name)
    }

    fn prepare_deny_read_matcher(
        &self,
        context: &FileSystemSandboxPolicyContext<'_>,
        invalid_glob_behavior: InvalidDenyReadGlobBehavior,
    ) -> Result<PreparedReadDenyMatcher, String> {
        let file_system_root = file_system_root(context);
        let denied_roots = self
            .resolved_entries(context)
            .into_iter()
            .filter(|(_, access)| *access == FileSystemAccessMode::Deny)
            .filter(|(root, _)| {
                !file_system_root.as_ref().is_some_and(|file_system_root| {
                    root.starts_with(file_system_root) && file_system_root.starts_with(root)
                })
            })
            .map(|(root, _)| root)
            .collect();
        let Some(convention) = context.cwd.infer_path_convention() else {
            return Ok(PreparedReadDenyMatcher {
                denied_roots,
                deny_read_matchers: Vec::new(),
                invalid_pattern: true,
            });
        };
        let mut deny_read_matchers = Vec::new();
        let mut invalid_pattern = false;
        let patterns = match self.deny_read_globs(context) {
            Ok(patterns) => patterns,
            Err(err) => match invalid_glob_behavior {
                InvalidDenyReadGlobBehavior::FailClosed => {
                    return Ok(PreparedReadDenyMatcher {
                        denied_roots,
                        deny_read_matchers: Vec::new(),
                        invalid_pattern: true,
                    });
                }
<<<<<<< HEAD
                FileSystemPath::GlobPattern { pattern, scan } => {
                    if let (Some(cwd), Some(subpath)) =
                        (cwd.as_ref(), parse_project_roots_glob_pattern(pattern))
                    {
                        entry.path = FileSystemPath::GlobPattern {
                            scan: scan.clone(),
                            pattern: resolve_project_roots_glob_pattern(subpath, cwd),
                        };
||||||| 413492cd6c
                FileSystemPath::GlobPattern { pattern } => {
                    if let (Some(cwd), Some(subpath)) =
                        (cwd.as_ref(), parse_project_roots_glob_pattern(pattern))
                    {
                        entry.path = FileSystemPath::GlobPattern {
                            pattern: resolve_project_roots_glob_pattern(subpath, cwd),
                        };
=======
                InvalidDenyReadGlobBehavior::ReturnError => return Err(err),
            },
        };
        for pattern in patterns {
            match build_glob_matcher(&pattern, convention) {
                Ok(matcher) => deny_read_matchers.push(matcher),
                Err(err) => match invalid_glob_behavior {
                    InvalidDenyReadGlobBehavior::FailClosed => invalid_pattern = true,
                    InvalidDenyReadGlobBehavior::ReturnError => {
                        return Err(format!("invalid deny-read glob pattern `{pattern}`: {err}"));
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                    }
                },
            }
        }
        Ok(PreparedReadDenyMatcher {
            denied_roots,
            deny_read_matchers,
            invalid_pattern,
        })
    }

    fn matches_prepared_read_deny(
        path: &PathUri,
        context: &FileSystemSandboxPolicyContext<'_>,
        prepared: &PreparedReadDenyMatcher,
    ) -> bool {
        let Some(convention) = context.cwd.infer_path_convention() else {
            return true;
        };
        if path.infer_path_convention() != Some(convention)
            || path.lexical_depth().is_none()
            || context.cwd.lexical_depth().is_none()
        {
            return true;
        }
        if prepared.invalid_pattern {
            return true;
        }
        prepared
            .denied_roots
            .iter()
            .any(|root| path.starts_with(root))
            || prepared.deny_read_matchers.iter().any(|matcher| {
                let path = match convention {
                    PathConvention::Posix => path.decoded_path_bytes(),
                    PathConvention::Windows => Cow::Owned(
                        path.inferred_native_path_string()
                            .replace('\\', "/")
                            .into_bytes(),
                    ),
                };
                matcher.is_match_candidate(&Candidate::from_bytes(path.as_ref()))
            })
    }

    fn deny_read_globs(
        &self,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> Result<Vec<String>, String> {
        self.entries
            .iter()
            .filter(|entry| entry.access == FileSystemAccessMode::Deny)
            .filter_map(|entry| {
                let FileSystemPath::GlobPattern { pattern } = &entry.path else {
                    return None;
                };
                let is_windows =
                    context.cwd.infer_path_convention() == Some(PathConvention::Windows);
                let home_relative = pattern.strip_prefix("~/").or_else(|| {
                    is_windows.then(|| pattern.strip_prefix("~\\")).flatten()
                });
                let (root, pattern) = match home_relative {
                    Some(suffix) => match context.user_home_dir {
                        Some(home) => (
                            home,
                            suffix.trim_start_matches(|separator| {
                                separator == '/' || is_windows && separator == '\\'
                            }),
                        ),
                        None => {
                            return Some(Err(format!(
                                "unable to resolve deny-read glob pattern `{pattern}` without executor home"
                            )));
                        }
                    },
                    None => (context.cwd, pattern.as_str()),
                };
                let convention = if is_windows {
                    PathConvention::Windows
                } else {
                    PathConvention::Posix
                };
                if LegacyAppPathString::from_string(pattern)
                    .to_path_uri(convention)
                    .is_err()
                    && let Err(error) = root.validate_glob_directory(convention)
                {
                    return Some(Err(error.to_string()));
                }
                Some(
                    root
                        .join(pattern)
                        .map(|path| path.inferred_native_path_string())
                        .map_err(|_| {
                            format!("unable to resolve deny-read glob pattern `{pattern}`")
                        }),
                )
            })
            .collect()
    }

    /// Replaces symbolic `:workspace_roots` entries with concrete entries for
    /// each workspace root.
    pub fn materialize_project_roots_with_workspace_roots(
        self,
        workspace_roots: &[AbsolutePathBuf],
    ) -> Self {
        let roots = workspace_roots
            .iter()
            .map(PathUri::from_abs_path)
            .collect::<Vec<_>>();
        self.materialize_project_roots_with_path_uris(&roots)
    }

    /// Materializes workspace-root entries without projecting executor paths onto the host.
    /// Legacy home-relative workspace denials clear all grants when their target is unknown.
    pub fn materialize_project_roots_with_path_uris(self, workspace_roots: &[PathUri]) -> Self {
        self.try_materialize_project_roots_with_path_uris(workspace_roots)
            .unwrap_or_else(|| Self::restricted(Vec::new()))
    }

    fn try_materialize_project_roots_with_path_uris(
        mut self,
        workspace_roots: &[PathUri],
    ) -> Option<Self> {
        let mut entries = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            let (subpath, is_glob) = match &entry.path {
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::ProjectRoots { subpath },
                } => (subpath.as_deref(), false),
                FileSystemPath::GlobPattern { pattern }
                    if pattern.starts_with(PROJECT_ROOTS_GLOB_PATTERN_PREFIX) =>
                {
                    (
                        Some(&pattern[PROJECT_ROOTS_GLOB_PATTERN_PREFIX.len()..]),
                        true,
                    )
                }
<<<<<<< HEAD
                FileSystemPath::GlobPattern { pattern, scan } => {
                    if let Some(subpath) = parse_project_roots_glob_pattern(&pattern) {
                        entries.extend(workspace_roots.iter().map(|root| FileSystemSandboxEntry {
                            path: FileSystemPath::GlobPattern {
                                scan: scan.clone(),
                                pattern: resolve_project_roots_glob_pattern(subpath, root),
                            },
                            access: entry.access,
                            missing_path_behavior: entry.missing_path_behavior,
                        }));
                    } else {
                        entries.push(FileSystemSandboxEntry {
                            path: FileSystemPath::GlobPattern { pattern, scan },
                            access: entry.access,
                            missing_path_behavior: entry.missing_path_behavior,
                        });
                    }
                }
                FileSystemPath::Path { path } => {
                    entries.push(FileSystemSandboxEntry {
                        path: FileSystemPath::Path { path },
                        access: entry.access,
                        missing_path_behavior: entry.missing_path_behavior,
                    });
                }
                FileSystemPath::Special { value } => {
                    entries.push(FileSystemSandboxEntry {
                        path: FileSystemPath::Special { value },
                        access: entry.access,
                        missing_path_behavior: entry.missing_path_behavior,
                    });
||||||| 413492cd6c
                FileSystemPath::GlobPattern { pattern } => {
                    if let Some(subpath) = parse_project_roots_glob_pattern(&pattern) {
                        entries.extend(workspace_roots.iter().map(|root| FileSystemSandboxEntry {
                            path: FileSystemPath::GlobPattern {
                                pattern: resolve_project_roots_glob_pattern(subpath, root),
                            },
                            access: entry.access,
                            missing_path_behavior: entry.missing_path_behavior,
                        }));
                    } else {
                        entries.push(FileSystemSandboxEntry {
                            path: FileSystemPath::GlobPattern { pattern },
                            access: entry.access,
                            missing_path_behavior: entry.missing_path_behavior,
                        });
                    }
                }
                FileSystemPath::Path { path } => {
                    entries.push(FileSystemSandboxEntry {
                        path: FileSystemPath::Path { path },
                        access: entry.access,
                        missing_path_behavior: entry.missing_path_behavior,
                    });
                }
                FileSystemPath::Special { value } => {
                    entries.push(FileSystemSandboxEntry {
                        path: FileSystemPath::Special { value },
                        access: entry.access,
                        missing_path_behavior: entry.missing_path_behavior,
                    });
=======
                _ => {
                    entries.push(entry);
                    continue;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
                }
            };
            if entry.access == FileSystemAccessMode::Deny
                && subpath.is_some_and(|subpath| {
                    workspace_roots.iter().any(|root| {
                        root.infer_path_convention().is_some_and(|convention| {
                            convention.home_relative_suffix(subpath).is_some()
                        })
                    })
                })
            {
                // Older policies could expand these rules outside the workspace.
                // Without a home fact, denying only the workspace would weaken them.
                return None;
            }
            entries.extend(workspace_roots.iter().filter_map(|root| {
                let path = if is_glob
                    && root
                        .infer_path_convention()
                        .is_none_or(|convention| root.validate_glob_directory(convention).is_err())
                {
                    // An infallible materializer cannot return an unsafe glob.
                    // Use the existing deny-root fallback for unresolvable rules.
                    None
                } else {
                    subpath.map_or_else(
                        || Some(root.clone()),
                        |subpath| resolve_scoped_workspace_path(root, subpath),
                    )
                };
                let (path, access) = match path {
                    Some(path) if is_glob => (
                        FileSystemPath::GlobPattern {
                            pattern: path.inferred_native_path_string(),
                        },
                        entry.access,
                    ),
                    Some(path) => (FileSystemPath::Path { path }, entry.access),
                    None if !entry.access.can_write() => (
                        FileSystemPath::Path { path: root.clone() },
                        FileSystemAccessMode::Deny,
                    ),
                    None => return None,
                };
                Some(FileSystemSandboxEntry {
                    path,
                    access,
                    missing_path_behavior: entry.missing_path_behavior,
                })
            }));
        }
        self.entries = entries;
        Some(self)
    }

    /// Preserves symbolic `:workspace_roots` entries while also adding concrete
    /// entries for each provided workspace root.
    pub fn with_materialized_project_roots_for_workspace_roots(
        self,
        workspace_roots: &[AbsolutePathBuf],
    ) -> Self {
        let roots = workspace_roots
            .iter()
            .map(PathUri::from_abs_path)
            .collect::<Vec<_>>();
        self.with_materialized_project_roots_for_path_uris(&roots)
    }

    pub fn with_additional_readable_roots(
        mut self,
        cwd: &Path,
        additional_readable_roots: &[AbsolutePathBuf],
    ) -> Self {
        if self.has_full_disk_read_access() {
            return self;
        }

        for path in additional_readable_roots {
            if self.can_read_local_path_with_cwd(path.as_path(), cwd) {
                continue;
            }

            self.entries.push(FileSystemSandboxEntry::new(
                path.clone().into(),
                FileSystemAccessMode::Read,
            ));
        }

        self
    }

    pub fn with_additional_writable_roots(
        mut self,
        cwd: &Path,
        additional_writable_roots: &[AbsolutePathBuf],
    ) -> Self {
        for path in additional_writable_roots {
            if self.can_write_local_path_with_cwd(path.as_path(), cwd) {
                continue;
            }

            self.entries.push(FileSystemSandboxEntry::new(
                path.clone().into(),
                FileSystemAccessMode::Write,
            ));
        }

        self
    }

    /// Add roots using legacy `WorkspaceWrite` behavior.
    ///
    /// Unlike [`Self::with_additional_writable_roots`], this mirrors legacy
    /// writable-roots semantics by adding exact roots even when they are
    /// already writable through `:workspace_roots`, and by adding the default
    /// read-only protected subpaths for each new root.
    pub fn with_additional_legacy_workspace_writable_roots(
        mut self,
        additional_writable_roots: &[AbsolutePathBuf],
    ) -> Self {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return self;
        }

        for path in additional_writable_roots {
            if !self.entries.iter().any(|entry| {
                entry.access.can_write()
                    && matches!(&entry.path, FileSystemPath::Path { path: existing } if existing == &PathUri::from_abs_path(path))
            }) {
                self.entries.push(FileSystemSandboxEntry::new(
                    path.clone().into(),
                    FileSystemAccessMode::Write,
                ));
            }

            for protected_path in default_read_only_subpaths_for_writable_root(
                path, /*protect_missing_dot_codex*/ false,
            ) {
                append_default_read_only_path_if_no_explicit_rule(
                    &mut self.entries,
                    protected_path,
                );
            }
        }

        self
    }

    pub fn needs_direct_runtime_enforcement(
        &self,
        network_policy: NetworkSandboxPolicy,
        cwd: &Path,
    ) -> bool {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return false;
        }

        let Ok(legacy_policy) = self.to_legacy_sandbox_policy(network_policy, cwd) else {
            return true;
        };

        if protected_metadata_names_need_direct_runtime_enforcement(self, &legacy_policy, cwd) {
            return true;
        }

        self.semantic_signature(cwd)
            != legacy_runtime_file_system_policy_for_cwd(&legacy_policy, cwd)
                .semantic_signature(cwd)
    }

    /// Returns true when two policies resolve to the same filesystem access
    /// model for `cwd`, ignoring incidental entry ordering.
    pub fn is_semantically_equivalent_to(&self, other: &Self, cwd: &Path) -> bool {
        self.semantic_signature(cwd) == other.semantic_signature(cwd)
    }

    /// Returns the explicit readable roots resolved against the provided cwd.
    pub fn get_readable_roots_with_cwd(&self, cwd: &Path) -> Vec<AbsolutePathBuf> {
        if self.has_full_disk_read_access() {
            return Vec::new();
        }
        let Some(local_context) = LocalPolicyContext::new(cwd) else {
            return Vec::new();
        };
        let Ok(matching) = self.prepare_local_matching(&local_context.as_context()) else {
            return Vec::new();
        };

        dedup_absolute_paths(
            self.resolved_entries_with_cwd(cwd)
                .into_iter()
                .filter(|entry| entry.access.can_read())
                .filter(|entry| {
                    matching
                        .resolve_access(&PathUri::from_abs_path(&entry.path))
                        .can_read()
                })
                .map(|entry| entry.path)
                .collect(),
            /*normalize_effective_paths*/ true,
        )
    }

    /// Returns the writable roots together with read-only carveouts resolved
    /// against the provided cwd.
    ///
    /// Issue #535: a root that is reached through a link whose real location
    /// is, or lies inside, another root's read-only path is not a grant and
    /// is left out (see [`Self::writable_links_into_read_only_paths`]).
    pub fn get_writable_roots_with_cwd(&self, cwd: &Path) -> Vec<WritableRoot> {
<<<<<<< HEAD
        let roots = self.writable_roots_including_links_into_read_only_paths(cwd);
        let excluded = links_into_read_only_paths(&roots);
        roots
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !excluded.iter().any(|(link, _)| link == index))
            .map(|(_, root)| root)
            .collect()
    }

    /// Issue #535: the writable roots this policy names that are left out of
    /// [`Self::get_writable_roots_with_cwd`], each with the read-only path of
    /// another root its real location reaches. Such a root is a link (or
    /// passes through one) whose real location is, or lies inside, a path
    /// another root keeps read-only, like a writable link to a workspace's
    /// `.git/hooks`: it is not a grant, and the path stays read-only through
    /// it. Callers explain the exclusion to the person.
    pub fn writable_links_into_read_only_paths(
        &self,
        cwd: &Path,
    ) -> Vec<WritableLinkIntoReadOnlyPath> {
        let roots = self.writable_roots_including_links_into_read_only_paths(cwd);
        links_into_read_only_paths(&roots)
            .into_iter()
            .map(|(link, read_only_path)| WritableLinkIntoReadOnlyPath {
                link: roots[link].root.clone(),
                read_only_path,
            })
            .collect()
    }

    /// #606: whether `path` may be written as [`Self::can_write_path_with_cwd`]
    /// says, and not through a writable root
    /// [`Self::get_writable_roots_with_cwd`] leaves out (#535, `excluded`
    /// from [`Self::writable_links_into_read_only_paths`]): the sandboxes
    /// grant nothing there, so approvals must not count it as writable
    /// either (fail closed).
    pub fn can_write_path_outside_excluded_links(
        &self,
        path: &Path,
        cwd: &Path,
        excluded: &[WritableLinkIntoReadOnlyPath],
    ) -> bool {
        if !self.can_write_path_with_cwd(path, cwd) {
            return false;
        }
        if excluded.is_empty() {
            return true;
        }
        let Some(target) = resolve_candidate_path(path, cwd) else {
            return false;
        };
        let normalized = normalize_effective_absolute_path(target.clone());
        !excluded.iter().any(|excluded| {
            target.as_path().starts_with(excluded.link.as_path())
                || normalized
                    .as_path()
                    .starts_with(normalize_effective_absolute_path(excluded.link.clone()).as_path())
        })
    }

    fn writable_roots_including_links_into_read_only_paths(&self, cwd: &Path) -> Vec<WritableRoot> {
||||||| 413492cd6c
=======
        self.get_writable_roots_with_cwd_impl(
            cwd,
            WritableRootPathResolution::Effective,
            RootMetadataWriteMounts::Separate,
        )
    }

    /// Omits redundant root-metadata mounts when the filesystem root is writable.
    ///
    /// Explicit root metadata writes still disable their default protection. Linux
    /// inherits those writes from its root bind so a metadata symlink cannot cause
    /// a second bind to reopen its target under an explicitly denied directory.
    pub fn get_writable_roots_with_cwd_inheriting_root_metadata(
        &self,
        cwd: &Path,
    ) -> Vec<WritableRoot> {
        self.get_writable_roots_with_cwd_impl(
            cwd,
            WritableRootPathResolution::Effective,
            RootMetadataWriteMounts::InheritWritableRoot,
        )
    }

    /// Reports configured writable roots for diagnostics without inspecting the filesystem.
    ///
    /// Unlike runtime root resolution, this includes configured roots even if they
    /// do not currently exist (including `/tmp`). Do not use this result to authorize
    /// filesystem access or replace the resolution needed for sandbox enforcement.
    pub fn has_configured_writable_roots_with_cwd(&self, cwd: &Path) -> bool {
        with_local_policy_context(cwd, cwd, |_, context| {
            self.has_configured_writable_roots(context)
        })
        .unwrap_or(false)
    }

    /// Reports configured writable roots for executor-context diagnostics, excluding
    /// full-disk policies and without inspecting the filesystem.
    pub fn has_configured_writable_roots(
        &self,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> bool {
        !self.has_full_disk_write_access_with_context(context)
            && self
                .resolved_entries(context)
                .into_iter()
                .any(|(path, access)| access.can_write() && self.can_write_path(&path, context))
    }

    /// Returns writable roots without following attacker-mutable path components.
    ///
    /// Trusted top-level aliases such as `/tmp -> /private/tmp` are still
    /// normalized so roots and carveouts are compared in the same namespace.
    /// Deeper components remain exactly as configured until the platform
    /// sandbox binds them.
    pub fn get_writable_roots_with_cwd_preserving_mutable_paths(
        &self,
        cwd: &Path,
    ) -> Vec<WritableRoot> {
        self.get_writable_roots_with_cwd_impl(
            cwd,
            WritableRootPathResolution::PreserveMutableComponents,
            RootMetadataWriteMounts::Separate,
        )
    }

    fn get_writable_roots_with_cwd_impl(
        &self,
        cwd: &Path,
        path_resolution: WritableRootPathResolution,
        root_metadata_mounts: RootMetadataWriteMounts,
    ) -> Vec<WritableRoot> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        if self.has_full_disk_write_access() {
            return Vec::new();
        }

        let resolved_entries = self.resolved_entries_with_cwd(cwd);
        if !resolved_entries
            .iter()
            .any(|entry| entry.access.can_write())
        {
            return Vec::new();
        }
        let Some(local_context) = LocalPolicyContext::new(cwd) else {
            return Vec::new();
        };
        let Ok(matching) = self.prepare_local_matching(&local_context.as_context()) else {
            return Vec::new();
        };
        // Resolve precedence and filesystem aliases once per entry, rather than repeating
        // that work for every writable root while collecting its read-only carveouts.
        let effective_entries: Vec<&ResolvedFileSystemEntry> = resolved_entries
            .iter()
            .filter(|entry| {
                entry.access.can_write()
                    == matching
                        .can_write_path(&PathUri::from_abs_path(&entry.path))
                        .unwrap_or(false)
            })
            .collect();
        if !effective_entries
            .iter()
            .any(|entry| entry.access.can_write())
        {
            return Vec::new();
        }
        // Include resolved gitdirs in the entries used to carve out broader grants.
        // Seatbelt grants are independent, and later bubblewrap binds cover earlier mounts.
        let include_resolved_gitdirs = match path_resolution {
            WritableRootPathResolution::Effective => cfg!(target_os = "linux"),
            WritableRootPathResolution::PreserveMutableComponents => cfg!(target_os = "macos"),
        };
        let resolved_gitdir_entries: Vec<ResolvedFileSystemEntry> = if include_resolved_gitdirs {
            effective_entries
                .iter()
                .filter(|entry| entry.access.can_write())
                .filter_map(|entry| {
                    let dot_git = path_resolution
                        .resolve(entry.path.clone())
                        .join(PROTECTED_METADATA_GIT_PATH_NAME);
                    is_git_pointer_file(&dot_git)
                        .then(|| resolve_gitdir_from_file(&dot_git))
                        .flatten()
                })
                .filter(|path| !has_explicit_resolved_path_entry(&resolved_entries, path))
                .map(|path| ResolvedFileSystemEntry {
                    path,
                    access: FileSystemAccessMode::Read,
                })
                .collect()
        } else {
            Vec::new()
        };
        let prepared_entries: Vec<PreparedFileSystemEntry<'_>> = effective_entries
            .into_iter()
            .map(|entry| PreparedFileSystemEntry {
                entry,
                effective_path: path_resolution.resolve(entry.path.clone()),
            })
            .chain(resolved_gitdir_entries.iter().map(|entry| {
                PreparedFileSystemEntry {
                    entry,
                    // Resolve the exclusion target while retaining the logical entry path.
                    effective_path: entry
                        .path
                        .as_path()
                        .canonicalize()
                        .ok()
                        .and_then(|path| AbsolutePathBuf::from_absolute_path(path).ok())
                        .unwrap_or_else(|| {
                            WritableRootPathResolution::Effective.resolve(entry.path.clone())
                        }),
                }
            }))
            .collect();
        let inherits_root_metadata = matches!(
            root_metadata_mounts,
            RootMetadataWriteMounts::InheritWritableRoot
        ) && prepared_entries.iter().any(|entry| {
            entry.entry.access.can_write() && entry.entry.path.as_path().parent().is_none()
        });
        let writable_entries: Vec<&PreparedFileSystemEntry<'_>> = prepared_entries
            .iter()
            .filter(|entry| entry.entry.access.can_write())
            .filter(|entry| {
                let path = entry.entry.path.as_path();
                !inherits_root_metadata
                    || path.parent().is_none_or(|parent| parent.parent().is_some())
                    || !path.file_name().is_some_and(is_protected_metadata_name)
            })
            .collect();

        let effective_cwd = AbsolutePathBuf::from_absolute_path(cwd)
            .ok()
            .map(|cwd| path_resolution.resolve(cwd));

        dedup_absolute_paths(
            writable_entries
                .iter()
                .map(|entry| entry.effective_path.clone())
                .collect(),
            /*normalize_effective_paths*/ false,
        )
        .into_iter()
        .map(|root| {
            // Filesystem-root policies stay in their effective canonical form
            // so root-wide aliases do not create duplicate top-level masks.
            // Example: keep `/var/...` normalized under `/` instead of
            // materializing both `/var/...` and `/private/var/...`.
            // Nested symlink paths under a writable root stay logical so
            // downstream sandboxes can still bind the real target while
            // masking the user-visible symlink inode when needed.
            let preserve_raw_carveout_paths = root.as_path().parent().is_some();
            let raw_writable_roots: Vec<&AbsolutePathBuf> = writable_entries
                .iter()
                .filter(|entry| entry.effective_path == root)
                .map(|entry| &entry.entry.path)
                .collect();
            let protected_metadata_names =
                protected_metadata_names_for_writable_root(&matching, &root, &raw_writable_roots);
            let protect_missing_dot_codex = effective_cwd.as_ref() == Some(&root);
            let mut read_only_subpaths: Vec<AbsolutePathBuf> =
                default_read_only_subpaths_for_writable_root(&root, protect_missing_dot_codex)
                    .into_iter()
                    .filter(|path| !has_explicit_resolved_path_entry(&resolved_entries, path))
                    .collect();
            // Narrower explicit non-write entries carve out broader writable roots.
            // More specific write entries still remain writable because they appear
            // as separate WritableRoot values and are checked independently.
            // Preserve symlink path components that live under the writable root
            // so downstream sandboxes can still mask the symlink inode itself.
            // Example: if `<root>/.codex -> <root>/decoy`, bwrap must still see
            // `<root>/.codex`, not only the resolved `<root>/decoy`.
            read_only_subpaths.extend(
                prepared_entries
                    .iter()
                    .filter(|entry| !entry.entry.access.can_write())
                    .filter_map(|prepared| {
                        let entry = prepared.entry;
                        let effective_path = &prepared.effective_path;
                        // Preserve the literal in-root path whenever the
                        // carveout itself lives under this writable root, even
                        // if following symlinks would resolve back to the root
                        // or escape outside it. Downstream sandboxes need that
                        // raw path so they can mask the symlink inode itself.
                        // Examples:
                        // - `<root>/linked-private -> <root>/decoy-private`
                        // - `<root>/linked-private -> /tmp/outside-private`
                        // - `<root>/alias-root -> <root>`
                        let raw_carveout_path = if preserve_raw_carveout_paths {
                            if entry.path == root {
                                None
                            } else if entry.path.as_path().starts_with(root.as_path()) {
                                Some(entry.path.clone())
                            } else {
                                raw_writable_roots.iter().find_map(|raw_root| {
                                    let suffix = entry
                                        .path
                                        .as_path()
                                        .strip_prefix(raw_root.as_path())
                                        .ok()?;
                                    if suffix.as_os_str().is_empty() {
                                        return None;
                                    }
                                    Some(root.join(suffix))
                                })
                            }
                        } else {
                            None
                        };

                        if let Some(raw_carveout_path) = raw_carveout_path {
                            return Some(raw_carveout_path);
                        }

                        if effective_path == &root
                            || !effective_path.as_path().starts_with(root.as_path())
                        {
                            return None;
                        }

                        Some(effective_path.clone())
                    }),
            );
            WritableRoot {
                protected_metadata_names,
                root,
                // Preserve literal in-root protected paths like `.git` and
                // `.codex` so downstream sandboxes can still detect and mask
                // the symlink itself instead of only its resolved target.
                read_only_subpaths: dedup_absolute_paths(
                    read_only_subpaths,
                    /*normalize_effective_paths*/ false,
                ),
            }
        })
        .collect()
    }

    /// Returns explicit unreadable roots resolved against the provided cwd.
    pub fn get_unreadable_roots_with_cwd(&self, cwd: &Path) -> Vec<AbsolutePathBuf> {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return Vec::new();
        }

        let root = AbsolutePathBuf::from_absolute_path(cwd)
            .ok()
            .map(|cwd| absolute_root_path_for_cwd(&cwd));
        let local_context = LocalPolicyContext::new(cwd);
        let matching = local_context
            .as_ref()
            .and_then(|context| self.prepare_local_matching(&context.as_context()).ok());

        dedup_absolute_paths(
            self.resolved_entries_with_cwd(cwd)
                .iter()
                .filter(|entry| entry.access == FileSystemAccessMode::Deny)
                .filter(|entry| {
                    !matching.as_ref().is_some_and(|matching| {
                        matching
                            .resolve_access(&PathUri::from_abs_path(&entry.path))
                            .can_read()
                    })
                })
                // Restricted policies already deny reads outside explicit allow roots,
                // so materializing the filesystem root here would erase narrower
                // readable carveouts when downstream sandboxes apply deny masks last.
                .filter(|entry| root.as_ref() != Some(&entry.path))
                .map(|entry| entry.path.clone())
                .collect(),
            /*normalize_effective_paths*/ true,
        )
    }

    /// Includes literal denies alongside their resolved targets so Linux can reject a denial
    /// through a writable symlink instead of only masking the symlink's current target.
    pub fn get_unreadable_roots_with_cwd_preserving_symlinks(
        &self,
        cwd: &Path,
    ) -> Vec<AbsolutePathBuf> {
        let mut roots = self.get_unreadable_roots_with_cwd(cwd);
        if matches!(self.kind, FileSystemSandboxKind::Restricted) {
            for entry in self.resolved_entries_with_cwd(cwd) {
                if entry.access == FileSystemAccessMode::Deny
                    && entry.path.as_path().parent().is_some()
                    && !self.can_read_local_path_with_cwd(entry.path.as_path(), cwd)
                    && !roots.contains(&entry.path)
                {
                    roots.push(entry.path);
                }
            }
        }
        roots
    }

    /// Returns unreadable glob patterns resolved against the provided cwd.
    pub fn get_unreadable_globs_with_cwd(&self, cwd: &Path) -> Vec<String> {
        let mut patterns: Vec<String> = self
            .get_unreadable_glob_scans_with_cwd(cwd)
            .into_iter()
            .map(|glob| glob.pattern)
            .collect();
        patterns.sort();
        patterns.dedup();
        patterns
    }

    /// Keeps each scan's provenance: an identical generic and generated
    /// database pattern must remain distinct when Linux expands them.
    pub fn get_unreadable_glob_scans_with_cwd(&self, cwd: &Path) -> Vec<FileSystemDenyGlob> {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return Vec::new();
        }
        let mut patterns = self
            .entries
            .iter()
            .filter(|entry| entry.access == FileSystemAccessMode::Deny)
            .filter_map(|entry| match &entry.path {
                FileSystemPath::GlobPattern { pattern, scan } => Some(FileSystemDenyGlob {
                    pattern: AbsolutePathBuf::resolve_path_against_base(pattern, cwd)
                        .to_string_lossy()
                        .into_owned(),
                    scan: scan.clone(),
                }),
                FileSystemPath::Path { .. } | FileSystemPath::Special { .. } => None,
            })
            .collect::<Vec<_>>();
        patterns.sort();
        patterns.dedup();
        patterns
    }

<<<<<<< HEAD
    /// Directories agent commands may not list (issue #506), from
    /// [`FileSystemSandboxEntry::deny_directory_listing`] entries. Only a
    /// restricted policy carries them; what is below each directory keeps its
    /// own access.
    pub fn get_unlistable_directories(&self) -> Vec<AbsolutePathBuf> {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return Vec::new();
        }
        dedup_absolute_paths(
            self.entries
                .iter()
                .filter_map(FileSystemSandboxEntry::unlistable_directory)
                .collect(),
            /*normalize_effective_paths*/ true,
        )
    }

    /// The explicit read-only paths resolved against `cwd`: entries that
    /// neither write nor deny reads, and that no more specific entry makes
    /// writable. Unlike [`WritableRoot::read_only_subpaths`] these are not
    /// tied to a writable root, nor are the default `.git`/`.codex`/`.agents`
    /// carve-outs counted.
    pub fn get_read_only_paths_with_cwd(&self, cwd: &Path) -> Vec<AbsolutePathBuf> {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return Vec::new();
        }
        let root = AbsolutePathBuf::from_absolute_path(cwd)
            .ok()
            .map(|cwd| absolute_root_path_for_cwd(&cwd));
        dedup_absolute_paths(
            self.resolved_entries_with_cwd(cwd)
                .into_iter()
                .filter(|entry| entry.access == FileSystemAccessMode::Read)
                .filter(|entry| root.as_ref() != Some(&entry.path))
                .filter(|entry| !self.can_write_path_with_cwd(entry.path.as_path(), cwd))
                .map(|entry| entry.path)
                .collect(),
            /*normalize_effective_paths*/ false,
        )
    }

    /// Paths agent commands may not move aside (issues #594, #598), from
    /// [`FileSystemSandboxEntry::deny_moving`] entries, as written. Only a
    /// restricted policy carries them.
    pub fn get_unmovable_paths(&self) -> Vec<AbsolutePathBuf> {
        if !matches!(self.kind, FileSystemSandboxKind::Restricted) {
            return Vec::new();
        }
        dedup_absolute_paths(
            self.entries
                .iter()
                .filter_map(FileSystemSandboxEntry::unmovable_path)
                .collect(),
            /*normalize_effective_paths*/ false,
        )
    }

    /// Whether agent commands are kept away from the macOS keychain services
    /// (issue #758), from a [`FileSystemSandboxEntry::deny_keychain_services`]
    /// entry. Only a restricted policy carries it.
    pub fn denies_keychain_services(&self) -> bool {
        matches!(self.kind, FileSystemSandboxKind::Restricted)
            && self
                .entries
                .iter()
                .any(FileSystemSandboxEntry::denies_keychain_services)
    }

    /// Whether agent commands may not create user namespaces (issue #826),
    /// from a [`FileSystemSandboxEntry::deny_user_namespaces`] entry. Only a
    /// restricted policy carries it.
    pub fn denies_user_namespaces(&self) -> bool {
        matches!(self.kind, FileSystemSandboxKind::Restricted)
            && self
                .entries
                .iter()
                .any(FileSystemSandboxEntry::denies_user_namespaces)
||||||| 413492cd6c
=======
    /// Replace deny globs with a backend's concrete snapshot, preserving other entries.
    /// An empty snapshot still removes the original patterns.
    pub fn with_expanded_deny_globs(mut self, paths: Vec<AbsolutePathBuf>) -> Self {
        self.entries
            .retain(|entry| !matches!(entry.path, FileSystemPath::GlobPattern { .. }));
        self.entries.extend(
            paths
                .into_iter()
                .map(|path| FileSystemSandboxEntry::new(path.into(), FileSystemAccessMode::Deny)),
        );
        self
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    pub fn to_legacy_sandbox_policy(
        &self,
        network_policy: NetworkSandboxPolicy,
        cwd: &Path,
    ) -> io::Result<SandboxPolicy> {
        Ok(match self.kind {
            FileSystemSandboxKind::ExternalSandbox => SandboxPolicy::ExternalSandbox {
                network_access: if network_policy.is_enabled() {
                    NetworkAccess::Enabled
                } else {
                    NetworkAccess::Restricted
                },
            },
            FileSystemSandboxKind::Unrestricted => {
                if network_policy.is_enabled() {
                    SandboxPolicy::DangerFullAccess
                } else {
                    SandboxPolicy::ExternalSandbox {
                        network_access: NetworkAccess::Restricted,
                    }
                }
            }
            FileSystemSandboxKind::Restricted => {
                let cwd_absolute = AbsolutePathBuf::from_absolute_path(cwd).ok();
                let has_full_disk_write_access = self.has_full_disk_write_access();
                let mut workspace_root_writable = false;
                let mut writable_roots = Vec::new();
                let mut tmpdir_writable = false;
                let mut slash_tmp_writable = false;
                let mut unbridgeable_root_write = false;

                for entry in &self.entries {
                    match &entry.path {
                        FileSystemPath::GlobPattern { .. } => {}
                        FileSystemPath::Path { path } => {
                            if entry.access.can_write() {
                                let path = path.to_abs_path()?;
                                if cwd_absolute.as_ref().is_some_and(|cwd| cwd == &path) {
                                    workspace_root_writable = true;
                                } else {
                                    writable_roots.push(path);
                                }
                            }
                        }
                        FileSystemPath::Special { value } => match value {
                            FileSystemSpecialPath::Root => match entry.access {
                                FileSystemAccessMode::Deny => {}
                                FileSystemAccessMode::Read => {}
                                FileSystemAccessMode::Write => {
                                    unbridgeable_root_write = true;
                                }
                            },
                            FileSystemSpecialPath::Minimal => {}
                            FileSystemSpecialPath::ProjectRoots { subpath } => {
                                if subpath.is_none() && entry.access.can_write() {
                                    workspace_root_writable = true;
                                } else if let Some(path) =
                                    resolve_file_system_special_path(value, cwd_absolute.as_ref())
                                    && entry.access.can_write()
                                {
                                    writable_roots.push(path);
                                }
                            }
                            FileSystemSpecialPath::Tmpdir => {
                                if entry.access.can_write() {
                                    tmpdir_writable = true;
                                }
                            }
                            FileSystemSpecialPath::SlashTmp => {
                                if entry.access.can_write() {
                                    slash_tmp_writable = true;
                                }
                            }
                            FileSystemSpecialPath::Unknown { .. } => {}
                        },
                    }
                }

                if has_full_disk_write_access {
                    return Ok(if network_policy.is_enabled() {
                        SandboxPolicy::DangerFullAccess
                    } else {
                        SandboxPolicy::ExternalSandbox {
                            network_access: NetworkAccess::Restricted,
                        }
                    });
                }

                if workspace_root_writable {
                    SandboxPolicy::WorkspaceWrite {
                        writable_roots: dedup_absolute_paths(
                            writable_roots,
                            /*normalize_effective_paths*/ false,
                        ),
                        network_access: network_policy.is_enabled(),
                        exclude_tmpdir_env_var: !tmpdir_writable,
                        exclude_slash_tmp: !slash_tmp_writable,
                    }
                } else if unbridgeable_root_write
                    || !writable_roots.is_empty()
                    || tmpdir_writable
                    || (cfg!(unix) && slash_tmp_writable)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "permissions profile requests filesystem writes outside the workspace root, which is not supported until the runtime enforces FileSystemSandboxPolicy directly",
                    ));
                } else {
                    SandboxPolicy::ReadOnly {
                        network_access: network_policy.is_enabled(),
                    }
                }
            }
        })
    }

    fn resolved_entries_with_cwd(&self, cwd: &Path) -> Vec<ResolvedFileSystemEntry> {
        let cwd_absolute = AbsolutePathBuf::from_absolute_path(cwd).ok();
        self.entries
            .iter()
            .filter_map(|entry| {
                resolve_entry_path(&entry.path, cwd_absolute.as_ref()).map(|path| {
                    ResolvedFileSystemEntry {
                        path,
                        access: entry.access,
                    }
                })
            })
            .collect()
    }

    /// Resolves configured roots using executor paths without inspecting the filesystem.
    /// Glob patterns are excluded; access precedence is evaluated by `resolve_access`.
    pub fn resolved_entries(
        &self,
        context: &FileSystemSandboxPolicyContext<'_>,
    ) -> Vec<(PathUri, FileSystemAccessMode)> {
        let convention = context.cwd.infer_path_convention();
        self.entries
            .iter()
            .flat_map(|entry| {
                let paths = match &entry.path {
                    FileSystemPath::Path { path } => vec![path.clone()],
                    FileSystemPath::GlobPattern { .. } => Vec::new(),
                    FileSystemPath::Special { value } => match value {
                        FileSystemSpecialPath::Root => {
                            file_system_root(context).into_iter().collect()
                        }
                        FileSystemSpecialPath::ProjectRoots { subpath } => context
                            .workspace_roots
                            .iter()
                            .filter_map(|root| match subpath {
                                Some(subpath) => root.join(subpath).ok(),
                                None => Some(root.clone()),
                            })
                            .collect(),
                        FileSystemSpecialPath::Tmpdir => {
                            context.temporary_directories.unwrap_or_default().to_vec()
                        }
                        FileSystemSpecialPath::SlashTmp
                            if convention == Some(PathConvention::Posix) =>
                        {
                            context.cwd.join("/tmp").into_iter().collect()
                        }
                        FileSystemSpecialPath::SlashTmp
                        | FileSystemSpecialPath::Minimal
                        | FileSystemSpecialPath::Unknown { .. } => Vec::new(),
                    },
                };
                paths
                    .into_iter()
                    .filter(move |path| path.infer_path_convention() == convention)
                    .map(move |path| (path, entry.access))
            })
            .collect()
    }

    fn semantic_signature(&self, cwd: &Path) -> FileSystemSemanticSignature {
        FileSystemSemanticSignature {
            has_full_disk_read_access: self.has_full_disk_read_access(),
            has_full_disk_write_access: self.has_full_disk_write_access(),
            include_platform_defaults: self.include_platform_defaults(),
            readable_roots: sorted_absolute_paths(self.get_readable_roots_with_cwd(cwd)),
            writable_roots: sorted_writable_roots(self.get_writable_roots_with_cwd(cwd)),
            unreadable_roots: sorted_absolute_paths(self.get_unreadable_roots_with_cwd(cwd)),
            unreadable_globs: self.get_unreadable_glob_scans_with_cwd(cwd),
            unlistable_directories: sorted_absolute_paths(self.get_unlistable_directories()),
            unmovable_paths: sorted_absolute_paths(self.get_unmovable_paths()),
            keychain_services_denied: self.denies_keychain_services(),
            user_namespaces_denied: self.denies_user_namespaces(),
        }
    }
}

impl From<&SandboxPolicy> for NetworkSandboxPolicy {
    fn from(value: &SandboxPolicy) -> Self {
        if value.has_full_network_access() {
            NetworkSandboxPolicy::Enabled
        } else {
            NetworkSandboxPolicy::Restricted
        }
    }
}

impl From<&SandboxPolicy> for FileSystemSandboxPolicy {
    fn from(value: &SandboxPolicy) -> Self {
        match value {
            SandboxPolicy::DangerFullAccess => FileSystemSandboxPolicy::unrestricted(),
            SandboxPolicy::ExternalSandbox { .. } => FileSystemSandboxPolicy::external_sandbox(),
            SandboxPolicy::ReadOnly { .. } => {
                FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    FileSystemAccessMode::Read,
                )])
            }
            SandboxPolicy::WorkspaceWrite {
                writable_roots,
                exclude_tmpdir_env_var,
                exclude_slash_tmp,
                ..
            } => FileSystemSandboxPolicy::workspace_write(
                writable_roots,
                *exclude_tmpdir_env_var,
                *exclude_slash_tmp,
            ),
        }
    }
}

fn resolve_file_system_path(
    path: &FileSystemPath,
    cwd: Option<&AbsolutePathBuf>,
) -> Option<AbsolutePathBuf> {
    match path {
        FileSystemPath::Path { path } => path.to_abs_path().ok(),
        FileSystemPath::GlobPattern { .. } => None,
        FileSystemPath::Special { value } => resolve_file_system_special_path(value, cwd),
    }
}

fn resolve_entry_path(
    path: &FileSystemPath,
    cwd: Option<&AbsolutePathBuf>,
) -> Option<AbsolutePathBuf> {
    match path {
        FileSystemPath::Special {
            value: FileSystemSpecialPath::Root,
        } => cwd.map(absolute_root_path_for_cwd),
        _ => resolve_file_system_path(path, cwd),
    }
}

fn resolve_candidate_path(path: &Path, cwd: &Path) -> Option<AbsolutePathBuf> {
    if path.is_absolute() {
        AbsolutePathBuf::from_absolute_path(path).ok()
    } else {
        Some(AbsolutePathBuf::from_absolute_path(cwd).ok()?.join(path))
    }
}

/// Resolves a workspace-relative path using the root's own path convention.
///
/// Rejects absolute paths, traversal, Windows drive changes, and any result
/// outside the root so foreign-platform permission rules cannot escape scope.
fn resolve_scoped_workspace_path(root: &PathUri, subpath: &str) -> Option<PathUri> {
    let convention = root.infer_path_convention()?;
    if subpath.starts_with('/')
        || convention == PathConvention::Windows && subpath.starts_with('\\')
        || convention
            .path_segments(subpath)
            .any(|segment| segment == "." || segment == "..")
        || convention == PathConvention::Windows
            && convention
                .path_segments(subpath)
                .any(|segment| segment.contains(':'))
    {
        return None;
    }
    let path = root.join(subpath).ok()?;
    path.starts_with(root).then_some(path)
}

fn with_local_policy_context<T>(
    path: &Path,
    cwd: &Path,
    evaluate: impl FnOnce(&PathUri, &FileSystemSandboxPolicyContext<'_>) -> T,
) -> Option<T> {
    let cwd = AbsolutePathBuf::from_absolute_path(cwd).ok()?;
    let path = PathUri::from(resolve_candidate_path(path, cwd.as_path())?);
    let context = LocalPolicyContext::new(cwd.as_path())?;
    Some(evaluate(&path, &context.as_context()))
}

pub fn file_system_root(context: &FileSystemSandboxPolicyContext<'_>) -> Option<PathUri> {
    context.cwd.lexical_depth()?;
    context.cwd.ancestors().last()
}

fn local_temporary_directories() -> Vec<PathUri> {
    let Some(tmpdir) = std::env::var_os("TMPDIR").filter(|path| !path.is_empty()) else {
        return Vec::new();
    };
    AbsolutePathBuf::from_absolute_path(PathBuf::from(tmpdir))
        .ok()
        .map(PathUri::from)
        .into_iter()
        .collect()
}

/// Returns true when two config paths refer to the same exact target before
/// any prefix matching is applied.
///
/// This is intentionally narrower than full path resolution: it only answers
/// the "can one entry shadow another at the same specificity?" question used
/// by `has_write_narrowing_entries`.
fn file_system_paths_share_target(
    left: &FileSystemPath,
    right: &FileSystemPath,
    convention: PathConvention,
) -> bool {
    match (left, right) {
        (FileSystemPath::Path { path: left }, FileSystemPath::Path { path: right }) => {
            left == right
        }
        (FileSystemPath::Special { value: left }, FileSystemPath::Special { value: right }) => {
            special_paths_share_target(left, right)
        }
        (FileSystemPath::Path { path }, FileSystemPath::Special { value })
        | (FileSystemPath::Special { value }, FileSystemPath::Path { path }) => {
            path.infer_path_convention() == Some(convention)
                && special_path_matches_path_uri(value, path)
        }
        (
            FileSystemPath::GlobPattern {
                pattern: left,
                scan: left_scan,
            },
            FileSystemPath::GlobPattern {
                pattern: right,
                scan: right_scan,
            },
        ) => left == right && left_scan == right_scan,
        (FileSystemPath::GlobPattern { .. }, _) | (_, FileSystemPath::GlobPattern { .. }) => false,
    }
}

/// Compares special-path tokens that resolve to the same concrete target
/// without needing a cwd.
fn special_paths_share_target(left: &FileSystemSpecialPath, right: &FileSystemSpecialPath) -> bool {
    match (left, right) {
        (FileSystemSpecialPath::Root, FileSystemSpecialPath::Root)
        | (FileSystemSpecialPath::Minimal, FileSystemSpecialPath::Minimal)
        | (FileSystemSpecialPath::Tmpdir, FileSystemSpecialPath::Tmpdir)
        | (FileSystemSpecialPath::SlashTmp, FileSystemSpecialPath::SlashTmp) => true,
        (
            FileSystemSpecialPath::ProjectRoots { subpath: left },
            FileSystemSpecialPath::ProjectRoots { subpath: right },
        ) => left == right,
        (
            FileSystemSpecialPath::Unknown {
                path: left,
                subpath: left_subpath,
            },
            FileSystemSpecialPath::Unknown {
                path: right,
                subpath: right_subpath,
            },
        ) => left == right && left_subpath == right_subpath,
        _ => false,
    }
}

/// Matches cwd-independent special paths against `PathUri` entries when
/// they name the same location.
///
/// We intentionally only fold the special paths whose concrete meaning is
/// stable without a cwd, such as `/` and `/tmp`.
fn special_path_matches_path_uri(value: &FileSystemSpecialPath, path: &PathUri) -> bool {
    match value {
        FileSystemSpecialPath::Root => path.lexical_depth().is_some() && path.parent().is_none(),
        FileSystemSpecialPath::SlashTmp => {
            path.infer_path_convention() == Some(PathConvention::Posix)
                && path.lexical_depth() == Some(1)
                && path.basename().as_deref() == Some("tmp")
        }
        _ => false,
    }
}

fn absolute_root_path_for_cwd(cwd: &AbsolutePathBuf) -> AbsolutePathBuf {
    let root = cwd
        .as_path()
        .ancestors()
        .last()
        .unwrap_or_else(|| panic!("cwd must have a filesystem root"));
    AbsolutePathBuf::from_absolute_path(root)
        .unwrap_or_else(|err| panic!("cwd root must be an absolute path: {err}"))
}

<<<<<<< HEAD
fn normalized_and_canonical_candidates(path: &Path) -> Vec<PathBuf> {
    // Compare the lexical absolute form plus the canonical target when it
    // exists. Missing paths still need the lexical candidate so future-created
    // denied paths remain blocked by direct tool checks.
    let mut candidates = Vec::new();

    if let Ok(normalized) = AbsolutePathBuf::from_absolute_path(path) {
        push_unique(&mut candidates, normalized.to_path_buf());
    } else {
        push_unique(&mut candidates, path.to_path_buf());
    }

    if let Ok(canonical) = path.canonicalize()
        && let Ok(canonical_absolute) = AbsolutePathBuf::from_absolute_path(canonical)
    {
        push_unique(&mut candidates, canonical_absolute.to_path_buf());
    }

    candidates
}

fn push_unique(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

fn build_glob_matcher(pattern: &str, case_insensitive: bool) -> Result<GlobMatcher, String> {
||||||| 413492cd6c
fn normalized_and_canonical_candidates(path: &Path) -> Vec<PathBuf> {
    // Compare the lexical absolute form plus the canonical target when it
    // exists. Missing paths still need the lexical candidate so future-created
    // denied paths remain blocked by direct tool checks.
    let mut candidates = Vec::new();

    if let Ok(normalized) = AbsolutePathBuf::from_absolute_path(path) {
        push_unique(&mut candidates, normalized.to_path_buf());
    } else {
        push_unique(&mut candidates, path.to_path_buf());
    }

    if let Ok(canonical) = path.canonicalize()
        && let Ok(canonical_absolute) = AbsolutePathBuf::from_absolute_path(canonical)
    {
        push_unique(&mut candidates, canonical_absolute.to_path_buf());
    }

    candidates
}

fn push_unique(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
}

fn build_glob_matcher(pattern: &str) -> Result<GlobMatcher, String> {
=======
fn build_glob_matcher(pattern: &str, convention: PathConvention) -> Result<GlobMatcher, String> {
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    // Keep `*` and `?` within a single path component and preserve an unclosed
    // `[` as a literal so matcher behavior stays aligned with config parsing.
    let pattern = if convention == PathConvention::Windows {
        pattern.replace('\\', "/")
    } else {
        pattern.to_string()
    };
    GlobBuilder::new(&pattern)
        .literal_separator(true)
        .allow_unclosed_class(true)
<<<<<<< HEAD
        .case_insensitive(case_insensitive)
||||||| 413492cd6c
=======
        .backslash_escape(convention == PathConvention::Posix)
        .case_insensitive(convention == PathConvention::Windows)
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        .build()
        .map(|glob| glob.compile_matcher())
        .map_err(|err| err.to_string())
}

fn resolve_file_system_special_path(
    value: &FileSystemSpecialPath,
    cwd: Option<&AbsolutePathBuf>,
) -> Option<AbsolutePathBuf> {
    match value {
        FileSystemSpecialPath::Root
        | FileSystemSpecialPath::Minimal
        | FileSystemSpecialPath::Unknown { .. } => None,
        FileSystemSpecialPath::ProjectRoots { subpath } => {
            let cwd = cwd?;
            match subpath.as_ref() {
                Some(subpath) => Some(AbsolutePathBuf::resolve_path_against_base(
                    subpath,
                    cwd.as_path(),
                )),
                None => Some(cwd.clone()),
            }
        }
        FileSystemSpecialPath::Tmpdir => {
            let tmpdir = std::env::var_os("TMPDIR")?;
            if tmpdir.is_empty() {
                None
            } else {
                let tmpdir = AbsolutePathBuf::from_absolute_path(PathBuf::from(tmpdir)).ok()?;
                Some(tmpdir)
            }
        }
        FileSystemSpecialPath::SlashTmp => {
            if !cfg!(unix) {
                return None;
            }
            #[allow(clippy::expect_used)]
            let slash_tmp = AbsolutePathBuf::from_absolute_path("/tmp").expect("/tmp is absolute");
            if !slash_tmp.as_path().is_dir() {
                return None;
            }
            Some(slash_tmp)
        }
    }
}

fn dedup_absolute_paths(
    paths: Vec<AbsolutePathBuf>,
    normalize_effective_paths: bool,
) -> Vec<AbsolutePathBuf> {
    let mut deduped = Vec::with_capacity(paths.len());
    let mut seen = HashSet::new();
    for path in paths {
        let dedup_path = if normalize_effective_paths {
            normalize_effective_absolute_path(path)
        } else {
            path
        };
        if seen.insert(dedup_path.to_path_buf()) {
            deduped.push(dedup_path);
        }
    }
    deduped
}

/// Issue #535: a writable root left out because it is a link into another
/// root's read-only path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritableLinkIntoReadOnlyPath {
    /// The writable root as the policy resolves it (still spelled through
    /// the link).
    pub link: AbsolutePathBuf,
    /// The read-only path of another writable root it reaches.
    pub read_only_path: AbsolutePathBuf,
}

/// Issue #536: a writable root with the real location it had when the
/// session first saw it. Sandboxes grant writes there, not wherever the root
/// leads by the time they are set up, so a link swapped in meanwhile grants
/// nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedWritableRoot {
    /// The writable root as [`FileSystemSandboxPolicy::get_writable_roots_with_cwd`]
    /// resolves it.
    pub root: AbsolutePathBuf,
    /// Its real location (see [`real_location`]).
    pub real: PathBuf,
}

/// Issue #535: the roots (by index) reached through a link whose real
/// location is, or lies inside, a read-only path of another root (protected
/// metadata names count, also not created yet), with that path. A root
/// spelled through the read-only path itself (an explicit `.git/hooks`
/// root) and a root without a link below its top-level folder (`/tmp` and
/// `/var` on macOS are top-level aliases) stay grants.
#[cfg(unix)]
fn links_into_read_only_paths(roots: &[WritableRoot]) -> Vec<(usize, AbsolutePathBuf)> {
    let mut excluded = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        if !passes_through_nested_link(root.root.as_path()) {
            continue;
        }
        let real_root = real_location(root.root.as_path());
        let reached = roots
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .flat_map(|(_, other)| {
                other.read_only_subpaths.iter().cloned().chain(
                    other
                        .protected_metadata_names
                        .iter()
                        .map(|name| other.root.join(name)),
                )
            })
            .find(|read_only| {
                !root.root.as_path().starts_with(read_only.as_path())
                    && location_within(&real_root, &real_location(read_only.as_path()))
            });
        if let Some(read_only) = reached {
            excluded.push((index, read_only));
        }
    }
    excluded
}

#[cfg(not(unix))]
fn links_into_read_only_paths(_roots: &[WritableRoot]) -> Vec<(usize, AbsolutePathBuf)> {
    Vec::new()
}

/// Whether a folder of `path` below its top-level one is a link.
#[cfg(unix)]
fn passes_through_nested_link(path: &Path) -> bool {
    path.ancestors()
        .filter(|ancestor| ancestor.parent().and_then(Path::parent).is_some())
        .any(|ancestor| {
            std::fs::symlink_metadata(ancestor)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        })
}

/// Issue #536: the links on `path` below its top-level folder, shallowest
/// first (`/tmp` and `/var` on macOS are top-level aliases).
#[cfg(unix)]
pub fn nested_links(path: &Path) -> Vec<PathBuf> {
    let mut links: Vec<PathBuf> = path
        .ancestors()
        .filter(|ancestor| ancestor.parent().and_then(Path::parent).is_some())
        .filter(|ancestor| {
            std::fs::symlink_metadata(ancestor)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        })
        .map(Path::to_path_buf)
        .collect();
    links.reverse();
    links
}

/// Issue #536: the link on `path` that no longer leads where it led when
/// `path` really was at `was`: for each link on the path (shallowest first),
/// where it should lead is `was` without the part of `path` below the link.
/// The deepest link when none can be told apart, or `path` itself when it
/// has no link.
#[cfg(unix)]
pub fn changed_link(path: &Path, was: &Path) -> PathBuf {
    let links = nested_links(path);
    links
        .iter()
        .find(|link| {
            let below = path
                .strip_prefix(link)
                .map(|below| below.components().count())
                .unwrap_or(0);
            let mut expected = was;
            for _ in 0..below {
                match expected.parent() {
                    Some(parent) => expected = parent,
                    None => return true,
                }
            }
            real_location(link) != expected
        })
        .or(links.last())
        .cloned()
        .unwrap_or_else(|| path.to_path_buf())
}

/// Where `path` really is: every link followed, including a link whose
/// target does not exist yet, and the components that do not exist kept as
/// written. Unlike `canonicalize`, this also places a path not created yet.
#[cfg(unix)]
pub fn real_location(path: &Path) -> PathBuf {
    use std::collections::VecDeque;
    use std::path::Component;

    const MAX_LINKS: usize = 40;
    let mut pending: VecDeque<std::ffi::OsString> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_os_string()),
            Component::ParentDir => Some("..".into()),
            Component::RootDir | Component::CurDir | Component::Prefix(_) => None,
        })
        .collect();
    let mut resolved = PathBuf::from("/");
    let mut links = 0;
    while let Some(name) = pending.pop_front() {
        if name == ".." {
            resolved.pop();
            continue;
        }
        let candidate = resolved.join(&name);
        let target = std::fs::symlink_metadata(&candidate)
            .ok()
            .filter(|metadata| metadata.file_type().is_symlink() && links < MAX_LINKS)
            .and_then(|_| std::fs::read_link(&candidate).ok());
        let Some(target) = target else {
            resolved = candidate;
            continue;
        };
        links += 1;
        if target.is_absolute() {
            resolved = PathBuf::from("/");
        }
        for component in target.components().rev() {
            match component {
                Component::Normal(name) => pending.push_front(name.to_os_string()),
                Component::ParentDir => pending.push_front("..".into()),
                Component::RootDir | Component::CurDir | Component::Prefix(_) => {}
            }
        }
    }
    canonical_existing_prefix(resolved)
}

/// `path` (links already followed) with its deepest existing folder in the
/// form the filesystem stores: on macOS, whose default filesystem ignores
/// case and Unicode form, `<ws>/.GIT` is `<ws>/.git`. The missing rest is
/// kept as written.
#[cfg(unix)]
fn canonical_existing_prefix(path: PathBuf) -> PathBuf {
    let mut existing = path.as_path();
    let mut missing = Vec::new();
    while std::fs::symlink_metadata(existing).is_err() {
        let (Some(parent), Some(name)) = (existing.parent(), existing.file_name()) else {
            return path;
        };
        missing.push(name.to_os_string());
        existing = parent;
    }
    let Ok(mut canonical) = std::fs::canonicalize(existing) else {
        return path;
    };
    for name in missing.into_iter().rev() {
        canonical.push(name);
    }
    canonical
}

/// Whether real location `path` is `prefix` or lies inside it. On macOS the
/// comparison ignores case, as its default filesystem does, so a part not
/// created yet cannot dodge it by case.
#[cfg(unix)]
pub fn location_within(path: &Path, prefix: &Path) -> bool {
    if cfg!(target_os = "macos") {
        let fold = |path: &Path| -> Vec<String> {
            path.components()
                .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
                .collect()
        };
        fold(path).starts_with(&fold(prefix))
    } else {
        path.starts_with(prefix)
    }
}

fn sorted_absolute_paths(mut paths: Vec<AbsolutePathBuf>) -> Vec<AbsolutePathBuf> {
    paths.sort_by(|left, right| left.as_path().cmp(right.as_path()));
    paths
}

fn sorted_writable_roots(mut roots: Vec<WritableRoot>) -> Vec<WritableRoot> {
    for root in &mut roots {
        root.read_only_subpaths =
            sorted_absolute_paths(std::mem::take(&mut root.read_only_subpaths));
        root.protected_metadata_names.sort();
        root.protected_metadata_names.dedup();
    }
    roots.sort_by(|left, right| left.root.as_path().cmp(right.root.as_path()));
    roots
}

#[cfg(any(test, debug_assertions))]
thread_local! {
    static EFFECTIVE_PATH_NORMALIZATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// #907: how many paths this thread has normalized with filesystem calls
/// (each walks the path's ancestors, then resolves the first that exists),
/// so tests can bound what a policy check costs. Instrumentation only.
/// Available in debug builds and this crate's unit tests, not release libraries.
#[cfg(any(test, debug_assertions))]
#[doc(hidden)]
pub fn effective_path_normalizations_on_this_thread() -> u64 {
    EFFECTIVE_PATH_NORMALIZATIONS.with(std::cell::Cell::get)
}

fn normalize_effective_absolute_path(path: AbsolutePathBuf) -> AbsolutePathBuf {
    #[cfg(any(test, debug_assertions))]
    EFFECTIVE_PATH_NORMALIZATIONS.with(|count| count.set(count.get() + 1));
    let raw_path = path.to_path_buf();
    for ancestor in raw_path.ancestors() {
        if std::fs::symlink_metadata(ancestor).is_err() {
            continue;
        }
        let Ok(normalized_ancestor) = canonicalize_preserving_symlinks(ancestor) else {
            continue;
        };
        let Ok(suffix) = raw_path.strip_prefix(ancestor) else {
            continue;
        };
        if let Ok(normalized_path) =
            AbsolutePathBuf::from_absolute_path(normalized_ancestor.join(suffix))
        {
            return normalized_path;
        }
    }
    path
}

pub(crate) fn default_read_only_subpaths_for_writable_root(
    writable_root: &AbsolutePathBuf,
    protect_missing_dot_codex: bool,
) -> Vec<AbsolutePathBuf> {
    let mut subpaths: Vec<AbsolutePathBuf> = Vec::new();
    let top_level_git = writable_root.join(PROTECTED_METADATA_GIT_PATH_NAME);
    // This applies to typical repos (directory .git), worktrees/submodules
    // (file .git with gitdir pointer), and bare repos when the gitdir is the
    // writable root itself.
    let top_level_git_is_file = top_level_git.as_path().is_file();
    let top_level_git_is_dir = top_level_git.as_path().is_dir();
    let should_protect_top_level = top_level_git_is_dir || top_level_git_is_file;
    if should_protect_top_level {
        if top_level_git_is_file
            && is_git_pointer_file(&top_level_git)
            && let Some(gitdir) = resolve_gitdir_from_file(&top_level_git)
        {
            subpaths.push(gitdir);
        }
        subpaths.push(top_level_git);
    }

    let top_level_agents = writable_root.join(PROTECTED_METADATA_AGENTS_PATH_NAME);
    if top_level_agents.as_path().is_dir() {
        subpaths.push(top_level_agents);
    }

    // Keep top-level project metadata under .codex read-only to the agent by
    // default. For the workspace root itself, protect it even before the
    // directory exists so first-time creation still goes through the
    // protected-path approval flow.
    let top_level_codex = writable_root.join(PROTECTED_METADATA_CODEX_PATH_NAME);
    if protect_missing_dot_codex || top_level_codex.as_path().is_dir() {
        subpaths.push(top_level_codex);
    }

    // AWS profiles can select credential helpers that the application executes.
    let top_level_aws = writable_root.join(PROTECTED_METADATA_AWS_PATH_NAME);
    if top_level_aws.as_path().is_dir() {
        subpaths.push(top_level_aws);
    }

    dedup_absolute_paths(subpaths, /*normalize_effective_paths*/ false)
}

/// Rebuilds the filesystem policy that legacy sandbox runtimes enforce for a
/// concrete cwd.
///
/// Unlike [`FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd`], this
/// intentionally does not add symbolic project-root metadata carveouts. Legacy
/// runtime expansion only protected `.git`/`.agents` when those paths already
/// existed, so missing-path carveouts still require direct profile enforcement.
fn legacy_runtime_file_system_policy_for_cwd(
    sandbox_policy: &SandboxPolicy,
    cwd: &Path,
) -> FileSystemSandboxPolicy {
    let SandboxPolicy::WorkspaceWrite {
        writable_roots,
        exclude_tmpdir_env_var,
        exclude_slash_tmp,
        ..
    } = sandbox_policy
    else {
        return FileSystemSandboxPolicy::from(sandbox_policy);
    };

    let mut entries = vec![
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Read,
        ),
        FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            FileSystemAccessMode::Write,
        ),
    ];

    if !*exclude_slash_tmp {
        entries.push(FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::SlashTmp,
            },
            FileSystemAccessMode::Write,
        ));
    }
    if !*exclude_tmpdir_env_var {
        entries.push(FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Tmpdir,
            },
            FileSystemAccessMode::Write,
        ));
    }
    entries.extend(
        writable_roots
            .iter()
            .cloned()
            .map(|path| FileSystemSandboxEntry::new(path.into(), FileSystemAccessMode::Write)),
    );

    if let Ok(cwd_root) = AbsolutePathBuf::from_absolute_path(cwd) {
        for protected_path in default_read_only_subpaths_for_writable_root(
            &cwd_root, /*protect_missing_dot_codex*/ true,
        ) {
            append_default_read_only_path_if_no_explicit_rule(&mut entries, protected_path);
        }
    }
    for writable_root in writable_roots {
        for protected_path in default_read_only_subpaths_for_writable_root(
            writable_root,
            /*protect_missing_dot_codex*/ false,
        ) {
            append_default_read_only_path_if_no_explicit_rule(&mut entries, protected_path);
        }
    }

    FileSystemSandboxPolicy::restricted(entries)
}

fn append_default_read_only_project_root_subpath_if_no_explicit_rule(
    entries: &mut Vec<FileSystemSandboxEntry>,
    subpath: impl Into<String>,
) {
    append_default_read_only_entry_if_no_explicit_rule(
        entries,
        FileSystemPath::Special {
            value: FileSystemSpecialPath::project_roots(Some(subpath.into())),
        },
    );
}

fn append_default_read_only_path_if_no_explicit_rule(
    entries: &mut Vec<FileSystemSandboxEntry>,
    path: AbsolutePathBuf,
) {
    append_default_read_only_entry_if_no_explicit_rule(entries, path.into());
}

fn append_default_read_only_entry_if_no_explicit_rule(
    entries: &mut Vec<FileSystemSandboxEntry>,
    path: FileSystemPath,
) {
    if entries
        .iter()
        .any(|entry| file_system_paths_share_target(&entry.path, &path, PathConvention::native()))
    {
        return;
    }

    entries.push(FileSystemSandboxEntry::skip_missing_path(
        path,
        FileSystemAccessMode::Read,
    ));
}

fn has_explicit_resolved_path_entry(
    entries: &[ResolvedFileSystemEntry],
    path: &AbsolutePathBuf,
) -> bool {
    entries.iter().any(|entry| &entry.path == path)
}

<<<<<<< HEAD
/// The protected metadata name `name` is; with `any_case` also in another
/// case (#606).
fn metadata_path_name(name: &OsStr, any_case: bool) -> Option<&'static str> {
    PROTECTED_METADATA_PATH_NAMES
        .iter()
        .copied()
        .find(|metadata_name| {
            name == OsStr::new(metadata_name)
                || (any_case && name.eq_ignore_ascii_case(metadata_name))
        })
}

fn metadata_child_of_writable_root(
    policy: &FileSystemSandboxPolicy,
    target: &Path,
    cwd: &Path,
) -> Option<(AbsolutePathBuf, &'static str)> {
    // #606: macOS volumes ignore case, so a `.GIT` made where `.git` is
    // missing is the folder git opens as `.git`, and Seatbelt refuses it
    // (its rules match without regard to case there). Approvals agree once
    // the policy denies reads (Moderate and Aggressive); a policy without
    // denied paths keeps today's answers.
    let any_case = cfg!(target_os = "macos") && policy.has_denied_read_restrictions();
    policy
        .resolved_entries_with_cwd(cwd)
        .iter()
        .filter(|entry| entry.access.can_write())
        .filter_map(|entry| {
            let relative_path = target.strip_prefix(entry.path.as_path()).ok()?;
            let first_component = relative_path.components().next()?;
            let metadata_name = metadata_path_name(first_component.as_os_str(), any_case)?;
            // As spelled (#606: `.GIT` on macOS), so only an explicit entry
            // for that spelling opens it.
            Some((entry.path.join(first_component), metadata_name))
        })
        .next()
}

||||||| 413492cd6c
fn metadata_path_name(name: &OsStr) -> Option<&'static str> {
    PROTECTED_METADATA_PATH_NAMES
        .iter()
        .copied()
        .find(|metadata_name| name == OsStr::new(metadata_name))
}

fn metadata_child_of_writable_root(
    policy: &FileSystemSandboxPolicy,
    target: &Path,
    cwd: &Path,
) -> Option<(AbsolutePathBuf, &'static str)> {
    policy
        .resolved_entries_with_cwd(cwd)
        .iter()
        .filter(|entry| entry.access.can_write())
        .filter_map(|entry| {
            let relative_path = target.strip_prefix(entry.path.as_path()).ok()?;
            let first_component = relative_path.components().next()?;
            let metadata_name = metadata_path_name(first_component.as_os_str())?;
            Some((entry.path.join(metadata_name), metadata_name))
        })
        .next()
}

=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
fn protected_metadata_names_for_writable_root(
    matching: &LocalFileSystemPolicyMatcher<'_>,
    root: &AbsolutePathBuf,
    raw_writable_roots: &[&AbsolutePathBuf],
) -> Vec<String> {
    let mut protected_names = Vec::new();
    for metadata_name in PROTECTED_METADATA_PATH_NAMES {
        let mut metadata_paths = vec![root.join(*metadata_name)];
        metadata_paths.extend(
            raw_writable_roots
                .iter()
                .map(|raw_root| raw_root.join(*metadata_name)),
        );

        if metadata_paths.iter().all(|metadata_path| {
            !matching
                .can_write_path(&PathUri::from_abs_path(metadata_path))
                .unwrap_or(false)
        }) {
            protected_names.push((*metadata_name).to_string());
        }
    }
    protected_names
}

fn protected_metadata_names_need_direct_runtime_enforcement(
    policy: &FileSystemSandboxPolicy,
    legacy_policy: &SandboxPolicy,
    cwd: &Path,
) -> bool {
    let legacy_roots = legacy_policy.get_writable_roots_with_cwd(cwd);
    policy
        .get_writable_roots_with_cwd(cwd)
        .into_iter()
        .any(|writable_root| {
            let Some(legacy_root) = legacy_roots
                .iter()
                .find(|candidate| candidate.root == writable_root.root)
            else {
                return !writable_root.protected_metadata_names.is_empty();
            };

            writable_root
                .protected_metadata_names
                .iter()
                .any(|metadata_name| {
                    let metadata_path = writable_root.root.join(metadata_name);
                    !legacy_root
                        .read_only_subpaths
                        .iter()
                        .any(|subpath| subpath == &metadata_path)
                })
        })
}

fn is_git_pointer_file(path: &AbsolutePathBuf) -> bool {
    path.as_path().is_file()
        && path.as_path().file_name() == Some(OsStr::new(PROTECTED_METADATA_GIT_PATH_NAME))
}

fn resolve_gitdir_from_file(dot_git: &AbsolutePathBuf) -> Option<AbsolutePathBuf> {
    let contents = match std::fs::read_to_string(dot_git.as_path()) {
        Ok(contents) => contents,
        Err(err) => {
            error!(
                "Failed to read {path} for gitdir pointer: {err}",
                path = dot_git.as_path().display()
            );
            return None;
        }
    };

    let trimmed = contents.trim();
    let (_, gitdir_raw) = match trimmed.split_once(':') {
        Some((prefix, gitdir_raw)) if prefix.trim() == "gitdir" => (prefix, gitdir_raw),
        Some(_) => {
            error!(
                "Expected {path} to contain a gitdir pointer, but it did not match `gitdir: <path>`.",
                path = dot_git.as_path().display()
            );
            return None;
        }
        None => {
            error!(
                "Expected {path} to contain a gitdir pointer, but it did not match `gitdir: <path>`.",
                path = dot_git.as_path().display()
            );
            return None;
        }
    };
    let gitdir_raw = gitdir_raw.trim();
    if gitdir_raw.is_empty() {
        error!(
            "Expected {path} to contain a gitdir pointer, but it was empty.",
            path = dot_git.as_path().display()
        );
        return None;
    }
    let base = match dot_git.as_path().parent() {
        Some(base) => base,
        None => {
            error!(
                "Unable to resolve parent directory for {path}.",
                path = dot_git.as_path().display()
            );
            return None;
        }
    };
    let gitdir_path = AbsolutePathBuf::resolve_path_against_base(gitdir_raw, base);
    if !gitdir_path.as_path().exists() {
        error!(
            "Resolved gitdir path {path} does not exist.",
            path = gitdir_path.as_path().display()
        );
        return None;
    }
    Some(gitdir_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    #[cfg(unix)]
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[cfg(unix)]
    const SYMLINKED_TMPDIR_TEST_ENV: &str = "CODEX_PROTOCOL_TEST_SYMLINKED_TMPDIR";

    #[cfg(unix)]
    fn symlink_dir(original: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }

<<<<<<< HEAD
    /// D293 (#1542 item 2): an entry spelled through a linked root matches
    /// the path listed against the root's resolved location; a different
    /// path or a writable entry doesn't.
    #[cfg(unix)]
    #[test]
    fn explicit_non_write_entry_matches_through_a_link() -> std::io::Result<()> {
        let dir = TempDir::new()?;
        let real = dir.path().join("real");
        fs::create_dir_all(real.join("a/b"))?;
        fs::create_dir_all(real.join("w"))?;
        let linked = dir.path().join("linked");
        symlink_dir(&real, &linked)?;
        let absolute = |path: std::path::PathBuf| {
            AbsolutePathBuf::from_absolute_path(path).expect("absolute path")
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: absolute(linked.join("a/b")),
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: absolute(linked.join("w")),
                },
                FileSystemAccessMode::Write,
            ),
        ]);
        let resolved = fs::canonicalize(&real)?;
        assert_eq!(
            (
                policy.has_explicit_non_write_entry_for_path_with_cwd(
                    &resolved.join("a/b"),
                    dir.path()
                ),
                policy.has_explicit_non_write_entry_resolving_to(&resolved.join("a/b"), dir.path()),
                policy.has_explicit_non_write_entry_resolving_to(&linked.join("a/b"), dir.path()),
                policy.has_explicit_non_write_entry_resolving_to(&resolved.join("a"), dir.path()),
                policy.has_explicit_non_write_entry_resolving_to(&resolved.join("w"), dir.path()),
            ),
            (false, true, true, false, false)
        );
        Ok(())
||||||| 413492cd6c
=======
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn writable_roots_protect_gitdir_target_outside_alias_root() {
        let temp = TempDir::new().expect("tempdir");
        let base = temp.path().canonicalize().expect("canonical tempdir");
        let workspace = base.join("workspace");
        let writable = base.join("writable");
        let aliases = base.join("readonly-aliases");
        let gitdir = writable.join("gitdir");
        fs::create_dir(&workspace).expect("create workspace");
        fs::create_dir_all(&gitdir).expect("create gitdir");
        fs::create_dir(&aliases).expect("create aliases");
        let alias = aliases.join("repo");
        symlink_dir(&writable, &alias).expect("create alias");
        fs::write(
            workspace.join(".git"),
            format!("gitdir: {}\n", alias.join("gitdir").display()),
        )
        .expect("write Git pointer");
        let policy = FileSystemSandboxPolicy::restricted(
            [&workspace, &writable]
                .into_iter()
                .map(|path| {
                    FileSystemSandboxEntry::new(
                        AbsolutePathBuf::from_absolute_path(path)
                            .expect("absolute root")
                            .into(),
                        FileSystemAccessMode::Write,
                    )
                })
                .collect(),
        );
        let roots = if cfg!(target_os = "macos") {
            policy.get_writable_roots_with_cwd_preserving_mutable_paths(&workspace)
        } else {
            policy.get_writable_roots_with_cwd(&workspace)
        };
        let root = roots
            .iter()
            .find(|root| root.root.as_path() == writable)
            .expect("additional writable root");
        assert_eq!(
            root.read_only_subpaths,
            vec![AbsolutePathBuf::from_absolute_path(gitdir).expect("absolute gitdir")],
        );
    }

    #[test]
    fn permission_paths_preserve_native_strings_across_path_conventions() {
        for path in [
            "/workspace/src",
            r"C:\workspace\src",
            r"\\server\share\src",
            r"\\localhost\share",
        ] {
            let expected = serde_json::json!({ "type": "path", "path": path });
            let actual = serde_json::from_value::<RawFileSystemPath>(expected.clone())
                .expect("valid raw permission path");
            assert_eq!(
                serde_json::to_value(actual).expect("lossless raw permission path"),
                expected
            );
        }
    }

    #[test]
    fn writable_root_presence_matches_materialized_roots() {
        let cwd = TempDir::new().expect("tempdir");
        let writable_root = AbsolutePathBuf::resolve_path_against_base("work", cwd.path());
        let policies = [
            FileSystemSandboxPolicy::read_only(),
            FileSystemSandboxPolicy::unrestricted(),
            FileSystemSandboxPolicy::external_sandbox(),
            FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Write,
            )]),
            FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                writable_root.clone().into(),
                FileSystemAccessMode::Write,
            )]),
            FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry::new(
                    writable_root.clone().into(),
                    FileSystemAccessMode::Write,
                ),
                FileSystemSandboxEntry::new(writable_root.into(), FileSystemAccessMode::Deny),
            ]),
            FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::unknown(
                        ":future_special_path",
                        /*subpath*/ None,
                    ),
                },
                FileSystemAccessMode::Write,
            )]),
        ];

        for policy in policies {
            assert_eq!(
                policy.has_configured_writable_roots_with_cwd(cwd.path()),
                !policy.get_writable_roots_with_cwd(cwd.path()).is_empty()
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn permission_paths_preserve_native_slash_unc_strings() {
        for path in ["//server/share/src", r"/\server/share/src"] {
            let expected = serde_json::json!({ "type": "path", "path": path });
            let actual = serde_json::from_value::<RawFileSystemPath>(expected.clone())
                .expect("valid raw slash UNC permission path");
            assert_eq!(
                serde_json::to_value(actual).expect("lossless raw slash UNC permission path"),
                expected
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn native_ambiguous_permission_paths_keep_deny_semantics() {
        let cwd = TempDir::new().expect("tempdir");
        for path in ["//server/share/secret", "/C:/secret"] {
            let denied_path = serde_json::from_value::<RawFileSystemPath>(serde_json::json!({
                "type": "path",
                "path": path,
            }))
            .expect("raw permission path")
            .try_into()
            .expect("runtime permission path");
            let policy = FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry::new(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    FileSystemAccessMode::Read,
                ),
                FileSystemSandboxEntry::new(denied_path, FileSystemAccessMode::Deny),
            ]);

            assert!(
                !policy.can_read_local_path_with_cwd(Path::new(path), cwd.path()),
                "deny should apply to {path}"
            );
            assert!(
                policy.can_read_local_path_with_cwd(&cwd.path().join("ordinary"), cwd.path()),
                "opaque deny for {path} should not poison ordinary paths"
            );
        }
    }

    #[test]
    fn uri_matcher_resolves_selected_executor_paths() {
        let path = |path| PathUri::parse(path).expect("valid path URI");
        let cwd = path("file:///C:/workspace");
        let workspace_roots = [cwd.clone()];
        let temporary_directories = [path("file:///C:/Temp")];
        let context = FileSystemSandboxPolicyContext {
            cwd: &cwd,
            workspace_roots: &workspace_roots,
            user_home_dir: None,
            temporary_directories: Some(&temporary_directories),
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                path("file:///C:/workspace/private").into(),
                FileSystemAccessMode::Deny,
            ),
            FileSystemSandboxEntry::new(
                path("file:///C:/workspace/private/public").into(),
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Tmpdir,
                },
                FileSystemAccessMode::Write,
            ),
        ]);

        for (candidate, expected) in [
            (
                "file:///C:/workspace/src/main.rs",
                FileSystemAccessMode::Write,
            ),
            (
                "file:///c:/WORKSPACE/private/key",
                FileSystemAccessMode::Deny,
            ),
            (
                "file:///C:/workspace/private/public/ok",
                FileSystemAccessMode::Write,
            ),
            ("file:///C:/Temp/cache", FileSystemAccessMode::Write),
            ("file:///C:/outside", FileSystemAccessMode::Read),
            ("file:///tmp/cache", FileSystemAccessMode::Deny),
        ] {
            assert_eq!(
                policy.resolve_access(&path(candidate), &context),
                expected,
                "resolving {candidate}"
            );
        }
        assert!(!policy.can_write_path(&path("file:///c:/WORKSPACE/.git/config"), &context,));

        let scoped = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(Some("docs/../public".to_string())),
            },
            FileSystemAccessMode::Write,
        )]);
        assert_eq!(
            scoped.resolve_access(&path("file:///C:/workspace/public/file"), &context),
            FileSystemAccessMode::Write
        );
        assert_eq!(
            scoped.resolve_access(&path("file:///C:/outside/file"), &context),
            FileSystemAccessMode::Deny
        );
    }

    #[test]
    fn uri_matcher_uses_validated_native_components_for_precedence() {
        let path = |path| PathUri::parse(path).expect("valid path URI");
        let cwd = path("file:///workspace");
        let context = FileSystemSandboxPolicyContext {
            cwd: &cwd,
            workspace_roots: std::slice::from_ref(&cwd),
            user_home_dir: None,
            temporary_directories: None,
        };
        let candidate = path("file:///workspace/private/secret/key");

        for (case, writable_path, denied_path) in [
            (
                "encoded component",
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                path("file:///workspace/%70rivate").into(),
            ),
            (
                "encoded separator",
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                path("file:///workspace/private%2Fsecret").into(),
            ),
            (
                "repeated separators",
                path("file:///workspace////").into(),
                path("file:///workspace/private").into(),
            ),
        ] {
            let policy = FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry::new(writable_path, FileSystemAccessMode::Write),
                FileSystemSandboxEntry::new(denied_path, FileSystemAccessMode::Deny),
            ]);

            assert_eq!(
                policy.resolve_access(&candidate, &context),
                FileSystemAccessMode::Deny,
                "resolving {case}"
            );
        }
    }

    #[test]
    fn uri_matcher_fails_closed_without_executor_roots_or_lexical_paths() {
        let path = |path| PathUri::parse(path).expect("valid path URI");
        let cwd = path("file://server/share/workspace");
        let context = FileSystemSandboxPolicyContext {
            cwd: &cwd,
            workspace_roots: &[],
            user_home_dir: None,
            temporary_directories: None,
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                FileSystemAccessMode::Write,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Tmpdir,
                },
                FileSystemAccessMode::Write,
            ),
        ]);

        assert_eq!(
            policy.resolve_access(&cwd, &context),
            FileSystemAccessMode::Deny
        );
        assert_eq!(
            policy.resolve_access(&path("file:///tmp/cache"), &context),
            FileSystemAccessMode::Deny
        );

        let opaque = path("file:///%00/bad/path/YQ");
        let opaque_context = FileSystemSandboxPolicyContext {
            cwd: &opaque,
            workspace_roots: std::slice::from_ref(&opaque),
            user_home_dir: None,
            temporary_directories: None,
        };
        let opaque_policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            opaque.clone().into(),
            FileSystemAccessMode::Write,
        )]);
        assert_eq!(
            opaque_policy.resolve_access(&opaque, &opaque_context),
            FileSystemAccessMode::Deny
        );
    }

    #[test]
    fn uri_deny_and_metadata_matcher_use_executor_paths() {
        let path = |path| PathUri::parse(path).expect("valid path URI");
        let cwd = path("file:///C:/workspace");
        let roots = [cwd.clone()];
        let context = FileSystemSandboxPolicyContext {
            cwd: &cwd,
            workspace_roots: &roots,
            user_home_dir: None,
            temporary_directories: None,
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(cwd.clone().into(), FileSystemAccessMode::Write),
            FileSystemSandboxEntry::new(path("file:///C:/").into(), FileSystemAccessMode::Deny),
            FileSystemSandboxEntry::new(
                path("file:///C:/workspace/.codex").into(),
                FileSystemAccessMode::Read,
            ),
            unreadable_glob_entry(r"C:\workspace\**\*.env".to_string()),
        ]);
        let prepared = policy
            .prepare_deny_read_matcher(&context, InvalidDenyReadGlobBehavior::ReturnError)
            .expect("remote deny matcher");

        assert!(prepared.denied_roots.is_empty());
        assert!(FileSystemSandboxPolicy::matches_prepared_read_deny(
            &path("file:///c:/WORKSPACE/app/.ENV"),
            &context,
            &prepared,
        ));
        for candidate in ["file:///%00/bad/path/YQ", "file:///C:/workspace/%2Fsecret"] {
            assert!(FileSystemSandboxPolicy::matches_prepared_read_deny(
                &path(candidate),
                &context,
                &prepared,
            ));
        }
        assert!(!policy.can_write_path(&path("file:///C:/workspace/.codex/config"), &context));
        assert_eq!(
            policy.metadata_write_denial(&path("file:///C:/workspace/.codex/config"), &context),
            Some(".codex"),
        );
    }

    #[test]
    fn uri_deny_globs_use_executor_home() {
        for (cwd, home, pattern, candidate) in [
            (
                "file:///workspace",
                "file:///home/executor",
                "~/private/*.env",
                "file:///home/executor/private/secret.env",
            ),
            (
                "file:///workspace",
                "file:///home/executor",
                "~//private/*.env",
                "file:///home/executor/private/secret.env",
            ),
            (
                "file:///C:/workspace",
                "file:///C:/Users/executor",
                r"~\private\*.env",
                "file:///C:/Users/executor/private/secret.env",
            ),
            (
                "file:///C:/workspace",
                "file:///C:/Users/executor",
                r"~\\private\*.env",
                "file:///C:/Users/executor/private/secret.env",
            ),
        ] {
            let cwd = PathUri::parse(cwd).expect("executor cwd");
            let home = PathUri::parse(home).expect("executor home");
            let candidate = PathUri::parse(candidate).expect("denied path");
            let context = FileSystemSandboxPolicyContext {
                cwd: &cwd,
                workspace_roots: std::slice::from_ref(&cwd),
                user_home_dir: Some(&home),
                temporary_directories: None,
            };
            let policy = FileSystemSandboxPolicy::restricted(vec![unreadable_glob_entry(
                pattern.to_string(),
            )]);
            let prepared = policy
                .prepare_deny_read_matcher(&context, InvalidDenyReadGlobBehavior::ReturnError)
                .expect("home-relative deny glob");

            assert!(FileSystemSandboxPolicy::matches_prepared_read_deny(
                &candidate, &context, &prepared,
            ));

            let without_home = FileSystemSandboxPolicyContext {
                user_home_dir: None,
                ..context
            };
            assert!(
                policy
                    .prepare_deny_read_matcher(
                        &without_home,
                        InvalidDenyReadGlobBehavior::ReturnError,
                    )
                    .is_err()
            );
            let prepared = policy
                .prepare_deny_read_matcher(&without_home, InvalidDenyReadGlobBehavior::FailClosed)
                .expect("missing executor home fails closed");
            assert!(prepared.invalid_pattern);
        }
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    }

    #[test]
    fn unknown_special_paths_are_ignored_by_legacy_bridge() -> std::io::Result<()> {
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::unknown(
                        ":future_special_path",
                        /*subpath*/ None,
                    ),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
        ]);

        let sandbox_policy = policy.to_legacy_sandbox_policy(
            NetworkSandboxPolicy::Restricted,
            Path::new("/tmp/workspace"),
        )?;

        assert_eq!(
            sandbox_policy,
            SandboxPolicy::ReadOnly {
                network_access: false,
            }
        );
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn workspace_write_ignores_slash_tmp_special_path_but_preserves_literal_path() {
        let cwd = TempDir::new().expect("tempdir");
        let slash_tmp = AbsolutePathBuf::from_absolute_path("/tmp").expect("absolute tmp path");
        let slash_tmp_only_policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::SlashTmp,
                },
                FileSystemAccessMode::Write,
            ),
        ]);
        assert_eq!(
            slash_tmp_only_policy
                .to_legacy_sandbox_policy(NetworkSandboxPolicy::Restricted, cwd.path())
                .expect("legacy sandbox policy"),
            SandboxPolicy::ReadOnly {
                network_access: false,
            }
        );
        assert!(
            !slash_tmp_only_policy
                .needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path())
        );

        for access in [FileSystemAccessMode::Read, FileSystemAccessMode::Deny] {
            let policy = FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry::new(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    FileSystemAccessMode::Write,
                ),
                FileSystemSandboxEntry::new(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::SlashTmp,
                    },
                    access,
                ),
            ]);
            assert!(policy.has_full_disk_write_access());
            assert!(policy.has_full_disk_read_access());
        }

        let legacy_policy = SandboxPolicy::new_workspace_write_policy();
        assert_eq!(
            FileSystemSandboxPolicy::from(&legacy_policy)
                .to_legacy_sandbox_policy(NetworkSandboxPolicy::Restricted, cwd.path())
                .expect("legacy workspace-write policy"),
            legacy_policy
        );
        assert_eq!(
            FileSystemSandboxPolicy::from(&legacy_policy)
                .get_writable_roots_with_cwd(cwd.path())
                .into_iter()
                .map(|root| root.root)
                .collect::<Vec<_>>(),
            legacy_policy
                .get_writable_roots_with_cwd(cwd.path())
                .into_iter()
                .map(|root| normalize_effective_absolute_path(root.root))
                .collect::<Vec<_>>()
        );
        assert!(
            FileSystemSandboxPolicy::workspace_write(
                std::slice::from_ref(&slash_tmp),
                /*exclude_tmpdir_env_var*/ true,
                /*exclude_slash_tmp*/ false,
            )
            .can_write_local_path_with_cwd(slash_tmp.as_path(), cwd.path())
        );
    }

    #[cfg(unix)]
    #[test]
    fn writable_roots_proactively_protect_missing_dot_codex() {
        let cwd = TempDir::new().expect("tempdir");
        let expected_root = AbsolutePathBuf::from_absolute_path(
            cwd.path().canonicalize().expect("canonicalize cwd"),
        )
        .expect("absolute canonical root");
        let expected_dot_codex = expected_root.join(".codex");

        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        }]);

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_dot_codex)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn preserving_mutable_paths_normalizes_top_level_aliases_consistently() {
        let root = TempDir::new_in("/tmp").expect("tempdir under /tmp");
        let logical_root =
            AbsolutePathBuf::from_absolute_path(root.path()).expect("absolute logical root");
        let canonical_root = AbsolutePathBuf::from_absolute_path(
            root.path().canonicalize().expect("canonicalize root"),
        )
        .expect("absolute canonical root");
        let protected = canonical_root.join("protected");
        fs::create_dir(&protected).expect("create protected path");
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(logical_root.into(), FileSystemAccessMode::Write),
            FileSystemSandboxEntry::new(protected.clone().into(), FileSystemAccessMode::Read),
        ]);

        let roots = policy.get_writable_roots_with_cwd_preserving_mutable_paths(root.path());

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].root, canonical_root);
        assert!(roots[0].read_only_subpaths.contains(&protected));
    }

    #[cfg(unix)]
    #[test]
    fn preserving_writable_roots_cannot_be_rebound_during_projection() {
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::AtomicUsize;
        use std::sync::atomic::Ordering;
        use std::thread;

        let tmp = TempDir::new().expect("tempdir");
        let active_ancestor = tmp.path().join("active");
        let parked_ancestor = tmp.path().join("parked");
        let outside_ancestor = tmp.path().join("outside");
        let writable_root = active_ancestor.join("workspace");
        let outside_root = outside_ancestor.join("workspace");
        fs::create_dir_all(&writable_root).expect("create writable root");
        fs::create_dir_all(&outside_root).expect("create outside root");
        let writable_root =
            AbsolutePathBuf::from_absolute_path(writable_root).expect("absolute writable root");
        let outside_root =
            AbsolutePathBuf::from_absolute_path(outside_root).expect("absolute outside root");
        let expected_writable_root = writable_root
            .normalize_system_aliases()
            .expect("normalize writable root");
        let expected_outside_root = outside_root
            .normalize_system_aliases()
            .expect("normalize outside root");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            writable_root.into(),
            FileSystemAccessMode::Write,
        )]);

        let stop = Arc::new(AtomicBool::new(false));
        let swaps = Arc::new(AtomicUsize::new(0));
        let racer_stop = Arc::clone(&stop);
        let racer_swaps = Arc::clone(&swaps);
        let racer = thread::spawn(move || {
            while !racer_stop.load(Ordering::Relaxed) {
                if fs::rename(&active_ancestor, &parked_ancestor).is_err() {
                    thread::yield_now();
                    continue;
                }
                if symlink_dir(&outside_ancestor, &active_ancestor).is_ok() {
                    racer_swaps.fetch_add(1, Ordering::Relaxed);
                    thread::yield_now();
                    let _ = fs::remove_file(&active_ancestor);
                }
                fs::rename(&parked_ancestor, &active_ancestor).expect("restore writable ancestor");
            }
        });

        let mut rebound_root = None;
        for _ in 0..2_000 {
            let roots = policy.get_writable_roots_with_cwd_preserving_mutable_paths(tmp.path());
            if roots.len() != 1 || roots[0].root != expected_writable_root {
                rebound_root = roots.first().map(|root| root.root.clone());
                break;
            }
            assert_ne!(roots[0].root, expected_outside_root);
            thread::yield_now();
        }
        stop.store(true, Ordering::Relaxed);
        racer.join().expect("join path racer");

        assert!(swaps.load(Ordering::Relaxed) > 0, "racer did not run");
        assert_eq!(rebound_root, None);
    }

    #[test]
    fn legacy_workspace_write_projection_preserves_symbolic_project_root() {
        let policy = SandboxPolicy::WorkspaceWrite {
            writable_roots: Vec::new(),
            network_access: false,
            exclude_tmpdir_env_var: true,
            exclude_slash_tmp: true,
        };

        assert_eq!(
            FileSystemSandboxPolicy::from(&policy),
            FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::Root,
                    },
                    access: FileSystemAccessMode::Read,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(Some(".git".into())),
                    },
                    FileSystemAccessMode::Read,
                ),
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(Some(".agents".into())),
                    },
                    FileSystemAccessMode::Read,
                ),
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(Some(".codex".into())),
                    },
                    FileSystemAccessMode::Read,
                ),
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(Some(".aws".into())),
                    },
                    FileSystemAccessMode::Read,
                ),
            ])
        );
    }

    #[test]
    fn legacy_current_working_directory_special_path_deserializes_as_project_roots()
    -> serde_json::Result<()> {
        let value = serde_json::json!({
            "kind": "current_working_directory",
        });

        let special_path = serde_json::from_value::<FileSystemSpecialPath>(value)?;
        assert_eq!(
            special_path,
            FileSystemSpecialPath::project_roots(/*subpath*/ None)
        );
        assert_eq!(
            serde_json::to_value(&special_path)?,
            serde_json::json!({
                "kind": "project_roots",
            })
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn writable_roots_skip_default_metadata_when_explicit_user_rule_exists() {
        for name in [".codex", ".aws"] {
            let cwd = TempDir::new().expect("tempdir");
            let expected_root = AbsolutePathBuf::from_absolute_path(
                cwd.path().canonicalize().expect("canonicalize cwd"),
            )
            .expect("absolute canonical root");
            let explicit_metadata = expected_root.join(name);
            fs::create_dir(&explicit_metadata).expect("create metadata directory");

            let policy = FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: explicit_metadata.clone().into(),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
            ]);

            let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
            let workspace_root = writable_roots
                .iter()
                .find(|root| root.root == expected_root)
                .expect("workspace writable root");
            assert!(
                !workspace_root
                    .protected_metadata_names
                    .contains(&name.to_string()),
                "explicit {name} rule should remove the metadata-name protection"
            );
            assert!(
                !workspace_root
                    .read_only_subpaths
                    .contains(&explicit_metadata),
                "explicit {name} rule should win over the default protected carveout"
            );
            assert!(policy.can_write_local_path_with_cwd(
                explicit_metadata.join("config.toml").as_path(),
                cwd.path()
            ));
        }
    }

    #[test]
    fn filesystem_policy_blocks_protected_metadata_path_writes_by_default() {
        let cwd = TempDir::new().expect("tempdir");
        let dot_git_config = cwd.path().join(".git").join("config");
        let dot_agents_config = cwd.path().join(".agents").join("config");
        let dot_codex_config = cwd.path().join(".codex").join("config.toml");
        let dot_aws_config = cwd.path().join(".aws").join("config");
        let root = AbsolutePathBuf::from_absolute_path(cwd.path()).expect("absolute cwd");
        let file_system_policy =
            FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
                path: root.into(),
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            }]);

        assert!(!file_system_policy.can_write_local_path_with_cwd(&dot_git_config, cwd.path()));
        assert!(!file_system_policy.can_write_local_path_with_cwd(&dot_agents_config, cwd.path()));
        assert!(!file_system_policy.can_write_local_path_with_cwd(&dot_codex_config, cwd.path()));
        assert!(!file_system_policy.can_write_local_path_with_cwd(&dot_aws_config, cwd.path()));

        let writable_roots = file_system_policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(
            writable_roots[0].protected_metadata_names,
            vec![
                ".git".to_string(),
                ".agents".to_string(),
                ".codex".to_string(),
                ".aws".to_string(),
            ]
        );
        assert!(!writable_roots[0].is_path_writable(&dot_git_config));
        assert!(!writable_roots[0].is_path_writable(&dot_agents_config));
        assert!(!writable_roots[0].is_path_writable(&dot_codex_config));
        assert!(!writable_roots[0].is_path_writable(&dot_aws_config));
    }

    #[test]
    fn legacy_workspace_write_projection_accepts_relative_cwd() {
        let relative_cwd = Path::new("workspace");
        let expected_root = AbsolutePathBuf::from_absolute_path(
            std::env::current_dir()
                .expect("current dir")
                .join(relative_cwd),
        )
        .expect("absolute root");
        let policy = SandboxPolicy::WorkspaceWrite {
            writable_roots: vec![],
            network_access: false,
            exclude_tmpdir_env_var: true,
            exclude_slash_tmp: true,
        };

        let file_system_policy =
            FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(&policy, relative_cwd);

        let mut expected_entries = vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
        ];
        expected_entries.extend(PROTECTED_METADATA_PATH_NAMES.iter().map(|name| {
            FileSystemSandboxEntry::skip_missing_path(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(Some((*name).into())),
                },
                FileSystemAccessMode::Read,
            )
        }));
        expected_entries.extend(
            default_read_only_subpaths_for_writable_root(
                &expected_root,
                /*protect_missing_dot_codex*/ true,
            )
            .into_iter()
            .map(|path| {
                FileSystemSandboxEntry::skip_missing_path(path.into(), FileSystemAccessMode::Read)
            }),
        );

        assert_eq!(
            file_system_policy,
            FileSystemSandboxPolicy::restricted(expected_entries)
        );
        assert_eq!(
            forbidden_agent_metadata_write(
                Path::new(".git/config"),
                relative_cwd,
                &file_system_policy,
            ),
            Some(".git")
        );
        assert!(
            file_system_policy
                .can_write_local_path_with_cwd(Path::new("src/main.rs"), relative_cwd,)
        );
        assert!(
            !file_system_policy
                .can_write_local_path_with_cwd(Path::new(".codex/config.toml"), relative_cwd,)
        );
        assert!(!file_system_policy.can_write_local_path_with_cwd(
            Path::new(".agents/skills/example/SKILL.md"),
            relative_cwd,
        ));
    }

    #[cfg(unix)]
    #[test]
    fn effective_runtime_roots_preserve_symlinked_paths() {
        let cwd = TempDir::new().expect("tempdir");
        let real_root = cwd.path().join("real");
        let link_root = cwd.path().join("link");
        let blocked = real_root.join("blocked");
        let codex_dir = real_root.join(".codex");

        fs::create_dir_all(&blocked).expect("create blocked");
        fs::create_dir_all(&codex_dir).expect("create .codex");
        symlink_dir(&real_root, &link_root).expect("create symlinked root");

        let link_root =
            AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute symlinked root");
        let link_blocked = link_root.join("blocked");
        let expected_root = link_root.clone();
        let expected_blocked = link_blocked.clone();
        let expected_codex = link_root.join(".codex");

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: link_root.into(),
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: link_blocked.into(),
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        assert_eq!(
            policy.get_unreadable_roots_with_cwd(cwd.path()),
            vec![expected_blocked.clone()]
        );

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_blocked)
        );
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_codex)
        );
    }

    #[cfg(unix)]
    #[test]
    fn project_roots_special_path_preserves_symlinked_root() {
        let cwd = TempDir::new().expect("tempdir");
        let real_root = cwd.path().join("real");
        let link_root = cwd.path().join("link");
        let blocked = real_root.join("blocked");
        let agents_dir = real_root.join(".agents");
        let codex_dir = real_root.join(".codex");

        fs::create_dir_all(&blocked).expect("create blocked");
        fs::create_dir_all(&agents_dir).expect("create .agents");
        fs::create_dir_all(&codex_dir).expect("create .codex");
        symlink_dir(&real_root, &link_root).expect("create symlinked cwd");

        let link_blocked =
            AbsolutePathBuf::from_absolute_path(link_root.join("blocked")).expect("link blocked");
        let expected_root =
            AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute symlinked root");
        let expected_blocked = link_blocked.clone();
        let expected_agents = expected_root.join(".agents");
        let expected_codex = expected_root.join(".codex");

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Minimal,
                },
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: link_blocked.into(),
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        assert_eq!(
            policy.get_readable_roots_with_cwd(&link_root),
            vec![expected_root.clone()]
        );
        assert_eq!(
            policy.get_unreadable_roots_with_cwd(&link_root),
            vec![expected_blocked.clone()]
        );

        let writable_roots = policy.get_writable_roots_with_cwd(&link_root);
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_blocked)
        );
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_agents)
        );
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_codex)
        );
    }

    #[cfg(unix)]
    #[test]
    fn writable_roots_preserve_symlinked_protected_subpaths() {
        let cwd = TempDir::new().expect("tempdir");
        let root = cwd.path().join("root");
        let decoy = root.join("decoy-codex");
        let dot_codex = root.join(".codex");
        fs::create_dir_all(&decoy).expect("create decoy");
        symlink_dir(&decoy, &dot_codex).expect("create .codex symlink");

        let root = AbsolutePathBuf::from_absolute_path(&root).expect("absolute root");
        let expected_dot_codex = AbsolutePathBuf::from_absolute_path(
            root.as_path()
                .canonicalize()
                .expect("canonicalize root")
                .join(".codex"),
        )
        .expect("absolute .codex symlink");
        let unexpected_decoy =
            AbsolutePathBuf::from_absolute_path(decoy.canonicalize().expect("canonicalize decoy"))
                .expect("absolute canonical decoy");

        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: root.into(),
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        }]);

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(
            writable_roots[0].read_only_subpaths,
            vec![expected_dot_codex]
        );
        assert!(
            !writable_roots[0]
                .read_only_subpaths
                .contains(&unexpected_decoy)
        );
    }

    #[cfg(unix)]
    #[test]
    fn writable_roots_preserve_explicit_symlinked_carveouts_under_symlinked_roots() {
        let cwd = TempDir::new().expect("tempdir");
        let real_root = cwd.path().join("real");
        let link_root = cwd.path().join("link");
        let decoy = real_root.join("decoy-private");
        let linked_private = real_root.join("linked-private");
        fs::create_dir_all(&decoy).expect("create decoy");
        symlink_dir(&real_root, &link_root).expect("create symlinked root");
        symlink_dir(&decoy, &linked_private).expect("create linked-private symlink");

        let link_root =
            AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute symlinked root");
        let link_private = link_root.join("linked-private");
        let expected_root = link_root.clone();
        let expected_linked_private = link_private.clone();
        let unexpected_decoy =
            AbsolutePathBuf::from_absolute_path(decoy.canonicalize().expect("canonicalize decoy"))
                .expect("absolute canonical decoy");

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: link_root.into(),
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: link_private.into(),
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert_eq!(
            writable_roots[0].read_only_subpaths,
            vec![expected_linked_private]
        );
        assert!(
            !writable_roots[0]
                .read_only_subpaths
                .contains(&unexpected_decoy)
        );
    }

    #[cfg(unix)]
    #[test]
    fn writable_roots_preserve_explicit_symlinked_carveouts_that_escape_root() {
        let cwd = TempDir::new().expect("tempdir");
        let real_root = cwd.path().join("real");
        let link_root = cwd.path().join("link");
        let decoy = cwd.path().join("outside-private");
        let linked_private = real_root.join("linked-private");
        fs::create_dir_all(&decoy).expect("create decoy");
        fs::create_dir_all(&real_root).expect("create real root");
        symlink_dir(&real_root, &link_root).expect("create symlinked root");
        symlink_dir(&decoy, &linked_private).expect("create linked-private symlink");

        let link_root =
            AbsolutePathBuf::from_absolute_path(&link_root).expect("absolute symlinked root");
        let link_private = link_root.join("linked-private");
        let expected_root = link_root.clone();
        let expected_linked_private = link_private.clone();
        let unexpected_decoy =
            AbsolutePathBuf::from_absolute_path(decoy.canonicalize().expect("canonicalize decoy"))
                .expect("absolute canonical decoy");

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: link_root.into(),
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: link_private.into(),
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert_eq!(
            writable_roots[0].read_only_subpaths,
            vec![expected_linked_private]
        );
        assert!(
            !writable_roots[0]
                .read_only_subpaths
                .contains(&unexpected_decoy)
        );
    }

    #[cfg(unix)]
    #[test]
    fn writable_roots_preserve_explicit_symlinked_carveouts_that_alias_root() {
        let cwd = TempDir::new().expect("tempdir");
        let root = cwd.path().join("root");
        let alias = root.join("alias-root");
        fs::create_dir_all(&root).expect("create root");
        symlink_dir(&root, &alias).expect("create alias symlink");

        let root = AbsolutePathBuf::from_absolute_path(&root).expect("absolute root");
        let alias = root.join("alias-root");
        let expected_root = AbsolutePathBuf::from_absolute_path(
            root.as_path().canonicalize().expect("canonicalize root"),
        )
        .expect("absolute canonical root");
        let expected_alias = expected_root.join("alias-root");

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: root.into(),
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: alias.into(),
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert_eq!(writable_roots[0].read_only_subpaths, vec![expected_alias]);
    }

    #[cfg(unix)]
    #[test]
    fn tmpdir_special_path_preserves_symlinked_tmpdir() {
        if std::env::var_os(SYMLINKED_TMPDIR_TEST_ENV).is_none() {
            let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
                .env(SYMLINKED_TMPDIR_TEST_ENV, "1")
                .arg("--exact")
                .arg("permissions::tests::tmpdir_special_path_preserves_symlinked_tmpdir")
                .output()
                .expect("run tmpdir subprocess test");

            assert!(
                output.status.success(),
                "tmpdir subprocess test failed\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let cwd = TempDir::new().expect("tempdir");
        let real_tmpdir = cwd.path().join("real-tmpdir");
        let link_tmpdir = cwd.path().join("link-tmpdir");
        let blocked = real_tmpdir.join("blocked");
        let codex_dir = real_tmpdir.join(".codex");

        fs::create_dir_all(&blocked).expect("create blocked");
        fs::create_dir_all(&codex_dir).expect("create .codex");
        symlink_dir(&real_tmpdir, &link_tmpdir).expect("create symlinked tmpdir");

        let link_blocked =
            AbsolutePathBuf::from_absolute_path(link_tmpdir.join("blocked")).expect("link blocked");
        let expected_root =
            AbsolutePathBuf::from_absolute_path(&link_tmpdir).expect("absolute symlinked tmpdir");
        let expected_blocked = link_blocked.clone();
        let expected_codex = expected_root.join(".codex");

        unsafe {
            std::env::set_var("TMPDIR", &link_tmpdir);
        }

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Tmpdir,
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: link_blocked.into(),
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        assert_eq!(
            policy.get_unreadable_roots_with_cwd(cwd.path()),
            vec![expected_blocked.clone()]
        );

        let writable_roots = policy.get_writable_roots_with_cwd(cwd.path());
        assert_eq!(writable_roots.len(), 1);
        assert_eq!(writable_roots[0].root, expected_root);
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_blocked)
        );
        assert!(
            writable_roots[0]
                .read_only_subpaths
                .contains(&expected_codex)
        );
    }

    #[cfg(unix)]
    fn deny_in_workspace(denied: &Path) -> FileSystemSandboxPolicy {
        FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Path {
                    path: AbsolutePathBuf::from_absolute_path(denied).expect("absolute"),
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ])
    }

    /// #467: `/tmp` and `/private/tmp` name one file to the sandbox, so a
    /// rule and a workspace spelled differently through that alias agree.
    #[cfg(target_os = "macos")]
    #[test]
    fn can_read_effective_path_matches_through_top_level_aliases() {
        let root = tempfile::tempdir_in("/tmp").expect("tempdir in /tmp");
        let logical = root.path().to_path_buf();
        assert!(logical.starts_with("/tmp"), "{logical:?}");
        let physical = logical.canonicalize().expect("canonical");
        assert!(physical.starts_with("/private/tmp"), "{physical:?}");
        fs::write(physical.join(".env"), "x").expect(".env");
        for (denied, cwd) in [(&physical, &logical), (&logical, &physical)] {
            let policy = deny_in_workspace(&denied.join(".env"));
            for path in [
                logical.join(".env"),
                physical.join(".env"),
                PathBuf::from(".env"),
            ] {
                assert!(
                    !policy.can_read_effective_path_with_cwd(&path, cwd),
                    "{path:?} readable with {denied:?}/.env denied, cwd {cwd:?}"
                );
            }
            // Not created yet: still denied.
            let later = deny_in_workspace(&denied.join("later.key"));
            assert!(!later.can_read_effective_path_with_cwd(&logical.join("later.key"), cwd));
            assert!(policy.can_read_effective_path_with_cwd(&logical.join("other"), cwd));
        }
        // The literal check is what #467 reported.
        let policy = deny_in_workspace(&physical.join(".env"));
        assert!(policy.can_read_path_with_cwd(&logical.join(".env"), &logical));
    }

    /// #619: a writable root spelled `/tmp` holds `/private/tmp/x` for the
    /// sandbox; the effective check says so, the literal one does not. A
    /// more specific read-only entry in either spelling still wins.
    #[cfg(target_os = "macos")]
    #[test]
    fn can_write_effective_path_matches_through_top_level_aliases() {
        let root = tempfile::tempdir_in("/tmp").expect("tempdir in /tmp");
        let logical = root.path().to_path_buf();
        let physical = logical.canonicalize().expect("canonical");
        assert!(physical.starts_with("/private/tmp"), "{physical:?}");
        let entry = |path: &Path, access| FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
            },
            access,
            missing_path_behavior: None,
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            entry(&logical, FileSystemAccessMode::Write),
        ]);
        let target = physical.join("home/AGENTS.md");
        assert!(!policy.can_write_path_with_cwd(&target, &physical));
        assert!(policy.can_write_effective_path_with_cwd(&target, &physical));
        assert!(policy.can_write_effective_path_with_cwd(&logical.join("x"), &physical));
        for read_only in [physical.join("home"), logical.join("home")] {
            let mut narrowed = policy.clone();
            narrowed
                .entries
                .push(entry(&read_only, FileSystemAccessMode::Read));
            assert!(
                !narrowed.can_write_effective_path_with_cwd(&target, &physical),
                "{read_only:?}"
            );
        }
        assert!(!policy.can_write_effective_path_with_cwd(Path::new("/etc/x"), &physical));
        // The root's protected metadata stays protected in either spelling.
        for path in [physical.join(".codex/x"), logical.join(".git/x")] {
            assert!(
                !policy.can_write_effective_path_with_cwd(&path, &physical),
                "{path:?}"
            );
        }
    }

    /// #652: a policy naming one folder in two spellings with different
    /// access. Seatbelt grants the writable one, so a path there counts as
    /// writable where it becomes read-only; a write carve-out there is never
    /// granted (the read-only entry wins).
    #[cfg(target_os = "macos")]
    #[test]
    fn can_grant_write_lets_a_read_only_spelling_win_a_tie() {
        let root = tempfile::tempdir_in("/tmp").expect("tempdir in /tmp");
        let logical = root.path().to_path_buf();
        let physical = logical.canonicalize().expect("canonical");
        let entry = |path: &Path, access| FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
            },
            access,
            missing_path_behavior: None,
        };
        for (write, read) in [(&logical, &physical), (&physical, &logical)] {
            let policy = FileSystemSandboxPolicy::restricted(vec![
                entry(write, FileSystemAccessMode::Write),
                entry(read, FileSystemAccessMode::Read),
            ]);
            let target = physical.join("home/worktrees");
            assert!(policy.can_write_effective_path_with_cwd(&target, &physical));
            assert!(!policy.can_grant_write_effective_path_with_cwd(&target, &physical));
        }
        let plain =
            FileSystemSandboxPolicy::restricted(vec![entry(&logical, FileSystemAccessMode::Write)]);
        assert!(plain.can_grant_write_effective_path_with_cwd(&physical.join("x"), &physical));
    }

    /// #652: a writable root reached through a nested link is granted at its
    /// real location, so a path at that location is writable through it,
    /// spelled through the root; a carve-out there is not granted.
    #[cfg(unix)]
    #[test]
    fn writable_roots_reached_through_a_nested_link_are_followed() {
        let dir = TempDir::new().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonical");
        let real = base.join("real");
        std::fs::create_dir_all(real.join("home")).expect("home");
        let link = base.join("link");
        std::os::unix::fs::symlink(&real, &link).expect("link");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::from_absolute_path(&link).expect("absolute"),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        }]);
        let target = real.join("home/hooks.json");
        assert!(!policy.can_write_path_with_cwd(&target, &base));
        assert!(policy.can_write_effective_path_with_cwd(&target, &base));
        assert!(!policy.can_grant_write_effective_path_with_cwd(&target, &base));
        assert_eq!(
            policy.writable_root_spellings_with_cwd(&target, &base),
            vec![AbsolutePathBuf::from_absolute_path(link.join("home/hooks.json")).expect("abs")]
        );
        // Spelled through the root already: nothing more.
        assert_eq!(
            policy.writable_root_spellings_with_cwd(&link.join("home/x"), &base),
            Vec::<AbsolutePathBuf>::new()
        );
        assert!(!policy.can_write_effective_path_with_cwd(&base.join("other/x"), &base));
    }

    /// Seatbelt matches a denial to a readable root by its normalized
    /// spelling, which keeps nested links: a rule reached only through one
    /// is not counted, so the check stays on the safe side.
    #[cfg(unix)]
    #[test]
    fn can_read_effective_path_keeps_nested_links_apart() {
        let root = TempDir::new().expect("tempdir");
        let real = root.path().join("real");
        fs::create_dir(&real).expect("real");
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");
        fs::write(real.join(".env"), "x").expect(".env");
        for (denied, cwd) in [(&real, &link), (&link, &real)] {
            let policy = deny_in_workspace(&denied.join(".env"));
            let other = if denied == &real { &link } else { &real };
            assert!(
                policy.can_read_effective_path_with_cwd(&other.join(".env"), cwd),
                "{other:?}/.env counted as denied by {denied:?}/.env"
            );
            assert!(!policy.can_read_effective_path_with_cwd(&denied.join(".env"), denied));
        }
    }

    #[test]
    fn resolve_access_for_local_path_with_cwd_uses_most_specific_entry() {
        let cwd = TempDir::new().expect("tempdir");
        let docs = AbsolutePathBuf::resolve_path_against_base("docs", cwd.path());
        let docs_private = AbsolutePathBuf::resolve_path_against_base("docs/private", cwd.path());
        let docs_private_public =
            AbsolutePathBuf::resolve_path_against_base("docs/private/public", cwd.path());
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: docs.clone().into(),
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Path {
                    path: docs_private.clone().into(),
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Path {
                    path: docs_private_public.clone().into(),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
        ]);

        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(cwd.path(), cwd.path()),
            FileSystemAccessMode::Write
        );
        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(docs.as_path(), cwd.path()),
            FileSystemAccessMode::Read
        );
        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(docs_private.as_path(), cwd.path()),
            FileSystemAccessMode::Deny
        );
        assert_eq!(
            policy
                .resolve_access_for_local_path_with_cwd(docs_private_public.as_path(), cwd.path()),
            FileSystemAccessMode::Write
        );
    }

    #[test]
    fn split_only_nested_carveouts_need_direct_runtime_enforcement() {
        let cwd = TempDir::new().expect("tempdir");
        let docs = AbsolutePathBuf::resolve_path_against_base("docs", cwd.path());
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: docs.into(),
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
        ]);

        assert!(
            policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path(),)
        );

        let legacy_workspace_write = legacy_runtime_file_system_policy_for_cwd(
            &SandboxPolicy::new_workspace_write_policy(),
            cwd.path(),
        );
        assert!(
            legacy_workspace_write
                .needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path(),),
            "metadata-name protections must stay in the direct enforcement path even when legacy concrete read-only paths match"
        );
    }

    #[test]
    fn legacy_projection_runtime_enforcement_ignores_entry_order() {
        let cwd = TempDir::new().expect("tempdir");
        let legacy_policy = SandboxPolicy::WorkspaceWrite {
            writable_roots: Vec::new(),
            network_access: false,
            exclude_tmpdir_env_var: true,
            exclude_slash_tmp: true,
        };
        let legacy_order = legacy_runtime_file_system_policy_for_cwd(&legacy_policy, cwd.path());
        let mut reordered_entries = legacy_order.entries.clone();
        reordered_entries.reverse();
        let reordered = FileSystemSandboxPolicy::restricted(reordered_entries);

        assert!(
            legacy_order.is_semantically_equivalent_to(&reordered, cwd.path()),
            "entry order should not affect filesystem semantics"
        );
        assert_eq!(
            legacy_order
                .needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path()),
            reordered
                .needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path()),
            "entry order should not affect direct-enforcement classification"
        );
    }

    #[test]
    fn missing_symbolic_metadata_carveouts_need_direct_runtime_enforcement() {
        let cwd = TempDir::new().expect("tempdir");
        let legacy_policy = SandboxPolicy::WorkspaceWrite {
            writable_roots: Vec::new(),
            network_access: false,
            exclude_tmpdir_env_var: true,
            exclude_slash_tmp: true,
        };

        let profile_projection =
            FileSystemSandboxPolicy::from_legacy_sandbox_policy_for_cwd(&legacy_policy, cwd.path());
        assert!(
            profile_projection
                .needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path()),
            "symbolic .git/.agents carveouts protect missing paths that legacy sandboxes cannot represent"
        );

        let legacy_runtime_projection =
            legacy_runtime_file_system_policy_for_cwd(&legacy_policy, cwd.path());
        assert!(
            legacy_runtime_projection
                .needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path()),
            "metadata-name protections are outside the legacy SandboxPolicy writable-root contract"
        );
    }

    #[test]
    fn root_write_with_read_only_child_is_not_full_disk_write() {
        let cwd = TempDir::new().expect("tempdir");
        let docs = AbsolutePathBuf::resolve_path_against_base("docs", cwd.path());
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: docs.clone().into(),
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
        ]);

        assert!(!policy.has_full_disk_write_access());
        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(docs.as_path(), cwd.path()),
            FileSystemAccessMode::Read
        );
        assert!(
            policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd.path(),)
        );
        assert!(
            policy
                .to_legacy_sandbox_policy(NetworkSandboxPolicy::Restricted, cwd.path())
                .is_err()
        );
    }

    #[test]
    fn root_deny_does_not_materialize_as_unreadable_root() {
        let cwd = TempDir::new().expect("tempdir");
        let docs = AbsolutePathBuf::resolve_path_against_base("docs", cwd.path());
        let expected_docs = AbsolutePathBuf::from_absolute_path(
            canonicalize_preserving_symlinks(cwd.path())
                .expect("canonicalize cwd")
                .join("docs"),
        )
        .expect("canonical docs");
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: docs.clone().into(),
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
        ]);

        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(docs.as_path(), cwd.path()),
            FileSystemAccessMode::Read
        );
        assert_eq!(
            policy.get_readable_roots_with_cwd(cwd.path()),
            vec![expected_docs]
        );
        assert!(policy.get_unreadable_roots_with_cwd(cwd.path()).is_empty());
    }

    #[test]
    fn duplicate_root_deny_prevents_full_disk_write_access() {
        let cwd = TempDir::new().expect("tempdir");
        let root = AbsolutePathBuf::from_absolute_path(cwd.path())
            .map(|cwd| absolute_root_path_for_cwd(&cwd))
            .expect("resolve filesystem root");
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        assert!(!policy.has_full_disk_write_access());
        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(root.as_path(), cwd.path()),
            FileSystemAccessMode::Deny
        );
    }

    #[test]
    fn same_specificity_write_override_keeps_full_disk_write_access() {
        let cwd = TempDir::new().expect("tempdir");
        let docs = AbsolutePathBuf::resolve_path_against_base("docs", cwd.path());
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: docs.clone().into(),
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: docs.clone().into(),
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
        ]);

        assert!(policy.has_full_disk_write_access());
        assert_eq!(
            policy.resolve_access_for_local_path_with_cwd(docs.as_path(), cwd.path()),
            FileSystemAccessMode::Write
        );
    }

    #[test]
    fn with_additional_readable_roots_skips_existing_effective_access() {
        let cwd = TempDir::new().expect("tempdir");
        let cwd_root = AbsolutePathBuf::from_absolute_path(cwd.path()).expect("absolute cwd");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Read,
            missing_path_behavior: None,
        }]);

        let actual = policy
            .clone()
            .with_additional_readable_roots(cwd.path(), std::slice::from_ref(&cwd_root));

        assert_eq!(actual, policy);
    }

    #[test]
    fn with_additional_writable_roots_skips_existing_effective_access() {
        let cwd = TempDir::new().expect("tempdir");
        let cwd_root = AbsolutePathBuf::from_absolute_path(cwd.path()).expect("absolute cwd");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        }]);

        let actual = policy
            .clone()
            .with_additional_writable_roots(cwd.path(), std::slice::from_ref(&cwd_root));

        assert_eq!(actual, policy);
    }

    #[test]
    fn with_additional_writable_roots_adds_new_root() {
        let temp_dir = TempDir::new().expect("tempdir");
        let cwd = temp_dir.path().join("workspace");
        let extra = AbsolutePathBuf::from_absolute_path(temp_dir.path().join("extra"))
            .expect("resolve extra root");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        }]);

        let actual = policy.with_additional_writable_roots(&cwd, std::slice::from_ref(&extra));

        assert_eq!(
            actual,
            FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: extra.into(),
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
            ])
        );
    }

    #[test]
    fn materialize_project_roots_with_workspace_roots_expands_exact_and_glob_entries() {
        let temp_dir = TempDir::new().expect("tempdir");
        let first = AbsolutePathBuf::from_absolute_path(temp_dir.path().join("first"))
            .expect("resolve first root");
        let second = AbsolutePathBuf::from_absolute_path(temp_dir.path().join("second"))
            .expect("resolve second root");
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::project_roots(Some(".git".into())),
                },
                access: FileSystemAccessMode::Read,
                missing_path_behavior: None,
            },
            FileSystemSandboxEntry {
                path: FileSystemPath::GlobPattern {
                    scan: None,
                    pattern: project_roots_glob_pattern(Path::new("**/*.env")),
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            },
        ]);

        let actual =
            policy.materialize_project_roots_with_workspace_roots(&[first.clone(), second.clone()]);

        assert_eq!(
            actual,
            FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: first.clone().into(),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: second.clone().into(),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: first.join(".git").into(),
                    },
                    access: FileSystemAccessMode::Read,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: second.join(".git").into(),
                    },
                    access: FileSystemAccessMode::Read,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::GlobPattern {
                        scan: None,
                        pattern: AbsolutePathBuf::resolve_path_against_base(
                            "**/*.env",
                            first.as_path(),
                        )
                        .to_string_lossy()
                        .into_owned(),
                    },
                    access: FileSystemAccessMode::Deny,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::GlobPattern {
                        scan: None,
                        pattern: AbsolutePathBuf::resolve_path_against_base(
                            "**/*.env",
                            second.as_path(),
                        )
                        .to_string_lossy()
                        .into_owned(),
                    },
                    access: FileSystemAccessMode::Deny,
                    missing_path_behavior: None,
                },
            ])
        );
    }

    #[test]
<<<<<<< HEAD
    fn materialize_project_roots_with_cwd_expands_symbolic_glob_entries() {
        let cwd = TempDir::new().expect("tempdir");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                scan: None,
                pattern: project_roots_glob_pattern(Path::new("**/*.env")),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }]);

        let actual = policy.materialize_project_roots_with_cwd(cwd.path());

        assert_eq!(
            actual,
            FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
                path: FileSystemPath::GlobPattern {
                    scan: None,
                    pattern: AbsolutePathBuf::resolve_path_against_base("**/*.env", cwd.path())
                        .to_string_lossy()
                        .into_owned(),
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            }])
        );
    }

    #[test]
||||||| 413492cd6c
    fn materialize_project_roots_with_cwd_expands_symbolic_glob_entries() {
        let cwd = TempDir::new().expect("tempdir");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern: project_roots_glob_pattern(Path::new("**/*.env")),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }]);

        let actual = policy.materialize_project_roots_with_cwd(cwd.path());

        assert_eq!(
            actual,
            FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
                path: FileSystemPath::GlobPattern {
                    pattern: AbsolutePathBuf::resolve_path_against_base("**/*.env", cwd.path())
                        .to_string_lossy()
                        .into_owned(),
                },
                access: FileSystemAccessMode::Deny,
                missing_path_behavior: None,
            }])
        );
    }

    #[test]
=======
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    fn with_additional_legacy_workspace_writable_roots_protects_metadata() {
        let temp_dir = TempDir::new().expect("tempdir");
        let extra = AbsolutePathBuf::from_absolute_path(temp_dir.path().join("extra"))
            .expect("resolve extra root");
        std::fs::create_dir_all(extra.join(".git")).expect("create .git dir");
        let policy = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Special {
                value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
            },
            access: FileSystemAccessMode::Write,
            missing_path_behavior: None,
        }]);

        let actual =
            policy.with_additional_legacy_workspace_writable_roots(std::slice::from_ref(&extra));

        assert_eq!(
            actual,
            FileSystemSandboxPolicy::restricted(vec![
                FileSystemSandboxEntry {
                    path: FileSystemPath::Special {
                        value: FileSystemSpecialPath::project_roots(/*subpath*/ None),
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry {
                    path: FileSystemPath::Path {
                        path: extra.clone().into()
                    },
                    access: FileSystemAccessMode::Write,
                    missing_path_behavior: None,
                },
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Path {
                        path: extra.join(".git").into()
                    },
                    FileSystemAccessMode::Read,
                ),
            ])
        );
    }

    #[test]
    fn file_system_access_mode_orders_by_conflict_precedence() {
        assert!(FileSystemAccessMode::Write > FileSystemAccessMode::Read);
        assert!(FileSystemAccessMode::Deny > FileSystemAccessMode::Write);
    }

    #[test]
    fn legacy_bridge_preserves_explicit_deny_entries() {
        let denied = AbsolutePathBuf::try_from("/tmp/private").expect("absolute path");
        let existing = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: denied.clone().into(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }]);

        let rebuilt = FileSystemSandboxPolicy::from_legacy_sandbox_policy_preserving_deny_entries(
            &SandboxPolicy::new_workspace_write_policy(),
            Path::new("/tmp/workspace"),
            &existing,
        );

        assert!(
            rebuilt.entries.iter().any(|entry| {
                entry.path
                    == FileSystemPath::Path {
                        path: denied.clone().into(),
                    }
                    && entry.access == FileSystemAccessMode::Deny
            }),
            "expected explicit deny entry to be preserved"
        );
    }

    #[test]
    fn preserving_deny_entries_keeps_unrestricted_policy_enforceable() {
        let deny_entry = unreadable_glob_entry("/tmp/project/**/*.env".to_string());
        let mut existing = FileSystemSandboxPolicy::restricted(vec![deny_entry.clone()]);
        existing.glob_scan_max_depth = Some(2);
        let mut replacement = FileSystemSandboxPolicy::unrestricted();

        replacement.preserve_deny_read_restrictions_from(&existing);

        let mut expected = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry {
                path: FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                access: FileSystemAccessMode::Write,
                missing_path_behavior: None,
            },
            deny_entry,
        ]);
        expected.glob_scan_max_depth = Some(2);
        assert_eq!(replacement, expected);
    }

    fn deny_policy(path: &Path) -> FileSystemSandboxPolicy {
        FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry {
            path: FileSystemPath::Path {
                path: AbsolutePathBuf::try_from(path)
                    .expect("absolute deny path")
                    .into(),
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }])
    }

    fn unreadable_glob_entry(pattern: String) -> FileSystemSandboxEntry {
        FileSystemSandboxEntry {
            path: FileSystemPath::GlobPattern {
                pattern,
                scan: None,
            },
            access: FileSystemAccessMode::Deny,
            missing_path_behavior: None,
        }
    }

    fn default_policy_with_unreadable_glob(pattern: String) -> FileSystemSandboxPolicy {
        let mut policy = FileSystemSandboxPolicy::default();
        policy.entries.push(unreadable_glob_entry(pattern));
        policy
    }

    fn is_read_denied(
        path: &Path,
        file_system_sandbox_policy: &FileSystemSandboxPolicy,
        cwd: &Path,
    ) -> bool {
        ReadDenyMatcher::try_new_for_local_paths(file_system_sandbox_policy, cwd)
            .expect("valid deny-read globs")
            .is_some_and(|matcher| matcher.is_local_path_read_denied(path))
    }

    #[test]
    fn exact_path_and_descendants_are_denied() {
        let temp = TempDir::new().expect("tempdir");
        let denied_dir = temp.path().join("denied");
        let nested = denied_dir.join("nested.txt");
        std::fs::create_dir_all(&denied_dir).expect("create denied dir");
        std::fs::write(&nested, "secret").expect("write secret");

        let policy = deny_policy(&denied_dir);
        assert!(is_read_denied(&denied_dir, &policy, temp.path()));
        assert!(is_read_denied(&nested, &policy, temp.path()));
        assert!(!is_read_denied(
            &temp.path().join("other.txt"),
            &policy,
            temp.path()
        ));
    }

    #[cfg(unix)]
    #[test]
    fn native_read_deny_matching_uses_uri_matcher_semantics() {
        let temp = TempDir::new().expect("tempdir");
        let real_dir = temp.path().join("real");
        let alias_dir = temp.path().join("alias");
        std::fs::create_dir_all(&real_dir).expect("create real dir");
        symlink_dir(&real_dir, &alias_dir).expect("symlink alias");

        let secret = real_dir.join("secret.txt");
        std::fs::write(&secret, "secret").expect("write secret");
        let alias_secret = alias_dir.join("secret.txt");

        for (denied_root, candidate) in [(&real_dir, &alias_secret), (&alias_dir, &secret)] {
            let policy = deny_policy(denied_root);
            let denied_root_uri =
                PathUri::from_host_native_path(denied_root).expect("deny root URI");
            let candidate_uri = PathUri::from_host_native_path(candidate).expect("candidate URI");

            assert_eq!(
                is_read_denied(candidate, &policy, temp.path()),
                candidate_uri.starts_with(&denied_root_uri)
            );
        }
    }

    #[test]
    fn literal_patterns_and_globs_are_denied() {
        let temp = TempDir::new().expect("tempdir");
        let literal = temp.path().join("private");
        let other = temp.path().join("notes.txt");
        std::fs::create_dir_all(&literal).expect("create literal dir");
        std::fs::write(&other, "notes").expect("write notes");

        let mut policy = deny_policy(&literal);
        policy.entries.push(unreadable_glob_entry(format!(
            "{}/**/*.txt",
            temp.path().display()
        )));

        assert!(is_read_denied(&literal, &policy, temp.path()));
        assert!(is_read_denied(&other, &policy, temp.path()));
    }

    #[test]
    fn glob_patterns_deny_matching_paths() {
        let temp = TempDir::new().expect("tempdir");
        let denied = temp.path().join("private").join("secret1.txt");
        std::fs::create_dir_all(denied.parent().expect("parent")).expect("create parent");
        std::fs::write(&denied, "secret").expect("write secret");

        let policy = default_policy_with_unreadable_glob(format!(
            "{}/private/secret?.txt",
            temp.path().display()
        ));

        assert!(is_read_denied(&denied, &policy, temp.path()));

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;

            let non_utf8 = denied
                .parent()
                .expect("parent")
                .join(OsStr::from_bytes(b"secret\xff.txt"));
            assert!(is_read_denied(&non_utf8, &policy, temp.path()));
        }
    }

    #[test]
    fn glob_patterns_do_not_cross_path_separators() {
        let temp = TempDir::new().expect("tempdir");
        let matching = temp.path().join("app").join("file42.txt");
        let nested = temp.path().join("app").join("nested").join("file42.txt");
        let short = temp.path().join("app").join("file4.txt");
        let letters = temp.path().join("app").join("fileab.txt");
        std::fs::create_dir_all(nested.parent().expect("parent")).expect("create parent");
        std::fs::write(&matching, "secret").expect("write matching");
        std::fs::write(&nested, "secret").expect("write nested");
        std::fs::write(&short, "secret").expect("write short");
        std::fs::write(&letters, "secret").expect("write letters");

        let policy = default_policy_with_unreadable_glob(format!(
            "{}/*/file[0-9]?.txt",
            temp.path().display()
        ));

        assert!(is_read_denied(&matching, &policy, temp.path()));
        assert!(!is_read_denied(&nested, &policy, temp.path()));
        assert!(!is_read_denied(&short, &policy, temp.path()));
        assert!(!is_read_denied(&letters, &policy, temp.path()));
    }

    #[test]
    fn globstar_patterns_deny_root_and_nested_matches() {
        let temp = TempDir::new().expect("tempdir");
        let root_env = temp.path().join(".env");
        let nested_env = temp.path().join("app").join(".env");
        let other = temp.path().join("app").join("notes.txt");
        std::fs::create_dir_all(nested_env.parent().expect("parent")).expect("create parent");
        std::fs::write(&root_env, "secret").expect("write root env");
        std::fs::write(&nested_env, "secret").expect("write nested env");
        std::fs::write(&other, "notes").expect("write notes");

        let policy =
            default_policy_with_unreadable_glob(format!("{}/**/*.env", temp.path().display()));

        assert!(is_read_denied(&root_env, &policy, temp.path()));
        assert!(is_read_denied(&nested_env, &policy, temp.path()));
        assert!(!is_read_denied(&other, &policy, temp.path()));
    }

    #[test]
    fn unclosed_character_classes_match_literal_brackets() {
        let temp = TempDir::new().expect("tempdir");
        let bracket_file = temp.path().join("[");
        let other = temp.path().join("notes.txt");
        std::fs::write(&bracket_file, "secret").expect("write bracket file");
        std::fs::write(&other, "notes").expect("write notes");
        let policy = default_policy_with_unreadable_glob(format!("{}/[", temp.path().display()));

        assert!(is_read_denied(&bracket_file, &policy, temp.path()));
        assert!(!is_read_denied(&other, &policy, temp.path()));
    }

    /// #442: the credential name patterns, the globs built from them, and the
    /// folder each glob covers.
    #[test]
    fn credential_file_names_globs_and_roots() {
        for name in [
            ".env",
            ".env.local",
            ".env.",
            ".envrc",
            ".ENV",
            ".Env.Local",
            ".ENVRC",
        ] {
            assert!(is_credential_file_name(OsStr::new(name)), "{name}");
        }
        for name in [".envrcx", "x.env", ".environment", "env", ".en"] {
            assert!(!is_credential_file_name(OsStr::new(name)), "{name}");
        }
        #[cfg(unix)]
        {
            assert_eq!(
                credential_file_globs("/work/"),
                vec!["/work/**/.env", "/work/**/.env.*", "/work/**/.envrc"]
            );
            assert_eq!(
                credential_file_globs("/"),
                vec!["/**/.env", "/**/.env.*", "/**/.envrc"]
            );
        }
        // Windows resolves policy globs with `\\`.
        #[cfg(windows)]
        for glob in [r"C:\work\**\.env.*", "C:/work/**/.env.*"] {
            assert_eq!(
                credential_file_glob_root(glob),
                Some(PathBuf::from(&glob[..7]))
            );
        }
        for root in ["/work", "/"] {
            for glob in credential_file_globs(root) {
                assert_eq!(
                    credential_file_glob_root(&glob),
                    Some(PathBuf::from(root)),
                    "{glob}"
                );
            }
        }
        for other in ["/work/**/*.pem", "/work/.env", "/work/*/.env"] {
            assert_eq!(credential_file_glob_root(other), None, "{other}");
        }

        // The policy's own matcher denies every credential name below the
        // root, at any depth, and nothing else.
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path().to_string_lossy().into_owned();
        let policy = FileSystemSandboxPolicy::restricted(
            credential_file_globs(&root)
                .into_iter()
                .map(|pattern| {
                    FileSystemSandboxEntry::new(
                        FileSystemPath::GlobPattern {
                            pattern,
                            scan: None,
                        },
                        FileSystemAccessMode::Deny,
                    )
                })
                .collect(),
        );
        for denied in [
            ".env",
            "app/.env.local",
            "a/b/c/.envrc",
            "app/.ENV",
            ".Env.local",
        ] {
            assert!(
                is_read_denied(&temp.path().join(denied), &policy, temp.path()),
                "{denied}"
            );
        }
        for allowed in ["env", "app/x.env", ".environment/x"] {
            assert!(
                !is_read_denied(&temp.path().join(allowed), &policy, temp.path()),
                "{allowed}"
            );
        }
    }

    /// #442 (Travis, 2026-10-12): `.env.example`, `.env.sample` and
    /// `.env.template`, in any case, are not credential files: the names and
    /// the policy's credential globs leave them readable. Everything else the
    /// globs cover stays denied, also a template name that is a link to one.
    #[test]
    fn credential_templates_are_exempt() {
        for name in [
            ".env.example",
            ".env.sample",
            ".env.template",
            ".ENV.EXAMPLE",
            ".Env.Sample",
            ".env.TEMPLATE",
        ] {
            assert!(is_credential_template_name(OsStr::new(name)), "{name}");
            assert!(!is_credential_file_name(OsStr::new(name)), "{name}");
        }
        for name in [
            ".env.example.local",
            ".env.examples",
            ".env.local",
            ".env.sample.bak",
            "env.example",
        ] {
            assert!(!is_credential_template_name(OsStr::new(name)), "{name}");
        }
        assert!(is_credential_file_name(OsStr::new(".env.example.local")));

        let temp_dir = TempDir::new().expect("tempdir");
        // The real location, so a link's target matches the globs too.
        let temp = std::fs::canonicalize(temp_dir.path()).expect("real");
        let root = temp.to_string_lossy().into_owned();
        let policy = FileSystemSandboxPolicy::restricted(
            credential_file_globs(&root)
                .into_iter()
                .map(|pattern| {
                    FileSystemSandboxEntry::new(
                        FileSystemPath::GlobPattern {
                            pattern,
                            scan: None,
                        },
                        FileSystemAccessMode::Deny,
                    )
                })
                .collect(),
        );
        for allowed in [".env.example", "app/.env.sample", "a/.ENV.TEMPLATE"] {
            assert!(
                !is_read_denied(&temp.as_path().join(allowed), &policy, temp.as_path()),
                "{allowed}"
            );
        }
        for denied in [".env", ".env.local", "app/.env.example.local", ".envrc"] {
            assert!(
                is_read_denied(&temp.as_path().join(denied), &policy, temp.as_path()),
                "{denied}"
            );
        }
        // A person's own `**/.env.*` glob alone (not the whole rule) still
        // covers the templates, at any level.
        let own_env_star = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            FileSystemPath::GlobPattern {
                scan: None,
                pattern: format!("{root}/**/.env.*"),
            },
            FileSystemAccessMode::Deny,
        )]);
        assert!(is_read_denied(
            &temp.as_path().join(".env.example"),
            &own_env_star,
            temp.as_path()
        ));
        assert_eq!(
            credential_rule_roots(&credential_file_globs(&root)),
            vec![PathBuf::from(&root)]
        );
        assert_eq!(
            credential_rule_roots(&[format!("{root}/**/.env.*")]),
            Vec::<PathBuf>::new()
        );
        // A template name that is a link to a credential file stays denied.
        #[cfg(unix)]
        {
            fs::write(temp.as_path().join(".env"), "SECRET=1").expect("env");
            std::os::unix::fs::symlink(".env", temp.as_path().join(".env.sample")).expect("link");
            assert!(is_read_denied(
                &temp.as_path().join(".env.sample"),
                &policy,
                temp.as_path()
            ));
        }
        // A user's own deny glob still covers a template name.
        let mut entries = policy.entries;
        entries.push(FileSystemSandboxEntry::new(
            FileSystemPath::GlobPattern {
                scan: None,
                pattern: format!("{root}/**/*.example"),
            },
            FileSystemAccessMode::Deny,
        ));
        let own = FileSystemSandboxPolicy::restricted(entries);
        assert!(is_read_denied(
            &temp.as_path().join(".env.example"),
            &own,
            temp.as_path()
        ));
    }

    /// #442 (Travis, 2026-10-12): the credential rule as a profile writes it
    /// (`**/.env` and so on below a folder), resolved by the policy with the
    /// platform's separator, is recognised as the whole rule, so the
    /// templates are exempt on every system, Windows included.
    #[test]
    fn credential_rule_is_recognised_as_the_policy_resolves_it() {
        let temp = TempDir::new().expect("tempdir");
        let cwd = temp.path();
        let policy = FileSystemSandboxPolicy::restricted(
            CREDENTIAL_FILE_NAME_PATTERNS
                .iter()
                .map(|name| {
                    FileSystemSandboxEntry::new(
                        FileSystemPath::GlobPattern {
                            scan: None,
                            pattern: format!("**/{name}"),
                        },
                        FileSystemAccessMode::Deny,
                    )
                })
                .collect(),
        );
        let globs = policy.get_unreadable_globs_with_cwd(cwd);
        let roots = credential_rule_roots(&globs);
        assert_eq!(roots.len(), 1, "{globs:?}");
        let mut expected = credential_file_globs(&roots[0].to_string_lossy());
        expected.sort();
        assert_eq!(globs, expected);
        for allowed in [".env.example", "app/.env.sample"] {
            assert!(
                !is_read_denied(&cwd.join(allowed), &policy, cwd),
                "{allowed}"
            );
        }
        for denied in [".env", "app/.env.local", ".envrc"] {
            assert!(is_read_denied(&cwd.join(denied), &policy, cwd), "{denied}");
        }
    }

    /// #442: which existing credential-named entries are unsafe because what
    /// the person writes to them would land somewhere readable.
    #[cfg(unix)]
    #[test]
    fn credential_entry_problems() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::write(root.join(".env"), "").expect("env");
        fs::write(root.join("linked"), "").expect("linked");
        fs::hard_link(root.join("linked"), root.join(".env.hard")).expect("hard");
        fs::create_dir(root.join("venv")).expect("dir");
        std::os::unix::fs::symlink(".env", root.join(".env.link")).expect("link");
        std::os::unix::fs::symlink("venv", root.join(".env.dir")).expect("link");
        let problem = |name: &str| {
            credential_entry_problem(&fs::symlink_metadata(root.join(name)).expect("metadata"))
        };
        assert_eq!(problem(".env"), None);
        assert_eq!(problem(".env.hard"), Some("has other hard links"));
        assert_eq!(problem(".env.link"), Some("is a link"));
        assert_eq!(problem(".env.dir"), Some("is a link"));
    }

    /// #506: a directory-listing denial is its own kind of entry: it denies
    /// listing that one directory, never reading or writing it or anything
    /// below it, and runtimes see it only through its accessors.
    #[test]
    fn issue_506_directory_listing_denial_only_denies_listing() {
        let temp = TempDir::new().expect("tempdir");
        let base = temp.path().canonicalize().expect("canonical");
        let home = AbsolutePathBuf::from_absolute_path(base.join("home")).expect("home");
        std::fs::create_dir_all(home.join("tmp")).expect("tmp");
        let entry = FileSystemSandboxEntry::deny_directory_listing(&home);
        assert_eq!(entry.unlistable_directory(), Some(home.clone()));
        let mut read = entry.clone();
        read.access = FileSystemAccessMode::Read;
        assert_eq!(read.unlistable_directory(), None);

        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Write,
            ),
            entry,
        ]);
        let cwd = base.as_path();
        assert_eq!(policy.get_unlistable_directories(), vec![home.clone()]);
        assert!(policy.has_full_disk_write_access());
        assert!(policy.get_unreadable_roots_with_cwd(cwd).is_empty());
        assert!(policy.get_unreadable_globs_with_cwd(cwd).is_empty());
        assert!(policy.can_write_path_with_cwd(home.as_path(), cwd));
        assert!(policy.can_read_path_with_cwd(home.join("tmp").as_path(), cwd));
        // A runtime that knows the entry cannot run it as plain full access.
        assert!(policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd));
        assert!(!policy.is_semantically_equivalent_to(
            &FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Write,
            )]),
            cwd,
        ));

        let matcher = ReadDenyMatcher::new(&policy, cwd).expect("matcher");
        assert!(matcher.is_listing_denied(home.as_path()));
        assert!(matcher.is_listing_denied(&home.as_path().join("tmp").join("..")));
        assert!(!matcher.is_read_denied(home.as_path()));
        assert!(!matcher.is_listing_denied(home.join("tmp").as_path()));
        assert!(!matcher.is_read_denied(home.join("tmp").as_path()));
        // Not carried by a policy that does not restrict.
        assert!(
            FileSystemSandboxPolicy::unrestricted()
                .get_unlistable_directories()
                .is_empty()
        );
    }

    /// #594/#598: a no-move entry is its own kind of entry: it changes no
    /// read or write access, keeps the path as written (a link spelling
    /// too), and needs the sandbox that enforces it.
    #[cfg(unix)]
    #[test]
    fn issue_598_no_move_entry_changes_no_access() {
        let temp = TempDir::new().expect("tempdir");
        let base = temp.path().canonicalize().expect("canonical");
        let home = AbsolutePathBuf::from_absolute_path(base.join("home")).expect("home");
        std::fs::create_dir_all(home.as_path()).expect("home");
        let link = base.join("link");
        std::os::unix::fs::symlink(home.as_path(), &link).expect("link");
        let link = AbsolutePathBuf::from_absolute_path(link).expect("link");
        let entry = FileSystemSandboxEntry::deny_moving(&link);
        assert_eq!(entry.unmovable_path(), Some(link.clone()));
        assert_eq!(entry.unlistable_directory(), None);
        let mut read = entry.clone();
        read.access = FileSystemAccessMode::Read;
        assert_eq!(read.unmovable_path(), None);

        let full = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        )]);
        let mut policy = full.clone();
        policy.entries.push(entry.clone());
        policy.entries.push(entry);
        let cwd = base.as_path();
        assert_eq!(policy.get_unmovable_paths(), vec![link]);
        assert!(full.get_unmovable_paths().is_empty());
        assert!(policy.has_full_disk_write_access());
        assert!(policy.get_unreadable_roots_with_cwd(cwd).is_empty());
        assert!(!policy.has_denied_read_restrictions());
        assert!(policy.can_write_path_with_cwd(home.as_path(), cwd));
        assert!(policy.can_read_path_with_cwd(home.as_path(), cwd));
        assert!(policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd));
        assert!(!policy.is_semantically_equivalent_to(&full, cwd));
        assert!(
            FileSystemSandboxPolicy::unrestricted()
                .get_unmovable_paths()
                .is_empty()
        );
    }

    /// #758: a keychain-services entry is its own kind of entry: it changes
    /// no file access, is no read denial, and needs the sandbox that
    /// enforces it.
    #[test]
    fn issue_758_keychain_services_entry_changes_no_file_access() {
        let temp = TempDir::new().expect("tempdir");
        let cwd = temp.path();
        let entry = FileSystemSandboxEntry::deny_keychain_services();
        assert!(entry.denies_keychain_services());
        assert_eq!(entry.unlistable_directory(), None);
        assert_eq!(entry.unmovable_path(), None);
        let mut read = entry.clone();
        read.access = FileSystemAccessMode::Read;
        assert!(!read.denies_keychain_services());

        let full = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        )]);
        let mut policy = full.clone();
        policy.entries.push(entry.clone());
        assert!(policy.denies_keychain_services());
        assert!(!full.denies_keychain_services());
        assert!(policy.has_full_disk_write_access());
        assert!(policy.has_full_disk_read_access());
        assert!(!policy.has_denied_read_restrictions());
        assert!(policy.get_unreadable_roots_with_cwd(cwd).is_empty());
        assert!(policy.get_unreadable_globs_with_cwd(cwd).is_empty());
        assert!(policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd));
        assert!(!policy.is_semantically_equivalent_to(&full, cwd));
        let mut unrestricted = FileSystemSandboxPolicy::unrestricted();
        unrestricted.entries.push(entry);
        assert!(!unrestricted.denies_keychain_services());
    }

    /// #826: a user-namespaces entry changes no file access, is no read
    /// denial, and needs the sandbox that enforces it.
    #[test]
    fn issue_826_user_namespaces_entry_changes_no_file_access() {
        let temp = TempDir::new().expect("tempdir");
        let cwd = temp.path();
        let entry = FileSystemSandboxEntry::deny_user_namespaces();
        assert!(entry.denies_user_namespaces());
        assert!(!entry.denies_keychain_services());
        assert!(!FileSystemSandboxEntry::deny_keychain_services().denies_user_namespaces());
        assert_eq!(entry.unlistable_directory(), None);
        assert_eq!(entry.unmovable_path(), None);
        let mut read = entry.clone();
        read.access = FileSystemAccessMode::Read;
        assert!(!read.denies_user_namespaces());

        let full = FileSystemSandboxPolicy::restricted(vec![FileSystemSandboxEntry::new(
            FileSystemPath::Special {
                value: FileSystemSpecialPath::Root,
            },
            FileSystemAccessMode::Write,
        )]);
        let mut policy = full.clone();
        policy.entries.push(entry.clone());
        assert!(policy.denies_user_namespaces());
        assert!(!full.denies_user_namespaces());
        assert!(policy.has_full_disk_write_access());
        assert!(policy.has_full_disk_read_access());
        assert!(!policy.has_denied_read_restrictions());
        assert!(policy.get_unreadable_roots_with_cwd(cwd).is_empty());
        assert!(policy.get_unreadable_globs_with_cwd(cwd).is_empty());
        assert!(policy.needs_direct_runtime_enforcement(NetworkSandboxPolicy::Restricted, cwd));
        assert!(!policy.is_semantically_equivalent_to(&full, cwd));
        let mut unrestricted = FileSystemSandboxPolicy::unrestricted();
        unrestricted.entries.push(entry);
        assert!(!unrestricted.denies_user_namespaces());
    }

    /// #535: a writable link whose real location is, or lies inside, another
    /// root's read-only path (also a protected name not created yet, and
    /// through a nested link) is left out and reported; an explicit root
    /// spelled through the read-only path and a link elsewhere are kept.
    #[cfg(unix)]
    #[test]
    fn writable_links_into_read_only_paths_are_left_out() {
        let dir = TempDir::new().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonical");
        let workspace = base.join("ws");
        let hooks = workspace.join(".git").join("hooks");
        fs::create_dir_all(&hooks).expect("hooks");
        let elsewhere = base.join("elsewhere");
        fs::create_dir_all(&elsewhere).expect("elsewhere");
        let links = base.join("links");
        fs::create_dir_all(&links).expect("links");
        symlink_dir(&hooks, &links.join("hooks")).expect("hooks link");
        symlink_dir(&workspace.join(".git"), &links.join("git")).expect("git link");
        symlink_dir(&workspace.join(".codex"), &links.join("codex")).expect("dangling link");
        symlink_dir(Path::new("../ws/.git"), &links.join("relative")).expect("relative link");
        symlink_dir(&elsewhere, &links.join("elsewhere")).expect("other link");
        let nested = links.join("git").join("hooks");
        let entry = |path: &Path| {
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
                },
                FileSystemAccessMode::Write,
            )
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            entry(&workspace),
            entry(&links.join("hooks")),
            entry(&links.join("git")),
            entry(&links.join("codex")),
            entry(&links.join("relative")),
            entry(&nested),
            entry(&links.join("elsewhere")),
            entry(&hooks),
        ]);
        let mut excluded: Vec<(PathBuf, PathBuf)> = policy
            .writable_links_into_read_only_paths(&workspace)
            .into_iter()
            .map(|exclusion| {
                (
                    exclusion.link.into_path_buf(),
                    exclusion.read_only_path.into_path_buf(),
                )
            })
            .collect();
        excluded.sort();
        let git = workspace.join(".git");
        assert_eq!(
            excluded,
            vec![
                (links.join("codex"), workspace.join(".codex")),
                (links.join("git"), git.clone()),
                (nested, git.clone()),
                (links.join("hooks"), git.clone()),
                (links.join("relative"), git),
            ]
        );
        let roots: Vec<PathBuf> = policy
            .get_writable_roots_with_cwd(&workspace)
            .into_iter()
            .map(|root| root.root.into_path_buf())
            .collect();
        assert_eq!(roots, vec![workspace, links.join("elsewhere"), hooks]);
    }

    /// #606: approvals agree with the sandboxes, which grant nothing through
    /// a writable link they leave out (#535): a path through one is not
    /// writable (fail closed); other paths are as before.
    #[cfg(unix)]
    #[test]
    fn paths_through_excluded_links_are_not_writable_for_approvals() {
        let dir = TempDir::new().expect("tempdir");
        let base = dir.path().canonicalize().expect("canonical");
        let workspace = base.join("ws");
        fs::create_dir_all(workspace.join(".git").join("hooks")).expect("hooks");
        let link = base.join("git-link");
        symlink_dir(&workspace.join(".git"), &link).expect("link");
        let entry = |path: &Path| {
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: AbsolutePathBuf::from_absolute_path(path).expect("absolute"),
                },
                FileSystemAccessMode::Write,
            )
        };
        let policy = FileSystemSandboxPolicy::restricted(vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            entry(&workspace),
            entry(&link),
        ]);
        let excluded = policy.writable_links_into_read_only_paths(&workspace);
        assert_eq!(excluded.len(), 1);
        let hook = link.join("hooks").join("pre-commit");
        // The literal check still counts it (the disagreement #606 names).
        assert!(policy.can_write_path_with_cwd(&hook, &workspace));
        for path in [hook.as_path(), link.as_path(), &link.join("config")] {
            assert!(
                !policy.can_write_path_outside_excluded_links(path, &workspace, &excluded),
                "{}",
                path.display()
            );
        }
        assert!(policy.can_write_path_outside_excluded_links(
            &workspace.join("src/main.rs"),
            &workspace,
            &excluded
        ));
        assert!(!policy.can_write_path_outside_excluded_links(
            &workspace.join(".git/config"),
            &workspace,
            &excluded
        ));
        assert!(policy.can_write_path_outside_excluded_links(
            &workspace.join("src/main.rs"),
            &workspace,
            &[]
        ));
    }

    /// #606: macOS volumes ignore case, so `.GIT`, `.Codex` and `.AGENTS`
    /// are protected like `.git`, `.codex` and `.agents` once the policy
    /// denies reads; a policy without denied paths keeps today's rules.
    #[cfg(target_os = "macos")]
    #[test]
    fn metadata_names_match_in_any_case_once_reads_are_denied() {
        let dir = TempDir::new().expect("tempdir");
        let workspace = dir.path().canonicalize().expect("canonical").join("ws");
        let absolute = |path: &Path| AbsolutePathBuf::from_absolute_path(path).expect("absolute");
        let entries = vec![
            FileSystemSandboxEntry::new(
                FileSystemPath::Special {
                    value: FileSystemSpecialPath::Root,
                },
                FileSystemAccessMode::Read,
            ),
            FileSystemSandboxEntry::new(
                FileSystemPath::Path {
                    path: absolute(&workspace),
                },
                FileSystemAccessMode::Write,
            ),
        ];
        let open = FileSystemSandboxPolicy::restricted(entries.clone());
        let mut denying = entries;
        denying.push(FileSystemSandboxEntry::new(
            FileSystemPath::Path {
                path: absolute(&dir.path().join("denied")),
            },
            FileSystemAccessMode::Deny,
        ));
        let denying = FileSystemSandboxPolicy::restricted(denying);
        for (name, metadata) in [
            (".GIT", ".git"),
            (".Codex", ".codex"),
            (".AGENTS", ".agents"),
        ] {
            let path = workspace.join(name).join("config");
            assert!(
                !denying.can_write_path_with_cwd(&path, &workspace),
                "{name}"
            );
            assert_eq!(
                forbidden_agent_metadata_write(&path, &workspace, &denying),
                Some(metadata),
                "{name}"
            );
            assert!(open.can_write_path_with_cwd(&path, &workspace), "{name}");
            assert_eq!(
                forbidden_agent_metadata_write(&path, &workspace, &open),
                None,
                "{name}"
            );
        }
        assert!(denying.can_write_path_with_cwd(&workspace.join(".gitx/config"), &workspace));
    }
}
