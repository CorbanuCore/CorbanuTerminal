//! Durable GPU rental control plane used by PFTerminal's `/gpu` product surface.

mod controller;
mod credential;
mod provider;
mod provider_http;
mod recipe;
mod runpod;
mod vast;

pub use controller::ControllerEvent;
pub use controller::GpuRentalController;
pub use controller::ReconcileConfig;
pub use credential::GpuCredential;
pub use credential::GpuCredentialError;
pub use credential::GpuCredentialKind;
pub use credential::GpuCredentialResolver;
pub use credential::SecretValue;
pub use credential::VaultGpuCredentialResolver;
pub use provider::BillingState;
pub use provider::CreateInstanceRequest;
pub use provider::GpuInstance;
pub use provider::GpuInstanceState;
pub use provider::GpuOffer;
pub use provider::GpuProvider;
pub use provider::HardwareRequirements;
pub use provider::OwnedInstanceQuery;
pub use provider::ProviderCapabilities;
pub use provider::ProviderError;
pub use provider::ProviderErrorKind;
pub use provider::ProviderResult;
pub use provider::SearchOffersRequest;
pub use recipe::GpuRecipe;
pub use recipe::RecipeCatalog;
pub use runpod::RunpodProvider;
pub use vast::VastProvider;
