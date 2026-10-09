//! #323: the session registry a sync checks before it removes any entry.

use super::DENY_READ_SYNC_LOCK_WAIT;
use super::DenyReadSessions;
use super::ProcessSessions;
use super::Registration;
use super::lock_out_other_sessions;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::time::Duration;

fn sessions(registry: &tempfile::TempDir, own: Option<&Registration>) -> DenyReadSessions {
    DenyReadSessions::in_test_registry(registry.path(), own)
}

fn register(registry: &Path) -> Registration {
    Registration::register(
        registry,
        /*trust_this_user*/ true,
        DENY_READ_SYNC_LOCK_WAIT,
    )
    .expect("register")
}

#[test]
fn sec_win_323_a_sync_removes_only_while_no_other_session_lives() {
    let registry = tempfile::tempdir().expect("registry");
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_some());

    let first = register(registry.path());
    let second = register(registry.path());
    assert_ne!(first.name(), second.name());
    // Another session lives: no removal, whoever asks.
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_none());
    assert!(lock_out_other_sessions(&sessions(&registry, Some(&first))).is_none());
    drop(second);
    // Only the caller's own session is left.
    assert!(lock_out_other_sessions(&sessions(&registry, Some(&first))).is_some());
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_none());
    drop(first);
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_some());

    // A session that died without cleaning up (its file is not locked)
    // counts as gone, and its file is deleted.
    let dead = registry.path().join("session-1-dead.lock");
    std::fs::write(&dead, b"").expect("dead session file");
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_some());
    assert!(!dead.exists(), "left a dead session's file");

    // No registry, or one that can't be read: nothing may be removed.
    assert!(lock_out_other_sessions(&DenyReadSessions::default()).is_none());
    let file = registry.path().join("a-file");
    std::fs::write(&file, b"").expect("file");
    assert!(
        lock_out_other_sessions(&DenyReadSessions::in_test_registry(
            &file, /*own*/ None
        ))
        .is_none()
    );
}

/// A session that registers while a sync is between its check and its
/// removals waits for it; its launch then puts back whatever was removed.
#[test]
fn sec_win_323_registering_waits_for_a_sync_that_is_removing() {
    let registry = tempfile::tempdir().expect("registry");
    let removing =
        lock_out_other_sessions(&sessions(&registry, /*own*/ None)).expect("no other session");
    let path = registry.path().to_path_buf();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let registering = std::thread::spawn(move || {
        let registration = Registration::register(
            &path,
            /*trust_this_user*/ true,
            Duration::from_secs(10),
        )
        .expect("register");
        let _ = done_tx.send(());
        registration
    });
    assert_eq!(
        done_rx.recv_timeout(Duration::from_millis(300)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout),
        "registered while a sync was removing"
    );
    drop(removing);
    done_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("registered once the sync was done");
    let registration = registering.join().expect("registering thread");
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_none());
    assert!(lock_out_other_sessions(&sessions(&registry, Some(&registration))).is_some());
}

/// Review M1: no one can delete or rename a live session's file, which
/// would make it look gone to every sync.
#[test]
fn sec_win_323_a_live_sessions_file_cannot_be_deleted_or_renamed() {
    let registry = tempfile::tempdir().expect("registry");
    let session = register(registry.path());
    let renamed = registry.path().join("renamed");
    let deleted = std::fs::remove_file(session.path()).is_ok();
    let moved = std::fs::rename(session.path(), &renamed).is_ok();
    eprintln!("sec-win-323: live session file deleted: {deleted}; renamed: {moved}");
    assert!(!deleted, "deleted a live session's file");
    assert!(!moved, "renamed a live session's file");
    assert!(lock_out_other_sessions(&sessions(&registry, /*own*/ None)).is_none());
}

/// Review M2: a registry folder that is a link, or that the wrong user owns,
/// is not trusted: nothing is removed and no one registers there.
#[test]
fn sec_win_323_a_linked_or_foreign_owned_registry_is_refused() {
    let dir = tempfile::tempdir().expect("dir");
    let target = dir.path().join("target");
    std::fs::create_dir(&target).expect("target");
    let link = dir.path().join("registry");
    let status = std::process::Command::new("cmd")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .output()
        .expect("mklink");
    assert!(status.status.success(), "{status:?}");
    let linked = DenyReadSessions::in_test_registry(&link, /*own*/ None);
    assert!(
        lock_out_other_sessions(&linked).is_none(),
        "trusted a junction"
    );
    assert!(
        Registration::register(
            &link,
            /*trust_this_user*/ true,
            DENY_READ_SYNC_LOCK_WAIT
        )
        .is_err(),
        "registered through a junction"
    );
    // The same folder, not through the link, is fine.
    assert!(
        lock_out_other_sessions(&DenyReadSessions::in_test_registry(
            &target, /*own*/ None
        ))
        .is_some()
    );

    // A folder this user owns is not trusted outside tests: only one
    // Administrators or SYSTEM own is.
    let owned = dir.path().join("owned");
    std::fs::create_dir(&owned).expect("owned");
    set_owner_to_this_user(&owned);
    let production = DenyReadSessions {
        registry: Some(owned.clone()),
        own: None,
        trust_this_user: false,
    };
    assert!(
        lock_out_other_sessions(&production).is_none(),
        "trusted a user's folder"
    );
    assert!(
        Registration::register(
            &owned,
            /*trust_this_user*/ false,
            DENY_READ_SYNC_LOCK_WAIT
        )
        .is_err(),
        "registered in a user's folder"
    );
}

