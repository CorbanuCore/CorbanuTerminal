//! #907: write checks for many paths under one policy and working folder.

use std::cell::OnceCell;
use std::path::Path;
use std::path::PathBuf;

use super::FileSystemSandboxKind;
use super::FileSystemSandboxPolicy;
use super::WritableRootLinks;
use super::normalize_effective_absolute_path;
use super::resolve_candidate_path;

/// [`FileSystemSandboxPolicy::can_write_effective_path_with_cwd`] and
/// [`FileSystemSandboxPolicy::can_grant_write_effective_path_with_cwd`] for
/// many paths in one working folder. The policy's entries are normalized
/// (one filesystem walk each), and its roots reached through a nested link
/// found, the first time a check needs them instead of once per path. The
/// filesystem is read then, so build one per check of a set of paths.
/// The lazy caches use [`OnceCell`], so this view is not [`Sync`].
pub struct EffectiveWrites<'a> {
    policy: &'a FileSystemSandboxPolicy,
    cwd: PathBuf,
    /// The policy with its entries normalized.
    normalized: OnceCell<FileSystemSandboxPolicy>,
    /// The same, with a writable entry tied with a read-only one made
    /// read-only.
    read_only_ties: OnceCell<FileSystemSandboxPolicy>,
    links: OnceCell<WritableRootLinks>,
}

impl<'a> EffectiveWrites<'a> {
    pub(super) fn new(policy: &'a FileSystemSandboxPolicy, cwd: &Path) -> Self {
        Self {
            policy,
            cwd: cwd.to_path_buf(),
            normalized: OnceCell::new(),
            read_only_ties: OnceCell::new(),
            links: OnceCell::new(),
        }
    }

    /// The policy's writable roots with their normalized spellings and real
    /// locations ([`FileSystemSandboxPolicy::writable_root_links_with_cwd`]).
    pub fn links(&self) -> &WritableRootLinks {
        self.links
            .get_or_init(|| self.policy.writable_root_links_with_cwd(&self.cwd))
    }

    /// [`FileSystemSandboxPolicy::can_write_effective_path_with_cwd`].
    pub fn can_write_effective_path(&self, path: &Path) -> bool {
        self.can_write_normalized(path, /*read_only_wins*/ false)
            || self
                .links()
                .spellings(path, &self.cwd)
                .iter()
                .any(|spelled| {
                    self.policy
                        .can_write_path_with_cwd(spelled.as_path(), &self.cwd)
                })
    }

    /// [`FileSystemSandboxPolicy::can_grant_write_effective_path_with_cwd`].
    pub fn can_grant_write_effective_path(&self, path: &Path) -> bool {
        self.can_write_normalized(path, /*read_only_wins*/ true)
    }

    fn can_write_normalized(&self, path: &Path, read_only_wins: bool) -> bool {
        match self.policy.kind {
            FileSystemSandboxKind::Unrestricted | FileSystemSandboxKind::ExternalSandbox => {
                return true;
            }
            FileSystemSandboxKind::Restricted => {}
        }
        let Some(path) = resolve_candidate_path(path, &self.cwd) else {
            return false;
        };
        let normalized = if read_only_wins {
            self.read_only_ties()
        } else {
            self.normalized()
        };
        normalized
            .can_write_path_with_cwd(normalize_effective_absolute_path(path).as_path(), &self.cwd)
    }

    fn normalized(&self) -> &FileSystemSandboxPolicy {
        self.normalized
            .get_or_init(|| self.policy.with_normalized_entries(&self.cwd))
    }

    fn read_only_ties(&self) -> &FileSystemSandboxPolicy {
        self.read_only_ties
            .get_or_init(|| self.normalized().clone().with_read_only_ties())
    }
}

#[cfg(test)]
#[path = "effective_writes_tests.rs"]
mod tests;
