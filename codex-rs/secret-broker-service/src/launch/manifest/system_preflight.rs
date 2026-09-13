//! Uncalled fixed-system preparation only; no launch, enrollment or readiness API.
use super::SyntheticManifestInspection;
use super::denied;
use super::sealed::SyntheticProfileInspectedImage;
use super::sealed::SyntheticSealedImage;
use super::spawn::Reservation;
use codex_protected_state::ControllerRoot;
use std::io;
use std::sync::Arc;

pub(super) struct SystemPreflight {
    _images: [SyntheticProfileInspectedImage; 2],
    _roots: [Arc<ControllerRoot>; 2],
    // Declared last: intermediates drop before the reservation is released.
    _reservation: Reservation,
}
enum RootNamespace {
    Journal,
    Policy,
}

/// Private preparation operations; production uses only fixed system factories.
/// Test implementations are confined to the cfg(test) module, never caller inputs.
trait Inputs {
    fn inspect(&mut self) -> io::Result<SyntheticManifestInspection>;
    fn seal(&mut self, image: SyntheticManifestInspection) -> io::Result<SyntheticSealedImage> {
        image.seal()
    }
    fn profile(
        &mut self,
        image: SyntheticSealedImage,
    ) -> io::Result<SyntheticProfileInspectedImage> {
        image.inspect_static_profile()
    }
    fn root(&mut self, namespace: RootNamespace) -> io::Result<Arc<ControllerRoot>>;
}
struct System;
impl Inputs for System {
    fn inspect(&mut self) -> io::Result<SyntheticManifestInspection> {
        SyntheticManifestInspection::inspect_system()
    }
    fn root(&mut self, namespace: RootNamespace) -> io::Result<Arc<ControllerRoot>> {
        let root = match namespace {
            RootNamespace::Journal => ControllerRoot::open_journal_system(),
            RootNamespace::Policy => ControllerRoot::open_policy_system(),
        };
        root.map(Arc::new)
            .map_err(|_| denied("existing root unavailable"))
    }
}
impl SystemPreflight {
    pub(super) fn inspect_system() -> io::Result<Self> {
        Self::prepare(&mut System)
    }
    fn prepare(inputs: &mut impl Inputs) -> io::Result<Self> {
        let reservation = Reservation::acquire()?;
        let first = inputs.inspect()?;
        let second = inputs.inspect()?;
        // Executable path encodes the validated source commit. Compare the full
        // source stamp and recipe as well as digest BEFORE consuming either.
        // This is consistency of retained observations, not filesystem atomicity.
        if first._recipe.executable != second._recipe.executable
            || first._stamp != second._stamp
            || first._digest != second._digest
            || first._recipe.identities != second._recipe.identities
            || first._recipe.anchor_gid != second._recipe.anchor_gid
        {
            return Err(denied("inconsistent system preparation"));
        }
        let first = inputs.seal(first)?;
        let first = inputs.profile(first)?;
        let second = inputs.seal(second)?;
        let second = inputs.profile(second)?;
        let journal = inputs.root(RootNamespace::Journal)?;
        let policy = inputs.root(RootNamespace::Policy)?;
        Ok(Self {
            _images: [first, second],
            _roots: [journal, policy],
            _reservation: reservation,
        })
    }
}

#[cfg(test)]
#[path = "system_preflight_tests.rs"]
mod tests;
