use super::*;
use pretty_assertions::assert_eq;
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;

fn bytes() -> Vec<u8> {
    let mut b = vec![0; 96];
    b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    b[16] = 2;
    b[18] = 62;
    b[20] = 1;
    b
}

fn inspected() -> SyntheticManifestInspection {
    let mut file = tempfile::tempfile().unwrap();
    file.write_all(&bytes()).unwrap();
    SyntheticManifestInspection {
        _stamp: files::stamp(&file).unwrap(),
        _image: file,
        _recipe: SyntheticLaunchRecipe::new(
            "/unused".into(),
            [(101, 201), (102, 202), (103, 203)],
            204,
        )
        .unwrap(),
        _digest: files::image_digest(&mut bytes().as_slice(), IMAGE_LIMIT).unwrap(),
    }
}

fn identity(file: &File) -> (i32, u64, u64) {
    let m = file.metadata().unwrap();
    (file.as_raw_fd(), m.dev(), m.ino())
}
fn assert_closed((fd, dev, ino): (i32, u64, u64)) {
    // A concurrent test may reuse the number, but cannot retain this inode.
    if let Ok(m) = std::fs::metadata(format!("/proc/self/fd/{fd}")) {
        assert_ne!((m.dev(), m.ino()), (dev, ino));
    }
}

#[test]
fn pf_27_s01_sealed_image_kernel_denials_and_drop() {
    assert!(
        !nix::unistd::geteuid().is_root(),
        "run fixture unprivileged"
    );
    let mut pre = KernelOps.create().unwrap();
    pre.write_all(b"writable positive control").unwrap();
    pre.set_len(4).unwrap();
    let original = inspected();
    let source_id = identity(&original._image);
    let mut sealed = original.seal().unwrap();
    assert_closed(source_id);
    assert!(
        fcntl_get_seals(&sealed._image)
            .unwrap()
            .contains(required_seals())
    );
    assert!(
        rustix::io::fcntl_getfd(&sealed._image)
            .unwrap()
            .contains(rustix::io::FdFlags::CLOEXEC)
    );
    let target_id = identity(&sealed._image);
    sealed._image.rewind().unwrap();
    let mut actual = Vec::new();
    sealed._image.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, bytes());
    for error in [
        sealed._image.write_all(b"x").unwrap_err(),
        sealed._image.set_len(1).unwrap_err(),
        sealed._image.set_len(1000).unwrap_err(),
    ] {
        assert_eq!(error.raw_os_error(), Some(nix::libc::EPERM));
    }
    assert_eq!(
        fcntl_add_seals(&sealed._image, SealFlags::FUTURE_WRITE),
        Err(rustix::io::Errno::PERM)
    );
    drop(sealed);
    assert_closed(target_id);
}

struct Faults {
    fail: usize,
    step: usize,
    created: Option<(i32, u64, u64)>,
    corrupt: bool,
    incomplete: bool,
}
impl Faults {
    fn new(fail: usize) -> Self {
        Self {
            fail,
            step: 0,
            created: None,
            corrupt: false,
            incomplete: false,
        }
    }
    fn next(&mut self) -> io::Result<()> {
        self.step += 1;
        if self.step == self.fail {
            Err(io::Error::other("injected boundary"))
        } else {
            Ok(())
        }
    }
}
impl Operations for Faults {
    fn source_stamp(&mut self, f: &File) -> io::Result<files::Stamp> {
        self.next()?;
        KernelOps.source_stamp(f)
    }
    fn create(&mut self) -> io::Result<File> {
        self.next()?;
        let f = KernelOps.create()?;
        self.created = Some(identity(&f));
        Ok(f)
    }
    fn rewind(&mut self, f: &mut File) -> io::Result<()> {
        self.next()?;
        KernelOps.rewind(f)
    }
    fn copy(&mut self, s: &mut File, d: &mut File) -> io::Result<()> {
        self.next()?;
        KernelOps.copy(s, d)?;
        if self.corrupt {
            d.write_all(b"corruption")?;
        }
        Ok(())
    }
    fn seal(&mut self, f: &File) -> io::Result<SealFlags> {
        self.next()?;
        if self.incomplete {
            return Ok(SealFlags::GROW);
        }
        KernelOps.seal(f)
    }
    fn digest(&mut self, f: &mut File) -> io::Result<String> {
        self.next()?;
        KernelOps.digest(f)
    }
}

#[test]
fn pf_27_s01_sealed_image_every_failure_drops_both_handles() {
    for failure in 1..=8 {
        let source = inspected();
        let source_id = identity(&source._image);
        let mut ops = Faults::new(failure);
        assert!(prepare(source, &mut ops).is_err());
        assert_eq!(ops.step, failure);
        assert_closed(source_id);
        if let Some(created) = ops.created {
            assert_closed(created);
        }
    }
    let mut incomplete = Faults::new(0);
    incomplete.incomplete = true;
    assert_eq!(
        prepare(inspected(), &mut incomplete)
            .err()
            .unwrap()
            .to_string(),
        "image seals incomplete"
    );
    assert_closed(incomplete.created.unwrap());
    let mut corrupt = Faults::new(0);
    corrupt.corrupt = true;
    assert_eq!(
        prepare(inspected(), &mut corrupt)
            .err()
            .unwrap()
            .to_string(),
        "sealed image digest mismatch"
    );
    assert_closed(corrupt.created.unwrap());
}

#[test]
fn pf_27_s01_sealed_image_stale_source_and_invalid_bytes_deny() {
    let mut source = inspected();
    source._image.write_all(b"changed").unwrap();
    assert!(source.seal().is_err());
    let mut source = inspected();
    source._digest = "0".repeat(64);
    assert!(source.seal().is_err());
    for bad in [Vec::new(), b"short".to_vec(), vec![0; 96]] {
        let mut source = inspected();
        source._image.set_len(0).unwrap();
        source._image.rewind().unwrap();
        source._image.write_all(&bad).unwrap();
        source._stamp = files::stamp(&source._image).unwrap();
        assert!(source.seal().is_err());
    }
}

#[test]
fn pf_27_s01_sealed_copy_bounds_and_partial_io_errors() {
    let mut output = Vec::new();
    copy_bounded(&mut bytes().as_slice(), &mut output, 96).unwrap();
    assert_eq!(output, bytes());
    assert!(copy_bounded(&mut bytes().as_slice(), &mut Vec::new(), 95).is_err());
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("secret injected detail"))
        }
    }
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("secret injected detail"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let data = bytes();
    let mut partial = data.as_slice().chain(Broken);
    assert_eq!(
        copy_bounded(&mut partial, &mut Vec::new(), 200)
            .unwrap_err()
            .to_string(),
        "image copy read failed"
    );
    assert_eq!(
        copy_bounded(&mut bytes().as_slice(), &mut Broken, 200)
            .unwrap_err()
            .to_string(),
        "image copy write failed"
    );
    struct Partial {
        calls: usize,
    }
    impl Write for Partial {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            if self.calls == 1 {
                Ok(b.len().min(1))
            } else {
                Err(io::Error::other("private partial write"))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Partial { calls: 0 };
    assert_eq!(
        copy_bounded(&mut data.as_slice(), &mut writer, 200)
            .unwrap_err()
            .to_string(),
        "image copy write failed"
    );
    assert_eq!(writer.calls, 2);
}
