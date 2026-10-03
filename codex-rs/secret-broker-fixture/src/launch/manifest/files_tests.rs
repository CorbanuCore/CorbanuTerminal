use super::*;
use pretty_assertions::assert_eq;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn pf_27_s01_file_metadata_and_stability_guards() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("file");
    fs::write(&p, b"fixture").unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
    let f = File::open(&p).unwrap();
    let m = f.metadata().unwrap();
    let owner = (m.uid(), m.gid());
    assert!(check(&f, Kind::Regular(0o600), owner).is_ok());
    assert!(check(&f, Kind::Regular(0o600), (owner.0 ^ 1, owner.1)).is_err());
    assert!(check(&f, Kind::Regular(0o600), (owner.0, owner.1 ^ 1)).is_err());
    let dir = File::open(t.path()).unwrap();
    assert!(check(&dir, Kind::Ancestor, (owner.0 ^ 1, owner.1)).is_err());
    let before = stamp(&f).unwrap();
    assert!(unchanged(before, before).is_ok());
    for i in 0..before.len() {
        let mut after = before;
        after[i] ^= 1;
        assert!(unchanged(before, after).is_err());
    }
    fs::write(&p, b"changed length").unwrap();
    assert!(unchanged(before, stamp(&f).unwrap()).is_err());
    fs::set_permissions(&p, fs::Permissions::from_mode(0o4600)).unwrap();
    assert!(check(&f, Kind::Regular(0o600), owner).is_err());
    let device = File::open("/dev/null").unwrap();
    assert!(check(&device, Kind::Regular(0o600), owner).is_err());
    assert!(open(&dir, "../file", Kind::Regular(0o600), owner).is_err());
}

#[test]
fn pf_27_s01_digest_bounds_identity_and_read_failures() {
    let mut header = [0; 64];
    header[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    header[16] = 2;
    header[18] = 62;
    header[20] = 1;
    assert_eq!(
        image_digest(&mut header.as_slice(), 64).unwrap(),
        format!("{:x}", Sha256::digest(header))
    );
    for i in [0, 4, 5, 6, 16, 18, 20] {
        let mut bad = header;
        bad[i] = 0;
        assert!(image_digest(&mut bad.as_slice(), 64).is_err());
    }
    assert!(image_digest(&mut &header[..63], 64).is_err());
    assert!(image_digest(&mut header.as_slice(), 63).is_err());
    let mut too_big = header.to_vec();
    too_big.push(1);
    assert!(image_digest(&mut too_big.as_slice(), 64).is_err());
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("private injected detail"))
        }
    }
    assert_eq!(
        image_digest(&mut Broken, 64).unwrap_err().to_string(),
        "image header unavailable"
    );
    let mut later = header.as_slice().chain(Broken);
    assert_eq!(
        image_digest(&mut later, 128).unwrap_err().to_string(),
        "image read failed"
    );
}
