use codex_vault::Vault;
use std::sync::Arc;
use thiserror::Error;
use zeroize::Zeroize;

pub const VAST_API_KEY_LABEL: &str = "gpu/provider/vast/api-key";
pub const RUNPOD_API_KEY_LABEL: &str = "gpu/provider/runpod/api-key";
pub const HUGGINGFACE_TOKEN_LABEL: &str = "gpu/huggingface/token";
const RENTAL_ENDPOINT_TOKEN_PREFIX: &str = "gpu/rental/";
const RENTAL_ENDPOINT_TOKEN_SUFFIX: &str = "/endpoint-token";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuCredentialKind {
    ProviderApiKey { provider: String },
    HuggingFaceToken,
    RentalEndpointToken { rental_id: String },
}

impl GpuCredentialKind {
    pub fn canonical_label(&self) -> Result<String, GpuCredentialError> {
        match self {
            Self::ProviderApiKey { provider } if provider == "vast" => {
                Ok(VAST_API_KEY_LABEL.to_string())
            }
            Self::ProviderApiKey { provider } if provider == "runpod" => {
                Ok(RUNPOD_API_KEY_LABEL.to_string())
            }
            Self::ProviderApiKey { provider } => {
                Err(GpuCredentialError::UnsupportedProvider(provider.clone()))
            }
            Self::HuggingFaceToken => Ok(HUGGINGFACE_TOKEN_LABEL.to_string()),
            Self::RentalEndpointToken { rental_id } => {
                validate_rental_id(rental_id)?;
                Ok(format!("gpu/rental/{rental_id}/endpoint-token"))
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: String) -> Result<Self, GpuCredentialError> {
        if value.trim().is_empty() {
            return Err(GpuCredentialError::Empty);
        }
        Ok(Self(value))
    }

    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretValue([REDACTED])")
    }
}

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub struct GpuCredential {
    pub label: String,
    pub secret: SecretValue,
}

impl std::fmt::Debug for GpuCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuCredential")
            .field("label", &self.label)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GpuCredentialError {
    #[error("GPU credential is not configured")]
    Missing,
    #[error("GPU credential is empty")]
    Empty,
    #[error("GPU provider {0:?} is unsupported")]
    UnsupportedProvider(String),
    #[error("rental id is invalid")]
    InvalidRentalId,
    #[error("GPU credential store is unavailable")]
    StoreUnavailable,
}

impl GpuCredentialError {
    pub fn retryable(&self) -> bool {
        matches!(self, Self::Missing | Self::StoreUnavailable)
    }
}

/// Resolves the narrowly scoped credentials used by GPU providers and rental endpoints.
///
/// Implementations backed by a writable credential store should override
/// [`GpuCredentialResolver::ensure_rental_endpoint_token`] to create a missing per-rental token
/// idempotently. Read-only implementations may use the default resolver-only behavior.
pub trait GpuCredentialResolver: Send + Sync {
    fn resolve(&self, kind: &GpuCredentialKind) -> Result<GpuCredential, GpuCredentialError>;

    /// Lists rental ids with managed endpoint tokens. Read-only resolvers may return no ids.
    fn rental_endpoint_token_ids(&self) -> Result<Vec<String>, GpuCredentialError> {
        Ok(Vec::new())
    }

    /// Deletes a managed endpoint token after its rental is provider-confirmed terminated.
    fn delete_rental_endpoint_token(&self, _rental_id: &str) -> Result<bool, GpuCredentialError> {
        Ok(false)
    }

    /// Deletes managed endpoint tokens in one store operation when supported.
    fn delete_rental_endpoint_tokens(
        &self,
        rental_ids: &[String],
    ) -> Result<usize, GpuCredentialError> {
        let mut deleted = 0;
        for rental_id in rental_ids {
            deleted += usize::from(self.delete_rental_endpoint_token(rental_id)?);
        }
        Ok(deleted)
    }

