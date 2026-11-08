//! #1791 and D400 (owner decisions D405–D407; needs security sign-off): the
//! elevated backend's one-time admin setup closes shared folders outside the
//! workspace to the sandbox users' group.
//!
//! Elevated commands run as a sandbox account with a write-restricted token.
//! Windows doesn't consult the restricting SIDs for `FILE_DELETE_CHILD`
//! (#158), so a folder that grants Users, Authenticated Users, INTERACTIVE
//! (or another group every signed-in token holds) Full control lets a
//! command delete other people's files in it. Everyone is one of the
//! restricting SIDs, so a folder that grants Everyone write lets a command
//! write there (D400).
//!
//! The setup scans `ProgramData`, `Users\Public` and `Windows\Temp` with
//! every folder below them, and each top-level folder of the system drive
//! (D407: the folder itself, not below it), and denies the sandbox users'
//! group:
//! - [`SharedFolderDenyKind::DeleteChild`] where a folder's own entries (a
//!   root's inherited ones too) give such a group delete-child;
//! - [`SharedFolderDenyKind::Write`] where they give Everyone a right to
//!   write, delete or change permissions in it or in the files it holds.
//!
//! Both also deny `WRITE_DAC` and `WRITE_OWNER`, so a command can't take the
//! entry out again. Each entry goes first in the DACL, so it wins over the
//! folder's own allows (and, inherited, over its subfolders' inherited ones).
//!
//! Every write goes through a handle opened without following links
//! ([`super::open_exact`]): a folder swapped for a junction after the scan is
//! refused, not followed. Every entry added is recorded, with the folder's
//! identity and the homes whose setups rely on it, in
//! `%ProgramData%\CorbanuTerminalSandbox\shared-folder-denies`, which only
//! Administrators and SYSTEM may use. [`remove_shared_folder_denies_for_home`]
//! removes an entry, exactly, once no recorded home relies on it.

use super::DaclTarget;
use super::INHERIT_ONLY_ACE;
use super::INHERITED_ACE;
use super::open_exact;
use super::remove_aces_from;
use super::security_info;
use crate::token::LocalSid;
use crate::winutil::to_wide;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;
use std::ffi::c_void;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::ACCESS_ALLOWED_ACE;
use windows_sys::Win32::Security::ACCESS_DENIED_ACE;
use windows_sys::Win32::Security::ACE_HEADER;
use windows_sys::Win32::Security::ACL;
use windows_sys::Win32::Security::ACL_SIZE_INFORMATION;
use windows_sys::Win32::Security::AclSizeInformation;
use windows_sys::Win32::Security::AddAccessDeniedAceEx;
use windows_sys::Win32::Security::AddAce;
use windows_sys::Win32::Security::Authorization::GetNamedSecurityInfoW;
use windows_sys::Win32::Security::Authorization::SetSecurityInfo;
use windows_sys::Win32::Security::CONTAINER_INHERIT_ACE;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetAclInformation;
use windows_sys::Win32::Security::GetLengthSid;
use windows_sys::Win32::Security::GetSecurityDescriptorControl;
use windows_sys::Win32::Security::InitializeAcl;
use windows_sys::Win32::Security::OBJECT_INHERIT_ACE;
use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Security::SE_DACL_PROTECTED;
use windows_sys::Win32::Security::UNPROTECTED_DACL_SECURITY_INFORMATION;
use windows_sys::Win32::Storage::FileSystem::DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_ADD_FILE;
use windows_sys::Win32::Storage::FileSystem::FILE_ADD_SUBDIRECTORY;
use windows_sys::Win32::Storage::FileSystem::FILE_APPEND_DATA;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_DELETE_CHILD;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_DATA;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_EA;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;
use windows_sys::Win32::Storage::FileSystem::WRITE_OWNER;

const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
const ACCESS_DENIED_ACE_TYPE: u8 = 1;
const GENERIC_ALL: u32 = 0x1000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const SE_FILE_OBJECT: i32 = 1;
const EVERYONE: &str = "S-1-1-0";

