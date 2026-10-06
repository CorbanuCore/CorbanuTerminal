//! Per-home key that authenticates persisted origin records (PF-30-S02).
//!
//! A record written by this home carries an HMAC-SHA256 tag over its entries.
//! A session file exported to (or downloaded onto) another home, or a record
//! edited by hand, fails verification, so its content resumes unattributed and
//! reaches protected providers as labelled data. A process that can read this
//! home's key can still forge records; the key only separates homes.

use rand::TryRngCore;
use sha2::Digest;
use sha2::Sha256;
use std::io::Read;
use std::io::Write;
use std::path::Path;

const KEY_FILE: &str = "source-origin.key";
const KEY_BYTES: usize = 32;
const DOMAIN: &[u8] = b"corbanu-source-origin-record-v2\0";

#[derive(Clone)]
pub(crate) struct OriginKey([u8; KEY_BYTES]);

impl std::fmt::Debug for OriginKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OriginKey(..)")
    }
}

impl OriginKey {
    #[cfg(test)]
    pub(crate) fn from_bytes(bytes: [u8; KEY_BYTES]) -> Self {
        Self(bytes)
    }

    /// Read this home's key, creating it (owner-only) on first use.
    pub(crate) fn load_or_create(codex_home: &Path) -> std::io::Result<Self> {
        let path = codex_home.join(KEY_FILE);
        match read_key(&path) {
            Ok(key) => return Ok(key),
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            Err(_) => {}
        }
        std::fs::create_dir_all(codex_home)?;
        let mut bytes = [0_u8; KEY_BYTES];
        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|_| std::io::Error::other("no entropy for the origin key"))?;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        match options.open(&path) {
            Ok(mut file) => {
                file.write_all(&bytes)?;
                file.sync_all()?;
                Ok(Self(bytes))
            }
            // Another session created it first: use theirs.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => read_key(&path),
            Err(error) => Err(error),
        }
    }

    /// Lowercase hex HMAC-SHA256 tag over `message`.
    pub(crate) fn tag(&self, message: &[u8]) -> String {
        hex(&self.mac(message))
    }

    /// Constant-time check of a hex tag produced by [`Self::tag`].
    pub(crate) fn verify(&self, message: &[u8], tag: &str) -> bool {
        let expected = self.tag(message);
        expected.len() == tag.len()
            && expected
                .bytes()
                .zip(tag.bytes())
                .fold(0_u8, |acc, (a, b)| acc | (a ^ b))
                == 0
    }

    fn mac(&self, message: &[u8]) -> [u8; 32] {
        let mut inner_pad = [0x36_u8; 64];
        let mut outer_pad = [0x5c_u8; 64];
        for (index, byte) in self.0.iter().enumerate() {
            inner_pad[index] ^= byte;
            outer_pad[index] ^= byte;
        }
        let inner = Sha256::new()
            .chain_update(inner_pad)
            .chain_update(DOMAIN)
            .chain_update(message)
            .finalize();
        Sha256::new()
            .chain_update(outer_pad)
            .chain_update(inner)
            .finalize()
            .into()
    }
}

fn read_key(path: &Path) -> std::io::Result<OriginKey> {
    let mut bytes = Vec::with_capacity(KEY_BYTES + 1);
    std::fs::File::open(path)?
        .take(KEY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let bytes: [u8; KEY_BYTES] = bytes
        .try_into()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad origin key"))?;
    Ok(OriginKey(bytes))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}
