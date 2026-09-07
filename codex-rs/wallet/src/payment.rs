use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use base64::engine::general_purpose::URL_SAFE;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::TryRngCore;
use rand::rngs::OsRng;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_hash::Hash;
use solana_keypair::Keypair;
use solana_message::VersionedMessage;
use solana_message::v0::Message as MessageV0;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;
use solana_transaction::Instruction;
use solana_transaction::versioned::VersionedTransaction;
use thiserror::Error;

use crate::Network;
use crate::UnlockedWallet;
use crate::solana_usdc_mint;

const SOLANA_MAINNET: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
const SOLANA_DEVNET: &str = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";
const ATA_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const MEMO_PROGRAM: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
const COMPUTE_UNIT_LIMIT: u32 = 20_000;
const NETWORK_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const PAYMENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentIntent {
    pub payment_url: String,
    pub gateway_origin: String,
    pub network: String,
    pub rpc_url: String,
    pub asset: String,
    pub amount_atomic: String,
    pub pay_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentReceipt {
    pub status: u16,
    pub payment_response: Option<String>,
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanPurchaseIntent {
    pub gateway_origin: String,
    pub plan_id: String,
    pub network: String,
    pub rpc_url: String,
    pub asset: String,
    pub amount_atomic: String,
    pub pay_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvisionedPlan {
    pub plan_id: String,
    pub key_id: String,
    pub api_key: String,
    pub display_prefix: String,
    /// Public Solana settlement signature returned by the x402 facilitator.
    /// This is safe to show as a payment receipt; the encoded payment response is not.
    pub transaction: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayKey {
    pub key_id: String,
    pub api_key: String,
    pub display_prefix: String,
}

impl std::fmt::Debug for GatewayKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GatewayKey")
            .field("key_id", &self.key_id)
            .field("api_key", &"[REDACTED]")
            .field("display_prefix", &self.display_prefix)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum X402PaymentError {
    #[error("payment intent was refused: {0}")]
    Intent(String),
    #[error("payment transport failed: {0}")]
    Transport(String),
    #[error("payment endpoint returned HTTP {status}: {message}")]
    Rejected { status: u16, message: String },
}

pub(crate) async fn pay(
    wallet: &UnlockedWallet,
    intent: PaymentIntent,
) -> Result<PaymentReceipt, X402PaymentError> {
    validate_intent(wallet, &intent)?;
    let client = secure_client(&intent.gateway_origin)?;
    let challenge_response = client
        .post(&intent.payment_url)
        .header("Accept", "application/json")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(transport)?;
    if challenge_response.status().as_u16() != 402 {
        return response_result(challenge_response).await;
    }
    let encoded = challenge_response
        .headers()
        .get("payment-required")
        .or_else(|| challenge_response.headers().get("x-payment-required"))
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| invalid("payment challenge header is missing"))?;
    let challenge_bytes = decode_x402_header(encoded)?;
    let challenge: PaymentRequired = serde_json::from_slice(&challenge_bytes)
        .map_err(|_| invalid("payment challenge is malformed"))?;
    if challenge.x402_version != 2 {
        return Err(invalid("only x402 version 2 is supported"));
    }
    let accepted = challenge
        .accepts
        .iter()
        .find(|candidate| candidate.matches(&intent))
        .ok_or_else(|| invalid("gateway offered no exact match for the confirmed payment"))?;
    accepted.validate_fee_payer(wallet)?;
    let transaction = build_transaction(wallet, &intent, accepted).await?;
    let payment = PaymentPayload {
        x402_version: 2,
        resource: challenge.resource,
        accepted: accepted.clone(),
        payload: TransactionPayload { transaction },
        extensions: challenge.extensions,
    };
    let signature = STANDARD.encode(serde_json::to_vec(&payment).map_err(transport)?);
    let response = client
        .post(&intent.payment_url)
        .header("Accept", "application/json")
        .header("Payment-Signature", signature)
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(transport)?;
    response_result(response).await
}

pub(crate) async fn provision_plan(
    wallet: &UnlockedWallet,
    intent: PlanPurchaseIntent,
) -> Result<ProvisionedPlan, X402PaymentError> {
    if !matches!(
        intent.plan_id.as_str(),
        "starter" | "basic" | "power" | "pro"
    ) {
        return Err(invalid("plan is not supported"));
    }
    let payment_url = format!(
        "{}/v1/subscriptions/{}",
        intent.gateway_origin.trim_end_matches('/'),
        intent.plan_id
    );
    let payment = pay(
        wallet,
        PaymentIntent {
            payment_url,
            gateway_origin: intent.gateway_origin.clone(),
            network: intent.network,
            rpc_url: intent.rpc_url,
            asset: intent.asset,
            amount_atomic: intent.amount_atomic,
            pay_to: intent.pay_to,
        },
    )
    .await?;
    let transaction = payment_transaction(&payment);

    let gateway_key = issue_gateway_key(wallet, intent.gateway_origin).await?;
    Ok(ProvisionedPlan {
        plan_id: intent.plan_id,
        key_id: gateway_key.key_id,
        api_key: gateway_key.api_key,
        display_prefix: gateway_key.display_prefix,
        transaction,
    })
}

pub(crate) fn payment_transaction(receipt: &PaymentReceipt) -> Option<String> {
    receipt
        .payment_response
        .as_deref()
        .and_then(decode_header_json)
        .and_then(|response| {
            response
                .get("transaction")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
        })
        .or_else(|| {
            receipt
                .body
                .get("transaction")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
        })
}

pub(crate) async fn issue_gateway_key(
    wallet: &UnlockedWallet,
    gateway_origin: String,
) -> Result<GatewayKey, X402PaymentError> {
    validate_gateway_origin(&gateway_origin)?;
    let client = secure_client(&gateway_origin)?;
    let challenge_response = client
        .post(format!(
            "{}/v1/keys/challenge",
            gateway_origin.trim_end_matches('/')
        ))
        .json(&serde_json::json!({ "walletAddress": wallet.manifest().address }))
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
    let challenge = challenge_body
        .get("challenge")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("gateway returned no ownership challenge"))?;
    let signature = wallet.sign_ownership_challenge(&gateway_origin, challenge);
    let key_response = client
        .post(format!(
            "{}/v1/keys/wallet",
            gateway_origin.trim_end_matches('/')
        ))
        .json(&serde_json::json!({
            "walletAddress": wallet.manifest().address,
            "challenge": challenge,
            "signature": signature,
        }))
        .send()
        .await
        .map_err(transport)?;
    let key_status = key_response.status().as_u16();
    let key_body = key_response
        .json::<serde_json::Value>()
        .await
        .map_err(transport)?;
    if !(200..300).contains(&key_status) {
        return Err(rejected(key_status, key_body));
    }
    Ok(GatewayKey {
        key_id: required_string(&key_body, "id")?,
        api_key: required_string(&key_body, "key")?,
        display_prefix: required_string(&key_body, "displayPrefix")?,
    })
}

async fn build_transaction(
    wallet: &UnlockedWallet,
    intent: &PaymentIntent,
    requirement: &PaymentRequirement,
) -> Result<String, X402PaymentError> {
    let signer = Keypair::new_from_array(*wallet.seed_for_payment());
    let owner = signer.pubkey();
    let mint = pubkey(&intent.asset, "asset")?;
    let receiver = pubkey(&intent.pay_to, "receiver")?;
    let fee_payer = pubkey(requirement.sponsored_fee_payer()?, "fee payer")?;
    let token_program = spl_token::id();
    let ata_program = pubkey(ATA_PROGRAM, "associated token program")?;
    let source = Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    )
    .0;
    let destination = Pubkey::find_program_address(
        &[receiver.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    )
    .0;
    let amount = intent
        .amount_atomic
        .parse::<u64>()
        .map_err(|_| invalid("invalid amount"))?;
    let transfer = spl_token::instruction::transfer_checked(
        &token_program,
        &source,
        &mint,
        &destination,
        &owner,
        &[],
        amount,
        6,
    )
    .map_err(transport)?;
    let memo_data = match requirement.extra.memo.as_deref() {
        Some(value) if value.len() <= 256 => value.as_bytes().to_vec(),
        Some(_) => return Err(invalid("payment memo is too long")),
        None => {
            let mut bytes = [0_u8; 16];
            OsRng.try_fill_bytes(&mut bytes).map_err(transport)?;
            let encoded = bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            encoded.into_bytes()
        }
    };
    let memo =
        Instruction::new_with_bytes(pubkey(MEMO_PROGRAM, "memo program")?, &memo_data, vec![]);
    let blockhash = latest_blockhash(&intent.rpc_url).await?;
    let instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(COMPUTE_UNIT_LIMIT),
        ComputeBudgetInstruction::set_compute_unit_price(1),
        transfer,
        memo,
    ];
    let message =
        MessageV0::try_compile(&fee_payer, &instructions, &[], blockhash).map_err(transport)?;
    let versioned = VersionedMessage::V0(message);
    let required = versioned.header().num_required_signatures as usize;
    let owner_index = versioned
        .static_account_keys()
        .iter()
        .take(required)
        .position(|key| key == &owner)
        .ok_or_else(|| invalid("wallet signer is absent from the payment transaction"))?;
    let signature = signer
        .try_sign_message(&versioned.serialize())
        .map_err(transport)?;
    let mut signatures = vec![Signature::default(); required];
    signatures[owner_index] = signature;
    let transaction = VersionedTransaction {
        signatures,
        message: versioned,
    };
    Ok(STANDARD.encode(bincode::serialize(&transaction).map_err(transport)?))
}

async fn latest_blockhash(rpc_url: &str) -> Result<Hash, X402PaymentError> {
    let response = Client::builder()
        .connect_timeout(NETWORK_CONNECT_TIMEOUT)
        .timeout(RPC_REQUEST_TIMEOUT)
        .build()
        .map_err(transport)?
        .post(rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": [{ "commitment": "confirmed" }],
        }))
        .send()
        .await
        .map_err(transport)?;
    let status = response.status().as_u16();
    let body = response
        .json::<serde_json::Value>()
        .await
        .map_err(transport)?;
    if !(200..300).contains(&status) {
        return Err(rejected(status, body));
    }
    let value = body
        .pointer("/result/value/blockhash")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("Solana RPC omitted the recent blockhash"))?;
    Hash::from_str(value).map_err(transport)
}

