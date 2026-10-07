//! PF-23-S01 slice 3: after untrusted content, under Moderate or Aggressive,
//! agent commands run in a sandbox that cannot read credential files or the
//! Corbanu home. This is the OS-level net under the PF-30-S03 command-text
//! classifier: run-time strings, build tools, unknown wrappers and hard links
//! reach the same denial whatever the command text says.
//!
//! The Corbanu home is denied by default: every existing entry except what
//! agent commands need to run (`tmp` holds the arg0 helpers, `shell_snapshots`
//! is sourced by every command) or read as instructions (skills, plugins).
//!
//! PF-23-S02 adds writes: files that run code or set policy later (shell
//! start-up files, login items, git hooks and config, project `.codex`, the
//! Corbanu home entries that stay readable) become read-only wherever the
//! sandbox would let the command write them. Under Aggressive all of this
//! applies from the start of the session, not only after untrusted content.

use crate::security::tainted_action::USER_PERSISTENCE;
use crate::security::tainted_action::WORKSPACE_PERSISTENCE;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemAccessMode;
use codex_protocol::permissions::FileSystemPath;
use codex_protocol::permissions::FileSystemSandboxEntry;
use codex_protocol::permissions::FileSystemSandboxKind;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::path::Path;

/// Corbanu home entries agent commands keep reading after untrusted content.
const RUNTIME_READABLE: &[&str] = &[
    "tmp",
    ".tmp",
    "shell_snapshots",
    "skills",
    "plugins",
    "packages",
    "worktrees",
    "AGENTS.md",
];

/// Denied even before they exist, so a store created later is covered too.
const CORBANU_STORES: &[&str] = &[
    "secrets",
    "auth.json",
    ".credentials.json",
    ".env",
    "provider_auth.json",
    "config.toml",
    "managed_config.toml",
    "source-origin.key",
    "security_level.toml",
    "security_state.json",
    "security_state.lock",
    "wallet",
    "run",
    "log",
    "sessions",
    "archived_sessions",
    "history.jsonl",
    "memories",
];

/// Corbanu home entries that stay readable but never writable: they run, or
/// are read as instructions, in later commands and sessions.
const CORBANU_READ_ONLY: &[&str] = &[
    "tmp",
    ".tmp",
    "shell_snapshots",
    "skills",
    "plugins",
    "packages",
    "AGENTS.md",
];

/// State and log databases, created after launch.
const CORBANU_DATABASES: &str = "*.sqlite*";

/// Other Corbanu home folders in the user's home: denied whole.
const OTHER_CORBANU_HOMES: &[&str] = &[".corbanu", ".codex", ".pfterminal"];

/// Credential folders and files in the user's home.
const USER_CREDENTIALS: &[&str] = &[
    ".ssh",
    ".aws",
    ".gnupg",
    ".kube",
    ".docker",
    ".azure",
    ".config/gh",
    ".config/gcloud",
    ".config/solana",
    ".netrc",
    ".git-credentials",
    ".npmrc",
    ".pypirc",
    ".curlrc",
    ".wgetrc",
    ".credentials.json",
    ".claude/.credentials.json",
    "Library/Keychains",
];

/// The paths one protected command may not read, and those it may not write.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ReadDenials {
    /// Existing paths: enforced as they are.
    existing: Vec<AbsolutePathBuf>,
    /// Fixed locations that may not exist yet.
    fixed: Vec<AbsolutePathBuf>,
    globs: Vec<String>,
    /// Readable but never writable (PF-23-S02), even before they exist.
    read_only: Vec<AbsolutePathBuf>,
    /// Where the profile's relative entries resolve (the turn's folder).
    cwd: Option<AbsolutePathBuf>,
    /// Denials dropped because they hold the command's folder or a writable
    /// root (denying them would deny the workspace itself).
    pub(crate) skipped: Vec<AbsolutePathBuf>,
}

