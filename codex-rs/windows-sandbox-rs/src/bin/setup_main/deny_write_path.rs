use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::os::windows::fs::MetadataExt as _;
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::Component;
use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
use windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_READ;
use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

/// #1953: inspect each component without following reparse points, before
/// creating anything below it. Keep every handle open through the ACL write.
/// No write/delete sharing means neither a rename nor a reparse-point edit
/// can redirect the helper after its check.
pub(super) fn pin(path: &Path) -> Result<Vec<File>> {
    anyhow::ensure!(
        path.is_absolute()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir)),
        "deny-write path must be absolute without parent traversal: {}",
        path.display()
    );
    let mut pins = Vec::new();
    let ancestors: Vec<_> = path.ancestors().collect();
    for part in ancestors.into_iter().rev() {
        if !part.has_root() {
            continue;
        }
        let open = || {
            std::fs::OpenOptions::new()
                .access_mode(FILE_GENERIC_READ)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(part)
        };
        let file = match open() {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // Its parent is already pinned. An entry raced into this
                // name is opened and checked, never followed.
                match std::fs::create_dir(part) {
                    Ok(()) => {}
                    Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(err) => {
                        return Err(err).with_context(|| format!("create {}", part.display()));
                    }
                }
                open().with_context(|| format!("pin {}", part.display()))?
            }
            Err(err) => return Err(err).with_context(|| format!("pin {}", part.display())),
        };
        let metadata = file
            .metadata()
            .with_context(|| format!("inspect {}", part.display()))?;
        anyhow::ensure!(
            metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0,
            "refusing a reparse point in deny-write path: {}",
            part.display()
        );
        anyhow::ensure!(
            part == path || metadata.is_dir(),
            "not a directory: {}",
            part.display()
        );
        pins.push(file);
    }
    Ok(pins)
}

#[cfg(test)]
#[path = "deny_write_path_tests.rs"]
mod tests;