    fn ensure_rental_endpoint_token(
        &self,
        rental_id: &str,
    ) -> Result<GpuCredential, GpuCredentialError> {
        self.resolve(&GpuCredentialKind::RentalEndpointToken {
            rental_id: rental_id.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct VaultGpuCredentialResolver {
    vault: Arc<Vault>,
}

impl VaultGpuCredentialResolver {
    pub fn new(vault: Arc<Vault>) -> Self {
        Self { vault }
    }

    pub fn ensure_rental_endpoint_token(
        &self,
        rental_id: &str,
    ) -> Result<GpuCredential, GpuCredentialError> {
        let kind = GpuCredentialKind::RentalEndpointToken {
            rental_id: rental_id.to_string(),
        };
        let label = kind.canonical_label()?;
        match self.resolve(&kind) {
            Ok(credential) => return Ok(credential),
            Err(GpuCredentialError::Missing) => {}
            Err(error) => return Err(error),
        }
        let token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let add_result = self.vault.add(codex_vault::AddCredential {
            label,
            credential_type: codex_vault::CredentialType::BearerToken,
            provider: Some("gpu-rental".to_string()),
            notes: Some("Corbanu Terminal per-rental inference endpoint token".to_string()),
            revocation_notes: Some("Delete after provider-confirmed termination".to_string()),
            secret: token,
        });
        match add_result {
            Ok(()) | Err(codex_vault::VaultError::CredentialExists { .. }) => self.resolve(&kind),
            Err(_) => Err(GpuCredentialError::StoreUnavailable),
        }
    }
}

impl std::fmt::Debug for VaultGpuCredentialResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultGpuCredentialResolver")
            .finish_non_exhaustive()
    }
}

impl GpuCredentialResolver for VaultGpuCredentialResolver {
    fn resolve(&self, kind: &GpuCredentialKind) -> Result<GpuCredential, GpuCredentialError> {
        let label = kind.canonical_label()?;
        let secret = self
            .vault
            .reveal(label.as_str())
            .map_err(|error| match error {
                codex_vault::VaultError::NotFound { .. } => GpuCredentialError::Missing,
                codex_vault::VaultError::EmptySecret => GpuCredentialError::Empty,
                _ => GpuCredentialError::StoreUnavailable,
            })?;
        Ok(GpuCredential {
            label,
            secret: SecretValue::new(secret)?,
        })
    }

    fn ensure_rental_endpoint_token(
        &self,
        rental_id: &str,
    ) -> Result<GpuCredential, GpuCredentialError> {
        VaultGpuCredentialResolver::ensure_rental_endpoint_token(self, rental_id)
    }

    fn rental_endpoint_token_ids(&self) -> Result<Vec<String>, GpuCredentialError> {
        let mut rental_ids = self
            .vault
            .list()
            .map_err(|_| GpuCredentialError::StoreUnavailable)?
            .into_iter()
            .filter(|credential| credential.provider.as_deref() == Some("gpu-rental"))
            .filter_map(|credential| {
                rental_id_from_endpoint_token_label(credential.label.as_str()).map(str::to_string)
            })
            .collect::<Vec<_>>();
        rental_ids.sort_unstable();
        rental_ids.dedup();
        Ok(rental_ids)
    }

    fn delete_rental_endpoint_token(&self, rental_id: &str) -> Result<bool, GpuCredentialError> {
        let label = GpuCredentialKind::RentalEndpointToken {
            rental_id: rental_id.to_string(),
        }
        .canonical_label()?;
        self.vault
            .delete(label.as_str())
            .map_err(|_| GpuCredentialError::StoreUnavailable)
    }

    fn delete_rental_endpoint_tokens(
        &self,
        rental_ids: &[String],
    ) -> Result<usize, GpuCredentialError> {
        let labels = rental_ids
            .iter()
            .map(|rental_id| {
                GpuCredentialKind::RentalEndpointToken {
                    rental_id: rental_id.clone(),
                }
                .canonical_label()
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.vault
            .delete_many(&labels)
            .map_err(|_| GpuCredentialError::StoreUnavailable)
    }
}

fn rental_id_from_endpoint_token_label(label: &str) -> Option<&str> {
    let rental_id = label
        .strip_prefix(RENTAL_ENDPOINT_TOKEN_PREFIX)?
        .strip_suffix(RENTAL_ENDPOINT_TOKEN_SUFFIX)?;
    validate_rental_id(rental_id).is_ok().then_some(rental_id)
}

fn validate_rental_id(rental_id: &str) -> Result<(), GpuCredentialError> {
    if rental_id.is_empty()
        || rental_id.len() > 128
        || !rental_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Err(GpuCredentialError::InvalidRentalId);
    }
    Ok(())
}

#[cfg(test)]
#[path = "credential_tests.rs"]
mod tests;