/// Groups every signed-in token holds (the sandbox account's too): Users,
/// Authenticated Users, INTERACTIVE, BATCH, This Organization, Local
/// account, LOCAL, CONSOLE LOGON and Everyone.
pub const SHARED_GROUP_SIDS: [&str; 9] = [
    "S-1-5-32-545",
    "S-1-5-11",
    "S-1-5-4",
    "S-1-5-3",
    "S-1-5-15",
    "S-1-5-113",
    "S-1-2-0",
    "S-1-2-1",
    EVERYONE,
];

/// The machine's record, in [`SHARED_FOLDER_RECORD_DIR`].
pub const SHARED_FOLDER_RECORD_FILE: &str = "record.json";
const SHARED_FOLDER_RECORD_LOCK: &str = "record.lock";
/// Under `%ProgramData%\CorbanuTerminalSandbox`.
pub const SHARED_FOLDER_RECORD_DIR: &str = "shared-folder-denies";
/// Rights an allow for Everyone may give that the [`SharedFolderDenyKind::Write`]
/// deny closes (Everyone is a restricting SID, so they pass both checks).
const EVERYONE_WRITE_RIGHTS: u32 = FILE_ADD_FILE
    | FILE_ADD_SUBDIRECTORY
    | FILE_WRITE_EA
    | FILE_WRITE_ATTRIBUTES
    | DELETE
    | FILE_DELETE_CHILD
    | WRITE_DAC
    | WRITE_OWNER
    | GENERIC_WRITE
    | GENERIC_ALL;

/// The deny entries the setup adds for the sandbox users' group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SharedFolderDenyKind {
    /// #1791: `FILE_DELETE_CHILD` (and changing permissions or owner),
    /// inherited by subfolders.
    DeleteChild,
    /// D400: creating, writing, deleting and changing permissions or owner
    /// (no `SYNCHRONIZE` or `READ_CONTROL`, so reads still work), inherited
    /// by files and subfolders.
    Write,
}

impl SharedFolderDenyKind {
    pub fn mask(self) -> u32 {
        match self {
            Self::DeleteChild => FILE_DELETE_CHILD | WRITE_DAC | WRITE_OWNER,
            Self::Write => {
                FILE_WRITE_DATA
                    | FILE_APPEND_DATA
                    | FILE_WRITE_EA
                    | FILE_WRITE_ATTRIBUTES
                    | DELETE
                    | FILE_DELETE_CHILD
                    | WRITE_DAC
                    | WRITE_OWNER
            }
        }
    }

    pub fn ace_flags(self) -> u8 {
        let flags = match self {
            Self::DeleteChild => CONTAINER_INHERIT_ACE,
            Self::Write => CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE,
        };
        flags as u8
    }
}

/// Where the scan looks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedFolderScanRoots {
    /// Scanned with every folder below them.
    pub recursive: Vec<PathBuf>,
    /// Scanned themselves only.
    pub single: Vec<PathBuf>,
    /// Not entered (container image layers, not live folders).
    pub skipped: Vec<PathBuf>,
}

/// `ProgramData`, `Users\Public` and `Windows\Temp` recursively; each
/// top-level folder of the system drive by itself (D407).
pub fn default_shared_folder_scan_roots() -> SharedFolderScanRoots {
    let env_dir = |name: &str, fallback: &str| {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(fallback))
    };
    let system_drive = env_dir("SystemDrive", "C:");
    let drive_root = PathBuf::from(format!("{}\\", system_drive.display()));
    let program_data = env_dir("ProgramData", "C:\\ProgramData");
    let recursive = vec![
        program_data.clone(),
        env_dir("PUBLIC", "C:\\Users\\Public"),
        env_dir("SystemRoot", "C:\\Windows").join("Temp"),
    ];
    let single = std::fs::read_dir(&drive_root)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry
                        .metadata()
                        .is_ok_and(|meta| meta.is_dir() && !is_reparse_point(&meta))
                })
                .map(|entry| entry.path())
                .filter(|path| !recursive.iter().any(|root| same_path(root, path)))
                .collect()
        })
        .unwrap_or_default();
    SharedFolderScanRoots {
        recursive,
        single,
        skipped: vec![program_data.join("docker").join("windowsfilter")],
    }
}

