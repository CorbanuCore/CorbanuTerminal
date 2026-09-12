//! Private synthetic admission receipts, not PF20 authority or native activation.
use super::super::SyntheticChildRole;
use super::sealed::SyntheticProfileInspectedImage;
use super::spawn::Backend;
use super::spawn::Child;
use super::spawn::LaunchHandle;
use super::spawn::Reservation;
use super::spawn::Shared;
use super::spawn::Status;
use super::spawn::start_worker;
use codex_linux_pidfd_spawn::OwnedChild;
use nix::poll::PollFd;
use nix::poll::PollFlags;
use nix::poll::PollTimeout;
use nix::poll::poll;
use nix::sys::socket::MsgFlags;
use nix::sys::socket::getsockopt;
use nix::sys::socket::recv;
use nix::sys::socket::sockopt;
use std::io;
use std::net::Shutdown;
use std::num::NonZeroU64;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Instant;

pub(super) struct PairImages<I = SyntheticProfileInspectedImage> {
    pub(super) journal: I,
    pub(super) policy: I,
}
struct Spec {
    generation: NonZeroU64,
    uid: u32,
}
struct Request {
    stream: UnixStream,
    reply: mpsc::SyncSender<io::Result<AdmittedPeer>>,
}
pub(super) struct PairHandle {
    owner: LaunchHandle,
    requests: mpsc::SyncSender<Request>,
}
pub(super) struct AdmissionTicket {
    reply: mpsc::Receiver<io::Result<AdmittedPeer>>,
    guard: Option<Arc<Shared>>,
}
pub(super) struct AdmittedPeer {
    role: SyntheticChildRole,
    generation: NonZeroU64,
    stream: UnixStream,
    control: Arc<Shared>,
}
impl AdmittedPeer {
    pub(super) fn role(&self) -> SyntheticChildRole {
        self.role
    }
    pub(super) fn generation(&self) -> NonZeroU64 {
        self.generation
    }
    pub(super) fn stream(&self) -> &UnixStream {
        &self.stream
    }
}
impl Drop for AdmittedPeer {
    fn drop(&mut self) {
        self.control.cancel();
    }
}
impl AdmissionTicket {
    pub(super) fn take_result(&mut self) -> Option<io::Result<AdmittedPeer>> {
        let result = match self.reply.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => Err(io::ErrorKind::ConnectionAborted.into()),
        };
        self.guard.take();
        Some(result)
    }
}
impl Drop for AdmissionTicket {
    fn drop(&mut self) {
        if let Some(control) = &self.guard {
            control.cancel();
        }
    }
}
impl PairHandle {
    pub(super) fn status(&self) -> Status {
        self.owner.status()
    }
    pub(super) fn cancel(&self) {
        self.owner.cancel();
    }
    pub(super) fn admit(
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
                guard: Some(control),
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
    pub(super) fn launch_pair(
        self,
        images: PairImages,
        generation: NonZeroU64,
        expected_uid: u32,
        deadline: Instant,
    ) -> Result<PairHandle, (io::Error, PairImages)> {
        if expected_uid == 0 || expected_uid != nix::unistd::getuid().as_raw() {
            return Err((io::ErrorKind::InvalidInput.into(), images));
        }
        launch_pair_with(
            self,
            images,
            [
                Kernel(SyntheticChildRole::Journal),
                Kernel(SyntheticChildRole::Policy),
            ],
            Spec {
                generation,
                uid: expected_uid,
            },
            deadline,
            start_worker,
        )
    }
}
fn launch_pair_with<B: Backend>(
    reservation: Reservation,
    images: PairImages<B::Image>,
    backends: [B; 2],
    spec: Spec,
    deadline: Instant,
    start: impl FnOnce(Box<dyn FnOnce() + Send>) -> io::Result<()>,
) -> Result<PairHandle, (io::Error, PairImages<B::Image>)>
where
    B::Child: Process,
{
    let (requests, receiver) = mpsc::sync_channel(2);
    let backend = PairBackend {
        backends,
        spec,
        requests: receiver,
    };
    let owner = reservation.launch_with(images, backend, deadline, start)?;
    Ok(PairHandle { owner, requests })
}
/// Trusted private process seam: a kernel owner or deterministic lifecycle test.
trait Process: Child {
    fn matches(&self, peer: BorrowedFd<'_>) -> io::Result<bool>;
}
impl Process for OwnedChild {
    fn matches(&self, peer: BorrowedFd<'_>) -> io::Result<bool> {
        self.is_same_process(peer)
    }
}
struct Kernel(SyntheticChildRole);
impl Backend for Kernel {
    type Image = SyntheticProfileInspectedImage;
    type Child = OwnedChild;
    fn spawn(self, image: Self::Image, _control: &Arc<Shared>) -> io::Result<OwnedChild> {
        image.launch_owned(self.0)
    }
}
struct PairBackend<B> {
    backends: [B; 2],
    spec: Spec,
    requests: mpsc::Receiver<Request>,
}
struct PairChild<C: Process> {
    children: [Option<C>; 2],
    channels: [Option<UnixStream>; 2],
    spec: Spec,
    requests: mpsc::Receiver<Request>,
    control: Arc<Shared>,
}
impl<B: Backend> Backend for PairBackend<B>
where
    B::Child: Process,
{
    type Image = PairImages<B::Image>;
    type Child = PairChild<B::Child>;
    fn spawn(self, images: Self::Image, control: &Arc<Shared>) -> io::Result<Self::Child> {
        let [journal, policy] = self.backends;
        let first = journal.spawn(images.journal, control)?;
        let mut pair = PairChild {
            children: [Some(first), None],
            channels: [None, None],
            spec: self.spec,
            requests: self.requests,
            control: Arc::clone(control),
        };
        if !control.cancelled() {
            match policy.spawn(images.policy, control) {
                Ok(child) => pair.children[1] = Some(child),
                Err(_) => control.cancel(),
            }
        }
        // Even second-spawn failure returns the first child's owner to the
        // supervisor. Rejection must never release a live first child/permit.
        Ok(pair)
    }
}
impl<C: Process> PairChild<C> {
    fn poll_children(&mut self) -> io::Result<bool> {
        let mut all = true;
        let mut error = None;
        for child in self.children.iter_mut().flatten() {
            match child.exited() {
                Ok(true) => self.control.cancel(),
                Ok(false) => all = false,
                Err(err) => {
                    error = Some(err);
                    all = false;
                    self.control.cancel();
                }
            }
        }
        match error {
            Some(error) => Err(error),
            None => Ok(all),
        }
    }
    fn admit(&mut self, stream: UnixStream) -> io::Result<AdmittedPeer> {
        self.poll_children()?;
        if self.control.cancelled() {
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        if getsockopt(&stream, sockopt::PeerCredentials)?.uid() != self.spec.uid {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let peer = getsockopt(&stream, sockopt::PeerPidfd)?;
        let mut pollfd = [PollFd::new(peer.as_fd(), PollFlags::POLLIN)];
        if poll(&mut pollfd, PollTimeout::ZERO)? != 0 {
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        let mut matched = None;
        for (index, child) in self.children.iter().enumerate() {
            if let Some(child) = child
                && child.matches(peer.as_fd())?
            {
                if matched.replace(index).is_some() {
                    return Err(io::ErrorKind::PermissionDenied.into());
                }
            }
        }
        let index = matched.ok_or(io::ErrorKind::PermissionDenied)?;
        if self.channels[index].is_some() {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let cancellation = stream.try_clone()?;
        self.poll_children()?;
        if self.control.cancelled() || poll(&mut pollfd, PollTimeout::ZERO)? != 0 {
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        self.channels[index] = Some(cancellation);
        Ok(AdmittedPeer {
            role: [SyntheticChildRole::Journal, SyntheticChildRole::Policy][index],
            generation: self.spec.generation,
            stream,
            control: Arc::clone(&self.control),
        })
    }
    fn close_channels(&self) {
        for stream in self.channels.iter().flatten() {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}
impl<C: Process> Child for PairChild<C> {
    fn stop(&self) -> io::Result<()> {
        self.control.cancel();
        self.close_channels();
        let mut error = None;
        for child in self.children.iter().flatten() {
            if let Err(err) = child.stop() {
                error = Some(err);
            }
        }
        error.map_or(Ok(()), Err)
    }
    fn exited(&mut self) -> io::Result<bool> {
        let all = self.poll_children()?;
        if !self.control.cancelled() {
            // Bound work per poll even if callers refill the queue concurrently.
            for _ in 0..2 {
                let Ok(request) = self.requests.try_recv() else {
                    break;
                };
                let result = self.admit(request.stream);
                if request.reply.try_send(result).is_err() {
                    self.control.cancel();
                }
                if self.control.cancelled() {
                    break;
                }
            }
            for stream in self.channels.iter().flatten() {
                match recv(
                    stream.as_raw_fd(),
                    &mut [0u8; 1],
                    MsgFlags::MSG_PEEK | MsgFlags::MSG_DONTWAIT,
                ) {
                    Ok(0) => self.control.cancel(),
                    Ok(_) | Err(nix::errno::Errno::EAGAIN | nix::errno::Errno::EINTR) => {}
                    Err(_) => self.control.cancel(),
                }
            }
        }
        if self.control.cancelled() {
            self.close_channels();
        }
        Ok(all)
    }
}
impl<C: Process> Drop for PairChild<C> {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
#[path = "pair_tests.rs"]
mod tests;
