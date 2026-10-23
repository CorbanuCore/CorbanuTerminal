use pretty_assertions::assert_eq;

use super::ApprovalsAtConfirmation;
use super::confirm;
use super::confirmed_here;

/// #505 (review r1, #619): a home confirmed under one spelling is the same
/// home under another (a link to it, as `/tmp` is to `/private/tmp`).
#[test]
fn issue_505_a_confirmed_home_is_found_by_any_spelling() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path().join("home");
    std::fs::create_dir(&home).expect("home");
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&home, &alias).expect("link");
    assert!(!confirmed_here(&home));

    let report = confirm(&alias);

    assert_eq!(report, ApprovalsAtConfirmation::default(), "no sessions");
    assert!(confirmed_here(&home));
    assert!(confirmed_here(&alias));
}
