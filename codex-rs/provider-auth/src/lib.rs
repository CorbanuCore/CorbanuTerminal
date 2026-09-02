//! Renderer-independent provider catalog and setup-capability contract.
//!
//! Runtime transport definitions remain owned by `codex-model-provider-info`.
//! This crate derives user-facing setup identities from the resolved runtime
//! provider map and never creates a second transport registry.

mod api_key_flow;
mod auth_flow;
mod claude_account_controller;
pub mod claude_account_flow;
mod claude_account_settlement;
mod eligibility;
mod management;
mod openai_account_controller;
mod openai_account_flow;
mod runtime_selection;
mod status;
mod status_contract;

pub use api_key_flow::ApiKeyAuthTarget;
pub use api_key_flow::ApiKeyFlowContext;
pub use api_key_flow::ApiKeyFlowIntent;
pub use api_key_flow::ApiKeyFlowStart;
pub use api_key_flow::ApiKeySecret;
pub use api_key_flow::ApiKeyTargetError;
pub use auth_flow::API_KEY_AUTH_TIMEOUT;
pub use auth_flow::ApiKeyPersistenceResult;
pub use auth_flow::PROVIDER_AUTH_FLOW_PROTOCOL_VERSION;
pub use auth_flow::ProviderAuthAction;
pub use auth_flow::ProviderAuthAttemptId;
pub use auth_flow::ProviderAuthBlockedReason;
pub use auth_flow::ProviderAuthCompletion;
pub use auth_flow::ProviderAuthController;
pub use auth_flow::ProviderAuthDisposition;
pub use auth_flow::ProviderAuthEffect;
pub use auth_flow::ProviderAuthFailureReason;
pub use auth_flow::ProviderAuthFlowSnapshot;
pub use auth_flow::ProviderAuthRejectionReason;
pub use auth_flow::ProviderAuthTransition;
pub use eligibility::ProviderActivationPolicy;
pub use eligibility::ProviderEligibility;
pub use eligibility::ProviderEligibilityError;
pub use eligibility::ProviderEligibilityId;
pub use eligibility::ProviderEligibilityStore;
pub use management::ProviderManagementSession;
pub use openai_account_flow::OpenAiAccountAction;
pub use openai_account_flow::OpenAiAccountBlockedReason;
pub use openai_account_flow::OpenAiAccountCancelPurpose;
pub use openai_account_flow::OpenAiAccountChallenge;
pub use openai_account_flow::OpenAiAccountCompletion;
pub use openai_account_flow::OpenAiAccountEffect;
pub use openai_account_flow::OpenAiAccountFailureReason;
pub use openai_account_flow::OpenAiAccountFlow;
pub use openai_account_flow::OpenAiAccountFlowStart;
pub use openai_account_flow::OpenAiAccountLoginContext;
pub use openai_account_flow::OpenAiAccountLoginId;
pub use openai_account_flow::OpenAiAccountLoginOutcome;
pub use openai_account_flow::OpenAiAccountMethod;
pub use openai_account_flow::OpenAiAccountOutcomeUnknownReason;
pub use openai_account_flow::OpenAiAccountRecoveryReason;
pub use openai_account_flow::OpenAiAccountSnapshot;
pub use openai_account_flow::OpenAiAccountStartResult;
pub use openai_account_flow::OpenAiAccountTarget;
pub use openai_account_flow::OpenAiAccountTargetError;
pub use openai_account_flow::OpenAiCancelResult;
pub use runtime_selection::CurrentSelectionDecision;
pub use runtime_selection::ProviderRuntimeAuthorization;
pub use runtime_selection::ProviderRuntimeAuthorizations;
pub use runtime_selection::ProviderRuntimeSelectionPolicy;
pub use runtime_selection::ProviderUseBlocker;
pub use runtime_selection::ProviderUseContext;
pub use runtime_selection::ProviderUseDecision;
pub use status::ProviderStatusResolver;
pub use status_contract::ApiKeyCredentialMetadata;
pub use status_contract::ClaudeCredentialMetadata;
pub use status_contract::ClaudeCredentialSource;
pub use status_contract::CommandAuthMetadata;
pub use status_contract::ConfiguredAvailability;
pub use status_contract::CorbanuCredentialSource;
pub use status_contract::CorbanuPlanMetadata;
pub use status_contract::CredentialControl;
pub use status_contract::CurrentProviderSelection;
pub use status_contract::EnvironmentCredentialMetadata;
pub use status_contract::LocalProviderMetadata;
pub use status_contract::ManagedApiKeyMetadata;
pub use status_contract::ProviderAvailabilityState;
pub use status_contract::ProviderConfigurationState;
pub use status_contract::ProviderCredentialSource;
pub use status_contract::ProviderCurrentState;
pub use status_contract::ProviderEligibilitySnapshot;
pub use status_contract::ProviderEligibilityState;
pub use status_contract::ProviderMetadata;
pub use status_contract::ProviderMetadataSnapshot;
pub use status_contract::ProviderMethodState;
pub use status_contract::ProviderMethodStatus;
pub use status_contract::ProviderRecoveryReason;
pub use status_contract::ProviderStatusCatalog;
pub use status_contract::ProviderStatusSnapshot;
pub use status_contract::ProviderUnavailableReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderManagementAttemptId(u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitProviderSelection {
    pub provider_id: ProviderCatalogId,
    pub runtime_provider_id: ProviderRuntimeId,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderManagementMutation {
    Eligibility {
        provider_id: ProviderCatalogId,
        policy: ProviderActivationPolicy,
    },
    ReplacementThenDeactivate {
        target_provider_id: ProviderCatalogId,
        replacement: ExplicitProviderSelection,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderManagementPhase {
    Browsing,
    Authenticating {
        attempt_id: ProviderManagementAttemptId,
        provider_id: ProviderCatalogId,
        preserve_inactive: bool,
    },
    AwaitingReplacement {
        target_provider_id: ProviderCatalogId,
    },
    Persisting {
        attempt_id: ProviderManagementAttemptId,
        mutation: ProviderManagementMutation,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderManagementPersistenceResult {
    Applied,
    ReplacementAppliedDeactivationFailed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderManagementAction {
    BeginAuthentication {
        provider_id: ProviderCatalogId,
    },
    AuthenticationConfigured {
        provider_id: ProviderCatalogId,
    },
    AuthenticationCancelled {
        provider_id: ProviderCatalogId,
    },
    RequestPolicy {
        provider_id: ProviderCatalogId,
        policy: ProviderActivationPolicy,
    },
    ChooseReplacement {
        target_provider_id: ProviderCatalogId,
        replacement: ExplicitProviderSelection,
    },
    CancelReplacement {
        target_provider_id: ProviderCatalogId,
    },
    PersistenceFinished {
        attempt_id: ProviderManagementAttemptId,
        result: ProviderManagementPersistenceResult,
    },
    Refresh {
        statuses: Vec<ProviderStatusSnapshot>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderManagementEffect {
    BeginAuthentication {
        attempt_id: ProviderManagementAttemptId,
        provider_id: ProviderCatalogId,
    },
    PersistEligibility {
        attempt_id: ProviderManagementAttemptId,
        provider_id: ProviderCatalogId,
        policy: ProviderActivationPolicy,
    },
    PresentReplacement {
        target_provider_id: ProviderCatalogId,
    },
    PersistReplacementThenDeactivate {
        attempt_id: ProviderManagementAttemptId,
        target_provider_id: ProviderCatalogId,
        replacement: ExplicitProviderSelection,
    },
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderManagementTransition {
    pub phase: ProviderManagementPhase,
    pub statuses: Vec<ProviderStatusSnapshot>,
    pub effects: Vec<ProviderManagementEffect>,
    pub applied: bool,
    pub persistence_result: Option<ProviderManagementPersistenceResult>,
}

use codex_model_provider_info::AMAZON_BEDROCK_PROVIDER_ID;
use codex_model_provider_info::AMBIENT_PROVIDER_ID;
use codex_model_provider_info::ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::BASETEN_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::BASETEN_PROVIDER_ID;
use codex_model_provider_info::CLAUDE_PLAN_PROVIDER_ID;
use codex_model_provider_info::CORBANU_PLAN_PROVIDER_ID;
use codex_model_provider_info::DEEPSEEK_PROVIDER_ID;
use codex_model_provider_info::KIMI_CODE_PROVIDER_ID;
use codex_model_provider_info::LMSTUDIO_OSS_PROVIDER_ID;
use codex_model_provider_info::META_PROVIDER_ID;
use codex_model_provider_info::ModelProviderCredentialSource;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_model_provider_info::OPENROUTER_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::OPENROUTER_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::PFTERMINAL_PLAN_PROVIDER_ID;
use codex_model_provider_info::VERCEL_ANTHROPIC_FAST_PROVIDER_ID;
use codex_model_provider_info::VERCEL_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::VERCEL_PROVIDER_ID;
use codex_model_provider_info::ZAI_ANTHROPIC_PROVIDER_ID;
use codex_model_provider_info::ZAI_PROVIDER_ID;
use codex_model_provider_info::canonical_provider_id;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

/// Stable identity of one product-facing provider setup entry.
///
/// This is deliberately distinct from runtime transport IDs. One setup entry
/// can own several runtime providers that share credentials or wire aliases.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderCatalogId(String);

impl ProviderCatalogId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderCatalogId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Identity of a transport provider in the resolved runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderRuntimeId(String);

impl ProviderRuntimeId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderRuntimeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Storage boundary used by an API-key setup method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyStorage {
    /// OpenAI's account-auth store, which is shared with OpenAI account login.
    OpenAiAuth,
    /// A provider-specific environment-variable identity resolved by the vault.
    EnvironmentVariable { env_key: String },
}

/// Supported local inference runtimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalProvider {
    Ollama,
    LmStudio,
}

/// Interactive support for a command-backed provider.
///
/// Arbitrary commands are intentionally not dispatchable from renderer hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAuthSetup {
    StatusOnly,
}

/// Why a provider has metadata/status presentation but no setup action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusOnlyReason {
    AwsCredentialChain,
    NoInteractiveSetup,
}

/// One setup or presentation capability for a catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderSetupCapability {
    OpenAiAccount,
    ApiKey { storage: ApiKeyStorage },
    ClaudeAccount,
    CorbanuPlan,
    Local { provider: LocalProvider },
    CommandAuth { setup: CommandAuthSetup },
    StatusOnly { reason: StatusOnlyReason },
}

/// Ordered, non-empty setup capabilities for one catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSetupCapabilities {
    pub primary: ProviderSetupCapability,
    pub alternatives: Vec<ProviderSetupCapability>,
}

impl ProviderSetupCapabilities {
    fn one(primary: ProviderSetupCapability) -> Self {
        Self {
            primary,
            alternatives: Vec::new(),
        }
    }

    fn openai() -> Self {
        Self {
            primary: ProviderSetupCapability::OpenAiAccount,
            alternatives: vec![ProviderSetupCapability::ApiKey {
                storage: ApiKeyStorage::OpenAiAuth,
            }],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProviderSetupCapability> {
        std::iter::once(&self.primary).chain(self.alternatives.iter())
    }
}

/// One renderer-independent provider catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCatalogEntry {
    pub id: ProviderCatalogId,
    pub display_name: String,
    pub runtime_provider_ids: Vec<ProviderRuntimeId>,
    pub setup_capabilities: ProviderSetupCapabilities,
}

/// Deterministically ordered provider inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderCatalog {
    entries: Vec<ProviderCatalogEntry>,
}

impl ProviderCatalog {
    /// Derive a catalog from the resolved runtime provider configuration.
    ///
    /// Runtime aliases and providers sharing one API-key environment variable
    /// are grouped into one setup entry. No transport definition is synthesized.
    pub fn from_runtime_providers(runtime_providers: &HashMap<String, ModelProviderInfo>) -> Self {
        let mut candidates = runtime_providers
            .iter()
            .filter_map(|(runtime_id, provider)| Candidate::new(runtime_id, provider))
            .collect::<Vec<_>>();
        candidates.sort_by(Candidate::compare);

        let mut groups: BTreeMap<GroupKey, Vec<Candidate<'_>>> = BTreeMap::new();
        for candidate in candidates {
            groups
                .entry(candidate.group_key.clone())
                .or_default()
                .push(candidate);
        }

        let mut ranked_entries = groups
            .into_values()
            .filter_map(CatalogEntryWithRank::from_candidates)
            .collect::<Vec<_>>();
        ranked_entries.sort_by(CatalogEntryWithRank::compare);

        Self {
            entries: ranked_entries
                .into_iter()
                .map(|ranked| ranked.entry)
                .collect(),
        }
    }

    pub fn entries(&self) -> &[ProviderCatalogEntry] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&ProviderCatalogEntry> {
        self.entries.iter().find(|entry| entry.id.as_str() == id)
    }

    pub fn into_entries(self) -> Vec<ProviderCatalogEntry> {
        self.entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum GroupKey {
    Identity(String),
    EnvironmentVariable(String),
}

struct Candidate<'a> {
    runtime_id: &'a str,
    catalog_id: String,
    display_name: String,
    group_key: GroupKey,
    setup_capabilities: ProviderSetupCapabilities,
    rank: usize,
}

impl<'a> Candidate<'a> {
    fn new(runtime_id: &'a str, provider: &ModelProviderInfo) -> Option<Self> {
        let runtime_id = runtime_id.trim();
        if runtime_id.is_empty() {
            return None;
        }

        let canonical_runtime_id = canonical_provider_id(runtime_id);
        let catalog_id = catalog_id(canonical_runtime_id).to_string();
        let setup_capabilities = setup_capabilities(canonical_runtime_id, provider);
        let group_key = group_key(&catalog_id, &setup_capabilities);
        let display_name = display_name(&catalog_id, provider);

        Some(Self {
            runtime_id,
            rank: provider_rank(canonical_runtime_id),
            catalog_id,
            display_name,
            group_key,
            setup_capabilities,
        })
    }

    fn compare(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.catalog_id.cmp(&right.catalog_id))
            .then_with(|| left.runtime_id.cmp(right.runtime_id))
    }
}

struct CatalogEntryWithRank {
    entry: ProviderCatalogEntry,
    rank: usize,
}

impl CatalogEntryWithRank {
    fn from_candidates(candidates: Vec<Candidate<'_>>) -> Option<Self> {
        let representative = candidates.first()?;
        let mut runtime_provider_ids = candidates
            .iter()
            .map(|candidate| ProviderRuntimeId(candidate.runtime_id.to_string()))
            .collect::<Vec<_>>();
        runtime_provider_ids.sort();
        runtime_provider_ids.dedup();

        Some(Self {
            rank: representative.rank,
            entry: ProviderCatalogEntry {
                id: ProviderCatalogId(representative.catalog_id.clone()),
                display_name: representative.display_name.clone(),
                runtime_provider_ids,
                setup_capabilities: representative.setup_capabilities.clone(),
            },
        })
    }

    fn compare(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.entry.display_name.cmp(&right.entry.display_name))
            .then_with(|| left.entry.id.cmp(&right.entry.id))
    }
}

fn catalog_id(canonical_runtime_id: &str) -> &str {
    match canonical_runtime_id {
        PFTERMINAL_PLAN_PROVIDER_ID | PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID => {
            CORBANU_PLAN_PROVIDER_ID
        }
        _ => canonical_runtime_id,
    }
}

fn group_key(catalog_id: &str, capabilities: &ProviderSetupCapabilities) -> GroupKey {
    match capabilities.iter().find_map(|capability| match capability {
        ProviderSetupCapability::ApiKey {
            storage: ApiKeyStorage::EnvironmentVariable { env_key },
        } => Some(env_key),
        ProviderSetupCapability::OpenAiAccount
        | ProviderSetupCapability::ApiKey {
            storage: ApiKeyStorage::OpenAiAuth,
        }
        | ProviderSetupCapability::ClaudeAccount
        | ProviderSetupCapability::CorbanuPlan
        | ProviderSetupCapability::Local { .. }
        | ProviderSetupCapability::CommandAuth { .. }
        | ProviderSetupCapability::StatusOnly { .. } => None,
    }) {
        Some(env_key) => GroupKey::EnvironmentVariable(env_key.clone()),
        None => GroupKey::Identity(catalog_id.to_string()),
    }
}

fn setup_capabilities(
    canonical_runtime_id: &str,
    provider: &ModelProviderInfo,
) -> ProviderSetupCapabilities {
    match canonical_runtime_id {
        OPENAI_PROVIDER_ID => ProviderSetupCapabilities::openai(),
        CLAUDE_PLAN_PROVIDER_ID => {
            ProviderSetupCapabilities::one(ProviderSetupCapability::ClaudeAccount)
        }
        PFTERMINAL_PLAN_PROVIDER_ID | PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID => {
            ProviderSetupCapabilities::one(ProviderSetupCapability::CorbanuPlan)
        }
        OLLAMA_OSS_PROVIDER_ID => ProviderSetupCapabilities::one(ProviderSetupCapability::Local {
            provider: LocalProvider::Ollama,
        }),
        LMSTUDIO_OSS_PROVIDER_ID => {
            ProviderSetupCapabilities::one(ProviderSetupCapability::Local {
                provider: LocalProvider::LmStudio,
            })
        }
        _ => match provider.credential_source() {
            ModelProviderCredentialSource::OpenAiAuth => ProviderSetupCapabilities::openai(),
            ModelProviderCredentialSource::EnvironmentApiKey { env_key }
                if !env_key.trim().is_empty() =>
            {
                ProviderSetupCapabilities::one(ProviderSetupCapability::ApiKey {
                    storage: ApiKeyStorage::EnvironmentVariable {
                        env_key: env_key.trim().to_string(),
                    },
                })
            }
            ModelProviderCredentialSource::Command => {
                ProviderSetupCapabilities::one(ProviderSetupCapability::CommandAuth {
                    setup: CommandAuthSetup::StatusOnly,
                })
            }
            ModelProviderCredentialSource::Aws => {
                ProviderSetupCapabilities::one(ProviderSetupCapability::StatusOnly {
                    reason: StatusOnlyReason::AwsCredentialChain,
                })
            }
            ModelProviderCredentialSource::EnvironmentApiKey { .. }
            | ModelProviderCredentialSource::None => {
                ProviderSetupCapabilities::one(ProviderSetupCapability::StatusOnly {
                    reason: StatusOnlyReason::NoInteractiveSetup,
                })
            }
        },
    }
}

fn display_name(catalog_id: &str, provider: &ModelProviderInfo) -> String {
    match catalog_id {
        OPENAI_PROVIDER_ID => "OpenAI".to_string(),
        CLAUDE_PLAN_PROVIDER_ID => "Claude Account".to_string(),
        CORBANU_PLAN_PROVIDER_ID => "Corbanu Plan".to_string(),
        OLLAMA_OSS_PROVIDER_ID => "Ollama".to_string(),
        LMSTUDIO_OSS_PROVIDER_ID => "LM Studio".to_string(),
        _ => {
            let name = provider.name.trim();
            if name.is_empty() {
                catalog_id.to_string()
            } else {
                name.to_string()
            }
        }
    }
}

fn provider_rank(canonical_runtime_id: &str) -> usize {
    match canonical_runtime_id {
        OPENAI_PROVIDER_ID => 0,
        CLAUDE_PLAN_PROVIDER_ID => 1,
        PFTERMINAL_PLAN_PROVIDER_ID | PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID => 2,
        ANTHROPIC_PROVIDER_ID => 3,
        AMBIENT_PROVIDER_ID => 4,
        KIMI_CODE_PROVIDER_ID => 5,
        ZAI_PROVIDER_ID | ZAI_ANTHROPIC_PROVIDER_ID => 6,
        DEEPSEEK_PROVIDER_ID => 7,
        OPENROUTER_PROVIDER_ID | OPENROUTER_ANTHROPIC_PROVIDER_ID => 8,
        META_PROVIDER_ID => 9,
        BASETEN_PROVIDER_ID | BASETEN_ANTHROPIC_PROVIDER_ID => 10,
        VERCEL_PROVIDER_ID | VERCEL_ANTHROPIC_PROVIDER_ID | VERCEL_ANTHROPIC_FAST_PROVIDER_ID => 11,
        AMAZON_BEDROCK_PROVIDER_ID => 12,
        OLLAMA_OSS_PROVIDER_ID => 13,
        LMSTUDIO_OSS_PROVIDER_ID => 14,
        _ => 15,
    }
}

#[cfg(test)]
#[path = "provider_catalog_tests.rs"]
mod tests;
