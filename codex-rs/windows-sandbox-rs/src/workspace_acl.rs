use crate::acl::add_deny_delete_child_ace;
use crate::acl::add_deny_write_ace;
use crate::path_normalization::canonicalize_path;
use crate::token::world_sid;
use anyhow::Result;
use std::ffi::c_void;
use std::path::Path;

pub fn is_command_cwd_root(root: &Path, canonical_command_cwd: &Path) -> bool {
    canonicalize_path(root) == canonical_command_cwd
}

/// # Safety
/// Caller must ensure `psid` is a valid SID pointer.
pub unsafe fn protect_workspace_codex_dir(cwd: &Path, psid: *mut c_void) -> Result<bool> {
    protect_workspace_subdir(cwd, psid, ".codex")
}

/// # Safety
/// Caller must ensure `psid` is a valid SID pointer.
pub unsafe fn protect_workspace_agents_dir(cwd: &Path, psid: *mut c_void) -> Result<bool> {
    protect_workspace_subdir(cwd, psid, ".agents")
}

unsafe fn protect_workspace_subdir(cwd: &Path, psid: *mut c_void, subdir: &str) -> Result<bool> {
    let path = cwd.join(subdir);
    if path.is_dir() {
        let added = add_deny_write_ace(&path, psid)?;
        deny_delete_child_route(&path)?;
        Ok(added)
    } else {
        Ok(false)
    }
}

/// Closes the `FILE_DELETE_CHILD` route to deleting or moving a protected path
/// from a legacy (WRITE_RESTRICTED) sandbox, which capability-SID deny ACEs do
/// not cover (#158): denies Everyone `FILE_DELETE_CHILD` on the path's parent
/// (that directory only) and on the path and its subdirectories.
///
/// Everyone is in every sandbox token's normal groups, so the deny applies no
/// matter which user or group ACE grants the right. Unsandboxed deletes keep
/// working because they use the object's own `DELETE` right.
///
/// # Safety
/// Calls Win32 ACL APIs; `path` should exist.
pub unsafe fn deny_delete_child_route(path: &Path) -> Result<()> {
    let mut everyone = world_sid()?;
    let psid = everyone.as_mut_ptr() as *mut c_void;
    if let Some(parent) = path.parent()
        && parent.is_dir()
    {
        add_deny_delete_child_ace(parent, psid, /*inherit_to_subdirs*/ false)?;
    }
    if path.is_dir() {
        add_deny_delete_child_ace(path, psid, /*inherit_to_subdirs*/ true)?;
    }
    Ok(())
}
