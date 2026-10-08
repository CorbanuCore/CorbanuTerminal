use crate::acl::remove_deny_read_ace;
use crate::deny_read_acl::apply_deny_read_acls;
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
    principals: BTreeMap<String, Vec<PathBuf>>,
}

/// Reconciles the persistent deny-read ACEs owned by one sandbox principal.
///
/// Workspace-write and elevated sandbox sessions intentionally leave ACLs in
/// place after a command exits, because descendants may outlive the launcher.
/// That makes the ACL set stateful across runs. Persist the paths applied for
/// each SID, apply the new desired set first, and only then remove stale
/// paths' deny entries from the same SID so profile changes do not leave old
/// deny-read ACEs behind.
///
/// #301: while any process has the secretless launch contract armed on this
/// `CODEX_HOME` (it holds [`SECRETLESS_LAUNCH_LOCK_FILE`]), no deny is
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

    let applied_paths = unsafe { apply_deny_read_acls(desired_paths, psid) }?;
    let desired_keys = applied_paths
        .iter()
        .map(|path| lexical_path_key(path))
        .collect::<HashSet<_>>();
    let stale_paths = previous_paths
        .into_iter()
        .filter(|path| !desired_keys.contains(&lexical_path_key(path)))
        .collect::<Vec<_>>();

    let mut recorded_paths = applied_paths.clone();
    // Held until the state is stored: no process can arm meanwhile.
    let armed = if stale_paths.is_empty() {
        ArmedContracts::Never
    } else {
        armed_contracts(codex_home)
    };
    if matches!(armed, ArmedContracts::Armed) {
        recorded_paths.extend(stale_paths);
    } else {
        for path in stale_paths {
            // A path that is gone has no entry left; keep any other failure
            // recorded so a later sync retries it.
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

    Ok(applied_paths)
}

/// #301: whether a process may have the launch contract armed on a
/// `CODEX_HOME`.
enum ArmedContracts {
    /// No lock file: no contract was ever armed there.
    Never,
    /// This process holds the lock exclusively, so none is armed and none
    /// can arm until it is dropped.
    LockedOut { _lock: File },
    /// A process holds the lock, or it could not be checked (fail safe).
    Armed,
}

fn armed_contracts(codex_home: &Path) -> ArmedContracts {
    use std::os::windows::fs::OpenOptionsExt as _;
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_WRITE: u32 = 0x2;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    // Never created here: the contract creates it with its own deny for the
    // sandbox's users. Never opened through a link.
    let file = match std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(codex_home.join(SECRETLESS_LAUNCH_LOCK_FILE))
    {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return ArmedContracts::Never,
        Err(_) => return ArmedContracts::Armed,
    };
    let regular = file
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink());
    if regular && file.try_lock().is_ok() {
        ArmedContracts::LockedOut { _lock: file }
    } else {
        ArmedContracts::Armed
    }
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
