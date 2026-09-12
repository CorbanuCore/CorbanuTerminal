//! Private construction stage. No native admission or process-crash recovery.
use super::super::SyntheticChildRole;
use super::sealed::SyntheticProfileInspectedImage;
use codex_linux_pidfd_spawn::OwnedChild;
use std::io;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

static PERMIT: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(false)));
type Job = Box<dyn FnOnce() + Send>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Completion {
    NotLaunched,
    Exited,
    Rejected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Launching,
    Running,
    CleanupPending,
    Complete(Completion),
    Quarantined,
}

pub(super) struct Shared {
    cancel: AtomicBool,
    deadline: Instant,
    phase: Mutex<Status>,
}
impl Shared {
    pub(super) fn cancelled(&self) -> bool {
        if Instant::now() >= self.deadline {
            self.cancel.store(true, Ordering::Release);
        }
        self.cancel.load(Ordering::Acquire)
    }
    pub(super) fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    fn set(&self, status: Status) {
        *self
            .phase
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = status;
    }
}

/// Obtain this reservation before preparing a sealed image. A quarantined
/// worker keeps its permit forever unless it actually observes cleanup.
pub(super) struct Reservation {
    permit: Option<Arc<AtomicBool>>,
}
impl Reservation {
    pub(super) fn acquire() -> io::Result<Self> {
        Self::acquire_from(Arc::clone(&PERMIT))
    }
    fn acquire_from(permit: Arc<AtomicBool>) -> io::Result<Self> {
        permit
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "launch owner busy or quarantined",
                )
            })?;
        Ok(Self {
            permit: Some(permit),
        })
    }
    pub(super) fn launch(
        self,
        image: SyntheticProfileInspectedImage,
        role: SyntheticChildRole,
        deadline: Instant,
    ) -> Result<LaunchHandle, (io::Error, SyntheticProfileInspectedImage)> {
        self.launch_with(image, Kernel(role), deadline, start_worker)
    }
    pub(super) fn launch_with<B: Backend>(
        mut self,
        image: B::Image,
        backend: B,
        deadline: Instant,
        start: impl FnOnce(Job) -> io::Result<()>,
    ) -> Result<LaunchHandle, (io::Error, B::Image)> {
        let Some(permit) = self.permit.as_ref().map(Arc::clone) else {
            return Err((io::Error::other("reservation has no permit"), image));
        };
        let shared = Arc::new(Shared {
            cancel: AtomicBool::new(false),
            deadline,
            phase: Mutex::new(Status::Launching),
        });
        let pending = Arc::new(Mutex::new(Some(image)));
        let worker_pending = Arc::clone(&pending);
        let worker_state = Arc::clone(&shared);
        // The boxed job and retained input exist BEFORE thread creation. A
        // failed Builder::spawn drops its job without executing it; pending
        // retains the image so it can be returned to this trusted caller.
        let job = Box::new(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let Some(image) = worker_pending
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                else {
                    // Internal ownership violation: catch_unwind quarantines
                    // it instead of manufacturing a no-launch cleanup receipt.
                    panic!("worker lost its pending image");
                };
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
            let Some(image) = pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            else {
                // Builder::spawn cannot run the job and return Err. The private
                // injected factory must preserve that same ownership contract.
                panic!("failed worker factory consumed its input");
            };
            return Err((error, image));
        }
        // Worker now owns the permit. Dropping a caller must never release it.
        self.permit.take();
        Ok(LaunchHandle { shared })
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some(permit) = &self.permit {
            permit.store(false, Ordering::Release);
        }
    }
}

pub(super) struct LaunchHandle {
    shared: Arc<Shared>,
}
impl LaunchHandle {
    pub(super) fn control(&self) -> Arc<Shared> {
        Arc::clone(&self.shared)
    }
    pub(super) fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Release);
    }
    pub(super) fn status(&self) -> Status {
        let cancelled = self.shared.cancelled();
        let phase = *self
            .shared
            .phase
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match phase {
            Status::Launching | Status::Running if cancelled => Status::CleanupPending,
            other => other,
        }
    }
}
impl Drop for LaunchHandle {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Private fault seam: actual kernel child or deterministic owned test token.
pub(super) trait Child: Send + 'static {
    fn stop(&self) -> io::Result<()>;
    fn exited(&mut self) -> io::Result<bool>;
}
impl Child for OwnedChild {
    fn stop(&self) -> io::Result<()> {
        self.terminate()
    }
    fn exited(&mut self) -> io::Result<bool> {
        self.try_wait().map(|status| status.is_some())
    }
}
/// Private launch seam, never supplied by an agent or public service caller.
pub(super) trait Backend: Send + 'static {
    type Image: Send + 'static;
    type Child: Child;
    fn spawn(self, image: Self::Image, control: &Arc<Shared>) -> io::Result<Self::Child>;
}
struct Kernel(SyntheticChildRole);
impl Backend for Kernel {
    type Image = SyntheticProfileInspectedImage;
    type Child = OwnedChild;
    fn spawn(self, image: Self::Image, _control: &Arc<Shared>) -> io::Result<Self::Child> {
        image.launch_owned(self.0)
    }
}

pub(super) fn start_worker(job: Job) -> io::Result<()> {
    std::thread::Builder::new()
        .name("corbanu-image-owner".into())
        .spawn(job)
        .map(|_| ())
}

fn supervise<B: Backend>(image: B::Image, backend: B, shared: &Arc<Shared>) -> Completion {
    if shared.cancelled() {
        drop(image);
        return Completion::NotLaunched;
    }
    let mut child = match backend.spawn(image, shared) {
        Ok(child) => child,
        Err(_) => return Completion::Rejected,
    };
    loop {
        if shared.cancelled() {
            shared.set(Status::CleanupPending);
            // Signal failure does not discard the child. Poll and retry using
            // this same owner when OS permissions/support recover.
            let _ = child.stop();
        } else {
            shared.set(Status::Running);
        }
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
pub(super) mod tests;
