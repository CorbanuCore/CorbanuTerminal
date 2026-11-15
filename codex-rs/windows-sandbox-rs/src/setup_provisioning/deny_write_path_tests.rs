use super::*;
use pretty_assertions::assert_eq;
use std::fs;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE;
use windows_sys::Win32::Storage::FileSystem::FILE_WRITE_ATTRIBUTES;
use windows_sys::Win32::System::IO::DeviceIoControl;

fn control(file: &File, code: u32, buffer: &[u8]) {
    let mut returned = 0_u32;
    // SAFETY: a live handle and valid input/output buffers.
    let ok = unsafe {
        DeviceIoControl(
            file.as_raw_handle() as _,
            code,
            buffer.as_ptr().cast(),
            buffer.len() as u32,
            std::ptr::null_mut(),
            /*noutbuffersize*/ 0,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(
        ok,
        0,
        "reparse control: {}",
        std::io::Error::last_os_error()
    );
}

fn attribute_writer(path: &Path) -> File {
    fs::OpenOptions::new()
        .access_mode(FILE_WRITE_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .expect("attribute-only writer")
}

fn remove_tag(path: &Path, tag: u32) {
    let file = attribute_writer(path);
    let mut buffer = [0_u8; 8];
    buffer[..4].copy_from_slice(&tag.to_le_bytes());
    control(&file, /*code*/ 0x0009_00AC, &buffer);
}

fn retag_as_junction(path: &Path, target: &Path) {
    let substitute: Vec<u16> = format!(r"\??\{}", target.display())
        .encode_utf16()
        .collect();
    let print: Vec<u16> = target.as_os_str().encode_wide().collect();
    let mut names = substitute.clone();
    names.push(0);
    names.extend(&print);
    names.push(0);
    let fields = [
        0_u16,
        (substitute.len() * 2) as u16,
        ((substitute.len() + 1) * 2) as u16,
        (print.len() * 2) as u16,
    ];
    let mut buffer = Vec::new();
    buffer.extend(0xA000_0003_u32.to_le_bytes());
    buffer.extend((((fields.len() + names.len()) * 2) as u16).to_le_bytes());
    buffer.extend(0_u16.to_le_bytes());
    for word in fields.iter().chain(names.iter()) {
        buffer.extend(word.to_le_bytes());
    }
    control(&attribute_writer(path), /*code*/ 0x0009_00A4, &buffer);
}

#[test]
fn issue_1953_non_redirecting_reparse_folder_is_supported() {
    let temp = tempfile::tempdir().expect("temp");
    let path = temp.path().join("placeholder");
    fs::create_dir(&path).expect("folder");
    let mut buffer = [0_u8; 16];
    buffer[..4].copy_from_slice(&0x8000_000A_u32.to_le_bytes());
    buffer[4..6].copy_from_slice(&8_u16.to_le_bytes());
    buffer[8..].copy_from_slice(b"corbanu1");
    control(&attribute_writer(&path), /*code*/ 0x0009_00A4, &buffer);
    let result = pin(&path);
    remove_tag(&path, /*tag*/ 0x8000_000A);
    assert!(
        result.is_ok(),
        "non-redirecting reparse folder refused: {result:?}"
    );
}

#[test]
fn issue_1953_attribute_only_junction_swap_cannot_redirect_creation_or_acl() {
    for final_component in [false, true] {
        let temp = tempfile::tempdir().expect("temp");
        let root = dunce::canonicalize(temp.path()).expect("canonical");
        let target = root.join("outside");
        fs::create_dir(&target).expect("target");
        let swapped = root.join("swapped");
        fs::create_dir(&swapped).expect("swapped");
        let path = if final_component {
            swapped.clone()
        } else {
            swapped.join("protected")
        };
        let before = std::process::Command::new("icacls.exe")
            .arg(&target)
            .output()
            .expect("ACL");
        assert!(before.status.success());
        let swap = swapped.clone();
        let outside = target.clone();
        AFTER_PIN.with(|hook| {
            *hook.borrow_mut() = Some(Box::new(move |part| {
                if part == swap {
                    retag_as_junction(part, &outside);
                }
            }))
        });
        let result = pin(&path);
        AFTER_PIN.with(|hook| hook.borrow_mut().take());
        if let Ok(pins) = result.as_ref() {
            unsafe {
                let sid = crate::convert_string_sid_to_sid("S-1-5-21-1-2-3-999")
                    .expect("SID");
                let applied = crate::add_deny_write_ace_to_handle(
                    pins.last().expect("target").as_raw_handle() as _,
                    sid,
                );
                windows_sys::Win32::Foundation::LocalFree(sid as _);
                applied.expect("deny on the opened object");
            }
        }
        let after = std::process::Command::new("icacls.exe")
            .arg(&target)
            .output()
            .expect("ACL");
        assert!(after.status.success());
        assert_eq!(
            before.stdout, after.stdout,
            "the junction target ACL changed"
        );
        assert!(
            !target.join("protected").exists(),
            "created through a retagged parent"
        );
        drop(result);
        remove_tag(&swapped, /*tag*/ 0xA000_0003);
    }
}

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
        let sid = crate::convert_string_sid_to_sid("S-1-5-21-1-2-3-999")
            .expect("test SID");
        let applied = crate::add_deny_write_ace_to_handle(
            pins.last().expect("target").as_raw_handle() as _,
            sid,
        );
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
    let writer = fs::OpenOptions::new()
        .write(true)
        .open(&file)
        .expect("existing writer");
    let pins = pin(&file).expect("pin existing file with writer");
    drop(writer);
    assert_eq!(fs::read_to_string(&file).expect("read"), "keep");
    drop(pins);
    let path = temp.path().join("missing/../elsewhere");
    assert!(pin(&path).is_err(), "ambiguous parent traversal");
    assert!(!temp.path().join("missing").exists());
}
