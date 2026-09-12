use super::*;
use pretty_assertions::assert_eq;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;

fn image() -> Vec<u8> {
    let mut b = vec![0; 96];
    b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    b[16] = 2;
    b[18] = 62;
    b[20] = 1;
    b
}

fn document() -> String {
    serde_json::json!({"schema_version":1,"purpose":"synthetic-identity-preparation",
        "source_commit":"a".repeat(40),"probe_sha256":files::image_digest(&mut image().as_slice(),IMAGE_LIMIT).unwrap(),
        "principals":{"journal":{"uid":2001,"gid":3001},"policy":{"uid":2002,"gid":3002},"worker":{"uid":2003,"gid":3003}},"anchor_gid":4000}).to_string()
}

fn fixture() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    fs::set_permissions(t.path(), fs::Permissions::from_mode(0o700)).unwrap();
    for (p, mode) in [
        ("etc".to_owned(), 0o755),
        ("opt".to_owned(), 0o755),
        ("etc/corbanu-protected-test".to_owned(), 0o700),
        ("opt/corbanu-protected-test".to_owned(), 0o755),
        (
            format!("opt/corbanu-protected-test/{}", "a".repeat(40)),
            0o755,
        ),
    ] {
        fs::create_dir(t.path().join(&p)).unwrap();
        fs::set_permissions(t.path().join(p), fs::Permissions::from_mode(mode)).unwrap();
    }
    let manifest = t.path().join("etc/corbanu-protected-test/launch.json");
    fs::write(&manifest, document()).unwrap();
    fs::set_permissions(manifest, fs::Permissions::from_mode(0o600)).unwrap();
    let binary = t.path().join(image_path());
    fs::write(&binary, image()).unwrap();
    fs::set_permissions(binary, fs::Permissions::from_mode(0o755)).unwrap();
    t
}

fn image_path() -> String {
    format!(
        "opt/corbanu-protected-test/{}/codex-protected-root-probe",
        "a".repeat(40)
    )
}
fn inspect(t: &tempfile::TempDir) -> io::Result<SyntheticManifestInspection> {
    let root = File::open(t.path())?;
    let m = root.metadata()?;
    inspect_at(&root, (m.uid(), m.gid()))
}

#[test]
fn pf_27_s01_manifest_strict_schema_and_role_table() {
    let valid = document();
    assert!(Manifest::parse(valid.as_bytes()).is_ok());
    let value: serde_json::Value = serde_json::from_str(&valid).unwrap();
    for pointer in [
        "",
        "/principals",
        "/principals/journal",
        "/principals/policy",
        "/principals/worker",
    ] {
        let keys: Vec<_> = value
            .pointer(pointer)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        for key in keys {
            let mut v = value.clone();
            v.pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(&key);
            assert!(
                Manifest::parse(v.to_string().as_bytes()).is_err(),
                "missing {pointer}/{key}"
            );
        }
        let mut v = value.clone();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), 0.into());
        assert!(Manifest::parse(v.to_string().as_bytes()).is_err());
    }
    for (needle, replacement) in [
        (
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
        ),
        ("\"journal\":{", "\"journal\":{},\"journal\":{"),
        ("\"uid\":2001", "\"uid\":2001,\"uid\":2001"),
        ("\"uid\":2002", "\"uid\":2002,\"uid\":2002"),
        ("\"uid\":2003", "\"uid\":2003,\"uid\":2003"),
    ] {
        assert!(valid.contains(needle));
        assert!(Manifest::parse(valid.replace(needle, replacement).as_bytes()).is_err());
    }
    for (pointer, bad) in [
        ("/schema_version", serde_json::json!(2)),
        ("/purpose", serde_json::json!("native")),
        ("/source_commit", serde_json::json!("../escape")),
        ("/source_commit", serde_json::json!("A".repeat(40))),
        ("/probe_sha256", serde_json::json!("z".repeat(64))),
        ("/anchor_gid", serde_json::json!(3001)),
        ("/principals/policy/uid", serde_json::json!(2001)),
        ("/principals/worker/gid", serde_json::json!(3002)),
    ] {
        let mut v = value.clone();
        *v.pointer_mut(pointer).unwrap() = bad;
        assert!(Manifest::parse(v.to_string().as_bytes()).is_err());
    }
    for bad in [
        serde_json::json!(0),
        serde_json::json!(4294967295u64),
        serde_json::json!(4294967296u64),
        serde_json::json!(-1),
        serde_json::json!(1.5),
        serde_json::json!("2001"),
    ] {
        let mut v = value.clone();
        v["principals"]["journal"]["uid"] = bad;
        assert!(Manifest::parse(v.to_string().as_bytes()).is_err());
    }
    assert!(Manifest::parse(format!("{valid} false").as_bytes()).is_err());
    assert!(Manifest::parse(&vec![b' '; 8193]).is_err());
}

