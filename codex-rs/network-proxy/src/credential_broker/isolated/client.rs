//! Controller-side handle for the isolated credential broker process.
//!
//! Core keeps only opaque references and signs typed provider requests; it has
//! no resolver that can turn a reference back into a raw credential.

use super::protocol::BROKER_SESSION_ID;
use super::protocol::BROKER_TASK_ID;
use super::protocol::CONTROL_PROTOCOL_VERSION;
use super::protocol::ControlRequest;
use super::protocol::ControlResponse;
use super::protocol::FRAME_HEADER;
use super::protocol::HostBindingWire;
use super::protocol::MAX_CONTROL_LINE_BYTES;
use super::protocol::ProviderId;
use super::protocol::encode_hex;
use super::protocol::valid_id;
use crate::credential_broker::providers;
use crate::upstream::UpstreamClient;
use base64::Engine as _;
use codex_secret_broker::BrokerBinding;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::ProviderRequestOperation;
use rama_core::Service as _;
use rama_http::HeaderValue;
use rama_http::Request;
use rama_http::Response;
use rama_http::Version;
use rama_http::header::HOST;
use rand::RngCore as _;
use std::ffi::OsString;
use std::fmt;
use std::io::BufRead as _;
use std::io::BufReader;
use std::io::Read as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use thiserror::Error;
use zeroize::Zeroizing;

/// Hidden argv[1] that turns the Corbanu executable into the broker process.
pub const CODEX_CREDENTIAL_BROKER_ARG1: &str = "--codex-run-as-credential-broker";

const CONTROL_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_HARNESS_PREAMBLE_LINES: usize = 16;
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct IsolatedBrokerOptions {
    pub(crate) allow_local_binding: bool,
    pub(crate) allow_upstream_proxy: bool,
}

/// How Core starts the broker. Production re-executes the current binary.
#[derive(Clone, Debug)]
pub(crate) struct IsolatedBrokerLauncher {
    program: Option<PathBuf>,
    args: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
    skip_harness_preamble: bool,
    controller_pid_override: Option<u32>,
}

impl IsolatedBrokerLauncher {
    pub(crate) fn current_exe() -> Self {
        Self {
            program: None,
            args: vec![OsString::from(CODEX_CREDENTIAL_BROKER_ARG1)],
            envs: Vec::new(),
            skip_harness_preamble: false,
            controller_pid_override: None,
        }
    }

