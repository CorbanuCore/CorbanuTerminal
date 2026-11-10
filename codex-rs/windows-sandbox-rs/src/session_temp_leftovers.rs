//! D377 (#1871): what older versions' commands left in the temp folder.
//!
//! Older setups granted the sandbox's group Modify on the user's whole temp
//! folder, so elevated commands made files and folders there that their
//! sandbox accounts own. An owner may always change an object's
//! permissions, so taking back the folder's grant alone leaves the sandbox
//! able to change those: grant itself access again, and, in a folder it
//! owns, replace what the user's own programs later put there. At the same
//! upgrade step, before that grant goes, the user takes them: becomes their
//! owner and removes every allow entry the sandbox could have put there (all
//! but the user's, SYSTEM's, Administrators' and OWNER RIGHTS'; deny entries
//! stay), so what they allow comes from the folder they are in. Nothing is
//! deleted. One the user may not take (its DACL keeps the user from
//! changing it) is renamed in place, so what would use it by name makes a
//! new one, the user's; it is counted.
//!
//! D456: the code caches other programs keep in the temp folder and later
//! run code from (Node's compile cache, Python's bytecode, ...) are then
//! renamed in place too, so those programs make new ones. The old grant let
//! the sandbox write into any of them, whoever owns them, so ownership
//! doesn't tell which it changed: every folder positively identified as one
//! (by name, see [`code_cache`]) is renamed, and so is a link with such a
//! name (the link itself), which would still lead those programs to a
//! folder the sandbox may write. Nothing is deleted.

use crate::acl::file_identity;
use crate::acl::final_path;
use crate::acl::take_for_user;
use crate::deny_read_sessions::current_user_sid;
use crate::deny_read_sessions::owner_sid;
use crate::session_temp::SESSION_TEMP_PREFIX;
use crate::token::LocalSid;
use anyhow::Result;
use rand::RngCore as _;
use rand::SeedableRng as _;
use rand::rngs::SmallRng;
use std::fs::File;
use std::os::windows::fs::MetadataExt as _;
use std::os::windows::fs::OpenOptionsExt as _;
use std::os::windows::io::AsRawHandle as _;
use std::path::Path;
use std::path::PathBuf;

/// Entries looked at, at most, in one temp folder.
const MAX_ENTRIES: usize = 500_000;
/// What a leftover the user may not take is renamed to end with.
const SET_ASIDE: &str = "corbanu-sandbox-leftover";
/// What a code cache is renamed to end with (D456).
const CACHE_SET_ASIDE: &str = "corbanu-sandbox-cache";
/// D456: code caches at the top of the temp folder, by name: Node's
/// (`module.enableCompileCache()`, `NODE_COMPILE_CACHE` unset), the
/// `v8-compile-cache` package's, `jiti`'s transpile cache (v1, v2), and
/// .NET single-file apps' extracted libraries (reused between runs).
const TOP_LEVEL_CODE_CACHES: &[&str] = &[
    "node-compile-cache",
    "v8-compile-cache",
    "node-jiti",
    "jiti",
    ".net",
];
/// Also at the top: `v8-compile-cache` named per user where there are uids,
/// and `tsx`'s transform cache (`tsx-<user>`).
const TOP_LEVEL_CODE_CACHE_PREFIXES: &[&str] = &["v8-compile-cache-", "tsx-"];
/// D456: Python's bytecode folder (a name Python reserves), anywhere in the
/// temp folder, next to the sources it was compiled from.
const PYTHON_CODE_CACHE: &str = "__pycache__";
/// Code caches renamed, at most, in one temp folder.
const MAX_CODE_CACHES: usize = 10_000;
const DELETE: u32 = 0x1_0000;
const READ_CONTROL: u32 = 0x2_0000;
const WRITE_DAC: u32 = 0x4_0000;
const WRITE_OWNER: u32 = 0x8_0000;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_SHARE_READ: u32 = 0x1;
const FILE_SHARE_WRITE: u32 = 0x2;
const FILE_SHARE_ALL: u32 = 0x7;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

