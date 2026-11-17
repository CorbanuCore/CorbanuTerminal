use super::*;

/// Lease release and atomic record replacement can remove a listed entry
/// before validation opens it. This must not become an admin-recovery alarm.
#[test]
fn sec_win_1953_entry_removed_during_scan_is_not_untrusted() {
    let root = tempfile::tempdir().expect("private records");
    for name in ["session-ended.json", "labels.json.tmp", "registry.lock"] {
        let path = root.path().join(name);
        std::fs::write(&path, "host").expect("entry");
        let entry = std::fs::read_dir(root.path())
            .expect("list")
            .next()
            .expect("listed entry")
            .expect("entry path");
        std::fs::remove_file(&path).expect("concurrent removal");
        check_directory_entry(entry).expect("a vanished entry is not untrusted");
        // Required paths (the registry and its parent) still fail closed.
        assert!(check_path(&path).is_err());
    }
    check_entries(root.path()).expect("empty registry");
}