    /// Re-executes a libtest binary filtered to one child-entry test. The test
    /// harness prints a header before the broker starts, which is skipped.
    #[cfg(test)]
    pub(crate) fn test_harness(
        program: Option<PathBuf>,
        args: Vec<OsString>,
        envs: Vec<(OsString, OsString)>,
        controller_pid_override: Option<u32>,
    ) -> Self {
        Self {
            program,
            args,
            envs,
            skip_harness_preamble: true,
            controller_pid_override,
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub(crate) enum IsolatedBrokerError {
    #[error("credential broker could not be started")]
    Spawn,
    #[error("credential broker control channel failed")]
    Control,
    #[error("credential broker rejected the request")]
    Rejected,
    #[error("credential broker is unavailable")]
    Unavailable,
}

struct ControlChannel {
    stdin: Option<ChildStdin>,
    lines: mpsc::Receiver<std::io::Result<Zeroizing<Vec<u8>>>>,
}

pub(crate) struct IsolatedBrokerClient {
    control: Mutex<ControlChannel>,
    child: Mutex<Option<Child>>,
    mac: BrokerChannelMac,
    controller_instance: String,
    broker_instance: String,
    socket_path: PathBuf,
    run_generation: AtomicU64,
    next_sequence: AtomicU64,
    alive: AtomicBool,
}

impl fmt::Debug for IsolatedBrokerClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("IsolatedBrokerClient(<authenticated>)")
    }
}

impl IsolatedBrokerClient {
    pub(crate) fn spawn(
        launcher: &IsolatedBrokerLauncher,
        options: IsolatedBrokerOptions,
    ) -> Result<Self, IsolatedBrokerError> {
        let program = match launcher.program.clone() {
            Some(program) => program,
            None => std::env::current_exe().map_err(|_| IsolatedBrokerError::Spawn)?,
        };
        let mut command = Command::new(program);
        command
            .args(&launcher.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // Own process group: terminal hangups and interrupts aimed at the TUI
        // must not kill the broker before it removes its socket on stdin EOF.
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        // Raw values reach the broker only through the private control pipe.
        for key in providers::credential_broker_env_keys() {
            command.env_remove(key);
        }
        for (key, value) in &launcher.envs {
            command.env(key, value);
        }
        let mut child = command.spawn().map_err(|_| IsolatedBrokerError::Spawn)?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        };
        let (sender, lines) = mpsc::channel();
        std::thread::Builder::new()
            .name("credential-broker-control".to_string())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let line = read_control_line(&mut reader);
                    let stop = !matches!(line, Ok(Some(_)));
                    let message = match line {
                        Ok(Some(line)) => Ok(line),
                        Ok(None) => Err(std::io::ErrorKind::UnexpectedEof.into()),
                        Err(error) => Err(error),
                    };
                    if sender.send(message).is_err() || stop {
                        break;
                    }
                }
            })
            .map_err(|_| IsolatedBrokerError::Spawn)?;

        let mut key = Zeroizing::new([0_u8; 32]);
        rand::rng().fill_bytes(key.as_mut());
        let controller_instance = format!("controller-{}", random_hex::<8>());
        let mut control = ControlChannel {
            stdin: Some(stdin),
            lines,
        };
        let hello = ControlRequest::Hello {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            controller_pid: launcher
                .controller_pid_override
                .unwrap_or_else(std::process::id),
            controller_instance: controller_instance.clone(),
            channel_key: encode_hex(key.as_ref()),
            allow_local_binding: options.allow_local_binding,
            allow_upstream_proxy: options.allow_upstream_proxy,
        };
        let ready = control
            .send(&hello)
            .and_then(|()| control.receive(launcher.skip_harness_preamble));
        drop(hello);
        let Ok(ControlResponse::Ready {
            protocol_version,
            broker_instance,
            socket_path,
            run_generation,
        }) = ready
        else {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        };
        let socket_path = PathBuf::from(socket_path);
        if protocol_version != CONTROL_PROTOCOL_VERSION
            || !valid_id(&broker_instance)
            || !socket_path.is_absolute()
            || run_generation != 1
        {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        }
        Ok(Self {
            control: Mutex::new(control),
            child: Mutex::new(Some(child)),
            mac: BrokerChannelMac::from_secret(*key),
            controller_instance,
            broker_instance,
            socket_path,
            run_generation: AtomicU64::new(run_generation),
            next_sequence: AtomicU64::new(1),
            alive: AtomicBool::new(true),
        })
    }

    #[cfg(test)]
    pub(crate) fn broker_instance(&self) -> &str {
        &self.broker_instance
    }

    #[cfg(test)]
    pub(crate) fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }

    #[cfg(test)]
    pub(crate) fn run_generation(&self) -> u64 {
        self.run_generation.load(Ordering::Acquire)
    }

    /// Hands one raw value to the broker and returns its opaque reference.
    pub(crate) fn register(
        &self,
        provider: ProviderId,
        binding: HostBindingWire,
        value: &str,
    ) -> Result<CredentialReference, IsolatedBrokerError> {
        let response = self.call(&ControlRequest::Register {
            provider,
            binding,
            value: value.to_string(),
        })?;
        match response {
            ControlResponse::Registered {
                reference,
                run_generation,
            } if run_generation == self.run_generation.load(Ordering::Acquire) => {
                CredentialReference::from_sha256_hex(reference)
                    .map_err(|_| self.fail(IsolatedBrokerError::Control))
            }
            ControlResponse::Error { .. } => Err(IsolatedBrokerError::Rejected),
            _ => Err(self.fail(IsolatedBrokerError::Control)),
        }
    }

    /// Invalidates every reference and closes in-flight broker channels.
    pub(crate) fn revoke(&self) -> Result<u64, IsolatedBrokerError> {
        match self.call(&ControlRequest::Revoke)? {
            ControlResponse::Revoked { run_generation }
                if run_generation > self.run_generation.load(Ordering::Acquire) =>
            {
                self.run_generation.store(run_generation, Ordering::Release);
                Ok(run_generation)
            }
            _ => Err(self.fail(IsolatedBrokerError::Control)),
        }
    }

    pub(crate) fn is_alive(&self) -> bool {
        if !self.alive.load(Ordering::Acquire) {
            return false;
        }
        let exited = match self.child.lock() {
            Ok(mut child) => child
                .as_mut()
                .is_none_or(|child| !matches!(child.try_wait(), Ok(None))),
            Err(_) => true,
        };
        if exited && self.alive.swap(false, Ordering::AcqRel) {
            // A crashed broker cannot remove its own socket directory.
            remove_socket_dir(&self.socket_path);
        }
        !exited
    }

    pub(crate) fn sign_frame(
        &self,
        credential: &CredentialReference,
        operation: &ProviderRequestOperation,
    ) -> Result<String, IsolatedBrokerError> {
        let sequence = self.next_sequence.fetch_add(1, Ordering::AcqRel);
        let frame = self
            .mac
            .sign_provider_request(
                self.binding(self.run_generation.load(Ordering::Acquire)),
                sequence,
                credential.clone(),
                operation.clone(),
            )
            .map_err(|_| IsolatedBrokerError::Rejected)?;
        Ok(base64::engine::general_purpose::STANDARD.encode(frame.as_bytes()))
    }

    pub(crate) fn binding(&self, run_generation: u64) -> BrokerBinding {
        BrokerBinding {
            controller_instance: self.controller_instance.clone(),
            worker_instance: self.broker_instance.clone(),
            session_id: BROKER_SESSION_ID.to_string(),
            task_id: BROKER_TASK_ID.to_string(),
            run_id: self.controller_instance.clone(),
            run_generation,
        }
    }

    /// Sends a typed provider request through the broker. The broker replaces
    /// the provider credential header and performs the upstream request.
    pub(crate) async fn forward(
        &self,
        credential: &CredentialReference,
        operation: &ProviderRequestOperation,
        request: Request,
    ) -> Result<Response, IsolatedBrokerError> {
        if !self.is_alive() {
            return Err(IsolatedBrokerError::Unavailable);
        }
        let frame = self.sign_frame(credential, operation)?;
        self.send_with_frame(operation, request, &frame).await
    }

    pub(crate) async fn send_with_frame(
        &self,
        operation: &ProviderRequestOperation,
        request: Request,
        frame: &str,
    ) -> Result<Response, IsolatedBrokerError> {
        let (mut parts, body) = request.into_parts();
        parts.version = Version::HTTP_11;
        parts.uri = operation
            .path()
            .parse()
            .map_err(|_| IsolatedBrokerError::Rejected)?;
        parts.headers.insert(
            FRAME_HEADER,
            HeaderValue::from_str(frame).map_err(|_| IsolatedBrokerError::Rejected)?,
        );
        parts.headers.insert(
            HOST,
            HeaderValue::from_str(operation.host()).map_err(|_| IsolatedBrokerError::Rejected)?,
        );
        let socket_path = self.socket_path.to_string_lossy().into_owned();
        UpstreamClient::unix_socket(&socket_path)
            .serve(Request::from_parts(parts, body))
            .await
            .map_err(|_| {
                if self.is_alive() {
                    IsolatedBrokerError::Unavailable
                } else {
                    self.fail(IsolatedBrokerError::Unavailable)
                }
            })
    }

    /// Signs a frame for an arbitrary binding, for cross-run qualification.
    #[cfg(test)]
    pub(crate) fn sign_frame_with_binding(
        &self,
        binding: BrokerBinding,
        credential: &CredentialReference,
        operation: &ProviderRequestOperation,
    ) -> String {
        let sequence = self.next_sequence.fetch_add(1, Ordering::AcqRel);
        let frame = self
            .mac
            .sign_provider_request(binding, sequence, credential.clone(), operation.clone())
            .expect("test frame");
        base64::engine::general_purpose::STANDARD.encode(frame.as_bytes())
    }

    #[cfg(test)]
    pub(crate) fn pid_for_test(&self) -> Option<u32> {
        self.child
            .lock()
            .ok()
            .and_then(|child| child.as_ref().map(Child::id))
    }

    /// Terminates the broker immediately. Used for crash qualification.
    #[cfg(test)]
    pub(crate) fn kill_for_test(&self) {
        if let Ok(mut child) = self.child.lock()
            && let Some(child) = child.as_mut()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn call(&self, request: &ControlRequest) -> Result<ControlResponse, IsolatedBrokerError> {
        if !self.is_alive() {
            return Err(IsolatedBrokerError::Unavailable);
        }
        let result = match self.control.lock() {
            Ok(mut control) => control
                .send(request)
                .and_then(|()| control.receive(/*skip_harness_preamble*/ false)),
            Err(_) => Err(IsolatedBrokerError::Control),
        };
        result.map_err(|error| self.fail(error))
    }

    fn fail(&self, error: IsolatedBrokerError) -> IsolatedBrokerError {
        self.alive.store(false, Ordering::Release);
        if let Ok(mut control) = self.control.lock() {
            control.stdin.take();
        }
        if let Ok(mut child) = self.child.lock()
            && let Some(child) = child.take()
        {
            kill_and_reap_with_cleanup(child, Some(self.socket_path.clone()));
        }
        error
    }
}

