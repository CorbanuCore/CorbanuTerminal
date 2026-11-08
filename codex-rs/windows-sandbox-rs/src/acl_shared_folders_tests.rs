//! #1791 / D400: what the scan finds, and that the denies go in and come out
//! exactly. No elevation needed: the test owns its folders.

use super::*;
use crate::acl::add_deny_read_ace;
use crate::acl::tests::dacl_sddl;
use pretty_assertions::assert_eq;

const GROUP: &str = "S-1-5-21-2718281828-3141592653-1618033988-1791";
const SYNCHRONIZE: u32 = 0x0010_0000;
const READ_CONTROL: u32 = 0x0002_0000;
const OI: u8 = 0x1;
const CI: u8 = 0x2;
const NP: u8 = 0x4;
const IO: u8 = 0x8;
const ID: u8 = 0x10;
const FULL: u32 = 0x001f_01ff;
const MODIFY: u32 = 0x0013_01bf;

fn icacls(path: &Path, args: &[&str]) {
    let output = std::process::Command::new("icacls.exe")
        .arg(path)
        .args(args)
        .output()
        .expect("run icacls");
    assert!(
        output.status.success(),
        "icacls {args:?}: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A temporary folder only SYSTEM, Administrators and this user may use,
/// whatever the runner's `TEMP` grants (the scan reads a root's inherited
/// entries too).
fn private_tempdir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tmp");
    let output = std::process::Command::new("whoami.exe")
        .arg("/user")
        .output()
        .expect("run whoami");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let user = stdout
        .split_whitespace()
        .find(|token| token.starts_with("S-1-"))
        .unwrap_or_else(|| panic!("unexpected whoami output: {stdout:?}"));
    let user = format!("*{user}:(OI)(CI)F");
    icacls(
        dir.path(),
        &[
            "/inheritance:r",
            "/grant:r",
            "*S-1-5-18:(OI)(CI)F",
            "*S-1-5-32-544:(OI)(CI)F",
            user.as_str(),
        ],
    );
    dir
}

fn folder(path: &Path, grant: Option<&str>) -> PathBuf {
    std::fs::create_dir_all(path).expect("create folder");
    if let Some(grant) = grant {
        icacls(path, &["/grant", grant]);
    }
    path.to_path_buf()
}

#[test]
fn shared_folders_classify_allow() {
    let users = Some("S-1-5-32-545");
    let everyone = Some(EVERYONE);
    // (group, flags, mask, root) -> (delete child, Everyone create)
    let cases = [
        (users, OI | CI, FULL, false, (true, false)),
        (users, 0, FILE_DELETE_CHILD, false, (true, false)),
        (users, 0, GENERIC_ALL, false, (true, false)),
        // Modify gives DELETE on each object, which the restricting SIDs
        // still check; not delete-child.
        (users, OI | CI, MODIFY, false, (false, false)),
        // Inherited: closed at the folder it comes from, unless a root.
        (users, OI | CI | ID, FULL, false, (false, false)),
        (users, OI | CI | ID, FULL, true, (true, false)),
        // For subfolders only: counts; for files only: doesn't.
        (users, CI | IO, FULL, false, (true, false)),
        (users, OI | IO, FULL, false, (false, false)),
        (users, CI | IO | NP, FULL, false, (false, false)),
        (everyone, OI | CI, MODIFY, false, (false, true)),
        (everyone, 0, FILE_ADD_SUBDIRECTORY, false, (false, true)),
        (everyone, 0, GENERIC_WRITE, false, (false, true)),
        (everyone, OI | CI, FULL, false, (true, true)),
        (everyone, OI | CI, 0x0012_00a9, false, (false, false)),
        // Users writing is the restricting SIDs' business (#1770).
        (
            users,
            OI | CI,
            MODIFY | FILE_ADD_FILE,
            false,
            (false, false),
        ),
        (None, OI | CI, FULL, false, (false, false)),
    ];
    for (group, flags, mask, root, expected) in cases {
        assert_eq!(
            classify_allow(group, flags, mask, root),
            expected,
            "{group:?} flags={flags:#x} mask={mask:#x} root={root}"
        );
    }
}

#[test]
fn shared_folders_denies_keep_reads_working() {
    assert_eq!(SharedFolderDenyKind::DeleteChild.mask(), FILE_DELETE_CHILD);
    for kind in [
        SharedFolderDenyKind::DeleteChild,
        SharedFolderDenyKind::Write,
    ] {
        assert_eq!(kind.mask() & (SYNCHRONIZE | READ_CONTROL), 0, "{kind:?}");
    }
    assert_eq!(SharedFolderDenyKind::Write.mask(), 0x0001_0156);
}

#[test]
fn shared_folders_scan_finds_shared_folders_only() {
    let tmp = private_tempdir();
    let root = tmp.path();
    let users_full = folder(&root.join("users-full"), Some("*S-1-5-32-545:(OI)(CI)F"));
    let auth_deep = folder(
        &root.join("plain").join("auth-full"),
        Some("*S-1-5-11:(OI)(CI)F"),
    );
    let interactive_subfolders = folder(
        &root.join("interactive-subfolders"),
        Some("*S-1-5-4:(CI)(IO)F"),
    );
    let everyone_modify = folder(&root.join("everyone-modify"), Some("*S-1-1-0:(OI)(CI)M"));
    folder(&root.join("users-modify"), Some("*S-1-5-32-545:(OI)(CI)M"));
    folder(&root.join("files-only"), Some("*S-1-5-32-545:(OI)(IO)F"));
    folder(&users_full.join("inherits"), None);
    let skipped = folder(&root.join("skipped"), Some("*S-1-5-32-545:(OI)(CI)F"));
    folder(&skipped.join("below"), Some("*S-1-5-11:(OI)(CI)F"));
    let link = root.join("link");
    let made = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&users_full)
        .output()
        .expect("mklink");
    assert!(made.status.success(), "mklink /J failed");
    let single_parent = private_tempdir();
    let single = folder(
        &single_parent.path().join("top"),
        Some("*S-1-5-32-545:(OI)(CI)F"),
    );
    folder(&single.join("below"), Some("*S-1-5-11:(OI)(CI)F"));

    let scan = scan_shared_folders(&SharedFolderScanRoots {
        recursive: vec![root.to_path_buf()],
        single: vec![single.clone()],
        skipped: vec![skipped],
    })
    .expect("scan");

    let mut found: Vec<(PathBuf, SharedFolderDenyKind)> = scan
        .findings
        .into_iter()
        .map(|finding| (finding.path, finding.kind))
        .collect();
    found.sort();
    let mut expected = vec![
        (users_full, SharedFolderDenyKind::DeleteChild),
        (auth_deep, SharedFolderDenyKind::DeleteChild),
        (interactive_subfolders, SharedFolderDenyKind::DeleteChild),
        (everyone_modify, SharedFolderDenyKind::Write),
        (single, SharedFolderDenyKind::DeleteChild),
    ];
    expected.sort();
    assert_eq!(found, expected);
    assert_eq!(scan.unreadable, 0);
}

