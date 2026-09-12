//! Existing-root composition only. No listener, enrollment or process launcher.
use crate::ChildRole;
use crate::TrustedChildRun;
use codex_protected_state::ControllerRoot;
use codex_protected_state::RootError;
use std::io;
use std::num::NonZeroU64;
use std::os::unix::net::UnixStream;
use std::process::Child;
use std::sync::Arc;
use std::sync::Mutex;

/// Private dispatch boundary. Production has exactly the fixed-path PF20
/// implementation; tests can inject failures without minting native authority.
trait RootHandler: Send + Sync + 'static {
    fn serve(&self, stream: UnixStream, child: &mut Child) -> Result<(), RootError>;
}

impl RootHandler for ControllerRoot {
    fn serve(&self, stream: UnixStream, child: &mut Child) -> Result<(), RootError> {
        self.serve_child(stream, child)
    }
}

struct Roots<R> {
    journal: R,
    policy: R,
}

struct Composition<R> {
    children: TrustedChildRun,
    roots: Arc<Roots<R>>,
    failures: Arc<Mutex<[Option<RootError>; 2]>>,
}

fn role_index(role: ChildRole) -> usize {
    match role {
        ChildRole::Journal => 0,
        ChildRole::Policy => 1,
    }
}

impl<R: RootHandler> Composition<R> {
    fn open(
        children: TrustedChildRun,
        journal: impl FnOnce() -> Result<R, RootError>,
        policy: impl FnOnce() -> Result<R, RootError>,
    ) -> Result<Self, RootError> {
        // On either error, locals release any opened root and Drop fences the
        // owned children using their preallocated reaper. Never enroll here.
        let journal = journal()?;
        let policy = policy()?;
        Ok(Self {
            children,
            roots: Arc::new(Roots { journal, policy }),
            failures: Arc::new(Mutex::new([None; 2])),
        })
    }

    fn admit(&mut self, stream: UnixStream) -> io::Result<ChildRole> {
        let roots = Arc::clone(&self.roots);
        let failures = Arc::clone(&self.failures);
        self.children.admit(stream, move |role, stream, child| {
            let root = match role {
                ChildRole::Journal => &roots.journal,
                ChildRole::Policy => &roots.policy,
            };
            let result = root.serve(stream, child);
            if let Err(error) = result {
                failures
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)[role_index(role)] =
                    Some(error);
            }
            // TrustedChildRun fences on every return (also panic). Retain the
            // typed error separately: Ambiguous must not become a safe retry.
            result.map_err(io::Error::other)
        })
    }
}

/// Fixed native roots joined to an already captured trusted child generation.
/// Caller still owns executable/UID/containment qualification and must drive
/// health polling and bounded shutdown. No installation or activation is implied.
pub struct RootChildRun(Composition<ControllerRoot>);

impl RootChildRun {
    /// Open both existing fixed-path roots as the real root principal. Failure
    /// fences and relinquishes the supplied children; it never repairs state.
    pub fn open_existing(children: TrustedChildRun) -> Result<Self, RootError> {
        Composition::open(
            children,
            ControllerRoot::open_journal_system,
            ControllerRoot::open_policy_system,
        )
        .map(Self)
    }

    /// Namespace derives solely from the owned child's kernel-verified role.
    /// Successful admission is not handshake completion or protected readiness.
    pub fn admit(&mut self, stream: UnixStream) -> io::Result<ChildRole> {
        self.0.admit(stream)
    }

    pub fn generation(&self) -> NonZeroU64 {
        self.0.children.generation()
    }

    pub fn poll_health(&self) -> io::Result<()> {
        self.0.children.poll_health()
    }

    /// Last terminal PF20 error for this role. None is not a health/readiness
    /// indication: a panic or child death may fence without a returned error.
    pub fn root_error(&self, role: ChildRole) -> Option<RootError> {
        self.0
            .failures
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[role_index(role)]
    }

    /// False means cleanup is unfinished. Keep polling during bounded shutdown;
    /// never bootstrap a replacement while previous cleanup is outstanding.
    pub fn shutdown(&mut self) -> bool {
        self.0.children.shutdown()
    }
}

#[cfg(all(test, feature = "synthetic-fixture"))]
#[path = "root_tests.rs"]
mod tests;
