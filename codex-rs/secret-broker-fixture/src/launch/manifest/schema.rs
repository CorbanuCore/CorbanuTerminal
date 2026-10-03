use serde::Deserialize;
use std::io;
use std::path::PathBuf;

use super::MANIFEST_LIMIT;
use super::SyntheticLaunchRecipe;
use super::denied;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Principal {
    uid: u32,
    gid: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Principals {
    journal: Principal,
    policy: Principal,
    worker: Principal,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Manifest {
    schema_version: u32,
    purpose: String,
    pub(super) source_commit: String,
    pub(super) probe_sha256: String,
    principals: Principals,
    anchor_gid: u32,
}

impl Manifest {
    pub(super) fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() as u64 > MANIFEST_LIMIT {
            return Err(denied("manifest oversized"));
        }
        let m: Self = serde_json::from_slice(bytes).map_err(|_| denied("manifest malformed"))?;
        let hex = |s: &str, len| {
            s.len() == len
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if m.schema_version != 1
            || m.purpose != "synthetic-identity-preparation"
            || !hex(&m.source_commit, 40)
            || !hex(&m.probe_sha256, 64)
        {
            return Err(denied("manifest contract invalid"));
        }
        m.recipe()?;
        Ok(m)
    }

    pub(super) fn recipe(&self) -> io::Result<SyntheticLaunchRecipe> {
        let path = PathBuf::from("/opt/corbanu-protected-test")
            .join(&self.source_commit)
            .join("codex-protected-root-probe");
        let p = &self.principals;
        SyntheticLaunchRecipe::new(
            path,
            [
                (p.journal.uid, p.journal.gid),
                (p.policy.uid, p.policy.gid),
                (p.worker.uid, p.worker.gid),
            ],
            self.anchor_gid,
        )
    }
}
