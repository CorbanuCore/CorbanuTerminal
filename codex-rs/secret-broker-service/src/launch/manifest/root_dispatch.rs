//! Private synthetic composition. No listener, enrollment or public launch API.
#[path = "root_session.rs"]
mod session;

use super::pair::admission::AdmissionTicket;
use super::pair::admission::AdmittedPeer;
use super::pair::admission::PairHandle;
use super::spawn::Status;
use crate::SyntheticChildRole;
use codex_protected_state::ControllerRoot;
use codex_protected_state::RootError;
use std::io;
use std::num::NonZeroU64;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread::JoinHandle;

type Job = Box<dyn FnOnce() -> Result<(), RootError> + Send>;
pub(super) struct RootDispatch {
    pair: PairHandle,
    roots: [Arc<ControllerRoot>; 2],
    jobs: [Option<JoinHandle<Result<(), RootError>>>; 2],
    used: [bool; 2],
    errors: [Option<RootError>; 2],
    generation: Option<NonZeroU64>,
}
impl RootDispatch {
    pub(super) fn open_existing(pair: PairHandle) -> Result<Self, RootError> {
        Self::open(pair, || {
            Ok([
                Arc::new(ControllerRoot::open_journal_system()?),
                Arc::new(ControllerRoot::open_policy_system()?),
            ])
        })
    }
    fn open(
        pair: PairHandle,
        roots: impl FnOnce() -> Result<[Arc<ControllerRoot>; 2], RootError>,
    ) -> Result<Self, RootError> {
        // On open failure/unwind, dropping pair cancels the existing sole owner.
        Ok(Self {
            roots: roots()?,
            pair,
            jobs: [None, None],
            used: [false; 2],
            errors: [None; 2],
            generation: None,
        })
    }
    pub(super) fn admit(
        &self,
        stream: UnixStream,
    ) -> Result<AdmissionTicket, (io::Error, UnixStream)> {
        self.pair.admit(stream)
    }
    pub(super) fn dispatch(&mut self, peer: AdmittedPeer) -> Result<(), RootError> {
        self.start(peer, |job| {
            std::thread::Builder::new()
                .name("corbanu-root-channel".into())
                .spawn(job)
        })
    }
    fn start(
        &mut self,
        mut peer: AdmittedPeer,
        start: impl FnOnce(Job) -> io::Result<JoinHandle<Result<(), RootError>>>,
    ) -> Result<(), RootError> {
        let role = match peer.role() {
            SyntheticChildRole::Journal => 0,
            SyntheticChildRole::Policy => 1,
            SyntheticChildRole::Worker => {
                self.pair.cancel();
                return Err(RootError::Invalid);
            }
        };
        if !self.pair.owns(&peer)
            || self.used[role]
            || self.generation.is_some_and(|old| old != peer.generation())
        {
            self.pair.cancel();
            return Err(RootError::Invalid);
        }
        self.generation = Some(peer.generation());
        self.used[role] = true;
        let root = Arc::clone(&self.roots[role]);
        // The receipt stays inside the job: every return/panic/drop cancels both
        // channels through its existing RAII guard, independent of caller polling.
        let job = Box::new(move || peer.serve(&root));
        match start(job) {
            Ok(job) => self.jobs[role] = Some(job),
            Err(_) => {
                self.pair.cancel();
                self.errors[role] = Some(RootError::Unavailable);
                return Err(RootError::Unavailable);
            }
        }
        Ok(())
    }
    pub(super) fn poll_complete(&mut self) -> bool {
        for index in 0..2 {
            if self.jobs[index]
                .as_ref()
                .is_some_and(JoinHandle::is_finished)
                && let Some(job) = self.jobs[index].take()
            {
                self.errors[index] = job.join().unwrap_or(Err(RootError::Unavailable)).err();
                self.pair.cancel();
            }
        }
        self.jobs.iter().all(Option::is_none) && matches!(self.pair.status(), Status::Complete(_))
    }
    pub(super) fn errors(&self) -> [Option<RootError>; 2] {
        self.errors
    }
    pub(super) fn cancel(&self) {
        self.pair.cancel();
    }
}
impl Drop for RootDispatch {
    fn drop(&mut self) {
        self.pair.cancel();
        for job in self.jobs.iter_mut().filter_map(Option::take) {
            // No detached root work or false completion if a storage operation
            // has not returned. Caller can poll boundedly before dropping.
            let _ = job.join();
        }
    }
}

#[cfg(test)]
#[path = "root_dispatch_tests.rs"]
mod tests;
