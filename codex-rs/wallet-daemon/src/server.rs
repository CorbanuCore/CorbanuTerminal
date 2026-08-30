use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use codex_uds::UnixListener;
use codex_uds::prepare_private_socket_directory;
use codex_wallet::UnlockedWallet;
use codex_wallet::Wallet;
use codex_wallet::WalletError;
use rand::TryRngCore;
use rand::rngs::OsRng;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroize;

use crate::client::run_dir;
use crate::client::socket_path;
use crate::protocol::DaemonStatus;
use crate::protocol::Request;
use crate::protocol::Response;
use crate::protocol::UnlockPolicy;

const ONE_ACTION_STAGING_SECONDS: u64 = 5 * 60;

#[derive(Clone, Copy)]
struct CapabilityGrant {
    expiry: Instant,
    one_action: bool,
}

struct State {
    wallet: Wallet,
    unlocked: Option<UnlockedWallet>,
    expires_at: Option<Instant>,
    capabilities: HashMap<String, CapabilityGrant>,
    failed_unlocks: u32,
    generation: u64,
    active_operation: Option<CancellationToken>,
}

pub async fn run_wallet_daemon(codex_home: PathBuf) -> Result<()> {
    #[cfg(target_os = "linux")]
    codex_process_hardening::disable_process_dumping()
        .context("disable wallet daemon core dumps")?;
    let wallet_dir = codex_home.join("wallet");
    tokio::fs::create_dir_all(&wallet_dir).await?;
    #[cfg(unix)]
    tokio::fs::set_permissions(
        &wallet_dir,
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .await?;
    prepare_private_socket_directory(run_dir(&codex_home)).await?;
    // The non-blocking ownership lock is the proof that no live daemon owns this home. Keep
    // the guard for the listener's entire lifetime and only then reclaim a stale socket left by
    // a crashed owner. A second daemon fails here without touching the live socket.
    let _ownership = acquire_ownership(&codex_home)?;
    let socket = socket_path(&codex_home);
    if tokio::fs::try_exists(&socket).await? {
        tokio::fs::remove_file(&socket).await?;
    }
    let mut listener = UnixListener::bind(&socket).await?;
    #[cfg(unix)]
    tokio::fs::set_permissions(&socket, std::os::unix::fs::PermissionsExt::from_mode(0o600))
        .await?;
    let state = Arc::new(Mutex::new(State {
        wallet: Wallet::new(codex_home),
        unlocked: None,
        expires_at: None,
        capabilities: HashMap::new(),
        failed_unlocks: 0,
        generation: 0,
        active_operation: None,
    }));
    loop {
        let stream = listener.accept().await?;
        #[cfg(unix)]
        // SAFETY: `geteuid` reads process credentials and has no pointer or lifetime contract.
        if stream.peer_user_id()? != Some(unsafe { libc::geteuid() }) {
            continue;
        }
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let (read, mut write) = tokio::io::split(stream);
            let mut line = String::new();
            let response = match BufReader::new(read).read_line(&mut line).await {
                Ok(0) => Response::Error {
                    code: "empty_request".into(),
                    message: "request was empty".into(),
                },
                Ok(_) => match serde_json::from_str::<Request>(&line) {
                    Ok(request) => handle(state, request).await,
                    Err(_) => Response::Error {
                        code: "invalid_request".into(),
                        message: "request was malformed".into(),
                    },
                },
                Err(_) => return,
            };
            if let Ok(mut payload) = serde_json::to_vec(&response) {
                payload.push(b'\n');
                let _ = write.write_all(&payload).await;
            }
        });
    }
}

async fn handle(state: Arc<Mutex<State>>, request: Request) -> Response {
    match request {
        Request::Unlock {
            mut passcode,
            duration_seconds,
            one_action,
        } => {
            let policy = if one_action {
                UnlockPolicy::OneAction
            } else {
                UnlockPolicy::Timed { duration_seconds }
            };
            handle_unlock(state, &mut passcode, policy).await
        }
        Request::ProvisionPlan {
            mut capability,
            intent,
        } => handle_provision_plan(state, &mut capability, intent).await,
        Request::IssueGatewayKey {
            mut capability,
            gateway_origin,
        } => handle_issue_gateway_key(state, &mut capability, gateway_origin).await,
        Request::CorbanuApiOperation {
            mut capability,
            gateway_origin,
            operation,
        } => handle_corbanu_api_operation(state, &mut capability, gateway_origin, operation).await,
        request => handle_immediate(state, request).await,
    }
}

