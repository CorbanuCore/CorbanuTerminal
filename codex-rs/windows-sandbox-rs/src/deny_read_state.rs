use crate::acl::ensure_explicit_deny_read_ace;
use crate::acl::file_link_count;
use crate::acl::remove_deny_read_ace;
use crate::deny_read_acl::apply_deny_read_acls_tracked;
use crate::deny_read_acl::lexical_path_key;
use crate::setup::sandbox_dir;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ffi::c_void;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

const DENY_READ_ACL_STATE_FILE: &str = "deny_read_acl_state.json";

/// PF-27-S07: the lock file in `CODEX_HOME` that every process with the
/// secretless launch contract armed holds shared for its lifetime.
pub const SECRETLESS_LAUNCH_LOCK_FILE: &str = ".secretless-launch.lock";

#[derive(Default, Deserialize, Serialize)]
struct PersistentDenyReadAclState {
    /// Per SID, the paths whose deny entry a sync added and has not removed
    /// (#304: an entry already there belongs to someone else, such as
    /// another `CODEX_HOME`'s sessions, and is never removed).
    principals: BTreeMap<String, Vec<PathBuf>>,
}

/// Reconciles the persistent deny-read ACEs owned by one sandbox principal.
///
/// Workspace-write and elevated sandbox sessions intentionally leave ACLs in
/// place after a command exits, because descendants may outlive the launcher.
/// That makes the ACL set stateful across runs. Persist, per SID, the paths
/// whose entry a sync added; apply the new desired set first, and only then
/// remove the entries of recorded paths it no longer lists, so profile
/// changes do not leave old deny-read ACEs behind (#304). Each desired path
/// carries its own entry before any is removed, so removing a parent's entry
/// never uncovers a path that is still denied.
///
/// #301: while any process has the secretless launch contract armed on this
/// `CODEX_HOME` (it holds [`SECRETLESS_LAUNCH_LOCK_FILE`]), no entry is
/// removed: that process's commands rely on its denies, which another
/// session's launch (with the flag off) does not list. Stale paths stay
/// recorded and are removed by the first sync after no contract is armed.
///
/// # Safety
/// Caller must pass a valid SID pointer matching `principal_sid`.
pub unsafe fn sync_persistent_deny_read_acls(
    codex_home: &Path,
    principal_sid: &str,
    desired_paths: &[PathBuf],
    psid: *mut c_void,
) -> Result<Vec<PathBuf>> {
    let state_path = sandbox_dir(codex_home).join(DENY_READ_ACL_STATE_FILE);
    let mut state = load_state(&state_path)?;
    let previous_paths = state
        .principals
        .get(principal_sid)
        .cloned()
        .unwrap_or_default();

    let applied = unsafe { apply_deny_read_acls_tracked(desired_paths, psid) }?;
    let desired_keys = applied
        .paths
        .iter()
        .map(|path| lexical_path_key(path))
        .collect::<HashSet<_>>();
    let (kept_paths, stale_paths): (Vec<_>, Vec<_>) = previous_paths
        .into_iter()
        .partition(|path| desired_keys.contains(&lexical_path_key(path)));
    let mut recorded_paths = Vec::new();
    let mut recorded_keys = HashSet::new();
    for path in kept_paths.into_iter().chain(applied.added) {
        if recorded_keys.insert(lexical_path_key(&path)) {
            recorded_paths.push(path);
        }
    }

    // Held until the state is stored: no process can arm meanwhile.
    let lock = if stale_paths.is_empty() {
        None
    } else {
        lock_out_armed_contracts(codex_home, psid)
    };
    if lock.is_none() {
        recorded_paths.extend(stale_paths);
    } else {
        for path in stale_paths {
            // A path that is gone has no entry left. Keep any other failure
            // (including a link left at the path) recorded, so a later sync
            // retries it.
            if unsafe { remove_deny_read_ace(&path, psid) }.is_err() && path.exists() {
                recorded_paths.push(path);
            }
        }
    }

    if recorded_paths.is_empty() {
        state.principals.remove(principal_sid);
    } else {
        state
            .principals
            .insert(principal_sid.to_string(), recorded_paths);
    }
    store_state(&state_path, &state)?;
    drop(lock);

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
