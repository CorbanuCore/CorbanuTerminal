use crate::acl::DenyReadObject;
use crate::acl::add_deny_read_ace_to_object;
use crate::acl::remove_deny_read_ace;
use crate::path_normalization::canonicalize_path;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::c_void;
use std::path::Path;
use std::path::PathBuf;

/// Build the exact ACL paths that should receive a deny-read ACE.
///
/// We keep both the lexical policy path and, when it already exists, the
/// canonical target. The lexical path covers the path users configured and lets
/// missing exact denies be materialized later; the canonical path also covers
/// an existing reparse-point target so a sandbox cannot read the same object
/// through the resolved location.
pub fn plan_deny_read_acl_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut planned = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        push_planned_path(&mut planned, &mut seen, path.to_path_buf());
        if path.exists() {
            push_planned_path(&mut planned, &mut seen, canonicalize_path(path));
        }
    }
    planned
}

fn push_planned_path(planned: &mut Vec<PathBuf>, seen: &mut HashSet<String>, path: PathBuf) {
    if seen.insert(lexical_path_key(&path)) {
        planned.push(path);
    }
}

pub(crate) fn lexical_path_key(path: &Path) -> String {
    crate::deny_read_targets::path_key(path)
}

/// Applies deny-read ACEs to explicit paths. Missing paths are materialized as
/// directories before the ACE is applied so a sandboxed command cannot create a
/// previously absent denied path and then read from it in the same run.
/// If any path fails, deny ACEs this call added directly (not through a link)
/// are removed before the error is returned so a one-shot sandbox run does
/// not leave partial state. Persistent sessions use
/// [`crate::sync_persistent_deny_read_acls`], which records them instead.
///
/// # Safety
/// Caller must pass a valid SID pointer for the sandbox principal being denied.
pub unsafe fn apply_deny_read_acls(paths: &[PathBuf], psid: *mut c_void) -> Result<Vec<PathBuf>> {
    let (applied, result) = unsafe { apply_deny_read_acls_tracked(paths, &HashMap::new(), psid) };
    if let Err(err) = result {
        for object in &applied.objects {
            if object.owned {
                let _ = unsafe { remove_deny_read_ace(&object.object, psid) };
            }
        }
        return Err(err);
    }
    Ok(applied.paths)
}

/// What [`apply_deny_read_acls_tracked`] did.
pub(crate) struct AppliedDenyReads {
    /// Every path that now has the deny (planned paths, deduplicated).
    pub(crate) paths: Vec<PathBuf>,
    /// The objects those paths resolve to.
    pub(crate) objects: Vec<AppliedDenyRead>,
}

pub(crate) struct AppliedDenyRead {
    pub(crate) object: DenyReadObject,
    /// This call added the entry, through one of the given paths as is (not
    /// through a link, nor as the canonical form the planner adds). Only
    /// those are a sync's to remove later: an entry that landed elsewhere
    /// through a link stays (#304).
    pub(crate) owned: bool,
    /// The keys of the rules whose given paths reached the object as is.
    pub(crate) rules: BTreeSet<String>,
}

/// [`apply_deny_read_acls`] without the rollback: the objects that have the
/// entry are reported even when a later path fails (the error is returned
/// alongside), so the caller can record what was added instead of removing
/// it while another session may rely on it. `rules_by_path` maps a given
/// path's [`lexical_path_key`] to the keys of the rules that produced it.
///
/// # Safety
/// As for [`apply_deny_read_acls`].
pub(crate) unsafe fn apply_deny_read_acls_tracked(
    paths: &[PathBuf],
    rules_by_path: &HashMap<String, BTreeSet<String>>,
    psid: *mut c_void,
) -> (AppliedDenyReads, Result<()>) {
    let given = paths
        .iter()
        .map(|path| lexical_path_key(path))
        .collect::<HashSet<_>>();
    let mut applied = AppliedDenyReads {
        paths: Vec::new(),
        objects: Vec::new(),
    };
    let mut seen = HashSet::new();
    for path in plan_deny_read_acl_paths(paths) {
        let result = (|| -> Result<(bool, DenyReadObject)> {
            if !path.exists() {
                std::fs::create_dir_all(&path)
                    .with_context(|| format!("create deny-read path {}", path.display()))?;
            }
            add_deny_read_ace_to_object(&path, psid)
                .with_context(|| format!("apply deny-read ACE to {}", path.display()))
        })();
        let (added, object) = match result {
            Ok(result) => result,
            Err(err) => return (applied, Err(err)),
        };
        let key = lexical_path_key(&path);
        let direct = given.contains(&key) && lexical_path_key(&object.path) == key;
        let rules = if direct {
            rules_by_path.get(&key).cloned().unwrap_or_default()
        } else {
            BTreeSet::new()
        };
        match applied
            .objects
            .iter_mut()
            .find(|known| known.object.identity() == object.identity())
        {
            Some(known) => {
                known.owned |= added && direct;
                known.rules.extend(rules);
            }
            None => applied.objects.push(AppliedDenyRead {
                object,
                owned: added && direct,
                rules,
            }),
        }
        push_planned_path(&mut applied.paths, &mut seen, path);
    }
    (applied, Ok(()))
}

#[cfg(test)]
mod tests {
    use super::plan_deny_read_acl_paths;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn plan_preserves_missing_paths() {
        let tmp = TempDir::new().expect("tempdir");
        let missing = tmp.path().join("future-secret.env");

        assert_eq!(
            plan_deny_read_acl_paths(std::slice::from_ref(&missing)),
            vec![missing]
        );
    }

    #[test]
    fn plan_includes_existing_canonical_targets() {
        let tmp = TempDir::new().expect("tempdir");
        let existing = tmp.path().join("secret.env");
        std::fs::write(&existing, "secret").expect("write secret");

        let planned: HashSet<PathBuf> = plan_deny_read_acl_paths(std::slice::from_ref(&existing))
            .into_iter()
            .collect();
        let expected: HashSet<PathBuf> = [
            existing.clone(),
            dunce::canonicalize(&existing).expect("canonical path"),
        ]
        .into_iter()
        .collect();

        assert_eq!(planned, expected);
    }
}