async fn response_result(response: reqwest::Response) -> Result<PaymentReceipt, X402PaymentError> {
    let status = response.status().as_u16();
    let payment_response = response
        .headers()
        .get("payment-response")
        .or_else(|| response.headers().get("x-payment-response"))
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let payment_required = response
        .headers()
        .get("payment-required")
        .or_else(|| response.headers().get("x-payment-required"))
        .and_then(|value| value.to_str().ok())
        .and_then(decode_header_json);
    let body = response
        .json::<serde_json::Value>()
        .await
        .unwrap_or(serde_json::Value::Null);
    if !(200..300).contains(&status) {
        let payment_result = payment_response.as_deref().and_then(decode_header_json);
        let details = if payment_result.is_some() || payment_required.is_some() {
            serde_json::json!({
                "body": body,
                "paymentResponse": payment_result,
                "paymentRequired": payment_required,
            })
        } else {
            body
        };
        return Err(rejected(status, details));
    }
    Ok(PaymentReceipt {
        status,
        payment_response,
        body,
    })
}

fn validate_intent(
    wallet: &UnlockedWallet,
    intent: &PaymentIntent,
) -> Result<(), X402PaymentError> {
    validate_gateway_origin(&intent.gateway_origin)?;
    let gateway = reqwest::Url::parse(&intent.gateway_origin)
        .map_err(|_| invalid("gateway origin is not a URL"))?;
    let payment = reqwest::Url::parse(&intent.payment_url)
        .map_err(|_| invalid("payment URL is not a URL"))?;
    if gateway.origin() != payment.origin() {
        return Err(invalid(
            "payment URL does not belong to the approved gateway origin",
        ));
    }
    let expected_network = match wallet.manifest().network {
        Network::Mainnet => SOLANA_MAINNET,
        Network::Devnet => SOLANA_DEVNET,
    };
    if intent.network != expected_network {
        return Err(invalid("payment network does not match the wallet network"));
    }
    if intent.asset != solana_usdc_mint(wallet.manifest().network) {
        return Err(invalid("payment asset is not canonical Solana USDC"));
    }
    if intent.amount_atomic.parse::<u64>().is_err() || intent.amount_atomic == "0" {
        return Err(invalid("payment amount is invalid"));
    }
    pubkey(&intent.pay_to, "receiver")?;
    Ok(())
}

