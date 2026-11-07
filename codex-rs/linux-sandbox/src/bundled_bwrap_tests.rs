//! Verify bundled bubblewrap discovery and executable digest validation.

use super::*;
use codex_install_context::CodexPackageLayout;
use codex_install_context::InstallMethod;
use pretty_assertions::assert_eq;
use std::fs;
use tempfile::NamedTempFile;
use tempfile::tempdir;

#[test]
fn finds_package_layout_bwrap_from_install_context() {
    let temp_dir = tempdir().expect("temp dir");
    let package_dir = temp_dir.path();
    let bin_dir = package_dir.join("bin");
    let resources_dir = package_dir.join("codex-resources");
    let expected_bwrap = resources_dir.join("bwrap");
    fs::create_dir_all(&bin_dir).expect("create bin dir");
    write_executable(&expected_bwrap);

    let context = InstallContext {
        method: InstallMethod::Other,
        package_layout: Some(CodexPackageLayout {
            package_dir: AbsolutePathBuf::from_absolute_path(package_dir).expect("absolute"),
            bin_dir: AbsolutePathBuf::from_absolute_path(&bin_dir).expect("absolute"),
            resources_dir: Some(
                AbsolutePathBuf::from_absolute_path(&resources_dir).expect("absolute"),
            ),
            path_dir: None,
        }),
    };

    assert_eq!(
        find_for_install_context(&context),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn finds_legacy_standalone_bundled_bwrap_next_to_exe_resources() {
    let temp_dir = tempdir().expect("temp dir");
    let exe = temp_dir.path().join("codex");
    let expected_bwrap = temp_dir.path().join("codex-resources").join("bwrap");
    write_executable(&exe);
    write_executable(&expected_bwrap);

    assert_eq!(
        find_legacy_for_exe(&exe, /*bazel_candidate*/ None),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn finds_npm_bundled_bwrap_next_to_target_vendor_dir() {
    let temp_dir = tempdir().expect("temp dir");
    let target_dir = temp_dir.path().join("vendor/x86_64-unknown-linux-musl");
    let exe = target_dir.join("codex").join("codex");
    let expected_bwrap = target_dir.join("codex-resources").join("bwrap");
    write_executable(&exe);
    write_executable(&expected_bwrap);

    assert_eq!(
        find_legacy_for_exe(&exe, /*bazel_candidate*/ None),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn finds_adjacent_dev_bwrap() {
    let temp_dir = tempdir().expect("temp dir");
    let exe = temp_dir.path().join("codex");
    let expected_bwrap = temp_dir.path().join("bwrap");
    write_executable(&exe);
    write_executable(&expected_bwrap);

    assert_eq!(
        find_legacy_for_exe(&exe, /*bazel_candidate*/ None),
        Some(AbsolutePathBuf::from_absolute_path(&expected_bwrap).expect("absolute"))
    );
}

#[test]
fn digest_verification_skips_missing_expected_digest() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), b"contents").expect("write file");

    verify_digest(file.as_file(), /*expected*/ None, file.path())
        .expect("missing digest should skip verification");
}

#[test]
fn digest_verification_accepts_matching_digest() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), b"contents").expect("write file");
    let expected: [u8; 32] = Sha256::digest(b"contents").into();

    verify_digest(file.as_file(), Some(expected), file.path())
        .expect("matching digest should verify");
}

#[test]
fn digest_verification_rejects_mismatched_digest() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), b"contents").expect("write file");

    let err = verify_digest(file.as_file(), Some([0xab; 32]), file.path())
        .expect_err("mismatched digest should fail");
    assert!(err.contains("bundled bubblewrap digest mismatch"));
}

/// #581: a copy whose digest does not match is never probed (so never
/// run); one that matches is probed through its descriptor.
#[test]
fn a_copy_that_fails_verification_is_never_probed() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), b"contents").expect("write file");
    let program = AbsolutePathBuf::from_absolute_path(file.path()).expect("absolute");
    let mut probed = false;
    let launcher = apparmor_launcher_with(&program, Some([0xab; 32]), |_| {
        probed = true;
        None
    });
    assert_eq!(launcher, None);
    assert!(!probed);
    let expected: [u8; 32] = Sha256::digest(b"contents").into();
    let mut probed_fd = None;
    apparmor_launcher_with(&program, Some(expected), |fd| {
        probed_fd = Some(fd);
        None
    });
    assert!(probed_fd.is_some_and(|fd| fd > libc::STDERR_FILENO));
}

/// The digest is read by offset, so verifying a descriptor twice gives
/// the same answer (#581 verifies again right before `aa-exec`).
#[test]
fn digest_verification_can_repeat_on_one_descriptor() {
    let file = NamedTempFile::new().expect("temp file");
    fs::write(file.path(), b"contents").expect("write file");
    let expected: [u8; 32] = Sha256::digest(b"contents").into();
    for _ in 0..2 {
        verify_digest(file.as_file(), Some(expected), file.path()).expect("verifies");
    }
}

#[test]
fn parses_sha256_hex_digest() {
    assert_eq!(parse_sha256_hex(&"ab".repeat(32)), Ok([0xab; 32]));
    assert_eq!(parse_sha256_hex(&"00".repeat(32)), Ok(NULL_SHA256_DIGEST));
    assert!(parse_sha256_hex("ab").is_err());
    assert!(parse_sha256_hex(&format!("{}xx", "00".repeat(31))).is_err());
}

fn write_executable(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    codex_utils_cargo_bin::write_executable(path, "").expect("write executable");
}