async fn handle_immediate(state: Arc<Mutex<State>>, mut request: Request) -> Response {
    let mut state = state.lock().await;
    expire(&mut state);
    match &mut request {
        Request::Ping => Response::Pong,
        Request::Status => Response::Status(status(&state)),
        Request::Lock => {
            lock(&mut state);
            Response::Locked
        }
        Request::RemoveWallet { expected_address } => {
            if state.active_operation.is_some() {
                return operation_busy();
            }
            lock(&mut state);
            match state.wallet.remove_from_device(expected_address) {
                Ok(()) => Response::WalletRemoved,
                Err(WalletError::Missing) => Response::WalletRemoved,
                Err(error) => Response::Error {
                    code: "wallet_removal_failed".into(),
                    message: error.to_string(),
                },
            }
        }
        Request::Unlock { .. } => unreachable!("unlock is handled without an await-held lock"),
        Request::SignOwnership {
            capability,
            gateway_origin,
            challenge,
        } => {
            let grant = state.capabilities.get(capability).copied();
            let valid = grant.is_some_and(|grant| grant.expiry > Instant::now());
            if grant.is_some_and(|grant| grant.one_action) {
                state.capabilities.remove(capability);
            }
            capability.zeroize();
            if !valid {
                return Response::Error {
                    code: "capability_invalid".into(),
                    message: "signing capability is invalid or expired".into(),
                };
            }
            let response = if codex_wallet::validate_gateway_origin(gateway_origin).is_err() {
                Response::Error {
                    code: "origin_refused".into(),
                    message: "gateway origin is not permitted".into(),
                }
            } else {
                match state.unlocked.as_ref() {
                    Some(wallet) => Response::Signature {
                        signature: wallet.sign_ownership_challenge(gateway_origin, challenge),
                    },
                    None => Response::Error {
                        code: "locked".into(),
                        message: "wallet is locked".into(),
                    },
                }
            };
            if grant.is_some_and(|grant| grant.one_action) {
                lock(&mut state);
            }
            response
        }
        Request::ProvisionPlan { .. }
        | Request::IssueGatewayKey { .. }
        | Request::CorbanuApiOperation { .. } => {
            unreachable!("network operations are handled without an await-held lock")
        }
    }
}

async fn handle_unlock(
    state: Arc<Mutex<State>>,
    passcode: &mut String,
    policy: UnlockPolicy,
) -> Response {
    let failed_unlocks = {
        let mut state = state.lock().await;
        expire(&mut state);
        if state.active_operation.is_some() {
            passcode.zeroize();
            return operation_busy();
        }
        state.failed_unlocks
    };
    if failed_unlocks > 0 {
        tokio::time::sleep(Duration::from_millis(
            (250_u64 << failed_unlocks.min(4)).min(4000),
        ))
        .await;
    }

    let mut state = state.lock().await;
    expire(&mut state);
    if state.active_operation.is_some() {
        passcode.zeroize();
        return operation_busy();
    }
    let (duration, one_action) = match policy {
        UnlockPolicy::OneAction => (ONE_ACTION_STAGING_SECONDS, true),
        UnlockPolicy::Timed { duration_seconds } => {
            (duration_seconds.clamp(60, 8 * 60 * 60), false)
        }
    };
    match state.wallet.unlock(passcode) {
        Ok(wallet) => {
            passcode.zeroize();
            let capability = match new_capability() {
                Ok(value) => value,
                Err(error) => {
                    return Response::Error {
                        code: "entropy_unavailable".into(),
                        message: error.to_string(),
                    };
                }
            };
            state.failed_unlocks = 0;
            state.unlocked = Some(wallet);
            state.generation = state.generation.wrapping_add(1);
            let expiry = Instant::now() + Duration::from_secs(duration);
            state.expires_at = Some(expiry);
            state
                .capabilities
                .insert(capability.clone(), CapabilityGrant { expiry, one_action });
            Response::Unlocked {
                capability,
                expires_in_seconds: duration,
            }
        }
        Err(error) => {
            passcode.zeroize();
            state.failed_unlocks = state.failed_unlocks.saturating_add(1);
            Response::Error {
                code: "unlock_failed".into(),
                message: error.to_string(),
            }
        }
    }
}

struct OperationLease {
    wallet: UnlockedWallet,
    generation: u64,
    cancel: CancellationToken,
    relock_after_operation: bool,
}

