use super::*;
use crate::checkpoint::Binding;
use codex_security_policy::BoundedText;
use codex_security_policy::PolicyPrincipal;
use codex_security_policy::PrincipalKind;
use pretty_assertions::assert_eq;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::process::Stdio;

fn fixture() -> (tempfile::TempDir, ControllerRoot) {
    let temp = tempfile::tempdir().unwrap();
    for name in ["registry", "storage"] {
        fs::create_dir(temp.path().join(name)).unwrap();
        fs::set_permissions(temp.path().join(name), fs::Permissions::from_mode(0o700)).unwrap();
    }
    let root = ControllerRoot::enroll(
        &temp.path().join("registry"),
        &temp.path().join("storage"),
        Binding::Journal {
            producer: PolicyPrincipal::new(PrincipalKind::Service, "native-fixture").unwrap(),
            owner_generation: 1,
            integrity_key_id: BoundedText::new("fixture-key").unwrap(),
        },
    )
    .unwrap();
    (temp, root)
}

#[test]
fn pf20_s03_post_exec_child_native_cas_recovery() {
    let (temp, root) = fixture();
    let path = temp.path().join("native.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "native::tests::native_child", "--ignored"])
        .env("CORBANU_ANCHOR_NATIVE_FIXTURE", &path)
        .spawn()
        .unwrap();
    let (socket, _) = listener.accept().unwrap();
    assert_eq!(peer(&socket).unwrap().pid as u32, child.id());
    // EOF after the child finishes is a closed capability, not service success.
    assert!(root.serve_child(socket, &mut child).is_err());
    assert!(child.wait().unwrap().success());
    assert_eq!(
        IntegrityRootStore::load(&root).unwrap().unwrap().sequence,
        1
    );
}

#[test]
#[ignore = "invoked by real subprocess fixture"]
fn native_child() {
    let path = std::env::var_os("CORBANU_ANCHOR_NATIVE_FIXTURE").unwrap();
    let client = NativeAnchorClient::from_authenticated_stream(
        connect_without_waiting(Path::new(&path)).unwrap(),
    )
    .unwrap();
    assert_eq!(client.load(), Ok(None));
    let checkpoint = IntegrityCheckpoint {
        schema_version: 1,
        sequence: 1,
        record_sha256: "a".repeat(64),
        producer: PolicyPrincipal::new(PrincipalKind::Service, "native-fixture").unwrap(),
        owner_generation: 1,
        integrity_key_id: BoundedText::new("fixture-key").unwrap(),
        policy_generation: 1,
        run_generation: 1,
    };
    client.compare_and_store(None, &checkpoint).unwrap();
    assert_eq!(client.load(), Ok(Some(checkpoint)));
}

#[test]
fn pf20_s03_inherited_socketpair_does_not_prove_child_identity() {
    let (_temp, root) = fixture();
    let (socket, _other) = UnixStream::pair().unwrap();
    let mut child = Command::new("/bin/sleep")
        .arg("2")
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    assert_eq!(
        root.serve_child(socket, &mut child),
        Err(RootError::Invalid)
    );
    child.wait().unwrap();
}

#[test]
fn pf20_s03_saturated_native_backlog_fails_without_waiting() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("full.sock");
    let listener = UnixListener::bind(&path).unwrap();
    // SAFETY: live listening Unix socket; narrow the queue for this fixture.
    assert_eq!(unsafe { libc::listen(listener.as_raw_fd(), 1) }, 0);
    let mut held = Vec::new();
    let started = Instant::now();
    for _ in 0..8 {
        match connect_without_waiting(&path) {
            Ok(stream) => held.push(stream),
            Err(error) => {
                assert_eq!(error, RootError::Unavailable);
                assert!(!held.is_empty());
                assert!(started.elapsed() < Duration::from_secs(3));
                return;
            }
        }
    }
    panic!("fixture did not reach the saturated backlog");
}

