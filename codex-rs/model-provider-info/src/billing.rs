//! Declared billing basis: how the route and credential a request used are paid
//! for (PF-60-S05, option B).
//!
//! Every basis here is declared, never inferred from the kind of login. The
//! built-in table states a basis for each built-in provider and each credential
//! it accepts; the route table states one for known routes a custom provider can
//! point at. Anything else is not declared: its tokens are still recorded, and
//! the user is told how to declare it with `model_providers.<id>.billing`.
//! Sources for every row: `qa/portfolio/agent-cost-accounting/pf-60-s05/
//! billing-basis-defaults.md`.

use crate::AMAZON_BEDROCK_PROVIDER_ID;
use crate::AMBIENT_PROVIDER_ID;
use crate::ANTHROPIC_PROVIDER_ID;
use crate::BASETEN_ANTHROPIC_PROVIDER_ID;
use crate::BASETEN_PROVIDER_ID;
use crate::CLAUDE_PLAN_PROVIDER_ID;
use crate::DEEPSEEK_PROVIDER_ID;
use crate::KIMI_CODE_PROVIDER_ID;
use crate::LMSTUDIO_OSS_PROVIDER_ID;
use crate::META_PROVIDER_ID;
use crate::ModelProviderCredentialSource;
use crate::ModelProviderInfo;
use crate::OLLAMA_OSS_PROVIDER_ID;
use crate::OPENAI_PROVIDER_ID;
use crate::OPENROUTER_ANTHROPIC_PROVIDER_ID;
use crate::OPENROUTER_PROVIDER_ID;
use crate::PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID;
use crate::PFTERMINAL_PLAN_PROVIDER_ID;
use crate::VERCEL_ANTHROPIC_FAST_PROVIDER_ID;
use crate::VERCEL_ANTHROPIC_PROVIDER_ID;
use crate::VERCEL_PROVIDER_ID;
use crate::ZAI_ANTHROPIC_PROVIDER_ID;
use crate::ZAI_PROVIDER_ID;
use codex_protocol::auth::AuthMode;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// How work on a route is paid for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BillingBasis {
    /// Covered by a membership or plan; never counted as money spent.
    Subscription,
    /// Charged per request from a balance or card.
    PayPerUse,
    /// Runs on the user's own machine; no charge.
    Local,
}

/// The kind of credential a request carries, as far as billing is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillingCredential {
    /// A ChatGPT account sign-in (or a token issued for one).
    ChatgptLogin,
    /// An API key or bearer token the user supplied.
    ApiKey,
    /// The provider's own command-based login, such as Claude Plan's.
    ProviderLogin,
    /// AWS request signing.
    Aws,
    /// No credential.
    None,
}

impl BillingCredential {
    /// The credential `provider` sends, given the OpenAI auth mode in use.
    pub fn of(provider: &ModelProviderInfo, auth_mode: Option<AuthMode>) -> Self {
        match provider.credential_source() {
            ModelProviderCredentialSource::OpenAiAuth => match auth_mode {
                Some(AuthMode::ApiKey) => Self::ApiKey,
                Some(
                    AuthMode::Chatgpt
                    | AuthMode::ChatgptAuthTokens
                    | AuthMode::Headers
                    | AuthMode::AgentIdentity
                    | AuthMode::PersonalAccessToken,
                ) => Self::ChatgptLogin,
                // A Bedrock key is not an OpenAI credential.
                Some(AuthMode::BedrockApiKey) | None => Self::None,
            },
            ModelProviderCredentialSource::EnvironmentApiKey { .. } => Self::ApiKey,
            ModelProviderCredentialSource::Command => Self::ProviderLogin,
            ModelProviderCredentialSource::Aws => Self::Aws,
            ModelProviderCredentialSource::None if provider.experimental_bearer_token.is_some() => {
                Self::ApiKey
            }
            ModelProviderCredentialSource::None => Self::None,
        }
    }
}

/// A request's billing basis, and where it was declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillingDeclaration {
    /// From the built-in provider or route table.
    BuiltIn(BillingBasis),
    /// From `model_providers.<id>.billing` in the user's config.
    UserConfig(BillingBasis),
    /// Nothing declares this route and credential.
    NotDeclared,
}

impl BillingDeclaration {
    pub fn basis(self) -> Option<BillingBasis> {
        match self {
            Self::BuiltIn(basis) | Self::UserConfig(basis) => Some(basis),
            Self::NotDeclared => None,
        }
    }
}

use BillingBasis::Local;
use BillingBasis::PayPerUse;
use BillingBasis::Subscription;
use BillingCredential as C;