fn set_owner_to_this_user(path: &Path) {
    use windows_sys::Win32::Security::Authorization::SE_FILE_OBJECT;
    use windows_sys::Win32::Security::Authorization::SetNamedSecurityInfoW;
    use windows_sys::Win32::Security::OWNER_SECURITY_INFORMATION;
    let user = super::current_user_sid().expect("this user");
    let sid = crate::LocalSid::from_string(&user).expect("SID");
    let wide = crate::winutil::to_wide(path);
    // SAFETY: valid path and SID.
    let status = unsafe {
        SetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            sid.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(status, 0, "set the owner");
}

/// Review M3: once a process has launched with two rule sets (a launch
/// without rules is one too), its own session counts as another's in its
/// syncs.
#[test]
fn sec_win_323_a_process_with_two_rule_sets_keeps_its_own_entries() {
    for other in ["path:b", ""] {
        let registry = tempfile::tempdir().expect("registry");
        let mut process = ProcessSessions::new();
        let mut view = |rules: Option<&str>| {
            process.view(
                Some(registry.path().to_path_buf()),
                rules.map(str::to_string),
                /*trust_this_user*/ true,
            )
        };
        let (first, error) = view(Some("path:a"));
        assert!(error.is_none(), "{error:?}");
        assert!(first.own.is_some());
        assert!(lock_out_other_sessions(&first).is_some());
        // The same rules again, or a launch whose sync doesn't run: still
        // its own.
        assert_eq!(view(Some("path:a")).0, first);
        assert_eq!(view(/*rules*/ None).0, first);
        let (second, _) = view(Some(other));
        assert_eq!(second.own, None, "{other:?}");
        assert!(
            lock_out_other_sessions(&second).is_none(),
            "removed under {other:?} what path:a relies on"
        );
        assert_eq!(view(Some("path:a")).0.own, None);
    }
}

/// Review M3': a launch whose profile has no deny-read rules still syncs
/// (removing what this home added), so it counts as a rule set of its own.
#[test]
fn sec_win_323_a_launch_without_rules_is_a_rule_set() {
    use crate::setup::deny_read_rule_set;
    let empty = crate::DenyReadTargets::default();
    assert_eq!(deny_read_rule_set(Some(&empty)), Some(String::new()));
    assert_eq!(deny_read_rule_set(/*deny_read*/ None), None);
    let path = codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path(
        std::env::temp_dir().join("Secret"),
    )
    .expect("absolute");
    let one = crate::DenyReadTargets::from_exact_paths([path]);
    let rules = deny_read_rule_set(Some(&one)).expect("rules");
    assert!(!rules.is_empty());
    assert_eq!(
        rules,
        rules.to_lowercase(),
        "keys as Windows compares paths"
    );
}

/// The elevated setup creates the machine-wide registry, owned by
/// Administrators, and this process registers in it. Run elevated first and
/// then in a normal session (as CI does), this also checks that a normal
/// session's Core may register there.
#[test]
fn sec_win_323_this_process_registers_in_program_data() {
    let registry = super::registry_dir().expect("ProgramData");
    if crate::setup::is_elevated().expect("elevation") {
        let group = crate::winutil::resolve_sid("CodexSandboxUsers")
            .ok()
            .and_then(|sid| crate::winutil::string_from_sid_bytes(&sid).ok());
        assert_eq!(
            super::ensure_registry(group.as_deref()).expect("set up the registry"),
            registry
        );
    }
    super::check_registry(&registry, /*trust_this_user*/ false)
        .expect("the elevated setup's registry (run this test elevated first)");
    let name = super::register_this_process(DENY_READ_SYNC_LOCK_WAIT).expect("register");
    assert_eq!(
        super::register_this_process(DENY_READ_SYNC_LOCK_WAIT).expect("again"),
        name
    );
    assert!(registry.join(&name).is_file(), "{}", registry.display());
    assert!(std::fs::remove_file(registry.join(&name)).is_err());
}