impl ReadDenials {
    /// `keep` are folders the command must keep (its working folder and
    /// workspace roots): a denial equal to or above one of them is skipped.
    pub(crate) fn collect(
        codex_home: &Path,
        user_home: Option<&Path>,
        claude_config_dir: Option<&Path>,
        keep: &[AbsolutePathBuf],
    ) -> Self {
        let mut denials = Self {
            cwd: keep.first().cloned(),
            ..Self::default()
        };
        let Ok(codex_home) = AbsolutePathBuf::from_absolute_path(codex_home) else {
            return denials;
        };
        for entry in CORBANU_READ_ONLY {
            denials.push_home_read_only(codex_home.join(entry), keep);
        }
        for root in keep {
            for entry in WORKSPACE_PERSISTENCE {
                denials.push_read_only(root.join(entry), keep);
            }
            // The repository the root is in, found the way git finds it; a
            // worktree or submodule keeps hooks and config in the git folder
            // its `.git` file points to (hooks in the common one).
            for path in git_persistence_paths(root) {
                denials.push_read_only(path, keep);
            }
        }
        if let Ok(entries) = std::fs::read_dir(codex_home.as_path()) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if RUNTIME_READABLE.iter().any(|keep| name == *keep) {
                    continue;
                }
                denials.push_existing(codex_home.join(name), keep);
            }
        }
        for store in CORBANU_STORES {
            denials.push_fixed(codex_home.join(store), keep);
        }
        denials.globs.push(
            codex_home
                .join(CORBANU_DATABASES)
                .to_string_lossy()
                .into_owned(),
        );
        if let Some(home) =
            user_home.and_then(|home| AbsolutePathBuf::from_absolute_path(home).ok())
        {
            for other in OTHER_CORBANU_HOMES {
                let other = home.join(other);
                // The active home is handled entry by entry above.
                if codex_home.as_path().starts_with(other.as_path()) {
                    continue;
                }
                denials.push_fixed(other, keep);
            }
            for credential in USER_CREDENTIALS {
                denials.push_fixed(home.join(credential), keep);
            }
            for file in USER_PERSISTENCE {
                denials.push_home_read_only(home.join(file), keep);
            }
        }
        if let Some(dir) =
            claude_config_dir.and_then(|dir| AbsolutePathBuf::from_absolute_path(dir).ok())
        {
            denials.push_fixed(dir.join(".credentials.json"), keep);
        }
        denials
    }

    /// The denials for this host's user: what stays readable is the turn's
    /// working folder, its workspace roots and the profile's writable roots
    /// resolved against that folder.
    pub(crate) fn for_turn(
        codex_home: &Path,
        turn_cwd: &AbsolutePathBuf,
        workspace_roots: &[AbsolutePathBuf],
        profile: &PermissionProfile,
    ) -> Self {
        let mut keep = vec![turn_cwd.clone()];
        keep.extend(workspace_roots.iter().cloned());
        keep.extend(
            profile
                .file_system_sandbox_policy()
                .get_writable_roots_with_cwd(turn_cwd.as_path())
                .into_iter()
                .map(|root| root.root),
        );
        Self::collect(
            codex_home,
            dirs::home_dir().as_deref(),
            std::env::var_os("CLAUDE_CONFIG_DIR")
                .as_deref()
                .map(Path::new),
            &keep,
        )
    }

    fn holds_kept(path: &AbsolutePathBuf, keep: &[AbsolutePathBuf]) -> bool {
        keep.iter()
            .any(|kept| kept.as_path().starts_with(path.as_path()))
    }

    fn push_existing(&mut self, path: AbsolutePathBuf, keep: &[AbsolutePathBuf]) {
        if Self::holds_kept(&path, keep) {
            self.skipped.push(path);
        } else if !self.existing.contains(&path) {
            self.existing.push(path);
        }
    }

    fn push_fixed(&mut self, path: AbsolutePathBuf, keep: &[AbsolutePathBuf]) {
        if Self::holds_kept(&path, keep) {
            self.skipped.push(path);
        } else if !self.existing.contains(&path) && !self.fixed.contains(&path) {
            self.fixed.push(path);
        }
    }

    /// A read-only path in a home folder. On Linux a missing one is left out:
    /// the sandbox would put an empty placeholder at it in the real home for
    /// as long as the command runs (an empty `~/.bash_profile` hides
    /// `~/.profile` from the user's own login shells). The command-text net
    /// still asks before such a file is written.
    fn push_home_read_only(&mut self, path: AbsolutePathBuf, keep: &[AbsolutePathBuf]) {
        // `metadata` follows links: a link to a missing target is left out too.
        if cfg!(target_os = "linux") && std::fs::metadata(path.as_path()).is_err() {
            return;
        }
        self.push_read_only(path, keep);
    }

    /// A read-only path, by its real location: below a symlinked folder the
    /// Linux sandbox cannot bind the link name. A path below a regular file
    /// (`.git` in a worktree) cannot exist and is left out. On macOS the link
    /// name is kept too, so the sandbox also refuses to remove the link.
    fn push_read_only(&mut self, path: AbsolutePathBuf, keep: &[AbsolutePathBuf]) {
        let Some(real) = real_location(&path) else {
            return;
        };
        let mut spellings = vec![real];
        if cfg!(target_os = "macos") && !spellings.contains(&path) {
            spellings.push(path);
        }
        for path in spellings {
            if Self::holds_kept(&path, keep) {
                self.skipped.push(path);
            } else if !self.read_only.contains(&path) {
                self.read_only.push(path);
            }
        }
    }

    /// Every path made read-only.
    pub(crate) fn read_only_paths(&self) -> impl Iterator<Item = &AbsolutePathBuf> {
        self.read_only.iter()
    }

    /// Every denied path, existing first.
    pub(crate) fn paths(&self) -> impl Iterator<Item = &AbsolutePathBuf> {
        self.existing.iter().chain(&self.fixed)
    }

    /// `profile` with these reads denied. A profile without a sandbox gets
    /// one that keeps its full write and network access; an external
    /// sandbox cannot take extra rules and is returned as `None`.
    pub(crate) fn apply(&self, profile: &PermissionProfile) -> Option<PermissionProfile> {
        if matches!(profile, PermissionProfile::External { .. }) {
            return None;
        }
        let (mut file_system, network) = profile.to_runtime_permissions();
        let entries: Vec<FileSystemSandboxEntry> = self
            .existing
            .iter()
            .map(|path| {
                FileSystemSandboxEntry::new(
                    FileSystemPath::Path { path: path.clone() },
                    FileSystemAccessMode::Deny,
                )
            })
            .chain(self.fixed.iter().map(|path| {
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Path { path: path.clone() },
                    FileSystemAccessMode::Deny,
                )
            }))
            .chain(self.globs.iter().map(|pattern| {
                FileSystemSandboxEntry::new(
                    FileSystemPath::GlobPattern {
                        pattern: pattern.clone(),
                    },
                    FileSystemAccessMode::Deny,
                )
            }))
            .collect();
        if entries.is_empty() && self.read_only.is_empty() {
            return Some(profile.clone());
        }
        // Turns an unrestricted policy into full write access plus the
        // denials, and appends them to a restricted one.
        file_system
            .preserve_deny_read_restrictions_from(&FileSystemSandboxPolicy::restricted(entries));
        if file_system.kind != FileSystemSandboxKind::Restricted {
            return None;
        }
        // Only narrows: a path becomes read-only where the profile would let
        // the command write it, never readable where it was not.
        let cwd = self
            .cwd
            .as_ref()
            .map_or_else(|| Path::new("/"), AbsolutePathBuf::as_path);
        let read_only: Vec<FileSystemSandboxEntry> = self
            .read_only
            .iter()
            .filter(|path| file_system.can_write_path_with_cwd(path.as_path(), cwd))
            .map(|path| {
                FileSystemSandboxEntry::skip_missing_path(
                    FileSystemPath::Path { path: path.clone() },
                    FileSystemAccessMode::Read,
                )
            })
            .collect();
        file_system.entries.extend(read_only);
        Some(PermissionProfile::from_runtime_permissions(
            &file_system,
            network,
        ))
    }
}

