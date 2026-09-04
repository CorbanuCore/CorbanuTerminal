use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use codex_uds::UnixStream;
use codex_wallet::CorbanuApiOperation;
use codex_wallet::CorbanuApiOperationResult;
use codex_wallet::GatewayKey;
use codex_wallet::PlanPurchaseIntent;
use codex_wallet::ProvisionedPlan;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

use crate::protocol::DaemonStatus;
use crate::protocol::Request;
use crate::protocol::Response;
use crate::protocol::UnlockPolicy;
use crate::protocol::WalletDaemonError;

const PING_TIMEOUT: Duration = Duration::from_millis(100);
const STATUS_TIMEOUT: Duration = Duration::from_secs(5);
const LOCAL_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);
const NETWORK_OPERATION_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone)]
pub struct WalletDaemonClient {
    codex_home: PathBuf,
}

impl WalletDaemonClient {
    pub fn new(codex_home: PathBuf) -> Self {
        Self { codex_home }
    }

    pub async fn ensure_running(&self) -> Result<(), WalletDaemonError> {
        self.ensure_available().await?;
        match self
            .call_with_timeout(Request::ProtocolVersion, STATUS_TIMEOUT)
            .await
        {
            Ok(Response::ProtocolVersion { version })
                if version == crate::protocol::PROTOCOL_VERSION =>
            {
                Ok(())
            }
            Ok(Response::ProtocolVersion { .. }) => Err(upgrade_required(&self.codex_home)),
            Err(WalletDaemonError::Refused { code, .. }) if code == "invalid_request" => {
                Err(upgrade_required(&self.codex_home))
            }
            Ok(other) => response_error(other),
            Err(error) => Err(error),
        }
    }

    // A live socket is not proof that this installation's protocol is running.
    // Older releases leave a detached daemon alive when the TUI exits.
    async fn ensure_available(&self) -> Result<(), WalletDaemonError> {
        if matches!(
            self.call_with_timeout(Request::Ping, PING_TIMEOUT).await,
            Ok(Response::Pong)
        ) {
            return Ok(());
        }
        let executable = daemon_executable().map_err(unavailable)?;
        if !executable.is_file() {
            return Err(WalletDaemonError::Unavailable(format!(
                "required wallet daemon executable is missing from this Corbanu Terminal installation: {}",
                executable.display()
            )));
        }
        tokio::process::Command::new(executable)
            .arg("--codex-home")
            .arg(&self.codex_home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(false)
            .spawn()
            .map_err(unavailable)?;
        for _ in 0..40 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            if matches!(
                self.call_with_timeout(Request::Ping, PING_TIMEOUT).await,
                Ok(Response::Pong)
            ) {
                return Ok(());
            }
        }
        Err(WalletDaemonError::Unavailable(
            "startup timed out".to_string(),
        ))
    }

    pub async fn status(&self) -> Result<DaemonStatus, WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call_with_timeout(Request::Status, STATUS_TIMEOUT)
            .await?
        {
            Response::Status(status) => Ok(status),
            other => response_error(other),
        }
    }

    pub async fn unlock(
        &self,
        passcode: String,
        policy: UnlockPolicy,
    ) -> Result<(String, u64), WalletDaemonError> {
        self.ensure_running().await?;
        let (duration_seconds, one_action) = match policy {
            UnlockPolicy::OneAction => (5 * 60, true),
            UnlockPolicy::Timed { duration_seconds } => (duration_seconds, false),
        };
        match self
            .call_with_timeout(
                Request::Unlock {
                    passcode,
                    duration_seconds,
                    one_action,
                },
                LOCAL_OPERATION_TIMEOUT,
            )
            .await?
        {
            Response::Unlocked {
                capability,
                expires_in_seconds,
            } => Ok((capability, expires_in_seconds)),
            other => response_error(other),
        }
    }

    pub async fn lock(&self) -> Result<(), WalletDaemonError> {
        // Revocation remains available against a legacy daemon. Never use it
        // as an upgrade/drain handshake: cancellation is not payment completion.
        self.ensure_available().await?;
        match self
            .call_with_timeout(Request::Lock, STATUS_TIMEOUT)
            .await?
        {
            Response::Locked => Ok(()),
            other => response_error(other),
        }
    }

    pub async fn remove_wallet(&self, expected_address: String) -> Result<(), WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call_with_timeout(
                Request::RemoveWallet { expected_address },
                LOCAL_OPERATION_TIMEOUT,
            )
            .await?
        {
            Response::WalletRemoved => Ok(()),
            other => response_error(other),
        }
    }

    pub async fn sign_ownership(
        &self,
        capability: String,
        gateway_origin: String,
        challenge: String,
    ) -> Result<String, WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call_with_timeout(
                Request::SignOwnership {
                    capability,
                    gateway_origin,
                    challenge,
                },
                LOCAL_OPERATION_TIMEOUT,
            )
            .await?
        {
            Response::Signature { signature } => Ok(signature),
            other => response_error(other),
        }
    }

    pub async fn provision_plan(
        &self,
        capability: String,
        intent: PlanPurchaseIntent,
    ) -> Result<ProvisionedPlan, WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call_with_timeout(
                Request::ProvisionPlan { capability, intent },
                NETWORK_OPERATION_TIMEOUT,
            )
            .await?
        {
            Response::PlanProvisioned(result) => Ok(result),
            other => response_error(other),
        }
    }

    pub async fn issue_gateway_key(
        &self,
        capability: String,
        gateway_origin: String,
    ) -> Result<GatewayKey, WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call_with_timeout(
                Request::IssueGatewayKey {
                    capability,
                    gateway_origin,
                },
                NETWORK_OPERATION_TIMEOUT,
            )
            .await?
        {
            Response::GatewayKeyIssued(result) => Ok(result),
            other => response_error(other),
        }
    }

    pub async fn execute_corbanu_api_operation(
        &self,
        capability: String,
        gateway_origin: String,
        operation: CorbanuApiOperation,
    ) -> Result<CorbanuApiOperationResult, WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call_with_timeout(
                Request::CorbanuApiOperation {
                    capability,
                    gateway_origin,
                    operation,
                },
                NETWORK_OPERATION_TIMEOUT,
            )
            .await?
        {
            Response::CorbanuApiOperationCompleted(result) => Ok(result),
            other => response_error(other),
        }
    }

    async fn call_with_timeout(
        &self,
        request: Request,
        timeout: Duration,
    ) -> Result<Response, WalletDaemonError> {
        tokio::time::timeout(timeout, self.call_unbounded(request))
            .await
            .map_err(|_| {
                WalletDaemonError::Unavailable(format!(
                    "request timed out after {} second(s)",
                    timeout.as_secs_f64()
                ))
            })?
    }

    async fn call_unbounded(&self, request: Request) -> Result<Response, WalletDaemonError> {
        let mut stream = UnixStream::connect(socket_path(&self.codex_home))
            .await
            .map_err(unavailable)?;
        let mut payload = serde_json::to_vec(&request).map_err(unavailable)?;
        payload.push(b'\n');
        stream.write_all(&payload).await.map_err(unavailable)?;
        stream.flush().await.map_err(unavailable)?;
        let mut line = String::new();
        BufReader::new(stream)
            .read_line(&mut line)
            .await
            .map_err(unavailable)?;
        let response: Response = serde_json::from_str(&line).map_err(unavailable)?;
        match response {
            Response::Error { code, message } => Err(WalletDaemonError::Refused { code, message }),
            other => Ok(other),
        }
    }
}