#[test]
fn shared_folders_deny_goes_first_and_comes_out_exactly() {
    let tmp = tempfile::tempdir().expect("tmp");
    let dir = folder(&tmp.path().join("shared"), Some("*S-1-5-32-545:(OI)(CI)F"));
    std::fs::write(dir.join("file.txt"), "x").expect("file");
    let group = LocalSid::from_string(GROUP).expect("group SID");
    // A deny-read entry for the same group and inheritance as the write deny,
    // which `SetEntriesInAclW` could merge with it.
    // SAFETY: a valid SID and an existing folder.
    unsafe {
        assert!(add_deny_read_ace(&dir, group.as_ptr()).expect("deny read"));
    }
    let before = dacl_sddl(&dir);
    let file_before = dacl_sddl(&dir.join("file.txt"));
    for kind in [
        SharedFolderDenyKind::DeleteChild,
        SharedFolderDenyKind::Write,
    ] {
        // SAFETY: as above.
        unsafe {
            assert!(add_shared_folder_deny(&dir, group.as_ptr(), kind).expect("add"));
            assert!(!add_shared_folder_deny(&dir, group.as_ptr(), kind).expect("add again"));
            assert!(has_shared_folder_deny(&dir, group.as_ptr(), kind).expect("has"));
        }
        let sddl = dacl_sddl(&dir);
        let first = sddl
            .trim_start_matches("D:")
            .trim_start_matches(|c| c != '(')
            .split(')')
            .next()
            .unwrap_or_default()
            .to_string();
        assert!(
            first.starts_with("(D;") && first.ends_with(GROUP),
            "{kind:?} is not first: {sddl}"
        );
        // SAFETY: as above.
        unsafe {
            assert!(remove_shared_folder_deny(&dir, group.as_ptr(), kind).expect("remove"));
            assert!(!remove_shared_folder_deny(&dir, group.as_ptr(), kind).expect("remove again"));
        }
        assert_eq!(dacl_sddl(&dir), before, "{kind:?}");
        assert_eq!(dacl_sddl(&dir.join("file.txt")), file_before, "{kind:?}");
    }
}