/// `path` with its nearest existing folder resolved, or `None` when that
/// folder is not a folder (nothing can be created below a file).
fn real_location(path: &AbsolutePathBuf) -> Option<AbsolutePathBuf> {
    real_location_within(path, /*hops*/ 8)
}

fn real_location_within(path: &AbsolutePathBuf, hops: usize) -> Option<AbsolutePathBuf> {
    let mut existing = path.as_path();
    let mut tail = Vec::new();
    while std::fs::symlink_metadata(existing).is_err() {
        tail.push(existing.file_name()?);
        existing = existing.parent()?;
    }
    // A link (the path or a folder above it) whose target does not exist
    // yet: protect the path below the target.
    if std::fs::metadata(existing).is_err()
        && let Ok(target) = std::fs::read_link(existing)
    {
        let mut next = AbsolutePathBuf::resolve_path_against_base(target, existing.parent()?);
        for name in tail.iter().rev() {
            next = next.join(name);
        }
        return real_location_within(&next, hops.checked_sub(1)?);
    }
    if !tail.is_empty() && !existing.is_dir() {
        return None;
    }
    let mut real = std::fs::canonicalize(existing).ok()?;
    for name in tail.iter().rev() {
        real.push(name);
    }
    AbsolutePathBuf::from_absolute_path(real).ok()
}

