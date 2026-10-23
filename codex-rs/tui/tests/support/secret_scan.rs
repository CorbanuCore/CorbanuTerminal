//! Secret-canary scans over fixture trees that the TUI under test may still be writing.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;

/// Reports whether any regular file under `root` contains `needle`.
///
/// Symlinks, sockets, and paths for which `skip` returns true are not read. The TUI may still be
/// running and rotating files in the tree (shell snapshots are written under a temporary name and
/// then renamed, for example), so an entry that disappears between listing and reading is treated
/// as absent: it no longer holds any bytes. Every other I/O error fails the scan and names the
/// path.
pub(crate) fn tree_contains(
    root: &Path,
    needle: &[u8],
    skip: &dyn Fn(&Path) -> bool,
) -> Result<bool> {
    if skip(root) {
        return Ok(false);
    }
    let Some(metadata) = unless_vanished(fs::symlink_metadata(root), root)? else {
        return Ok(false);
    };
    let file_type = metadata.file_type();
    if file_type.is_file() {
        let Some(bytes) = unless_vanished(fs::read(root), root)? else {
            return Ok(false);
        };
        return Ok(bytes.windows(needle.len()).any(|window| window == needle));
    }
    if !file_type.is_dir() {
        return Ok(false);
    }
    let Some(entries) = unless_vanished(fs::read_dir(root), root)? else {
        return Ok(false);
    };
    for entry in entries {
        let Some(entry) = unless_vanished(entry, root)? else {
            continue;
        };
        if tree_contains(&entry.path(), needle, skip)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn unless_vanished<T>(result: io::Result<T>, path: &Path) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("scan {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::tree_contains;

    #[test]
    fn scan_finds_nested_canary_and_honors_skip() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let nested = root.path().join("a/b");
        std::fs::create_dir_all(&nested)?;
        std::fs::write(nested.join("custody.json"), b"canary")?;
        assert!(tree_contains(root.path(), b"canary", &|_| false)?);
        assert!(!tree_contains(root.path(), b"canary", &|path| {
            path.ends_with("custody.json")
        })?);
        Ok(())
    }

    #[test]
    fn scan_treats_missing_root_as_absent() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        assert!(!tree_contains(
            &root.path().join("renamed.tmp-1"),
            b"canary",
            &|_| false
        )?);
        Ok(())
    }
}
