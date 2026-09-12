use super::denied;
use rustix::fs::Mode;
use rustix::fs::OFlags;
use rustix::fs::ResolveFlags;
use rustix::fs::openat2;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::fs::Metadata;
use std::io::Read;
use std::io::{self};
use std::os::unix::fs::MetadataExt;

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Ancestor,
    Directory(u32),
    Regular(u32),
}
pub(super) type Stamp = [u64; 11];

pub(super) fn check(file: &File, kind: Kind, owner: (u32, u32)) -> io::Result<Stamp> {
    let metadata = file
        .metadata()
        .map_err(|_| denied("metadata unavailable"))?;
    check_metadata(&metadata, kind, owner)?;
    // The read baseline must be the same snapshot whose permissions were checked.
    Ok(metadata_stamp(&metadata))
}

fn check_metadata(m: &Metadata, kind: Kind, owner: (u32, u32)) -> io::Result<()> {
    let mode = m.mode() & 0o7777;
    let valid = match kind {
        Kind::Ancestor => m.is_dir() && mode & 0o7022 == 0,
        Kind::Directory(wanted) => m.is_dir() && mode == wanted,
        Kind::Regular(wanted) => m.is_file() && m.nlink() == 1 && mode == wanted,
    };
    if !valid || (m.uid(), m.gid()) != owner {
        return Err(denied("unsafe file metadata"));
    }
    Ok(())
}

pub(super) fn open(parent: &File, name: &str, kind: Kind, owner: (u32, u32)) -> io::Result<File> {
    let mut flags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK;
    if matches!(kind, Kind::Ancestor | Kind::Directory(_)) {
        flags |= OFlags::DIRECTORY;
    }
    let fd = openat2(
        parent,
        name,
        flags,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|_| denied("safe open failed"))?;
    let file = File::from(fd);
    check(&file, kind, owner)?;
    Ok(file)
}

pub(super) fn stamp(file: &File) -> io::Result<Stamp> {
    let m = file
        .metadata()
        .map_err(|_| denied("metadata unavailable"))?;
    Ok(metadata_stamp(&m))
}

fn metadata_stamp(m: &Metadata) -> Stamp {
    [
        m.dev(),
        m.ino(),
        m.mode().into(),
        m.uid().into(),
        m.gid().into(),
        m.nlink(),
        m.len(),
        m.mtime() as u64,
        m.mtime_nsec() as u64,
        m.ctime() as u64,
        m.ctime_nsec() as u64,
    ]
}

pub(super) fn unchanged(before: Stamp, after: Stamp) -> io::Result<()> {
    if before != after {
        return Err(denied("file changed while reading"));
    }
    Ok(())
}

pub(super) fn image_digest(reader: &mut impl Read, limit: u64) -> io::Result<String> {
    let mut bounded = reader.take(limit + 1);
    let mut header = [0; 64];
    bounded
        .read_exact(&mut header)
        .map_err(|_| denied("image header unavailable"))?;
    if &header[..7] != b"\x7fELF\x02\x01\x01"
        || ![2, 3].contains(&u16::from_le_bytes([header[16], header[17]]))
        || header[18..20] != [62, 0]
        || header[20..24] != [1, 0, 0, 0]
    {
        return Err(denied("image identity invalid"));
    }
    let mut hash = Sha256::new();
    hash.update(header);
    let mut total = header.len() as u64;
    if total > limit {
        return Err(denied("image oversized"));
    }
    let mut chunk = [0; 65536];
    loop {
        let n = bounded
            .read(&mut chunk)
            .map_err(|_| denied("image read failed"))?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > limit {
            return Err(denied("image oversized"));
        }
        hash.update(&chunk[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
#[path = "files_tests.rs"]
mod tests;
