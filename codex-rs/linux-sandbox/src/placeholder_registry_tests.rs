use super::*;
use pretty_assertions::assert_eq;

#[test]
fn a_marker_names_its_pid_namespace() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let marker = temp_dir.path().join(std::process::id().to_string());
    write_marker(&marker, b"synthetic\n").expect("marker");
    assert_eq!(
        fs::read_to_string(pid_namespace_file(&marker))
            .expect("namespace file")
            .parse::<u64>()
            .ok(),
        own_pid_namespace()
    );
    assert!(marker_run_is_active(
        &marker,
        std::process::id() as libc::pid_t
    ));
    remove_marker(&marker).expect("remove");
    assert!(!marker.exists() && !pid_namespace_file(&marker).exists());
}

#[test]
fn a_run_in_another_pid_namespace_is_taken_as_running() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let mut child = std::process::Command::new("true").spawn().expect("spawn");
    child.wait().expect("wait");
    let pid = child.id() as libc::pid_t;
    let marker = temp_dir.path().join(pid.to_string());
    fs::write(&marker, b"synthetic\n").expect("marker");
    // An older run's marker names no namespace: as before.
    assert!(!marker_run_is_active(&marker, pid));
    fs::write(pid_namespace_file(&marker), "1").expect("another namespace");
    assert!(marker_run_is_active(&marker, pid));
    fs::write(
        pid_namespace_file(&marker),
        own_pid_namespace().expect("namespace").to_string(),
    )
    .expect("this namespace");
    assert!(!marker_run_is_active(&marker, pid));
}

#[test]
fn removal_does_not_follow_a_link_above_the_placeholder() {
    let temp_dir = tempfile::TempDir::new().expect("tempdir");
    let base = temp_dir.path().canonicalize().expect("canonical");
    let folder = base.join("folder");
    let elsewhere = base.join("elsewhere");
    fs::create_dir_all(&folder).expect("folder");
    fs::create_dir_all(&elsewhere).expect("elsewhere");
    fs::write(folder.join("x"), "").expect("placeholder");
    let found = find_without_links(&folder.join("x"))
        .expect("find")
        .expect("found");
    assert!(found.is_empty_file());

    fs::rename(&folder, base.join("moved")).expect("move the folder");
    std::os::unix::fs::symlink(&elsewhere, &folder).expect("a link in its place");
    fs::write(elsewhere.join("x"), "").expect("someone else's empty file");
    assert!(
        find_without_links(&folder.join("x"))
            .expect("find")
            .is_none()
    );
    assert!(
        find_without_links(&base.join("missing"))
            .expect("find")
            .is_none()
    );
}

#[test]
fn a_placeholder_is_one_made_for_its_record() {
    let recorded_at = SystemTime::now();
    assert!(made_for_record(recorded_at, recorded_at));
    assert!(made_for_record(
        recorded_at + Duration::from_secs(1),
        recorded_at
    ));
    // A file server's clock may run behind.
    assert!(made_for_record(
        recorded_at - Duration::from_secs(3600),
        recorded_at
    ));
    assert!(made_for_record(
        recorded_at + Duration::from_secs(300),
        recorded_at
    ));
    assert!(!made_for_record(
        recorded_at + Duration::from_secs(3600),
        recorded_at
    ));
}
