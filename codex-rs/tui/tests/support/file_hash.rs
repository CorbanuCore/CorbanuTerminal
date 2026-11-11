//! SHA-256 of the large binaries tmux tests record as evidence.

use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use sha2::Digest;
use sha2::Sha256;

/// The lowercase hex SHA-256 of `path`.
///
/// Debug binaries are 1-2 GB, and `sha2` in an unoptimized test binary hashes
/// them at about 50 MB/s: 30-90 s a file on CI runners, which serialized
/// provider journeys paid once per test (#1125). The system tool takes about
/// a second, so it is tried first; the in-process hash, streamed, is the
/// fallback.
pub(crate) fn sha256_file(path: &Path) -> Result<String> {
    for (program, arguments) in [("sha256sum", &[][..]), ("shasum", &["-a", "256"][..])] {
        let Ok(output) = Command::new(program).args(arguments).arg(path).output() else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        // GNU tools put a `\` before the hash when they escape the name.
        if let Some(hash) = std::str::from_utf8(&output.stdout)
            .ok()
            .and_then(|text| text.split_whitespace().next())
            .map(|hash| hash.strip_prefix('\\').unwrap_or(hash))
            && hash.len() == 64
            && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Ok(hash.to_ascii_lowercase());
        }
    }
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0_u8; 1 << 20];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// Also for names the system tool escapes (a backslash) or splits on.
    #[test]
    fn sha256_file_matches_the_in_process_hash() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let content = (0..3_000_000_u32)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>();
        let expected = format!("{:x}", Sha256::digest(&content));
        for name in ["binary", "with space", "back\\slash"] {
            let path = dir.path().join(name);
            fs::write(&path, &content)?;
            assert_eq!(sha256_file(&path)?, expected, "{name}");
        }
        Ok(())
    }
}
