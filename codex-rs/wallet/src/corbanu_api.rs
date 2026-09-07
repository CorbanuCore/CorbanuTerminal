//! Wallet-scoped Corbanu API account operations.
//!
//! The caller chooses a typed operation. This module obtains an operation-bound
//! gateway challenge, signs only that challenge, and performs the matching
//! account action. Seed material never leaves [`UnlockedWallet`].

use serde::Deserialize;
use serde::Serialize;

use crate::GatewayKey;
use crate::PaymentIntent;
use crate::UnlockedWallet;
use crate::X402PaymentError;
use crate::payment::invalid;
use crate::payment::pay;
use crate::payment::payment_transaction;
use crate::payment::rejected;
use crate::payment::required_string;
use crate::payment::secure_client;
use crate::payment::transport;
use crate::validate_gateway_origin;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CorbanuApiOperation {
    Account,
    TopUpIntent {
        #[serde(rename = "amountUsd")]
        amount_usd: String,
    },
    CreateKey,
    RevokeKey {
        #[serde(rename = "keyId")]
        key_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorbanuApiBalance {
    pub balance_microusd: String,
    pub reserved_microusd: String,
    pub available_microusd: String,
    pub balance_usd: String,
    pub reserved_usd: String,
    pub available_usd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorbanuApiKeySummary {
    pub id: String,
    pub display_prefix: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorbanuApiPricing {
    pub input_usd: String,
    pub output_usd: String,
    pub cache_read_usd: String,
    pub cache_write_usd: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorbanuApiModel {
    pub id: String,
    pub display_name: String,
    pub recommended: bool,
    pub balance_rate: String,
    pub privacy: String,
    pub pricing: CorbanuApiPricing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CorbanuApiAccount {
    pub balance: CorbanuApiBalance,
    pub keys: Vec<CorbanuApiKeySummary>,
    pub models: Vec<CorbanuApiModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CorbanuApiOperationResult {
    Account {
        account: CorbanuApiAccount,
    },
    TopUp {
        balance: CorbanuApiBalance,
        api_key: Option<GatewayKey>,
        transaction: Option<String>,
    },
    KeyCreated {
        api_key: GatewayKey,
    },
    KeyRevoked {
        key_id: String,
    },
}

pub(crate) async fn execute(
    wallet: &UnlockedWallet,
    gateway_origin: String,
    operation: CorbanuApiOperation,
) -> Result<CorbanuApiOperationResult, X402PaymentError> {
    validate_gateway_origin(&gateway_origin)?;
    let response = signed_wallet_operation(wallet, &gateway_origin, &operation).await?;
    match operation {
        CorbanuApiOperation::Account => Ok(CorbanuApiOperationResult::Account {
            account: serde_json::from_value(response).map_err(transport)?,
        }),
        CorbanuApiOperation::TopUpIntent { amount_usd } => {
            let intent: CorbanuApiTopUpIntent =
                serde_json::from_value(response).map_err(transport)?;
            let expected_amount = parse_usd_micros(&amount_usd)?;
            if intent.intent.amount_microusd != expected_amount {
                return Err(invalid("gateway changed the confirmed top-up amount"));
            }
            if intent.payment.method != "POST" {
                return Err(invalid("gateway returned an unsupported payment method"));
            }
            let receipt = pay(
                wallet,
                PaymentIntent {
                    payment_url: intent.payment.url,
                    gateway_origin: gateway_origin.clone(),
                    network: intent.payment.network,
                    rpc_url: intent.payment.rpc_url,
                    asset: intent.payment.asset,
                    amount_atomic: intent.intent.amount_microusd,
                    pay_to: intent.payment.pay_to,
                },
            )
            .await?;
            let transaction = payment_transaction(&receipt);
            let account =
                signed_wallet_operation(wallet, &gateway_origin, &CorbanuApiOperation::Account)
                    .await
                    .and_then(|response| {
                        serde_json::from_value::<CorbanuApiAccount>(response).map_err(transport)
                    })?;
            let api_key = if !needs_initial_api_key(&account) {
                None
            } else {
                Some(
                    signed_wallet_operation(
                        wallet,
                        &gateway_origin,
                        &CorbanuApiOperation::CreateKey,
                    )
                    .await
                    .and_then(|response| {
                        serde_json::from_value::<CreatedApiKey>(response)
                            .map(Into::into)
                            .map_err(transport)
                    })?,
                )
            };
            Ok(CorbanuApiOperationResult::TopUp {
                balance: account.balance,
                api_key,
                transaction,
            })
        }
        CorbanuApiOperation::CreateKey => Ok(CorbanuApiOperationResult::KeyCreated {
            api_key: serde_json::from_value::<CreatedApiKey>(response)
                .map(Into::into)
                .map_err(transport)?,
        }),
        CorbanuApiOperation::RevokeKey { key_id } => {
            let revoked: CorbanuApiRevoked = serde_json::from_value(response).map_err(transport)?;
            if !revoked.revoked {
                return Err(invalid("gateway did not revoke the selected API key"));
            }
            Ok(CorbanuApiOperationResult::KeyRevoked { key_id })
        }
    }
}

fn needs_initial_api_key(account: &CorbanuApiAccount) -> bool {
    !account.keys.iter().any(|key| key.revoked_at.is_none())
}

async fn signed_wallet_operation(
    wallet: &UnlockedWallet,
    gateway_origin: &str,
    operation: &CorbanuApiOperation,
) -> Result<serde_json::Value, X402PaymentError> {
    let client = secure_client(gateway_origin)?;
    let challenge_response = client
        .post(format!(
            "{}/v1/wallet/challenge",
            gateway_origin.trim_end_matches('/')
        ))
        .json(&serde_json::json!({
            "walletAddress": wallet.manifest().address,
            "operation": operation,
        }))
        .send()
        .await
        .map_err(transport)?;
    let challenge_status = challenge_response.status().as_u16();
    let challenge_body = challenge_response
        .json::<serde_json::Value>()
        .await
        .map_err(transport)?;
    if !(200..300).contains(&challenge_status) {
        return Err(rejected(challenge_status, challenge_body));
    }
    let challenge = required_string(&challenge_body, "challenge")?;
    let signature = wallet.sign_ownership_challenge(gateway_origin, &challenge);
    let execute_response = client
        .post(format!(
            "{}/v1/wallet/execute",
            gateway_origin.trim_end_matches('/')
        ))
        .json(&serde_json::json!({
            "walletAddress": wallet.manifest().address,
            "operation": operation,
            "challenge": challenge,
            "signature": signature,
        }))
        .send()
        .await
        .map_err(transport)?;
    let execute_status = execute_response.status().as_u16();
    let execute_body = execute_response
        .json::<serde_json::Value>()
        .await
        .map_err(transport)?;
    if !(200..300).contains(&execute_status) {
        return Err(rejected(execute_status, execute_body));
    }
    Ok(execute_body)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorbanuApiTopUpIntent {
    intent: CorbanuApiIntent,
    payment: CorbanuApiPayment,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorbanuApiIntent {
    amount_microusd: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorbanuApiPayment {
    method: String,
    url: String,
    network: String,
    asset: String,
    pay_to: String,
    rpc_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatedApiKey {
    id: String,
    key: String,
    display_prefix: String,
}

impl From<CreatedApiKey> for GatewayKey {
    fn from(value: CreatedApiKey) -> Self {
        Self {
            key_id: value.id,
            api_key: value.key,
            display_prefix: value.display_prefix,
        }
    }
}

#[derive(Deserialize)]
struct CorbanuApiRevoked {
    revoked: bool,
}

fn parse_usd_micros(value: &str) -> Result<String, X402PaymentError> {
    let (whole, fractional) = value
        .split_once('.')
        .map_or((value, ""), |(whole, fractional)| (whole, fractional));
    if whole.is_empty()
        || (value.contains('.') && fractional.is_empty())
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fractional.len() > 6
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid(
            "top-up amount must be a positive decimal with at most 6 places",
        ));
    }
    let whole = whole
        .parse::<u64>()
        .map_err(|_| invalid("top-up amount is too large"))?;
    let fractional = format!("{fractional:0<6}")
        .parse::<u64>()
        .map_err(|_| invalid("top-up amount is invalid"))?;
    let micros = whole
        .checked_mul(1_000_000)
        .and_then(|amount| amount.checked_add(fractional))
        .ok_or_else(|| invalid("top-up amount is too large"))?;
    if micros == 0 {
        return Err(invalid("top-up amount must be positive"));
    }
    Ok(micros.to_string())
}

#[cfg(test)]
#[path = "corbanu_api_tests.rs"]
mod tests;