async fn checkout_wallet(
    state: &Arc<Mutex<State>>,
    capability: &mut String,
    invalid_message: &'static str,
) -> Result<OperationLease, Response> {
    let mut state = state.lock().await;
    expire(&mut state);
    if state.active_operation.is_some() {
        capability.zeroize();
        return Err(operation_busy());
    }
    let grant = state.capabilities.get(capability).copied();
    let valid = grant.is_some_and(|grant| grant.expiry > Instant::now());
    if grant.is_some_and(|grant| grant.one_action) {
        state.capabilities.remove(capability);
    }
    capability.zeroize();
    if !valid {
        return Err(Response::Error {
            code: "capability_invalid".into(),
            message: invalid_message.into(),
        });
    }
    let Some(wallet) = state.unlocked.take() else {
        return Err(Response::Error {
            code: "locked".into(),
            message: "wallet is locked".into(),
        });
    };
    let cancel = CancellationToken::new();
    state.active_operation = Some(cancel.clone());
    Ok(OperationLease {
        wallet,
        generation: state.generation,
        cancel,
        relock_after_operation: grant.is_some_and(|grant| grant.one_action),
    })
}

async fn finish_operation(state: &Arc<Mutex<State>>, lease: OperationLease) {
    let mut state = state.lock().await;
    if lease.relock_after_operation {
        state.active_operation = None;
        lock(&mut state);
        return;
    }
    let still_current = state.generation == lease.generation
        && !lease.cancel.is_cancelled()
        && state
            .expires_at
            .is_some_and(|expiry| expiry > Instant::now());
    state.active_operation = None;
    if still_current {
        state.unlocked = Some(lease.wallet);
    }
}

async fn handle_provision_plan(
    state: Arc<Mutex<State>>,
    capability: &mut String,
    intent: codex_wallet::PlanPurchaseIntent,
) -> Response {
    let lease = match checkout_wallet(
        &state,
        capability,
        "payment capability is invalid or expired",
    )
    .await
    {
        Ok(lease) => lease,
        Err(response) => return response,
    };
    let result = tokio::select! {
        _ = lease.cancel.cancelled() => Err("wallet was locked while the purchase was in progress".to_string()),
        result = lease.wallet.provision_plan(intent) => result.map_err(|error| error.to_string()),
    };
    finish_operation(&state, lease).await;
    match result {
        Ok(result) => Response::PlanProvisioned(result),
        Err(message) => Response::Error {
            code: "purchase_failed".into(),
            message,
        },
    }
}

async fn handle_issue_gateway_key(
    state: Arc<Mutex<State>>,
    capability: &mut String,
    gateway_origin: String,
) -> Response {
    let lease = match checkout_wallet(
        &state,
        capability,
        "key-recovery capability is invalid or expired",
    )
    .await
    {
        Ok(lease) => lease,
        Err(response) => return response,
    };
    let result = tokio::select! {
        _ = lease.cancel.cancelled() => Err("wallet was locked while key recovery was in progress".to_string()),
        result = lease.wallet.issue_gateway_key(gateway_origin) => result.map_err(|error| error.to_string()),
    };
    finish_operation(&state, lease).await;
    match result {
        Ok(result) => Response::GatewayKeyIssued(result),
        Err(message) => Response::Error {
            code: "key_recovery_failed".into(),
            message,
        },
    }
}

async fn handle_corbanu_api_operation(
    state: Arc<Mutex<State>>,
    capability: &mut String,
    gateway_origin: String,
    operation: codex_wallet::CorbanuApiOperation,
) -> Response {
    let lease = match checkout_wallet(
        &state,
        capability,
        "Corbanu API capability is invalid or expired",
    )
    .await
    {
        Ok(lease) => lease,
        Err(response) => return response,
    };
    let result = tokio::select! {
        _ = lease.cancel.cancelled() => Err("wallet was locked while the Corbanu API operation was in progress".to_string()),
        result = lease.wallet.execute_corbanu_api_operation(gateway_origin, operation) => {
            result.map_err(|error| error.to_string())
        },
    };
    finish_operation(&state, lease).await;
    match result {
        Ok(result) => Response::CorbanuApiOperationCompleted(result),
        Err(message) => Response::Error {
            code: "corbanu_api_operation_failed".into(),
            message,
        },
    }
}

fn operation_busy() -> Response {
    Response::Error {
        code: "operation_in_progress".into(),
        message: "another wallet signing operation is still in progress".into(),
    }
}