/// What [`take_sandbox_leftovers`] found and did.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Leftovers {
    /// Entries looked at.
    pub scanned: usize,
    /// Entries a sandbox account owned.
    pub sandbox_owned: usize,
    /// Of those, taken for the user.
    pub taken: usize,
    /// Of those, not taken (the user may not take them, or they changed
    /// while being taken).
    pub not_taken: usize,
    /// Of those not taken, renamed in place.
    pub set_aside: usize,
    /// Entries whose owner could not be read.
    pub unreadable: usize,
    /// Stopped at [`MAX_ENTRIES`].
    pub capped: bool,
    /// D456: code caches found ([`code_cache`]).
    pub code_caches: usize,
    /// Of those, renamed in place.
    pub code_caches_set_aside: usize,
    /// More were found than [`MAX_CODE_CACHES`]: those weren't renamed.
    pub code_caches_capped: bool,
}

/// The SIDs (as strings) of the sandbox's accounts: the members of its
/// users group that are local or domain accounts.
pub(crate) fn sandbox_account_sids() -> Result<Vec<String>> {
    use windows_sys::Win32::NetworkManagement::NetManagement::LOCALGROUP_MEMBERS_INFO_0;
    use windows_sys::Win32::NetworkManagement::NetManagement::MAX_PREFERRED_LENGTH;
    use windows_sys::Win32::NetworkManagement::NetManagement::NetApiBufferFree;
    use windows_sys::Win32::NetworkManagement::NetManagement::NetLocalGroupGetMembers;
    use windows_sys::Win32::Security::GetLengthSid;
    let group = crate::winutil::to_wide(crate::token::SANDBOX_USERS_GROUP);
    let mut buffer: *mut u8 = std::ptr::null_mut();
    let (mut read, mut total, mut resume) = (0_u32, 0_u32, 0_usize);
    // SAFETY: valid out pointers; `buffer` is freed below.
    let status = unsafe {
        NetLocalGroupGetMembers(
            std::ptr::null(),
            group.as_ptr(),
            0,
            &mut buffer,
            MAX_PREFERRED_LENGTH,
            &mut read,
            &mut total,
            &mut resume,
        )
    };
    if status != 0 {
        if !buffer.is_null() {
            // SAFETY: allocated by NetLocalGroupGetMembers.
            unsafe { NetApiBufferFree(buffer.cast()) };
        }
        anyhow::bail!("list the members of the sandbox's group: {status}");
    }
    let mut sids = Vec::new();
    for i in 0..read as usize {
        // SAFETY: `buffer` holds `read` entries; each SID is valid while
        // `buffer` lives.
        let sid = unsafe {
            let entry = &*(buffer as *const LOCALGROUP_MEMBERS_INFO_0).add(i);
            let len = GetLengthSid(entry.lgrmi0_sid) as usize;
            std::slice::from_raw_parts(entry.lgrmi0_sid as *const u8, len).to_vec()
        };
        if let Ok(sid) = crate::winutil::string_from_sid_bytes(&sid)
            && sid.starts_with("S-1-5-21-")
        {
            sids.push(sid);
        }
    }
    if !buffer.is_null() {
        // SAFETY: allocated by NetLocalGroupGetMembers.
        unsafe { NetApiBufferFree(buffer.cast()) };
    }
    Ok(sids)
}

