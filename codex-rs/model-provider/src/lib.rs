mod account_check;
mod amazon_bedrock;
mod auth;
mod bearer_auth_provider;
<<<<<<< HEAD
mod model_key_broker;
||||||| 413492cd6c
=======
mod capabilities;
mod chatgpt_base_url;
mod combined_auth;
mod gcp;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
mod models_endpoint;
mod models_identity;
mod provider;
mod request_preparation;
mod runtime_auth;
mod shared_state;
pub mod test_support;
mod workspace_routing;
pub use workspace_routing::ACCOUNT_ROUTING_HEADER;
pub use workspace_routing::ResolvedResponsesProvider;
pub use workspace_routing::ResponsesConnectionKey;
pub use workspace_routing::WorkspaceRoutingContext;

<<<<<<< HEAD
pub use account_check::AccountRecovery;
pub use account_check::selected_account_error;
pub use account_check::selected_account_error_with_recovery;
pub use amazon_bedrock::amazon_bedrock_model_presets;
||||||| 413492cd6c
=======
pub use amazon_bedrock::is_amazon_bedrock_gov_cloud_region;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
pub use amazon_bedrock::is_supported_amazon_bedrock_region;
pub use amazon_bedrock::resolve_amazon_bedrock_region;
pub use auth::AgentIdentitySessionFallback;
<<<<<<< HEAD
pub use auth::ProviderApiKey;
pub use auth::ProviderApiKeyHeader;
||||||| 413492cd6c
=======
pub use auth::ProviderAuthContext;
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
pub use auth::ProviderAuthScope;
pub use auth::ResolvedProviderAuth;
pub use auth::auth_provider_from_auth;
pub use auth::auth_provider_from_auth_manager;
pub use auth::named_sign_in_account;
pub use auth::named_sign_in_refusal;
pub use auth::provider_api_key;
pub use auth::session_sign_in;
pub use auth::unauthenticated_auth_provider;
pub use bearer_auth_provider::BearerAuthProvider;
pub use bearer_auth_provider::BearerAuthProvider as CoreAuthProvider;
pub use capabilities::ProviderCapabilities;
pub use chatgpt_base_url::ChatgptBaseUrl;
pub use chatgpt_base_url::DEFAULT_CHATGPT_BASE_URL;
pub use codex_model_provider_info::AMAZON_BEDROCK_PROVIDER_ID;
pub use codex_model_provider_info::AMAZON_BEDROCK_RUNTIME_PROVIDER_ID;
pub use codex_model_provider_info::CHATGPT_CODEX_BASE_URL;
pub use codex_model_provider_info::GCP_PROVIDER_ID;
pub use codex_model_provider_info::RemoteCompactionSupport;
pub use codex_protocol::account::ProviderAccount;
pub use model_key_broker::BROKER_NOT_RUNNING;
pub use model_key_broker::BROKERED_KEY_PLACEHOLDER;
pub use model_key_broker::BrokeredAuthRequest;
pub use model_key_broker::BrokeredKeySource;
pub use model_key_broker::ModelKeyBroker;
pub use model_key_broker::brokered_auth;
pub use model_key_broker::install_model_key_broker;
pub use model_key_broker::model_key_broker_installed;
pub use model_key_broker::model_key_broker_refused;
pub use model_key_broker::model_key_broker_required;
pub use model_key_broker::model_key_broker_unavailable;
pub use model_key_broker::require_model_key_broker;
pub use provider::DEFAULT_APPROVAL_REVIEW_PREFERRED_MODEL;
pub use provider::DEFAULT_MEMORY_CONSOLIDATION_PREFERRED_MODEL;
pub use provider::DEFAULT_MEMORY_EXTRACTION_PREFERRED_MODEL;
pub use provider::ModelProvider;
pub use provider::ModelProviderFuture;
pub use provider::ProviderAccountError;
pub use provider::ProviderAccountResult;
pub use provider::ProviderAccountState;
pub use provider::ProviderAuthRecoveryMessages;
pub use provider::ProviderUnauthorizedRecovery;
pub use provider::SharedModelProvider;
pub use provider::create_model_provider;
pub use provider::provider_uses_first_party_auth_path;
pub use provider::resolve_api_provider;

pub use request_preparation::prepare_provider_request;
pub use runtime_auth::select_runtime_auth;
pub use runtime_auth::validate_runtime_destination;
pub use runtime_auth::validate_runtime_provider;
pub use runtime_auth::validate_runtime_selectors;

#[cfg(test)]
#[path = "workspace_routing_tests.rs"]
mod workspace_routing_tests;
