use crate::BrowserError;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Child;
use tokio::process::Command;

/// Trusted host CLI context, never forwarded as container environment.
#[derive(Clone)]
pub(crate) struct EngineCommand {
    pub executable: AbsolutePathBuf,
    pub environment: BTreeMap<OsString, OsString>,
}

impl EngineCommand {
    pub fn spawn(&self, args: &[String]) -> Result<Child, BrowserError> {
        Command::new(self.executable.as_path())
            .args(args)
            .env_clear()
            .envs(&self.environment)
            .current_dir(std::env::temp_dir())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| BrowserError::RuntimeUnavailable)
    }

    pub async fn run(&self, args: &[String], deadline: Duration) -> Result<Vec<u8>, BrowserError> {
        let mut child = self.spawn(args)?;
        drop(child.stdin.take());
        let mut stdout = child
            .stdout
            .take()
            .ok_or(BrowserError::RuntimeUnavailable)?;
        let operation = async {
            let mut bytes = Vec::new();
            (&mut stdout)
                .take(131_073)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| BrowserError::RuntimeUnavailable)?;
            if bytes.len() > 131_072 {
                return Err(BrowserError::ResourceLimit);
            }
            if !child
                .wait()
                .await
                .map_err(|_| BrowserError::RuntimeUnavailable)?
                .success()
            {
                return Err(BrowserError::RuntimeUnavailable);
            }
            Ok(bytes)
        };
        tokio::time::timeout(deadline, operation)
            .await
            .map_err(|_| BrowserError::ResourceLimit)?
    }
}

pub(crate) async fn write_json(
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
    value: &impl serde::Serialize,
) -> Result<(), BrowserError> {
    let mut bytes = serde_json::to_vec(value).map_err(|_| BrowserError::InvalidWorkerResponse)?;
    bytes.push(b'\n');
    writer
        .write_all(&bytes)
        .await
        .map_err(|_| BrowserError::InvalidWorkerResponse)?;
    writer
        .flush()
        .await
        .map_err(|_| BrowserError::InvalidWorkerResponse)
}
