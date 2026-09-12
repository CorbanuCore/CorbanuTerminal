//! Construction seam only: no executable qualification, principal isolation,
//! native root enrollment or protected-mode authorization is established here.
use codex_secret_broker::linux_transport::observed_peer;
use nix::poll::PollFd;
use nix::poll::PollFlags;
use nix::poll::PollTimeout;
use nix::poll::poll;
use nix::sys::socket::getsockopt;
use nix::sys::socket::sockopt::PeerPidfd;
use rustix::process::Pid;
use rustix::process::PidfdFlags;
use rustix::process::Signal;
use rustix::process::pidfd_open;
use rustix::process::pidfd_send_signal;
use std::io;
use std::net::Shutdown;
use std::num::NonZeroU64;
use std::os::fd::AsFd;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread::JoinHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildRole {
    Journal,
    Policy,
}

struct Slot {
    role: ChildRole,
    uid: u32,
    pid: u32,
    pidfd: OwnedFd,
    child: Option<Child>,
    channel: Option<UnixStream>,
}

struct Control {
    fenced: bool,
    slots: Vec<Slot>,
}

impl Control {
    fn fence(&mut self) {
        self.fenced = true;
        for slot in &self.slots {
            if let Some(channel) = &slot.channel {
                let _ = channel.shutdown(Shutdown::Both);
            }
            // Stable kernel handles only; never signal a recycled numeric PID.
            let _ = pidfd_send_signal(&slot.pidfd, Signal::KILL);
        }
    }

    fn check(&mut self) -> io::Result<()> {
        if self.fenced {
            return Err(unavailable());
        }
        let mut fds: Vec<_> = self
            .slots
            .iter()
            .map(|slot| PollFd::new(slot.pidfd.as_fd(), PollFlags::POLLIN))
            .collect();
        // Any poll failure also fences. In particular, EINTR must not turn a
        // failed liveness observation into a successful admission.
        if !matches!(poll(&mut fds, PollTimeout::ZERO), Ok(0)) {
            self.fence();
            return Err(unavailable());
        }
        Ok(())
    }
}

fn unavailable() -> io::Error {
    io::Error::other("trusted child generation unavailable")
}

fn start_reaper(receiver: mpsc::Receiver<Child>, reaped: Arc<AtomicUsize>) -> io::Result<()> {
    std::thread::Builder::new()
        .name("corbanu-child-reaper".into())
        .spawn(move || {
            while let Ok(mut child) = receiver.recv() {
                if child.wait().is_ok() {
                    reaped.fetch_add(1, Ordering::Release);
                }
            }
        })?;
    Ok(())
}

/// Owns exactly two independently spawned children for one immutable generation.
/// The trusted caller supplies expected UIDs and commands, never a worker frame.
/// `poll_health` must be driven by the supervising event loop; this object does
/// not mint authority or provide a background death/idle monitor. The native
/// handler must enforce its own absolute frame/idle deadlines (PF20: ten seconds).
pub struct TrustedChildRun {
    generation: NonZeroU64,
    control: Arc<Mutex<Control>>,
    handlers: Vec<JoinHandle<()>>,
    cleanup: mpsc::Sender<Child>,
    reaped: Arc<AtomicUsize>,
}

impl TrustedChildRun {
    /// On capture failure returns both children to the trusted launcher for
    /// cleanup. No child is silently detached or adopted by numeric PID.
    pub fn capture(
        generation: NonZeroU64,
        journal: Child,
        journal_uid: u32,
        policy: Child,
        policy_uid: u32,
    ) -> Result<Self, (io::Error, Child, Child)> {
        Self::capture_with_reaper(
            generation,
            journal,
            journal_uid,
            policy,
            policy_uid,
            start_reaper,
        )
    }

    fn capture_with_reaper(
        generation: NonZeroU64,
        mut journal: Child,
        journal_uid: u32,
        mut policy: Child,
        policy_uid: u32,
        spawn: fn(mpsc::Receiver<Child>, Arc<AtomicUsize>) -> io::Result<()>,
    ) -> Result<Self, (io::Error, Child, Child)> {
        let capture = |child: &mut Child| -> io::Result<OwnedFd> {
            if child.try_wait()?.is_some() {
                return Err(unavailable());
            }
            let pid = Pid::from_raw(i32::try_from(child.id()).map_err(|_| unavailable())?)
                .ok_or_else(unavailable)?;
            let fd = pidfd_open(pid, PidfdFlags::empty())?;
            if child.try_wait()?.is_some() {
                return Err(unavailable());
            }
            Ok(fd)
        };
        let fds = capture(&mut journal).and_then(|a| capture(&mut policy).map(|b| (a, b)));
        let (journal_fd, policy_fd) = match fds {
            Ok(fds) => fds,
            Err(error) => return Err((error, journal, policy)),
        };
        // Reserve cleanup capacity before accepting ownership. Thread creation
        // failure returns both children; Drop and admission failure never need
        // to create a replacement thread under resource pressure.
        let (cleanup, receiver) = mpsc::channel();
        let reaped = Arc::new(AtomicUsize::new(0));
        if let Err(error) = spawn(receiver, Arc::clone(&reaped)) {
            return Err((error, journal, policy));
        }
        let slot = |role, uid, pidfd, child: Child| Slot {
            role,
            uid,
            pid: child.id(),
            pidfd,
            child: Some(child),
            channel: None,
        };
        Ok(Self {
            generation,
            control: Arc::new(Mutex::new(Control {
                fenced: false,
                slots: vec![
                    slot(ChildRole::Journal, journal_uid, journal_fd, journal),
                    slot(ChildRole::Policy, policy_uid, policy_fd, policy),
                ],
            })),
            handlers: Vec::with_capacity(2),
            cleanup,
            reaped,
        })
    }