pub fn validate_gateway_origin(value: &str) -> Result<(), X402PaymentError> {
    let gateway = reqwest::Url::parse(value).map_err(|_| invalid("gateway origin is not a URL"))?;
    if gateway.path() != "/" || gateway.query().is_some() || gateway.fragment().is_some() {
        return Err(invalid(
            "gateway origin must not contain a path, query, or fragment",
        ));
    }
    if gateway.scheme() != "https" && !is_loopback_origin(value) {
        return Err(invalid("remote payment gateways must use HTTPS"));
    }
    Ok(())
}

pub(crate) fn secure_client(origin: &str) -> Result<Client, X402PaymentError> {
    Client::builder()
        .https_only(!is_loopback_origin(origin))
        .connect_timeout(NETWORK_CONNECT_TIMEOUT)
        .timeout(PAYMENT_REQUEST_TIMEOUT)
        .build()
        .map_err(transport)
}

fn is_loopback_origin(value: &str) -> bool {
    reqwest::Url::parse(value)
        .is_ok_and(|url| matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")))
}

fn pubkey(value: &str, label: &str) -> Result<Pubkey, X402PaymentError> {
    Pubkey::from_str(value).map_err(|_| invalid(&format!("{label} is not a Solana address")))
}

pub(crate) fn required_string(
    body: &serde_json::Value,
    field: &str,
) -> Result<String, X402PaymentError> {
    body.get(field)
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid(&format!("gateway response omitted {field}")))
}

