use crate::GpuInstance;
use crate::ProviderError;
use crate::ProviderErrorKind;
use crate::ProviderResult;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

const SSH_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) struct VastSshIdentity {
    pub(crate) private_key: PathBuf,
    pub(crate) public_key: String,
}

struct SshTunnel {
    child: Child,
    endpoint: String,
}

pub(crate) struct VastSshTransport {
    root: PathBuf,
    tunnels: Mutex<HashMap<String, SshTunnel>>,
}

impl VastSshTransport {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            root,
            tunnels: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn identity(&self) -> ProviderResult<VastSshIdentity> {
        std::fs::create_dir_all(&self.root).map_err(local_transport_error)?;
        set_private_directory_permissions(&self.root).map_err(local_transport_error)?;
        let private_key = self.root.join("id_ed25519");
        let public_key_path = self.root.join("id_ed25519.pub");
        match (private_key.exists(), public_key_path.exists()) {
            (false, false) => generate_identity(&private_key)?,
            (true, false) => derive_public_key(&private_key, &public_key_path)?,
            (false, true) => {
                return Err(ProviderError::new(
                    ProviderErrorKind::Permanent,
                    "The Vast SSH identity is incomplete; no rental endpoint was exposed.",
                ));
            }
            (true, true) => {}
        }
        set_private_file_permissions(&private_key).map_err(local_transport_error)?;
        let public_key = std::fs::read_to_string(&public_key_path)
            .map_err(local_transport_error)?
            .trim()
            .to_string();
        if public_key.len() > 16_384 || ssh_public_key_identity(public_key.as_str()).is_none() {
            return Err(ProviderError::new(
                ProviderErrorKind::Permanent,
                "The Vast SSH public key is invalid; no rental endpoint was exposed.",
            ));
        }
        Ok(VastSshIdentity {
            private_key,
            public_key,
        })
    }

    pub(crate) async fn endpoint(
        &self,
        instance: &GpuInstance,
        inference_port: u16,
        private_key: &Path,
    ) -> ProviderResult<String> {
        if let Some(endpoint) = self.running_endpoint(instance.resource_id.as_str())? {
            return Ok(endpoint);
        }
        let ssh_host = instance.public_ip.as_deref().ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Retryable,
                "Vast has not assigned the direct SSH address yet.",
            )
            .with_retry_after_ms(5_000)
        })?;
        let ssh_port = instance.ssh_port.ok_or_else(|| {
            ProviderError::new(
                ProviderErrorKind::Retryable,
                "Vast has not assigned the direct SSH port yet.",
            )
            .with_retry_after_ms(5_000)
        })?;
        let local_port = available_loopback_port()?;
        let endpoint = format!("http://127.0.0.1:{local_port}/v1");
        let known_hosts = self
            .root
            .join(format!("known-hosts-{}", instance.resource_id));
        let log_path = self.root.join(format!("ssh-{}.log", instance.resource_id));
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(local_transport_error)?;
        let stderr = log.try_clone().map_err(local_transport_error)?;
        let mut child = Command::new("ssh")
            .arg("-N")
            .arg("-T")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("ExitOnForwardFailure=yes")
            .arg("-o")
            .arg("ServerAliveInterval=15")
            .arg("-o")
            .arg("ServerAliveCountMax=3")
            .arg("-o")
            .arg("ConnectTimeout=10")
            .arg("-o")
            .arg("StrictHostKeyChecking=accept-new")
            .arg("-o")
            .arg(format!("UserKnownHostsFile={}", known_hosts.display()))
            .arg("-i")
            .arg(private_key)
            .arg("-p")
            .arg(ssh_port.to_string())
            .arg("-L")
            .arg(format!("127.0.0.1:{local_port}:127.0.0.1:{inference_port}"))
            .arg(format!("root@{ssh_host}"))
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|_| {
                ProviderError::new(
                    ProviderErrorKind::Permanent,
                    "OpenSSH is unavailable; no Vast rental endpoint was exposed.",
                )
            })?;
        let deadline = Instant::now() + SSH_CONNECT_TIMEOUT;
        loop {
            if let Some(_status) = child.try_wait().map_err(local_transport_error)? {
                return Err(ProviderError::new(
                    ProviderErrorKind::Retryable,
                    "The authenticated Vast SSH forward has not started yet.",
                )
                .with_retry_after_ms(5_000));
            }
            if TcpStream::connect_timeout(
                &SocketAddrV4::new(Ipv4Addr::LOCALHOST, local_port).into(),
                Duration::from_millis(200),
            )
            .is_ok()
            {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProviderError::new(
                    ProviderErrorKind::Retryable,
                    "The authenticated Vast SSH forward has not started yet.",
                )
                .with_retry_after_ms(5_000));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let mut tunnels = self.tunnels.lock().map_err(|_| lock_error())?;
        if let Some(mut stale) = tunnels.insert(
            instance.resource_id.clone(),
            SshTunnel {
                child,
                endpoint: endpoint.clone(),
            },
        ) {
            let _ = stale.child.kill();
            let _ = stale.child.wait();
        }
        Ok(endpoint)
    }

    pub(crate) fn stop(&self, resource_id: &str) {
        if let Ok(mut tunnels) = self.tunnels.lock()
            && let Some(mut tunnel) = tunnels.remove(resource_id)
        {
            let _ = tunnel.child.kill();
            let _ = tunnel.child.wait();
        }
    }

    fn running_endpoint(&self, resource_id: &str) -> ProviderResult<Option<String>> {
        let mut tunnels = self.tunnels.lock().map_err(|_| lock_error())?;
        let Some(tunnel) = tunnels.get_mut(resource_id) else {
            return Ok(None);
        };
        match tunnel.child.try_wait().map_err(local_transport_error)? {
            None => Ok(Some(tunnel.endpoint.clone())),
            Some(_) => {
                tunnels.remove(resource_id);
                Ok(None)
            }
        }
    }
}

