#![forbid(unsafe_code)]

//! Platform-containment contracts for Corbanu's isolated credential broker.

pub mod ipc;
pub mod journal_adapter;
#[cfg(target_os = "linux")]
pub mod linux_transport;
pub mod platform_contract;
pub mod resolver;
mod resolver_types;

pub use ipc::BrokerBinding;
pub use ipc::BrokerChannelMac;
pub use ipc::BrokerOperation;
pub use ipc::CredentialReference;
pub use ipc::ObservedPeer;
pub use ipc::OpenAiResponsesOperation;
pub use ipc::SignedBrokerFrame;
pub use resolver::BrokerRuntime;
pub use resolver_types::BackendDispatchError;
pub use resolver_types::BrokerAuditError;
pub use resolver_types::BrokerAuditIntent;
pub use resolver_types::BrokerAuditResolution;
pub use resolver_types::BrokerCredentialGrant;
pub use resolver_types::BrokerDispatchError;
pub use resolver_types::BrokerRuntimeConfig;
pub use resolver_types::BrokerSessionHandle;
pub use resolver_types::CancellationFence;
pub use resolver_types::DurableBrokerAudit;
pub use resolver_types::TypedCredentialBackend;
pub use resolver_types::TypedOperationReceipt;

#[cfg(test)]
#[path = "platform_contract_fixture_tests.rs"]
mod platform_contract_fixture_tests;

#[cfg(test)]
#[path = "ipc_tests.rs"]
mod ipc_tests;

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod resolver_tests;
