//! PF-20 local controller checkpoints. Neither a checkpoint value nor a native
//! client is protected-mode authorization. PF-27 must separately qualify the
//! actual controller/worker launch and containment boundary.
//!
//! Restoring agent-accessible data cannot reset the independent controller
//! root. Restoring the entire trusted machine is explicitly outside this model.

mod checkpoint;
mod error;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
mod native;
#[cfg(target_os = "linux")]
mod store;

pub use checkpoint::PolicyCheckpoint;
pub use error::RootError;
#[cfg(target_os = "linux")]
pub use native::NativeAnchorClient;
#[cfg(target_os = "linux")]
pub use store::ControllerRoot;
#[cfg(target_os = "linux")]
pub use store::Enrollment;

/// PF-20's existing policy-anchor operations, independent of Core's private
/// adapter type. Implementations supply exact durable CAS for a previously
/// authenticated namespace; implementing this trait does not confer authority.
pub trait PolicyRootStore: std::fmt::Debug + Send + Sync {
    fn load_policy(&self) -> Result<Option<PolicyCheckpoint>, RootError>;
    fn compare_policy(&self, expected: Option<&PolicyCheckpoint>, next: &PolicyCheckpoint) -> Result<(), RootError>;
}
