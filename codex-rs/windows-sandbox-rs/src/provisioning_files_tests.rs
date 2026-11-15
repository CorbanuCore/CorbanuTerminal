use super::open_provisioning_entry;
use crate::DirectoryOpenDisposition;
use crate::open_directory_no_reparse;
use pretty_assertions::assert_eq;
use std::fs;
use std::os::windows::io::AsHandle;
use windows_sys::Win32::Storage::FileSystem as winfs;

#[test]
fn issue_2315_leaf_open_uses_parent_handle_after_parent_rename() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join(".sandbox");
    let moved = temp.path().join("moved");
    fs::create_dir(&original).unwrap();
    fs::write(original.join("leaf"), b"original").unwrap();
    let parent = open_directory_no_reparse(
        &original,
        winfs::FILE_READ_ATTRIBUTES,
        winfs::FILE_SHARE_READ | winfs::FILE_SHARE_WRITE | winfs::FILE_SHARE_DELETE,
        DirectoryOpenDisposition::OpenExisting,
    )
    .unwrap();
    fs::rename(&original, &moved).unwrap();
    fs::create_dir(&original).unwrap();
    fs::write(original.join("leaf"), b"replacement with different size").unwrap();
    let file =
        fs::File::from(open_provisioning_entry(parent.as_handle(), "leaf".as_ref()).unwrap());
    assert_eq!(file.metadata().unwrap().len(), 8);
}

#[test]
fn issue_2315_leaf_open_refuses_links_and_ambiguous_names() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join(".sandbox");
    fs::create_dir(&original).unwrap();
    let parent = open_directory_no_reparse(
        &original,
        winfs::FILE_READ_ATTRIBUTES,
        winfs::FILE_SHARE_READ | winfs::FILE_SHARE_WRITE,
        DirectoryOpenDisposition::OpenExisting,
    )
    .unwrap();
    let outside = temp.path().join("outside");
    fs::write(&outside, b"unchanged").unwrap();
    fs::hard_link(&outside, original.join("linked")).unwrap();
    for name in [
        "linked",
        "../outside",
        "leaf:stream",
        "",
        ".",
        "..",
        r"sub\leaf",
    ] {
        assert!(
            open_provisioning_entry(parent.as_handle(), name.as_ref()).is_err(),
            "{name}"
        );
    }
    assert_eq!(fs::read(outside).unwrap(), b"unchanged");
}

#[test]
fn issue_2315_leaf_open_refuses_junction() {
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join(".sandbox");
    let outside = temp.path().join("outside");
    fs::create_dir(&original).unwrap();
    fs::create_dir(&outside).unwrap();
    let link = original.join("junction");
    let output = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&outside)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let parent = open_directory_no_reparse(
        &original,
        winfs::FILE_READ_ATTRIBUTES,
        winfs::FILE_SHARE_READ | winfs::FILE_SHARE_WRITE,
        DirectoryOpenDisposition::OpenExisting,
    )
    .unwrap();
    assert!(open_provisioning_entry(parent.as_handle(), "junction".as_ref()).is_err());
    fs::remove_dir(link).unwrap();
}
