//! Synthetic construction proof, not installed PF20 CAS or UID containment.
// Fixture setup/assertion failure should immediately fail the test.
#![allow(clippy::unwrap_used)]
use super::*;
use pretty_assertions::assert_eq;
use std::io::Read;
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

struct Fixture {
    _dir: tempfile::TempDir,
    run: TrustedChildRun,
    journal: UnixStream,
    policy: UnixStream,
    pids: [u32; 2],
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s");
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let launch = || {
            Command::new(
                codex_utils_cargo_bin::cargo_bin("codex-secret-broker-service-fixture").unwrap(),
            )
            .arg("--synthetic-post-exec-child")
            .arg(&path)
            .arg("1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
        };
        let policy_child = launch();
        let policy = accept(&listener);
        let journal_child = launch();
        let journal = accept(&listener);
        let pids = [journal_child.id(), policy_child.id()];
        let uid = nix::unistd::geteuid().as_raw();
        let run = TrustedChildRun::capture(
            NonZeroU64::new(7).unwrap(),
            journal_child,
            uid,
            policy_child,
            uid,
        )
        .unwrap_or_else(|(error, mut a, mut b)| {
            let _ = a.kill();
            let _ = b.kill();
            let _ = a.wait();
            let _ = b.wait();
            panic!("capture failed: {error}");
        });
        Self {
            _dir: dir,
            run,
            journal,
            policy,
            pids,
        }
    }
}

fn accept(listener: &UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut ready = [0];
                stream.read_exact(&mut ready).unwrap();
                assert_eq!(ready, *b"R");
                stream.set_read_timeout(None).unwrap();
                return stream;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
}

fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !condition() {
        assert!(Instant::now() < deadline, "bounded cleanup did not finish");
        std::thread::sleep(Duration::from_millis(5));
    }
}

enum Outcome {
    Stall,
    Fail(RootError),
    Panic,
    Return,
}
struct FakeRoot {
    role: ChildRole,
    seen: mpsc::Sender<(ChildRole, u32)>,
    outcome: Outcome,
    dropped: Arc<AtomicUsize>,
}
impl Drop for FakeRoot {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
impl RootHandler for FakeRoot {
    fn serve(&self, mut stream: UnixStream, child: &mut Child) -> Result<(), RootError> {
        self.seen.send((self.role, child.id())).unwrap();
        match self.outcome {
            Outcome::Stall => {
                let mut byte = [0];
                let _ = stream.read_exact(&mut byte);
                Err(RootError::Unavailable)
            }
            Outcome::Fail(error) => Err(error),
            Outcome::Panic => panic!("synthetic root panic"),
            Outcome::Return => Ok(()),
        }
    }
}
fn fake(
    role: ChildRole,
    seen: &mpsc::Sender<(ChildRole, u32)>,
    dropped: &Arc<AtomicUsize>,
    outcome: Outcome,
) -> FakeRoot {
    FakeRoot {
        role,
        seen: seen.clone(),
        dropped: Arc::clone(dropped),
        outcome,
    }
}

#[test]
fn pf_27_s01_root_mapping_uses_child_role_and_shutdown_releases_roots() {
    let f = Fixture::new();
    let (tx, rx) = mpsc::channel();
    let drops = Arc::new(AtomicUsize::new(0));
    let mut composed = Composition::open(
        f.run,
        || Ok(fake(ChildRole::Journal, &tx, &drops, Outcome::Stall)),
        || Ok(fake(ChildRole::Policy, &tx, &drops, Outcome::Stall)),
    )
    .unwrap();
    // Reverse arrival order still selects the correct stored root and Child.
    assert_eq!(composed.admit(f.policy).unwrap(), ChildRole::Policy);
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        (ChildRole::Policy, f.pids[1])
    );
    assert_eq!(composed.admit(f.journal).unwrap(), ChildRole::Journal);
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        (ChildRole::Journal, f.pids[0])
    );
    assert!(composed.children.poll_health().is_ok());
    wait_until(|| composed.children.shutdown());
    assert!(composed.children.poll_health().is_err());
    drop(composed);
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn pf_27_s01_root_open_failures_release_first_root_and_both_children() {
    for failure_role in [ChildRole::Journal, ChildRole::Policy] {
        let f = Fixture::new();
        let (tx, _) = mpsc::channel();
        let drops = Arc::new(AtomicUsize::new(0));
        let result = Composition::open(
            f.run,
            || {
                if failure_role == ChildRole::Journal {
                    Err(RootError::MissingKey)
                } else {
                    Ok(fake(ChildRole::Journal, &tx, &drops, Outcome::Stall))
                }
            },
            || Err(RootError::MissingKey),
        );
        assert!(matches!(result, Err(RootError::MissingKey)));
        wait_until(|| {
            f.pids
                .iter()
                .all(|pid| !std::path::Path::new(&format!("/proc/{pid}")).exists())
        });
        assert_eq!(
            drops.load(Ordering::SeqCst),
            usize::from(failure_role == ChildRole::Policy)
        );
    }
}

#[test]
fn pf_27_s01_native_root_open_denies_nonroot_and_fences_children() {
    // This qualification intentionally requires a real non-root runner.
    assert!(
        !nix::unistd::geteuid().is_root(),
        "run native denial as non-root"
    );
    let f = Fixture::new();
    assert!(matches!(
        RootChildRun::open_existing(f.run),
        Err(RootError::Unavailable)
    ));
    wait_until(|| {
        f.pids
            .iter()
            .all(|pid| !std::path::Path::new(&format!("/proc/{pid}")).exists())
    });
}

#[test]
fn pf_27_s01_root_terminal_outcomes_fence_and_preserve_ambiguous_errors() {
    for outcome in [
        Outcome::Fail(RootError::Ambiguous),
        Outcome::Panic,
        Outcome::Return,
    ] {
        let f = Fixture::new();
        let (tx, rx) = mpsc::channel();
        let drops = Arc::new(AtomicUsize::new(0));
        let expected = if matches!(outcome, Outcome::Fail(_)) {
            Some(RootError::Ambiguous)
        } else {
            None
        };
        let mut composed = Composition::open(
            f.run,
            || Ok(fake(ChildRole::Journal, &tx, &drops, outcome)),
            || Ok(fake(ChildRole::Policy, &tx, &drops, Outcome::Stall)),
        )
        .unwrap();
        composed.admit(f.journal).unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        wait_until(|| composed.children.poll_health().is_err());
        assert_eq!(*composed.failures.lock().unwrap(), [expected, None]);
        assert!(composed.admit(f.policy).is_err());
        wait_until(|| composed.children.shutdown());
        drop(composed);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}
