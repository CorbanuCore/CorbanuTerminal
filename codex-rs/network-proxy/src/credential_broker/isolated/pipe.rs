//! PF-27-S06: Windows named-pipe transport for the isolated credential broker.
//!
//! The broker creates both of its pipes (control and data):
//! - under a random name, as the first instance, so nothing can create it
//!   first (see #390 below for instances added later). The name is not a
//!   secret: pipe names can be listed;
//! - with a protected DACL that grants only the broker's own user SID, and
//!   the capability SID of the broker's token (PF-27-S08), without which the
//!   broker could not add pipe instances. The elevated sandbox runs commands
//!   as another user, which cannot open the pipe at all. The unelevated
//!   sandbox's write-restricted token cannot open it for writing (its
//!   restricting SIDs are not granted), but can still connect read-only
//!   (reads are checked against its normal SIDs);
//! - refusing remote clients, with handles that are never inheritable.
//!
//! The guarantee is the peer check: every connection is matched to the
//! expected process before a byte is read or written, so the broker serves
//! only its controller, and Core talks only to the broker it spawned. Any
//! same-user process (the DACL grants the user everything, including adding
//! instances) can connect and be dropped, which can delay Core (a denial of
//! service, not a disclosure).
//!
//! #390: the server process id Windows reports belongs to the process that
//! created the pipe's first instance, for every instance (measured), so it
//! cannot tell the broker's instances from ones another same-user process
//! adds to a live pipe. Hence:
//! - the control pipe allows one instance only, which the broker creates and
//!   never replaces, so nobody can add one;
//! - on each data connection Core sends a random challenge and writes its
//!   request only after the server answers with a MAC under the channel key
//!   over the challenge and both process ids as the server sees them
//!   ([`BrokerChannelMac::pipe_peer_proof`]). An added instance cannot
//!   answer, and relaying to the broker fails: the broker serves only its
//!   controller, and the relay's process id is in the MAC;
//! - a name that another process created first (before the broker starts,
//!   or after it exits) reports that process's id, so Core refuses it before
//!   writing.
//!
//! Refused servers are closed; Core tries again until its connect deadline
//! and then fails with [`squatted_pipe_error`]. A broker that finds its name
//! taken does not join the existing pipe: it reports that and exits.
//!
//! Because the broker runs at low integrity
//! (PF-27-S08), its pipe objects get a low integrity label, so low-integrity
//! processes of the user can now open them too (before, the implicit medium
//! label blocked them); the capability ACE admits only processes with equal
//! trust: the broker, its holder, and COM servers it launches under its own
//! token. Clients connect at identification-level impersonation, so a pipe
//! server cannot act with Core's token. The broker holds a handle to its
//! controller and exits with it, so the controller's process id cannot be
//! reused while the broker serves it.

use codex_secret_broker::BrokerChannelMac;
use codex_secret_broker::ipc::PIPE_CHALLENGE_BYTES;
use codex_secret_broker::ipc::PIPE_PROOF_BYTES;
use rama_core::Service;
use rama_core::error::BoxError;
use rama_core::extensions::Extensions;
use rama_core::extensions::ExtensionsMut;
use rama_core::extensions::ExtensionsRef;
use rama_net::client::EstablishedClientConnection;
use std::ffi::c_void;
use std::io;
use std::os::windows::fs::OpenOptionsExt as _;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::FromRawHandle as _;
use std::os::windows::io::IntoRawHandle as _;
use std::os::windows::io::OwnedHandle;
use std::os::windows::io::RawHandle;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt as _;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt as _;
use tokio::io::ReadBuf;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::net::windows::named_pipe::NamedPipeClient;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio::net::windows::named_pipe::PipeMode;
use tokio::net::windows::named_pipe::ServerOptions;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_BROKEN_PIPE;
use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows_sys::Win32::Foundation::ERROR_IO_PENDING;
use windows_sys::Win32::Foundation::ERROR_PIPE_CONNECTED;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::SDDL_REVISION_1;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_FIRST_PIPE_INSTANCE;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows_sys::Win32::Storage::FileSystem::ReadFile;
use windows_sys::Win32::Storage::FileSystem::WriteFile;
use windows_sys::Win32::System::IO::CancelIoEx;
use windows_sys::Win32::System::IO::GetOverlappedResult;
use windows_sys::Win32::System::IO::OVERLAPPED;
use windows_sys::Win32::System::Pipes::ConnectNamedPipe;
use windows_sys::Win32::System::Pipes::CreateNamedPipeW;
use windows_sys::Win32::System::Pipes::DisconnectNamedPipe;
use windows_sys::Win32::System::Pipes::GetNamedPipeClientProcessId;
use windows_sys::Win32::System::Pipes::GetNamedPipeServerProcessId;
use windows_sys::Win32::System::Pipes::PIPE_READMODE_BYTE;
use windows_sys::Win32::System::Pipes::PIPE_REJECT_REMOTE_CLIENTS;
use windows_sys::Win32::System::Pipes::PIPE_TYPE_BYTE;
use windows_sys::Win32::System::Pipes::PIPE_WAIT;
use windows_sys::Win32::System::Pipes::WaitNamedPipeW;
use windows_sys::Win32::System::Threading::CreateEventW;
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetProcessTimes;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_SYNCHRONIZE;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

