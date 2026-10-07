//! PF-23-S01 slice 3: after untrusted content, under Moderate or Aggressive,
//! agent commands run in a sandbox that cannot read credential files or the
//! Corbanu home. This is the OS-level net under the PF-30-S03 command-text
//! classifier: run-time strings, build tools, unknown wrappers and hard links
//! reach the same denial whatever the command text says.
//!
//! The Corbanu home is denied by default: every existing entry except what
//! agent commands need to run (`tmp` holds the arg0 helpers, `shell_snapshots`
//! is sourced by every command) or read as instructions (skills, plugins).

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
    "wallet",
    "run",
    "log",
    "sessions",
    "archived_sessions",
    "history.jsonl",
    "memories",
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

/// The paths one tainted command may not read.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ReadDenials {
    /// Existing paths: enforced as they are.
    existing: Vec<AbsolutePathBuf>,
    /// Fixed locations that may not exist yet.
    fixed: Vec<AbsolutePathBuf>,
    globs: Vec<String>,
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
        let mut denials = Self::default();
        let Ok(codex_home) = AbsolutePathBuf::from_absolute_path(codex_home) else {
            return denials;
        };
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
        }
        if let Some(dir) =
            claude_config_dir.and_then(|dir| AbsolutePathBuf::from_absolute_path(dir).ok())
        {
            denials.push_fixed(dir.join(".credentials.json"), keep);
        }
        denials
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
        if entries.is_empty() {
            return Some(profile.clone());
        }
        // Turns an unrestricted policy into full write access plus the
        // denials, and appends them to a restricted one.
        file_system
            .preserve_deny_read_restrictions_from(&FileSystemSandboxPolicy::restricted(entries));
        (file_system.kind == FileSystemSandboxKind::Restricted)
            .then(|| PermissionProfile::from_runtime_permissions(&file_system, network))
    }
}

#[cfg(test)]
#[path = "read_denials_tests.rs"]
mod tests;