#[test]
fn pf20_s03_lost_reply_consumes_capability_without_retry() {
    let (mut server, client) = UnixStream::pair().unwrap();
    let thread = std::thread::spawn(move || {
        write(&mut server, &[3_u8; 32]).unwrap();
        let _: Packet = read(&mut server).unwrap();
        // Simulates death/ack loss after potentially applying the received CAS.
    });
    let client = NativeAnchorClient::from_authenticated_stream(client).unwrap();
    let checkpoint = IntegrityCheckpoint {
        schema_version: 1,
        sequence: 1,
        record_sha256: "a".repeat(64),
        producer: PolicyPrincipal::new(PrincipalKind::Service, "native-fixture").unwrap(),
        owner_generation: 1,
        integrity_key_id: BoundedText::new("fixture-key").unwrap(),
        policy_generation: 1,
        run_generation: 1,
    };
    assert_eq!(
        client.compare_and_store(None, &checkpoint),
        Err(IntegrityRootError::Timeout)
    );
    assert_eq!(client.load(), Err(IntegrityRootError::Unavailable));
    thread.join().unwrap();
}

#[test]
fn pf20_s03_native_replay_cross_generation_and_oversized_frames_deny() {
    for different_key in [false, true] {
        let (server, client) = UnixStream::pair().unwrap();
        let mut sender = Channel {
            stream: client,
            key: Zeroizing::new([1; 32]),
            sequence: 1,
        };
        let mut receiver = Channel {
            stream: server,
            key: Zeroizing::new(if different_key { [2; 32] } else { [1; 32] }),
            sequence: if different_key { 1 } else { 2 },
        };
        sender
            .send(&Request::LoadJournal, b"corbanu-anchor-request/v1")
            .unwrap();
        assert!(matches!(
            receiver.receive::<Request>(b"corbanu-anchor-request/v1"),
            Err(RootError::Invalid)
        ));
    }
    let (mut server, mut client) = UnixStream::pair().unwrap();
    client.write_all(&u32::MAX.to_be_bytes()).unwrap();
    assert!(matches!(
        read::<Packet>(&mut server),
        Err(RootError::Invalid)
    ));
}

#[test]
fn pf20_s03_authenticated_definite_conflict_is_not_ambiguous() {
    let (mut server, client) = UnixStream::pair().unwrap();
    let thread = std::thread::spawn(move || {
        write(&mut server, &[3_u8; 32]).unwrap();
        let mut channel = Channel {
            stream: server,
            key: Zeroizing::new([3; 32]),
            sequence: 1,
        };
        let _: Request = channel.receive(b"corbanu-anchor-request/v1").unwrap();
        channel
            .send(
                &Reply::Rejected(RootError::Conflict),
                b"corbanu-anchor-reply/v1",
            )
            .unwrap();
    });
    let client = NativeAnchorClient::from_authenticated_stream(client).unwrap();
    let owner = codex_config::AuthoritativeStateOwner::new("a".repeat(64), "fixture", 1).unwrap();
    let next = PolicyCheckpoint {
        schema_version: 1,
        revision: 1,
        owner,
        state_sha256: "a".repeat(64),
        commit_sha256: "b".repeat(64),
    };
    assert_eq!(
        crate::PolicyRootStore::compare_policy(&client, None, &next),
        Err(RootError::Conflict)
    );
    assert_eq!(client.load(), Err(IntegrityRootError::Unavailable));
    thread.join().unwrap();
}