/// Every broker pipe name starts with this.
pub(crate) const PIPE_PREFIX: &str = r"\\.\pipe\corbanu-cbk-";
const CONTROL_SUFFIX: &str = "-c";
const DATA_SUFFIX: &str = "-b";
const NONCE_HEX_LEN: usize = 32;
/// `SECURITY_IDENTIFICATION` (`SecurityIdentification << 16`); std and tokio
/// add `SECURITY_SQOS_PRESENT` when QoS flags are set.
const SECURITY_IDENTIFICATION: u32 = 0x0001_0000;
const ERROR_PIPE_BUSY: i32 = 231;
const ERROR_ACCESS_DENIED: i32 = 5;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const BUSY_RETRY: Duration = Duration::from_millis(10);
/// How long either side waits for the other's part of the data-pipe peer
/// challenge (#390).
const PEER_PROOF_TIMEOUT: Duration = Duration::from_secs(1);
/// Buffer size of the control pipe (tokio's default for its pipes).
const SINGLE_PIPE_BUFFER: u32 = 65_536;
/// Tests only (#390): the nonce the broker uses for its pipe names, so a
/// probe can claim them first. Read only in test builds.
#[cfg(test)]
pub(crate) const TEST_NONCE_ENV: &str = "CODEX_SEC390_TEST_PIPE_NONCE";

/// A fresh pair of pipe names: (control, data).
pub(crate) fn pipe_names() -> (String, String) {
    #[cfg(test)]
    if let Ok(nonce) = std::env::var(TEST_NONCE_ENV) {
        return (
            format!("{PIPE_PREFIX}{nonce}{CONTROL_SUFFIX}"),
            format!("{PIPE_PREFIX}{nonce}{DATA_SUFFIX}"),
        );
    }
    let nonce: String = rand::random::<[u8; NONCE_HEX_LEN / 2]>()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    (
        format!("{PIPE_PREFIX}{nonce}{CONTROL_SUFFIX}"),
        format!("{PIPE_PREFIX}{nonce}{DATA_SUFFIX}"),
    )
}

