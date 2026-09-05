//! Bounded native Linux IPC primitives, not a service installer or containment
//! claim. Trusted setup supplies an already connected socket and independently
//! authenticated expected peer. Same-user sockets alone cannot enable protection.

use crate::BrokerDispatchError;
use crate::ObservedPeer;
use crate::SignedBrokerFrame;
use crate::TypedOperationReceipt;
use crate::ipc::MAX_FRAME_BYTES;
use nix::poll::PollFd;
use nix::poll::PollFlags;
use nix::poll::poll;
use nix::sys::socket::getsockopt;
use nix::sys::socket::sockopt::PeerCredentials;
use serde::Deserialize;
use serde::Serialize;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

const MAX_RECEIPT_BYTES: usize = 512;
const IO_DEADLINE: Duration = Duration::from_secs(5);

/// Observe the actual connected process; wire payloads never supply identity.
pub fn observed_peer(stream: &UnixStream) -> Result<ObservedPeer, BrokerDispatchError> {
    let credentials = getsockopt(stream, PeerCredentials).map_err(unavailable)?;
    let pid = u32::try_from(credentials.pid()).map_err(unavailable)?;
    ObservedPeer::from_os(format!("uid:{}", credentials.uid()), pid)
        .map_err(BrokerDispatchError::from)
}

/// The service adapter owns the registered session and must cancel it on any
/// channel exit. `dispatch` must apply the runtime's generation/revocation fence.
pub trait LinuxBrokerHandler: Sync {
    fn dispatch(
        &self,
        peer: &ObservedPeer,
        frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError>;
    fn close(&self);
}

/// Binds a service connection to one existing runtime registration. The handle
/// is never deserialized from the client, and EOF cancels that exact session.
pub struct LinuxBrokerSession<B, A> {
    runtime: Arc<crate::BrokerRuntime<B, A>>,
    handle: crate::BrokerSessionHandle,
}

impl<B, A> LinuxBrokerSession<B, A> {
    pub fn new(
        runtime: Arc<crate::BrokerRuntime<B, A>>,
        handle: crate::BrokerSessionHandle,
    ) -> Self {
        Self { runtime, handle }
    }
}

impl<B: crate::TypedCredentialBackend, A: crate::DurableBrokerAudit> LinuxBrokerHandler
    for LinuxBrokerSession<B, A>
{
    fn dispatch(
        &self,
        peer: &ObservedPeer,
        frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        self.runtime.dispatch(&self.handle, peer, frame)
    }

    fn close(&self) {
        let _ = self.runtime.cancel_session(&self.handle);
    }
}

struct CloseGuard<'a, H: LinuxBrokerHandler> {
    handler: &'a H,
    closed: AtomicBool,
}
impl<H: LinuxBrokerHandler> CloseGuard<'_, H> {
    fn close(&self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            self.handler.close();
        }
    }
}
impl<H: LinuxBrokerHandler> Drop for CloseGuard<'_, H> {
    fn drop(&mut self) {
        self.close();
    }
}

/// Serve one bounded frame per iteration. EOF, wrong peer, malformed framing,
/// timeout and handler rejection all close this registered generation.
pub fn serve_connection<H: LinuxBrokerHandler>(
    mut stream: UnixStream,
    expected_peer: &ObservedPeer,
    handler: &H,
) -> Result<(), BrokerDispatchError> {
    let guard = CloseGuard {
        handler,
        closed: AtomicBool::new(false),
    };
    configure(&stream)?;
    let peer = observed_peer(&stream)?;
    if &peer != expected_peer {
        return Err(BrokerDispatchError::WrongPeer);
    }
    let monitor = stream.try_clone().map_err(unavailable)?;
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("broker-disconnect".into())
            .spawn_scoped(scope, || watch_disconnect(&monitor, &guard))
            .map_err(unavailable)?;
        let result = serve_frames(&mut stream, &peer, handler);
        guard.close();
        result
    })
}

fn serve_frames<H: LinuxBrokerHandler>(
    stream: &mut UnixStream,
    peer: &ObservedPeer,
    handler: &H,
) -> Result<(), BrokerDispatchError> {
    loop {
        let frame = read_frame(stream)?;
        match handler.dispatch(peer, &frame) {
            Ok(receipt) => write_receipt(stream, &WireReceipt::from(receipt))?,
            Err(error) => return Err(error),
        }
    }
}

fn watch_disconnect<H: LinuxBrokerHandler>(stream: &UnixStream, guard: &CloseGuard<'_, H>) {
    // Nix 0.30 does not name Linux POLLRDHUP. Retain the platform bit when
    // requesting half-close notification; an unknown returned flag is treated
    // as closure, never healthy. Do not consume queued authenticated frames.
    let events = PollFlags::from_bits_retain(nix::libc::POLLRDHUP);
    let mut fds = [PollFd::new(stream.as_fd(), events)];
    while !guard.closed.load(Ordering::Acquire) {
        match poll(&mut fds, 100_u16) {
            Ok(0) => {}
            Ok(_) | Err(_) => {
                guard.close();
                let _ = stream.shutdown(Shutdown::Both);
                return;
            }
        }
    }
}

/// One peer-verified channel. Any ambiguous I/O permanently closes it, never
/// reconnecting or replaying a possibly consumed frame on a new service boot.
pub struct LinuxBrokerChannel {
    state: Mutex<Option<UnixStream>>,
    shutdown: UnixStream,
    closed: AtomicBool,
}

