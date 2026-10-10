//! The broker process: the only place raw brokered credential values live.
//!
//! It accepts typed provider requests on a private Unix socket (a named pipe
//! on Windows, PF-27-S06) from exactly the controller process that spawned
//! it, substitutes the provider header for the bound host, and performs the
//! upstream HTTPS request itself.

use super::protocol::BROKER_ERROR_HEADER;
use super::protocol::BROKER_RUNTIME_DIR_ENV;
use super::protocol::BROKER_SESSION_ID;
use super::protocol::BROKER_STORE_HOME_ENV;
use super::protocol::BROKER_TASK_ID;
use super::protocol::BrokerBootstrap;
use super::protocol::CONTROL_PROTOCOL_VERSION;
use super::protocol::ControlErrorCode;
use super::protocol::ControlRequest;
use super::protocol::ControlResponse;
use super::protocol::FRAME_HEADER;
use super::protocol::HostBindingWire;
use super::protocol::MAX_CONTROL_LINE_BYTES;
use super::protocol::MAX_CREDENTIAL_VALUE_BYTES;
use super::protocol::MAX_STASHED_ENV;
use super::protocol::ModelAuthHeader;
use super::protocol::ModelBindingWire;
use super::protocol::ProviderId;
use super::protocol::StoredKeyAccount;
use super::protocol::decode_key;
use super::protocol::encode_hex;
use super::protocol::valid_env_name;
use super::protocol::valid_id;
use crate::config::NetworkProxyConfig;
use crate::connect_policy::PinnedPeers;
use crate::credential_broker::providers;
use crate::credential_broker::response_scrub;
use crate::runtime::NetworkProxyState;
use crate::runtime::StaticConfigReloader;
use crate::state::NetworkProxyConstraints;
use crate::state::build_config_state;
use crate::upstream::UpstreamClient;
use anyhow::Context as _;
use base64::Engine as _;
use codex_secret_broker::BrokerBinding;
use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::CredentialReference;
use codex_secret_broker::SignedBrokerFrame;
use codex_secret_broker::VerifiedProviderRequest;
use codex_secret_broker::response_gate::ResponseGate;
use rama_core::Layer as _;
use rama_core::Service as _;
use rama_core::bytes::Bytes;
use rama_core::error::BoxError;
use rama_core::rt::Executor;
use rama_core::service::service_fn;
use rama_http::Body;
use rama_http::HeaderValue;
use rama_http::Request;
use rama_http::Response;
use rama_http::StatusCode;
use rama_http::StreamingBody;
use rama_http::Version;
use rama_http::header::HOST;
use rama_http::layer::remove_header::RemoveRequestHeaderLayer;
use rama_http::layer::remove_header::RemoveResponseHeaderLayer;
use rama_http_backend::server::HttpServer;
use rand::RngCore as _;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncBufReadExt as _;
use tokio::io::AsyncReadExt as _;
use tokio::io::AsyncWriteExt as _;
use tokio::io::BufReader;
use tokio::sync::watch;
use zeroize::Zeroizing;

pub(crate) const MAX_BROKER_CREDENTIALS: usize = 64;
pub(crate) const MAX_BROKER_IN_FLIGHT: usize = 64;
const REPLAY_WINDOW: u64 = 1_024;
const ACCEPT_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(50);
/// Longest wait for upstream response headers or between body frames.
const RESPONSE_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);
const MAX_FRAME_HEADER_BYTES: usize = 22 * 1024;
const BROKER_EXIT_UNAVAILABLE: i32 = 78;
const CONTROL_ACCEPT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
#[cfg(unix)]
const PARENT_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

/// PF-27-S05: reads the provider key stored for `provider_key_id` under a
/// Corbanu home (encrypted vault, then the legacy file) inside the broker, so
/// Core never decrypts it. Supplied by the binary, which links the vault.
/// PF-84: with an account, only that named account's key is read (never the
/// default key or an environment variable).
pub type StoredKeyResolver =
    fn(&std::path::Path, &str, Option<&StoredKeyAccount>) -> std::io::Result<Option<String>>;

/// Entry point for `corbanu --codex-run-as-credential-broker`.
pub fn run_credential_broker_main() -> ! {
    run_credential_broker_main_with(/*resolver*/ None)
}

/// Like [`run_credential_broker_main`], resolving stored provider keys with
/// `resolver` (PF-27-S05).
pub fn run_credential_broker_main_with(resolver: Option<StoredKeyResolver>) -> ! {
    // Raw values and the channel key live here: refuse debugger attach and
    // core dumps, and drop loader-injection variables, before anything else.
    codex_process_hardening::pre_main_hardening();
    // PF-27-S02: confine the broker while it is still single-threaded, so
    // every runtime thread inherits it: no program execution or new
    // processes, no cross-process inspection, writes only under the runtime
    // directory (and the vault's lock file, which a vault read must lock).
    let runtime_dir = runtime_dir();
    #[cfg(unix)]
    let (containment, stored_keys) = {
        let store_home = resolver.and(store_home());
        // Landlock rules need the file to exist (the vault may be created later
        // in the session), so the lock is created here, before containment. Only
        // a regular file reached without symlinks is made writable.
        let vault_lock: Vec<std::path::PathBuf> = store_home
            .iter()
            .filter_map(|home| prepare_vault_lock(home))
            .collect();
        let containment = codex_process_hardening::contain_credential_broker_with_files(
            &runtime_dir,
            &vault_lock,
        );
        (containment, resolver.zip(store_home))
    };
    // PF-27-S06/S08: checks the broker token Core started it under, then
    // applies the process DACL and a no-child-process job. PF-27-S09: stored
    // provider keys are read here too, the vault key from Credential Manager
    // under the broker token (Travis's key-path decision (c)). The token
    // cannot create the vault lock; Core creates it before the broker starts
    // (`ModelCredentialBroker::spawn`), and a vault read locks it through a
    // read-only handle.
    #[cfg(windows)]
    let (containment, stored_keys) = (
        codex_process_hardening::contain_credential_broker(&runtime_dir),
        resolver.zip(store_home()),
    );
    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    else {
        std::process::exit(BROKER_EXIT_UNAVAILABLE);
    };
    let exit_code =
        match runtime.block_on(run_broker(runtime_dir, containment.mechanism, stored_keys)) {
            Ok(()) => 0,
            Err(_) => BROKER_EXIT_UNAVAILABLE,
        };
    // Exit without dropping the runtime: a blocking reader may still be
    // parked on a channel the controller holds open (signal-initiated shutdown).
    std::process::exit(exit_code)
}