fn is_reparse_point(meta: &std::fs::Metadata) -> bool {
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        p.to_string_lossy()
            .trim_end_matches('\\')
            .to_ascii_lowercase()
    };
    norm(a) == norm(b)
}

/// A folder the scan found, and the deny it needs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedFolderFinding {
    pub path: PathBuf,
    pub kind: SharedFolderDenyKind,
}

#[derive(Clone, Debug, Default)]
pub struct SharedFolderScan {
    pub findings: Vec<SharedFolderFinding>,
    pub visited: usize,
    /// Folders whose DACL or listing couldn't be read.
    pub unreadable: usize,
    /// Folders with no DACL at all (open to everyone): reported, not changed.
    pub no_dacl: Vec<PathBuf>,
    pub seconds: f64,
}

/// What one allow entry gives a shared group that a deny should close:
/// `(delete_child, everyone_write)`. Delete-child counts when the entry
/// applies to the folder itself or to subfolders (the deny reaches all of
/// them); Everyone's rights also when it applies to files. On a folder below
/// a root only its own entries count (inherited ones come from a folder the
/// scan checks).
pub(crate) fn classify_allow(
    shared_group: Option<&str>,
    ace_flags: u8,
    mask: u32,
    is_root: bool,
) -> (bool, bool) {
    let Some(sid) = shared_group else {
        return (false, false);
    };
    if ace_flags & INHERITED_ACE != 0 && !is_root {
        return (false, false);
    }
    let applies_here = ace_flags & INHERIT_ONLY_ACE == 0;
    // With or without NO_PROPAGATE: it reaches at least the subfolders
    // directly below.
    let applies_to_subfolders = u32::from(ace_flags) & CONTAINER_INHERIT_ACE != 0;
    let applies_to_files = u32::from(ace_flags) & OBJECT_INHERIT_ACE != 0;
    let delete_child =
        (applies_here || applies_to_subfolders) && mask & (FILE_DELETE_CHILD | GENERIC_ALL) != 0;
    let everyone_write = sid == EVERYONE
        && (applies_here || applies_to_subfolders || applies_to_files)
        && mask & EVERYONE_WRITE_RIGHTS != 0;
    (delete_child, everyone_write)
}

