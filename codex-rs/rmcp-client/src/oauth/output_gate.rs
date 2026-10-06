//! PF-28-S02: while the secret output gate is armed, MCP OAuth tokens enter
//! it whenever a store loads or saves them (start-up, refresh, persist), so a
//! token refreshed after start is protected before any tool result that
//! reflects it is gated. Per server and field, the current value and the one
//! before it stay registered (the older may still be in flight); the one
//! before that is retired, which bounds the registry across refreshes.

use super::StoredOAuthTokens;
use anyhow::Result;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::SecretClass;
use codex_secret_broker::output_gate::SecretHandle;
use oauth2::TokenResponse;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

/// Values shorter than this are not credentials.
const MIN_TOKEN_BYTES: usize = 8;

#[derive(Clone, Copy)]
struct Registered {
    current: SecretHandle,
    digest: [u8; 32],
    previous: Option<SecretHandle>,
}

/// Server name, URL and token field.
type TokenKey = (String, String, &'static str);

/// Holds handles and digests, never values.
static REGISTERED: LazyLock<Mutex<HashMap<TokenKey, Registered>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Registers the tokens in `tokens`. A token the gate cannot protect is not
/// used: the error stops the load or save.
pub(super) fn protect(tokens: &StoredOAuthTokens) -> Result<()> {
    let Some(gate) = output_gate::active() else {
        return Ok(());
    };
    let response = &tokens.token_response.0;
    let refresh = response.refresh_token();
    let values = [
        ("access_token", Some(response.access_token().secret())),
        ("refresh_token", refresh.map(oauth2::RefreshToken::secret)),
    ];
    let mut registered = match REGISTERED.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    for (field, value) in values {
        let Some(value) = value.map(|value| value.trim()) else {
            continue;
        };
        if value.len() < MIN_TOKEN_BYTES {
            continue;
        }
        let key = (tokens.server_name.clone(), tokens.url.clone(), field);
        let digest: [u8; 32] = Sha256::digest(value.as_bytes()).into();
        let known = registered.get(&key).copied();
        if known.is_some_and(|known| known.digest == digest) {
            continue;
        }
        let label = format!("mcp-oauth:{}:{field}", tokens.server_name);
        let handle = gate
            .register(&label, SecretClass::Operational, value)
            .map_err(|err| {
                anyhow::anyhow!(
                    "secret_output_gate cannot protect the MCP OAuth {field} of {}: {err}",
                    tokens.server_name
                )
            })?;
        if let Some(retired) = known.and_then(|known| known.previous) {
            gate.retire(retired);
        }
        registered.insert(
            key,
            Registered {
                current: handle,
                digest,
                previous: known.map(|known| known.current),
            },
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "output_gate_tests.rs"]
mod tests;