/// Where the private socket directory is created: the controller's runtime
/// directory (`CODEX_HOME/run`), or the temporary directory as a fallback.
fn runtime_dir() -> std::path::PathBuf {
    std::env::var_os(BROKER_RUNTIME_DIR_ENV)
        .map(std::path::PathBuf::from)
        .filter(|dir| dir.is_absolute() && dir.is_dir())
        .unwrap_or_else(socket_parent)
}

/// Unix socket paths are limited to roughly 100 bytes; fall back to `/tmp`
/// when the per-user temporary directory is too deep. Windows pipes need no
/// directory.
fn socket_parent() -> std::path::PathBuf {
    const MAX_SOCKET_PARENT_BYTES: usize = 80;
    let temp_dir = std::env::temp_dir();
    if cfg!(windows) || temp_dir.as_os_str().len() <= MAX_SOCKET_PARENT_BYTES {
        temp_dir
    } else {
        std::path::PathBuf::from("/tmp")
    }
}

/// Creates `<home>/secrets/.vault.lock` if needed (never through a symlink)
/// and returns its path when it is a regular file in a real directory.
#[cfg(unix)]
pub(crate) fn prepare_vault_lock(home: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let dir = home.join("secrets");
    if !std::fs::symlink_metadata(&dir).is_ok_and(|meta| meta.is_dir()) {
        use std::os::unix::fs::DirBuilderExt as _;
        std::fs::DirBuilder::new().mode(0o700).create(&dir).ok()?;
    }
    if !std::fs::symlink_metadata(&dir).is_ok_and(|meta| meta.is_dir()) {
        return None;
    }
    let lock = dir.join(".vault.lock");
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&lock)
        .ok()?;
    std::fs::symlink_metadata(&lock)
        .is_ok_and(|meta| meta.is_file())
        .then_some(lock)
}

/// PF-27-S09: Windows counterpart of the Unix `prepare_vault_lock`, run by
/// Core before it starts the broker, whose token cannot create files in the
/// user's profile. A `secrets` directory or lock file that is a reparse point
/// (symlink or junction) is refused, and the opened file must be the lock in
/// `home`'s own `secrets` directory (a directory swapped meanwhile is caught).
#[cfg(windows)]
pub(crate) fn prepare_vault_lock(home: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::os::windows::fs::OpenOptionsExt as _;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;
    let home = std::fs::canonicalize(home).ok()?;
    let dir = home.join("secrets");
    match std::fs::symlink_metadata(&dir) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => return None,
        Err(_) => std::fs::create_dir(&dir).ok()?,
    }
    let lock = dir.join(".vault.lock");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&lock)
        .ok()?;
    let opened_is_lock = file.metadata().is_ok_and(|meta| meta.is_file())
        && final_path(&file).is_some_and(|path| path.eq_ignore_ascii_case(lock.as_os_str()));
    opened_is_lock.then_some(lock)
}

/// The path Windows reports for an open file (`\\?\` form, like
/// `std::fs::canonicalize`).
#[cfg(windows)]
fn final_path(file: &std::fs::File) -> Option<std::ffi::OsString> {
    use std::os::windows::ffi::OsStringExt as _;
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::Storage::FileSystem::GetFinalPathNameByHandleW;
    let mut buffer = vec![0_u16; 1024];
    loop {
        // SAFETY: `buffer` holds `buffer.len()` units; flags 0: normalized,
        // with a drive letter.
        let len = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle() as _,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                0,
            )
        } as usize;
        if len == 0 {
            return None;
        }
        if len < buffer.len() {
            return Some(std::ffi::OsString::from_wide(&buffer[..len]));
        }
        buffer.resize(len + 1, 0);
    }
}

/// The Corbanu home named by the controller, if it is an absolute directory.
fn store_home() -> Option<std::path::PathBuf> {
    std::env::var_os(BROKER_STORE_HOME_ENV)
        .map(std::path::PathBuf::from)
        .filter(|dir| dir.is_absolute() && dir.is_dir())
}

