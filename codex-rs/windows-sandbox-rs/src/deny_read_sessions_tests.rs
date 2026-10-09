//! #323: the session registry a sync checks before it removes any entry.

use super::DenyReadSessions;
use super::Registration;
use super::lock_out_other_sessions;
use pretty_assertions::assert_eq;
use std::time::Duration;

fn sessions(registry: &tempfile::TempDir, own: Option<&Registration>) -> DenyReadSessions {
    DenyReadSessions {
        registry: Some(registry.path().to_path_buf()),
        own: own.map(|registration| registration.name().to_string()),
    }
}

#[test]
fn sec_win_323_a_sync_removes_only_while_no_other_session_lives() {
    let registry = tempfile::tempdir().expect("registry");
    assert!(lock_out_other_sessions(&sessions(&registry, None)).is_some());

    let first = Registration::register(registry.path()).expect("register");
    let second = Registration::register(registry.path()).expect("register again");
    assert_ne!(first.name(), second.name());
    // Another session lives: no removal, whoever asks.
    assert!(lock_out_other_sessions(&sessions(&registry, None)).is_none());
    assert!(lock_out_other_sessions(&sessions(&registry, Some(&first))).is_none());
    drop(second);
    // Only the caller's own session is left.
    assert!(lock_out_other_sessions(&sessions(&registry, Some(&first))).is_some());
    assert!(lock_out_other_sessions(&sessions(&registry, None)).is_none());
    drop(first);
    assert!(lock_out_other_sessions(&sessions(&registry, None)).is_some());

    // A session that died without cleaning up (its file is not locked)
    // counts as gone, and its file is deleted.
    let dead = registry.path().join("session-1-dead.lock");
    std::fs::write(&dead, b"").expect("dead session file");
    assert!(lock_out_other_sessions(&sessions(&registry, None)).is_some());
    assert!(!dead.exists(), "left a dead session's file");

    // No registry, or one that can't be read: nothing may be removed.
    assert!(lock_out_other_sessions(&DenyReadSessions::default()).is_none());
    let file = registry.path().join("a-file");
    std::fs::write(&file, b"").expect("file");
    let missing = DenyReadSessions {
        registry: Some(file.join("registry")),
        own: None,
    };
    assert!(lock_out_other_sessions(&missing).is_none());
}

/// A session that registers while a sync is between its check and its
/// removals waits for it; its launch then puts back whatever was removed.
#[test]
fn sec_win_323_registering_waits_for_a_sync_that_is_removing() {
    let registry = tempfile::tempdir().expect("registry");
    let removing = lock_out_other_sessions(&sessions(&registry, None)).expect("no other session");
    let path = registry.path().to_path_buf();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let registering = std::thread::spawn(move || {
        let registration = Registration::register(&path).expect("register");
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
    assert!(lock_out_other_sessions(&sessions(&registry, None)).is_none());
    assert!(lock_out_other_sessions(&sessions(&registry, Some(&registration))).is_some());
}
