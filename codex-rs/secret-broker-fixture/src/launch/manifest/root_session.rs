//! Bounded private rendezvous pump; listeners never confer child authority.
use super::AdmissionTicket;
use super::RootDispatch;
use super::RootError;
use super::Status;
use std::io;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;

pub(super) struct RootSession {
    run: RootDispatch,
    listeners: Option<[UnixListener; 2]>,
    pending: [Option<AdmissionTicket>; 2],
    next: usize,
    failure: Option<RootError>,
}
impl RootSession {
    pub(super) fn new(run: RootDispatch, listeners: [UnixListener; 2]) -> io::Result<Self> {
        for listener in &listeners {
            listener.set_nonblocking(true)?;
        }
        // Constructor failure drops run and therefore fences the pair.
        Ok(Self {
            run,
            listeners: Some(listeners),
            pending: [None, None],
            next: 0,
            failure: None,
        })
    }
    pub(super) fn poll(&mut self) -> bool {
        self.poll_with(|listener| listener.accept().map(|(stream, _)| stream))
    }
    fn poll_with(
        &mut self,
        mut accept: impl FnMut(&UnixListener) -> io::Result<UnixStream>,
    ) -> bool {
        if self.stopping() {
            self.cancel();
        }
        if self.run.poll_complete() {
            self.cancel();
            return true;
        }
        if self.listeners.is_none() {
            return false;
        }
        // Each slot is examined once; no blocking receive or unbounded drain.
        for index in 0..2 {
            let result = self.pending[index]
                .as_mut()
                .and_then(AdmissionTicket::take_result);
            if let Some(result) = result {
                self.pending[index].take();
                if let Ok(peer) = result
                    && let Err(error) = self.run.dispatch(peer)
                {
                    self.failure = Some(error);
                    self.cancel();
                    break;
                }
                // Rejected peers receive no receipt/key. Existing admission
                // decides whether rejection also cancelled this generation.
            }
        }
        if self.stopping() {
            self.cancel();
        }
        for _ in 0..4 {
            let Some(slot) = self.pending.iter().position(Option::is_none) else {
                break;
            };
            let Some(listeners) = &self.listeners else {
                break;
            };
            let index = self.next;
            self.next = (self.next + 1) % 2;
            match accept(&listeners[index]) {
                Ok(stream) => match self.run.admit(stream) {
                    Ok(ticket) => self.pending[slot] = Some(ticket),
                    Err((error, stream)) => {
                        drop(stream);
                        if error.kind() != io::ErrorKind::WouldBlock {
                            self.failure = Some(RootError::Unavailable);
                            self.cancel();
                            break;
                        }
                    }
                },
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => {
                    self.failure = Some(RootError::Unavailable);
                    self.cancel();
                    break;
                }
            }
        }
        if self.stopping() {
            self.cancel();
        }
        let complete = self.run.poll_complete();
        if complete {
            self.cancel();
        }
        complete
    }
    fn stopping(&self) -> bool {
        !matches!(self.run.pair.status(), Status::Launching | Status::Running)
    }
    pub(super) fn cancel(&mut self) {
        self.run.cancel();
        self.listeners.take();
        self.pending = [None, None];
    }
    pub(super) fn failure(&self) -> Option<RootError> {
        self.failure
    }
    pub(super) fn root_errors(&self) -> [Option<RootError>; 2] {
        self.run.errors()
    }
}
impl Drop for RootSession {
    fn drop(&mut self) {
        self.cancel();
        // RootDispatch retains/join-waits every job; the sole owner reaps.
    }
}

#[cfg(test)]
#[path = "root_session_tests.rs"]
mod tests;
