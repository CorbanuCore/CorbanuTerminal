//! Controller-side handle for the isolated credential broker process.
//!
//! Core keeps only opaque references and signs typed provider requests; it has
//! no resolver that can turn a reference back into a raw credential.

use super::protocol::BROKER_RUNTIME_DIR_ENV;
use super::protocol::BROKER_SESSION_ID;
use super::protocol::BROKER_TASK_ID;
use super::protocol::BrokerBootstrap;
use super::protocol::CONTROL_PROTOCOL_VERSION;
use super::protocol::ControlErrorCode;
use super::protocol::ControlRequest;
use super::protocol::ControlResponse;
use super::protocol::FRAME_HEADER;
use super::protocol::HostBindingWire;
use super::protocol::MAX_CONTROL_LINE_BYTES;
use super::protocol::ModelBindingWire;
use super::protocol::ProviderId;
use super::protocol::StoredKeyAccount;
use super::protocol::encode_hex;
use super::protocol::valid_id;
use crate::connect_policy::PinnedPeers;
use crate::credential_broker::providers;
use crate::upstream::UpstreamClient;
use base64::Engine as _;
#[cfg(windows)]
use codex_process_hardening::ProtectedChild as Child;
use codex_secret_broker::BrokerBinding;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::ProviderRequestOperation;
use rama_core::Service as _;
use rama_core::extensions::ExtensionsRef as _;
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
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Child;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct IsolatedBrokerOptions {
    pub(crate) allow_local_binding: bool,
    pub(crate) allow_upstream_proxy: bool,
    /// PF-27-S02: parent of the broker's private socket directory. Core
    /// passes `CODEX_HOME/run`, which agent commands cannot write.
    pub(crate) runtime_dir: Option<PathBuf>,
    /// PF-27-S02: refuse a broker that could not confine itself (Seatbelt on
    /// macOS, at least seccomp on Linux) instead of running it unconfined.
    pub(crate) require_containment: bool,
    /// PF-28-S02: scrub the broker's credentials from the responses it
    /// returns.
    pub(crate) scrub_responses: bool,
    /// PF-33-S02: every provider request must carry the destination guard's
    /// checked answers, which the broker dials instead of resolving.
    pub(crate) pin_connections: bool,
}

/// How Core starts the broker. Production re-executes the current binary.
#[derive(Clone, Debug)]
pub(crate) struct IsolatedBrokerLauncher {
    program: Option<PathBuf>,
    args: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
    /// PF-27-S05: variables the broker must not inherit.
    removed_envs: Vec<OsString>,
    skip_harness_preamble: bool,
    controller_pid_override: Option<u32>,
}

impl IsolatedBrokerLauncher {
    pub(crate) fn current_exe() -> Self {
        Self {
            program: None,
            args: vec![OsString::from(CODEX_CREDENTIAL_BROKER_ARG1)],
            envs: Vec::new(),
            removed_envs: Vec::new(),
            skip_harness_preamble: false,
            controller_pid_override: None,
        }
    }

    /// Runs `program` (a Corbanu executable) instead of the current one.
    pub(crate) fn with_program(mut self, program: Option<PathBuf>) -> Self {
        if program.is_some() {
            self.program = program;
        }
        self
    }

    pub(crate) fn with_env(mut self, key: &str, value: impl Into<OsString>) -> Self {
        self.envs.push((OsString::from(key), value.into()));
        self
    }