fn decode_x402_header(value: &str) -> Result<Vec<u8>, X402PaymentError> {
    STANDARD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value))
        .map_err(|_| invalid("payment challenge is not valid base64"))
}

fn decode_header_json(value: &str) -> Option<serde_json::Value> {
    let bytes = STANDARD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value))
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(crate) fn invalid(message: &str) -> X402PaymentError {
    X402PaymentError::Intent(message.to_string())
}
pub(crate) fn transport(error: impl std::fmt::Display) -> X402PaymentError {
    X402PaymentError::Transport(error.to_string())
}
pub(crate) fn rejected(status: u16, body: serde_json::Value) -> X402PaymentError {
    X402PaymentError::Rejected {
        status,
        message: body.to_string(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaymentRequired {
    x402_version: u8,
    resource: Option<serde_json::Value>,
    accepts: Vec<PaymentRequirement>,
    #[serde(default)]
    extensions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaymentRequirement {
    scheme: String,
    network: String,
    amount: String,
    asset: String,
    pay_to: String,
    max_timeout_seconds: u64,
    extra: PaymentExtra,
}

impl PaymentRequirement {
    fn matches(&self, intent: &PaymentIntent) -> bool {
        self.scheme == "exact"
            && self.network == intent.network
            && self.amount == intent.amount_atomic
            && self.asset == intent.asset
            && self.pay_to == intent.pay_to
            && self.max_timeout_seconds > 0
            && self
                .extra
                .fee_payer
                .as_deref()
                .is_some_and(|fee_payer| !fee_payer.is_empty())
    }

    fn validate_fee_payer(&self, wallet: &UnlockedWallet) -> Result<(), X402PaymentError> {
        let fee_payer = pubkey(self.sponsored_fee_payer()?, "fee payer")?;
        let owner = pubkey(&wallet.manifest().address, "wallet owner")?;
        if fee_payer == owner {
            return Err(invalid(
                "payment fee payer must be sponsored and cannot be the wallet owner",
            ));
        }
        Ok(())
    }

    fn sponsored_fee_payer(&self) -> Result<&str, X402PaymentError> {
        self.extra
            .fee_payer
            .as_deref()
            .filter(|fee_payer| !fee_payer.is_empty())
            .ok_or_else(|| invalid("payment requirement has no sponsored fee payer"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaymentExtra {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fee_payer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    memo: Option<String>,
    #[serde(flatten)]
    fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentPayload {
    x402_version: u8,
    resource: Option<serde_json::Value>,
    accepted: PaymentRequirement,
    payload: TransactionPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct TransactionPayload {
    transaction: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unlocked_mainnet_wallet() -> (tempfile::TempDir, UnlockedWallet) {
        let home = tempfile::tempdir().expect("tempdir");
        let wallet = crate::Wallet::new(home.path().to_path_buf());
        wallet
            .create(
                "a sufficiently long payment test passphrase",
                Network::Mainnet,
            )
            .expect("create wallet");
        let unlocked = wallet
            .unlock("a sufficiently long payment test passphrase")
            .expect("unlock wallet");
        (home, unlocked)
    }

    fn valid_intent() -> PaymentIntent {
        PaymentIntent {
            payment_url: "http://127.0.0.1:4021/v1/subscriptions/starter".to_string(),
            gateway_origin: "http://127.0.0.1:4021".to_string(),
            network: SOLANA_MAINNET.to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            asset: crate::SOLANA_MAINNET_USDC_MINT.to_string(),
            amount_atomic: "1000000".to_string(),
            pay_to: "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd".to_string(),
        }
    }

    fn valid_requirement() -> PaymentRequirement {
        PaymentRequirement {
            scheme: "exact".to_string(),
            network: SOLANA_MAINNET.to_string(),
            amount: "1000000".to_string(),
            asset: crate::SOLANA_MAINNET_USDC_MINT.to_string(),
            pay_to: "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd".to_string(),
            max_timeout_seconds: 300,
            extra: PaymentExtra {
                fee_payer: Some("2wKupLR9q6wXYppw8Gr2NvWxKBUqm4PPJKkQfoxHDBg4".to_string()),
                memo: None,
                fields: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn payment_intent_tampering_is_rejected_before_network_or_signing() {
        let (_home, wallet) = unlocked_mainnet_wallet();
        let valid = valid_intent();
        validate_intent(&wallet, &valid).expect("valid confirmed intent");

        let mut cases = Vec::new();
        let mut changed = valid.clone();
        changed.gateway_origin = "http://payments.example.com".to_string();
        changed.payment_url = "http://payments.example.com/v1/subscriptions/starter".to_string();
        cases.push(("insecure remote origin", changed));
        let mut changed = valid.clone();
        changed.payment_url = "https://attacker.example/v1/subscriptions/starter".to_string();
        cases.push(("cross-origin payment URL", changed));
        let mut changed = valid.clone();
        changed.network = SOLANA_DEVNET.to_string();
        cases.push(("network", changed));
        let mut changed = valid.clone();
        changed.asset = crate::SOLANA_DEVNET_USDC_MINT.to_string();
        cases.push(("asset mint", changed));
        let mut changed = valid.clone();
        changed.amount_atomic = "0".to_string();
        cases.push(("zero amount", changed));
        let mut changed = valid.clone();
        changed.amount_atomic = "not-a-number".to_string();
        cases.push(("malformed amount", changed));
        let mut changed = valid;
        changed.pay_to = "not-a-solana-address".to_string();
        cases.push(("recipient", changed));

        for (field, tampered) in cases {
            assert!(
                validate_intent(&wallet, &tampered).is_err(),
                "tampered {field} must fail before signing"
            );
        }
    }

    #[test]
    fn x402_requirement_tampering_never_matches_the_confirmed_intent() {
        let intent = valid_intent();
        let valid = valid_requirement();
        assert!(valid.matches(&intent));

        let mut cases = Vec::new();
        let mut changed = valid.clone();
        changed.scheme = "upto".to_string();
        cases.push(("scheme", changed));
        let mut changed = valid.clone();
        changed.network = SOLANA_DEVNET.to_string();
        cases.push(("network", changed));
        let mut changed = valid.clone();
        changed.amount = "2000000".to_string();
        cases.push(("amount", changed));
        let mut changed = valid.clone();
        changed.asset = crate::SOLANA_DEVNET_USDC_MINT.to_string();
        cases.push(("asset mint", changed));
        let mut changed = valid.clone();
        changed.pay_to = "2wKupLR9q6wXYppw8Gr2NvWxKBUqm4PPJKkQfoxHDBg4".to_string();
        cases.push(("recipient", changed));
        let mut changed = valid.clone();
        changed.max_timeout_seconds = 0;
        cases.push(("timeout", changed));
        let mut changed = valid;
        changed.extra.fee_payer = None;
        cases.push(("sponsor", changed));

        for (field, tampered) in cases {
            assert!(
                !tampered.matches(&intent),
                "tampered x402 {field} must not match the confirmed intent"
            );
        }
    }

    #[test]
    fn v2_requirement_round_trip_preserves_matching_timeout() {
        let value = serde_json::json!({
            "scheme": "exact",
            "network": SOLANA_MAINNET,
            "amount": "1000000",
            "asset": crate::SOLANA_MAINNET_USDC_MINT,
            "payTo": "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd",
            "maxTimeoutSeconds": 300,
            "extra": { "feePayer": "2wKupLR9q6wXYppw8Gr2NvWxKBUqm4PPJKkQfoxHDBg4" }
        });
        let requirement: PaymentRequirement =
            serde_json::from_value(value.clone()).expect("valid requirement");
        assert_eq!(
            serde_json::to_value(&requirement).expect("serialize requirement"),
            value
        );
        let payload = PaymentPayload {
            x402_version: 2,
            resource: None,
            accepted: requirement,
            payload: TransactionPayload {
                transaction: "encoded".to_string(),
            },
            extensions: None,
        };
        assert!(
            serde_json::to_value(payload)
                .expect("serialize payload")
                .get("extensions")
                .is_none()
        );
    }

    #[test]
    fn heterogeneous_chain_offers_preserve_fields_and_select_exact_solana_payment() {
        let value = serde_json::json!({
            "x402Version": 2,
            "resource": {
                "url": "https://gateway.example/v1/topups?intent=top-up-1",
                "description": "Fund a dollar balance",
                "mimeType": "application/json"
            },
            "accepts": [
                {
                    "scheme": "exact",
                    "network": "eip155:8453",
                    "amount": "1000000",
                    "asset": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                    "payTo": "0x1455Bd7FBfBF92a171eF36025E13959E3b0ad8c0",
                    "maxTimeoutSeconds": 300,
                    "extra": { "name": "USD Coin", "version": "2" }
                },
                {
                    "scheme": "exact",
                    "network": SOLANA_MAINNET,
                    "amount": "1000000",
                    "asset": crate::SOLANA_MAINNET_USDC_MINT,
                    "payTo": "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd",
                    "maxTimeoutSeconds": 300,
                    "extra": {
                        "feePayer": "2wKupLR9q6wXYppw8Gr2NvWxKBUqm4PPJKkQfoxHDBg4"
                    }
                }
            ],
            "error": "Payment required"
        });
        let challenge: PaymentRequired =
            serde_json::from_value(value).expect("heterogeneous x402 challenge");
        let intent = valid_intent();
        let selected = challenge
            .accepts
            .iter()
            .find(|requirement| requirement.matches(&intent))
            .expect("exact Solana payment offer");

        assert_eq!(selected.network, SOLANA_MAINNET);
        assert_eq!(
            serde_json::to_value(&challenge.accepts[0]).expect("serialize EVM alternative"),
            serde_json::json!({
                "scheme": "exact",
                "network": "eip155:8453",
                "amount": "1000000",
                "asset": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                "payTo": "0x1455Bd7FBfBF92a171eF36025E13959E3b0ad8c0",
                "maxTimeoutSeconds": 300,
                "extra": { "name": "USD Coin", "version": "2" }
            })
        );
    }

    #[test]
    fn extracts_public_transaction_from_x402_receipt_without_exposing_header() {
        let response = STANDARD.encode(
            serde_json::to_vec(&serde_json::json!({
                "success": true,
                "transaction": "5EttlementSignature",
                "payer": "wallet",
            }))
            .expect("json"),
        );
        let receipt = PaymentReceipt {
            status: 200,
            payment_response: Some(response),
            body: serde_json::Value::Null,
        };

        assert_eq!(
            payment_transaction(&receipt).as_deref(),
            Some("5EttlementSignature")
        );
    }

    #[test]
    fn rejects_a_challenge_that_makes_the_wallet_owner_pay_network_fees() {
        let home = tempfile::tempdir().expect("tempdir");
        let wallet = crate::Wallet::new(home.path().to_path_buf());
        let created = wallet
            .create(
                "a sufficiently long payment test passphrase",
                Network::Mainnet,
            )
            .expect("create wallet");
        let unlocked = wallet
            .unlock("a sufficiently long payment test passphrase")
            .expect("unlock wallet");
        let requirement = PaymentRequirement {
            scheme: "exact".to_string(),
            network: SOLANA_MAINNET.to_string(),
            amount: "1000000".to_string(),
            asset: crate::SOLANA_MAINNET_USDC_MINT.to_string(),
            pay_to: "G3s13pAE8f72jPPWSvwEfLr6Gg1WRh6Nv7i98HNMoVcd".to_string(),
            max_timeout_seconds: 300,
            extra: PaymentExtra {
                fee_payer: Some(created.manifest.address),
                memo: None,
                fields: BTreeMap::new(),
            },
        };

        let error = requirement
            .validate_fee_payer(&unlocked)
            .expect_err("the wallet must not silently become the fee payer");
        assert!(error.to_string().contains("must be sponsored"));
    }
}
