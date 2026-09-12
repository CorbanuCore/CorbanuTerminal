//! Narrow static-PIE format inspection, not a loader or execution approval.
use super::IMAGE_LIMIT;
use super::denied;
use std::collections::BTreeSet;
use std::io::{self, Read, Seek, SeekFrom};

fn require(value: bool) -> io::Result<()> {
    if value { Ok(()) } else { Err(denied("unsupported ELF profile")) }
}
fn end(start: u64, size: u64) -> io::Result<u64> {
    start.checked_add(size).ok_or_else(|| denied("ELF range overflow"))
}
fn read<const N: usize>(input: &mut (impl Read + Seek), offset: u64) -> io::Result<[u8; N]> {
    let mut bytes = [0; N];
    input.seek(SeekFrom::Start(offset)).and_then(|_| input.read_exact(&mut bytes))
        .map_err(|_| denied("ELF read failed"))?;
    Ok(bytes)
}
fn u16_at(b: &[u8], at: usize) -> u16 { u16::from_le_bytes([b[at], b[at + 1]]) }
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}
fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3], b[at + 4], b[at + 5], b[at + 6], b[at + 7]])
}

struct Segment { flags: u32, offset: u64, address: u64, size: u64, memory: u64 }
fn overlaps(a: u64, a_size: u64, b: u64, b_size: u64) -> io::Result<bool> {
    Ok(a_size != 0 && b_size != 0 && a < end(b, b_size)? && b < end(a, a_size)?)
}

pub(super) fn inspect(input: &mut (impl Read + Seek), size: u64) -> io::Result<()> {
    require((64..=IMAGE_LIMIT).contains(&size))?;
    let h = read::<64>(input, 0)?;
    require(&h[..7] == b"\x7fELF\x02\x01\x01" && matches!(h[7], 0 | 3)
        && h[8..16] == [0; 8] && u16_at(&h, 16) == 3 && u16_at(&h, 18) == 62
        && u32_at(&h, 20) == 1 && u32_at(&h, 48) == 0
        && u16_at(&h, 52) == 64 && u16_at(&h, 54) == 56)?;
    let count = u64::from(u16_at(&h, 56));
    let table = u64_at(&h, 32);
    require((1..=128).contains(&count) && table >= 64 && end(table, count * 56)? <= size)?;
    let mut loads: Vec<Segment> = Vec::new();
    let mut singletons = BTreeSet::new();
    let mut dynamic = None;
    for n in 0..count {
        let p = read::<56>(input, table + n * 56)?;
        let kind = u32_at(&p, 0);
        let segment = Segment { flags: u32_at(&p, 4), offset: u64_at(&p, 8),
            address: u64_at(&p, 16), size: u64_at(&p, 32), memory: u64_at(&p, 40) };
        let alignment = u64_at(&p, 48);
        require(segment.size <= segment.memory && end(segment.offset, segment.size)? <= size)?;
        end(segment.address, segment.memory)?;
        require(alignment <= 1 || (alignment.is_power_of_two()
            && segment.offset % alignment == segment.address % alignment))?;
        let allowed_flags = match kind {
            1 => matches!(segment.flags, 4..=6), // LOAD: R, RX, RW, never WX
            2 | 0x6474e551 => segment.flags == 6, // DYNAMIC / GNU_STACK
            6 | 7 | 0x6474e550 | 0x6474e552 => segment.flags == 4,
            _ => false, // Includes INTERP and every unqualified header type.
        };
        require(allowed_flags && (kind == 1 || singletons.insert(kind)))?;
        match kind {
            1 => {
                for prior in &loads {
                    require(!overlaps(segment.offset, segment.size, prior.offset, prior.size)?
                        && !overlaps(segment.address, segment.memory, prior.address, prior.memory)?)?;
                }
                loads.push(segment);
            }
            2 => dynamic = Some(segment),
            _ => {}
        }
    }
    let entry = u64_at(&h, 24);
    require(loads.iter().any(|l| l.flags == 5 && entry >= l.address
        && entry - l.address < l.size))?;
    let d = dynamic.ok_or_else(|| denied("ELF dynamic table missing"))?;
    require(d.size == d.memory && d.size >= 16 && d.size % 16 == 0 && d.size / 16 <= 4096)?;
    let mut backing = 0;
    for l in &loads {
        if d.offset >= l.offset && end(d.offset, d.size)? <= end(l.offset, l.size)?
            && d.address >= l.address && d.address - l.address == d.offset - l.offset {
            backing += 1;
        }
    }
    require(backing == 1)?;
    let mut seen = BTreeSet::new();
    let mut terminated = false;
    let mut pie = false;
    for n in 0..d.size / 16 {
        let b = read::<16>(input, d.offset + n * 16)?;
        let tag = u64_at(&b, 0);
        let value = u64_at(&b, 8);
        if terminated || tag == 0 {
            require(tag == 0 && value == 0)?; // Zero-only padding, never active tags.
            terminated = true;
            continue;
        }
        require(seen.insert(tag))?;
        let supported = match tag {
            4..=8 | 10 | 12 | 13 | 25..=28 | 0x6ffffef5 | 0x6ffffff9 => true,
            9 | 11 => value == 24,
            21 => value == 0,
            30 => value == 8,
            0x6ffffffb => { pie = value == 0x08000001; pie }
            _ => false,
        };
        require(supported)?;
    }
    require(terminated && pie)
}

#[cfg(test)]
#[path = "elf_tests.rs"]
pub(super) mod tests;
