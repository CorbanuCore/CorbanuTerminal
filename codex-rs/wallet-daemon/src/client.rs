use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use codex_uds::UnixStream;
use codex_wallet::GatewayKey;
use codex_wallet::PlanPurchaseIntent;
use codex_wallet::ProvisionedPlan;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;

use crate::protocol::DaemonStatus;
use crate::protocol::Request;
use crate::protocol::Response;
use crate::protocol::WalletDaemonError;

#[derive(Clone)]
pub struct WalletDaemonClient {
    codex_home: PathBuf,
}

impl WalletDaemonClient {
    pub fn new(codex_home: PathBuf) -> Self {
        Self { codex_home }
    }

    pub async fn ensure_running(&self) -> Result<(), WalletDaemonError> {
        if matches!(self.call(Request::Ping).await, Ok(Response::Pong)) {
            return Ok(());
        }
        let executable = daemon_executable().map_err(unavailable)?;
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
            if matches!(self.call(Request::Ping).await, Ok(Response::Pong)) {
                return Ok(());
            }
        }
        Err(WalletDaemonError::Unavailable(
            "startup timed out".to_string(),
        ))
    }

    pub async fn status(&self) -> Result<DaemonStatus, WalletDaemonError> {
        self.ensure_running().await?;
        match self.call(Request::Status).await? {
            Response::Status(status) => Ok(status),
            other => response_error(other),
        }
    }

    pub async fn unlock(
        &self,
        passcode: String,
        duration_seconds: u64,
    ) -> Result<(String, u64), WalletDaemonError> {
        self.ensure_running().await?;
        match self
            .call(Request::Unlock {
                passcode,
                duration_seconds,
            })
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
        self.ensure_running().await?;
        match self.call(Request::Lock).await? {
            Response::Locked => Ok(()),
            other => response_error(other),
        }
    }

    pub async fn sign_ownership(
        &self,
        capability: String,
        gateway_origin: String,
        challenge: String,
    ) -> Result<String, WalletDaemonError> {
        match self
            .call(Request::SignOwnership {
                capability,
                gateway_origin,
                challenge,
            })
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
        match self
            .call(Request::ProvisionPlan { capability, intent })
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
        match self
            .call(Request::IssueGatewayKey {
                capability,
                gateway_origin,
            })
            .await?
        {
            Response::GatewayKeyIssued(result) => Ok(result),
            other => response_error(other),
        }
    }

    async fn call(&self, request: Request) -> Result<Response, WalletDaemonError> {
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
    let name = if cfg!(windows) {
        "pfterminal-walletd.exe"
    } else {
        "pfterminal-walletd"
    };
    Ok(current
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(name))
}
fn unavailable(error: impl std::fmt::Display) -> WalletDaemonError {
    WalletDaemonError::Unavailable(error.to_string())
}
fn response_error<T>(response: Response) -> Result<T, WalletDaemonError> {
    match response {
        Response::Error { code, message } => Err(WalletDaemonError::Refused { code, message }),
        _ => Err(WalletDaemonError::Protocol),
    }
}