/// True for a control (`control == true`) or data pipe name made by
/// [`pipe_names`]; the controller checks names it reads from the broker.
pub(crate) fn valid_pipe_name(name: &str, control: bool) -> bool {
    let suffix = if control { CONTROL_SUFFIX } else { DATA_SUFFIX };
    name.strip_prefix(PIPE_PREFIX)
        .and_then(|rest| rest.strip_suffix(suffix))
        .is_some_and(|nonce| {
            nonce.len() == NONCE_HEX_LEN && nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
}

/// Server side of one broker pipe. One instance always listens; a connected
/// instance is handed out only if its client is the expected process.
pub(crate) struct PipeListener {
    name: String,
    listening: NamedPipeServer,
}

impl PipeListener {
    /// Creates the first instance of a pipe that grows an instance per
    /// connection (the data pipe); fails if the name already exists, with
    /// [`io::ErrorKind::AddrInUse`] when another process holds it (#390).
    /// Never joins an existing pipe.
    pub(crate) fn bind(name: &str) -> io::Result<Self> {
        let listening =
            create_instance(name, /*first*/ true).map_err(|error| name_taken(name, error))?;
        Ok(Self {
            name: name.to_string(),
            listening,
        })
    }

    /// Waits for a client whose process id is `expected_pid`. Other clients
    /// are disconnected before anything is read from them. The next instance
    /// is created before a connected one is released, so the name never
    /// becomes free for another process to claim.
    pub(crate) async fn accept(&mut self, expected_pid: u32) -> io::Result<NamedPipeServer> {
        loop {
            self.listening.connect().await?;
            let next = match create_instance(&self.name, /*first*/ false) {
                Ok(next) => next,
                Err(error) => {
                    // Never leave an unchecked client connected.
                    let _ = self.listening.disconnect();
                    return Err(error);
                }
            };
            let connected = std::mem::replace(&mut self.listening, next);
            if client_pid(&connected) == Some(expected_pid) {
                return Ok(connected);
            }
            tracing::warn!("credential broker pipe: dropped a client that is not the controller");
            let _ = connected.disconnect();
        }
    }
}

fn create_instance(name: &str, first: bool) -> io::Result<NamedPipeServer> {
    let descriptor = OwnerOnlyDescriptor::new()?;
    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        // Never inheritable: no child of the broker or Core receives it.
        bInheritHandle: 0,
    };
    let mut options = ServerOptions::new();
    options
        .access_inbound(true)
        .access_outbound(true)
        .first_pipe_instance(first)
        .reject_remote_clients(true)
        .pipe_mode(PipeMode::Byte);
    // SAFETY: `attributes` and the descriptor it points to outlive the call.
    unsafe {
        options.create_with_security_attributes_raw(
            name,
            (&mut attributes as *mut SECURITY_ATTRIBUTES).cast::<c_void>(),
        )
    }
}

/// #390: a first-instance create failed. FILE_FLAG_FIRST_PIPE_INSTANCE on an
/// existing name fails with ERROR_ACCESS_DENIED, a name at its instance limit
/// with ERROR_PIPE_BUSY; either is [`io::ErrorKind::AddrInUse`] only when the
/// name really exists, so other failures keep their own error.
fn name_taken(name: &str, error: io::Error) -> io::Error {
    if matches!(
        error.raw_os_error(),
        Some(ERROR_ACCESS_DENIED | ERROR_PIPE_BUSY)
    ) && pipe_exists(name)
    {
        io::Error::new(
            io::ErrorKind::AddrInUse,
            "credential broker pipe name is held by another process",
        )
    } else {
        error
    }
}

/// Whether a pipe of this name exists, without connecting to it.
fn pipe_exists(name: &str) -> bool {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is NUL-terminated. Waiting 1 ms connects to nothing.
    let available = unsafe { WaitNamedPipeW(wide.as_ptr(), 1) };
    // SAFETY: reads this thread's last error.
    available != 0 || unsafe { GetLastError() } != ERROR_FILE_NOT_FOUND
}

/// #390: server side of the control pipe, which allows one instance at most
/// (Windows enforces the first instance's limit), so no other process can
/// add one. Its handle is not given to tokio until the controller is
/// connected: tokio reads ahead from a connected pipe, so an instance it has
/// seen could pass another client's bytes to the next one.
pub(crate) struct SinglePipeListener {
    handle: OwnedHandle,
}

impl SinglePipeListener {
    /// Creates the one instance; fails like [`PipeListener::bind`].
    pub(crate) fn bind(name: &str) -> io::Result<Self> {
        let descriptor = OwnerOnlyDescriptor::new()?;
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0,
            // Never inheritable: no child of the broker or Core receives it.
            bInheritHandle: 0,
        };
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is NUL-terminated; `attributes` and its descriptor
        // outlive the call.
        let handle = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                /*nMaxInstances*/ 1,
                SINGLE_PIPE_BUFFER,
                SINGLE_PIPE_BUFFER,
                /*nDefaultTimeOut*/ 0,
                &attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(name_taken(name, io::Error::last_os_error()));
        }
        // SAFETY: a fresh handle that nothing else owns.
        let handle = unsafe { OwnedHandle::from_raw_handle(handle as RawHandle) };
        Ok(Self { handle })
    }

    /// Waits for the client whose process id is `expected_pid`. Another
    /// client is disconnected before anything is read from it, and the
    /// instance waits again (blocking; call from a blocking thread).
    pub(crate) fn accept_blocking(self, expected_pid: u32) -> io::Result<OwnedHandle> {
        let raw = self.handle.as_raw_handle() as HANDLE;
        loop {
            connect_blocking(raw)?;
            let mut pid = 0_u32;
            // SAFETY: a valid pipe handle and out pointer.
            let known = unsafe { GetNamedPipeClientProcessId(raw, &mut pid) } != 0;
            if known && pid == expected_pid {
                return Ok(self.handle);
            }
            tracing::warn!(
                "credential broker pipe: dropped a control client that is not the controller"
            );
            // SAFETY: a valid pipe handle; discards the client unread.
            if unsafe { DisconnectNamedPipe(raw) } == 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    /// [`Self::accept_blocking`] on a blocking thread, then the connected
    /// instance for async use.
    pub(crate) async fn accept(self, expected_pid: u32) -> io::Result<NamedPipeServer> {
        let handle = tokio::task::spawn_blocking(move || self.accept_blocking(expected_pid))
            .await
            .map_err(io::Error::other)??;
        // SAFETY: an overlapped pipe server handle, owned and handed over.
        unsafe { NamedPipeServer::from_raw_handle(handle.into_raw_handle()) }
    }
}

/// Waits until a client connects to `pipe`, an overlapped server handle.
fn connect_blocking(pipe: HANDLE) -> io::Result<()> {
    // SAFETY: a manual-reset event for this operation; closed below.
    let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if event == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: zeroed POD with the event set.
    let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
    overlapped.hEvent = event;
    // SAFETY: `overlapped` outlives the operation: GetOverlappedResult waits.
    let result = unsafe {
        if ConnectNamedPipe(pipe, &mut overlapped) != 0 {
            Ok(())
        } else {
            match GetLastError() {
                ERROR_PIPE_CONNECTED => Ok(()),
                ERROR_IO_PENDING => {
                    let mut transferred = 0_u32;
                    if GetOverlappedResult(pipe, &overlapped, &mut transferred, 1) != 0 {
                        Ok(())
                    } else {
                        Err(io::Error::last_os_error())
                    }
                }
                _ => Err(io::Error::last_os_error()),
            }
        }
    };
    // SAFETY: created above.
    unsafe { CloseHandle(event) };
    result
}

/// `D:P(A;;GA;;;<current user>)` plus the current token's capability SIDs,
/// freed on drop.
struct OwnerOnlyDescriptor(PSECURITY_DESCRIPTOR);

impl OwnerOnlyDescriptor {
    fn new() -> io::Result<Self> {
        let user = codex_process_hardening::current_user_sid_string()?;
        let capabilities = codex_process_hardening::current_capability_sid_strings()?;
        let sddl: Vec<u16> = pipe_dacl_sddl(&user, &capabilities)
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        // SAFETY: `sddl` is NUL-terminated; the result is freed on drop.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(descriptor))
    }
}

impl Drop for OwnerOnlyDescriptor {
    fn drop(&mut self) {
        // SAFETY: allocated by the SDDL conversion with LocalAlloc.
        unsafe { LocalFree(self.0 as HLOCAL) };
    }
}

/// The protected DACL on every broker pipe instance: the user, and the
/// capability SIDs of the creator's token (the broker token's, which only
/// the broker holds; none for an unrestricted token).
pub(crate) fn pipe_dacl_sddl(user_sid: &str, capability_sids: &[String]) -> String {
    let mut sddl = format!("D:P(A;;GA;;;{user_sid})");
    for sid in capability_sids {
        sddl.push_str(&format!("(A;;GA;;;{sid})"));
    }
    sddl
}

/// The process id of the client connected to a server instance.
pub(crate) fn client_pid(server: &NamedPipeServer) -> Option<u32> {
    let mut pid = 0_u32;
    // SAFETY: a valid pipe handle and out pointer.
    let ok = unsafe { GetNamedPipeClientProcessId(server.as_raw_handle() as HANDLE, &mut pid) };
    (ok != 0).then_some(pid)
}

/// The process id of the server a client handle is connected to.
pub(crate) fn server_pid(client: &impl AsRawHandle) -> Option<u32> {
    let mut pid = 0_u32;
    // SAFETY: a valid pipe handle and out pointer.
    let ok = unsafe { GetNamedPipeServerProcessId(client.as_raw_handle() as HANDLE, &mut pid) };
    (ok != 0).then_some(pid)
}

#[derive(Debug, thiserror::Error)]
#[error("credential broker pipe is served by another process")]
struct SquattedPipe;

/// The error for a broker pipe that only another process serves (#390).
pub(crate) fn squatted_pipe_error() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, SquattedPipe)
}

