//! Bounded snapshot matching without additional credential-derived metadata.
//! A rolling hash indexes a small cache of *nonmatching text windows*. Cache
//! hits require exact byte equality, so collisions cannot hide credentials.
//! Keyed SHA-256 remains the authority; no rolling hash of a secret is stored.

use sha2::Digest;
use std::collections::BTreeMap;
use std::collections::HashSet;

// Eight distinct lengths in a 3 MiB snapshot cost at most 24 Mi windows.
// Keep the existing 16 MiB input limit, and bound expensive hashing/comparisons
// separately, including adversarial cache collisions and 8192-byte keys.
pub(super) const MAX_WINDOWS: usize = 32 * 1024 * 1024;
const MAX_WORK_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const CACHE_SLOTS: usize = 16 * 1024;
const BASE: u64 = 257;

pub(super) struct Fingerprint {
    pub length: usize,
    pub prefix: Vec<u8>,
    pub digest: [u8; 32],
}

pub(super) fn scan(text: &str, records: Vec<Fingerprint>, max_windows: usize) -> Result<bool, ()> {
    if text.len() > codex_secret_broker::output_gate::MAX_SCAN_BYTES || records.len() > 128 {
        return Err(());
    }
    let mut groups = BTreeMap::<usize, BTreeMap<Vec<u8>, HashSet<[u8; 32]>>>::new();
    for record in records {
        if !(3..=8192).contains(&record.length) || record.prefix.len() > 8192 {
            return Err(());
        }
        groups
            .entry(record.length)
            .or_default()
            .entry(record.prefix)
            .or_default()
            .insert(record.digest);
    }
    let mut windows = 0usize;
    for length in groups.keys() {
        windows = windows
            .checked_add(text.len().checked_sub(*length).map_or(0, |n| n + 1))
            .ok_or(())?;
    }
    if windows > max_windows {
        return Err(());
    }
    let text = text.as_bytes();
    let mut work_left = MAX_WORK_BYTES;
    for (length, prefixes) in groups {
        if length > text.len() {
            continue;
        }
        let prefixes: Vec<_> = prefixes
            .into_iter()
            .map(|(prefix, digests)| {
                (
                    sha2::Sha256::new().chain_update(&prefix),
                    prefix.len(),
                    digests,
                )
            })
            .collect();
        // Only positions in the caller's existing text are retained, never
        // copies of windows or credential material. Memory is 256 KiB.
        let mut cache = vec![(0u64, usize::MAX); CACHE_SLOTS];
        let power = BASE.wrapping_pow((length - 1) as u32);
        let mut rolling = text[..length].iter().fold(0u64, |hash, byte| {
            hash.wrapping_mul(BASE).wrapping_add(u64::from(*byte))
        });
        for start in 0..=text.len() - length {
            if start > 0 {
                rolling = rolling
                    .wrapping_sub(u64::from(text[start - 1]).wrapping_mul(power))
                    .wrapping_mul(BASE)
                    .wrapping_add(u64::from(text[start + length - 1]));
            }
            let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
            if length < 8
                && ((start > 0 && word(text[start - 1]))
                    || text.get(start + length).copied().is_some_and(word))
            {
                continue;
            }
            let window = &text[start..start + length];
            // Mix high bits into the slot index: consecutive shell text often
            // has identical low polynomial bits.
            let slot = ((rolling ^ (rolling >> 32)) as usize) & (CACHE_SLOTS - 1);
            let (hash, previous) = cache[slot];
            if hash == rolling && previous != usize::MAX {
                work_left = work_left.checked_sub(length as u64).ok_or(())?;
                if window == &text[previous..previous + length] {
                    continue;
                }
            }
            for (prefix, prefix_len, expected) in &prefixes {
                work_left = work_left
                    .checked_sub((length + prefix_len + 64) as u64)
                    .ok_or(())?;
                let digest: [u8; 32] = prefix.clone().chain_update(window).finalize().into();
                if expected.contains(&digest) {
                    return Ok(true);
                }
            }
            cache[slot] = (rolling, start);
        }
    }
    Ok(false)
}

#[cfg(test)]
#[path = "snapshot_scan_tests.rs"]
mod tests;
