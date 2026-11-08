use super::*;
use pretty_assertions::assert_eq;

#[test]
fn rescan_is_due_until_recorded_for_this_version() {
    let home = tempfile::tempdir().expect("home");
    assert!(rescan_due(home.path()), "no stamp yet");

    record_rescan(home.path()).expect("record");
    assert!(!rescan_due(home.path()));
    let stamp: ScanStamp =
        serde_json::from_slice(&std::fs::read(stamp_path(home.path())).expect("read"))
            .expect("parse");
    assert_eq!(stamp.cli_version, cli_version());
}

#[test]
fn rescan_is_due_after_the_version_changes_or_the_stamp_is_damaged() {
    let home = tempfile::tempdir().expect("home");
    std::fs::create_dir_all(crate::setup::sandbox_dir(home.path())).expect("sandbox dir");

    std::fs::write(stamp_path(home.path()), r#"{"cli_version":"0.0.1-old"}"#).expect("write");
    assert!(rescan_due(home.path()));

    std::fs::write(stamp_path(home.path()), "not json").expect("write");
    assert!(rescan_due(home.path()));
}