/// What git reads to decide which hooks and config run for the repository
/// `root` is in (the nearest `.git` at or above it, found the way git finds
/// it): hooks, config, config.worktree and commondir of its git folder; for
/// a `.git` file (worktree, submodule) the file itself and the same entries
/// in the folder it points to and its common folder. A missing commondir is
/// covered on macOS only: on Linux the sandbox would put an empty one in
/// place, which breaks git.
fn git_persistence_paths(root: &AbsolutePathBuf) -> Vec<AbsolutePathBuf> {
    let Some(dot_git) = root
        .as_path()
        .ancestors()
        .map(|folder| folder.join(".git"))
        .find(|dot_git| std::fs::symlink_metadata(dot_git).is_ok())
        .and_then(|dot_git| AbsolutePathBuf::from_absolute_path(dot_git).ok())
    else {
        return Vec::new();
    };
    let entries = |git_dir: &AbsolutePathBuf| {
        let commondir = git_dir.join("commondir");
        let with_commondir =
            !cfg!(target_os = "linux") || std::fs::symlink_metadata(commondir.as_path()).is_ok();
        ["hooks", "config", "config.worktree"]
            .into_iter()
            .map(|entry| git_dir.join(entry))
            .chain(with_commondir.then_some(commondir))
            .collect::<Vec<_>>()
    };
    if dot_git.as_path().is_dir() {
        return entries(&dot_git);
    }
    let mut paths = vec![dot_git.clone()];
    let Ok(text) = std::fs::read_to_string(dot_git.as_path()) else {
        return paths;
    };
    let (Some(folder), Some(git_dir)) = (
        dot_git.parent(),
        text.lines().find_map(|line| line.strip_prefix("gitdir:")),
    ) else {
        return paths;
    };
    let git_dir = AbsolutePathBuf::resolve_path_against_base(git_dir.trim(), folder.as_path());
    let common = std::fs::read_to_string(git_dir.join("commondir").as_path())
        .ok()
        .map(|dir| AbsolutePathBuf::resolve_path_against_base(dir.trim(), git_dir.as_path()));
    for dir in std::iter::once(git_dir).chain(common) {
        paths.extend(entries(&dir));
    }
    paths
}

/// In-process file tools (the patch pre-check, structured edits, image
/// viewing, extension tools) read files with the same denials after
/// untrusted content. No approval lifts them here.
pub(crate) fn protect_file_tool_context(
    session: &crate::session::session::Session,
    turn: &crate::session::turn_context::TurnContext,
    mut context: codex_file_system::FileSystemSandboxContext,
) -> codex_file_system::FileSystemSandboxContext {
    if session
        .services
        .model_client()
        .post_taint_state()
        .is_none_or(|state| !state.protected_paths_apply())
    {
        return context;
    }
    let Ok(profile) = PermissionProfile::try_from(context.permissions.clone()) else {
        // Unreadable permissions: no file access at all (fail closed).
        context.permissions = PermissionProfile::from_runtime_permissions(
            &FileSystemSandboxPolicy::restricted(Vec::new()),
            codex_protocol::permissions::NetworkSandboxPolicy::Restricted,
        )
        .into();
        return context;
    };
    #[allow(deprecated)]
    let turn_cwd = turn.cwd.clone();
    let workspace_roots: Vec<AbsolutePathBuf> = context
        .workspace_roots
        .iter()
        .filter_map(|root| root.to_abs_path().ok())
        .collect();
    // Keep roots come from the turn's own profile: extra permissions granted
    // to this call never make a denied folder readable.
    let denials = ReadDenials::for_turn(
        turn.config.codex_home.as_path(),
        &turn_cwd,
        &workspace_roots,
        turn.config.permissions.permission_profile(),
    );
    if let Some(protected) = denials.apply(&profile) {
        context.permissions = protected.into();
    }
    context
}

/// The read policy for host code that reads a model-named file (Codex Apps
/// uploads) once the protected-path rules apply: `None` before that. A
/// profile that cannot take the rules reads nothing.
pub(crate) fn post_taint_read_policy(
    session: &crate::session::session::Session,
    turn: &crate::session::turn_context::TurnContext,
) -> Option<FileSystemSandboxPolicy> {
    session
        .services
        .model_client()
        .post_taint_state()
        .filter(crate::security::tainted_action::PostTaintState::protected_paths_apply)?;
    #[allow(deprecated)]
    let turn_cwd = turn.cwd.clone();
    let workspace_roots: Vec<AbsolutePathBuf> = turn
        .environments
        .primary()
        .map(|environment| {
            environment
                .workspace_roots()
                .iter()
                .filter_map(|root| root.to_abs_path().ok())
                .collect()
        })
        .unwrap_or_default();
    let profile = turn.config.permissions.permission_profile();
    let denials = ReadDenials::for_turn(
        turn.config.codex_home.as_path(),
        &turn_cwd,
        &workspace_roots,
        profile,
    );
    Some(
        denials
            .apply(profile)
            .map(|protected| protected.file_system_sandbox_policy())
            .unwrap_or_else(|| FileSystemSandboxPolicy::restricted(Vec::new())),
    )
}

/// Whether `path` (and what it resolves to) may be read under `policy`.
pub(crate) fn readable_under(policy: &FileSystemSandboxPolicy, path: &Path, cwd: &Path) -> bool {
    policy.can_read_path_with_cwd(path, cwd)
        && std::fs::canonicalize(path).map_or(true, |canonical| {
            policy.can_read_path_with_cwd(&canonical, cwd)
        })
}

#[cfg(test)]
#[path = "read_denials_tests.rs"]
mod tests;
