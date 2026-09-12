//! Private construction stage. No native admission or process-crash recovery.
use super::sealed::SyntheticProfileInspectedImage;
use super::super::SyntheticChildRole;
use codex_linux_pidfd_spawn::OwnedChild;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

static PERMIT: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));
type Job = Box<dyn FnOnce() + Send>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Completion { NotLaunched, Exited, Rejected }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status { Launching, Running, CleanupPending, Complete(Completion), Quarantined }

struct Shared { cancel: AtomicBool, deadline: Instant, phase: Mutex<Status> }
impl Shared {
    fn cancelled(&self) -> bool {
        if Instant::now() >= self.deadline { self.cancel.store(true, Ordering::Release); }
        self.cancel.load(Ordering::Acquire)
    }
    fn set(&self, status: Status) {
        *self.phase.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = status;
    }
}

/// Obtain this reservation before preparing a sealed image. A quarantined
/// worker keeps its permit forever unless it actually observes cleanup.
pub(super) struct Reservation { permit: Option<Arc<AtomicBool>> }
impl Reservation {
    pub(super) fn acquire() -> io::Result<Self> { Self::acquire_from(Arc::clone(&PERMIT)) }
    fn acquire_from(permit: Arc<AtomicBool>) -> io::Result<Self> {
        permit.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| io::Error::new(io::ErrorKind::WouldBlock, "launch owner busy or quarantined"))?;
        Ok(Self { permit: Some(permit) })
    }
    pub(super) fn launch(
        self, image: SyntheticProfileInspectedImage, role: SyntheticChildRole, deadline: Instant,
    ) -> Result<LaunchHandle, (io::Error, SyntheticProfileInspectedImage)> {
        self.launch_with(image, Kernel(role), deadline, start_worker)
    }
    fn launch_with<B: Backend>(
        mut self, image: B::Image, backend: B, deadline: Instant,
        start: impl FnOnce(Job) -> io::Result<()>,
    ) -> Result<LaunchHandle, (io::Error, B::Image)> {
        let shared = Arc::new(Shared { cancel: AtomicBool::new(false), deadline, phase: Mutex::new(Status::Launching) });
        let pending = Arc::new(Mutex::new(Some(image)));
        let worker_pending = Arc::clone(&pending);
        let worker_state = Arc::clone(&shared);
        let permit = Arc::clone(self.permit.as_ref().expect("reservation owns permit"));
        // The boxed job and retained input exist BEFORE thread creation. A
        // failed Builder::spawn drops its job without executing it; pending
        // retains the image so it can be returned to this trusted caller.
        let job = Box::new(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let image = worker_pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take().expect("one worker owns the pending image");
                supervise(image, backend, &worker_state)
            }));
            match result {
                Ok(completion) => {
                    worker_state.set(Status::Complete(completion));
                    permit.store(false, Ordering::Release);
                }
                Err(_) => {
                    // Even if unwinding dropped/reaped the child, panic is not
                    // our positive cleanup receipt. No force-reset/relaunch API.
                    worker_state.set(Status::Quarantined);
                }
            }
        });
        if let Err(error) = start(job) {
            let image = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
                .take().expect("failed worker creation did not run job");
            return Err((error, image));
        }
        // Worker now owns the permit. Dropping a caller must never release it.
        self.permit.take();
        Ok(LaunchHandle { shared })
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some(permit) = &self.permit { permit.store(false, Ordering::Release); }
    }
}

pub(super) struct LaunchHandle { shared: Arc<Shared> }
impl LaunchHandle {
    pub(super) fn cancel(&self) { self.shared.cancel.store(true, Ordering::Release); }
    pub(super) fn status(&self) -> Status {
        let cancelled = self.shared.cancelled();
        let phase = *self.shared.phase.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        match phase {
            Status::Launching | Status::Running if cancelled => Status::CleanupPending,
            other => other,
        }
    }
}
impl Drop for LaunchHandle {
    fn drop(&mut self) { self.cancel(); }
}

/// Private fault seam: actual kernel child or deterministic owned test token.
trait Child: Send + 'static {
    fn stop(&self) -> io::Result<()>;
    fn exited(&mut self) -> io::Result<bool>;
}
impl Child for OwnedChild {
    fn stop(&self) -> io::Result<()> { self.terminate() }
    fn exited(&mut self) -> io::Result<bool> { self.try_wait().map(|status| status.is_some()) }
}
/// Private launch seam, never supplied by an agent or public service caller.
trait Backend: Send + 'static {
    type Image: Send + 'static;
    type Child: Child;
    fn spawn(self, image: Self::Image) -> io::Result<Self::Child>;
}
struct Kernel(SyntheticChildRole);
impl Backend for Kernel {
    type Image = SyntheticProfileInspectedImage;
    type Child = OwnedChild;
    fn spawn(self, image: Self::Image) -> io::Result<Self::Child> { image.launch_owned(self.0) }
}

fn start_worker(job: Job) -> io::Result<()> {
    std::thread::Builder::new().name("corbanu-image-owner".into()).spawn(job).map(|_| ())
}

fn supervise<B: Backend>(image: B::Image, backend: B, shared: &Shared) -> Completion {
    if shared.cancelled() { drop(image); return Completion::NotLaunched; }
    let mut child = match backend.spawn(image) {
        Ok(child) => child,
        Err(_) => return Completion::Rejected,
    };
    loop {
        if shared.cancelled() {
            shared.set(Status::CleanupPending);
            // Signal failure does not discard the child. Poll and retry using
            // this same owner when OS permissions/support recover.
            let _ = child.stop();
        } else { shared.set(Status::Running); }
        match child.exited() {
            Ok(true) => {
                drop(child);
                return Completion::Exited;
            }
            Ok(false) => {}
            Err(_) => {
                shared.cancel.store(true, Ordering::Release);
                shared.set(Status::CleanupPending);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(test)]
#[path = "spawn_tests.rs"]
mod tests;