/// True for [`squatted_pipe_error`].
pub(crate) fn is_squatted_pipe_error(error: &io::Error) -> bool {
    matches!(error.get_ref(), Some(inner) if inner.is::<SquattedPipe>())
}

/// What one attempt to open a broker pipe found.
enum Attempt<T> {
    Served(T),
    /// Another process serves the instance reached; it was closed unused.
    Refused,
    /// The server let go before its process could be read; nothing was
    /// sent. The broker never drops its controller, so this counts as
    /// refused.
    Dropped(io::Error),
    Busy,
    Failed(io::Error),
}

/// Checks the server of a freshly opened pipe before anything is written:
/// a pipe served by another process is closed unused (#390).
fn check_server<T: AsRawHandle>(pipe: T, broker_pid: u32) -> Attempt<T> {
    match server_pid(&pipe) {
        Some(pid) if pid == broker_pid => Attempt::Served(pipe),
        Some(_) => {
            tracing::warn!(
                "credential broker pipe: refused a pipe server that is not the broker; nothing was sent"
            );
            drop(pipe);
            Attempt::Refused
        }
        None => Attempt::Dropped(io::ErrorKind::BrokenPipe.into()),
    }
}

fn open_attempt<T: AsRawHandle>(opened: io::Result<T>, broker_pid: u32) -> Attempt<T> {
    match opened {
        Ok(pipe) => check_server(pipe, broker_pid),
        Err(error) if error.raw_os_error() == Some(ERROR_PIPE_BUSY) => Attempt::Busy,
        Err(error) => Attempt::Failed(error),
    }
}

