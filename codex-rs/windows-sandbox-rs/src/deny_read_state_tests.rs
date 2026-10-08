//! #304: a deny-read path a later launch no longer lists loses exactly the
//! entry the sync added. #301: never while another process has the
//! secretless launch contract armed on the same `CODEX_HOME`.

use super::SECRETLESS_LAUNCH_LOCK_FILE;
use super::sync_persistent_deny_read_acls;
use crate::acl::add_allow_ace;
use crate::acl::add_deny_read_ace;
use crate::acl::add_deny_read_ace_for_new_files;
use crate::acl::ensure_explicit_deny_read_ace;
use crate::setup::sandbox_dir;
use crate::token::LocalSid;
use crate::winutil::to_wide;
use pretty_assertions::assert_eq;
use std::ffi::c_void;
use std::io::BufRead as _;
use std::io::BufReader;
use std::io::Read as _;
use std::io::Write as _;
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::GetNamedSecurityInfoW;
use windows_sys::Win32::Security::Authorization::SDDL_REVISION_1;
use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;

const SANDBOX_GROUP: &str = "S-1-5-21-2718281828-3141592653-1618033988-1001";
const OTHER_SID: &str = "S-1-5-21-2718281828-3141592653-1618033988-1002";
const ARMED_HOME_ENV: &str = "CODEX_SEC_WIN_301_ARMED_HOME";
const ARMED_ENTRY: &str = "deny_read_state::tests::sec_win_301_armed_session_entry";
const ARMED_LINE: &str = "sec-win-301: armed";

struct Home {
    _dir: tempfile::TempDir,
    codex_home: PathBuf,
    /// A protected directory, and a directory in it.
    secret: PathBuf,
    nested: PathBuf,
}

fn home() -> Home {
    let dir = tempfile::tempdir().expect("codex home");
    let codex_home = dunce::canonicalize(dir.path()).expect("canonical codex home");
    std::fs::create_dir_all(sandbox_dir(&codex_home)).expect("sandbox dir");
    let secret = codex_home.join("vault-secret");
    std::fs::create_dir(&secret).expect("secret dir");
    let nested = secret.join("nested");
    std::fs::create_dir(&nested).expect("nested dir");
    Home {
        _dir: dir,
        codex_home,
        secret,
        nested,
    }
}

/// One launch's deny-read sync for the sandbox group.
fn sync(home: &Home, paths: &[PathBuf], group: &LocalSid) {
    // SAFETY: `group` is a valid SID for the call.
    unsafe {
        sync_persistent_deny_read_acls(&home.codex_home, SANDBOX_GROUP, paths, group.as_ptr())
    }
    .expect("sync deny-read ACLs");
}

/// Whether the sync state still lists the protected directory.
fn recorded(home: &Home) -> bool {
    std::fs::read_to_string(sandbox_dir(&home.codex_home).join("deny_read_acl_state.json"))
        .is_ok_and(|state| state.contains("vault-secret"))
}

#[test]
fn sec_win_304_sync_removes_exactly_the_deny_it_added() {
    let home = home();
    let group = LocalSid::from_string(SANDBOX_GROUP).expect("group SID");
    let other = LocalSid::from_string(OTHER_SID).expect("other SID");
    // Entries the removal must leave: an allow for the same SID (which
    // `REVOKE_ACCESS` would take), another SID's deny, and another kind of
    // deny for the same SID.
    // SAFETY: valid SIDs and existing paths.
    unsafe {
        assert!(add_allow_ace(&home.secret, group.as_ptr()).expect("allow"));
        assert!(add_deny_read_ace(&home.secret, other.as_ptr()).expect("other deny"));
        assert!(add_deny_read_ace_for_new_files(&home.secret, group.as_ptr()).expect("new-file deny"));
    }
    let dir_before = dacl_sddl(&home.secret);
    let nested_before = dacl_sddl(&home.nested);

    sync(&home, std::slice::from_ref(&home.secret), &group);
    assert_ne!(dacl_sddl(&home.secret), dir_before);
    // Subdirectories inherit the deny (the new-file deny does not reach them).
    assert_ne!(dacl_sddl(&home.nested), nested_before);
    assert!(recorded(&home));

    // The rule is gone: the next launch no longer lists the path.
    sync(&home, &[], &group);
    assert_eq!(dacl_sddl(&home.secret), dir_before);
    assert_eq!(dacl_sddl(&home.nested), nested_before);
    assert!(!recorded(&home));
}

