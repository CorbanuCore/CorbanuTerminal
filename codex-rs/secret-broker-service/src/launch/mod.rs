//! Synthetic launch configuration, NOT validated executable or OS authority.
mod manifest;
pub use manifest::SyntheticManifestInspection;
pub use manifest::SyntheticSealedImage;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntheticChildRole {
    Journal,
    Policy,
    Worker,
}

impl SyntheticChildRole {
    fn name(self) -> &'static str {
        match self {
            Self::Journal => "journal",
            Self::Policy => "policy",
            Self::Worker => "worker",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Journal => 0,
            Self::Policy => 1,
            Self::Worker => 2,
        }
    }
}

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid synthetic launch recipe",
    )
}

fn valid_id(id: u32) -> bool {
    id != 0 && id != u32::MAX
}

/// Validated numeric preparation arguments; only the actual root caller may apply them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticChildIdentity {
    uid: u32,
    gid: u32,
    groups: Vec<u32>,
}

impl SyntheticChildIdentity {
    pub fn parse(args: &[String]) -> io::Result<Self> {
        let [role, uid, gid, anchor] = args else {
            return Err(invalid());
        };
        let number = |text: &str| -> io::Result<u32> {
            let value: u32 = text.parse().map_err(|_| invalid())?;
            if !valid_id(value) || value.to_string() != text {
                return Err(invalid());
            }
            Ok(value)
        };
        let uid = number(uid)?;
        let gid = number(gid)?;
        let groups = match (role.as_str(), anchor.as_str()) {
            ("worker", "none") => vec![],
            ("journal" | "policy", value) => {
                let group = number(value)?;
                if group == gid {
                    return Err(invalid());
                }
                vec![group]
            }
            _ => return Err(invalid()),
        };
        Ok(Self { uid, gid, groups })
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }
    pub fn gid(&self) -> u32 {
        self.gid
    }
    pub fn groups(&self) -> &[u32] {
        &self.groups
    }
}

/// A command recipe only. The later root supervisor must validate a root-owned
/// manifest and pin the executable before spawning; this type does neither.
pub struct SyntheticLaunchRecipe {
    executable: PathBuf,
    identities: [(u32, u32); 3],
    anchor_gid: u32,
}

impl SyntheticLaunchRecipe {
    pub fn new(
        executable: PathBuf,
        identities: [(u32, u32); 3],
        anchor_gid: u32,
    ) -> io::Result<Self> {
        if !executable.is_absolute() || !valid_id(anchor_gid) {
            return Err(invalid());
        }
        for (index, (uid, gid)) in identities.iter().enumerate() {
            if !valid_id(*uid)
                || !valid_id(*gid)
                || *gid == anchor_gid
                || identities[..index]
                    .iter()
                    .any(|(u, g)| u == uid || g == gid)
            {
                return Err(invalid());
            }
        }
        Ok(Self {
            executable,
            identities,
            anchor_gid,
        })
    }

    pub fn command(&self, role: SyntheticChildRole) -> Command {
        let (uid, gid) = self.identities[role.index()];
        let anchor = match role {
            SyntheticChildRole::Worker => "none".to_owned(),
            SyntheticChildRole::Journal | SyntheticChildRole::Policy => self.anchor_gid.to_string(),
        };
        let mut command = Command::new(&self.executable);
        command
            .args([
                "--prepare-synthetic-child",
                role.name(),
                &uid.to_string(),
                &gid.to_string(),
                &anchor,
            ])
            .env_clear()
            .current_dir("/")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    }
}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
