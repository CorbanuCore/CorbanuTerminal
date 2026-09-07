use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use argon2::Algorithm;
use argon2::Argon2;
use argon2::Params;
use argon2::Version;
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::XNonce;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::aead::KeyInit;
use chrono::DateTime;
use chrono::Utc;
use codex_keyring_store::DefaultKeyringStore;
use codex_keyring_store::KeyringStore;
use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;
use rand::TryRngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use thiserror::Error;
use uuid::Uuid;
use zeroize::Zeroize;
use zeroize::Zeroizing;

const SCHEMA_VERSION: u32 = 1;
const KDF_MEMORY_KIB: u32 = 64 * 1024;
const KDF_ITERATIONS: u32 = 3;
const KDF_LANES: u32 = 1;
const MIN_MEMORY_KIB: u32 = 32 * 1024;
const MAX_MEMORY_KIB: u32 = 512 * 1024;
const MIN_ITERATIONS: u32 = 2;
const MAX_ITERATIONS: u32 = 8;
const MIN_SHORT_PASSCODE: usize = 6;
const MIN_PORTABLE_PASSPHRASE: usize = 12;
const KEYRING_SERVICE: &str = "pfterminal-wallet";
const AAD: &[u8] = b"pfterminal-wallet-envelope-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    Mainnet,
    Devnet,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicManifest {
    pub schema_version: u32,
    pub network: Network,
    pub address: String,
    pub supported_assets: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreatedWallet {
    pub manifest: PublicManifest,
    pub recovery_material: Zeroizing<String>,
}

#[derive(Debug)]
pub struct RecoveryBackup {
    pub manifest: PublicManifest,
    pub recovery_material: Zeroizing<String>,
}

#[derive(Debug)]
pub struct UnlockedWallet {
    manifest: PublicManifest,
    seed: Zeroizing<[u8; 32]>,
}

impl UnlockedWallet {
    pub fn manifest(&self) -> &PublicManifest {
        &self.manifest
    }

    /// Signs only a domain-separated gateway ownership challenge.
    pub fn sign_ownership_challenge(&self, gateway_origin: &str, challenge: &str) -> String {
        let message = format!("pfterminal-plan-ownership-v1\n{gateway_origin}\n{challenge}");
        let signing_key = SigningKey::from_bytes(&self.seed);
        bs58::encode(signing_key.sign(message.as_bytes()).to_bytes()).into_string()
    }

    pub async fn pay_x402(
        &self,
        intent: crate::PaymentIntent,
    ) -> Result<crate::PaymentReceipt, crate::X402PaymentError> {
        crate::payment::pay(self, intent).await
    }

    pub async fn provision_plan(
        &self,
        intent: crate::PlanPurchaseIntent,
    ) -> Result<crate::ProvisionedPlan, crate::X402PaymentError> {
        crate::payment::provision_plan(self, intent).await
    }

    pub async fn issue_gateway_key(
        &self,
        gateway_origin: String,
    ) -> Result<crate::GatewayKey, crate::X402PaymentError> {
        crate::payment::issue_gateway_key(self, gateway_origin).await
    }

    pub async fn execute_corbanu_api_operation(
        &self,
        gateway_origin: String,
        operation: crate::CorbanuApiOperation,
    ) -> Result<crate::CorbanuApiOperationResult, crate::X402PaymentError> {
        crate::corbanu_api::execute(self, gateway_origin, operation).await
    }

    pub(crate) fn seed_for_payment(&self) -> &[u8; 32] {
        &self.seed
    }
}

impl Drop for UnlockedWallet {
    fn drop(&mut self) {
        self.seed.zeroize();
    }
}

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("a wallet already exists")]
    AlreadyExists,
    #[error("no wallet exists")]
    Missing,
    #[error("passcode must contain at least {0} characters")]
    PasscodeTooShort(usize),
    #[error(
        "short passcodes require the operating-system credential store; use a passphrase of at least 12 characters or repair the credential store"
    )]
    MachineSecretUnavailable,
    #[error("wallet passcode is incorrect or the wallet file was modified")]
    UnlockFailed,
    #[error("wallet envelope uses unsafe or unsupported key-derivation parameters")]
    UnsafeParameters,
    #[error("recovery material is invalid")]
    InvalidRecovery,
    #[error("wallet storage failed: {0}")]
    Storage(String),
    #[error("wallet address confirmation did not match the wallet on this device")]
    AddressMismatch,
}

