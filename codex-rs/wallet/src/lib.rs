//! Passcode-gated local Solana wallet storage and narrowly scoped signing.

mod balance;
mod corbanu_api;
mod envelope;
mod payment;

pub use balance::BalanceClient;
pub use balance::WalletBalances;
pub use corbanu_api::CorbanuApiAccount;
pub use corbanu_api::CorbanuApiBalance;
pub use corbanu_api::CorbanuApiKeySummary;
pub use corbanu_api::CorbanuApiModel;
pub use corbanu_api::CorbanuApiOperation;
pub use corbanu_api::CorbanuApiOperationResult;
pub use corbanu_api::CorbanuApiPricing;
pub use envelope::CreatedWallet;
pub use envelope::Network;
pub use envelope::PublicManifest;
pub use envelope::RecoveryBackup;
pub use envelope::UnlockedWallet;
pub use envelope::Wallet;
pub use envelope::WalletError;
pub use payment::GatewayKey;
pub use payment::PaymentIntent;
pub use payment::PaymentReceipt;
pub use payment::PlanPurchaseIntent;
pub use payment::ProvisionedPlan;
pub use payment::X402PaymentError;
pub use payment::validate_gateway_origin;

pub const SOLANA_MAINNET_USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const SOLANA_DEVNET_USDC_MINT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

pub fn solana_usdc_mint(network: Network) -> &'static str {
    match network {
        Network::Mainnet => SOLANA_MAINNET_USDC_MINT,
        Network::Devnet => SOLANA_DEVNET_USDC_MINT,
    }
}
