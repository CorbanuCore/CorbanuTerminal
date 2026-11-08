use super::*;
use pretty_assertions::assert_eq;

fn stamp(version: &str) -> SharedFolderScanStamp {
    SharedFolderScanStamp {
        cli_version: version.to_string(),
        recorded: vec![RecordedSharedFolder {
            volume: 7,
            file_id: "00ff".to_string(),
            kind: SharedFolderDenyKind::Write,
        }],
        foreign: Vec::new(),
        unchanged: Vec::new(),
        unopened: vec![PathBuf::from(r"C:\ProgramData\Locked")],
        clean_versions: Vec::new(),
    }
}

fn finding(path: &str) -> SharedFolderFinding {
    SharedFolderFinding {
        path: PathBuf::from(path),
        kind: SharedFolderDenyKind::Write,
    }
}

#[test]
fn no_stamp_runs_the_admin_setup_without_scanning() {
    let decision = rescan_decision(None, "2.0.0", |_| panic!("no user scan")).expect("decision");
    assert!(
        matches!(decision, RescanDecision::AdminSetup(_)),
        "{decision:?}"
    );
}

#[test]
fn same_version_does_not_scan() {
    let decision =
        rescan_decision(Some(stamp("2.0.0")), "2.0.0", |_| panic!("no scan")).expect("decision");
    assert_eq!(decision, RescanDecision::NotDue);
}

#[test]
fn new_version_with_nothing_missing_keeps_the_setups_lists() {
    let decision = rescan_decision(Some(stamp("1.0.0")), "2.0.0", |seen| {
        assert_eq!(seen, &stamp("1.0.0"));
        Ok(Vec::new())
    })
    .expect("decision");
    let mut clean = stamp("1.0.0");
    clean.clean_versions.push("2.0.0".to_string());
    assert_eq!(decision, RescanDecision::Clean(clean.clone()));

    // Both versions sharing the home: neither scans again.
    for version in ["1.0.0", "2.0.0"] {
        let decision =
            rescan_decision(Some(clean.clone()), version, |_| panic!("no scan")).expect("decision");
        assert_eq!(decision, RescanDecision::NotDue);
    }
    // A third version scans.
    let decision = rescan_decision(Some(clean), "3.0.0", |_| Ok(Vec::new())).expect("decision");
    assert!(matches!(decision, RescanDecision::Clean(_)), "{decision:?}");
}

#[test]
fn new_version_with_a_folder_missing_its_deny_runs_the_admin_setup() {
    let decision = rescan_decision(Some(stamp("1.0.0")), "2.0.0", |_| {
        Ok(vec![finding(r"C:\ProgramData\NewVendor")])
    })
    .expect("decision");
    let RescanDecision::AdminSetup(reason) = decision else {
        panic!("{decision:?}");
    };
    assert!(reason.contains(r"C:\ProgramData\NewVendor"), "{reason}");
}

#[test]
fn a_failed_scan_is_an_error_not_a_clean_result() {
    let result = rescan_decision(Some(stamp("1.0.0")), "2.0.0", |_| {
        Err(anyhow::anyhow!("no group"))
    });
    assert!(result.is_err());
}

#[test]
fn stamp_round_trips_and_is_removed() {
    let home = tempfile::tempdir().expect("home");
    assert_eq!(read_shared_folder_scan_stamp(home.path()), None);
    write_shared_folder_scan_stamp(home.path(), &stamp("1.0.0")).expect("write");
    assert_eq!(
        read_shared_folder_scan_stamp(home.path()),
        Some(stamp("1.0.0"))
    );
    remove_shared_folder_scan_stamp(home.path()).expect("remove");
    assert_eq!(read_shared_folder_scan_stamp(home.path()), None);
    remove_shared_folder_scan_stamp(home.path()).expect("already gone");

    std::fs::write(stamp_path(home.path()), "not json").expect("write");
    assert_eq!(read_shared_folder_scan_stamp(home.path()), None);
}