#[test]
fn pf20_s03_partial_frame_obeys_one_total_deadline() {
    let (mut server, mut client) = UnixStream::pair().unwrap();
    client.write_all(&[1]).unwrap();
    let mut output = [0; 2];
    let start = Instant::now();
    assert_eq!(
        read_before(&mut server, &mut output, start + Duration::from_millis(25)),
        Err(RootError::Unavailable)
    );
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
#[derive(Clone, Copy)]
enum Entry {
    Descriptor,
    Legacy,
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
enum RelayOwner {
    Descriptor(codex_linux_pidfd_spawn::OwnedChild),
    Legacy(Child),
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
impl Drop for RelayOwner {
    fn drop(&mut self) {
        if let Self::Legacy(child) = self {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
fn sealed_artifact(key: &str) -> std::os::fd::OwnedFd {
    let bytes = fs::read(std::env::var_os(key).expect("hashed fixture required")).unwrap();
    // SAFETY: fixed valid C string and flags; returned fd is newly owned.
    let raw = unsafe {
        libc::memfd_create(
            c"pf27-root-relay".as_ptr(),
            libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING,
        )
    };
    assert!(raw >= 0);
    let mut file = unsafe { File::from_raw_fd(raw) };
    file.write_all(&bytes).unwrap();
    // SAFETY: live descriptor, only fixed sealing flags.
    assert_eq!(
        unsafe {
            libc::fcntl(
                file.as_raw_fd(),
                libc::F_ADD_SEALS,
                libc::F_SEAL_WRITE | libc::F_SEAL_SHRINK | libc::F_SEAL_GROW | libc::F_SEAL_SEAL,
            )
        },
        0
    );
    file.into()
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
fn relay(entry: Entry) -> (RelayOwner, UnixStream, UnixStream, UnixStream) {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::SocketAddr;
    let name = format!("corbanu-pf27-root-{}", std::process::id());
    let listener = UnixListener::bind_addr(&SocketAddr::from_abstract_name(name).unwrap()).unwrap();
    listener.set_nonblocking(true).unwrap();
    let owner = match entry {
        Entry::Descriptor => RelayOwner::Descriptor(
            codex_linux_pidfd_spawn::spawn_synthetic_probe(
                sealed_artifact("PF27_ROOT_RELAY"),
                codex_linux_pidfd_spawn::SyntheticRole::Journal,
            )
            .unwrap(),
        ),
        Entry::Legacy => RelayOwner::Legacy(
            Command::new(std::env::var_os("PF27_ROOT_RELAY").unwrap())
                .args(["--prepare-synthetic-child", "journal", "101", "201", "204"])
                .env_clear()
                .current_dir("/")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        ),
    };
    let accept = || {
        let until = Instant::now() + Duration::from_secs(3);
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    configure(&stream).unwrap();
                    return stream;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < until, "relay did not connect");
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(error) => panic!("accept: {error}"),
            }
        }
    };
    (owner, accept(), accept(), accept())
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
fn relay_server(
    root: std::sync::Arc<ControllerRoot>,
    mut owner: RelayOwner,
    stream: UnixStream,
) -> std::thread::JoinHandle<Result<(), RootError>> {
    std::thread::spawn(move || match &mut owner {
        RelayOwner::Descriptor(child) => {
            root.serve_owned_child(stream, child.retain_identity().unwrap())
        }
        RelayOwner::Legacy(child) => root.serve_child(stream, child),
    })
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
fn relay_checkpoint() -> IntegrityCheckpoint {
    IntegrityCheckpoint {
        schema_version: 1,
        sequence: 1,
        record_sha256: "a".repeat(64),
        producer: PolicyPrincipal::new(PrincipalKind::Service, "native-fixture").unwrap(),
        owner_generation: 1,
        integrity_key_id: BoundedText::new("fixture-key").unwrap(),
        policy_generation: 1,
        run_generation: 1,
    }
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
#[test]
#[ignore = "source-qualified static relay and non-root kernel proof"]
fn pf27_root_compat_both_entries_cas_and_death() {
    for entry in [Entry::Descriptor, Entry::Legacy] {
        let (_temp, root) = fixture();
        let root = std::sync::Arc::new(root);
        let (owner, server, client, mut control) = relay(entry);
        let worker = relay_server(std::sync::Arc::clone(&root), owner, server);
        let client = NativeAnchorClient::from_authenticated_stream(client).unwrap();
        assert_eq!(client.load(), Ok(None));
        let next = relay_checkpoint();
        client.compare_and_store(None, &next).unwrap();
        assert_eq!(client.load(), Ok(Some(next.clone())));
        control.write_all(b"x").unwrap();
        assert!(worker.join().unwrap().is_err());
        assert!(client.load().is_err());
        assert_eq!(client.load(), Err(IntegrityRootError::Unavailable));
        assert_eq!(IntegrityRootStore::load(root.as_ref()).unwrap(), Some(next));
    }
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
#[test]
#[ignore = "source-qualified static relay and non-root kernel proof"]
fn pf27_root_compat_both_entries_reject_malformed_and_replay() {
    for entry in [Entry::Descriptor, Entry::Legacy] {
        for replay in [false, true] {
            let (_temp, root) = fixture();
            let (owner, server, mut client, _control) = relay(entry);
            let worker = relay_server(std::sync::Arc::new(root), owner, server);
            let key: [u8; 32] = read(&mut client).unwrap();
            if replay {
                let mut channel = Channel {
                    stream: client,
                    key: Zeroizing::new(key),
                    sequence: 1,
                };
                channel
                    .send(&Request::LoadJournal, b"corbanu-anchor-request/v1")
                    .unwrap();
                assert!(matches!(
                    channel.receive::<Reply>(b"corbanu-anchor-reply/v1"),
                    Ok(Reply::Loaded(None))
                ));
                // Intentionally reuse the authenticated old sequence.
                channel
                    .send(&Request::LoadJournal, b"corbanu-anchor-request/v1")
                    .unwrap();
                assert!(
                    channel
                        .receive::<Reply>(b"corbanu-anchor-reply/v1")
                        .is_err()
                );
            } else {
                client.write_all(&u32::MAX.to_be_bytes()).unwrap();
                let mut byte = [0];
                assert_eq!(client.read(&mut byte).unwrap(), 0);
            }
            assert_eq!(worker.join().unwrap(), Err(RootError::Invalid));
        }
    }
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
#[test]
#[ignore = "source-qualified static relay and non-root kernel proof"]
fn pf27_root_compat_both_entries_lost_reply_is_ambiguous() {
    for entry in [Entry::Descriptor, Entry::Legacy] {
        let (_temp, root) = fixture();
        let root = std::sync::Arc::new(root);
        let (owner, server, client, mut control) = relay(entry);
        let worker = relay_server(std::sync::Arc::clone(&root), owner, server);
        let client = NativeAnchorClient::from_authenticated_stream(client).unwrap();
        control.write_all(b"p").unwrap();
        let mut ack = [0];
        control.read_exact(&mut ack).unwrap();
        assert_eq!(ack, *b"k");
        let next = relay_checkpoint();
        let writer = std::thread::spawn(move || {
            let result = client.exchange(&Request::Compare {
                expected: None,
                next: Box::new(Checkpoint::Journal(next)),
            });
            assert!(matches!(result, Err(RootError::Ambiguous)));
            assert_eq!(client.load(), Err(IntegrityRootError::Unavailable));
        });
        let until = Instant::now() + Duration::from_secs(3);
        while IntegrityRootStore::load(root.as_ref()).unwrap().is_none() {
            assert!(Instant::now() < until, "compare never committed");
            std::thread::sleep(Duration::from_millis(2));
        }
        // Publication is proven; discard its withheld reply by killing relay.
        control.write_all(b"x").unwrap();
        writer.join().unwrap();
        assert!(worker.join().unwrap().is_err());
        assert_eq!(
            IntegrityRootStore::load(root.as_ref()).unwrap(),
            Some(relay_checkpoint())
        );
    }
}

#[cfg(all(target_env = "gnu", feature = "synthetic-fixture"))]
#[test]
#[ignore = "source-qualified static relay and non-root kernel proof"]
fn pf27_root_compat_rejected_identity_never_receives_key() {
    for case in ["wrong", "inherited", "dead", "stale", "kernel"] {
        let (_temp, root) = fixture();
        let (owner, server, mut client, _control) = relay(Entry::Descriptor);
        let RelayOwner::Descriptor(child) = &owner else {
            unreachable!()
        };
        let wrong = (case == "wrong").then(|| {
            codex_linux_pidfd_spawn::spawn_synthetic_probe(
                sealed_artifact("PF27_SYNTHETIC_HOLD"),
                codex_linux_pidfd_spawn::SyntheticRole::Journal,
            )
            .unwrap()
        });
        let identity = wrong.as_ref().unwrap_or(child).retain_identity().unwrap();
        let (server, mut observer) = if case == "inherited" {
            UnixStream::pair().unwrap()
        } else if case == "kernel" {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let tcp = std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
            let (other, _) = listener.accept().unwrap();
            let fd: std::os::fd::OwnedFd = tcp.into();
            let other: std::os::fd::OwnedFd = other.into();
            (UnixStream::from(fd), UnixStream::from(other))
        } else {
            if case != "wrong" {
                child.terminate().unwrap();
                let until = Instant::now() + Duration::from_secs(3);
                while identity.check_live().is_ok() {
                    assert!(Instant::now() < until, "child failed to exit");
                    std::thread::yield_now();
                }
            }
            (server, client.try_clone().unwrap())
        };
        configure(&observer).unwrap();
        if case == "stale" {
            drop(owner);
        }
        assert!(root.serve_owned_child(server, identity).is_err());
        // The directly rejected endpoint is closed without even a key prefix.
        if case == "inherited" || case == "kernel" {
            let mut byte = [0];
            assert_eq!(observer.read(&mut byte).unwrap(), 0);
        } else {
            let mut byte = [0];
            assert_eq!(client.read(&mut byte).unwrap(), 0);
        }
    }
}