/// Takes for the user each object in `base` (the folder itself excluded,
/// links not followed, the user's session temp folders skipped) that one of
/// `accounts` owns, and sets aside one the user may not take. Best effort.
pub(crate) fn take_sandbox_leftovers(base: &Path, accounts: &[String]) -> Result<Leftovers> {
    let mut found = Leftovers::default();
    let user = current_user_sid()?;
    // A group that named this user would make the user's own objects look
    // like the sandbox's.
    let accounts: Vec<&String> = accounts
        .iter()
        .filter(|sid| !sid.eq_ignore_ascii_case(&user))
        .collect();
    // Even with none (accounts since removed), the code caches (D456).
    let user_sid = LocalSid::from_string(&user)?;
    // The user, SYSTEM, Administrators and OWNER RIGHTS (the user, once the
    // owner).
    let trusted_sids = [user.as_str(), "S-1-5-18", "S-1-5-32-544", "S-1-3-4"]
        .into_iter()
        .map(LocalSid::from_string)
        .collect::<Result<Vec<_>>>()?;
    let trusted: Vec<_> = trusted_sids.iter().map(LocalSid::as_ptr).collect();
    let base_dir = open(base, FILE_READ_ATTRIBUTES)?;
    // SAFETY: the handle is open.
    let base_key = comparable(&unsafe { final_path(base_dir.as_raw_handle() as _) }?);
    let mut stack: Vec<PathBuf> = vec![base.to_path_buf()];
    let mut caches: Vec<PathBuf> = Vec::new();
    'walk: while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if found.scanned >= MAX_ENTRIES {
                found.capped = true;
                break 'walk;
            }
            found.scanned += 1;
            let path = entry.path();
            let session_folder = dir == base
                && entry
                    .file_name()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .starts_with(SESSION_TEMP_PREFIX);
            let Ok(look) = open(&path, READ_CONTROL | FILE_READ_ATTRIBUTES) else {
                found.unreadable += 1;
                continue;
            };
            let Ok(attributes) = look.metadata().map(|meta| meta.file_attributes()) else {
                found.unreadable += 1;
                continue;
            };
            let Ok(owner) = owner_sid(&look) else {
                found.unreadable += 1;
                continue;
            };
            // A session's folder (the user's, as sessions make them) is its
            // session's to clean up.
            if session_folder && owner.eq_ignore_ascii_case(&user) {
                continue;
            }
            if accounts
                .iter()
                .any(|account| account.eq_ignore_ascii_case(&owner))
            {
                found.sandbox_owned += 1;
                match take_one(&path, &look, &base_key, user_sid.as_ptr(), &trusted) {
                    Take::Taken => found.taken += 1,
                    Take::Changed => found.not_taken += 1,
                    Take::Refused => {
                        found.not_taken += 1;
                        if set_aside(&path, &look, &base_key, SET_ASIDE) {
                            found.set_aside += 1;
                        }
                        // Not walked: not the user's.
                        continue;
                    }
                }
            }
            let is_link = attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0;
            let is_folder = attributes & FILE_ATTRIBUTE_DIRECTORY != 0;
            // D456: a link too, by its name only.
            if (is_folder || is_link) && code_cache(&entry.file_name(), dir == base) {
                if caches.len() < MAX_CODE_CACHES {
                    caches.push(path.clone());
                } else {
                    found.code_caches_capped = true;
                }
            }
            // Taken (and so made the user's) before what is in it.
            if is_folder && !is_link {
                stack.push(path);
            }
        }
    }
    // Once what is in them is taken; one inside another first.
    found.code_caches = caches.len();
    for cache in caches.iter().rev() {
        let Ok(look) = open(cache, READ_CONTROL | FILE_READ_ATTRIBUTES) else {
            continue;
        };
        // A folder, or a link (opened, and so renamed, itself).
        let folder_or_link = look.metadata().is_ok_and(|meta| {
            meta.file_attributes() & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
        });
        if folder_or_link && set_aside(cache, &look, &base_key, CACHE_SET_ASIDE) {
            found.code_caches_set_aside += 1;
        }
    }
    Ok(found)
}

/// D456: whether a folder (or link) named `name` (`top`: at the top of the
/// temp folder) is a code cache another program runs code from, by name.
fn code_cache(name: &std::ffi::OsStr, top: bool) -> bool {
    let name = name.to_string_lossy().to_ascii_lowercase();
    name == PYTHON_CODE_CACHE
        || (top
            && (TOP_LEVEL_CODE_CACHES.contains(&name.as_str())
                || TOP_LEVEL_CODE_CACHE_PREFIXES
                    .iter()
                    .any(|prefix| name.starts_with(prefix))))
}

enum Take {
    Taken,
    /// Not the object looked at any more, or not in the temp folder.
    Changed,
    /// The user may not take it.
    Refused,
}

