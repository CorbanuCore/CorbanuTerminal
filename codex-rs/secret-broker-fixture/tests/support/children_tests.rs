// Immediate setup failure is intentional in this integration fixture.
#![allow(clippy::unwrap_used)]

use codex_secret_broker_service::ChildRole;
use codex_secret_broker_service::TrustedChildRun;
use pretty_assertions::assert_eq;
use std::io;
use std::io::Read;
use std::num::NonZeroU64;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

struct Fixture {
    dir: tempfile::TempDir,
    listener: UnixListener,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let listener = UnixListener::bind(dir.path().join("s")).unwrap();
        listener.set_nonblocking(true).unwrap();
        Self { dir, listener }
    }
    fn child(&self, count: &str) -> Child {
        Command::new(
            codex_utils_cargo_bin::cargo_bin("codex-secret-broker-service-fixture").unwrap(),
        )
        .args(["--synthetic-post-exec-child"])
        .arg(self.dir.path().join("s"))
        .arg(count)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
    }
    fn accept(&self) -> UnixStream {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match self.listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut ready = [0];
                    stream.read_exact(&mut ready).unwrap();
                    assert_eq!(ready, *b"R");
                    // Handlers deliberately block to prove shutdown interrupts.
                    stream.set_read_timeout(None).unwrap();
                    return stream;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "child connect deadline");
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        }
    }
    fn run(&self, journal: Child, policy: Child, journal_uid: u32) -> TrustedChildRun {
        TrustedChildRun::capture(
            NonZeroU64::new(1).unwrap(),
            journal,
            journal_uid,
            policy,
            nix::unistd::geteuid().as_raw(),
        )
        .unwrap_or_else(|(error, mut journal, mut policy)| {
            let _ = journal.kill();
            let _ = policy.kill();
            let _ = journal.wait();
            let _ = policy.wait();
            panic!("capture failed: {error}");
        })
    }
}

fn wait_for_shutdown(run: &mut TrustedChildRun) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !run.shutdown() {
        assert!(Instant::now() < deadline, "bounded shutdown failed");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(run.poll_health().is_err());
}

fn stalled(_: ChildRole, mut stream: UnixStream, _: &mut Child) -> io::Result<()> {
    let mut data = [0];
    stream.read_exact(&mut data)
}

#[test]
fn pf_27_s01_child_roles_route_independently_and_stalled_handlers_shutdown() {
    let f = Fixture::new();
    // Reverse arrival order must not decide namespace.
    let policy = f.child("1");
    let policy_stream = f.accept();
    let journal = f.child("1");
    let journal_stream = f.accept();
    let mut run = f.run(journal, policy, nix::unistd::geteuid().as_raw());
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    assert_eq!(
        run.admit(policy_stream, move |role, stream, child| {
            tx.send(role).unwrap();
            stalled(role, stream, child)
        })
        .unwrap(),
        ChildRole::Policy
    );
    assert_eq!(
        run.admit(journal_stream, move |role, stream, child| {
            tx2.send(role).unwrap();
            stalled(role, stream, child)
        })
        .unwrap(),
        ChildRole::Journal
    );
    let mut roles = vec![
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
    ];
    roles.sort_by_key(|role| match role {
        ChildRole::Journal => 0,
        ChildRole::Policy => 1,
    });
    assert_eq!(roles, vec![ChildRole::Journal, ChildRole::Policy]);
    assert!(run.poll_health().is_ok());
    wait_for_shutdown(&mut run);
}

#[test]
fn pf_27_s01_child_duplicate_parent_and_wrong_uid_connections_deny() {
    let f = Fixture::new();
    let journal = f.child("2");
    let first = f.accept();
    let second = f.accept();
    let policy = f.child("1");
    let _policy = f.accept();
    let mut run = f.run(journal, policy, nix::unistd::geteuid().as_raw());
    let (parent, _other) = UnixStream::pair().unwrap();
    assert!(run.admit(parent, stalled).is_err());
    assert_eq!(run.admit(first, stalled).unwrap(), ChildRole::Journal);
    assert!(run.admit(second, stalled).is_err());
    wait_for_shutdown(&mut run);

    let journal = f.child("1");
    let first = f.accept();
    let policy = f.child("1");
    let _policy = f.accept();
    let mut run = f.run(
        journal,
        policy,
        nix::unistd::geteuid().as_raw().wrapping_add(1),
    );
    assert!(run.admit(first, stalled).is_err());
    wait_for_shutdown(&mut run);
}

