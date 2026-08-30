use codex_wallet::CorbanuApiOperation;
use codex_wallet::CorbanuApiOperationResult;
use codex_wallet::GatewayKey;
use codex_wallet::PlanPurchaseIntent;
use codex_wallet::ProvisionedPlan;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnlockPolicy {
    OneAction,
    Timed { duration_seconds: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Request {
    Ping,
    Status,
    Unlock {
        passcode: String,
        duration_seconds: u64,
        #[serde(default)]
        one_action: bool,
    },
    Lock,
    RemoveWallet {
        expected_address: String,
    },
    SignOwnership {
        capability: String,
        gateway_origin: String,
        challenge: String,
    },
    ProvisionPlan {
        capability: String,
        intent: PlanPurchaseIntent,
    },
    IssueGatewayKey {
        capability: String,
        gateway_origin: String,
    },
    CorbanuApiOperation {
        capability: String,
        gateway_origin: String,
        operation: CorbanuApiOperation,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum Response {
    Pong,
    Status(DaemonStatus),
    Unlocked {
        capability: String,
        expires_in_seconds: u64,
    },
    Locked,
    WalletRemoved,
    Signature {
        signature: String,
    },
    PlanProvisioned(ProvisionedPlan),
    GatewayKeyIssued(GatewayKey),
    CorbanuApiOperationCompleted(CorbanuApiOperationResult),
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonStatus {
    pub wallet_exists: bool,
    pub address: Option<String>,
    pub network: Option<String>,
    pub locked: bool,
    #[serde(default)]
    pub busy: bool,
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum WalletDaemonError {
    #[error("wallet daemon is unavailable: {0}")]
    Unavailable(String),
    #[error("wallet daemon refused the request ({code}): {message}")]
    Refused { code: String, message: String },
    #[error("wallet daemon returned an unexpected response")]
    Protocol,
}

#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
