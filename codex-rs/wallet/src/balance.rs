use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

use crate::SOLANA_USDC_MINT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletBalances {
    pub sol_lamports: u64,
    pub usdc_atomic: u64,
}

#[derive(Clone)]
pub struct BalanceClient {
    rpc_url: String,
    client: reqwest::Client,
}

impl BalanceClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(15))
                .build()
                .expect("wallet balance HTTP client configuration is static"),
        }
    }

    pub async fn balances(&self, address: &str) -> Result<WalletBalances> {
        let sol: RpcResponse<u64> = self
            .rpc("getBalance", json!([address, {"commitment":"confirmed"}]))
            .await?;
        let tokens: RpcResponse<TokenAccounts> = self.rpc(
            "getTokenAccountsByOwner",
            json!([address,{"mint":SOLANA_USDC_MINT},{"encoding":"jsonParsed","commitment":"confirmed"}]),
        ).await?;
        let usdc_atomic = tokens.value.into_iter().try_fold(0_u64, |sum, account| {
            let amount = account
                .account
                .data
                .parsed
                .info
                .token_amount
                .amount
                .parse::<u64>()
                .context("invalid USDC amount from RPC")?;
            sum.checked_add(amount).context("USDC balance overflow")
        })?;
        Ok(WalletBalances {
            sol_lamports: sol.value,
            usdc_atomic,
        })
    }

    async fn rpc<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .send()
            .await?
            .error_for_status()?;
        let body: RpcEnvelope<T> = response.json().await?;
        body.result.context("Solana RPC returned no result")
    }
}

#[derive(Deserialize)]
struct RpcEnvelope<T> {
    result: Option<T>,
}
#[derive(Deserialize)]
struct RpcResponse<T> {
    value: T,
}
type TokenAccounts = Vec<TokenAccount>;
#[derive(Deserialize)]
struct TokenAccount {
    account: TokenAccountBody,
}
#[derive(Deserialize)]
struct TokenAccountBody {
    data: ParsedTokenData,
}
#[derive(Deserialize)]
struct ParsedTokenData {
    parsed: ParsedToken,
}
#[derive(Deserialize)]
struct ParsedToken {
    info: TokenInfo,
}
#[derive(Deserialize)]
struct TokenInfo {
    #[serde(rename = "tokenAmount")]
    token_amount: TokenAmount,
}
#[derive(Deserialize)]
struct TokenAmount {
    amount: String,
}