fn status(state: &State) -> DaemonStatus {
    let manifest = state.wallet.manifest().ok();
    let busy = state
        .active_operation
        .as_ref()
        .is_some_and(|operation| !operation.is_cancelled());
    DaemonStatus {
        wallet_exists: state.wallet.exists(),
        address: manifest.as_ref().map(|m| m.address.clone()),
        network: manifest.map(|m| format!("{:?}", m.network).to_lowercase()),
        locked: state.unlocked.is_none()
            && state
                .active_operation
                .as_ref()
                .is_none_or(CancellationToken::is_cancelled),
        busy,
        expires_in_seconds: state
            .expires_at
            .map(|expiry| expiry.saturating_duration_since(Instant::now()).as_secs()),
    }
}
fn expire(state: &mut State) {
    if state
        .expires_at
        .is_some_and(|expiry| expiry <= Instant::now())
    {
        lock(state);
    }
}
fn lock(state: &mut State) {
    if let Some(cancel) = &state.active_operation {
        cancel.cancel();
    }
    state.generation = state.generation.wrapping_add(1);
    state.unlocked = None;
    state.expires_at = None;
    state.capabilities.clear();
}
fn new_capability() -> Result<String> {
    let mut bytes = [0_u8; 32];
    OsRng.try_fill_bytes(&mut bytes)?;
    let value = URL_SAFE_NO_PAD.encode(bytes);
    bytes.zeroize();
    Ok(value)
}