/// Takes `path` for the user if it still is the object `look` is open to
/// and lies in the temp folder whose final path is `base_key`.
fn take_one(
    path: &Path,
    look: &File,
    base_key: &str,
    user: *mut std::ffi::c_void,
    trusted: &[*mut std::ffi::c_void],
) -> Take {
    let Ok(take) = open(
        path,
        READ_CONTROL | WRITE_DAC | WRITE_OWNER | FILE_READ_ATTRIBUTES,
    ) else {
        return Take::Refused;
    };
    // SAFETY: both handles are open.
    let same = unsafe {
        matches!(
            (
                file_identity(look.as_raw_handle() as _),
                file_identity(take.as_raw_handle() as _),
            ),
            (Ok(a), Ok(b)) if a == b
        )
    };
    // A folder on the way swapped for a link would lead elsewhere.
    // SAFETY: the handle is open.
    let inside = unsafe { final_path(take.as_raw_handle() as _) }
        .is_ok_and(|found| comparable(&found).starts_with(&format!("{base_key}\\")));
    if !(same && inside) {
        return Take::Changed;
    }
    // SAFETY: the handle is open with the rights `take_for_user` needs; the
    // SIDs are valid.
    match unsafe { take_for_user(take.as_raw_handle() as _, user, trusted) } {
        Ok(()) => Take::Taken,
        Err(_) => Take::Refused,
    }
}

/// Renames `path` in its folder, to end with `.{suffix}-` and a random
/// number, if it still is the object `look` is open to and lies in the temp
/// folder whose final path is `base_key` (the user may: the folder is the
/// user's or was taken first). While the rename's handle is open, neither it
/// nor a folder on the way can be renamed or replaced.
fn set_aside(path: &Path, look: &File, base_key: &str, suffix: &str) -> bool {
    use windows_sys::Win32::Storage::FileSystem::FILE_RENAME_INFO;
    use windows_sys::Win32::Storage::FileSystem::FileRenameInfo;
    use windows_sys::Win32::Storage::FileSystem::SetFileInformationByHandle;
    // Without delete sharing: no one can move the object (and so free the
    // folders above it) while it is checked and renamed.
    let Ok(moving) = std::fs::OpenOptions::new()
        .access_mode(DELETE | FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
    else {
        return false;
    };
    // SAFETY: both handles are open.
    let same = unsafe {
        matches!(
            (
                file_identity(look.as_raw_handle() as _),
                file_identity(moving.as_raw_handle() as _),
            ),
            (Ok(a), Ok(b)) if a == b
        )
    };
    // SAFETY: the handle is open.
    let Ok(found) = (unsafe { final_path(moving.as_raw_handle() as _) }) else {
        return false;
    };
    if !same || !comparable(&found).starts_with(&format!("{base_key}\\")) {
        return false;
    }
    // A full path, in the form that takes no length limit.
    let verbatim = match found.strip_prefix(r"\\") {
        Some(unc) => format!(r"\\?\UNC\{unc}"),
        None => format!(r"\\?\{found}"),
    };
    let target: Vec<u16> = format!(
        "{verbatim}.{suffix}-{:08x}",
        SmallRng::from_entropy().next_u32()
    )
    .encode_utf16()
    .collect();
    let name_offset = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    let size = name_offset + (target.len() + 1) * std::mem::size_of::<u16>();
    // u64 storage keeps the structure aligned.
    let mut buffer = vec![0_u64; size.div_ceil(8)];
    let info = buffer.as_mut_ptr() as *mut FILE_RENAME_INFO;
    // SAFETY: `buffer` holds the structure and the name after it; the
    // handle is open with `DELETE`.
    unsafe {
        (*info).FileNameLength = (target.len() * std::mem::size_of::<u16>()) as u32;
        std::ptr::copy_nonoverlapping(
            target.as_ptr(),
            (info as *mut u8).add(name_offset) as *mut u16,
            target.len(),
        );
        SetFileInformationByHandle(
            moving.as_raw_handle() as _,
            FileRenameInfo,
            info as *const std::ffi::c_void,
            size as u32,
        ) != 0
    }
}

fn open(path: &Path, access: u32) -> std::io::Result<File> {
    std::fs::OpenOptions::new()
        .access_mode(access)
        .share_mode(FILE_SHARE_ALL)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
}

fn comparable(path: &str) -> String {
    path.trim_end_matches('\\').to_ascii_lowercase()
}

#[cfg(test)]
#[path = "session_temp_leftovers_tests.rs"]
mod tests;
