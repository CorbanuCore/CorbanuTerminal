use codex_secret_broker::*;
use codex_secret_broker::linux_transport::*;
use pretty_assertions::assert_eq;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::Child;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

struct ChildService {
    child: Child,
    channel: LinuxBrokerChannel,
    output: BufReader<ChildStdout>,
}
impl ChildService {
    fn start(key: u8) -> Self {
        let (mut client, server) = UnixStream::pair().unwrap();
        let peer = observed_peer(&client).unwrap();
        // Inherited socketpair peer is the creator, not the executed child.
        assert_eq!(peer.process_id(), std::process::id());
        let mut child = Command::new(codex_utils_cargo_bin::cargo_bin("codex-secret-broker-service-fixture").unwrap())
            .arg("--synthetic-inherited-socket")
            .stdin(Stdio::from(OwnedFd::from(server)))
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        client.write_all(&[key; 32]).unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut ready = String::new();
        output.read_line(&mut ready).unwrap();
        assert_eq!(ready, "synthetic-only ready\n");
        Self { child, channel: LinuxBrokerChannel::new(client, &peer).unwrap(), output }
    }
}
impl Drop for ChildService {
    fn drop(&mut self) {
        let _ = self.channel.close();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn frame(key: u8, sequence: u64) -> SignedBrokerFrame {
    BrokerChannelMac::from_secret([key; 32]).sign(BrokerBinding { controller_instance: "controller".into(), worker_instance: "worker".into(), session_id: "session".into(), task_id: "task".into(), run_id: "run".into(), run_generation: 1 }, sequence, BrokerOperation::OpenAiResponses { credential: CredentialReference::from_sha256_hex("a".repeat(64)).unwrap(), request: OpenAiResponsesOperation::new("/v1/responses").unwrap() }).unwrap()
}

#[test]
fn pf_27_s01_subprocess_dispatch_uses_vault_and_settles_journal() {
    let mut service = ChildService::start(9);
    assert_eq!(service.channel.dispatch(&frame(9, 1)).unwrap(), TypedOperationReceipt { response_status: 204, uploaded_bytes: 0, downloaded_bytes: 0 });
    service.channel.close().unwrap();
    let mut output = String::new();
    service.output.read_to_string(&mut output).unwrap();
    assert_eq!(output, "synthetic-only journal-records=2\n");
    assert!(service.child.wait().unwrap().success());
    assert!(!output.contains("BROKER-SERVICE-SYNTHETIC-ONLY"));
}

#[test]
fn pf_27_s01_subprocess_death_and_restart_refuse_old_channel_and_key() {
    let mut old = ChildService::start(9);
    assert!(old.channel.dispatch(&frame(9, 1)).is_ok());
    old.child.kill().unwrap();
    old.child.wait().unwrap();
    assert!(old.channel.dispatch(&frame(9, 2)).is_err());
    assert_eq!(old.channel.dispatch(&frame(9, 2)), Err(BrokerDispatchError::SessionUnavailable));
    let restarted = ChildService::start(8);
    assert!(restarted.channel.dispatch(&frame(9, 1)).is_err());
    let fresh = ChildService::start(7);
    assert!(fresh.channel.dispatch(&frame(7, 1)).is_ok());
}

#[test]
fn pf_27_s01_subprocess_replay_denies_without_second_audit_or_secret() {
    let mut service = ChildService::start(9);
    assert!(service.channel.dispatch(&frame(9, 1)).is_ok());
    assert!(service.channel.dispatch(&frame(9, 1)).is_err());
    let mut output = String::new();
    service.output.read_to_string(&mut output).unwrap();
    assert_eq!(output, "synthetic-only journal-records=2\n");
    assert_eq!(service.child.wait().unwrap().code(), Some(78));
}
