use super::open_at;
use pretty_assertions::assert_eq;
use std::os::fd::AsRawFd;

/// #536: the folder is opened at its pinned location, and a spelling that
/// leads elsewhere now (a link swapped in) is refused, naming the link.
#[test]
fn open_at_checks_every_spelling_and_the_opened_folder() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().canonicalize().expect("canonical");
    let target = base.join("target");
    let elsewhere = base.join("elsewhere");
    std::fs::create_dir_all(&target).expect("target");
    std::fs::create_dir_all(&elsewhere).expect("elsewhere");

    let file = open_at(&[&target], &target).expect("opens the pinned folder");
    assert_eq!(
        std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd())).expect("fd"),
        target
    );

    std::fs::rename(&target, base.join("moved")).expect("move away");
    std::os::unix::fs::symlink(&elsewhere, &target).expect("link");
    let err = open_at(&[&target], &target).expect_err("refused");
    let message = err.to_string();
    assert!(
        message.contains("Refused to run the command")
            && message.contains(&format!("`!rm '{}'`", target.display())),
        "{message}"
    );
}