/// The result once the connect deadline passes: a refused server makes it
/// [`squatted_pipe_error`], whatever the last attempt saw.
fn deadline_error(refused: bool, last: io::Error) -> io::Error {
    if refused { squatted_pipe_error() } else { last }
}

/// Opens the broker's control pipe for blocking use, if `name` is a broker
/// control pipe served by process `broker_pid`. Instances served by another
/// process are closed unused and the open is retried until the deadline.
pub(crate) fn connect_control(name: &str, broker_pid: u32) -> io::Result<ControlPipe> {
    if !valid_pipe_name(name, /*control*/ true) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let deadline = std::time::Instant::now() + CONNECT_TIMEOUT;
    let mut refused = false;
    let file = loop {
        let opened = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(name);
        let last = match open_attempt(opened, broker_pid) {
            Attempt::Served(file) => break file,
            Attempt::Refused => {
                refused = true;
                squatted_pipe_error()
            }
            Attempt::Dropped(error) => {
                refused = true;
                error
            }
            Attempt::Busy => io::Error::from_raw_os_error(ERROR_PIPE_BUSY),
            Attempt::Failed(error) => return Err(deadline_error(refused, error)),
        };
        if std::time::Instant::now() >= deadline {
            return Err(deadline_error(refused, last));
        }
        std::thread::sleep(BUSY_RETRY);
    };
    Ok(ControlPipe {
        handle: Arc::new(OwnedHandle::from(file)),
        closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    })
}

/// The controller's end of the control pipe, opened for overlapped I/O. A
/// synchronous pipe handle serializes every operation on it, so a write would
/// wait behind the reader thread's pending read; each operation here has its
/// own `OVERLAPPED` instead. Clones share the handle.
#[derive(Clone)]
pub(crate) struct ControlPipe {
    handle: Arc<OwnedHandle>,
    closed: Arc<std::sync::atomic::AtomicBool>,
}