#[test]
fn shared_folders_record_lists_what_the_removal_takes_out() {
    let tmp = tempfile::tempdir().expect("tmp");
    let home = tmp.path().join("home");
    let users_full = folder(
        &tmp.path().join("users-full"),
        Some("*S-1-5-32-545:(OI)(CI)F"),
    );
    let everyone = folder(&tmp.path().join("everyone"), Some("*S-1-1-0:(OI)(CI)M"));
    let gone = folder(&tmp.path().join("gone"), Some("*S-1-5-32-545:(OI)(CI)F"));
    let findings = vec![
        SharedFolderFinding {
            path: users_full.clone(),
            kind: SharedFolderDenyKind::DeleteChild,
        },
        SharedFolderFinding {
            path: everyone.clone(),
            kind: SharedFolderDenyKind::Write,
        },
        SharedFolderFinding {
            path: gone.clone(),
            kind: SharedFolderDenyKind::DeleteChild,
        },
    ];
    let before: Vec<String> = [&users_full, &everyone]
        .iter()
        .map(|path| dacl_sddl(path))
        .collect();
    let group = LocalSid::from_string(GROUP).expect("group SID");

    // SAFETY: `group` is the SID `GROUP` names.
    let report = unsafe { apply_shared_folder_denies(&home, GROUP, group.as_ptr(), &findings) }
        .expect("apply");
    assert_eq!(
        (report.added.len(), report.already, report.errors.len()),
        (3, 0, 0)
    );
    let record = load_shared_folder_deny_record(&home).expect("record");
    assert_eq!(record.entries.len(), 3);
    assert!(record.entries.iter().all(|entry| entry.sid == GROUP));

    // A re-run (an upgrade) adds nothing and records nothing twice.
    // SAFETY: as above.
    let again = unsafe { apply_shared_folder_denies(&home, GROUP, group.as_ptr(), &findings) }
        .expect("apply again");
    assert_eq!((again.added.len(), again.already), (0, 3));
    assert_eq!(
        load_shared_folder_deny_record(&home).expect("record"),
        record
    );

    std::fs::remove_dir(&gone).expect("remove a recorded folder");
    let removal = remove_recorded_shared_folder_denies(&home).expect("remove");
    assert_eq!(
        (removal.removed, removal.absent, removal.errors.len()),
        (2, 1, 0)
    );
    assert!(!shared_folder_denies_path(&home).exists());
    let after: Vec<String> = [&users_full, &everyone]
        .iter()
        .map(|path| dacl_sddl(path))
        .collect();
    assert_eq!(after, before);
}

#[test]
fn shared_folders_unreadable_record_is_not_replaced() {
    let tmp = tempfile::tempdir().expect("tmp");
    let home = tmp.path();
    let path = shared_folder_denies_path(home);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("secrets dir");
    std::fs::write(&path, "not json").expect("write");
    let group = LocalSid::from_string(GROUP).expect("group SID");
    // SAFETY: `group` is the SID `GROUP` names.
    let result = unsafe { apply_shared_folder_denies(home, GROUP, group.as_ptr(), &[]) };
    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "not json");
}