#[derive(Clone)]
pub struct Wallet {
    root: PathBuf,
    keyring: Arc<dyn KeyringStore>,
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    schema_version: u32,
    suite: String,
    network: Network,
    address: String,
    machine_bound: bool,
    salt: String,
    nonce: String,
    memory_kib: u32,
    iterations: u32,
    lanes: u32,
    ciphertext: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct SecretPayload {
    seed: String,
    policy: String,
}

impl Wallet {
    pub fn new(codex_home: PathBuf) -> Self {
        Self::new_with_keyring(codex_home, Arc::new(DefaultKeyringStore))
    }

    pub fn new_with_keyring(codex_home: PathBuf, keyring: Arc<dyn KeyringStore>) -> Self {
        Self {
            root: codex_home.join("wallet"),
            keyring,
        }
    }

    pub fn exists(&self) -> bool {
        self.envelope_path().is_file()
    }

    pub fn manifest(&self) -> Result<PublicManifest, WalletError> {
        read_json(&self.manifest_path())
    }

    pub fn create(&self, passcode: &str, network: Network) -> Result<CreatedWallet, WalletError> {
        let _creation_lock = self.acquire_creation_lock()?;
        if self.exists() {
            return Err(WalletError::AlreadyExists);
        }
        let mut seed = Zeroizing::new([0_u8; 32]);
        OsRng.try_fill_bytes(seed.as_mut()).map_err(storage)?;
        self.persist_seed(passcode, network, &seed)
    }

    pub fn restore(
        &self,
        recovery_material: &str,
        passcode: &str,
        network: Network,
    ) -> Result<CreatedWallet, WalletError> {
        let _creation_lock = self.acquire_creation_lock()?;
        if self.exists() {
            return Err(WalletError::AlreadyExists);
        }
        let decoded = bs58::decode(recovery_material.trim())
            .into_vec()
            .map_err(|_| WalletError::InvalidRecovery)?;
        let seed: [u8; 32] = decoded
            .try_into()
            .map_err(|_| WalletError::InvalidRecovery)?;
        self.persist_seed(passcode, network, &Zeroizing::new(seed))
    }

    pub fn unlock(&self, passcode: &str) -> Result<UnlockedWallet, WalletError> {
        let envelope: Envelope = read_json(&self.envelope_path())?;
        validate_envelope(&envelope)?;
        let machine_secret = self.load_machine_secret(envelope.machine_bound)?;
        let mut key = Zeroizing::new([0_u8; 32]);
        derive_key(
            passcode,
            machine_secret.as_deref().map(String::as_str),
            &decode::<16>(&envelope.salt)?,
            envelope.memory_kib,
            envelope.iterations,
            envelope.lanes,
            &mut key,
        )?;
        let cipher = XChaCha20Poly1305::new_from_slice(key.as_ref())
            .map_err(|_| WalletError::UnlockFailed)?;
        let plaintext = cipher
            .decrypt(
                XNonce::from_slice(&decode::<24>(&envelope.nonce)?),
                chacha20poly1305::aead::Payload {
                    msg: &decode_vec(&envelope.ciphertext)?,
                    aad: AAD,
                },
            )
            .map_err(|_| WalletError::UnlockFailed)?;
        let mut plaintext = Zeroizing::new(plaintext);
        let payload: SecretPayload =
            serde_json::from_slice(&plaintext).map_err(|_| WalletError::UnlockFailed)?;
        plaintext.zeroize();
        let seed_vec = bs58::decode(payload.seed)
            .into_vec()
            .map_err(|_| WalletError::UnlockFailed)?;
        let seed: [u8; 32] = seed_vec.try_into().map_err(|_| WalletError::UnlockFailed)?;
        let manifest = self.manifest()?;
        if address_for_seed(&seed) != envelope.address || manifest.address != envelope.address {
            return Err(WalletError::UnlockFailed);
        }
        Ok(UnlockedWallet {
            manifest,
            seed: Zeroizing::new(seed),
        })
    }

    /// Decrypt the existing wallet with a freshly supplied passcode and return portable
    /// recovery material. Callers must keep the result in a host-owned secure view and must not
    /// persist or log it.
    pub fn export_recovery(&self, passcode: &str) -> Result<RecoveryBackup, WalletError> {
        let unlocked = self.unlock(passcode)?;
        Ok(RecoveryBackup {
            manifest: unlocked.manifest.clone(),
            recovery_material: Zeroizing::new(bs58::encode(unlocked.seed.as_ref()).into_string()),
        })
    }

