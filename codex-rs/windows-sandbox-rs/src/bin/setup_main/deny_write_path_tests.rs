use super::*;
use pretty_assertions::assert_eq;
use std::fs;

fn junction(link: &Path, target: &Path) {
    let output = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .expect("mklink");
    assert!(output.status.success(), "mklink: {output:?}");
}

#[test]
fn issue_1953_deny_path_refuses_existing_and_dangling_junctions() {
    let temp = tempfile::tempdir().expect("temp");
    let target = temp.path().join("target");
    fs::create_dir(&target).expect("target");
    let link = temp.path().join("link");
    junction(&link, &target);
    assert!(pin(&link).is_err(), "the deny writer accepted a junction");
    assert!(
        pin(&link.join("missing")).is_err(),
        "created through a junction"
    );
    assert!(!target.join("missing").exists());
    fs::remove_dir(&target).expect("remove target");
    assert!(
        pin(&link).is_err(),
        "the deny writer accepted a dangling junction"
    );
    assert!(!target.exists());
    fs::remove_dir(link).expect("remove junction");
}

#[test]
fn issue_1953_deny_path_stays_pinned_until_acl_write_finishes() {
    let temp = tempfile::tempdir().expect("temp");
    let parent = temp.path().join("parent");
    let path = parent.join("missing").join("protected");
    let pins = pin(&path).expect("materialize and pin");
    assert!(path.is_dir());
    // Pins must permit the helper's ACL operation, while preventing a
    // replacement before its path-based security API opens the object.
    unsafe {
        let sid = codex_windows_sandbox::convert_string_sid_to_sid("S-1-5-21-1-2-3-999")
            .expect("test SID");
        let applied = codex_windows_sandbox::add_deny_write_ace(&path, sid);
        windows_sys::Win32::Foundation::LocalFree(sid as _);
        assert!(applied.expect("write deny while pinned"));
    }
    assert!(fs::rename(&path, parent.join("moved")).is_err());
    assert!(fs::rename(&parent, temp.path().join("swapped")).is_err());
    drop(pins);
    fs::rename(&parent, temp.path().join("swapped")).expect("pins released");
    assert!(temp.path().join("swapped/missing/protected").is_dir());
}

#[test]
fn issue_1953_existing_file_and_parent_traversal() {
    let temp = tempfile::tempdir().expect("temp");
    let file = temp.path().join("protected.txt");
    fs::write(&file, "keep").expect("file");
    let pins = pin(&file).expect("pin existing file");
    assert_eq!(fs::read_to_string(&file).expect("read"), "keep");
    drop(pins);
    let path = temp.path().join("missing/../elsewhere");
    assert!(pin(&path).is_err(), "ambiguous parent traversal");
    assert!(!temp.path().join("missing").exists());
}
