use crate::acl::DenyReadObject;
use crate::acl::ensure_explicit_deny_read_ace;
use crate::acl::file_link_count;
use crate::acl::remove_deny_read_ace;
use crate::deny_read_acl::apply_deny_read_acls_tracked;
use crate::deny_read_acl::lexical_path_key;
use crate::deny_read_sessions::DenyReadSessions;
use crate::deny_read_sessions::lock_out_other_sessions;
use crate::deny_read_targets::DenyReadTargets;
use crate::setup::sandbox_secrets_dir;
use anyhow::Context;
use anyhow::Result;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::c_void;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

/// In `.sandbox-secrets`, which the sandbox's users cannot write: the
/// entries it lists are this sync's to remove. (Earlier versions kept every
/// applied path in `.sandbox`, which they can write; that file is no longer
/// read, so the entries it lists stay.)
const DENY_READ_ACL_STATE_FILE: &str = "deny_read_acl_rules.json";

/// PF-27-S07: the lock file in `CODEX_HOME` that every process with the
/// secretless launch contract armed holds shared for its lifetime.
pub const SECRETLESS_LAUNCH_LOCK_FILE: &str = ".secretless-launch.lock";

#[derive(Default, Deserialize, Serialize)]
struct PersistentDenyReadAclState {
    /// Per SID, the objects whose deny entry a sync added and has not
    /// removed (#304: an entry already there belongs to someone else, such
    /// as another `CODEX_HOME`'s sessions, and is never removed).
    principals: BTreeMap<String, Vec<OwnedDenyRead>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct OwnedDenyRead {
    object: DenyReadObject,
    /// The configured rules ([`crate::DenyReadRule::key`]) that produced the entry.
    /// It is removed once none of them is configured any more.
    rules: BTreeSet<String>,
}

/// Reconciles the persistent deny-read ACEs owned by one sandbox principal.
///
/// Workspace-write and elevated sandbox sessions intentionally leave ACLs in
/// place after a command exits, because descendants may outlive the launcher.
/// That makes the ACL set stateful across runs. Persist, per SID, the objects
/// a sync added the entry to (path, volume and file ID) and the configured
/// rules that produced each one. Apply the launch's targets first; then
/// remove the entries of recorded objects none of whose rules `targets` still
/// configures (#304). A rule that is still configured keeps its entries even
/// when its glob no longer lists them: a sandboxed process can hide a match
/// from the scan (by holding its folder open, say), and only a configuration
/// change may remove an entry (S1). Each path listed now carries its own
/// entry before any is removed, so removing a parent's entry never uncovers a
/// listed path; a match the scan missed keeps only the entries it had
/// before. An entry is removed only from the very object it was added to.
///
/// `targets` is `None` for a setup refresh that does not carry the launch's
/// rules (a read-root refresh, the first setup): it applies and removes
/// nothing.
///
/// #301: while any process has the secretless launch contract armed on this
/// `CODEX_HOME` (it holds [`SECRETLESS_LAUNCH_LOCK_FILE`]), no entry is
/// removed: that process's commands rely on its denies, which another
/// session's launch (with the flag off) does not list. Stale entries stay
/// recorded and are removed by the first sync after no contract is armed.
///
/// #323: the same holds across `CODEX_HOME`s, armed or not: no entry is
/// removed while a session other than `sessions.own` is registered in
/// `sessions.registry` (see [`crate::deny_read_sessions`]), since its
/// commands may rely on an entry this home added and it found there.
///
/// # Safety
/// Caller must pass a valid SID pointer matching `principal_sid`.
pub unsafe fn sync_persistent_deny_read_acls(
    codex_home: &Path,
    principal_sid: &str,
    targets: Option<&DenyReadTargets>,
    psid: *mut c_void,
    sessions: &DenyReadSessions,
) -> Result<Vec<PathBuf>> {
    let Some(targets) = targets else {
        return Ok(Vec::new());
    };
    let state_dir = sandbox_secrets_dir(codex_home);
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("create {}", state_dir.display()))?;
    let state_path = state_dir.join(DENY_READ_ACL_STATE_FILE);
    let mut state = load_state(&state_path)?;
    let previous = state
        .principals
        .get(principal_sid)
        .cloned()
        .unwrap_or_default();