fn acquire_ownership(home: &std::path::Path) -> Result<File> {
    let path = run_dir(home).join("walletd.lock");
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options.open(path)?;
    #[cfg(unix)]
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    #[cfg(unix)]
    {
        let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        anyhow::ensure!(
            result == 0,
            "another wallet daemon owns this Corbanu Terminal home"
        );
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Storage::FileSystem::LOCKFILE_EXCLUSIVE_LOCK;
        use windows_sys::Win32::Storage::FileSystem::LOCKFILE_FAIL_IMMEDIATELY;
        use windows_sys::Win32::Storage::FileSystem::LockFileEx;
        use windows_sys::Win32::System::IO::OVERLAPPED;

        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        let locked = unsafe {
            LockFileEx(
                file.as_raw_handle() as isize,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                1,
                0,
                &mut overlapped,
            )
        };
        anyhow::ensure!(
            locked != 0,
            "another wallet daemon owns this Corbanu Terminal home"
        );
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WalletDaemonClient;
    use codex_wallet::Network;

    #[tokio::test]
    async fn unlock_capability_remains_scoped_for_its_ttl_and_global_lock_is_real() {
        let home = tempfile::tempdir().expect("tempdir");
        Wallet::new(home.path().to_path_buf())
            .create("a sufficiently long test passphrase", Network::Devnet)
            .expect("create wallet");
        let daemon_home = home.path().to_path_buf();
        let server = tokio::spawn(async move { run_wallet_daemon(daemon_home).await });
        for _ in 0..40 {
            if tokio::fs::try_exists(socket_path(home.path()))
                .await
                .unwrap_or(false)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let client = WalletDaemonClient::new(home.path().to_path_buf());
        assert!(client.status().await.expect("status").locked);
        let (capability, _) = client
            .unlock(
                "a sufficiently long test passphrase".to_string(),
                UnlockPolicy::Timed {
                    duration_seconds: 60,
                },
            )
            .await
            .expect("unlock");
        assert!(!client.status().await.expect("status").locked);
        assert!(
            !client
                .sign_ownership(
                    capability.clone(),
                    "https://gateway.example".to_string(),
                    "challenge".to_string()
                )
                .await
                .expect("sign")
                .is_empty()
        );
        assert!(
            !client
                .sign_ownership(
                    capability.clone(),
                    "http://localhost:4021".to_string(),
                    "challenge".to_string()
                )
                .await
                .expect("loopback alias accepted consistently")
                .is_empty()
        );
        assert!(
            client
                .sign_ownership(
                    capability.clone(),
                    "http://gateway.example".to_string(),
                    "challenge".to_string()
                )
                .await
                .is_err()
        );
        assert!(
            !client
                .sign_ownership(
                    capability.clone(),
                    "https://gateway.example".to_string(),
                    "challenge".to_string()
                )
                .await
                .expect("second sign during TTL")
                .is_empty()
        );
        client.lock().await.expect("lock");
        assert!(client.status().await.expect("status").locked);
        assert!(
            client
                .sign_ownership(
                    capability,
                    "https://gateway.example".to_string(),
                    "challenge".to_string()
                )
                .await
                .is_err()
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind stalled gateway");
        let gateway_origin = format!("http://{}", listener.local_addr().expect("gateway address"));
        let stalled_gateway = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.expect("accept gateway request");
            tokio::time::sleep(Duration::from_secs(30)).await;
        });
        let (capability, _) = client
            .unlock(
                "a sufficiently long test passphrase".to_string(),
                UnlockPolicy::Timed {
                    duration_seconds: 60,
                },
            )
            .await
            .expect("unlock for cancellation");
        let issue_client = client.clone();
        let issue = tokio::spawn(async move {
            issue_client
                .issue_gateway_key(capability, gateway_origin)
                .await
        });
        tokio::time::sleep(Duration::from_millis(100)).await;
        let busy = client.status().await.expect("status during operation");
        assert!(busy.busy);
        assert!(!busy.locked);
        tokio::time::timeout(Duration::from_secs(1), client.lock())
            .await
            .expect("lock must not wait for the network operation")
            .expect("lock active operation");
        let issue_error = tokio::time::timeout(Duration::from_secs(1), issue)
            .await
            .expect("cancelled operation must return promptly")
            .expect("join issue task")
            .expect_err("cancelled operation must fail");
        assert!(issue_error.to_string().contains("wallet was locked"));
        assert!(
            client
                .status()
                .await
                .expect("status after cancellation")
                .locked
        );
        stalled_gateway.abort();
        server.abort();
    }

    #[tokio::test]
    async fn one_action_capability_locks_after_the_first_signing_attempt() {
        let home = tempfile::tempdir().expect("tempdir");
        Wallet::new(home.path().to_path_buf())
            .create("a sufficiently long test passphrase", Network::Devnet)
            .expect("create wallet");
        let daemon_home = home.path().to_path_buf();
        let server = tokio::spawn(async move { run_wallet_daemon(daemon_home).await });
        for _ in 0..40 {
            if tokio::fs::try_exists(socket_path(home.path()))
                .await
                .unwrap_or(false)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let client = WalletDaemonClient::new(home.path().to_path_buf());

        let (capability, expires_in_seconds) = client
            .unlock(
                "a sufficiently long test passphrase".to_string(),
                UnlockPolicy::OneAction,
            )
            .await
            .expect("one-action unlock");
        assert_eq!(expires_in_seconds, ONE_ACTION_STAGING_SECONDS);
        assert!(
            client
                .sign_ownership(
                    capability.clone(),
                    "http://gateway.example".to_string(),
                    "challenge".to_string(),
                )
                .await
                .is_err()
        );
        assert!(client.status().await.expect("status").locked);
        assert!(
            client
                .sign_ownership(
                    capability,
                    "https://gateway.example".to_string(),
                    "challenge".to_string(),
                )
                .await
                .is_err()
        );
        server.abort();
    }

    #[tokio::test]
    async fn second_daemon_cannot_remove_the_live_daemon_socket() {
        let home = tempfile::tempdir().expect("tempdir");
        let daemon_home = home.path().to_path_buf();
        let server = tokio::spawn(async move { run_wallet_daemon(daemon_home).await });
        for _ in 0..40 {
            if tokio::fs::try_exists(socket_path(home.path()))
                .await
                .unwrap_or(false)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        let error = run_wallet_daemon(home.path().to_path_buf())
            .await
            .expect_err("ownership lock must reject a second daemon");
        assert!(error.to_string().contains("another wallet daemon owns"));
        assert!(
            tokio::fs::try_exists(socket_path(home.path()))
                .await
                .expect("socket probe")
        );
        assert!(
            WalletDaemonClient::new(home.path().to_path_buf())
                .status()
                .await
                .is_ok()
        );
        server.abort();
    }

    #[tokio::test]
    async fn remove_wallet_requires_the_current_address_and_clears_daemon_state() {
        let home = tempfile::tempdir().expect("tempdir");
        let created = Wallet::new(home.path().to_path_buf())
            .create("a sufficiently long test passphrase", Network::Mainnet)
            .expect("create wallet");
        let daemon_home = home.path().to_path_buf();
        let server = tokio::spawn(async move { run_wallet_daemon(daemon_home).await });
        for _ in 0..40 {
            if tokio::fs::try_exists(socket_path(home.path()))
                .await
                .unwrap_or(false)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        let client = WalletDaemonClient::new(home.path().to_path_buf());

        assert!(
            client
                .remove_wallet("11111111111111111111111111111111".to_string())
                .await
                .is_err()
        );
        assert!(client.status().await.expect("status").wallet_exists);
        client
            .remove_wallet(created.manifest.address)
            .await
            .expect("remove wallet");
        client
            .remove_wallet("11111111111111111111111111111111".to_string())
            .await
            .expect("removing an already-absent wallet is idempotent");
        assert_eq!(
            client.status().await.expect("removed status"),
            DaemonStatus {
                wallet_exists: false,
                address: None,
                network: None,
                locked: true,
                busy: false,
                expires_in_seconds: None,
            }
        );

        server.abort();
    }
}
