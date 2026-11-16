use codex_http_client::DestinationPolicy;
use codex_http_client::HttpClientFactory;
use codex_http_client::NetworkPolicy;
use codex_http_client::NetworkPolicyController;

/// Auth-layer adapter around client-owned proxy policy.
///
/// `AuthConfig` carries this value while endpoint resolution and platform details remain in the
/// client layer.
#[derive(Clone)]
pub struct AuthRouteConfig {
    http_client_factory: HttpClientFactory,
    application_network_policy: NetworkPolicy,
    local_bootstrap_factory: Option<HttpClientFactory>,
    account_lifecycle: Option<NetworkPolicyController>,
}

impl std::fmt::Debug for AuthRouteConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthRouteConfig")
            .field("http_client_factory", &self.http_client_factory)
            .field(
                "application_network_policy",
                &self.application_network_policy,
            )
            .field("local_bootstrap_factory", &self.local_bootstrap_factory)
            .finish_non_exhaustive()
    }
}

impl PartialEq for AuthRouteConfig {
    fn eq(&self, other: &Self) -> bool {
        // The read-side policy includes the identity of the lifecycle controller.
        self.http_client_factory == other.http_client_factory
            && self.application_network_policy == other.application_network_policy
            && self.local_bootstrap_factory == other.local_bootstrap_factory
    }
}

impl Eq for AuthRouteConfig {}

impl AuthRouteConfig {
    /// Adapts an application-resolved HTTP client factory for auth requests.
    pub fn from_http_client_factory(http_client_factory: HttpClientFactory) -> Self {
        Self {
            application_network_policy: http_client_factory.network_policy().clone(),
            http_client_factory,
            local_bootstrap_factory: None,
            account_lifecycle: None,
        }
    }

    /// Named accounts own their credential lifecycle, but keep the application's live policy.
    pub(crate) fn for_named_account(&self) -> Self {
        let (policy, lifecycle) = self
            .application_network_policy
            .clone()
            .with_independent_account_lifecycle();
        Self {
            http_client_factory: self
                .http_client_factory
                .clone()
                .with_network_policy(policy.clone()),
            application_network_policy: policy,
            local_bootstrap_factory: self.local_bootstrap_factory.clone(),
            account_lifecycle: Some(lifecycle),
        }
    }

    /// Revoke old credentials before publishing a new named identity. Publication opens only
    /// the account gate: unavailable or restricted administrator policy still refuses requests.
    pub(crate) fn owner_changed(&self, signed_in: bool) {
        self.application_network_policy.invalidate();
        if signed_in && let Some(lifecycle) = &self.account_lifecycle {
            lifecycle.publish(
                lifecycle.policy().revision(),
                DestinationPolicy::Unrestricted,
            );
        }
    }

    /// Selects the account-owned policy revoked when the authenticated identity changes.
    /// Bootstrap discovery can use separate local rules until this policy is loaded.
    pub fn with_application_network_policy(mut self, policy: NetworkPolicy) -> Self {
        self.application_network_policy = policy;
        self
    }

    /// Installs the configuration owner's local-only policy for auth discovery.
    /// Endpoint constructors access it through destination-restricted factories.
    pub fn with_local_bootstrap_factory(mut self, factory: HttpClientFactory) -> Self {
        self.local_bootstrap_factory = Some(factory);
        self
    }

    pub(crate) fn authentication_factory(&self, endpoint: &str) -> HttpClientFactory {
        if self.local_bootstrap_factory.is_none() {
            return self.http_client_factory.clone();
        }
        self.http_client_factory_for_bootstrap(endpoint.parse::<url::Url>())
    }

    /// Uses local administrator network rules when available, limited to the supplied endpoints.
    /// Fetched policy must not block the sign-in and configuration requests needed to refresh it.
    pub fn http_client_factory_for_bootstrap(
        &self,
        endpoints: impl IntoIterator<Item = url::Url>,
    ) -> HttpClientFactory {
        let factory = self
            .local_bootstrap_factory
            .as_ref()
            .unwrap_or(&self.http_client_factory);
        factory.clone().with_network_policy(
            factory
                .network_policy()
                .clone()
                .restrict_to_endpoints(endpoints.into_iter().collect()),
        )
    }

    /// Returns the HTTP client factory represented by this routing configuration.
    pub fn http_client_factory(&self) -> &HttpClientFactory {
        &self.http_client_factory
    }

    pub(crate) fn application_network_policy(&self) -> &NetworkPolicy {
        &self.application_network_policy
    }
}
