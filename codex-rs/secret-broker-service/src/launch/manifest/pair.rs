//! Private two-child lifecycle only; admission and PF20 wiring are separate.
use super::super::SyntheticChildRole;
use super::sealed::SyntheticProfileInspectedImage;
use super::spawn::Backend;
use super::spawn::Child;
use super::spawn::LaunchHandle;
use super::spawn::Reservation;
use super::spawn::Shared;
use super::spawn::start_worker;
use std::io;
use std::sync::Arc;
use std::time::Instant;

pub(super) struct PairImages<I = SyntheticProfileInspectedImage> {
    pub(super) journal: I,
    pub(super) policy: I,
}
impl Reservation {
    pub(super) fn launch_pair(
        self,
        images: PairImages,
        deadline: Instant,
    ) -> Result<LaunchHandle, (io::Error, PairImages)> {
        self.launch_with(
            images,
            PairBackend([
                Kernel(SyntheticChildRole::Journal),
                Kernel(SyntheticChildRole::Policy),
            ]),
            deadline,
            start_worker,
        )
    }
}
struct Kernel(SyntheticChildRole);
impl Backend for Kernel {
    type Image = SyntheticProfileInspectedImage;
    type Child = codex_linux_pidfd_spawn::OwnedChild;
    fn spawn(self, image: Self::Image, _control: &Arc<Shared>) -> io::Result<Self::Child> {
        image.launch_owned(self.0)
    }
}
struct PairBackend<B>([B; 2]);
struct PairChild<C: Child> {
    children: [Option<C>; 2],
    control: Arc<Shared>,
}
impl<B: Backend> Backend for PairBackend<B> {
    type Image = PairImages<B::Image>;
    type Child = PairChild<B::Child>;
    fn spawn(self, images: Self::Image, control: &Arc<Shared>) -> io::Result<Self::Child> {
        let [journal, policy] = self.0;
        let first = journal.spawn(images.journal, control)?;
        let mut pair = PairChild {
            children: [Some(first), None],
            control: Arc::clone(control),
        };
        if !control.cancelled() {
            match policy.spawn(images.policy, control) {
                Ok(child) => pair.children[1] = Some(child),
                Err(_) => control.cancel(),
            }
        }
        // Second-spawn failure still returns ownership of the first child to
        // the precreated supervisor. Rejection cannot release its reservation.
        Ok(pair)
    }
}
impl<C: Child> Child for PairChild<C> {
    fn stop(&self) -> io::Result<()> {
        self.control.cancel();
        let mut error = None;
        for child in self.children.iter().flatten() {
            if let Err(err) = child.stop() {
                error = Some(err);
            }
        }
        error.map_or(Ok(()), Err)
    }
    fn exited(&mut self) -> io::Result<bool> {
        let mut all = true;
        let mut error = None;
        // Do not short-circuit on either an exit or an error: always poll both.
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
}
impl<C: Child> Drop for PairChild<C> {
    fn drop(&mut self) {
        // Signal BOTH before either OwnedChild's potentially blocking Drop.
        // Panic still quarantines the permit, even if cleanup later succeeds.
        let _ = self.stop();
    }
}

#[cfg(test)]
#[path = "pair_tests.rs"]
mod tests;