#[test]
fn pf_27_s01_manifest_tree_positive_and_leaf_mutations() {
    let t = fixture();
    assert_eq!(inspect(&t).err().map(|e| e.to_string()), None);
    let path = t.path().join(image_path());
    let mut corrupt = image();
    corrupt[80] ^= 1;
    fs::write(&path, &corrupt).unwrap();
    assert_eq!(
        inspect(&t).err().unwrap().to_string(),
        "image digest mismatch"
    );
    fs::write(&path, []).unwrap();
    assert!(inspect(&t).is_err());
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(IMAGE_LIMIT + 1)
        .unwrap();
    assert!(inspect(&t).is_err());
    fs::write(&path, image()).unwrap();
    fs::hard_link(&path, t.path().join("alias")).unwrap();
    assert!(inspect(&t).is_err());
    fs::remove_file(t.path().join("alias")).unwrap();
    let manifest = t.path().join("etc/corbanu-protected-test/launch.json");
    fs::write(&manifest, vec![b' '; 8193]).unwrap();
    assert!(inspect(&t).is_err());
    fs::write(&manifest, document()).unwrap();
    fs::hard_link(&manifest, t.path().join("manifest-alias")).unwrap();
    assert!(inspect(&t).is_err());
}

#[test]
fn pf_27_s01_manifest_every_component_denies_links_modes_and_missing() {
    for relative in [
        "etc".to_owned(),
        "etc/corbanu-protected-test".to_owned(),
        "etc/corbanu-protected-test/launch.json".to_owned(),
        "opt".to_owned(),
        "opt/corbanu-protected-test".to_owned(),
        format!("opt/corbanu-protected-test/{}", "a".repeat(40)),
        image_path(),
    ] {
        let t = fixture();
        assert_eq!(inspect(&t).err().map(|e| e.to_string()), None);
        let p = t.path().join(&relative);
        let saved = t.path().join("saved");
        let mode = fs::metadata(&p).unwrap().permissions();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(inspect(&t).is_err());
        fs::set_permissions(&p, mode).unwrap();
        fs::rename(&p, &saved).unwrap();
        assert!(inspect(&t).is_err());
        assert!(!p.exists());
        symlink(&saved, &p).unwrap();
        assert!(inspect(&t).is_err());
        fs::remove_file(&p).unwrap();
        fs::write(&p, b"wrong type").unwrap();
        assert!(inspect(&t).is_err());
    }
}

#[test]
fn pf_27_s01_manifest_leaf_fifo_socket_and_identity_guard() {
    for relative in [
        "etc/corbanu-protected-test/launch.json".to_owned(),
        image_path(),
    ] {
        let t = fixture();
        let p = t.path().join(relative);
        fs::remove_file(&p).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&p)
                .status()
                .expect("mkfifo required for FIFO control")
                .success()
        );
        assert!(inspect(&t).is_err());
        fs::remove_file(&p).unwrap();
        let short_socket = t.path().join("socket");
        let _socket = std::os::unix::net::UnixListener::bind(&short_socket).unwrap();
        fs::rename(short_socket, &p).unwrap();
        assert!(inspect(&t).is_err());
    }
    assert!(
        !nix::unistd::geteuid().is_root(),
        "run construction tests unprivileged"
    );
    assert_eq!(
        SyntheticManifestInspection::inspect_system()
            .err()
            .unwrap()
            .to_string(),
        "root identity required"
    );
}