    /// The broker's launch environment must not carry `key` either: a
    /// same-user process can read another process's launch environment.
    pub(crate) fn without_env(mut self, key: &str) -> Self {
        self.removed_envs.push(OsString::from(key));
        self
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
            removed_envs: Vec::new(),
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
    #[error("credential broker request carries no checked DNS answers")]
    Unpinned,
}

type LineReceiver = mpsc::Receiver<std::io::Result<Zeroizing<Vec<u8>>>>;

/// The controller's end of the control channel: a Unix socket, or on
/// Windows the broker's control pipe (PF-27-S06).
#[cfg(unix)]
type ControlStream = UnixStream;
#[cfg(windows)]
type ControlStream = super::pipe::ControlPipe;

struct ControlChannel {
    writer: Option<ControlStream>,
    lines: LineReceiver,
}

pub(crate) struct IsolatedBrokerClient {
    control: Mutex<ControlChannel>,
    child: Mutex<Option<Child>>,
    mac: BrokerChannelMac,
    controller_instance: String,
    broker_instance: String,
    socket_path: PathBuf,
    /// The broker's process id; on Windows each data connection checks it.
    #[cfg_attr(unix, allow(dead_code))]
    broker_pid: u32,
    #[cfg_attr(not(test), allow(dead_code))]
    containment: String,
    run_generation: AtomicU64,
    next_sequence: AtomicU64,
    alive: AtomicBool,
    pin_connections: bool,
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
        let runtime_dir = prepare_runtime_dir(options.runtime_dir.as_deref());
        // Raw values reach the broker only through the private control socket.
        let mut removed: Vec<OsString> = providers::credential_broker_env_keys()
            .map(OsString::from)
            .collect();
        removed.extend(launcher.removed_envs.iter().cloned());
        let mut envs = launcher.envs.clone();
        match runtime_dir.as_ref() {
            Some(dir) => envs.insert(
                0,
                (BROKER_RUNTIME_DIR_ENV.into(), dir.as_os_str().to_owned()),
            ),
            None => removed.push(BROKER_RUNTIME_DIR_ENV.into()),
        }
        let (child, stdout) = start_broker_process(&program, &launcher.args, &removed, &envs)?;
        let broker_pid = child.id();
        let Ok(bootstrap_lines) = spawn_line_reader("credential-broker-bootstrap", stdout) else {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        };
        let control_stream =
            receive_json::<BrokerBootstrap>(&bootstrap_lines, launcher.skip_harness_preamble)
                .ok()
                .and_then(|bootstrap| {
                    connect_control(&bootstrap, runtime_dir.as_deref(), child.id())
                });
        let Some(control_stream) = control_stream else {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        };
        let lines = control_stream
            .try_clone()
            .map_err(|_| IsolatedBrokerError::Spawn)
            .and_then(|reader| {
                spawn_line_reader("credential-broker-control", reader)
                    .map_err(|_| IsolatedBrokerError::Spawn)
            });
        let Ok(lines) = lines else {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        };

        let mut key = Zeroizing::new([0_u8; 32]);
        rand::rng().fill_bytes(key.as_mut());
        let controller_instance = format!("controller-{}", random_hex::<8>());
        let mut control = ControlChannel {
            writer: Some(control_stream),
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
            scrub_responses: options.scrub_responses,
            pin_connections: options.pin_connections,
        };
        let ready = control.send(&hello).and_then(|()| control.receive());
        drop(hello);
        let Ok(ControlResponse::Ready {
            protocol_version,
            broker_instance,
            socket_path,
            run_generation,
            containment,
        }) = ready
        else {
            kill_and_reap(child);
            return Err(IsolatedBrokerError::Spawn);
        };
        if options.require_containment && !containment_sufficient(&containment) {
            tracing::warn!(
                %containment,
                "isolated credential broker refused: it could not confine itself on this platform"
            );
            kill_and_reap_with_cleanup(child, Some(PathBuf::from(socket_path)));
            return Err(IsolatedBrokerError::Spawn);
        }
        let socket_path = PathBuf::from(socket_path);
        tracing::info!(
            %containment,
            broker_dir = %socket_path.parent().unwrap_or(&socket_path).display(),
            "isolated credential broker started"
        );
        if protocol_version != CONTROL_PROTOCOL_VERSION
            || !valid_id(&broker_instance)
            || !valid_data_endpoint(&socket_path)
            || run_generation != 1
        {
            kill_and_reap_with_cleanup(child, Some(socket_path));
            return Err(IsolatedBrokerError::Spawn);
        }
        Ok(Self {
            control: Mutex::new(control),
            child: Mutex::new(Some(child)),
            mac: BrokerChannelMac::from_secret(*key),
            controller_instance,
            broker_instance,
            socket_path,
            broker_pid,
            containment,
            run_generation: AtomicU64::new(run_generation),
            next_sequence: AtomicU64::new(1),
            alive: AtomicBool::new(true),
            pin_connections: options.pin_connections,
        })
    }