impl Drop for VastSshTransport {
    fn drop(&mut self) {
        if let Ok(tunnels) = self.tunnels.get_mut() {
            for tunnel in tunnels.values_mut() {
                let _ = tunnel.child.kill();
                let _ = tunnel.child.wait();
            }
        }
    }
}

pub(crate) fn ssh_public_key_identity(value: &str) -> Option<(&str, &str)> {
    let mut parts = value.split_whitespace();
    let algorithm = parts.next()?;
    let encoded = parts.next()?;
    if !algorithm.starts_with("ssh-") || encoded.is_empty() {
        return None;
    }
    Some((algorithm, encoded))
}

fn generate_identity(private_key: &Path) -> ProviderResult<()> {
    let status = Command::new("ssh-keygen")
        .arg("-q")
        .arg("-t")
        .arg("ed25519")
        .arg("-N")
        .arg("")
        .arg("-C")
        .arg("pfterminal-vast")
        .arg("-f")
        .arg(private_key)
        .status()
        .map_err(|_| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "ssh-keygen is unavailable; no Vast rental endpoint was exposed.",
            )
        })?;
    if !status.success() {
        return Err(ProviderError::new(
            ProviderErrorKind::Permanent,
            "PfTerminal could not create its Vast SSH identity.",
        ));
    }
    Ok(())
}

fn derive_public_key(private_key: &Path, public_key: &Path) -> ProviderResult<()> {
    let output = Command::new("ssh-keygen")
        .arg("-y")
        .arg("-f")
        .arg(private_key)
        .output()
        .map_err(|_| {
            ProviderError::new(
                ProviderErrorKind::Permanent,
                "ssh-keygen is unavailable; no Vast rental endpoint was exposed.",
            )
        })?;
    if !output.status.success() {
        return Err(ProviderError::new(
            ProviderErrorKind::Permanent,
            "PfTerminal could not recover its Vast SSH public key.",
        ));
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(public_key)
        .map_err(local_transport_error)?;
    file.write_all(&output.stdout)
        .map_err(local_transport_error)?;
    Ok(())
}

fn available_loopback_port() -> ProviderResult<u16> {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .map_err(local_transport_error)
}

fn local_transport_error(_error: std::io::Error) -> ProviderError {
    ProviderError::new(
        ProviderErrorKind::Permanent,
        "PfTerminal could not initialize its local Vast SSH transport.",
    )
}

fn lock_error() -> ProviderError {
    ProviderError::new(
        ProviderErrorKind::Permanent,
        "PfTerminal's local Vast SSH transport lock is unavailable.",
    )
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_directory_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
#[path = "vast_ssh_tests.rs"]
mod tests;