    let configured = targets
        .rules()
        .iter()
        .map(|rule| rule.rule.key())
        .collect::<BTreeSet<_>>();
    let mut rules_by_path = HashMap::<String, BTreeSet<String>>::new();
    for rule in targets.rules() {
        for path in &rule.paths {
            rules_by_path
                .entry(lexical_path_key(path.as_path()))
                .or_default()
                .insert(rule.rule.key());
        }
    }
    let paths = targets
        .paths()
        .into_iter()
        .map(AbsolutePathBuf::into_path_buf)
        .collect::<Vec<_>>();
    // On a failure, what was added is still recorded (another session may
    // already rely on it) and nothing is removed.
    let (applied, applied_result) =
        unsafe { apply_deny_read_acls_tracked(&paths, &rules_by_path, psid) };
    let still_configured = |rules: &BTreeSet<String>| {
        rules
            .intersection(&configured)
            .cloned()
            .collect::<BTreeSet<_>>()
    };
    let previous_by_id = previous
        .iter()
        .map(|owned| (owned.object.identity(), owned))
        .collect::<HashMap<_, _>>();
    // Ours and listed now: newly added, or recorded before (at its current
    // path, with the rules that list it now added).
    let mut recorded = applied
        .objects
        .iter()
        .filter_map(|applied| {
            let earlier = previous_by_id.get(&applied.object.identity());
            (applied.owned || earlier.is_some()).then(|| {
                let mut rules = applied.rules.clone();
                if let Some(earlier) = earlier {
                    rules.extend(still_configured(&earlier.rules));
                }
                OwnedDenyRead {
                    object: applied.object.clone(),
                    rules,
                }
            })
        })
        .collect::<Vec<_>>();
    let applied_ids = applied
        .objects
        .iter()
        .map(|applied| applied.object.identity())
        .collect::<HashSet<_>>();
    // Ours, not listed now: kept while one of its rules is still configured
    // (S1), stale otherwise.
    let mut stale = Vec::new();
    for owned in previous {
        if applied_ids.contains(&owned.object.identity()) {
            continue;
        }
        let rules = still_configured(&owned.rules);
        if rules.is_empty() {
            stale.push(owned);
        } else if matches!(owned.object.path.symlink_metadata(), Err(err) if err.kind() == std::io::ErrorKind::NotFound)
        {
            // Nothing at its path: it can never be removed there, so the
            // record goes (an object moved elsewhere keeps its entry).
        } else {
            recorded.push(OwnedDenyRead {
                object: owned.object,
                rules,
            });
        }
    }

    // Held until the state is stored: no session can register, and no
    // process can arm, meanwhile.
    let lock = if stale.is_empty() || applied_result.is_err() {
        None
    } else {
        lock_out_other_sessions(sessions).and_then(|sessions| {
            lock_out_armed_contracts(codex_home, psid).map(|home| (sessions, home))
        })
    };
    if lock.is_none() {
        recorded.extend(stale);
    } else {
        for owned in stale {
            // Removed, never there, or another object at the path now (its
            // entry, if any, is not ours): forget it. A path that is gone has
            // no entry left. Keep any other failure (a link left at the path
            // or above it) recorded, so a later sync retries it.
            if unsafe { remove_deny_read_ace(&owned.object, psid) }.is_err()
                && !matches!(owned.object.path.symlink_metadata(), Err(err) if err.kind() == std::io::ErrorKind::NotFound)
            {
                recorded.push(owned);
            }
        }
    }

    if recorded.is_empty() {
        state.principals.remove(principal_sid);
    } else {
        state.principals.insert(principal_sid.to_string(), recorded);
    }
    store_state(&state_path, &state)?;
    drop(lock);

    applied_result?;
    Ok(applied.paths)
}

/// #301: takes [`SECRETLESS_LAUNCH_LOCK_FILE`] exclusively, which succeeds
/// only while no process has the contract armed and keeps any from arming
/// until it is dropped. `None` when a process holds it or it cannot be
/// checked (fail safe: nothing is removed). Creates it if needed, with the
/// contract's own explicit read deny for `psid` (the sandbox's users group
/// on the elevated backend), so a sandboxed command cannot hold it.
fn lock_out_armed_contracts(codex_home: &Path, psid: *mut c_void) -> Option<File> {
    use std::os::windows::fs::OpenOptionsExt as _;
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_WRITE: u32 = 0x2;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let path = codex_home.join(SECRETLESS_LAUNCH_LOCK_FILE);
    // Opened as the launch contract opens it, never through a link.
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&path)
        .ok()?;
    // A hard link would also deny (and lock) another file.
    let regular = file
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        && file_link_count(&file).is_ok_and(|links| links == 1);
    // SAFETY: `psid` is valid for the caller's call; `path` exists.
    let denied = regular
        && matches!(
            unsafe { ensure_explicit_deny_read_ace(&path, psid) },
            Ok(true)
        );
    (denied && file.try_lock().is_ok()).then_some(file)
}

fn load_state(path: &Path) -> Result<PersistentDenyReadAclState> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .with_context(|| format!("parse deny-read ACL state {}", path.display())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok(PersistentDenyReadAclState::default())
        }
        Err(err) => {
            Err(err).with_context(|| format!("read deny-read ACL state {}", path.display()))
        }
    }
}

fn store_state(path: &Path, state: &PersistentDenyReadAclState) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(state).context("serialize deny-read ACL state")?;
    std::fs::write(path, bytes)
        .with_context(|| format!("write deny-read ACL state {}", path.display()))
}

#[cfg(test)]
#[path = "deny_read_state_tests.rs"]
mod tests;
