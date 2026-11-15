use super::*;
use pretty_assertions::assert_eq;

fn reparse(target: &str) -> Vec<u8> {
    let path: Vec<u8> = target.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut data = MOUNT_POINT.to_le_bytes().to_vec();
    data.extend_from_slice(&((8 + path.len()) as u16).to_le_bytes());
    data.extend_from_slice(&[0; 4]); // reserved and substitute offset
    data.extend_from_slice(&(path.len() as u16).to_le_bytes());
    data.extend_from_slice(&[0; 4]); // empty print name
    data.extend_from_slice(&path);
    data
}

#[test]
fn only_an_exact_volume_guid_root_is_recognized() {
    let guid = "01234567-89ab-cdef-0123-456789abcdef";
    let target = format!(r"\??\Volume{{{guid}}}\");
    assert_eq!(
        volume_root(&reparse(&target)),
        Some(format!(r"\\?\GLOBALROOT\GLOBAL??\Volume{{{guid}}}\"))
    );
    for target in [
        r"\??\C:\",
        r"\??\UNC\server\share\",
        r"\Device\NamedPipe\pipe",
        r"\??\Volume{01234567-89ab-cdef-0123-456789abcdef}\child",
        r"\??\Volume{01234567-89ab-cdef-0123-456789abcdef}:stream",
        r"\??\Volume{01234567-89ab-cdef-0123-456789abcdeg}\",
        r"\??\Volume{01234567-89ab-cdef-0123-456789abcdef}",
        r"\??\Volume{01234567-89ab-cdef-0123-456789abcdef}\..\",
    ] {
        assert_eq!(volume_root(&reparse(target)), None, "{target}");
    }
}

#[test]
fn malformed_mount_data_and_other_tags_are_refused() {
    let valid = reparse(r"\??\Volume{01234567-89ab-cdef-0123-456789abcdef}\");
    for end in 0..valid.len() {
        assert_eq!(volume_root(&valid[..end]), None, "truncated at {end}");
    }
    for (offset, value) in [
        (4, 0_u16),
        (8, 1),
        (8, u16::MAX - 1),
        (10, 3),
        (12, 1),
        (14, u16::MAX),
    ] {
        let mut bad = valid.clone();
        bad[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        assert_eq!(volume_root(&bad), None, "invalid field at {offset}");
    }
    let mut symlink = valid;
    symlink[..4].copy_from_slice(&0xA000_000C_u32.to_le_bytes());
    assert_eq!(volume_root(&symlink), None);
}