impl Drop for IsolatedBrokerClient {
    fn drop(&mut self) {
        // Closing stdin is the broker's shutdown signal; kill as a backstop.
        if let Ok(control) = self.control.get_mut() {
            control.stdin.take();
        }
        if let Ok(child) = self.child.get_mut()
            && let Some(child) = child.take()
        {
            reap_after_grace(child, self.socket_path.clone());
        }
    }
}

impl ControlChannel {
    fn send(&mut self, request: &ControlRequest) -> Result<(), IsolatedBrokerError> {
        let stdin = self.stdin.as_mut().ok_or(IsolatedBrokerError::Control)?;
        let mut line =
            Zeroizing::new(serde_json::to_vec(request).map_err(|_| IsolatedBrokerError::Control)?);
        if line.len() > MAX_CONTROL_LINE_BYTES {
            return Err(IsolatedBrokerError::Rejected);
        }
        line.push(b'\n');
        stdin
            .write_all(&line)
            .and_then(|()| stdin.flush())
            .map_err(|_| IsolatedBrokerError::Control)
    }

    fn receive(
        &mut self,
        skip_harness_preamble: bool,
    ) -> Result<ControlResponse, IsolatedBrokerError> {
        let mut skipped = 0;
        loop {
            let line = self
                .lines
                .recv_timeout(CONTROL_TIMEOUT)
                .map_err(|_| IsolatedBrokerError::Control)?
                .map_err(|_| IsolatedBrokerError::Control)?;
            let mut message = &line[..];
            if skip_harness_preamble && skipped < MAX_HARNESS_PREAMBLE_LINES {
                // libtest prints its own header before the child entry runs.
                match line.iter().position(|byte| *byte == b'{') {
                    Some(start) => message = &line[start..],
                    None => {
                        skipped += 1;
                        continue;
                    }
                }
            }
            return serde_json::from_slice(message).map_err(|_| IsolatedBrokerError::Control);
        }
    }
}