async fn run_broker(
    runtime_dir: std::path::PathBuf,
    containment: String,
    stored_keys: Option<(StoredKeyResolver, std::path::PathBuf)>,
) -> anyhow::Result<()> {
    // The control channel accepts exactly the process that spawned us. A
    // stale parent (reparented to init or launchd) means the controller died.
    let parent = transport::Parent::open().context("find the controller process")?;
    let parent_pid = parent.pid;
    // PF-27-S02: stdout carries only the (non-secret) control socket path.
    // Secrets never cross a descriptor the controller created, so a process
    // that inherits one during the macOS close-on-exec window gets nothing.
    let mut stdout = tokio::io::stdout();
    let mut endpoint = match transport::Endpoint::bind(&runtime_dir).await {
        Ok(endpoint) => endpoint,
        Err(error) => {
            // #390: a name held by another process is reported, never joined.
            if name_taken(&error) {
                write_json_line(
                    &mut stdout,
                    &BrokerBootstrap {
                        protocol_version: CONTROL_PROTOCOL_VERSION,
                        control_socket: String::new(),
                        pipe_taken: true,
                    },
                )
                .await?;
            }
            return Err(error);
        }
    };
    write_json_line(
        &mut stdout,
        &BrokerBootstrap {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            control_socket: endpoint.control_name(),
            pipe_taken: false,
        },
    )
    .await?;
    let control_stream = tokio::time::timeout(
        CONTROL_ACCEPT_TIMEOUT,
        endpoint.accept_controller(parent_pid),
    )
    .await
    .context("controller did not connect")??;
    let (control_read, mut control_write) = tokio::io::split(control_stream);
    let mut control = BufReader::new(control_read);

    let hello = read_control_line(&mut control)
        .await?
        .context("controller closed before hello")?;
    let hello: ControlRequest =
        serde_json::from_slice(&hello).context("malformed controller hello")?;
    let ControlRequest::Hello {
        protocol_version,
        controller_pid,
        controller_instance,
        channel_key,
        allow_local_binding,
        allow_upstream_proxy,
        scrub_responses,
        pin_connections,
    } = &hello
    else {
        anyhow::bail!("controller did not start with hello");
    };
    if *protocol_version != CONTROL_PROTOCOL_VERSION {
        write_json_line(
            &mut control_write,
            &ControlResponse::Error {
                code: ControlErrorCode::UnsupportedProtocol,
            },
        )
        .await?;
        anyhow::bail!("unsupported controller protocol");
    }
    let key = Zeroizing::new(decode_key(channel_key).context("invalid channel key")?);
    anyhow::ensure!(
        *controller_pid != 0 && valid_id(controller_instance),
        "invalid controller identity"
    );

    let broker_instance = format!("broker-{}", random_hex::<8>());
    let (generation, _) = watch::channel(1_u64);
    let broker = Arc::new(Broker {
        mac: BrokerChannelMac::from_secret(*key),
        controller_pid: *controller_pid,
        controller_instance: controller_instance.clone(),
        broker_instance: broker_instance.clone(),
        state: Mutex::new(BrokerState::default()),
        generation,
        upstream: upstream_client(*allow_local_binding, *allow_upstream_proxy)?,
        scrub_responses: *scrub_responses,
        pin_connections: *pin_connections,
        stored_keys,
    });
    drop(hello);

    write_json_line(
        &mut control_write,
        &ControlResponse::Ready {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            broker_instance,
            socket_path: endpoint.data_name(),
            run_generation: 1,
            containment,
        },
    )
    .await?;

    let accept = endpoint.serve(broker.clone());
    let parent_gone = transport::shutdown_requested(parent)?;
    tokio::pin!(parent_gone);
    loop {
        let line = tokio::select! {
            line = read_control_line(&mut control) => line?,
            () = &mut parent_gone => None,
        };
        let Some(line) = line else {
            break;
        };
        let response = match serde_json::from_slice::<ControlRequest>(&line) {
            // A stored-key read may block on the vault and the OS keyring.
            Ok(request) => tokio::task::block_in_place(|| broker.control(request)),
            Err(_) => ControlResponse::Error {
                code: ControlErrorCode::Malformed,
            },
        };
        write_json_line(&mut control_write, &response).await?;
    }
    // Controller EOF, controller death or a termination signal: close every
    // outstanding channel by revoking, then remove the private socket
    // directory before exiting.
    broker.revoke();
    accept.abort();
    drop(endpoint);
    Ok(())
}

/// True when binding failed because another process holds a pipe name
/// (Windows; Unix sockets live in a fresh private directory).
fn name_taken(error: &anyhow::Error) -> bool {
    cfg!(windows)
        && error
            .downcast_ref::<std::io::Error>()
            .is_some_and(|error| error.kind() == std::io::ErrorKind::AddrInUse)
}

fn upstream_client(
    allow_local_binding: bool,
    allow_upstream_proxy: bool,
) -> anyhow::Result<UpstreamClient> {
    let config = NetworkProxyConfig {
        enabled: true,
        allow_local_binding,
        ..NetworkProxyConfig::default()
    };
    let state = build_config_state(config, NetworkProxyConstraints::default())?;
    let state = Arc::new(NetworkProxyState::with_reloader(
        state,
        Arc::new(StaticConfigReloader),
    ));
    let roots = crate::certs::broker_upstream_root_store(&crate::certs::ca_env_from_process())?;
    Ok(if allow_upstream_proxy {
        UpstreamClient::from_env_proxy_with_tls_root_store(state, roots)
    } else {
        UpstreamClient::direct_with_tls_root_store(state, roots)
    })
}

/// Serves HTTP provider requests on one connection already checked to come
/// from the controller.
async fn serve_connection<S>(stream: S, broker: Arc<Broker>)
where
    S: rama_core::stream::Stream + rama_core::extensions::ExtensionsMut,
{
    let service = HttpServer::auto(Executor::default()).service(
        (
            RemoveResponseHeaderLayer::hop_by_hop(),
            RemoveRequestHeaderLayer::hop_by_hop(),
        )
            .into_layer(service_fn(move |request| {
                let broker = broker.clone();
                async move { Ok::<_, Infallible>(broker.handle(request).await) }
            })),
    );
    let _ = service.serve(stream).await;
}

