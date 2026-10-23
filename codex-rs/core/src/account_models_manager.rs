//! PF-84: the process's model list for an OpenAI sign-in provider, fetched
//! with the sign-in of the account the active session uses.
//!
//! The list (`/models`, and which models a ChatGPT or API-key sign-in may
//! pick) used to be bound to the account the process started on. This
//! manager keeps one stable handle (the app server, its refresh worker and
//! every session hold it) and swaps the manager behind it when a user's
//! session starts on, or switches to, another account. The process has one
//! list: with several user sessions on different accounts (an app-server
//! client with many threads) the last one to start or switch decides it.

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use codex_http_client::HttpClientFactory;
use codex_login::AuthManager;
use codex_login::NamedChatgptAccount;
use codex_model_provider::create_model_provider;
use codex_model_provider::named_sign_in_account;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::NamedProviderAccount;
use codex_models_manager::ModelsManagerConfig;
use codex_models_manager::manager::ModelsManager;
use codex_models_manager::manager::ModelsManagerFuture;
use codex_models_manager::manager::RefreshStrategy;
use codex_models_manager::manager::SharedModelsManager;
use codex_protocol::config_types::CollaborationModeMask;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ModelsResponse;
use tokio::sync::TryLockError;

/// Under the home: one model-list cache per named account.
const NAMED_ACCOUNT_MODELS_CACHE_DIR: &str = "models-cache";

pub(crate) struct AccountModelsManager {
    /// The provider the list is for, without an account.
    provider: ModelProviderInfo,
    default_auth: Arc<AuthManager>,
    codex_home: PathBuf,
    model_catalog: Option<ModelsResponse>,
    current: ArcSwap<Current>,
    switching: std::sync::Mutex<()>,
}

struct Current {
    account: Option<NamedChatgptAccount>,
    manager: SharedModelsManager,
}

impl fmt::Debug for AccountModelsManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountModelsManager")
            .field("provider", &self.provider.name)
            .field("account", &self.current.load().account)
            .finish_non_exhaustive()
    }
}

impl AccountModelsManager {
    /// `None` when `provider` has no OpenAI sign-in to follow.
    pub(crate) fn for_provider(
        provider: &ModelProviderInfo,
        auth_manager: Arc<AuthManager>,
        codex_home: PathBuf,
        model_catalog: Option<ModelsResponse>,
    ) -> Option<Self> {
        if !provider.requires_openai_auth || provider.aws.is_some() || provider.auth.is_some() {
            return None;
        }
        let account = named_sign_in_account(provider);
        let mut provider = provider.clone();
        provider.account = None;
        let default_auth = auth_manager.default_account_manager();
        let current = build(
            &provider,
            &default_auth,
            &codex_home,
            &model_catalog,
            account,
        );
        Some(Self {
            provider,
            default_auth,
            codex_home,
            model_catalog,
            current: ArcSwap::from_pointee(current),
            switching: std::sync::Mutex::new(()),
        })
    }

    fn inner(&self) -> SharedModelsManager {
        Arc::clone(&self.current.load().manager)
    }

    /// #658: while the session's account is not known yet (a resumed or
    /// forked thread before it starts), the default login's list comes from
    /// its cache only: it is never fetched with the default login.
    fn strategy(&self, refresh_strategy: RefreshStrategy) -> RefreshStrategy {
        if self.current.load().account.is_none() && self.default_auth.session_account_unknown() {
            RefreshStrategy::Offline
        } else {
            refresh_strategy
        }
    }

    #[cfg(test)]
    pub(crate) fn account(&self) -> Option<NamedChatgptAccount> {
        self.current.load().account.clone()
    }
}