    #[cfg(test)]
    pub(crate) fn broker_instance(&self) -> &str {
        &self.broker_instance
    }

    pub(crate) fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }

    /// PF-27-S09: HTTP over the broker's data pipe, every connection checked
    /// to be served by the broker process (PF-27-S06).
    #[cfg(windows)]
    pub(crate) fn data_pipe_client(&self) -> UpstreamClient {
        UpstreamClient::named_pipe(&self.socket_path.to_string_lossy(), self.broker_pid)
    }

    /// OS containment the broker reported for itself (PF-27-S02).
    #[cfg(test)]
    pub(crate) fn containment(&self) -> &str {
        &self.containment
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
        self.registered(response)
    }

    /// PF-27-S05: hands one of Core's model-provider keys to the broker.
    pub(crate) fn register_model(
        &self,
        binding: ModelBindingWire,
        value: &str,
    ) -> Result<CredentialReference, IsolatedBrokerError> {
        let response = self.call(&ControlRequest::RegisterModel {
            binding,
            value: value.to_string(),
        })?;
        self.registered(response)
    }

    /// PF-27-S05: hands one environment variable's value to the broker.
    pub(crate) fn stash_env(&self, name: &str, value: &str) -> Result<(), IsolatedBrokerError> {
        match self.call(&ControlRequest::StashEnv {
            name: name.to_string(),
            value: value.to_string(),
        })? {
            ControlResponse::Stashed => Ok(()),
            ControlResponse::Error { .. } => Err(IsolatedBrokerError::Rejected),
            _ => Err(self.fail(IsolatedBrokerError::Control)),
        }
    }

    /// PF-27-S05: registers a model key the broker reads itself.
    pub(crate) fn register_model_stored(
        &self,
        binding: ModelBindingWire,
        provider_key_id: &str,
        env_names: &[String],
        account: Option<&StoredKeyAccount>,
    ) -> Result<StoredRegistration, IsolatedBrokerError> {
        let response = self.call(&ControlRequest::RegisterModelStored {
            binding,
            provider_key_id: provider_key_id.to_string(),
            env_names: env_names.to_vec(),
            account: account.cloned(),
        })?;
        match response {
            ControlResponse::Error {
                code: ControlErrorCode::NotFound,
            } => Ok(StoredRegistration::NotFound),
            ControlResponse::Error {
                code: ControlErrorCode::Unavailable,
            } => Ok(StoredRegistration::StoreUnavailable),
            response => self
                .registered(response)
                .map(StoredRegistration::Registered),
        }
    }

    /// PF-27-S05: drops one reference.
    pub(crate) fn unregister(
        &self,
        reference: &CredentialReference,
    ) -> Result<(), IsolatedBrokerError> {
        match self.call(&ControlRequest::Unregister {
            reference: reference.as_str().to_string(),
        })? {
            ControlResponse::Unregistered => Ok(()),
            ControlResponse::Error { .. } => Err(IsolatedBrokerError::Rejected),
            _ => Err(self.fail(IsolatedBrokerError::Control)),
        }
    }

    fn registered(
        &self,
        response: ControlResponse,
    ) -> Result<CredentialReference, IsolatedBrokerError> {
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
        // PF-33-S02: the checked answers travel inside the signed frame, so
        // the broker connects to the peer Core authorized.
        let operation = match request.extensions().get::<PinnedPeers>() {
            Some(pin) if pin.covers_authority(operation.host(), operation.port()) => {
                tracing::info!(
                    "brokered request pinned (host={}, port={}, answers={})",
                    operation.host(),
                    operation.port(),
                    pin.addrs().len()
                );
                // The guard keeps at most 16 answers; a subset is still checked.
                operation
                    .clone()
                    .with_pinned_addrs(
                        pin.addrs()
                            .iter()
                            .copied()
                            .take(codex_secret_broker::ipc::MAX_PINNED_ADDRS),
                    )
                    .map_err(|_| IsolatedBrokerError::Unpinned)?
            }
            Some(_) => return Err(IsolatedBrokerError::Unpinned),
            None if self.pin_connections => return Err(IsolatedBrokerError::Unpinned),
            None => operation.clone(),
        };
        let frame = self.sign_frame(credential, &operation)?;
        self.send_with_frame(&operation, request, &frame).await
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
        #[cfg(unix)]
        let upstream = UpstreamClient::unix_socket(&self.socket_path.to_string_lossy());
        #[cfg(windows)]
        let upstream = self.data_pipe_client();
        upstream
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

    /// PF-27-S09: sends an already signed request over the data pipe.
    #[cfg(windows)]
    pub(crate) async fn send_signed(
        &self,
        request: Request,
    ) -> Result<Response, IsolatedBrokerError> {
        if !self.is_alive() {
            return Err(IsolatedBrokerError::Unavailable);
        }
        self.data_pipe_client().serve(request).await.map_err(|_| {
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
            Ok(mut control) => control.send(request).and_then(|()| control.receive()),
            Err(_) => Err(IsolatedBrokerError::Control),
        };
        result.map_err(|error| self.fail(error))
    }

    fn fail(&self, error: IsolatedBrokerError) -> IsolatedBrokerError {
        self.alive.store(false, Ordering::Release);
        if let Ok(mut control) = self.control.lock() {
            control.close();
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
        // Control EOF is the broker's shutdown signal; kill as a backstop.
        if let Ok(control) = self.control.get_mut() {
            control.close();
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
        let writer = self.writer.as_mut().ok_or(IsolatedBrokerError::Control)?;
        // PF-27-S05: the line may carry a raw value. Reserve it once so a
        // growing buffer never leaves an unzeroed copy behind; a request that
        // would outgrow it is refused.
        let mut line = Zeroizing::new(Vec::with_capacity(MAX_CONTROL_LINE_BYTES + 1));
        let mut bounded = BoundedWriter(&mut line);
        serde_json::to_writer(&mut bounded, request).map_err(|_| IsolatedBrokerError::Rejected)?;
        if line.len() > MAX_CONTROL_LINE_BYTES {
            return Err(IsolatedBrokerError::Rejected);
        }
        line.push(b'\n');
        writer
            .write_all(&line)
            .and_then(|()| writer.flush())
            .map_err(|_| IsolatedBrokerError::Control)
    }

    fn receive(&mut self) -> Result<ControlResponse, IsolatedBrokerError> {
        receive_json(&self.lines, /*skip_harness_preamble*/ false)
    }

    /// Shuts the socket down for every holder, so the broker sees EOF even if
    /// another process inherited a duplicate of this descriptor. On Windows
    /// the pipe handle is never inheritable: cancelling the reader's pending
    /// read lets the last handle close, which the broker sees as EOF.
    fn close(&mut self) {
        if let Some(writer) = self.writer.take() {
            #[cfg(unix)]
            let _ = writer.shutdown(std::net::Shutdown::Both);
            #[cfg(windows)]
            writer.shutdown();
        }
    }
}

/// PF-27-S05: outcome of a stored-key registration.
pub(crate) enum StoredRegistration {
    Registered(CredentialReference),
    /// No stashed variable and no stored key.
    NotFound,
    /// The store (vault or OS keyring) could not be read.
    StoreUnavailable,
}

/// Appends to a vector without letting it grow past its reserved capacity.
struct BoundedWriter<'a>(&'a mut Vec<u8>);

impl std::io::Write for BoundedWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        // Keep one byte for the newline.
        if self.0.len() + bytes.len() >= self.0.capacity() {
            return Err(std::io::ErrorKind::OutOfMemory.into());
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn receive_json<T: serde::de::DeserializeOwned>(
    lines: &LineReceiver,
    skip_harness_preamble: bool,
) -> Result<T, IsolatedBrokerError> {
    let mut skipped = 0;
    loop {
        let line = lines
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

fn spawn_line_reader<R: std::io::Read + Send + 'static>(
    name: &str,
    source: R,
) -> std::io::Result<LineReceiver> {
    let (sender, lines) = mpsc::channel();
    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            let mut reader = BufReader::new(source);
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
        })?;
    Ok(lines)
}

/// Seatbelt on macOS; seccomp (Landlock when the kernel has it) on Linux;
/// on Windows the broker token (PF-27-S08), the process DACL and the
/// no-child-process job.
fn containment_sufficient(containment: &str) -> bool {
    let has = |wanted: &str| containment.split('+').any(|mechanism| mechanism == wanted);
    if cfg!(target_os = "macos") {
        containment == "seatbelt"
    } else if cfg!(target_os = "linux") {
        has("seccomp")
    } else if cfg!(windows) {
        has("token") && has("dacl") && has("job")
    } else {
        false
    }
}

/// The broker's data endpoint: an absolute socket path, or on Windows a
/// broker data pipe name.
fn valid_data_endpoint(path: &Path) -> bool {
    #[cfg(windows)]
    {
        path.to_str()
            .is_some_and(|name| super::pipe::valid_pipe_name(name, /*control*/ false))
    }
    #[cfg(not(windows))]
    {
        path.is_absolute()
    }
}

/// Picks the broker's runtime parent directory: the configured one
/// (`CODEX_HOME/run`) when its socket paths fit, otherwise a per-user runtime
/// directory outside the sandbox's writable roots (the Darwin user cache
/// directory, or `$XDG_RUNTIME_DIR` on Linux). `None` falls back to the
/// broker's temporary directory, where an agent could delete the socket.
#[cfg(unix)]
fn prepare_runtime_dir(configured: Option<&Path>) -> Option<PathBuf> {
    let configured = configured?;
    create_runtime_dir(configured)
        .or_else(|| user_runtime_dir().and_then(|dir| create_runtime_dir(&dir)))
}

/// PF-27-S06: named pipes live outside the file system; no directory.
#[cfg(windows)]
fn prepare_runtime_dir(_configured: Option<&Path>) -> Option<PathBuf> {
    None
}

/// Creates `dir` (owner-only) when `<dir>/cbk-XXXXXX/c.sock` fits in a Unix
/// socket address.
#[cfg(unix)]
fn create_runtime_dir(dir: &Path) -> Option<PathBuf> {
    const MAX_RUNTIME_DIR_BYTES: usize = 100 - "/cbk-XXXXXX/c.sock".len();
    if !dir.is_absolute() || dir.as_os_str().len() > MAX_RUNTIME_DIR_BYTES {
        return None;
    }
    std::fs::create_dir_all(dir).ok()?;
    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
    std::fs::symlink_metadata(dir)
        .ok()
        .filter(std::fs::Metadata::is_dir)
        .map(|_| dir.to_path_buf())
}

pub(crate) fn user_runtime_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        None
    }
    #[cfg(target_os = "macos")]
    {
        let mut buffer = [0 as libc::c_char; 1024];
        // SAFETY: the buffer and its length are valid; confstr NUL-terminates.
        let len = unsafe {
            libc::confstr(
                libc::_CS_DARWIN_USER_CACHE_DIR,
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        if len == 0 || len > buffer.len() {
            return None;
        }
        // SAFETY: confstr wrote a NUL-terminated string into `buffer`.
        let dir = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) };
        let dir = PathBuf::from(dir.to_str().ok()?);
        Some(dir.join("corbanu-run"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|dir| dir.is_absolute())
            .map(|dir| dir.join("corbanu-run"))
    }
}

/// PF-27-S06: opens the control pipe named in the bootstrap line, if it is a
/// broker control pipe served by the broker process this controller spawned.
#[cfg(windows)]
fn connect_control(
    bootstrap: &BrokerBootstrap,
    _runtime_dir: Option<&Path>,
    broker_pid: u32,
) -> Option<ControlStream> {
    if bootstrap.protocol_version != CONTROL_PROTOCOL_VERSION {
        return None;
    }
    super::pipe::connect_control(&bootstrap.control_socket, broker_pid)
}

/// Connects to the control socket named in the bootstrap line, but only if
/// it lives in a broker directory under the expected runtime directory and
/// its OS peer is the broker process this controller spawned.
#[cfg(unix)]
fn connect_control(
    bootstrap: &BrokerBootstrap,
    runtime_dir: Option<&Path>,
    broker_pid: u32,
) -> Option<UnixStream> {
    if bootstrap.protocol_version != CONTROL_PROTOCOL_VERSION {
        return None;
    }
    let control_path = PathBuf::from(&bootstrap.control_socket);
    let broker_dir = control_path.parent()?;
    let in_broker_dir = control_path.is_absolute()
        && control_path.file_name()? == "c.sock"
        && broker_dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cbk-"));
    let under_runtime_dir =
        runtime_dir.is_none_or(|runtime_dir| broker_dir.parent() == Some(runtime_dir));
    if !in_broker_dir || !under_runtime_dir {
        return None;
    }
    let stream = UnixStream::connect(&control_path).ok()?;
    (peer_pid(&stream) == Some(broker_pid)).then_some(stream)
}

/// The process id of a connected Unix socket's peer.
#[cfg(unix)]
fn peer_pid(stream: &UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd as _;
    let fd = stream.as_raw_fd();
    #[cfg(target_os = "linux")]
    {
        let mut credentials = libc::ucred {
            pid: 0,
            uid: 0,
            gid: 0,
        };
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        // SAFETY: `credentials` and `len` are valid for the size passed.
        let result = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                (&mut credentials as *mut libc::ucred).cast(),
                &mut len,
            )
        };
        (result == 0)
            .then(|| u32::try_from(credentials.pid).ok())
            .flatten()
    }
    #[cfg(target_os = "macos")]
    {
        let mut pid: libc::pid_t = 0;
        let mut len = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
        // SAFETY: `pid` and `len` are valid for the size passed.
        let result = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_LOCAL,
                libc::LOCAL_PEERPID,
                (&mut pid as *mut libc::pid_t).cast(),
                &mut len,
            )
        };
        (result == 0).then(|| u32::try_from(pid).ok()).flatten()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = fd;
        None
    }
}

