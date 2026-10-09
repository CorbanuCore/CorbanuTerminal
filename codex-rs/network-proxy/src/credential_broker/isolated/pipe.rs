//! PF-27-S06: Windows named-pipe transport for the isolated credential broker.
//!
//! The broker creates both of its pipes (control and data):
//! - under a random name, as the first instance, so nothing can squat it or
//!   join it. The name is not a secret: pipe names can be listed;
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
//! service, not a disclosure). Because the broker runs at low integrity
//! (PF-27-S08), its pipe objects get a low integrity label, so low-integrity
//! processes of the user can now open them too (before, the implicit medium
//! label blocked them); the capability ACE admits only processes with equal
//! trust: the broker, its holder, and COM servers it launches under its own
//! token. Clients connect at identification-level impersonation, so a pipe
//! server cannot act with Core's token. The broker holds a handle to its
//! controller and exits with it, so the controller's process id cannot be
//! reused while the broker serves it.

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
use std::os::windows::io::OwnedHandle;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::net::windows::named_pipe::NamedPipeClient;
use tokio::net::windows::named_pipe::NamedPipeServer;
use tokio::net::windows::named_pipe::PipeMode;
use tokio::net::windows::named_pipe::ServerOptions;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_BROKEN_PIPE;
use windows_sys::Win32::Foundation::ERROR_IO_PENDING;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::SDDL_REVISION_1;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;
use windows_sys::Win32::Storage::FileSystem::ReadFile;
use windows_sys::Win32::Storage::FileSystem::WriteFile;
use windows_sys::Win32::System::IO::CancelIoEx;
use windows_sys::Win32::System::IO::GetOverlappedResult;
use windows_sys::Win32::System::IO::OVERLAPPED;
use windows_sys::Win32::System::Pipes::GetNamedPipeClientProcessId;
use windows_sys::Win32::System::Pipes::GetNamedPipeServerProcessId;
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
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const BUSY_RETRY: Duration = Duration::from_millis(10);

/// A fresh pair of pipe names: (control, data).
pub(crate) fn pipe_names() -> (String, String) {
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
    /// Creates the first instance; fails if the name already exists.
    pub(crate) fn bind(name: &str) -> io::Result<Self> {
        Ok(Self {
            name: name.to_string(),
            listening: create_instance(name, /*first*/ true)?,
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

/// Opens the broker's control pipe for blocking use, if `name` is a broker
/// control pipe served by process `broker_pid`.
pub(crate) fn connect_control(name: &str, broker_pid: u32) -> Option<ControlPipe> {
    if !valid_pipe_name(name, /*control*/ true) {
        return None;
    }
    let deadline = std::time::Instant::now() + CONNECT_TIMEOUT;
    let file = loop {
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(name)
        {
            Ok(file) => break file,
            Err(error)
                if error.raw_os_error() == Some(ERROR_PIPE_BUSY)
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(BUSY_RETRY);
            }
            Err(_) => return None,
        }
    };
    (server_pid(&file) == Some(broker_pid)).then(|| ControlPipe {
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
/// served by process `broker_pid`. Waits briefly while every instance is busy.
pub(crate) async fn connect_data(name: &str, broker_pid: u32) -> io::Result<NamedPipeClient> {
    if !valid_pipe_name(name, /*control*/ false) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let deadline = tokio::time::Instant::now() + CONNECT_TIMEOUT;
    let client = loop {
        match ClientOptions::new()
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(name)
        {
            Ok(client) => break client,
            Err(error)
                if error.raw_os_error() == Some(ERROR_PIPE_BUSY)
                    && tokio::time::Instant::now() < deadline =>
            {
                tokio::time::sleep(BUSY_RETRY).await;
            }
            Err(error) => return Err(error),
        }
    };
    if server_pid(&client) == Some(broker_pid) {
        Ok(client)
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "credential broker pipe is served by another process",
        ))
    }
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
}

impl PipeConnector {
    pub(crate) fn new(name: &str, broker_pid: u32) -> Self {
        Self {
            name: Arc::from(name),
            broker_pid,
        }
    }
}

impl<Input: Send + 'static> Service<Input> for PipeConnector {
    type Output = EstablishedClientConnection<PipeStream<NamedPipeClient>, Input>;
    type Error = BoxError;

    async fn serve(&self, input: Input) -> Result<Self::Output, Self::Error> {
        let client = connect_data(&self.name, self.broker_pid).await?;
        Ok(EstablishedClientConnection {
            input,
            conn: PipeStream::new(client),
        })
    }
}

#[cfg(test)]
#[path = "pipe_tests.rs"]
mod tests;
