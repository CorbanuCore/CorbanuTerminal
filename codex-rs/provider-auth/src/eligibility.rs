use std::collections::BTreeSet;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use serde::Deserialize;
use serde::Serialize;

use crate::ApiKeyStorage;
use crate::ProviderCatalogEntry;
use crate::ProviderSetupCapability;

const PROVIDER_ELIGIBILITY_VERSION: u32 = 1;
const PROVIDER_ELIGIBILITY_FILE: &str = "provider-eligibility.json";
const MAX_ELIGIBILITY_ID_BYTES: usize = 512;
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Stable identity used for persisted provider activation policy.
///
/// API-key entries use their credential environment identity so a shared-key
/// custom-provider group remains stable when its runtime members change.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderEligibilityId(String);

impl ProviderEligibilityId {
    pub fn for_entry(entry: &ProviderCatalogEntry) -> Self {
        let environment_key =
            entry
                .setup_capabilities
                .iter()
                .find_map(|capability| match capability {
                    ProviderSetupCapability::ApiKey {
                        storage: ApiKeyStorage::EnvironmentVariable { env_key },
                    } => Some(env_key.as_str()),
                    ProviderSetupCapability::OpenAiAccount
                    | ProviderSetupCapability::ApiKey {
                        storage: ApiKeyStorage::OpenAiAuth,
                    }
                    | ProviderSetupCapability::ClaudeAccount
                    | ProviderSetupCapability::CorbanuPlan
                    | ProviderSetupCapability::Local { .. }
                    | ProviderSetupCapability::CommandAuth { .. }
                    | ProviderSetupCapability::StatusOnly { .. } => None,
                });
        environment_key.map_or_else(
            || Self(format!("provider:{}", entry.id.as_str())),
            |env_key| Self(format!("credential-env:{env_key}")),
        )
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn parse(value: String) -> Result<Self, ProviderEligibilityError> {
        let value = value.trim();
        if value.is_empty()
            || value.len() > MAX_ELIGIBILITY_ID_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(ProviderEligibilityError::Malformed);
        }
        Ok(Self(value.to_string()))
    }
}

impl fmt::Display for ProviderEligibilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Requested durable activation policy for one provider setup identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderActivationPolicy {
    Active,
    Inactive,
}

/// Metadata-only persisted eligibility policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderEligibility {
    inactive_identities: BTreeSet<ProviderEligibilityId>,
}

impl ProviderEligibility {
    pub fn policy_for(&self, entry: &ProviderCatalogEntry) -> ProviderActivationPolicy {
        if self
            .inactive_identities
            .contains(&ProviderEligibilityId::for_entry(entry))
        {
            ProviderActivationPolicy::Inactive
        } else {
            ProviderActivationPolicy::Active
        }
    }

    pub fn set_policy(&mut self, entry: &ProviderCatalogEntry, policy: ProviderActivationPolicy) {
        let identity = ProviderEligibilityId::for_entry(entry);
        match policy {
            ProviderActivationPolicy::Active => {
                self.inactive_identities.remove(&identity);
            }
            ProviderActivationPolicy::Inactive => {
                self.inactive_identities.insert(identity);
            }
        }
    }

    pub fn inactive_identities(&self) -> &BTreeSet<ProviderEligibilityId> {
        &self.inactive_identities
    }
}

/// Typed persistence failure that never retains upstream error strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderEligibilityError {
    ReadUnavailable,
    WriteUnavailable,
    Malformed,
    UnsupportedVersion { found: u32 },
}

impl fmt::Display for ProviderEligibilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadUnavailable => {
                formatter.write_str("provider eligibility state is unavailable")
            }
            Self::WriteUnavailable => {
                formatter.write_str("provider eligibility state could not be saved")
            }
            Self::Malformed => formatter.write_str("provider eligibility state is malformed"),
            Self::UnsupportedVersion { found } => write!(
                formatter,
                "provider eligibility state version {found} is unsupported"
            ),
        }
    }
}

impl std::error::Error for ProviderEligibilityError {}

/// Durable store for global provider eligibility metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEligibilityStore {
    path: PathBuf,
}

impl ProviderEligibilityStore {
    pub fn new(codex_home: impl AsRef<Path>) -> Self {
        Self {
            path: codex_home.as_ref().join(PROVIDER_ELIGIBILITY_FILE),
        }
    }

    pub fn load(&self) -> Result<ProviderEligibility, ProviderEligibilityError> {
        let contents = match std::fs::read_to_string(&self.path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ProviderEligibility::default());
            }
            Err(_) => return Err(ProviderEligibilityError::ReadUnavailable),
        };
        let stored: StoredEligibility =
            serde_json::from_str(&contents).map_err(|_| ProviderEligibilityError::Malformed)?;
        if stored.version != PROVIDER_ELIGIBILITY_VERSION {
            return Err(ProviderEligibilityError::UnsupportedVersion {
                found: stored.version,
            });
        }
        let inactive_identities = stored
            .inactive_identities
            .into_iter()
            .map(ProviderEligibilityId::parse)
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(ProviderEligibility {
            inactive_identities,
        })
    }

    pub fn save(&self, eligibility: &ProviderEligibility) -> Result<(), ProviderEligibilityError> {
        let stored = StoredEligibility {
            version: PROVIDER_ELIGIBILITY_VERSION,
            inactive_identities: eligibility
                .inactive_identities
                .iter()
                .map(|identity| identity.as_str().to_string())
                .collect(),
        };
        let mut json = serde_json::to_string_pretty(&stored)
            .map_err(|_| ProviderEligibilityError::WriteUnavailable)?;
        json.push('\n');
        write_atomically(&self.path, json.as_bytes())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoredEligibility {
    version: u32,
    inactive_identities: Vec<String>,
}

fn write_atomically(path: &Path, contents: &[u8]) -> Result<(), ProviderEligibilityError> {
    let parent = path
        .parent()
        .ok_or(ProviderEligibilityError::WriteUnavailable)?;
    std::fs::create_dir_all(parent).map_err(|_| ProviderEligibilityError::WriteUnavailable)?;
    let temporary = path.with_file_name(format!(
        ".provider-eligibility.json.{}.{}.tmp",
        std::process::id(),
        NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|_| ProviderEligibilityError::WriteUnavailable)?;
    file.write_all(contents)
        .and_then(|()| file.sync_all())
        .map_err(|_| ProviderEligibilityError::WriteUnavailable)?;
    if std::fs::rename(&temporary, path).is_err() {
        let _ = std::fs::remove_file(&temporary);
        return Err(ProviderEligibilityError::WriteUnavailable);
    }
    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| ProviderEligibilityError::WriteUnavailable)?;
    Ok(())
}

#[cfg(test)]
#[path = "eligibility_tests.rs"]
mod tests;
