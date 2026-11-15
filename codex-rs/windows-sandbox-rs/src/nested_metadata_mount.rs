//! D490 / #1992: cross a volume mount above already-discovered metadata.
//! Ordinary junctions, symbolic links and ambiguous reparse data stay refused.

use crate::acl::walk::Handle;
use crate::acl::walk::Kind;
use std::path::Path;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::IO::DeviceIoControl;

const MOUNT_POINT: u32 = 0xA000_0003;

#[cfg(test)]
#[path = "nested_metadata_mount_parse_tests.rs"]
mod tests;

/// Open only a mount to an entire local volume, never the path a junction
/// could redirect. The reparse data is read from the entry itself relative
/// to its held parent. The caller still verifies the metadata's volume and
/// file ID on the handle it lists; a changed or unreadable target fails closed.
///
/// # Safety
/// `parent` must remain an open directory handle throughout this call.
pub(super) unsafe fn open_volume(parent: HANDLE, name: &[u16]) -> Option<Handle> {
    let mount = crate::acl::walk::open_itself(parent, name, /*access*/ 0).ok()?;
    let attributes = mount.kind().ok()?;
    if !attributes.is_dir() || attributes.tag != MOUNT_POINT {
        return None;
    }
    const FSCTL_GET_REPARSE_POINT: u32 = 0x0009_00A8;
    let mut buffer = [0_u8; 16 * 1024];
    let mut returned = 0_u32;
    if DeviceIoControl(
        mount.0,
        FSCTL_GET_REPARSE_POINT,
        std::ptr::null(),
        0,
        buffer.as_mut_ptr().cast(),
        buffer.len() as u32,
        &mut returned,
        std::ptr::null_mut(),
    ) == 0
    {
        return None;
    }
    let volume = volume_root(buffer.get(..returned as usize)?)?;
    const FILE_TRAVERSE: u32 = 0x20;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    // GLOBAL?? bypasses per-logon DOS-device aliases. Only the mount
    // manager's global volume name is resolved, never the mounted path.
    let root = Handle::open(
        Path::new(&volume),
        FILE_TRAVERSE | SYNCHRONIZE | FILE_READ_ATTRIBUTES,
    )
    .ok()?;
    (root.kind().ok()?.kind() == Kind::Folder).then_some(root)
}

/// Decode only the exact NT volume-root form. The tag alone also describes
/// ordinary junctions, so it is insufficient. Reject malformed lengths,
/// non-ASCII GUIDs, suffixes, streams, UNC/device paths and relative targets.
fn volume_root(buffer: &[u8]) -> Option<String> {
    let word = |at| Some(u16::from_le_bytes(buffer.get(at..at + 2)?.try_into().ok()?) as usize);
    if u32::from_le_bytes(buffer.get(..4)?.try_into().ok()?) != MOUNT_POINT {
        return None;
    }
    let length = word(4)?;
    if length < 8 || length.checked_add(8)? != buffer.len() {
        return None;
    }
    let offset = word(8)?;
    let size = word(10)?;
    let print_offset = word(12)?;
    let print_size = word(14)?;
    let paths = buffer.get(16..)?;
    if [offset, size, print_offset, print_size]
        .iter()
        .any(|value| value % 2 != 0)
    {
        return None;
    }
    paths.get(print_offset..print_offset.checked_add(print_size)?)?;
    let bytes = paths.get(offset..offset.checked_add(size)?)?;
    let wide: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let target = String::from_utf16(&wide).ok()?;
    let guid = target.strip_prefix(r"\??\Volume{")?.strip_suffix(r"}\")?;
    if guid.len() != 36
        || !guid.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
    {
        return None;
    }
    Some(format!(r"\\?\GLOBALROOT\GLOBAL??\Volume{{{guid}}}\"))
}
