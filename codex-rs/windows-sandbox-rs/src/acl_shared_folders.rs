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
//! - [`SharedFolderDenyKind::Write`] where they give Everyone a create
//!   right.
//!
//! Each entry goes first in the DACL, so it wins over the folder's own
//! allows (and, inherited, over its subfolders' inherited ones). Every entry
//! is recorded in `.sandbox-secrets`, which sandboxed commands can't open, so
//! [`remove_recorded_shared_folder_denies`] removes exactly those.

use super::INHERIT_ONLY_ACE;
use super::INHERITED_ACE;
use super::NO_PROPAGATE_INHERIT_ACE;
use super::remove_matching_aces;
use crate::setup::sandbox_secrets_dir;
use crate::token::LocalSid;
use crate::winutil::to_wide;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
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
use windows_sys::Win32::Security::Authorization::SetNamedSecurityInfoW;
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

/// The record [`apply_shared_folder_denies`] keeps, in `.sandbox-secrets`.
pub const SHARED_FOLDER_DENIES_FILE: &str = "shared_folder_denies.json";

/// The deny entries the setup adds for the sandbox users' group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SharedFolderDenyKind {
    /// #1791: `FILE_DELETE_CHILD`, inherited by subfolders.
    DeleteChild,
    /// D400: creating, writing and deleting (no `SYNCHRONIZE` or
    /// `READ_CONTROL`, so reads still work), inherited by files and
    /// subfolders.
    Write,
}