impl ControlPipe {
    pub(crate) fn try_clone(&self) -> io::Result<Self> {
        Ok(self.clone())
    }

    /// Cancels pending I/O (the reader's read returns), so the handle closes
    /// once the last clone is dropped and the broker sees the pipe end.
    pub(crate) fn shutdown(&self) {
        // Set first: an operation started after the cancel sees the flag.
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        // SAFETY: a valid handle; null cancels every operation on it.
        unsafe { CancelIoEx(self.raw(), std::ptr::null()) };
    }

    fn raw(&self) -> HANDLE {
        self.handle.as_raw_handle() as HANDLE
    }

    fn overlapped_io(&self, read: bool, buffer: *mut u8, len: usize) -> io::Result<usize> {
        let len = u32::try_from(len).unwrap_or(u32::MAX);
        // SAFETY: a manual-reset event for this operation; closed below.
        let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
        if event == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: zeroed POD with the event set.
        let mut overlapped: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlapped.hEvent = event;
        let mut transferred = 0_u32;
        // SAFETY: `buffer` is valid for `len` bytes and, like `overlapped`,
        // outlives the operation: GetOverlappedResult waits for it to finish.
        let result = unsafe {
            // Byte counts come from GetOverlappedResult, not the start call.
            let started = if read {
                ReadFile(
                    self.raw(),
                    buffer,
                    len,
                    std::ptr::null_mut(),
                    &mut overlapped,
                )
            } else {
                WriteFile(
                    self.raw(),
                    buffer,
                    len,
                    std::ptr::null_mut(),
                    &mut overlapped,
                )
            };
            if started != 0 || GetLastError() == ERROR_IO_PENDING {
                // A read that started after shutdown() cancelled is
                // cancelled here instead of waiting forever.
                if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
                    CancelIoEx(self.raw(), &overlapped);
                }
                if GetOverlappedResult(self.raw(), &overlapped, &mut transferred, 1) != 0 {
                    Ok(transferred as usize)
                } else {
                    Err(io::Error::last_os_error())
                }
            } else {
                Err(io::Error::last_os_error())
            }
        };
        // SAFETY: created above.
        unsafe { CloseHandle(event) };
        match result {
            // The broker closed its end: end of stream.
            Err(error) if read && error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) => Ok(0),
            other => other,
        }
    }
}

impl io::Read for ControlPipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.overlapped_io(/*read*/ true, buf.as_mut_ptr(), buf.len())
    }
}

impl io::Write for ControlPipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.overlapped_io(/*read*/ false, buf.as_ptr().cast_mut(), buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsRawHandle for ControlPipe {
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.handle.as_raw_handle()
    }
}

/// Opens one data connection to the broker, if `name` is a broker data pipe
/// served by process `broker_pid` that proves it holds the channel key
/// `mac` (#390). Waits briefly while every instance is busy; servers that
/// are another process or cannot answer the challenge are closed, having
/// received only the challenge, and the open is retried until the deadline.
pub(crate) async fn connect_data(
    name: &str,
    broker_pid: u32,
    mac: &BrokerChannelMac,
) -> io::Result<NamedPipeClient> {
    if !valid_pipe_name(name, /*control*/ false) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let deadline = tokio::time::Instant::now() + CONNECT_TIMEOUT;
    let mut refused = false;
    loop {
        let opened = ClientOptions::new()
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(name);
        let last = match open_attempt(opened, broker_pid) {
            Attempt::Served(mut client) => {
                let outcome = peer_proof(&mut client, broker_pid, mac, deadline).await;
                if matches!(outcome, PeerProof::Proven) {
                    return Ok(client);
                }
                // The broker never hangs up on its controller or fails its
                // proof, so each of these counts as refused; nothing was sent.
                refused = true;
                match outcome {
                    PeerProof::Proven | PeerProof::Wrong => tracing::warn!(
                        "credential broker pipe: refused a pipe server whose proof of the channel key failed; no request was sent"
                    ),
                    PeerProof::TimedOut => tracing::warn!(
                        "credential broker pipe: refused a pipe server that did not prove the channel key in time; no request was sent"
                    ),
                    PeerProof::Closed => tracing::warn!(
                        "credential broker pipe: a pipe server hung up before proving the channel key; no request was sent"
                    ),
                }
                squatted_pipe_error()
            }
            Attempt::Refused => {
                refused = true;
                squatted_pipe_error()
            }
            Attempt::Dropped(error) => {
                refused = true;
                error
            }
            Attempt::Busy => io::Error::from_raw_os_error(ERROR_PIPE_BUSY),
            Attempt::Failed(error) => return Err(deadline_error(refused, error)),
        };
        if tokio::time::Instant::now() >= deadline {
            return Err(deadline_error(refused, last));
        }
        tokio::time::sleep(BUSY_RETRY).await;
    }
}