struct Broker {
    mac: BrokerChannelMac,
    controller_pid: u32,
    controller_instance: String,
    broker_instance: String,
    state: Mutex<BrokerState>,
    generation: watch::Sender<u64>,
    upstream: UpstreamClient,
    /// PF-28-S02: scrub each credential from the responses it authorizes.
    scrub_responses: bool,
    /// PF-33-S02: provider requests must carry checked answers to dial.
    pin_connections: bool,
    /// PF-27-S05: how stored provider keys are read, and from which home.
    stored_keys: Option<(StoredKeyResolver, std::path::PathBuf)>,
}

struct BrokerState {
    run_generation: u64,
    credentials: HashMap<CredentialReference, BrokerCredential>,
    /// PF-27-S05: provider-key environment variables Core handed over.
    env_stash: HashMap<String, Zeroizing<String>>,
    replay: ReplayWindow,
    in_flight: usize,
}

impl Default for BrokerState {
    fn default() -> Self {
        Self {
            run_generation: 1,
            credentials: HashMap::new(),
            env_stash: HashMap::new(),
            replay: ReplayWindow::default(),
            in_flight: 0,
        }
    }
}

struct BrokerCredential {
    kind: CredentialKind,
    value: Zeroizing<String>,
    reflection: Option<ResponseGate>,
}

enum CredentialKind {
    /// An agent credential virtualized by the network proxy (PF-27-S04).
    Provider {
        provider: ProviderId,
        binding: HostBindingWire,
    },
    /// One of Core's own model-provider keys (PF-27-S05).
    Model(ModelBindingWire),
}

/// Where an authorized request's credential header goes.
#[derive(Clone, Copy)]
enum HeaderTarget {
    Provider(ProviderId),
    Model(ModelAuthHeader),
}

#[derive(Default)]
struct ReplayWindow {
    highest: u64,
    seen: BTreeSet<u64>,
}

impl ReplayWindow {
    fn accept(&mut self, sequence: u64) -> bool {
        if sequence.saturating_add(REPLAY_WINDOW) <= self.highest || !self.seen.insert(sequence) {
            return false;
        }
        self.highest = self.highest.max(sequence);
        let floor = self.highest.saturating_sub(REPLAY_WINDOW);
        self.seen = self.seen.split_off(&floor);
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DenyCode {
    MissingFrame,
    MalformedFrame,
    AuthenticationFailed,
    BindingMismatch,
    StaleGeneration,
    Replay,
    UnknownCredential,
    HostNotBound,
    RequestMismatch,
    Capacity,
    Revoked,
    UpstreamFailed,
    Unpinned,
}

impl DenyCode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::MissingFrame => "missing_frame",
            Self::MalformedFrame => "malformed_frame",
            Self::AuthenticationFailed => "authentication_failed",
            Self::BindingMismatch => "binding_mismatch",
            Self::StaleGeneration => "stale_generation",
            Self::Replay => "replay",
            Self::UnknownCredential => "unknown_credential",
            Self::HostNotBound => "host_not_bound",
            Self::RequestMismatch => "request_mismatch",
            Self::Capacity => "capacity",
            Self::Revoked => "revoked",
            Self::UpstreamFailed => "upstream_failed",
            Self::Unpinned => "unpinned",
        }
    }

    fn status(self) -> StatusCode {
        match self {
            Self::Capacity | Self::Revoked => StatusCode::SERVICE_UNAVAILABLE,
            Self::UpstreamFailed => StatusCode::BAD_GATEWAY,
            _ => StatusCode::FORBIDDEN,
        }
    }
}

struct Authorized {
    request: VerifiedProviderRequest,
    target: HeaderTarget,
    header: HeaderValue,
    /// PF-28-S02: scrubs this credential from the response.
    reflection: Option<ResponseGate>,
    generation: u64,
    guard: InFlightGuard,
}

impl Broker {
    fn control(&self, request: ControlRequest) -> ControlResponse {
        match &request {
            ControlRequest::Hello { .. } => ControlResponse::Error {
                code: ControlErrorCode::Malformed,
            },
            ControlRequest::Register {
                provider,
                binding,
                value,
            } => {
                let usable = binding.validate()
                    && providers::provider_by_id(*provider)
                        .request_header_value(value)
                        .is_some();
                self.register(
                    usable,
                    CredentialKind::Provider {
                        provider: *provider,
                        binding: binding.clone(),
                    },
                    value,
                )
            }
            ControlRequest::RegisterModel { binding, value } => {
                let usable = binding.validate() && binding.header.value(value).is_some();
                self.register(usable, CredentialKind::Model(binding.clone()), value)
            }
            ControlRequest::StashEnv { name, value } => self.stash_env(name, value),
            ControlRequest::RegisterModelStored {
                binding,
                provider_key_id,
                env_names,
                account,
            } => self.register_stored(binding, provider_key_id, env_names, account.as_ref()),
            ControlRequest::Unregister { reference } => {
                let Ok(reference) = CredentialReference::from_sha256_hex(reference.clone()) else {
                    return ControlResponse::Error {
                        code: ControlErrorCode::Malformed,
                    };
                };
                match self.state.lock() {
                    Ok(mut state) => {
                        state.credentials.remove(&reference);
                        ControlResponse::Unregistered
                    }
                    Err(_) => ControlResponse::Error {
                        code: ControlErrorCode::Unavailable,
                    },
                }
            }
            ControlRequest::Revoke => ControlResponse::Revoked {
                run_generation: self.revoke(),
            },
        }
    }