/// Each built-in provider at its own built-in route, per credential.
const BUILT_IN: &[(&str, BillingCredential, BillingBasis)] = &[
    (OPENAI_PROVIDER_ID, C::ChatgptLogin, Subscription),
    (OPENAI_PROVIDER_ID, C::ApiKey, PayPerUse),
    (ANTHROPIC_PROVIDER_ID, C::ApiKey, PayPerUse),
    (CLAUDE_PLAN_PROVIDER_ID, C::ProviderLogin, Subscription),
    (AMBIENT_PROVIDER_ID, C::ApiKey, PayPerUse),
    (PFTERMINAL_PLAN_PROVIDER_ID, C::ApiKey, PayPerUse),
    (PFTERMINAL_PLAN_ANTHROPIC_PROVIDER_ID, C::ApiKey, PayPerUse),
    (KIMI_CODE_PROVIDER_ID, C::ApiKey, Subscription),
    (ZAI_PROVIDER_ID, C::ApiKey, PayPerUse),
    (ZAI_ANTHROPIC_PROVIDER_ID, C::ApiKey, Subscription),
    (OPENROUTER_PROVIDER_ID, C::ApiKey, PayPerUse),
    (OPENROUTER_ANTHROPIC_PROVIDER_ID, C::ApiKey, PayPerUse),
    (DEEPSEEK_PROVIDER_ID, C::ApiKey, PayPerUse),
    (META_PROVIDER_ID, C::ApiKey, PayPerUse),
    (BASETEN_PROVIDER_ID, C::ApiKey, PayPerUse),
    (BASETEN_ANTHROPIC_PROVIDER_ID, C::ApiKey, PayPerUse),
    (VERCEL_PROVIDER_ID, C::ApiKey, PayPerUse),
    (VERCEL_ANTHROPIC_PROVIDER_ID, C::ApiKey, PayPerUse),
    (VERCEL_ANTHROPIC_FAST_PROVIDER_ID, C::ApiKey, PayPerUse),
    (AMAZON_BEDROCK_PROVIDER_ID, C::Aws, PayPerUse),
    (AMAZON_BEDROCK_PROVIDER_ID, C::ApiKey, PayPerUse),
    (AMAZON_BEDROCK_PROVIDER_ID, C::ProviderLogin, PayPerUse),
    (OLLAMA_OSS_PROVIDER_ID, C::None, Local),
    (LMSTUDIO_OSS_PROVIDER_ID, C::None, Local),
];

/// Known routes that are not a built-in provider's default, for any provider
/// pointed at them with an API key.
const ROUTES: &[(&str, BillingBasis)] = &[
    ("https://api.openai.com/v1", PayPerUse),
    ("https://api.z.ai/api/coding/paas/v4", Subscription),
    ("https://open.bigmodel.cn/api/coding/paas/v4", Subscription),
    ("https://open.bigmodel.cn/api/paas/v4", PayPerUse),
    ("https://api.moonshot.ai/v1", PayPerUse),
];

/// The built-in table's basis for `provider_id` with `credential`.
pub fn built_in_basis(provider_id: &str, credential: BillingCredential) -> Option<BillingBasis> {
    BUILT_IN
        .iter()
        .find(|(id, accepted, _)| *id == provider_id && *accepted == credential)
        .map(|(_, _, basis)| *basis)
}

/// The basis declared for `endpoint` reached with `credential` by any
/// provider: a row of `ROUTES`, or a built-in provider's own default route with
/// the credential that provider is built to send. Two built-ins at one route
/// with different bases would make it ambiguous, so it would be undeclared;
/// a test keeps the table free of that.
pub fn route_basis(endpoint: &str, credential: BillingCredential) -> Option<BillingBasis> {
    let endpoint = endpoint.trim_end_matches('/');
    if credential == C::ApiKey
        && let Some((_, basis)) = ROUTES.iter().find(|(route, _)| *route == endpoint)
    {
        return Some(*basis);
    }
    let mut bases = default_routes()
        .iter()
        .filter(|(route, accepted, _)| route == endpoint && *accepted == credential)
        .map(|(_, _, basis)| *basis);
    let basis = bases.next()?;
    bases.all(|other| other == basis).then_some(basis)
}

/// Built-in default routes that any provider id may use: those reached with an
/// API key or with no credential. A provider-held login (Claude Plan) or AWS
/// signing is declared only for the provider that holds it.
fn default_routes() -> &'static [(String, BillingCredential, BillingBasis)] {
    static ROUTES: std::sync::OnceLock<Vec<(String, BillingCredential, BillingBasis)>> =
        std::sync::OnceLock::new();
    ROUTES.get_or_init(|| {
        crate::built_in_model_providers(/*openai_base_url*/ None)
            .into_iter()
            .filter_map(|(id, provider)| {
                let credential = BillingCredential::of(&provider, /*auth_mode*/ None);
                let route = provider
                    .base_url
                    .as_deref()?
                    .trim_end_matches('/')
                    .to_string();
                matches!(credential, C::ApiKey | C::None)
                    .then(|| built_in_basis(&id, credential))
                    .flatten()
                    .map(|basis| (route, credential, basis))
            })
            .collect()
    })
}

/// The billing basis for a request on `provider` (configured as `provider_id`)
/// that carried `credential` to `endpoint`. `at_built_in_route` says the
/// request went to the provider's own built-in route for that credential.
pub fn declared_billing(
    provider_id: &str,
    provider: &ModelProviderInfo,
    credential: BillingCredential,
    endpoint: &str,
    at_built_in_route: bool,
) -> BillingDeclaration {
    if let Some(basis) = provider.billing {
        return BillingDeclaration::UserConfig(basis);
    }
    at_built_in_route
        .then(|| built_in_basis(provider_id, credential))
        .flatten()
        .or_else(|| route_basis(endpoint, credential))
        .map_or(BillingDeclaration::NotDeclared, BillingDeclaration::BuiltIn)
}

/// What to tell a user whose route has no declared basis.
pub fn not_declared_next_step(provider_id: &str) -> String {
    format!(
        "Billing basis not declared for this provider's route. To declare it, set \
         model_providers.{provider_id}.billing = \"subscription\", \"pay_per_use\" or \"local\" \
         in config.toml."
    )
}

#[cfg(test)]
#[path = "billing_tests.rs"]
mod tests;