    /// Remove the encrypted wallet from this device after confirming its public address.
    ///
    /// This does not move funds or affect the on-chain wallet. The caller must present the full
    /// address shown to the user so a stale UI cannot remove a different wallet.
    pub fn remove_from_device(&self, expected_address: &str) -> Result<(), WalletError> {
        let envelope: Envelope = read_json(&self.envelope_path())?;
        if envelope.address != expected_address {
            return Err(WalletError::AddressMismatch);
        }
        fs::remove_file(self.envelope_path()).map_err(storage)?;
        match fs::remove_file(self.manifest_path()) {
            Ok(()) if envelope.machine_bound => {
                // The encrypted wallet is already gone, so failure to remove this non-wallet
                // machine binding must not make the destructive operation look retryable.
                let _ = self
                    .keyring
                    .delete(KEYRING_SERVICE, &self.keyring_account());
            }
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(storage(error)),
        }
        Ok(())
    }

    fn persist_seed(
        &self,
        passcode: &str,
        network: Network,
        seed: &Zeroizing<[u8; 32]>,
    ) -> Result<CreatedWallet, WalletError> {
        let machine_secret = self.machine_secret_for_new_wallet(passcode)?;
        let machine_bound = machine_secret.is_some();
        let mut salt = [0_u8; 16];
        let mut nonce = [0_u8; 24];
        OsRng.try_fill_bytes(&mut salt).map_err(storage)?;
        OsRng.try_fill_bytes(&mut nonce).map_err(storage)?;
        let mut key = Zeroizing::new([0_u8; 32]);
        derive_key(
            passcode,
            machine_secret.as_deref().map(String::as_str),
            &salt,
            KDF_MEMORY_KIB,
            KDF_ITERATIONS,
            KDF_LANES,
            &mut key,
        )?;
        let address = address_for_seed(seed);
        let created_at = Utc::now();
        let payload = SecretPayload {
            seed: bs58::encode(seed.as_ref()).into_string(),
            policy: "bounded-v1".to_string(),
        };
        let mut plaintext = Zeroizing::new(serde_json::to_vec(&payload).map_err(storage)?);
        let cipher = XChaCha20Poly1305::new_from_slice(key.as_ref()).map_err(storage)?;
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                chacha20poly1305::aead::Payload {
                    msg: &plaintext,
                    aad: AAD,
                },
            )
            .map_err(storage)?;
        plaintext.zeroize();
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            suite: "argon2id-xchacha20poly1305-v1".to_string(),
            network,
            address: address.clone(),
            machine_bound,
            salt: STANDARD_NO_PAD.encode(salt),
            nonce: STANDARD_NO_PAD.encode(nonce),
            memory_kib: KDF_MEMORY_KIB,
            iterations: KDF_ITERATIONS,
            lanes: KDF_LANES,
            ciphertext: STANDARD_NO_PAD.encode(ciphertext),
            created_at,
        };
        let manifest = PublicManifest {
            schema_version: SCHEMA_VERSION,
            network,
            address,
            supported_assets: vec!["SOL".to_string(), "USDC".to_string()],
            created_at,
        };
        prepare_private_dir(&self.root)?;
        atomic_json(&self.envelope_path(), &envelope, /*mode*/ 0o600)?;
        atomic_json(&self.manifest_path(), &manifest, /*mode*/ 0o600)?;
        Ok(CreatedWallet {
            manifest,
            recovery_material: Zeroizing::new(bs58::encode(seed.as_ref()).into_string()),
        })
    }

    fn machine_secret_for_new_wallet(
        &self,
        passcode: &str,
    ) -> Result<Option<Zeroizing<String>>, WalletError> {
        if passcode.chars().count() < MIN_SHORT_PASSCODE {
            return Err(WalletError::PasscodeTooShort(MIN_SHORT_PASSCODE));
        }
        if passcode.chars().count() >= MIN_PORTABLE_PASSPHRASE {
            return Ok(None);
        }
        let mut bytes = [0_u8; 32];
        OsRng.try_fill_bytes(&mut bytes).map_err(storage)?;
        let value = STANDARD_NO_PAD.encode(bytes);
        bytes.zeroize();
        self.keyring
            .save(KEYRING_SERVICE, &self.keyring_account(), &value)
            .map_err(|_| WalletError::MachineSecretUnavailable)?;
        Ok(Some(Zeroizing::new(value)))
    }

    fn load_machine_secret(
        &self,
        required: bool,
    ) -> Result<Option<Zeroizing<String>>, WalletError> {
        if !required {
            return Ok(None);
        }
        self.keyring
            .load(KEYRING_SERVICE, &self.keyring_account())
            .map_err(|_| WalletError::MachineSecretUnavailable)?
            .map(Zeroizing::new)
            .ok_or(WalletError::MachineSecretUnavailable)
            .map(Some)
    }

    fn keyring_account(&self) -> String {
        let digest = format!(
            "{:x}",
            Sha256::digest(self.root.to_string_lossy().as_bytes())
        );
        format!("wallet|{}", &digest[..16])
    }

    fn envelope_path(&self) -> PathBuf {
        self.root.join("wallet.json")
    }

    fn acquire_creation_lock(&self) -> Result<File, WalletError> {
        prepare_private_dir(&self.root)?;
        let path = self.root.join("creation.lock");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        options.mode(0o600);
        let file = options.open(path).map_err(storage)?;
        #[cfg(unix)]
        {
            file.set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(storage)?;
            let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
            if result != 0 {
                return Err(storage(std::io::Error::last_os_error()));
            }
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::Storage::FileSystem::LOCKFILE_EXCLUSIVE_LOCK;
            use windows_sys::Win32::Storage::FileSystem::LockFileEx;
            use windows_sys::Win32::System::IO::OVERLAPPED;

            let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
            let locked = unsafe {
                LockFileEx(
                    file.as_raw_handle() as isize,
                    LOCKFILE_EXCLUSIVE_LOCK,
                    0,
                    1,
                    0,
                    &mut overlapped,
                )
            };
            if locked == 0 {
                return Err(storage(std::io::Error::last_os_error()));
            }
        }
        Ok(file)
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }
}