struct SharedGroups(Vec<(&'static str, LocalSid)>);

impl SharedGroups {
    fn new() -> Result<Self> {
        SHARED_GROUP_SIDS
            .iter()
            .map(|sid| Ok((*sid, LocalSid::from_string(sid)?)))
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }

    /// The shared group `psid` is, if any.
    unsafe fn find(&self, psid: *mut c_void) -> Option<&'static str> {
        self.0
            .iter()
            .find(|(_, sid)| unsafe { EqualSid(psid, sid.as_ptr()) } != 0)
            .map(|(name, _)| *name)
    }
}

/// The DACL of `path`, freed on drop.
struct Dacl {
    p_sd: *mut c_void,
    p_dacl: *mut ACL,
}

impl Dacl {
    unsafe fn read(path: &Path) -> Result<Self> {
        let mut p_sd: *mut c_void = std::ptr::null_mut();
        let mut p_dacl: *mut ACL = std::ptr::null_mut();
        let code = GetNamedSecurityInfoW(
            to_wide(path).as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut p_dacl,
            std::ptr::null_mut(),
            &mut p_sd,
        );
        if code != ERROR_SUCCESS {
            return Err(anyhow!(
                "reading the DACL of {} failed: {code}",
                path.display()
            ));
        }
        Ok(Self { p_sd, p_dacl })
    }

    /// Each entry: (type, flags, mask, SID).
    unsafe fn entries(&self) -> Vec<(u8, u8, u32, *mut c_void)> {
        let mut out = Vec::new();
        if self.p_dacl.is_null() {
            return out;
        }
        let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
        if GetAclInformation(
            self.p_dacl as *const ACL,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        ) == 0
        {
            return out;
        }
        for i in 0..info.AceCount {
            let mut p_ace: *mut c_void = std::ptr::null_mut();
            if GetAce(self.p_dacl as *const ACL, i, &mut p_ace) == 0 {
                continue;
            }
            let hdr = &*(p_ace as *const ACE_HEADER);
            // Allowed and denied entries share their layout.
            if hdr.AceType != ACCESS_ALLOWED_ACE_TYPE && hdr.AceType != ACCESS_DENIED_ACE_TYPE {
                continue;
            }
            let mask = (*(p_ace as *const ACCESS_ALLOWED_ACE)).Mask;
            let sid = (p_ace as usize
                + std::mem::size_of::<ACE_HEADER>()
                + std::mem::size_of::<u32>()) as *mut c_void;
            out.push((hdr.AceType, hdr.AceFlags, mask, sid));
        }
        out
    }
}

impl Drop for Dacl {
    fn drop(&mut self) {
        if !self.p_sd.is_null() {
            unsafe {
                LocalFree(self.p_sd as HLOCAL);
            }
        }
    }
}

/// Walks the roots and returns the folders that need a deny. Links and other
/// reparse points are neither checked nor entered.
pub fn scan_shared_folders(roots: &SharedFolderScanRoots) -> Result<SharedFolderScan> {
    let started = Instant::now();
    let groups = SharedGroups::new()?;
    let mut scan = SharedFolderScan::default();
    let mut stack: Vec<(PathBuf, bool, bool)> = Vec::new();
    for root in roots.single.iter().rev() {
        stack.push((root.clone(), true, false));
    }
    for root in roots.recursive.iter().rev() {
        stack.push((root.clone(), true, true));
    }
    while let Some((dir, is_root, recurse)) = stack.pop() {
        let Ok(meta) = std::fs::symlink_metadata(&dir) else {
            continue;
        };
        if !meta.is_dir() || is_reparse_point(&meta) {
            continue;
        }
        scan.visited += 1;
        match unsafe { Dacl::read(&dir) } {
            Ok(dacl) if dacl.p_dacl.is_null() => scan.no_dacl.push(dir.clone()),
            Ok(dacl) => {
                let (mut delete_child, mut everyone_create) = (false, false);
                for (ace_type, flags, mask, sid) in unsafe { dacl.entries() } {
                    if ace_type != ACCESS_ALLOWED_ACE_TYPE {
                        continue;
                    }
                    let (dc, ew) =
                        classify_allow(unsafe { groups.find(sid) }, flags, mask, is_root);
                    delete_child |= dc;
                    everyone_create |= ew;
                }
                if delete_child {
                    scan.findings.push(SharedFolderFinding {
                        path: dir.clone(),
                        kind: SharedFolderDenyKind::DeleteChild,
                    });
                }
                if everyone_create {
                    scan.findings.push(SharedFolderFinding {
                        path: dir.clone(),
                        kind: SharedFolderDenyKind::Write,
                    });
                }
            }
            Err(_) => scan.unreadable += 1,
        }
        if !recurse {
            continue;
        }
        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                let mut children: Vec<PathBuf> = entries
                    .flatten()
                    .filter(|entry| {
                        entry
                            .metadata()
                            .is_ok_and(|meta| meta.is_dir() && !is_reparse_point(&meta))
                    })
                    .map(|entry| entry.path())
                    .filter(|path| !roots.skipped.iter().any(|skip| same_path(skip, path)))
                    .collect();
                children.sort();
                for child in children.into_iter().rev() {
                    stack.push((child, false, true));
                }
            }
            Err(_) => scan.unreadable += 1,
        }
    }
    scan.seconds = started.elapsed().as_secs_f64();
    Ok(scan)
}

/// A folder's identity: volume serial number and 128-bit file ID.
pub type FolderIdentity = (u64, [u8; 16]);