#[test]
fn pf_27_s01_child_exit_and_handler_failure_fence_generation() {
    let f = Fixture::new();
    let journal = f.child("1");
    let first = f.accept();
    let policy = f.child("1");
    let policy_stream = f.accept();
    let mut run = f.run(journal, policy, nix::unistd::geteuid().as_raw());
    run.admit(first, |_, _, _| {
        Err(io::Error::other("synthetic handler failure"))
    })
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while run.poll_health().is_ok() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(run.admit(policy_stream, stalled).is_err());
    wait_for_shutdown(&mut run);

    let journal = f.child("1");
    let first = f.accept();
    let policy = f.child("1");
    let _policy = f.accept();
    let mut new_run = f.run(journal, policy, nix::unistd::geteuid().as_raw());
    drop(first); // actual post-exec child exits on EOF
    let deadline = Instant::now() + Duration::from_secs(2);
    while new_run.poll_health().is_ok() {
        assert!(Instant::now() < deadline, "pidfd failed to observe death");
        std::thread::sleep(Duration::from_millis(5));
    }
    wait_for_shutdown(&mut new_run);
}

#[test]
fn pf_27_s01_child_unknown_peer_and_old_generation_cannot_enter_replacement() {
    let f = Fixture::new();
    let journal = f.child("1");
    let old_stream = f.accept();
    let policy = f.child("1");
    let _old_policy = f.accept();
    let mut old = f.run(journal, policy, nix::unistd::geteuid().as_raw());
    let mut unknown = f.child("1");
    let unknown_stream = f.accept();
    assert!(old.admit(unknown_stream, stalled).is_err());
    unknown.kill().unwrap();
    unknown.wait().unwrap();
    wait_for_shutdown(&mut old);

    let journal = f.child("1");
    let new_stream = f.accept();
    let policy = f.child("1");
    let _new_policy = f.accept();
    let uid = nix::unistd::geteuid().as_raw();
    let mut new_run =
        TrustedChildRun::capture(NonZeroU64::new(2).unwrap(), journal, uid, policy, uid)
            .unwrap_or_else(|(error, mut journal, mut policy)| {
                let _ = journal.kill();
                let _ = policy.kill();
                let _ = journal.wait();
                let _ = policy.wait();
                panic!("capture failed: {error}");
            });
    assert_eq!((old.generation().get(), new_run.generation().get()), (1, 2));
    // Deny at socket identity/liveness, before numeric PID matching. Older
    // kernels reject pidfd creation for reaped peers; newer ones return a dead
    // pidfd. Do not require one kernel's error or accept a generic PID mismatch.
    let error = new_run.admit(old_stream, stalled).unwrap_err();
    assert!(
        error.kind() == io::ErrorKind::ConnectionAborted
            || [
                Some(nix::errno::Errno::ENODATA as i32),
                Some(nix::errno::Errno::EINVAL as i32)
            ]
            .contains(&error.raw_os_error()),
        "unexpected stale-peer refusal: {error}"
    );
    assert_eq!(
        new_run.admit(new_stream, stalled).unwrap(),
        ChildRole::Journal
    );
    assert!(old.poll_health().is_err());
    wait_for_shutdown(&mut new_run);
}

#[test]
fn pf_27_s01_child_handler_deadline_fences_without_blocking_supervisor() {
    let f = Fixture::new();
    let journal = f.child("1");
    let first = f.accept();
    let policy = f.child("1");
    let _policy = f.accept();
    let mut run = f.run(journal, policy, nix::unistd::geteuid().as_raw());
    run.admit(first, |role, stream, child| {
        stream.set_read_timeout(Some(Duration::from_millis(40)))?;
        stalled(role, stream, child)
    })
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while run.poll_health().is_ok() {
        assert!(Instant::now() < deadline, "handler deadline did not fence");
        std::thread::sleep(Duration::from_millis(5));
    }
    wait_for_shutdown(&mut run);
}