impl LinuxBrokerChannel {
    pub fn new(
        stream: UnixStream,
        expected_server: &ObservedPeer,
    ) -> Result<Self, BrokerDispatchError> {
        configure(&stream)?;
        if &observed_peer(&stream)? != expected_server {
            return Err(BrokerDispatchError::WrongPeer);
        }
        let shutdown = stream.try_clone().map_err(unavailable)?;
        Ok(Self {
            state: Mutex::new(Some(stream)),
            shutdown,
            closed: AtomicBool::new(false),
        })
    }

    pub fn dispatch(
        &self,
        frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        let mut state = self.state.lock().map_err(unavailable)?;
        if self.closed.load(Ordering::Acquire) {
            return Err(BrokerDispatchError::SessionUnavailable);
        }
        let stream = state
            .as_mut()
            .ok_or(BrokerDispatchError::SessionUnavailable)?;
        let result = stream
            .write_all(frame.as_bytes())
            .map_err(|_| BrokerDispatchError::OutcomeUnknown)
            .and_then(|()| read_receipt(stream));
        let result = if self.closed.load(Ordering::Acquire) {
            Err(BrokerDispatchError::OutcomeUnknown)
        } else {
            result
        };
        if result.is_err() {
            self.closed.store(true, Ordering::Release);
            let _ = stream.shutdown(Shutdown::Both);
            *state = None;
        }
        result
    }

    pub fn close(&self) -> Result<(), BrokerDispatchError> {
        // Never wait for the dispatch mutex: shutting down the duplicate fd
        // interrupts an in-flight read/write and rejects queued callers.
        self.closed.store(true, Ordering::Release);
        let _ = self.shutdown.shutdown(Shutdown::Both);
        Ok(())
    }
}

impl Drop for LinuxBrokerChannel {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireReceipt {
    response_status: u16,
    uploaded_bytes: u64,
    downloaded_bytes: u64,
}

impl From<TypedOperationReceipt> for WireReceipt {
    fn from(receipt: TypedOperationReceipt) -> Self {
        Self {
            response_status: receipt.response_status,
            uploaded_bytes: receipt.uploaded_bytes,
            downloaded_bytes: receipt.downloaded_bytes,
        }
    }
}

fn read_frame(stream: &mut UnixStream) -> Result<SignedBrokerFrame, BrokerDispatchError> {
    let mut prefix = [0; 4];
    stream.read_exact(&mut prefix).map_err(unavailable)?;
    let length = usize::try_from(u32::from_be_bytes(prefix)).map_err(unavailable)?;
    // Frame includes a four-byte prefix and 32-byte authenticator. Check before allocation.
    if length > MAX_FRAME_BYTES - 36 {
        return Err(BrokerDispatchError::Frame(
            crate::ipc::BrokerFrameError::FrameTooLarge,
        ));
    }
    let mut bytes = Vec::with_capacity(length + 36);
    bytes.extend_from_slice(&prefix);
    bytes.resize(length + 36, 0);
    stream.read_exact(&mut bytes[4..]).map_err(unavailable)?;
    SignedBrokerFrame::from_bytes(bytes).map_err(BrokerDispatchError::from)
}

fn read_receipt(stream: &mut UnixStream) -> Result<TypedOperationReceipt, BrokerDispatchError> {
    let mut prefix = [0; 4];
    stream
        .read_exact(&mut prefix)
        .map_err(|_| BrokerDispatchError::OutcomeUnknown)?;
    let length = usize::try_from(u32::from_be_bytes(prefix)).map_err(unavailable)?;
    if length > MAX_RECEIPT_BYTES {
        return Err(BrokerDispatchError::OutcomeUnknown);
    }
    let mut bytes = vec![0; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|_| BrokerDispatchError::OutcomeUnknown)?;
    let receipt: WireReceipt =
        serde_json::from_slice(&bytes).map_err(|_| BrokerDispatchError::OutcomeUnknown)?;
    if !(100..=599).contains(&receipt.response_status) {
        return Err(BrokerDispatchError::OutcomeUnknown);
    }
    Ok(TypedOperationReceipt {
        response_status: receipt.response_status,
        uploaded_bytes: receipt.uploaded_bytes,
        downloaded_bytes: receipt.downloaded_bytes,
    })
}

fn write_receipt(
    stream: &mut UnixStream,
    receipt: &WireReceipt,
) -> Result<(), BrokerDispatchError> {
    let bytes = serde_json::to_vec(receipt).map_err(unavailable)?;
    let length = u32::try_from(bytes.len()).map_err(unavailable)?;
    stream
        .write_all(&length.to_be_bytes())
        .map_err(unavailable)?;
    stream.write_all(&bytes).map_err(unavailable)
}

fn configure(stream: &UnixStream) -> Result<(), BrokerDispatchError> {
    stream
        .set_read_timeout(Some(IO_DEADLINE))
        .map_err(unavailable)?;
    stream
        .set_write_timeout(Some(IO_DEADLINE))
        .map_err(unavailable)
}

fn unavailable<T>(_: T) -> BrokerDispatchError {
    BrokerDispatchError::SessionUnavailable
}

#[cfg(test)]
#[path = "linux_transport_tests.rs"]
mod tests;