pub(crate) fn socket_path(home: &Path) -> PathBuf {
    home.join("wallet/run/walletd.sock")
}
pub(crate) fn run_dir(home: &Path) -> PathBuf {
    home.join("wallet/run")
}

fn daemon_executable() -> std::io::Result<PathBuf> {
    let current = std::env::current_exe()?;
    let current = current.canonicalize().unwrap_or(current);
    Ok(daemon_executable_beside(&current))
}

fn daemon_executable_beside(current: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "pfterminal-walletd.exe"
    } else {
        "pfterminal-walletd"
    };
    current
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(name)
}
fn unavailable(error: impl std::fmt::Display) -> WalletDaemonError {
    WalletDaemonError::Unavailable(error.to_string())
}

fn upgrade_required(home: &Path) -> WalletDaemonError {
    WalletDaemonError::Refused {
        code: "daemon_upgrade_required".into(),
        message: format!(
            "A wallet daemon from a different Corbanu Terminal release is still running. No new wallet operation was sent. Let any existing payment finish and verify its outcome; then close Terminal sessions for this home and stop only the pfterminal-walletd (or corbanu-walletd) process whose --codex-home is {} using your operating system's process manager. Reopen Terminal to start the matching daemon. Restarting only the TUI does not stop the daemon. Do not delete the wallet directory, socket or ownership lock, and do not repeat a payment whose outcome is unknown.",
            home.display()
        ),
    }
}
fn response_error<T>(response: Response) -> Result<T, WalletDaemonError> {
    match response {
        Response::Error { code, message } => Err(WalletDaemonError::Refused { code, message }),
        _ => Err(WalletDaemonError::Protocol),
    }
}

#[cfg(test)]
#[path = "client_upgrade_tests.rs"]
mod upgrade_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use codex_uds::UnixListener;
    use codex_uds::prepare_private_socket_directory;

    #[test]
    fn wallet_daemon_is_resolved_beside_the_running_executable() {
        let executable = if cfg!(windows) {
            Path::new(r"C:\PFTerminal\bin\pfterminal.exe")
        } else {
            Path::new("/opt/pfterminal/bin/pfterminal")
        };
        let expected = if cfg!(windows) {
            Path::new(r"C:\PFTerminal\bin\pfterminal-walletd.exe")
        } else {
            Path::new("/opt/pfterminal/bin/pfterminal-walletd")
        };

        assert_eq!(daemon_executable_beside(executable), expected);
    }

    #[tokio::test]
    async fn an_accepted_request_that_never_replies_times_out() {
        let home = tempfile::tempdir().expect("home");
        tokio::fs::create_dir_all(home.path().join("wallet"))
            .await
            .expect("wallet directory");
        prepare_private_socket_directory(run_dir(home.path()))
            .await
            .expect("private run directory");
        let mut listener = UnixListener::bind(socket_path(home.path()))
            .await
            .expect("bind socket");
        let stalled_server = tokio::spawn(async move {
            let _stream = listener.accept().await.expect("accept client");
            std::future::pending::<()>().await;
        });
        let client = WalletDaemonClient::new(home.path().to_path_buf());
        let error = client
            .call_with_timeout(Request::Ping, Duration::from_millis(25))
            .await
            .expect_err("stalled daemon must time out");
        assert!(error.to_string().contains("timed out"));
        stalled_server.abort();
    }
}