    pub fn generation(&self) -> NonZeroU64 {
        self.generation
    }

    pub fn poll_health(&self) -> io::Result<()> {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        control.check()
    }

    /// Admit once by kernel PID/UID, before invoking the trusted root handler.
    /// No bytes from the channel influence role selection. The handler receives
    /// the original Child for PF20's independent live-child/pidfd validation.
    /// It runs outside the admission lock so a stalled handler cannot block
    /// other admission, health checks or shutdown. At most two handlers exist.
    pub fn admit<F>(&mut self, stream: UnixStream, handler: F) -> io::Result<ChildRole>
    where
        F: FnOnce(ChildRole, UnixStream, &mut Child) -> io::Result<()> + Send + 'static,
    {
        let peer = observed_peer(&stream).map_err(|_| unavailable())?;
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        control.check()?;
        // SO_PEERCRED alone preserves a reusable numeric PID. Require a live
        // socket-associated pidfd before matching it to our still-unreaped
        // Child. An old retained socket cannot consume a replacement slot.
        // Unsupported kernels fail closed; there is no numeric-only fallback.
        let socket_pidfd = getsockopt(&stream, PeerPidfd)?;
        let mut peer_fds = [PollFd::new(socket_pidfd.as_fd(), PollFlags::POLLIN)];
        if !matches!(poll(&mut peer_fds, PollTimeout::ZERO), Ok(0)) {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "socket peer is no longer alive",
            ));
        }
        let slot = control
            .slots
            .iter_mut()
            .find(|slot| {
                slot.pid == peer.process_id() && peer.principal() == format!("uid:{}", slot.uid)
            })
            .ok_or_else(unavailable)?;
        if slot.channel.is_some() {
            return Err(unavailable());
        }
        // Clone before consuming the slot: a descriptor failure is not admission.
        let cancellation = stream.try_clone()?;
        let child = slot.child.take().ok_or_else(unavailable)?;
        let role = slot.role;
        slot.channel = Some(cancellation);
        let task = Arc::new(Mutex::new(Some((child, stream, handler))));
        let worker_task = Arc::clone(&task);
        let shared = Arc::clone(&self.control);
        let cleanup = self.cleanup.clone();
        let thread = std::thread::Builder::new()
            .name("corbanu-trusted-child".into())
            .spawn(move || {
                let Some((mut child, stream, handler)) = worker_task
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                else {
                    return;
                };
                // Handler panic is also a terminal generation failure. The
                // handler is trusted code, not a sandbox for arbitrary plugins.
                if shared
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .check()
                    .is_ok()
                {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handler(role, stream, &mut child)
                    }));
                }
                shared
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .fence();
                if let Err(mpsc::SendError(mut child)) = cleanup.send(child) {
                    // Only an unexpected reaper failure can disconnect while
                    // this sender is alive. Preserve ownership on this worker.
                    let _ = child.wait();
                }
            });
        match thread {
            Ok(thread) => self.handlers.push(thread),
            Err(error) => {
                if let Some((child, _, _)) = task
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                {
                    slot.child = Some(child);
                }
                control.fence();
                return Err(error);
            }
        }
        Ok(role)
    }

    /// Fence immediately and signal only retained pidfds. No blocking joins.
    /// Returns true only when handlers have finished and all owned children
    /// have been reaped. Retain the run and poll this during bounded shutdown;
    /// a false result is unfinished cleanup, never a successful stop report.
    pub fn shutdown(&mut self) -> bool {
        let mut control = self
            .control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        control.fence();
        for slot in &mut control.slots {
            if let Some(child) = slot.child.take()
                && let Err(mpsc::SendError(child)) = self.cleanup.send(child)
            {
                slot.child = Some(child);
            }
        }
        self.reaped.load(Ordering::Acquire) == 2
            && self.handlers.iter().all(JoinHandle::is_finished)
    }
}

impl Drop for TrustedChildRun {
    fn drop(&mut self) {
        // The reaper already exists. Early drop needs no thread creation and
        // leaves it owning at most two children until kernel termination.
        self.shutdown();
    }
}

#[cfg(test)]
#[path = "children_unit_tests.rs"]
mod tests;