    fn register(&self, usable: bool, kind: CredentialKind, value: &str) -> ControlResponse {
        let usable = usable && !value.is_empty() && value.len() <= MAX_CREDENTIAL_VALUE_BYTES;
        if !usable {
            return ControlResponse::Error {
                code: ControlErrorCode::InvalidCredential,
            };
        }
        // PF-28-S02: a value the response gate cannot hold is not admitted.
        let Ok(reflection) = self
            .scrub_responses
            .then(|| ResponseGate::new([("broker:credential", value)]))
            .transpose()
        else {
            return ControlResponse::Error {
                code: ControlErrorCode::InvalidCredential,
            };
        };
        let Ok(mut state) = self.state.lock() else {
            return ControlResponse::Error {
                code: ControlErrorCode::Unavailable,
            };
        };
        if state.credentials.len() >= MAX_BROKER_CREDENTIALS {
            return ControlResponse::Error {
                code: ControlErrorCode::CapacityReached,
            };
        }
        let Ok(reference) = CredentialReference::from_sha256_hex(random_hex::<32>()) else {
            return ControlResponse::Error {
                code: ControlErrorCode::Unavailable,
            };
        };
        state.credentials.insert(
            reference.clone(),
            BrokerCredential {
                kind,
                value: Zeroizing::new(value.to_string()),
                reflection,
            },
        );
        ControlResponse::Registered {
            reference: reference.as_str().to_string(),
            run_generation: state.run_generation,
        }
    }

    fn stash_env(&self, name: &str, value: &str) -> ControlResponse {
        if !valid_env_name(name) || value.is_empty() || value.len() > MAX_CREDENTIAL_VALUE_BYTES {
            return ControlResponse::Error {
                code: ControlErrorCode::InvalidCredential,
            };
        }
        let Ok(mut state) = self.state.lock() else {
            return ControlResponse::Error {
                code: ControlErrorCode::Unavailable,
            };
        };
        if !state.env_stash.contains_key(name) && state.env_stash.len() >= MAX_STASHED_ENV {
            return ControlResponse::Error {
                code: ControlErrorCode::CapacityReached,
            };
        }
        state
            .env_stash
            .insert(name.to_string(), Zeroizing::new(value.to_string()));
        ControlResponse::Stashed
    }

    /// Registers a model key Core never held: a stashed variable, else the
    /// stored provider key read here.
    fn register_stored(
        &self,
        binding: &ModelBindingWire,
        provider_key_id: &str,
        env_names: &[String],
        account: Option<&StoredKeyAccount>,
    ) -> ControlResponse {
        if !valid_env_name(provider_key_id)
            || env_names.len() > MAX_STASHED_ENV
            || !env_names.iter().all(|name| valid_env_name(name))
            || account.is_some_and(|account| !account.validate())
        {
            return ControlResponse::Error {
                code: ControlErrorCode::Malformed,
            };
        }
        let stashed = match self.state.lock() {
            // PF-84: a named account never takes the default's env key.
            Ok(_) if account.is_some() => None,
            Ok(state) => env_names
                .iter()
                .find_map(|name| state.env_stash.get(name).cloned()),
            Err(_) => {
                return ControlResponse::Error {
                    code: ControlErrorCode::Unavailable,
                };
            }
        };
        let value = match stashed {
            Some(value) => value,
            None => {
                let Some((resolver, home)) = self.stored_keys.as_ref() else {
                    return ControlResponse::Error {
                        code: ControlErrorCode::NotFound,
                    };
                };
                match resolver(home, provider_key_id, account) {
                    Ok(Some(value)) if !value.trim().is_empty() => Zeroizing::new(value),
                    Ok(Some(value)) => {
                        drop(Zeroizing::new(value));
                        return ControlResponse::Error {
                            code: ControlErrorCode::NotFound,
                        };
                    }
                    Ok(None) => {
                        return ControlResponse::Error {
                            code: ControlErrorCode::NotFound,
                        };
                    }
                    Err(_) => {
                        return ControlResponse::Error {
                            code: ControlErrorCode::Unavailable,
                        };
                    }
                }
            }
        };
        let usable = binding.validate() && binding.header.value(&value).is_some();
        self.register(usable, CredentialKind::Model(binding.clone()), &value)
    }

    /// Advances the run generation, drops every credential and replay entry,
    /// and wakes in-flight requests so their upstream channels close.
    fn revoke(&self) -> u64 {
        let generation = match self.state.lock() {
            Ok(mut state) => {
                state.run_generation = state.run_generation.saturating_add(1);
                state.credentials.clear();
                state.env_stash.clear();
                state.replay = ReplayWindow::default();
                state.run_generation
            }
            Err(poisoned) => {
                let mut state = poisoned.into_inner();
                state.credentials.clear();
                state.env_stash.clear();
                state.run_generation = state.run_generation.saturating_add(1);
                state.run_generation
            }
        };
        self.generation.send_replace(generation);
        generation
    }

    fn expected_binding(&self, run_generation: u64) -> BrokerBinding {
        BrokerBinding {
            controller_instance: self.controller_instance.clone(),
            worker_instance: self.broker_instance.clone(),
            session_id: BROKER_SESSION_ID.to_string(),
            task_id: BROKER_TASK_ID.to_string(),
            run_id: self.controller_instance.clone(),
            run_generation,
        }
    }