fn read_control_line(
    reader: &mut BufReader<std::process::ChildStdout>,
) -> std::io::Result<Option<Zeroizing<Vec<u8>>>> {
    let mut line = Zeroizing::new(Vec::new());
    let read = reader
        .by_ref()
        .take(MAX_CONTROL_LINE_BYTES as u64 + 1)
        .read_until(b'\n', &mut line)?;
    if read == 0 {
        return Ok(None);
    }
    if line.last() != Some(&b'\n') {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    line.pop();
    Ok(Some(line))
}

/// Gives the broker a short grace period to observe stdin EOF and remove its
/// socket directory, then kills it.
fn reap_after_grace(mut child: Child, socket_path: PathBuf) {
    let _ = std::thread::Builder::new()
        .name("credential-broker-reaper".to_string())
        .spawn(move || {
            let deadline = std::time::Instant::now() + SHUTDOWN_GRACE;
            while std::time::Instant::now() < deadline {
                if !matches!(child.try_wait(), Ok(None)) {
                    remove_socket_dir(&socket_path);
                    return;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = child.kill();
            let _ = child.wait();
            remove_socket_dir(&socket_path);
        });
}

fn kill_and_reap(child: Child) {
    kill_and_reap_with_cleanup(child, /*socket_path*/ None);
}

/// Kills the broker and, once it is reaped, removes a socket directory it
/// could not clean up itself.
fn kill_and_reap_with_cleanup(mut child: Child, socket_path: Option<PathBuf>) {
    let _ = child.kill();
    let _ = std::thread::Builder::new()
        .name("credential-broker-reaper".to_string())
        .spawn(move || {
            let _ = child.wait();
            if let Some(socket_path) = socket_path {
                remove_socket_dir(&socket_path);
            }
        });
}

fn remove_socket_dir(socket_path: &std::path::Path) {
    let Some(dir) = socket_path.parent() else {
        return;
    };
    let owned = dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("cbk-"));
    if owned {
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_dir(dir);
    }
}

fn random_hex<const N: usize>() -> String {
    let mut bytes = [0_u8; N];
    rand::rng().fill_bytes(&mut bytes);
    encode_hex(&bytes)
}