impl SharedFolderDenyKind {
    pub fn mask(self) -> u32 {
        match self {
            Self::DeleteChild => FILE_DELETE_CHILD,
            Self::Write => {
                FILE_WRITE_DATA
                    | FILE_APPEND_DATA
                    | FILE_WRITE_EA
                    | FILE_WRITE_ATTRIBUTES
                    | DELETE
                    | FILE_DELETE_CHILD
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
    pub seconds: f64,
}

/// What one allow entry gives a shared group that a deny should close:
/// `(delete_child, everyone_create)`. Only entries that apply to the folder
/// itself or the folders it will hold count; on a folder below a root only
/// its own entries count (inherited ones come from a folder the scan
/// checks).
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
    let applies_to_subfolders = u32::from(ace_flags) & CONTAINER_INHERIT_ACE != 0
        && u32::from(ace_flags) & NO_PROPAGATE_INHERIT_ACE == 0;
    if !applies_here && !applies_to_subfolders {
        return (false, false);
    }
    let delete_child = mask & (FILE_DELETE_CHILD | GENERIC_ALL) != 0;
    let everyone_create = sid == EVERYONE
        && mask & (FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY | GENERIC_WRITE | GENERIC_ALL) != 0;
    (delete_child, everyone_create)
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

/// Whether `path`'s DACL itself holds `kind`'s entry for `psid`, exactly.
///
/// # Safety
/// `psid` must point to a valid SID.
pub unsafe fn has_shared_folder_deny(
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> Result<bool> {
    let dacl = Dacl::read(path)?;
    Ok(dacl
        .entries()
        .into_iter()
        .any(|(ace_type, flags, mask, sid)| is_exact(ace_type, flags, mask, sid, psid, kind)))
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

/// Adds `kind`'s entry for `psid` first in `path`'s DACL unless it is there.
/// Built by hand, not with `SetEntriesInAclW`, which may merge it into
/// another deny for the same SID (a deny-read entry), so that it can be
/// removed exactly. Returns whether it added the entry; fails if it isn't
/// there afterwards.
///
/// # Safety
/// `psid` must point to a valid SID and `path` must be an existing folder.
pub unsafe fn add_shared_folder_deny(
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> Result<bool> {
    if has_shared_folder_deny(path, psid, kind)? {
        return Ok(false);
    }
    let dacl = Dacl::read(path)?;
    if dacl.p_dacl.is_null() {
        return Err(anyhow!("{} has no DACL", path.display()));
    }
    let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
    if GetAclInformation(
        dacl.p_dacl as *const ACL,
        &mut info as *mut _ as *mut c_void,
        std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
        AclSizeInformation,
    ) == 0
    {
        return Err(anyhow!("GetAclInformation failed: {}", GetLastError()));
    }
    let revision = u32::from((*dacl.p_dacl).AclRevision).max(2);
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
        if GetAce(dacl.p_dacl as *const ACL, i, &mut p_ace) == 0 {
            return Err(anyhow!("GetAce failed: {}", GetLastError()));
        }
        let ace_size = u32::from((*(p_ace as *const ACE_HEADER)).AceSize);
        if AddAce(new_dacl, revision, u32::MAX, p_ace, ace_size) == 0 {
            return Err(anyhow!("AddAce failed: {}", GetLastError()));
        }
    }
    let mut control: u16 = 0;
    let mut sd_revision: u32 = 0;
    if GetSecurityDescriptorControl(dacl.p_sd, &mut control, &mut sd_revision) == 0 {
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
    // ones, here and below.
    let code = SetNamedSecurityInfoW(
        to_wide(path).as_ptr() as *mut u16,
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
    if !has_shared_folder_deny(path, psid, kind)? {
        return Err(anyhow!(
            "the {kind:?} deny is missing from {} after writing it",
            path.display()
        ));
    }
    Ok(true)
}

/// Removes `kind`'s entry for `psid` from `path`'s DACL, exactly: an entry
/// with other rights or inheritance is left alone. Returns whether it was
/// there.
///
/// # Safety
/// `psid` must point to a valid SID and `path` must exist.
pub unsafe fn remove_shared_folder_deny(
    path: &Path,
    psid: *mut c_void,
    kind: SharedFolderDenyKind,
) -> Result<bool> {
    remove_matching_aces(path, |_, p_ace| {
        let hdr = &*(p_ace as *const ACE_HEADER);
        let mask = (*(p_ace as *const ACCESS_DENIED_ACE)).Mask;
        let sid = (p_ace as usize + std::mem::size_of::<ACE_HEADER>() + std::mem::size_of::<u32>())
            as *mut c_void;
        is_exact(hdr.AceType, hdr.AceFlags, mask, sid, psid, kind)
    })
}

/// One deny entry the setup added (or found and adopted).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SharedFolderDenyEntry {
    pub path: PathBuf,
    pub kind: SharedFolderDenyKind,
    pub sid: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFolderDenyRecord {
    pub entries: Vec<SharedFolderDenyEntry>,
}

pub fn shared_folder_denies_path(codex_home: &Path) -> PathBuf {
    sandbox_secrets_dir(codex_home).join(SHARED_FOLDER_DENIES_FILE)
}

pub fn load_shared_folder_deny_record(codex_home: &Path) -> Result<SharedFolderDenyRecord> {
    let path = shared_folder_denies_path(codex_home);
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

fn save_record(codex_home: &Path, record: &SharedFolderDenyRecord) -> Result<()> {
    let path = shared_folder_denies_path(codex_home);
    if record.entries.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
                Err(err).with_context(|| format!("removing {}", path.display()))
            }
            _ => Ok(()),
        };
    }
    let dir = sandbox_secrets_dir(codex_home);
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let tmp = dir.join(format!("{SHARED_FOLDER_DENIES_FILE}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec_pretty(record)?)
        .with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, &path).with_context(|| format!("replacing {}", path.display()))
}

#[derive(Clone, Debug, Default)]
pub struct SharedFolderDenyReport {
    pub added: Vec<SharedFolderFinding>,
    pub already: usize,
    pub errors: Vec<String>,
}

/// Adds each finding's deny for the sandbox users' group and records it.
/// The record is written before the DACLs, so an interrupted run leaves no
/// unrecorded entry; an entry that isn't there makes the removal a no-op.
/// A folder that can't be changed is reported, not fatal.
///
/// # Safety
/// `group_psid` must point to the SID `group_sid` names.
pub unsafe fn apply_shared_folder_denies(
    codex_home: &Path,
    group_sid: &str,
    group_psid: *mut c_void,
    findings: &[SharedFolderFinding],
) -> Result<SharedFolderDenyReport> {
    // A record that can't be read fails the run rather than being replaced,
    // which would lose the entries it lists.
    let mut record = load_shared_folder_deny_record(codex_home)?;
    for finding in findings {
        let entry = SharedFolderDenyEntry {
            path: finding.path.clone(),
            kind: finding.kind,
            sid: group_sid.to_string(),
        };
        if !record.entries.contains(&entry) {
            record.entries.push(entry);
        }
    }
    record.entries.sort();
    save_record(codex_home, &record)?;
    let mut report = SharedFolderDenyReport::default();
    for finding in findings {
        match add_shared_folder_deny(&finding.path, group_psid, finding.kind) {
            Ok(true) => report.added.push(finding.clone()),
            Ok(false) => report.already += 1,
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
    pub absent: usize,
    pub errors: Vec<String>,
}

/// Removes every recorded entry, exactly, and forgets those it removed or
/// found gone; one it couldn't remove stays recorded.
pub fn remove_recorded_shared_folder_denies(codex_home: &Path) -> Result<SharedFolderDenyRemoval> {
    let record = load_shared_folder_deny_record(codex_home)?;
    let mut kept = SharedFolderDenyRecord::default();
    let mut removal = SharedFolderDenyRemoval::default();
    for entry in record.entries {
        if !entry.path.exists() {
            removal.absent += 1;
            continue;
        }
        let result = LocalSid::from_string(&entry.sid).and_then(|sid| unsafe {
            remove_shared_folder_deny(&entry.path, sid.as_ptr(), entry.kind)
        });
        match result {
            Ok(true) => removal.removed += 1,
            Ok(false) => removal.absent += 1,
            Err(err) => {
                removal.errors.push(format!(
                    "{:?} deny on {}: {err:#}",
                    entry.kind,
                    entry.path.display()
                ));
                kept.entries.push(entry);
            }
        }
    }
    save_record(codex_home, &kept)?;
    Ok(removal)
}

#[cfg(test)]
#[path = "acl_shared_folders_tests.rs"]
mod tests;