    fn authorize(self: &Arc<Self>, request: &Request) -> Result<Authorized, DenyCode> {
        let mut frames = request.headers().get_all(FRAME_HEADER).iter();
        let (Some(frame), None) = (frames.next(), frames.next()) else {
            return Err(DenyCode::MissingFrame);
        };
        if frame.len() > MAX_FRAME_HEADER_BYTES {
            return Err(DenyCode::MalformedFrame);
        }
        let frame = base64::engine::general_purpose::STANDARD
            .decode(frame.as_bytes())
            .ok()
            .and_then(|bytes| SignedBrokerFrame::from_bytes(bytes).ok())
            .ok_or(DenyCode::MalformedFrame)?;
        let verified = self
            .mac
            .verify_provider_request(&frame)
            .map_err(|error| match error {
                codex_secret_broker::ipc::BrokerFrameError::AuthenticationFailed => {
                    DenyCode::AuthenticationFailed
                }
                _ => DenyCode::MalformedFrame,
            })?;

        let mut state = self.state.lock().map_err(|_| DenyCode::Revoked)?;
        if verified.binding != self.expected_binding(verified.binding.run_generation) {
            return Err(DenyCode::BindingMismatch);
        }
        if verified.binding.run_generation != state.run_generation {
            return Err(DenyCode::StaleGeneration);
        }
        if !state.replay.accept(verified.sequence) {
            return Err(DenyCode::Replay);
        }
        let credential = state
            .credentials
            .get(&verified.credential)
            .ok_or(DenyCode::UnknownCredential)?;
        let operation = &verified.request;
        let path = request
            .uri()
            .path_and_query()
            .map(rama_http::uri::PathAndQuery::as_str)
            .unwrap_or("/");
        if request.method().as_str() != operation.method() || path != operation.path() {
            return Err(DenyCode::RequestMismatch);
        }
        let (target, header) = match &credential.kind {
            CredentialKind::Provider { provider, binding } => {
                if !binding.matches_host(operation.host()) {
                    return Err(DenyCode::HostNotBound);
                }
                // PF-33-S02: proxied agent requests carry the guard's answers.
                if self.pin_connections && operation.pinned_addrs().is_empty() {
                    return Err(DenyCode::Unpinned);
                }
                (
                    HeaderTarget::Provider(*provider),
                    providers::provider_by_id(*provider)
                        .request_header_value(credential.value.as_str()),
                )
            }
            CredentialKind::Model(binding) => {
                if !binding.allows(operation.host(), operation.port(), operation.path()) {
                    return Err(DenyCode::HostNotBound);
                }
                (
                    HeaderTarget::Model(binding.header),
                    binding.header.value(credential.value.as_str()),
                )
            }
        };
        if state.in_flight >= MAX_BROKER_IN_FLIGHT {
            return Err(DenyCode::Capacity);
        }
        let mut header = header.ok_or(DenyCode::UnknownCredential)?;
        header.set_sensitive(true);
        let reflection = credential.reflection.clone();
        state.in_flight += 1;
        Ok(Authorized {
            target,
            generation: state.run_generation,
            request: verified,
            header,
            reflection,
            guard: InFlightGuard {
                broker: self.clone(),
            },
        })
    }

    async fn handle(self: Arc<Self>, request: Request) -> Response {
        let mut generation = self.generation.subscribe();
        let authorized = match self.authorize(&request) {
            Ok(authorized) => authorized,
            Err(code) => return deny(code),
        };
        if *generation.borrow_and_update() != authorized.generation {
            return deny(DenyCode::Revoked);
        }
        let operation = &authorized.request.request;
        let pinned_addrs = operation.pinned_addrs();
        let authority = if operation.port() == 443 {
            operation.host().to_string()
        } else {
            format!("{}:{}", operation.host(), operation.port())
        };
        let (mut parts, body) = request.into_parts();
        parts.headers.remove(FRAME_HEADER);
        match authorized.target {
            HeaderTarget::Provider(provider) => providers::provider_by_id(provider)
                .insert_request_header(&mut parts.headers, authorized.header),
            HeaderTarget::Model(header) => {
                // Core sends no credential header; drop any anyway.
                parts.headers.remove(rama_http::header::AUTHORIZATION);
                parts.headers.remove("x-api-key");
                parts.headers.insert(header.name(), authorized.header);
            }
        }
        let Ok(uri) = format!("https://{authority}{}", operation.path()).parse() else {
            return deny(DenyCode::RequestMismatch);
        };
        parts.uri = uri;
        let Ok(host) = HeaderValue::from_str(&authority) else {
            return deny(DenyCode::RequestMismatch);
        };
        parts.headers.insert(HOST, host);
        if authorized.reflection.is_some() {
            response_scrub::request_identity_body(&mut parts.headers);
        }
        // The agent may have spoken HTTP/2 to the proxy. Send HTTP/1.1 so an
        // origin that does not negotiate ALPN still understands the request.
        parts.version = Version::HTTP_11;
        if !pinned_addrs.is_empty() {
            // PF-33-S02: dial only the signed answers; never resolve the host.
            // `UpstreamClient` opens a fresh connection per request; if it ever
            // pools connections, the pool must be keyed by the pin.
            parts.extensions.insert(PinnedPeers::new(
                operation.host(),
                operation.port(),
                pinned_addrs.iter().copied(),
            ));
        }
        let upstream_request = Request::from_parts(parts, body);

        let response = tokio::select! {
            response = tokio::time::timeout(RESPONSE_IDLE_TIMEOUT, self.upstream.serve(upstream_request)) => response,
            _ = generation.changed() => return deny(DenyCode::Revoked),
        };
        let Ok(Ok(response)) = response else {
            return deny(DenyCode::UpstreamFailed);
        };
        let (parts, body) = response.into_parts();
        let body = RevocableBody {
            inner: body,
            revoked: Box::pin(async move {
                let _ = generation.changed().await;
            }),
            idle: Box::pin(tokio::time::sleep(RESPONSE_IDLE_TIMEOUT)),
            done: false,
            _guard: authorized.guard,
        };
        let response = Response::from_parts(parts, Body::new(body));
        match &authorized.reflection {
            Some(gate) => response_scrub::scrub_response(gate, response),
            None => response,
        }
    }
}

