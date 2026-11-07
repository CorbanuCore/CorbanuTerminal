//! D319: the `zip` decoder set must keep reading every compression method we
//! accept today (stored, deflate, deflate64, bzip2) and must refuse the others
//! with a clean error unless the security owner approves another decoder.
//!
//! The archives under `tests/fixtures/d319_zip/` stand in for the untrusted
//! skill and plugin bundles we extract. They were generated once before the
//! upstream sync touched `zip` and must never be regenerated (see the README
//! there). The set of accepted methods may grow only with the security owner's
//! approval; then only `REJECTED` changes, never the fixtures.
//!
//! Scope: this checks the decoders the workspace's `zip` features build, which
//! is what D319 decides. Path safety of the bundle extraction itself is tested
//! with that code in `src/startup_sync_tests.rs`.

use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use zip::CompressionMethod;
use zip::ZipArchive;
use zip::result::ZipError;

const ACCEPTED: &[(&str, CompressionMethod)] = &[
    ("stored.zip", CompressionMethod::Stored),
    ("deflate.zip", CompressionMethod::Deflated),
    ("deflate64.zip", CompressionMethod::Deflate64),
    ("bzip2.zip", CompressionMethod::Bzip2),
];

/// Methods the upstream `zip` feature set adds and we do not accept.
const REJECTED: &[(&str, u16)] = &[("rejected-lzma.zip", 14), ("rejected-zstd.zip", 93)];

fn fixtures_dir() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/d319_zip");
    if manifest.is_dir() {
        manifest
    } else {
        // Bazel runs integration tests from a Cargo-like crate directory.
        PathBuf::from("tests/fixtures/d319_zip")
    }
}

fn open_copy(name: &str) -> anyhow::Result<(tempfile::TempDir, ZipArchive<std::fs::File>)> {
    let dir = tempfile::tempdir()?;
    let copy = dir.path().join(name);
    std::fs::copy(fixtures_dir().join(name), &copy)?;
    let archive = ZipArchive::new(std::fs::File::open(&copy)?)?;
    Ok((dir, archive))
}

#[test]
fn accepted_methods_extract_the_expected_contents() -> anyhow::Result<()> {
    let expected_root = fixtures_dir().join("expected");
    for (name, method) in ACCEPTED {
        let (_dir, mut archive) = open_copy(name)?;
        let mut files = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            if entry.is_dir() {
                continue;
            }
            assert_eq!(
                entry.compression(),
                *method,
                "{name}: method of {:?}",
                entry.name()
            );
            let relative = entry
                .enclosed_name()
                .ok_or_else(|| anyhow::anyhow!("{name}: entry escapes the root"))?;
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)?;
            assert_eq!(
                contents,
                std::fs::read(expected_root.join(&relative))?,
                "{name}: contents of {}",
                relative.display()
            );
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
        files.sort();
        assert_eq!(
            files,
            vec![
                "bundle-root/README.md".to_string(),
                "bundle-root/data/bytes.bin".to_string(),
                "bundle-root/skills/demo/SKILL.md".to_string(),
            ],
            "{name}: file list"
        );
    }
    Ok(())
}

#[test]
fn rejected_methods_fail_with_a_clean_error() -> anyhow::Result<()> {
    for (name, method_id) in REJECTED {
        let (_dir, mut archive) = open_copy(name)?;
        let mut refused = 0;
        for index in 0..archive.len() {
            let result = archive.by_index(index).and_then(|mut entry| {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents).map_err(ZipError::from)?;
                Ok((entry.is_dir(), entry.compression()))
            });
            match result {
                Ok((true, _)) | Ok((_, CompressionMethod::Stored)) => {}
                Ok((_, other)) => panic!("{name}: entry {index} decoded with {other:?}"),
                Err(
                    ZipError::UnsupportedArchive(_) | ZipError::CompressionMethodNotSupported(_),
                ) => {
                    refused += 1;
                }
                Err(other) => {
                    panic!("{name}: entry {index} failed with an unexpected error: {other}")
                }
            }
        }
        assert_eq!(
            refused, 3,
            "{name}: every method-{method_id} entry is refused"
        );
    }
    Ok(())
}
