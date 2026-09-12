//! Bounded synthetic admission on the existing pair's sole worker.
use super::super::spawn::Status;
use super::*;
use codex_linux_pidfd_spawn::OwnedChild;
use nix::poll::PollFd;
use nix::poll::PollFlags;
use nix::poll::PollTimeout;
use nix::poll::poll;
use nix::sys::socket::MsgFlags;
use nix::sys::socket::getsockopt;
use nix::sys::socket::recv;
use nix::sys::socket::sockopt;
use std::net::Shutdown;
use std::num::NonZeroU64;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;

/// Identity from the retained kernel owner; test doubles cannot enter production.
pub(in super::super) trait Process: Child {
    fn matches(&self, peer: BorrowedFd<'_>) -> io::Result<bool>;
}
impl Process for OwnedChild {
    fn matches(&self, peer: BorrowedFd<'_>) -> io::Result<bool> {
        self.is_same_process(peer)
    }
}
pub(in super::super) struct AdmissionSpec {
    pub(in super::super) generation: NonZeroU64,
    pub(in super::super) expected_uid: u32,
}
struct Request {
    stream: UnixStream,
    reply: mpsc::SyncSender<io::Result<AdmittedPeer>>,
}
pub(in super::super) struct PairHandle {
    owner: LaunchHandle,
    requests: mpsc::SyncSender<Request>,
}
pub(in super::super) struct AdmissionTicket {
    reply: mpsc::Receiver<io::Result<AdmittedPeer>>,
    control: Option<Arc<Shared>>,
}
pub(in super::super) struct AdmittedPeer {
    role: SyntheticChildRole,
    generation: NonZeroU64,
    stream: UnixStream,
    control: Arc<Shared>,
}
impl AdmittedPeer {
    pub(in super::super) fn role(&self) -> SyntheticChildRole {
        self.role
    }
    pub(in super::super) fn generation(&self) -> NonZeroU64 {
        self.generation
    }
    pub(in super::super) fn stream(&self) -> &UnixStream {
        &self.stream
    }
}
impl Drop for AdmittedPeer {
    fn drop(&mut self) {
        self.control.cancel();
    }
}
impl AdmissionTicket {
    pub(in super::super) fn take_result(&mut self) -> Option<io::Result<AdmittedPeer>> {
        let result = match self.reply.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => Err(io::ErrorKind::ConnectionAborted.into()),
        };
        let cancelled = self
            .control
            .take()
            .is_none_or(|control| control.cancelled());
        Some(if cancelled {
            Err(io::ErrorKind::ConnectionAborted.into())
        } else {
            result
        })
    }
}
impl Drop for AdmissionTicket {
    fn drop(&mut self) {
        if let Some(control) = &self.control {
            control.cancel();
        }
    }
}
impl PairHandle {
    pub(in super::super) fn status(&self) -> Status {
        self.owner.status()
    }
    pub(in super::super) fn cancel(&self) {
        self.owner.cancel();
    }
    pub(in super::super) fn admit(
        &self,
        stream: UnixStream,
    ) -> Result<AdmissionTicket, (io::Error, UnixStream)> {
        let control = self.owner.control();
        if control.cancelled() {
            return Err((io::ErrorKind::ConnectionAborted.into(), stream));
        }
        let (reply, receiver) = mpsc::sync_channel(1);
        match self.requests.try_send(Request { stream, reply }) {
            Ok(()) => Ok(AdmissionTicket {
                reply: receiver,
                control: Some(control),
            }),
            Err(mpsc::TrySendError::Full(request)) => {
                Err((io::ErrorKind::WouldBlock.into(), request.stream))
            }
            Err(mpsc::TrySendError::Disconnected(request)) => {
                Err((io::ErrorKind::ConnectionAborted.into(), request.stream))
            }
        }
    }
}
impl Reservation {
    pub(in super::super) fn launch_admitting_pair(
        self,
        images: PairImages,
        spec: AdmissionSpec,
        deadline: Instant,
    ) -> Result<PairHandle, (io::Error, PairImages)> {
        if spec.expected_uid == 0 || spec.expected_uid != nix::unistd::getuid().as_raw() {
            return Err((io::ErrorKind::InvalidInput.into(), images));
        }
        let (requests, receiver) = mpsc::sync_channel(2);
        let admission = Admission {
            spec,
            requests: receiver,
            channels: [None, None],
        };
        let backend = PairBackend(
            [
                Kernel(SyntheticChildRole::Journal),
                Kernel(SyntheticChildRole::Policy),
            ],
            Some(admission),
        );
        self.launch_with(images, backend, deadline, start_worker)
            .map(|owner| PairHandle { owner, requests })
    }
}
pub(in super::super) struct Admission {
    spec: AdmissionSpec,
    requests: mpsc::Receiver<Request>,
    channels: [Option<UnixStream>; 2],
}
fn live<C: Process>(children: &mut [Option<C>; 2], control: &Shared) -> io::Result<()> {
    for child in children {
        if child
            .as_mut()
            .is_none_or(|child| !matches!(child.exited(), Ok(false)))
        {
            control.cancel();
        }
    }
    if control.cancelled() {
        Err(io::ErrorKind::ConnectionAborted.into())
    } else {
        Ok(())
    }
}
fn live_peer(peer: BorrowedFd<'_>) -> io::Result<()> {
    let mut fds = [PollFd::new(peer, PollFlags::POLLIN)];
    if poll(&mut fds, PollTimeout::ZERO)? != 0 {
        return Err(io::ErrorKind::ConnectionAborted.into());
    }
    Ok(())
}
impl Admission {
    fn admit<C: Process>(
        &mut self,
        stream: UnixStream,
        children: &mut [Option<C>; 2],
        control: &Arc<Shared>,
    ) -> io::Result<AdmittedPeer> {
        live(children, control)?;
        if getsockopt(&stream, sockopt::PeerCredentials)?.uid() != self.spec.expected_uid {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let peer = getsockopt(&stream, sockopt::PeerPidfd)?;
        live_peer(peer.as_fd())?;
        let mut index = None;
        for (i, child) in children.iter().enumerate() {
            if let Some(child) = child
                && child.matches(peer.as_fd())?
                && index.replace(i).is_some()
            {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        let index = index.ok_or(io::ErrorKind::PermissionDenied)?;
        if self.channels[index].is_some() {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let guard = stream.try_clone()?;
        live(children, control)?;
        live_peer(peer.as_fd())?;
        self.channels[index] = Some(guard);
        Ok(AdmittedPeer {
            role: [SyntheticChildRole::Journal, SyntheticChildRole::Policy][index],
            generation: self.spec.generation,
            stream,
            control: Arc::clone(control),
        })
    }
    pub(in super::super) fn close(&self) {
        for stream in self.channels.iter().flatten() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
    pub(in super::super) fn tick<C: Process>(
        &mut self,
        children: &mut [Option<C>; 2],
        control: &Arc<Shared>,
    ) {
        for _ in 0..2 {
            if control.cancelled() {
                break;
            }
            let Ok(request) = self.requests.try_recv() else {
                break;
            };
            let result = self.admit(request.stream, children, control);
            if control.cancelled() || request.reply.try_send(result).is_err() {
                control.cancel();
            }
        }
        for stream in self.channels.iter().flatten() {
            match recv(
                stream.as_raw_fd(),
                &mut [0u8; 1],
                MsgFlags::MSG_PEEK | MsgFlags::MSG_DONTWAIT,
            ) {
                Ok(0) => control.cancel(),
                Ok(_) | Err(nix::errno::Errno::EAGAIN | nix::errno::Errno::EINTR) => {}
                Err(_) => control.cancel(),
            }
        }
        if control.cancelled() {
            self.close();
        }
    }
}
#[cfg(test)]
#[path = "pair_admission_tests.rs"]
mod tests;