impl ModelsManager for AccountModelsManager {
    fn list_models(
        &self,
        refresh_strategy: RefreshStrategy,
        http_client_factory: HttpClientFactory,
    ) -> ModelsManagerFuture<'_, Vec<ModelPreset>> {
        let inner = self.inner();
        let refresh_strategy = self.strategy(refresh_strategy);
        Box::pin(async move {
            inner
                .list_models(refresh_strategy, http_client_factory)
                .await
        })
    }

    fn raw_model_catalog(
        &self,
        refresh_strategy: RefreshStrategy,
        http_client_factory: HttpClientFactory,
    ) -> ModelsManagerFuture<'_, ModelsResponse> {
        let inner = self.inner();
        let refresh_strategy = self.strategy(refresh_strategy);
        Box::pin(async move {
            inner
                .raw_model_catalog(refresh_strategy, http_client_factory)
                .await
        })
    }

    fn get_remote_models(&self) -> ModelsManagerFuture<'_, Vec<ModelInfo>> {
        let inner = self.inner();
        Box::pin(async move { inner.get_remote_models().await })
    }

    fn try_get_remote_models(&self) -> Result<Vec<ModelInfo>, TryLockError> {
        self.inner().try_get_remote_models()
    }

    /// The account changes under this handle, so no reference to one
    /// account's manager is lent out; every method that filters by sign-in
    /// asks the current manager instead.
    fn auth_manager(&self) -> Option<&AuthManager> {
        None
    }

    fn server_advertised_models(&self) -> Option<Vec<String>> {
        self.inner().server_advertised_models()
    }

    fn build_available_models(&self, remote_models: Vec<ModelInfo>) -> Vec<ModelPreset> {
        self.inner().build_available_models(remote_models)
    }

    fn list_collaboration_modes(&self) -> Vec<CollaborationModeMask> {
        self.inner().list_collaboration_modes()
    }

    fn try_list_models(&self) -> Result<Vec<ModelPreset>, TryLockError> {
        self.inner().try_list_models()
    }

    fn get_default_model<'a>(
        &'a self,
        model: &'a Option<String>,
        allow_provider_model_fallback: bool,
        refresh_strategy: RefreshStrategy,
        http_client_factory: HttpClientFactory,
    ) -> ModelsManagerFuture<'a, String> {
        let inner = self.inner();
        let refresh_strategy = self.strategy(refresh_strategy);
        Box::pin(async move {
            inner
                .get_default_model(
                    model,
                    allow_provider_model_fallback,
                    refresh_strategy,
                    http_client_factory,
                )
                .await
        })
    }

    fn get_model_info<'a>(
        &'a self,
        model: &'a str,
        config: &'a ModelsManagerConfig,
    ) -> ModelsManagerFuture<'a, ModelInfo> {
        let inner = self.inner();
        Box::pin(async move { inner.get_model_info(model, config).await })
    }

    fn refresh_if_new_etag(
        &self,
        etag: String,
        http_client_factory: HttpClientFactory,
    ) -> ModelsManagerFuture<'_, ()> {
        let inner = self.inner();
        let offline = self.strategy(RefreshStrategy::Online) == RefreshStrategy::Offline;
        Box::pin(async move {
            if !offline {
                inner.refresh_if_new_etag(etag, http_client_factory).await;
            }
        })
    }

    /// Only responses from this provider's sign-in, on the account the list
    /// follows (another provider's etag says nothing about this list).
    fn lists_for_provider(&self, provider: &ModelProviderInfo) -> bool {
        provider.requires_openai_auth
            && provider.aws.is_none()
            && provider.auth.is_none()
            && provider.base_url == self.provider.base_url
            && self.current.load().account == named_sign_in_account(provider)
    }

    fn load_cached_list(&self) -> ModelsManagerFuture<'_, bool> {
        let inner = self.inner();
        Box::pin(async move { inner.load_cached_list().await })
    }

    fn follow_sign_in_account(&self, account: Option<&NamedChatgptAccount>) -> bool {
        // Sessions starting together rebuild (and refetch) once.
        let _switching = self
            .switching
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.current.load().account.as_ref() == account {
            return false;
        }
        self.current.store(Arc::new(build(
            &self.provider,
            &self.default_auth,
            &self.codex_home,
            &self.model_catalog,
            account.cloned(),
        )));
        true
    }
}

/// `provider`'s model list with `account`'s sign-in. A named account's list
/// is cached in its own file (`models-cache/<provider>/<name>/`), so another
/// account's list is never shown for it and a switch shows its last list at
/// once.
fn build(
    provider: &ModelProviderInfo,
    default_auth: &Arc<AuthManager>,
    codex_home: &std::path::Path,
    model_catalog: &Option<ModelsResponse>,
    account: Option<NamedChatgptAccount>,
) -> Current {
    let mut provider = provider.clone();
    provider.account = account.as_ref().map(|account| NamedProviderAccount {
        provider_id: account.provider_id.clone(),
        name: account.name.clone(),
    });
    let provider = create_model_provider(provider, Some(Arc::clone(default_auth)));
    let manager = match &account {
        None => provider.models_manager(codex_home.to_path_buf(), model_catalog.clone()),
        Some(account) => match named_account_cache_dir(codex_home, account) {
            Some(dir) => provider.models_manager(dir, model_catalog.clone()),
            None => provider.models_manager_without_cache(model_catalog.clone()),
        },
    };
    Current { account, manager }
}

/// Where a named account's model list is cached; `None` for a name that is
/// not path-safe (names are validated; such a list is not cached).
fn named_account_cache_dir(
    codex_home: &std::path::Path,
    account: &NamedChatgptAccount,
) -> Option<PathBuf> {
    let path_safe = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    };
    (path_safe(&account.provider_id) && path_safe(&account.name)).then(|| {
        codex_home
            .join(NAMED_ACCOUNT_MODELS_CACHE_DIR)
            .join(&account.provider_id)
            .join(&account.name)
    })
}

#[cfg(test)]
#[path = "account_models_manager_tests.rs"]
mod tests;
