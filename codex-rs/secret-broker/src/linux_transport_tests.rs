use super::*;
use crate::BrokerBinding;
use crate::BrokerChannelMac;
use crate::BrokerOperation;
use crate::CredentialReference;
use crate::OpenAiResponsesOperation;
use pretty_assertions::assert_eq;
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

fn frame() -> SignedBrokerFrame {
    BrokerChannelMac::from_secret([9; 32]).sign(
        BrokerBinding {
            controller_instance: "controller".into(), worker_instance: "worker".into(),
            session_id: "session".into(), task_id: "task".into(), run_id: "run".into(), run_generation: 1,
        },
        1,
        BrokerOperation::OpenAiResponses {
            credential: CredentialReference::from_sha256_hex("a".repeat(64)).unwrap(),
            request: OpenAiResponsesOperation::new("/v1/responses").unwrap(),
        },
    ).unwrap()
}

struct Handler(Arc<AtomicUsize>);
impl LinuxBrokerHandler for Handler {
    fn dispatch(&self, _: &ObservedPeer, _: &SignedBrokerFrame) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        Ok(TypedOperationReceipt { response_status: 200, uploaded_bytes: 11, downloaded_bytes: 22 })
    }
    fn close(&self) { self.0.fetch_add(1, Ordering::SeqCst); }
}

#[test]
fn pf_27_s01_native_close_interrupts_inflight_read_and_queued_dispatch() {
    let (client, mut server) = UnixStream::pair().unwrap();
    let peer = observed_peer(&client).unwrap();
    let channel = Arc::new(LinuxBrokerChannel::new(client, &peer).unwrap());
    let dispatch_channel = channel.clone();
    let dispatch = thread::spawn(move || dispatch_channel.dispatch(&frame()));
    let _frame = read_frame(&mut server).unwrap();
    let started = std::time::Instant::now();
    channel.close().unwrap();
    assert_eq!(dispatch.join().unwrap(), Err(BrokerDispatchError::OutcomeUnknown));
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(channel.dispatch(&frame()), Err(BrokerDispatchError::SessionUnavailable));
}

#[test]
fn pf_27_s01_native_peer_roundtrip_and_eof_cancels_session() {
    let (client, server) = UnixStream::pair().unwrap();
    let peer = observed_peer(&client).unwrap();
    assert_eq!(peer.process_id(), std::process::id());
    assert_eq!(peer.principal(), format!("uid:{}", nix::unistd::getuid()));
    let channel = LinuxBrokerChannel::new(client, &peer).unwrap();
    let closed = Arc::new(AtomicUsize::new(0));
    let handler = Handler(closed.clone());
    let server = thread::spawn(move || serve_connection(server, &peer, &handler));
    assert_eq!(channel.dispatch(&frame()).unwrap(), TypedOperationReceipt { response_status: 200, uploaded_bytes: 11, downloaded_bytes: 22 });
    channel.close().unwrap();
    assert!(server.join().unwrap().is_err());
    assert_eq!(closed.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s01_native_wrong_peer_and_oversize_frame_close_before_dispatch() {
    let (mut client, server) = UnixStream::pair().unwrap();
    let actual = observed_peer(&client).unwrap();
    let wrong = ObservedPeer::from_os(actual.principal(), actual.process_id() + 1).unwrap();
    assert!(matches!(LinuxBrokerChannel::new(client.try_clone().unwrap(), &wrong), Err(BrokerDispatchError::WrongPeer)));
    let closed = Arc::new(AtomicUsize::new(0));
    let handler = Handler(closed.clone());
    assert_eq!(serve_connection(server, &wrong, &handler), Err(BrokerDispatchError::WrongPeer));
    let (new_client, server) = UnixStream::pair().unwrap();
    client = new_client;
    let server = thread::spawn(move || serve_connection(server, &actual, &handler));
    client.write_all(&u32::MAX.to_be_bytes()).unwrap();
    assert!(matches!(server.join().unwrap(), Err(BrokerDispatchError::Frame(_))));
    assert_eq!(closed.load(Ordering::SeqCst), 2);
}

#[test]
fn pf_27_s01_native_child_service_fixture() {
    let Some(path) = std::env::var_os("CORBANU_BROKER_TEST_SOCKET") else { return; };
    let listener = UnixListener::bind(path).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    let _frame = read_frame(&mut stream).unwrap();
    // Parent kills only this fixture process after dispatch, modeling service
    // death with an actual process and OS-reported peer PID, not a fake error.
    thread::sleep(Duration::from_secs(20));
}

#[test]
fn pf_27_s01_native_service_death_closes_client_without_replay() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("broker.sock");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "linux_transport::tests::pf_27_s01_native_child_service_fixture", "--nocapture"])
        .env("CORBANU_BROKER_TEST_SOCKET", &path)
        .spawn().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let stream = loop {
        if let Ok(stream) = UnixStream::connect(&path) { break stream; }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill(); let _ = child.wait(); panic!("fixture socket unavailable");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let expected = ObservedPeer::from_os(format!("uid:{}", nix::unistd::getuid()), child.id()).unwrap();
    let channel = Arc::new(LinuxBrokerChannel::new(stream, &expected).unwrap());
    let dispatch_channel = channel.clone();
    let dispatch = thread::spawn(move || dispatch_channel.dispatch(&frame()));
    thread::sleep(Duration::from_millis(50));
    child.kill().unwrap();
    child.wait().unwrap();
    assert_eq!(dispatch.join().unwrap(), Err(BrokerDispatchError::OutcomeUnknown));
    assert_eq!(channel.dispatch(&frame()), Err(BrokerDispatchError::SessionUnavailable));
}