/// What a data-pipe server did with Core's peer challenge (#390).
enum PeerProof {
    Proven,
    /// It closed the connection without answering.
    Closed,
    /// Its answer does not verify.
    Wrong,
    /// It did not answer in time (a starved broker would too).
    TimedOut,
}

/// Core's side of the data-pipe peer challenge (#390): sends a fresh random
/// challenge and checks the server's proof, bounded by `deadline`.
async fn peer_proof(
    client: &mut NamedPipeClient,
    broker_pid: u32,
    mac: &BrokerChannelMac,
    deadline: tokio::time::Instant,
) -> PeerProof {
    let challenge: [u8; PIPE_CHALLENGE_BYTES] = rand::random();
    let mut proof = [0_u8; PIPE_PROOF_BYTES];
    let wait = deadline
        .saturating_duration_since(tokio::time::Instant::now())
        .clamp(BUSY_RETRY, PEER_PROOF_TIMEOUT);
    let exchanged = tokio::time::timeout(wait, async {
        client.write_all(&challenge).await?;
        client.read_exact(&mut proof).await
    })
    .await;
    match exchanged {
        Ok(Ok(_))
            if mac.verify_pipe_peer_proof(&challenge, std::process::id(), broker_pid, &proof) =>
        {
            PeerProof::Proven
        }
        Ok(Err(_)) => PeerProof::Closed,
        Ok(Ok(_)) => PeerProof::Wrong,
        Err(_) => PeerProof::TimedOut,
    }
}

/// The broker's side of the data-pipe peer challenge (#390), on a connection
/// already checked to come from `client_pid`.
pub(crate) async fn answer_peer_challenge(
    server: &mut NamedPipeServer,
    mac: &BrokerChannelMac,
    client_pid: u32,
) -> io::Result<()> {
    let mut challenge = [0_u8; PIPE_CHALLENGE_BYTES];
    tokio::time::timeout(PEER_PROOF_TIMEOUT, server.read_exact(&mut challenge))
        .await
        .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))??;
    let proof = mac.pipe_peer_proof(&challenge, client_pid, std::process::id());
    server.write_all(&proof).await
}

/// The process id of the controller that spawned the broker. Started by
/// `spawn_protected` (as Core does), Windows reports a process that held the
/// broker's stdout as its parent (#320), and the controller's id is in the
/// environment instead.
fn controller_pid() -> Option<u32> {
    codex_process_hardening::protected_spawner_pid().or_else(parent_pid)
}

/// The process id of this process's parent.
fn parent_pid() -> Option<u32> {
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQueryInformationProcess(
            process: HANDLE,
            class: u32,
            information: *mut c_void,
            length: u32,
            return_length: *mut u32,
        ) -> i32;
    }
    // PROCESS_BASIC_INFORMATION (x64/x86 layout of pointer-sized fields).
    #[repr(C)]
    struct BasicInformation {
        exit_status: i32,
        peb: usize,
        affinity: usize,
        priority: i32,
        pid: usize,
        parent_pid: usize,
    }
    // SAFETY: zeroed POD out-structure.
    let mut info: BasicInformation = unsafe { std::mem::zeroed() };
    let mut returned = 0_u32;
    // SAFETY: the pseudo-handle is valid; `info` matches the size passed.
    let status = unsafe {
        NtQueryInformationProcess(
            GetCurrentProcess(),
            /*class*/ 0,
            (&mut info as *mut BasicInformation).cast(),
            std::mem::size_of::<BasicInformation>() as u32,
            &mut returned,
        )
    };
    (status >= 0)
        .then(|| u32::try_from(info.parent_pid).ok())
        .flatten()
}

