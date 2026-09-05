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
    let client =
        NativeAnchorClient::from_authenticated_stream(UnixStream::connect(path).unwrap()).unwrap();
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
