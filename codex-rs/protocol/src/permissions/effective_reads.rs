//! Issue918: effective read checks for one set of paths under a policy and cwd.

use std::cell::OnceCell;
use std::path::Path;

use super::FileSystemSandboxKind;
use super::FileSystemSandboxPolicy;
use super::ResolvedFileSystemEntry;
use super::normalize_effective_absolute_path;
use super::resolve_candidate_path;
use super::resolved_entry_precedence;

/// [`FileSystemSandboxPolicy::can_read_effective_path_with_cwd`] for many
/// paths in one working folder. The resolved policy entries are normalized
/// on the first check that needs them, rather than for every queried path.
///
/// Build one for each check of a set of paths: filesystem observations are
/// cached within that set, not across independent checks or launches. This
/// is not an atomic filesystem snapshot and does not model sandbox clauses
/// or globs. Like `EffectiveWrites`, its lazy `OnceCell` is not `Sync`.
pub struct EffectiveReads<'a> {
    policy: &'a FileSystemSandboxPolicy,
    cwd: &'a Path,
    entries: OnceCell<Vec<ResolvedFileSystemEntry>>,
}

impl<'a> EffectiveReads<'a> {
    pub(super) fn new(policy: &'a FileSystemSandboxPolicy, cwd: &'a Path) -> Self {
        Self {
            policy,
            cwd,
            entries: OnceCell::new(),
        }
    }

    /// The existing effective-read verdict, reusing this set's normalized
    /// policy entries. Each candidate path is still normalized separately.
    pub fn can_read_effective_path(&self, path: &Path) -> bool {
        match self.policy.kind {
            FileSystemSandboxKind::Unrestricted | FileSystemSandboxKind::ExternalSandbox => {
                return true;
            }
            FileSystemSandboxKind::Restricted => {}
        }
        let Some(path) = resolve_candidate_path(path, self.cwd) else {
            return false;
        };
        let path = normalize_effective_absolute_path(path);
        self.entries
            .get_or_init(|| {
                self.policy
                    .resolved_entries_with_cwd(self.cwd)
                    .into_iter()
                    .map(|entry| ResolvedFileSystemEntry {
                        path: normalize_effective_absolute_path(entry.path),
                        access: entry.access,
                    })
                    .collect()
            })
            .iter()
            .filter(|entry| path.as_path().starts_with(entry.path.as_path()))
            .max_by_key(|entry| resolved_entry_precedence(entry))
            .is_some_and(|entry| entry.access.can_read())
    }
}