/// The controller process, held open from broker start so its process id
/// cannot be reused while the broker serves it.
pub(crate) struct ParentProcess {
    pub(crate) pid: u32,
    handle: isize,
}

impl ParentProcess {
    /// Opens the controller that spawned this process, refusing one created
    /// after this process (its id was reused after the controller exited).
    pub(crate) fn open() -> io::Result<Self> {
        let pid = controller_pid().ok_or_else(|| io::Error::other("no parent process"))?;
        // SAFETY: wait and query-limited access only; closed on drop.
        let handle = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                pid,
            )
        };
        if handle == 0 {
            return Err(io::Error::last_os_error());
        }
        let parent = Self { pid, handle };
        // SAFETY: the pseudo-handle is always valid.
        let own = creation_time(unsafe { GetCurrentProcess() })?;
        if creation_time(parent.handle)? > own {
            return Err(io::Error::other("parent process id was reused"));
        }
        Ok(parent)
    }

    /// Resolves once the controller exits.
    pub(crate) fn exited(self) -> tokio::task::JoinHandle<()> {
        let handle = self.handle;
        // The blocking task now owns the handle.
        std::mem::forget(self);
        tokio::task::spawn_blocking(move || {
            // SAFETY: a valid process handle owned by this task.
            unsafe {
                WaitForSingleObject(handle as HANDLE, INFINITE);
                CloseHandle(handle as HANDLE);
            }
        })
    }
}

impl Drop for ParentProcess {
    fn drop(&mut self) {
        // SAFETY: opened in `open` and closed once.
        unsafe { CloseHandle(self.handle as HANDLE) };
    }
}

fn creation_time(process: HANDLE) -> io::Result<u64> {
    let zero = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
    // SAFETY: valid out pointers; `process` has query-limited access.
    let ok = unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

/// A pipe end carrying rama connection extensions.
pub(crate) struct PipeStream<S> {
    stream: S,
    extensions: Extensions,
}

impl<S> PipeStream<S> {
    pub(crate) fn new(stream: S) -> Self {
        Self {
            stream,
            extensions: Extensions::new(),
        }
    }
}

impl<S> ExtensionsRef for PipeStream<S> {
    fn extensions(&self) -> &Extensions {
        &self.extensions
    }
}

impl<S> ExtensionsMut for PipeStream<S> {
    fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PipeStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PipeStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

/// rama transport that opens a fresh data-pipe connection per request.
#[derive(Clone, Debug)]
pub(crate) struct PipeConnector {
    name: Arc<str>,
    broker_pid: u32,
    /// Set when a connection failed because only another process served
    /// the pipe (#390); the HTTP client's error does not carry that.
    squatted: Arc<std::sync::atomic::AtomicBool>,
    /// The channel key the broker must prove it holds (#390).
    mac: Arc<BrokerChannelMac>,
}

impl PipeConnector {
    pub(crate) fn new(
        name: &str,
        broker_pid: u32,
        mac: Arc<BrokerChannelMac>,
        squatted: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            broker_pid,
            squatted,
            mac,
        }
    }
}

impl<Input: Send + 'static> Service<Input> for PipeConnector {
    type Output = EstablishedClientConnection<PipeStream<NamedPipeClient>, Input>;
    type Error = BoxError;

    async fn serve(&self, input: Input) -> Result<Self::Output, Self::Error> {
        let client = connect_data(&self.name, self.broker_pid, &self.mac)
            .await
            .inspect_err(|error| {
                if is_squatted_pipe_error(error) {
                    self.squatted
                        .store(true, std::sync::atomic::Ordering::Release);
                }
            })?;
        Ok(EstablishedClientConnection {
            input,
            conn: PipeStream::new(client),
        })
    }
}

#[cfg(test)]
#[path = "pipe_tests.rs"]
mod tests;