fn read_control_line<R: std::io::Read>(
    reader: &mut BufReader<R>,
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

/// Starts the broker with `args`, without the `removed` variables and with
/// `envs` set (later entries win). Returns it and its stdout.
#[cfg(unix)]
fn start_broker_process(
    program: &Path,
    args: &[OsString],
    removed: &[OsString],
    envs: &[(OsString, OsString)],
) -> Result<(Child, std::process::ChildStdout), IsolatedBrokerError> {
    let mut command = Command::new(program);
    // PF-27-S02: no data ever crosses a descriptor created here. stdout
    // carries only the broker's control socket path; the control channel
    // is a socket the broker creates, so a process that inherits one of
    // these pipes during the macOS close-on-exec window learns nothing.
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    // Own process group: terminal hangups and interrupts aimed at the TUI
    // must not kill the broker before it removes its socket directory.
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    for key in removed {
        command.env_remove(key);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    let mut child = command.spawn().map_err(|_| IsolatedBrokerError::Spawn)?;
    match child.stdout.take() {
        Some(stdout) => Ok((child, stdout)),
        None => {
            kill_and_reap(child);
            Err(IsolatedBrokerError::Spawn)
        }
    }
}

/// PF-27-S07: no other process of the user can open the broker at any point
/// confines its writes and keeps it out of the user's other processes. No
/// confines its writes and keeps it out of the user's other processes. No
/// console, so console control events aimed at the TUI do not reach it
/// either.
#[cfg(windows)]
fn start_broker_process(
    program: &Path,
    args: &[OsString],
    removed: &[OsString],
    envs: &[(OsString, OsString)],
) -> Result<(Child, std::fs::File), IsolatedBrokerError> {
    let same = |a: &OsString, b: &OsString| a.eq_ignore_ascii_case(b);
    let mut env = inherited_env(|name| {
        !removed.iter().any(|key| same(key, name)) && !envs.iter().any(|(key, _)| same(key, name))
    });
    for (key, value) in envs {
        env.retain(|(name, _)| !same(name, key));
        env.push((key.clone(), value.clone()));
    }
    codex_process_hardening::spawn_protected(program, args, &env)
        .map_err(|_| IsolatedBrokerError::Spawn)
}

/// This process's environment variables whose names pass `keep`. Unlike
/// `std::env::vars_os`, the value of a variable that is not kept (a provider
/// key handed to the broker, PF-27-S09) is never copied: the snapshot is
/// read in place and wiped before it is freed.
#[cfg(windows)]
fn inherited_env(keep: impl Fn(&OsString) -> bool) -> Vec<(OsString, OsString)> {
    use std::os::windows::ffi::OsStringExt as _;
    use windows_sys::Win32::System::Environment::FreeEnvironmentStringsW;
    use windows_sys::Win32::System::Environment::GetEnvironmentStringsW;
    let mut env = Vec::new();
    // SAFETY: returns a private copy of the environment block (or null),
    // freed below.
    let block = unsafe { GetEnvironmentStringsW() };
    if block.is_null() {
        return env;
    }
    let mut offset = 0;
    loop {
        // SAFETY: the block is a sequence of NUL-terminated strings ended by
        // an empty one; `offset` stays on an entry start.
        let entry = unsafe {
            let start = block.add(offset);
            let mut len = 0;
            while *start.add(len) != 0 {
                len += 1;
            }
            std::slice::from_raw_parts_mut(start, len)
        };
        if entry.is_empty() {
            break;
        }
        offset += entry.len() + 1;
        // Hidden per-drive variables (`=C:=C:\...`) start with `=`.
        let Some(split) = entry
            .iter()
            .skip(1)
            .position(|unit| *unit == u16::from(b'='))
            .map(|position| position + 1)
        else {
            entry.fill(0);
            continue;
        };
        let name = OsString::from_wide(&entry[..split]);
        if keep(&name) {
            env.push((name, OsString::from_wide(&entry[split + 1..])));
        }
        // The private copy is wiped entry by entry before it is freed.
        for unit in entry.iter_mut() {
            // SAFETY: a valid element of the block.
            unsafe { std::ptr::write_volatile(unit, 0) };
        }
    }
    // SAFETY: from GetEnvironmentStringsW above.
    unsafe { FreeEnvironmentStringsW(block) };
    env
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

#[cfg(test)]
#[cfg(windows)]
mod pf_27_s08_tests {
    use super::containment_sufficient;

    /// Core refuses a Windows broker that reports no broker token, even with
    /// its other two layers.
    #[test]
    fn pf_27_s08_core_refuses_a_broker_without_its_token() {
        assert!(containment_sufficient("token+dacl+job"));
        assert!(!containment_sufficient("dacl+job"));
        assert!(!containment_sufficient("token+job"));
        assert!(!containment_sufficient("token+dacl"));
        assert!(!containment_sufficient("none"));
    }
}

#[cfg(test)]
#[cfg(unix)]
mod pf_27_s02_tests {
    use super::*;

    #[test]
    fn pf_27_s02_containment_requirement_is_platform_specific() {
        assert!(!containment_sufficient("none"));
        if cfg!(target_os = "macos") {
            assert!(containment_sufficient("seatbelt"));
            assert!(!containment_sufficient("seccomp"));
        } else if cfg!(target_os = "linux") {
            assert!(containment_sufficient("seccomp"));
            assert!(containment_sufficient("landlock+seccomp"));
            assert!(!containment_sufficient("landlock"));
        }
    }

    #[test]
    fn pf_27_s02_long_runtime_dirs_fall_back_to_the_user_runtime_dir() {
        let short = tempfile::Builder::new()
            .prefix("pf27s02-")
            .tempdir_in("/tmp")
            .expect("short dir");
        let configured = short.path().join("run");
        assert_eq!(
            prepare_runtime_dir(Some(&configured)),
            Some(configured.clone())
        );
        assert_eq!(prepare_runtime_dir(None), None);

        let long = PathBuf::from("/").join("x".repeat(120)).join("run");
        let fallback = user_runtime_dir().and_then(|dir| create_runtime_dir(&dir));
        assert_eq!(prepare_runtime_dir(Some(&long)), fallback);
    }
}