/// A read deny the sync found already in place (here, the explicit one the
/// launch contract gives its lock file) is not the sync's to remove.
#[test]
fn sec_win_304_sync_keeps_a_deny_it_did_not_add() {
    let home = home();
    let group = LocalSid::from_string(SANDBOX_GROUP).expect("group SID");
    // SAFETY: a valid SID and an existing directory.
    unsafe {
        assert!(ensure_explicit_deny_read_ace(&home.secret, group.as_ptr()).expect("deny"));
    }
    let denied = dacl_sddl(&home.secret);
    sync(&home, std::slice::from_ref(&home.secret), &group);
    assert_eq!(dacl_sddl(&home.secret), denied);
    sync(&home, &[], &group);
    assert_eq!(dacl_sddl(&home.secret), denied);
    assert!(!recorded(&home));
}

/// The armed session of [`sec_win_301_flag_off_session_keeps_denies_while_another_is_armed`]:
/// holds the lock as the launch contract does, until its stdin closes.
#[test]
fn sec_win_301_armed_session_entry() {
    let Some(codex_home) = std::env::var_os(ARMED_HOME_ENV) else {
        return;
    };
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_WRITE: u32 = 0x2;
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(Path::new(&codex_home).join(SECRETLESS_LAUNCH_LOCK_FILE))
        .expect("lock file");
    lock.try_lock_shared().expect("shared lock");
    let mut stdout = std::io::stdout();
    writeln!(stdout, "{ARMED_LINE}").expect("report");
    stdout.flush().expect("report");
    let _ = std::io::stdin().read_to_end(&mut Vec::new());
}

/// Starts another process that has the contract armed on `codex_home`.
fn armed_session(codex_home: &Path) -> Child {
    let mut child = Command::new(std::env::current_exe().expect("test binary"))
        .args([ARMED_ENTRY, "--exact", "--nocapture", "--test-threads=1"])
        .env(ARMED_HOME_ENV, codex_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start the armed session");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
    let mut line = String::new();
    // libtest prints the test's name without a newline before the line.
    while !line.trim_end().ends_with(ARMED_LINE) {
        line.clear();
        let read = stdout.read_line(&mut line).expect("armed session output");
        assert_ne!(read, 0, "the armed session did not take the lock");
    }
    // Keep reading, so its remaining output does not hit a closed pipe.
    std::thread::spawn(move || std::io::copy(&mut stdout, &mut std::io::sink()));
    child
}

#[test]
fn sec_win_301_flag_off_session_keeps_denies_while_another_is_armed() {
    let home = home();
    let group = LocalSid::from_string(SANDBOX_GROUP).expect("group SID");
    let unprotected = dacl_sddl(&home.secret);

    // Session A arms the contract and launches with its deny.
    let mut session_a = armed_session(&home.codex_home);
    sync(&home, std::slice::from_ref(&home.secret), &group);
    let protected = dacl_sddl(&home.secret);
    assert_ne!(protected, unprotected);

    // Session B, flag off, launches without that path while A runs.
    sync(&home, &[], &group);
    assert_eq!(
        dacl_sddl(&home.secret),
        protected,
        "removed under an armed session"
    );
    assert!(recorded(&home), "forgot a deny it did not remove");

    // A exits; B's next launch removes it.
    drop(session_a.stdin.take());
    assert!(session_a.wait().expect("armed session").success());
    sync(&home, &[], &group);
    assert_eq!(dacl_sddl(&home.secret), unprotected);
    assert!(!recorded(&home));
}

fn dacl_sddl(path: &Path) -> String {
    let mut sd: *mut c_void = std::ptr::null_mut();
    // SAFETY: valid path; `sd` is freed below.
    let code = unsafe {
        GetNamedSecurityInfoW(
            to_wide(path).as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut sd,
        )
    };
    assert_eq!(code, ERROR_SUCCESS, "GetNamedSecurityInfoW");
    let mut text: *mut u16 = std::ptr::null_mut();
    // SAFETY: `sd` is a valid descriptor; `text` is freed below.
    let ok = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(ok, 0, "ConvertSecurityDescriptorToStringSecurityDescriptorW");
    // SAFETY: `text` is a NUL-terminated string allocated by the call.
    let sddl = unsafe {
        let len = (0..).take_while(|&i| *text.add(i) != 0).count();
        String::from_utf16_lossy(std::slice::from_raw_parts(text, len))
    };
    // SAFETY: both were allocated by the calls above.
    unsafe {
        LocalFree(text as HLOCAL);
        LocalFree(sd as HLOCAL);
    }
    sddl
}
