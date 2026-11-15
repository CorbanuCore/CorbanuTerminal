use super::*;
use pretty_assertions::assert_eq;
use windows_sys::Win32::Storage::FileSystem::GetVolumeNameForVolumeMountPointW;
use windows_sys::Win32::Storage::FileSystem::GetVolumePathNameW;

/// Mount the fixture's own volume under a fresh directory, without changing
/// any drive letters. Only the fixture's payload is accessed through the mount.
struct Mounted {
    temp: tempfile::TempDir,
    link: PathBuf,
    root: PathBuf,
    payload: PathBuf,
}

impl Mounted {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("fixture");
        let payload = temp.path().join("payload");
        std::fs::create_dir(&payload).expect("payload");
        let mut volume_path = [0_u16; 32768];
        let mut volume_name = [0_u16; 64];
        // SAFETY: both output buffers are writable and their lengths are supplied.
        unsafe {
            assert_ne!(
                GetVolumePathNameW(
                    crate::winutil::to_wide(&payload).as_ptr(),
                    volume_path.as_mut_ptr(),
                    volume_path.len() as u32,
                ),
                0,
                "volume path: {}",
                std::io::Error::last_os_error()
            );
            assert_ne!(
                GetVolumeNameForVolumeMountPointW(
                    volume_path.as_ptr(),
                    volume_name.as_mut_ptr(),
                    volume_name.len() as u32,
                ),
                0,
                "volume GUID: {}",
                std::io::Error::last_os_error()
            );
        }
        let text = |wide: &[u16]| {
            String::from_utf16(&wide[..wide.iter().position(|c| *c == 0).expect("NUL")])
                .expect("Unicode volume name")
        };
        let link = temp.path().join("mount");
        junction(&link, Path::new(&text(&volume_name)));
        let root = link.join(payload.strip_prefix(text(&volume_path)).expect("on volume"));
        assert!(root.is_dir(), "mount reaches the fixture");
        Self {
            temp,
            link,
            root,
            payload,
        }
    }
}

impl Drop for Mounted {
    fn drop(&mut self) {
        // Remove only the reparse point, before TempDir cleans the fixture.
        std::fs::remove_dir(&self.link).expect("remove fixture mount");
    }
}

#[test]
fn mounted_volume_contents_are_stable_and_real_edits_are_flagged() {
    let fixture = Mounted::new();
    for name in [".codex", ".agents"] {
        let relative = PathBuf::from("project").join(name);
        let folder = fixture.payload.join(&relative);
        std::fs::create_dir_all(&folder).expect("metadata");
        std::fs::write(folder.join("instructions.md"), "original").expect("write");
        let roots = [fixture.root.clone()];
        let found = find(&roots);
        assert!(found.unlistable.is_empty());
        let before = with_contents(found.entries);
        assert_eq!(before.len(), 1);
        let prints = before[0].contents.as_ref().expect("contents recorded");
        assert_eq!(prints.len(), 1);
        assert!(
            prints.contains_key("instructions.md"),
            "must actually read the file"
        );
        assert_eq!(
            appeared(&before, find(&roots).entries),
            Vec::<PathBuf>::new()
        );
        assert_eq!(
            appeared(&before, find(&roots).entries),
            Vec::<PathBuf>::new()
        );
        std::fs::write(folder.join("instructions.md"), "changed").expect("edit");
        assert_eq!(
            appeared(&before, find(&roots).entries),
            vec![fixture.root.join(&relative)]
        );
        std::fs::remove_dir_all(folder).expect("remove metadata for next case");
    }
}

#[test]
fn mounted_volume_replaced_metadata_and_wrong_volume_are_flagged() {
    let fixture = Mounted::new();
    let folder = fixture.payload.join("project\\.codex");
    std::fs::create_dir_all(&folder).expect("metadata");
    std::fs::write(folder.join("instructions.md"), "same bytes").expect("write");
    let roots = [fixture.root.clone()];
    let before = with_contents(find(&roots).entries);
    let path = fixture.root.join("project\\.codex");
    let expected = before[0].identity.expect("identity");
    let wrong_volume = FileIdentity {
        volume: expected.volume ^ 1,
        ..expected
    };
    let prints = contents_of(&path, Some(wrong_volume), &ContentBudget::default());
    assert!(
        prints.contains_key(""),
        "matching file ID on another volume is refused"
    );
    assert!(!prints.contains_key("instructions.md"));
    std::fs::rename(&folder, folder.with_extension("aside")).expect("move old metadata");
    std::fs::create_dir(&folder).expect("replace");
    std::fs::write(folder.join("instructions.md"), "same bytes").expect("same contents");
    assert_eq!(appeared(&before, find(&roots).entries), vec![path]);
}

#[test]
fn mounted_volume_swapped_for_junction_stays_conservative() {
    let fixture = Mounted::new();
    let folder = fixture.payload.join("project\\.agents");
    std::fs::create_dir_all(&folder).expect("metadata");
    std::fs::write(folder.join("instructions.md"), "original").expect("write");
    let path = fixture.root.join("project\\.agents");
    let expected = identity(&path);
    assert!(
        contents(&path).contains_key("instructions.md"),
        "mount read works first"
    );
    std::fs::remove_dir(&fixture.link).expect("remove mount");
    // A normal junction back to the same volume reaches the exact same
    // metadata identity, but must still be refused before listing.
    let drive = fixture.temp.path().ancestors().last().expect("drive root");
    junction(&fixture.link, drive);
    assert_eq!(
        identity(&path),
        expected,
        "same target, different link kind"
    );
    let prints = contents_of(&path, expected, &ContentBudget::default());
    assert!(prints.contains_key(""));
    assert!(!prints.contains_key("instructions.md"));
}