/// Whether `path`'s DACL itself holds `kind`'s entry for `psid`, exactly.
/// Reads by path (following links): for checks, not for decisions to write.
///
/// # Safety
/// `psid` must point to a valid SID.
pub unsafe fn has_shared_folder_deny(
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> Result<bool> {
    let dacl = Dacl::read(path)?;
    Ok(dacl_has_exact(dacl.p_dacl, psid, kind))
}

unsafe fn dacl_has_exact(p_dacl: *mut ACL, psid: *mut c_void, kind: SharedFolderDenyKind) -> bool {
    let dacl = Dacl {
        p_sd: std::ptr::null_mut(),
        p_dacl,
    };
    let found = dacl
        .entries()
        .into_iter()
        .any(|(ace_type, flags, mask, sid)| is_exact(ace_type, flags, mask, sid, psid, kind));
    std::mem::forget(dacl);
    found
}

unsafe fn is_exact(
    ace_type: u8,
    flags: u8,
    mask: u32,
    sid: *mut c_void,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> bool {
    ace_type == ACCESS_DENIED_ACE_TYPE
        && flags == kind.ace_flags()
        && mask == kind.mask()
        && EqualSid(sid, psid) != 0
}

/// The folder `path` names, opened without following a link, with its DACL.
struct OpenFolder {
    handle: super::ExactHandle,
    p_sd: *mut c_void,
    p_dacl: *mut ACL,
}

impl OpenFolder {
    unsafe fn open(path: &Path) -> Result<Self> {
        let handle = open_exact(path, READ_CONTROL | WRITE_DAC)?;
        if !handle.is_dir {
            return Err(anyhow!("{} is not a folder", path.display()));
        }
        let (p_dacl, p_sd) = security_info(handle.raw)?;
        Ok(Self {
            handle,
            p_sd,
            p_dacl,
        })
    }

    fn identity(&self) -> FolderIdentity {
        (self.handle.volume, self.handle.file_id)
    }

    unsafe fn reread(&mut self) -> Result<()> {
        let (p_dacl, p_sd) = security_info(self.handle.raw)?;
        LocalFree(self.p_sd as HLOCAL);
        (self.p_dacl, self.p_sd) = (p_dacl, p_sd);
        Ok(())
    }
}

impl Drop for OpenFolder {
    fn drop(&mut self) {
        if !self.p_sd.is_null() {
            unsafe {
                LocalFree(self.p_sd as HLOCAL);
            }
        }
    }
}

/// Writes `kind`'s entry for `psid` first in the open folder's DACL unless
/// it is there. Built by hand, not with `SetEntriesInAclW`, which may merge
/// it into another deny for the same SID (a deny-read entry), so that it can
/// be removed exactly. Returns whether it added the entry; fails if it isn't
/// there afterwards.
unsafe fn add_to_open_folder(
    folder: &mut OpenFolder,
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> Result<bool> {
    // Another finding for the same folder may have changed it since it was
    // opened.
    folder.reread()?;
    if dacl_has_exact(folder.p_dacl, psid, kind) {
        return Ok(false);
    }
    if folder.p_dacl.is_null() {
        return Err(anyhow!("{} has no DACL", path.display()));
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    if GetAclInformation(
        folder.p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    ) == 0
    {
        return Err(anyhow!("GetAclInformation failed: {}", GetLastError()));
    }
    let revision = u32::from((*folder.p_dacl).AclRevision).max(2);
    let ace_size = std::mem::size_of::<ACCESS_DENIED_ACE>() - std::mem::size_of::<u32>()
        + GetLengthSid(psid) as usize;
    let size = info.AclBytesInUse as usize + ace_size;
    let size = u32::try_from(size.div_ceil(4) * 4).context("DACL too large")?;
    let mut buffer = vec![0_u32; size as usize / 4];
    let new_dacl = buffer.as_mut_ptr() as *mut ACL;
    if InitializeAcl(new_dacl, size, revision) == 0 {
        return Err(anyhow!("InitializeAcl failed: {}", GetLastError()));
    }
    if AddAccessDeniedAceEx(
        new_dacl,
        revision,
        u32::from(kind.ace_flags()),
        kind.mask(),
        psid,
    ) == 0
    {
        return Err(anyhow!("AddAccessDeniedAceEx failed: {}", GetLastError()));
    }
    for i in 0..info.AceCount {
        let mut p_ace: *mut c_void = std::ptr::null_mut();
        if GetAce(folder.p_dacl as *const ACL, i, &mut p_ace) == 0 {
            return Err(anyhow!("GetAce failed: {}", GetLastError()));
        }
        let ace_size = u32::from((*(p_ace as *const ACE_HEADER)).AceSize);
        if AddAce(new_dacl, revision, u32::MAX, p_ace, ace_size) == 0 {
            return Err(anyhow!("AddAce failed: {}", GetLastError()));
        }
    }
    let mut control: u16 = 0;
    let mut sd_revision: u32 = 0;
    if GetSecurityDescriptorControl(folder.p_sd, &mut control, &mut sd_revision) == 0 {
        return Err(anyhow!(
            "GetSecurityDescriptorControl failed: {}",
            GetLastError()
        ));
    }
    let protection = if control & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    // Windows writes only the explicit entries and recomputes the inherited
    // ones, here and below (without following links below).
    let code = SetSecurityInfo(
        folder.handle.raw,
        SE_FILE_OBJECT,
        DACL_SECURITY_INFORMATION | protection,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        new_dacl,
        std::ptr::null_mut(),
    );
    if code != ERROR_SUCCESS {
        return Err(anyhow!(
            "writing the DACL of {} failed: {code}",
            path.display()
        ));
    }
    folder.reread()?;
    if !dacl_has_exact(folder.p_dacl, psid, kind) {
        return Err(anyhow!(
            "the {kind:?} deny is missing from {} after writing it",
            path.display()
        ));
    }
    Ok(true)
}

/// Adds `kind`'s entry for `psid` to the folder `path` names (refusing a
/// link); returns whether it added it and the folder's identity.
///
/// # Safety
/// `psid` must point to a valid SID.
#[cfg(test)]
pub(crate) unsafe fn add_shared_folder_deny(
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> Result<(bool, FolderIdentity)> {
    let mut folder = OpenFolder::open(path)?;
    let added = add_to_open_folder(&mut folder, path, psid, kind)?;
    Ok((added, folder.identity()))
}

/// Removes `kind`'s entry for `psid` from the folder `path` names, exactly:
/// an entry with other rights or inheritance is left alone. Fails for a
/// link; a folder that isn't `identity` (when given) is left alone. Returns
/// whether it removed the entry.
///
/// # Safety
/// `psid` must point to a valid SID.
pub unsafe fn remove_shared_folder_deny(
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
    identity: Option<FolderIdentity>,
) -> Result<bool> {
    let folder = OpenFolder::open(path)?;
    if identity.is_some_and(|identity| identity != folder.identity()) {
        return Ok(false);
    }
    remove_aces_from(
        DaclTarget::Handle(folder.handle.raw),
        folder.p_sd,
        folder.p_dacl,
        |_, p_ace| {
            let hdr = &*(p_ace as *const ACE_HEADER);
            let mask = (*(p_ace as *const ACCESS_DENIED_ACE)).Mask;
            let sid = (p_ace as usize
                + std::mem::size_of::<ACE_HEADER>()
                + std::mem::size_of::<u32>()) as *mut c_void;
            is_exact(hdr.AceType, hdr.AceFlags, mask, sid, psid, kind)
        },
    )
}

/// One deny entry the setup added, and the homes whose setups rely on it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SharedFolderDenyEntry {
    pub path: PathBuf,
    pub kind: SharedFolderDenyKind,
    /// The sandbox users' group.
    pub sid: String,
    pub volume: u64,
    /// The folder's file ID, in hex.
    pub file_id: String,
    pub homes: BTreeSet<String>,
}

impl SharedFolderDenyEntry {
    fn identity(&self) -> Option<FolderIdentity> {
        let id = u128::from_str_radix(&self.file_id, 16).ok()?;
        Some((self.volume, id.to_le_bytes()))
    }
}

fn file_id_hex(identity: FolderIdentity) -> String {
    format!("{:032x}", u128::from_le_bytes(identity.1))
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFolderDenyRecord {
    pub entries: Vec<SharedFolderDenyEntry>,
}

/// The machine's record folder, without making it.
pub fn machine_shared_folder_record_dir() -> Option<PathBuf> {
    crate::deny_read_sessions::admin_state_dir(SHARED_FOLDER_RECORD_DIR)
}

/// Elevated setup: makes (or repairs) the machine's record folder, which
/// only Administrators and SYSTEM may use.
pub fn ensure_machine_shared_folder_record_dir(sandbox_group: &str) -> Result<PathBuf> {
    crate::deny_read_sessions::ensure_admin_state_dir(SHARED_FOLDER_RECORD_DIR, sandbox_group)
}

/// How the record names a home.
fn home_key(codex_home: &Path) -> String {
    codex_home
        .to_string_lossy()
        .trim_end_matches('\\')
        .to_lowercase()
}

/// The record in `record_dir`; empty if there is none.
pub fn load_shared_folder_deny_record(record_dir: &Path) -> Result<SharedFolderDenyRecord> {
    let path = record_dir.join(SHARED_FOLDER_RECORD_FILE);
    match std::fs::read(&path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Ok(SharedFolderDenyRecord::default())
        }
        Err(err) => Err(err).with_context(|| format!("reading {}", path.display())),
    }
}

/// The record, or (for one that can't be parsed, moved aside first) an empty
/// one. Its entries stay in their folders, unrecorded: still protecting,
/// but no longer removed.
fn load_or_set_aside(record_dir: &Path, notes: &mut Vec<String>) -> Result<SharedFolderDenyRecord> {
    match load_shared_folder_deny_record(record_dir) {
        Ok(record) => Ok(record),
        Err(err) => {
            let path = record_dir.join(SHARED_FOLDER_RECORD_FILE);
            if !path.exists() {
                return Err(err);
            }
            let aside = record_dir.join(format!(
                "{SHARED_FOLDER_RECORD_FILE}.damaged-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|elapsed| elapsed.as_secs())
                    .unwrap_or_default()
            ));
            std::fs::rename(&path, &aside)
                .with_context(|| format!("moving the damaged {} aside", path.display()))?;
            notes.push(format!("{err:#}; moved to {}", aside.display()));
            Ok(SharedFolderDenyRecord::default())
        }
    }
}

/// Held while a setup reads, changes and saves the record, so two at once
/// don't lose each other's updates. Only Administrators and SYSTEM may open
/// the record folder, so no one else can hold it.
fn lock_record(record_dir: &Path) -> Result<std::fs::File> {
    use std::os::windows::fs::OpenOptionsExt;
    const ERROR_SHARING_VIOLATION: i32 = 32;
    let path = record_dir.join(SHARED_FOLDER_RECORD_LOCK);
    let started = Instant::now();
    loop {
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(&path)
        {
            Ok(file) => return Ok(file),
            Err(err)
                if err.raw_os_error() == Some(ERROR_SHARING_VIOLATION)
                    && started.elapsed() < std::time::Duration::from_secs(120) =>
            {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(err) => return Err(err).with_context(|| format!("locking {}", path.display())),
        }
    }
}

fn save_record(record_dir: &Path, record: &SharedFolderDenyRecord) -> Result<()> {
    let path = record_dir.join(SHARED_FOLDER_RECORD_FILE);
    let tmp = record_dir.join(format!("{SHARED_FOLDER_RECORD_FILE}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec_pretty(record)?)
        .with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| format!("replacing {}", path.display()))
}

#[derive(Clone, Debug, Default)]
pub struct SharedFolderDenyReport {
    pub added: Vec<SharedFolderFinding>,
    /// Already there and recorded: this home now relies on them too.
    pub already: usize,
    /// Already there but not recorded (not ours): left alone.
    pub foreign: usize,
    pub errors: Vec<String>,
    /// A damaged record set aside.
    pub notes: Vec<String>,
}

/// Adds each finding's deny for the sandbox users' group and records it in
/// `record_dir` for `codex_home`. Every folder is opened (refusing links)
/// and the record written before any DACL changes, so an interrupted run
/// leaves no unrecorded entry. A folder that can't be opened or changed is
/// reported, not fatal.
///
/// # Safety
/// `group_psid` must point to the SID `group_sid` names.
pub unsafe fn apply_shared_folder_denies(
    record_dir: &Path,
    codex_home: &Path,
    group_sid: &str,
    group_psid: *mut c_void,
    findings: &[SharedFolderFinding],
) -> Result<SharedFolderDenyReport> {
    let _lock = lock_record(record_dir)?;
    let mut report = SharedFolderDenyReport::default();
    let mut record = load_or_set_aside(record_dir, &mut report.notes)?;
    let home = home_key(codex_home);
    let mut to_add: Vec<(OpenFolder, &SharedFolderFinding)> = Vec::new();
    for finding in findings {
        let folder = match OpenFolder::open(&finding.path) {
            Ok(folder) => folder,
            Err(err) => {
                report.errors.push(format!(
                    "{:?} deny on {}: {err:#}",
                    finding.kind,
                    finding.path.display()
                ));
                continue;
            }
        };
        let identity = folder.identity();
        // The folder, however its path is spelled.
        let recorded = record.entries.iter_mut().find(|entry| {
            entry.kind == finding.kind
                && entry.sid == group_sid
                && entry.identity() == Some(identity)
        });
        let present = dacl_has_exact(folder.p_dacl, group_psid, finding.kind);
        match (recorded, present) {
            (Some(entry), true) => {
                entry.homes.insert(home.clone());
                report.already += 1;
            }
            (None, true) => report.foreign += 1,
            (recorded, false) => {
                match recorded {
                    Some(entry) => {
                        entry.homes.insert(home.clone());
                    }
                    None => record.entries.push(SharedFolderDenyEntry {
                        path: finding.path.clone(),
                        kind: finding.kind,
                        sid: group_sid.to_string(),
                        volume: identity.0,
                        file_id: file_id_hex(identity),
                        homes: BTreeSet::from([home.clone()]),
                    }),
                }
                to_add.push((folder, finding));
            }
        }
    }
    record.entries.sort();
    save_record(record_dir, &record)?;
    for (mut folder, finding) in to_add {
        match add_to_open_folder(&mut folder, &finding.path, group_psid, finding.kind) {
            Ok(_) => report.added.push(finding.clone()),
            Err(err) => report.errors.push(format!(
                "{:?} deny on {}: {err:#}",
                finding.kind,
                finding.path.display()
            )),
        }
    }
    Ok(report)
}

#[derive(Clone, Debug, Default)]
pub struct SharedFolderDenyRemoval {
    pub removed: usize,
    /// Still relied on by another home: kept.
    pub kept: usize,
    /// Gone already (the folder or the entry).
    pub absent: usize,
    pub errors: Vec<String>,
}

/// Forgets `codex_home` in every entry and removes, exactly, each entry no
/// recorded home relies on any more. Only entries for `group_sid` (this
/// machine's sandbox users' group) are touched. A folder now replaced by
/// another one is left alone and forgotten; one that is now a link, or can't
/// be changed, is left alone and stays recorded.
///
/// # Safety
/// `group_psid` must point to the SID `group_sid` names.
pub unsafe fn remove_shared_folder_denies_for_home(
    record_dir: &Path,
    codex_home: &Path,
    group_sid: &str,
    group_psid: *mut c_void,
) -> Result<SharedFolderDenyRemoval> {
    let _lock = lock_record(record_dir)?;
    let record = load_shared_folder_deny_record(record_dir)?;
    let home = home_key(codex_home);
    let mut kept = SharedFolderDenyRecord::default();
    let mut removal = SharedFolderDenyRemoval::default();
    for mut entry in record.entries {
        if entry.sid != group_sid || !entry.homes.remove(&home) {
            kept.entries.push(entry);
            continue;
        }
        if !entry.homes.is_empty() {
            removal.kept += 1;
            kept.entries.push(entry);
            continue;
        }
        if !entry.path.exists() {
            removal.absent += 1;
            continue;
        }
        match remove_shared_folder_deny(&entry.path, group_psid, entry.kind, entry.identity()) {
            Ok(true) => removal.removed += 1,
            Ok(false) => removal.absent += 1,
            Err(err) => {
                removal.errors.push(format!(
                    "{:?} deny on {}: {err:#}",
                    entry.kind,
                    entry.path.display()
                ));
                entry.homes.insert(home.clone());
                kept.entries.push(entry);
            }
        }
    }
    save_record(record_dir, &kept)?;
    Ok(removal)
}

#[cfg(test)]
#[path = "acl_shared_folders_tests.rs"]
mod tests;