struct InFlightGuard {
    broker: Arc<Broker>,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = self.broker.state.lock() {
            state.in_flight = state.in_flight.saturating_sub(1);
        }
    }
}

/// Response body that terminates with an error as soon as the run generation
/// changes, so revocation closes streaming downloads too. A body that makes no
/// progress for [`RESPONSE_IDLE_TIMEOUT`] also ends, releasing its slot.
struct RevocableBody {
    inner: Body,
    revoked: Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
    idle: Pin<Box<tokio::time::Sleep>>,
    done: bool,
    _guard: InFlightGuard,
}

impl StreamingBody for RevocableBody {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<rama_http::body::Frame<Self::Data>, Self::Error>>> {
        if self.done {
            return Poll::Ready(None);
        }
        if self.revoked.as_mut().poll(cx).is_ready() {
            self.done = true;
            return Poll::Ready(Some(Err("credential broker run was revoked".into())));
        }
        let frame = Pin::new(&mut self.inner)
            .poll_frame(cx)
            .map_err(BoxError::from);
        if frame.is_ready() {
            let deadline = tokio::time::Instant::now() + RESPONSE_IDLE_TIMEOUT;
            self.idle.as_mut().reset(deadline);
            return frame;
        }
        if self.idle.as_mut().poll(cx).is_ready() {
            self.done = true;
            return Poll::Ready(Some(Err("credential broker response went idle".into())));
        }
        Poll::Pending
    }
}

fn deny(code: DenyCode) -> Response {
    Response::builder()
        .status(code.status())
        .header("content-type", "text/plain")
        .header(BROKER_ERROR_HEADER, code.as_str())
        .body(Body::from(format!(
            "credential broker denied: {}\n",
            code.as_str()
        )))
        .unwrap_or_else(|_| Response::new(Body::from("credential broker denied\n")))
}

fn random_hex<const N: usize>() -> String {
    let mut bytes = Zeroizing::new([0_u8; N]);
    rand::rng().fill_bytes(bytes.as_mut());
    encode_hex(bytes.as_ref())
}

pub(crate) async fn read_control_line<R>(
    reader: &mut R,
) -> std::io::Result<Option<Zeroizing<Vec<u8>>>>
where
    R: AsyncBufRead + Unpin,
{
    let mut line = Zeroizing::new(Vec::new());
    let read = reader
        .take(MAX_CONTROL_LINE_BYTES as u64 + 1)
        .read_until(b'\n', &mut line)
        .await?;
    if read == 0 {
        return Ok(None);
    }
    if line.last() != Some(&b'\n') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "control line exceeds the bounded size",
        ));
    }
    line.pop();
    Ok(Some(line))
}

async fn write_json_line<W, T>(writer: &mut W, message: &T) -> std::io::Result<()>
where
    W: tokio::io::AsyncWrite + Unpin,
    T: serde::Serialize,
{
    let mut line = serde_json::to_vec(message).map_err(std::io::Error::other)?;
    line.push(b'\n');
    writer.write_all(&line).await?;
    writer.flush().await
}

/// Unix transport: sockets in a private directory under the runtime
/// directory, peers checked by uid and pid.
#[cfg(unix)]
mod transport {
    use super::ACCEPT_RETRY_DELAY;
    use super::Broker;
    use super::serve_connection;
    use anyhow::Context as _;
    use rama_unix::server::UnixListener;
    use std::os::unix::fs::MetadataExt as _;
    use std::os::unix::fs::PermissionsExt as _;
    use std::sync::Arc;
    use tokio::signal::unix::SignalKind;
    use tokio::signal::unix::signal;

    /// The spawning controller (Unix: polled by process id).
    pub(super) struct Parent {
        pub(super) pid: u32,
    }

    impl Parent {
        pub(super) fn open() -> std::io::Result<Self> {
            Ok(Self {
                pid: std::os::unix::process::parent_id(),
            })
        }
    }

    pub(super) struct Endpoint {
        /// Dropping it removes the private socket directory.
        _socket_dir: tempfile::TempDir,
        owner_uid: u32,
        control_path: std::path::PathBuf,
        control: Option<tokio::net::UnixListener>,
        data_path: std::path::PathBuf,
        data: Option<UnixListener>,
    }

    impl Endpoint {
        pub(super) async fn bind(runtime_dir: &std::path::Path) -> anyhow::Result<Self> {
            let socket_dir = tempfile::Builder::new()
                .prefix("cbk-")
                .tempdir_in(runtime_dir)
                .context("create private broker directory")?;
            std::fs::set_permissions(socket_dir.path(), std::fs::Permissions::from_mode(0o700))?;
            let owner_uid = std::fs::metadata(socket_dir.path())?.uid();
            let control_path = socket_dir.path().join("c.sock");
            let control = tokio::net::UnixListener::bind(&control_path)?;
            std::fs::set_permissions(&control_path, std::fs::Permissions::from_mode(0o600))?;
            let data_path = socket_dir.path().join("b.sock");
            let data = UnixListener::bind_path(&data_path).await?;
            std::fs::set_permissions(&data_path, std::fs::Permissions::from_mode(0o600))?;
            Ok(Self {
                _socket_dir: socket_dir,
                owner_uid,
                control_path,
                control: Some(control),
                data_path,
                data: Some(data),
            })
        }

        pub(super) fn control_name(&self) -> String {
            self.control_path.to_string_lossy().into_owned()
        }

        pub(super) fn data_name(&self) -> String {
            self.data_path.to_string_lossy().into_owned()
        }

