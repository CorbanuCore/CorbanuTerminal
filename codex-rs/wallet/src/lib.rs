//! Passcode-gated local Solana wallet storage and narrowly scoped signing.

mod balance;
mod envelope;
mod payment;

pub use balance::BalanceClient;
pub use balance::WalletBalances;
pub use envelope::CreatedWallet;
pub use envelope::Network;
pub use envelope::PublicManifest;
pub use envelope::UnlockedWallet;
pub use envelope::Wallet;
pub use envelope::WalletError;
pub use payment::GatewayKey;
pub use payment::PaymentIntent;
pub use payment::PaymentReceipt;
pub use payment::PlanPurchaseIntent;
pub use payment::ProvisionedPlan;
pub use payment::X402PaymentError;

pub const SOLANA_USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
