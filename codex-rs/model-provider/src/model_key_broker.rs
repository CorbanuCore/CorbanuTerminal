//! PF-27-S05: Core's model-provider credentials held by the isolated
//! credential broker (feature `broker_model_auth`).
//!
//! Core installs one [`ModelKeyBroker`] per process. From then on every auth
//! this crate resolves for a plain provider key or sign-in token is a
//! brokered auth provider: it signs each request for the broker, which
//! attaches the credential. That covers the model client, web search, image
//! generation and the model catalog, which all resolve auth here.
//!
//! - Provider env-key providers are never read by Core: [`ConfiguredModelProvider`]
//!   hands out [`BROKERED_KEY_PLACEHOLDER`] instead of the key, and the broker
//!   uses the variable Core handed over at startup, else reads the stored key
//!   (encrypted vault, then the legacy file) itself.
//! - An OpenAI API-key login, `experimental_bearer_token` and a ChatGPT access
//!   token are values Core already holds; they are handed to the broker, and
//!   Core never attaches them to a request itself.
//! - Header-only uses of a first-party login ([`crate::auth_provider_from_auth`]:
//!   MCP uploads, plugins, skills, analytics, backend clients) cannot carry a
//!   signed frame, so they get no credential.
//!
//! Not brokered: agent identity, header and command auth, AWS auth.
//!
//! [`ConfiguredModelProvider`]: crate::create_model_provider

use crate::auth::ProviderApiKey;
use crate::auth::ProviderApiKeyHeader;
use codex_api::SharedAuthProvider;
use http::HeaderMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// What `CodexAuth::ApiKey` holds for a provider key Core did not read. It is
/// never sent: requests for such a provider are brokered.
pub const BROKERED_KEY_PLACEHOLDER: &str = "corbanu-brokered-provider-key";

/// Where a brokered credential comes from.
pub enum BrokeredKeySource {
    /// A provider key Core does not read: the first of `env_vars` Core handed
    /// over, else the key stored for `provider_key_id`.
    ProviderKey {
        provider_key_id: String,
        env_vars: Vec<String>,
    },
    /// A value Core already holds. `slot` names what a refreshed value of the
    /// same credential replaces (a ChatGPT access token); `None` for keys.
    Value {
        key: ProviderApiKey,
        slot: Option<String>,
    },
}

impl std::fmt::Debug for BrokeredKeySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProviderKey {
                provider_key_id, ..
            } => f
                .debug_struct("ProviderKey")
                .field("provider_key_id", provider_key_id)
                .finish_non_exhaustive(),
            Self::Value { key, slot } => f
                .debug_struct("Value")
                .field("key", key)
                .field("slot", slot)
                .finish(),
        }
    }
}

/// One brokered credential use: its source, where it may be sent, and the
/// non-secret headers that go with it.
#[derive(Debug)]
pub struct BrokeredAuthRequest {
    /// The provider base URL; the credential is bound to its origin and path.
    pub base_url: String,
    pub header: ProviderApiKeyHeader,
    pub source: BrokeredKeySource,
    /// Sent with every request (for example `ChatGPT-Account-ID`).
    pub extra_headers: HeaderMap,
}

/// Turns a credential use into auth that never attaches the credential in
/// Core. Problems (no broker, an unbindable URL, no stored key) are errors
/// (`CodexErr::Fatal`, not retried); nothing falls back to sending the
/// credential directly.
pub trait ModelKeyBroker: Send + Sync {
    fn auth(
        &self,
        request: BrokeredAuthRequest,
    ) -> codex_protocol::error::Result<SharedAuthProvider>;
}

static MODEL_KEY_BROKER: OnceLock<Arc<dyn ModelKeyBroker>> = OnceLock::new();
static REQUIRED: AtomicBool = AtomicBool::new(false);

/// The error for a credential use before the process's broker is running.
pub const BROKER_NOT_RUNNING: &str = "broker_model_auth: the credential broker is not running yet; \
     the credential is not sent until it is";

/// Marks this process as brokered (a loaded configuration enables
/// `broker_model_auth`). One-way and process-wide: from now on Core reads no
/// provider key and attaches no plain key or sign-in token itself, and until
/// a broker is installed such credential uses fail.
pub fn require_model_key_broker() {
    REQUIRED.store(true, Ordering::SeqCst);
}

/// Whether provider credentials in this process are brokered (required, or a
/// broker is installed).
pub fn model_key_broker_required() -> bool {
    #[cfg(test)]
    if TEST_REQUIRED.with(std::cell::Cell::get) {
        return true;
    }
    REQUIRED.load(Ordering::SeqCst) || model_key_broker().is_some()
}

#[cfg(test)]
thread_local! {
    static TEST_REQUIRED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Marks the current test thread as brokered without a broker installed.
#[cfg(test)]
pub(crate) fn set_test_broker_required(required: bool) {
    TEST_REQUIRED.with(|slot| slot.set(required));
}

/// Installs the process's broker. The first installation wins and stays for
/// the life of the process; returns whether this call installed it.
pub fn install_model_key_broker(broker: Arc<dyn ModelKeyBroker>) -> bool {
    MODEL_KEY_BROKER.set(broker).is_ok()
}

/// Whether provider credentials in this process are brokered.
pub fn model_key_broker_installed() -> bool {
    model_key_broker().is_some()
}

pub(crate) fn model_key_broker() -> Option<Arc<dyn ModelKeyBroker>> {
    #[cfg(test)]
    if let Some(broker) = TEST_BROKER.with(|broker| broker.borrow().clone()) {
        return Some(broker);
    }
    MODEL_KEY_BROKER.get().cloned()
}

#[cfg(test)]
thread_local! {
    static TEST_BROKER: std::cell::RefCell<Option<Arc<dyn ModelKeyBroker>>> =
        const { std::cell::RefCell::new(None) };
}

/// Installs `broker` for the current test thread only.
#[cfg(test)]
pub(crate) fn set_test_model_key_broker(broker: Option<Arc<dyn ModelKeyBroker>>) {
    TEST_BROKER.with(|slot| *slot.borrow_mut() = broker);
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::BrokeredAuthRequest;
    use super::ModelKeyBroker;
    use codex_api::SharedAuthProvider;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Records each brokered use and returns auth that attaches nothing.
    #[derive(Default)]
    pub(crate) struct RecordingBroker {
        pub(crate) uses: Mutex<Vec<BrokeredAuthRequest>>,
    }

    impl ModelKeyBroker for RecordingBroker {
        fn auth(
            &self,
            request: BrokeredAuthRequest,
        ) -> codex_protocol::error::Result<SharedAuthProvider> {
            self.uses.lock().expect("uses").push(request);
            Ok(crate::unauthenticated_auth_provider())
        }
    }

    /// Runs `test` with a recording broker installed on this thread.
    pub(crate) fn with_recording_broker<T>(test: impl FnOnce(&Arc<RecordingBroker>) -> T) -> T {
        let broker = Arc::new(RecordingBroker::default());
        super::set_test_model_key_broker(Some(broker.clone()));
        let result = test(&broker);
        super::set_test_model_key_broker(None);
        result
    }
}
