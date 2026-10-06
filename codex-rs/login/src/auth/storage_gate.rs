//! PF-28-S01: sign-in tokens enter the secret output gate whenever the sign-in
//! store loads or saves them, so a login, a token refresh or a keyring-held
//! credential is protected from the moment this process holds it. A changed
//! value rotates: the new one is admitted before the old one is retired.

use super::AgentIdentityStorage;
use super::AuthDotJson;
use super::AuthStorageBackend;
use codex_secret_broker::output_gate;
use codex_secret_broker::output_gate::SecretClass;
use codex_secret_broker::output_gate::SecretHandle;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use zeroize::Zeroizing;

/// Values shorter than this are not credentials (and would redact ordinary
/// text).
const MIN_TOKEN_BYTES: usize = 8;

#[derive(Debug)]
pub(super) struct GatedAuthStorage {
    inner: Arc<dyn AuthStorageBackend>,
    /// Per field: the registered value's handle and digest (never the value).
    registered: Mutex<HashMap<&'static str, (SecretHandle, [u8; 32])>>,
}

impl GatedAuthStorage {
    pub(super) fn wrap(inner: Arc<dyn AuthStorageBackend>) -> Arc<dyn AuthStorageBackend> {
        Arc::new(Self {
            inner,
            registered: Mutex::new(HashMap::new()),
        })
    }

    /// Registers every token in `auth`. Fails only while the gate is armed:
    /// a credential the gate cannot protect is not used.
    fn protect(&self, auth: &AuthDotJson) -> std::io::Result<()> {
        let gate = output_gate::global();
        let mut registered = match self.registered.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        for (field, value) in auth_values(auth) {
            let value = value.trim();
            if value.len() < MIN_TOKEN_BYTES {
                continue;
            }
            let digest: [u8; 32] = Sha256::digest(value.as_bytes()).into();
            let previous = registered.get(field).copied();
            if previous.is_some_and(|(_, known)| known == digest) {
                continue;
            }
            let label = format!("auth.json:{field}");
            let result = match previous {
                Some((old, _)) => gate.rotate(old, &label, SecretClass::Operational, value),
                None => gate.register(&label, SecretClass::Operational, value),
            };
            match result {
                Ok(handle) => {
                    registered.insert(field, (handle, digest));
                }
                Err(err) if gate.is_armed() => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("secret_output_gate cannot protect the sign-in {field}: {err}"),
                    ));
                }
                Err(_) => {}
            }
        }
        Ok(())
    }
}

impl AuthStorageBackend for GatedAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
        let auth = self.inner.load()?;
        if let Some(auth) = &auth {
            self.protect(auth)?;
        }
        Ok(auth)
    }

    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()> {
        self.protect(auth)?;
        self.inner.save(auth)
    }

    fn delete(&self) -> std::io::Result<bool> {
        self.inner.delete()
    }
}

fn auth_values(auth: &AuthDotJson) -> Vec<(&'static str, Zeroizing<String>)> {
    let mut values = Vec::new();
    let mut push = |field: &'static str, value: Option<&String>| {
        if let Some(value) = value {
            values.push((field, Zeroizing::new(value.clone())));
        }
    };
    push("OPENAI_API_KEY", auth.openai_api_key.as_ref());
    push("personal_access_token", auth.personal_access_token.as_ref());
    push(
        "bedrock_api_key",
        auth.bedrock_api_key.as_ref().map(|key| &key.api_key),
    );
    if let Some(tokens) = &auth.tokens {
        push("id_token", Some(&tokens.id_token.raw_jwt));
        push("access_token", Some(&tokens.access_token));
        push("refresh_token", Some(&tokens.refresh_token));
    }
    match &auth.agent_identity {
        Some(AgentIdentityStorage::Jwt(jwt)) => push("agent_identity", Some(jwt)),
        Some(AgentIdentityStorage::Record(record)) => {
            push("agent_private_key", Some(&record.agent_private_key));
        }
        None => {}
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_secret_broker::output_gate::OutputSink;

    #[derive(Debug, Default)]
    struct Memory(Mutex<Option<AuthDotJson>>);

    impl AuthStorageBackend for Memory {
        fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
            Ok(self.0.lock().expect("lock").clone())
        }
        fn save(&self, auth: &AuthDotJson) -> std::io::Result<()> {
            *self.0.lock().expect("lock") = Some(auth.clone());
            Ok(())
        }
        fn delete(&self) -> std::io::Result<bool> {
            Ok(self.0.lock().expect("lock").take().is_some())
        }
    }

    fn api_key_auth(key: &str) -> AuthDotJson {
        AuthDotJson {
            auth_mode: None,
            openai_api_key: Some(key.to_string()),
            tokens: None,
            last_refresh: None,
            agent_identity: None,
            personal_access_token: None,
            bedrock_api_key: None,
        }
    }

    #[test]
    fn pf_28_s01_login_and_rotation_after_arm_are_protected() {
        // Process-wide gate: armed before any sign-in, as Core does.
        output_gate::global().arm().expect("arm");
        let storage = GatedAuthStorage::wrap(Arc::new(Memory::default()));
        let scrub = |text: &str| output_gate::scrub_if_armed(OutputSink::ToolResult, text);
        // Synthetic values only.
        storage
            .save(&api_key_auth("sk-pf28-login-after-arm-1111"))
            .expect("login");
        assert_eq!(
            scrub("k=sk-pf28-login-after-arm-1111").as_deref(),
            Some("k=[REDACTED:auth.json:OPENAI_API_KEY]")
        );
        // A refresh replaces the key: the new one is protected, the old one
        // retired.
        storage
            .save(&api_key_auth("sk-pf28-refreshed-key-2222"))
            .expect("refresh");
        assert!(scrub("sk-pf28-refreshed-key-2222").is_some());
        assert_eq!(scrub("sk-pf28-login-after-arm-1111"), None);
        // Loading the same value again changes nothing.
        storage.load().expect("load");
        assert!(scrub("sk-pf28-refreshed-key-2222").is_some());
    }
}