        /// Accepts the first control connection whose OS peer is the spawning
        /// controller; any other peer is dropped before a byte is read.
        pub(super) async fn accept_controller(
            &mut self,
            parent_pid: u32,
        ) -> std::io::Result<tokio::net::UnixStream> {
            let listener = self
                .control
                .take()
                .ok_or(std::io::ErrorKind::NotConnected)?;
            loop {
                let (stream, _) = listener.accept().await?;
                if peer_is(&stream, self.owner_uid, parent_pid) {
                    let _ = std::fs::remove_file(&self.control_path);
                    return Ok(stream);
                }
            }
        }

        pub(super) fn serve(&mut self, broker: Arc<Broker>) -> tokio::task::JoinHandle<()> {
            let listener = self.data.take();
            let owner_uid = self.owner_uid;
            tokio::spawn(async move {
                if let Some(listener) = listener {
                    accept_loop(listener, owner_uid, broker).await;
                }
            })
        }
    }

    fn peer_is(stream: &tokio::net::UnixStream, owner_uid: u32, pid: u32) -> bool {
        stream.peer_cred().is_ok_and(|peer| {
            peer.uid() == owner_uid
                && peer
                    .pid()
                    .and_then(|peer_pid| u32::try_from(peer_pid).ok())
                    .is_some_and(|peer_pid| peer_pid == pid)
        })
    }

    async fn accept_loop(listener: UnixListener, owner_uid: u32, broker: Arc<Broker>) {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                // Back off so a persistent error (for example, no free file
                // descriptors) cannot pin a CPU.
                tokio::time::sleep(ACCEPT_RETRY_DELAY).await;
                continue;
            };
            if !peer_is(&stream.stream, owner_uid, broker.controller_pid) {
                // Wrong OS peer: close before reading a single request byte.
                drop(stream);
                continue;
            }
            tokio::spawn(serve_connection(stream, broker.clone()));
        }
    }

    /// Resolves on a termination signal or when the controller has died
    /// (the broker was reparented).
    pub(super) fn shutdown_requested(
        parent: Parent,
    ) -> anyhow::Result<impl std::future::Future<Output = ()>> {
        let parent_pid = parent.pid;
        let mut terminate = signal(SignalKind::terminate())?;
        let mut hangup = signal(SignalKind::hangup())?;
        let mut interrupt = signal(SignalKind::interrupt())?;
        Ok(async move {
            let mut interval = tokio::time::interval(super::PARENT_CHECK_INTERVAL);
            loop {
                tokio::select! {
                    _ = terminate.recv() => return,
                    _ = hangup.recv() => return,
                    _ = interrupt.recv() => return,
                    _ = interval.tick() => {
                        if std::os::unix::process::parent_id() != parent_pid {
                            return;
                        }
                    }
                }
            }
        })
    }
}

/// PF-27-S06 Windows transport: named pipes created by the broker with a
/// DACL for its user, peers checked by process id.
#[cfg(windows)]
mod transport {
    use super::ACCEPT_RETRY_DELAY;
    use super::Broker;
    use super::serve_connection;
    use crate::credential_broker::isolated::pipe;
    use std::sync::Arc;
    use tokio::net::windows::named_pipe::NamedPipeServer;

    /// The spawning controller, held open (Windows).
    pub(super) type Parent = pipe::ParentProcess;

    pub(super) struct Endpoint {
        control_name: String,
        control: Option<pipe::SinglePipeListener>,
        data_name: String,
        data: Option<pipe::PipeListener>,
    }

    impl Endpoint {
        pub(super) async fn bind(_runtime_dir: &std::path::Path) -> anyhow::Result<Self> {
            let (control_name, data_name) = pipe::pipe_names();
            // #390: one control instance at most, so none can be added.
            let control = pipe::SinglePipeListener::bind(&control_name)?;
            let data = pipe::PipeListener::bind(&data_name)?;
            Ok(Self {
                control_name,
                control: Some(control),
                data_name,
                data: Some(data),
            })
        }

        pub(super) fn control_name(&self) -> String {
            self.control_name.clone()
        }

        pub(super) fn data_name(&self) -> String {
            self.data_name.clone()
        }

        /// Accepts the controller on the control pipe, its only instance
        /// (#390); other clients are dropped unread.
        pub(super) async fn accept_controller(
            &mut self,
            parent_pid: u32,
        ) -> std::io::Result<NamedPipeServer> {
            let listener = self
                .control
                .take()
                .ok_or(std::io::ErrorKind::NotConnected)?;
            listener.accept(parent_pid).await
        }

        pub(super) fn serve(&mut self, broker: Arc<Broker>) -> tokio::task::JoinHandle<()> {
            let listener = self.data.take();
            tokio::spawn(async move {
                let Some(mut listener) = listener else {
                    return;
                };
                loop {
                    match listener.accept(broker.controller_pid).await {
                        Ok(mut stream) => {
                            let broker = broker.clone();
                            tokio::spawn(async move {
                                // #390: prove the channel key before Core
                                // sends its request.
                                if pipe::answer_peer_challenge(
                                    &mut stream,
                                    &broker.mac,
                                    broker.controller_pid,
                                )
                                .await
                                .is_ok()
                                {
                                    serve_connection(pipe::PipeStream::new(stream), broker).await;
                                }
                            });
                        }
                        Err(_) => tokio::time::sleep(ACCEPT_RETRY_DELAY).await,
                    }
                }
            })
        }
    }

    /// Resolves when the controller process exits.
    pub(super) fn shutdown_requested(
        parent: Parent,
    ) -> anyhow::Result<impl std::future::Future<Output = ()>> {
        let exited = parent.exited();
        Ok(async move {
            let _ = exited.await;
        })
    }
}