fn derive_key(
    passcode: &str,
    machine_secret: Option<&str>,
    salt: &[u8; 16],
    memory_kib: u32,
    iterations: u32,
    lanes: u32,
    out: &mut [u8; 32],
) -> Result<(), WalletError> {
    if !(MIN_MEMORY_KIB..=MAX_MEMORY_KIB).contains(&memory_kib)
        || !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&iterations)
        || lanes != 1
    {
        return Err(WalletError::UnsafeParameters);
    }
    let params = Params::new(memory_kib, iterations, lanes, Some(32))
        .map_err(|_| WalletError::UnsafeParameters)?;
    let mut input = Zeroizing::new(Vec::with_capacity(
        passcode.len() + machine_secret.map_or(0, str::len) + 1,
    ));
    input.extend_from_slice(passcode.as_bytes());
    input.push(0);
    if let Some(secret) = machine_secret {
        input.extend_from_slice(secret.as_bytes());
    }
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(&input, salt, out)
        .map_err(|_| WalletError::UnlockFailed)
}

fn address_for_seed(seed: &[u8; 32]) -> String {
    bs58::encode(SigningKey::from_bytes(seed).verifying_key().to_bytes()).into_string()
}

fn validate_envelope(envelope: &Envelope) -> Result<(), WalletError> {
    if envelope.schema_version != SCHEMA_VERSION
        || envelope.suite != "argon2id-xchacha20poly1305-v1"
    {
        return Err(WalletError::UnsafeParameters);
    }
    if !(MIN_MEMORY_KIB..=MAX_MEMORY_KIB).contains(&envelope.memory_kib)
        || !(MIN_ITERATIONS..=MAX_ITERATIONS).contains(&envelope.iterations)
        || envelope.lanes != 1
    {
        return Err(WalletError::UnsafeParameters);
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, WalletError> {
    let bytes = fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            WalletError::Missing
        } else {
            storage(error)
        }
    })?;
    serde_json::from_slice(&bytes).map_err(storage)
}

fn decode<const N: usize>(value: &str) -> Result<[u8; N], WalletError> {
    decode_vec(value)?
        .try_into()
        .map_err(|_| WalletError::UnlockFailed)
}

fn decode_vec(value: &str) -> Result<Vec<u8>, WalletError> {
    STANDARD_NO_PAD
        .decode(value)
        .map_err(|_| WalletError::UnlockFailed)
}

fn prepare_private_dir(path: &Path) -> Result<(), WalletError> {
    fs::create_dir_all(path).map_err(storage)?;
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(storage)?;
    Ok(())
}

fn atomic_json<T: Serialize>(path: &Path, value: &T, mode: u32) -> Result<(), WalletError> {
    let parent = path
        .parent()
        .ok_or_else(|| WalletError::Storage("wallet path has no parent".to_string()))?;
    let temporary = parent.join(format!(".wallet-{}.tmp", Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(mode);
    let mut file = options.open(&temporary).map_err(storage)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(storage)?;
    file.write_all(&bytes).map_err(storage)?;
    file.write_all(b"\n").map_err(storage)?;
    file.sync_all().map_err(storage)?;
    fs::rename(&temporary, path).map_err(storage)?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(storage)?;
    Ok(())
}

fn storage(error: impl std::fmt::Display) -> WalletError {
    WalletError::Storage(error.to_string())
}

#[cfg(test)]
mod tests;
