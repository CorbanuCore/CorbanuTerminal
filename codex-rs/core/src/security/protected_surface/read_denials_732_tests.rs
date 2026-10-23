#![cfg(unix)]
use super::*;
use codex_protocol::permissions::FileSystemGlobScan;
use pretty_assertions::assert_eq;

#[test]
fn separate_database_rules_keep_top_level_provenance_and_read_time_protection() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    for holds_home in [false, true] {
        let folder = root.join(if holds_home {
            "holds-home[1]"
        } else {
            "holds-work[1]"
        });
        let home = if holds_home {
            folder.join("home")
        } else {
            root.join("other-home")
        };
        let cwd = folder.join("project");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&cwd).unwrap();
        for name in [
            "state.sqlite",
            "state.sqlite-wal",
            "state.sqlite-shm",
            "state.sqlite-journal",
        ] {
            std::fs::write(folder.join(name), "fixture").unwrap();
        }
        std::fs::write(cwd.join("fixture.sqlite"), "project fixture").unwrap();
        let keep = [AbsolutePathBuf::from_absolute_path(&cwd).unwrap()];
        let mut denials = ReadDenials::collect(
            &home, /*user_home*/ None, /*claude_config_dir*/ None, &keep,
        );
        denials.protect_database_folder(&home, &folder, &keep, /*user_home*/ None, &root);
        let policy = denials
            .apply(&PermissionProfile::Disabled)
            .unwrap()
            .file_system_sandbox_policy();
        // The exec-server filesystem transport must retain the same hint.
        for entry in &policy.entries {
            let transport = codex_file_system::ExecFileSystemSandboxEntry::from(entry.clone());
            let bytes = serde_json::to_vec(&transport).unwrap();
            let decoded: codex_file_system::ExecFileSystemSandboxEntry =
                serde_json::from_slice(&bytes).unwrap();
            assert_eq!(FileSystemSandboxEntry::try_from(decoded).unwrap(), *entry);
        }
        let scans = policy.get_unreadable_glob_scans_with_cwd(&cwd);
        assert_eq!(
            scans
                .iter()
                .filter_map(|glob| glob.scan.clone())
                .collect::<Vec<_>>(),
            vec![FileSystemGlobScan::DatabaseEntries {
                directory: AbsolutePathBuf::from_absolute_path(&folder).unwrap()
            }]
        );
        let matcher = denials.matcher().unwrap();
        for name in [
            "state.sqlite",
            "state.sqlite-wal",
            "state.sqlite-shm",
            "state.sqlite-journal",
            "later.sqlite-wal",
        ] {
            assert!(matcher.is_read_denied(&folder.join(name)), "{name}");
        }
        assert!(!matcher.is_read_denied(&cwd.join("fixture.sqlite")));
    }
}
