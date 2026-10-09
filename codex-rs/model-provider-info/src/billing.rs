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
    /// A ChatGPT account sign-in (or the tokens issued for one).
    ChatgptLogin,
    /// Another OpenAI account credential (request headers, an agent identity
    /// or a personal access token). No row declares a basis for it.
    OtherAccountToken,
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
                Some(AuthMode::Chatgpt | AuthMode::ChatgptAuthTokens) => Self::ChatgptLogin,
                Some(
                    AuthMode::Headers | AuthMode::AgentIdentity | AuthMode::PersonalAccessToken,
                ) => Self::OtherAccountToken,
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
    (AMAZON_BEDROCK_PROVIDER_ID, C::ProviderLogin, PayPerUse),
    (OLLAMA_OSS_PROVIDER_ID, C::None, Local),
    (LMSTUDIO_OSS_PROVIDER_ID, C::None, Local),
];

/// Known routes that are not a built-in provider's default, for any provider
/// pointed at them with the given credential.
const ROUTES: &[(&str, BillingCredential, BillingBasis)] = &[
    ("https://api.openai.com/v1", C::ApiKey, PayPerUse),
    // Codex's realtime calls go to the API host with the ChatGPT sign-in;
    // that usage is part of the ChatGPT plan (developers.openai.com/codex/pricing).
    ("https://api.openai.com/v1", C::ChatgptLogin, Subscription),
    (
        "https://api.z.ai/api/coding/paas/v4",
        C::ApiKey,
        Subscription,
    ),
    (
        "https://open.bigmodel.cn/api/coding/paas/v4",
        C::ApiKey,
        Subscription,
    ),
    ("https://open.bigmodel.cn/api/paas/v4", C::ApiKey, PayPerUse),
    ("https://api.moonshot.ai/v1", C::ApiKey, PayPerUse),
];

/// Credentials that only one built-in provider can send, whatever endpoint it
/// resolves to: AWS signing and a command login on Bedrock, whose route is
/// regional and resolved at runtime.
fn provider_held(provider_id: &str, credential: BillingCredential) -> bool {
    provider_id == AMAZON_BEDROCK_PROVIDER_ID && matches!(credential, C::Aws | C::ProviderLogin)
}

/// Whether `endpoint` is on this machine. Local work is declared only there:
/// a local provider pointed at another host may be a paid service.
fn on_this_machine(endpoint: &str) -> bool {
    let rest = endpoint
        .split_once("://")
        .map_or(endpoint, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = if host.starts_with('[') {
        host.split_once(']')
            .map_or(host, |(host, _)| host)
            .trim_start_matches('[')
    } else {
        host.split(':').next().unwrap_or_default()
    };
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

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
    if let Some((_, _, basis)) = ROUTES
        .iter()
        .find(|(route, accepted, _)| *route == endpoint && *accepted == credential)
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
    static DEFAULTS: std::sync::OnceLock<Vec<(String, BillingCredential, BillingBasis)>> =
        std::sync::OnceLock::new();
    DEFAULTS.get_or_init(|| {
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
    (at_built_in_route || provider_held(provider_id, credential))
        .then(|| built_in_basis(provider_id, credential))
        .flatten()
        .or_else(|| route_basis(endpoint, credential))
        .filter(|basis| *basis != Local || on_this_machine(endpoint))
        .map_or(BillingDeclaration::NotDeclared, BillingDeclaration::BuiltIn)
}

/// What to tell a user whose route has no declared basis: the config key that
/// declares it, and what happens until then.
pub fn not_declared_next_step(provider_id: &str) -> String {
    // A bare TOML key holds only these characters; any other id is quoted.
    let bare = !provider_id.is_empty()
        && provider_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    let key = if bare {
        provider_id.to_string()
    } else {
        format!("{provider_id:?}")
    };
    format!(
        "set model_providers.{key}.billing = \"subscription\", \"pay_per_use\" or \"local\" \
         in config.toml. Until then these requests are counted neither as money spent nor as subscription work."
    )
}

#[cfg(test)]
#[path = "billing_tests.rs"]
mod tests;
