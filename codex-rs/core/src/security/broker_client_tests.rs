use super::*;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

const REFERENCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[derive(Default)]
struct TestTransport {
    dispatches: AtomicUsize,
    closes: AtomicUsize,
}

#[derive(Default)]
struct UnavailableTransport {
    dispatches: AtomicUsize,
    closes: AtomicUsize,
}

impl BrokerClientTransport for UnavailableTransport {
    fn dispatch(
        &self,
        _frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        self.dispatches.fetch_add(1, Ordering::SeqCst);
        Err(BrokerDispatchError::PlatformUnavailable)
    }

    fn close(&self, _binding: &BrokerBinding) -> Result<(), BrokerDispatchError> {
        self.closes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl BrokerClientTransport for TestTransport {
    fn dispatch(
        &self,
        frame: &SignedBrokerFrame,
    ) -> Result<TypedOperationReceipt, BrokerDispatchError> {
        assert!(!frame.as_bytes().is_empty());
        self.dispatches.fetch_add(1, Ordering::SeqCst);
        Ok(TypedOperationReceipt {
            response_status: 200,
            uploaded_bytes: 1,
            downloaded_bytes: 2,
        })
    }

    fn close(&self, _binding: &BrokerBinding) -> Result<(), BrokerDispatchError> {
        self.closes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn binding() -> BrokerBinding {
    BrokerBinding {
        controller_instance: "controller-1".to_string(),
        worker_instance: "worker-1".to_string(),
        session_id: "session-1".to_string(),
        task_id: "task-1".to_string(),
        run_id: "run-1".to_string(),
        run_generation: 1,
    }
}

#[test]
fn pf_27_s04_client_uses_authenticated_frames_and_closes_without_fallback() {
    let transport = Arc::new(TestTransport::default());
    let client = IsolatedBrokerClient::new(
        binding(),
        CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
        BrokerChannelMac::from_secret([7; 32]),
        transport.clone(),
    )
    .expect("client");
    assert_eq!(
        client
            .dispatch_openai_responses("/v1/responses")
            .expect("receipt")
            .response_status,
        200
    );
    client.close().expect("close");
    assert_eq!(
        client.dispatch_openai_responses("/v1/responses"),
        Err(BrokerClientError::Closed)
    );
    assert_eq!(transport.dispatches.load(Ordering::SeqCst), 1);
    assert_eq!(transport.closes.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s04_client_rejects_unsupported_path_before_transport() {
    let transport = Arc::new(TestTransport::default());
    let client = IsolatedBrokerClient::new(
        binding(),
        CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
        BrokerChannelMac::from_secret([7; 32]),
        transport.clone(),
    )
    .expect("client");
    assert_eq!(
        client.dispatch_openai_responses("/v1/../credentials"),
        Err(BrokerClientError::UnsupportedOperation)
    );
    client
        .dispatch_openai_responses("/v1/responses")
        .expect("first valid sequence remains usable");
    assert_eq!(transport.dispatches.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s04_client_drop_closes_the_generation_bound_channel() {
    let transport = Arc::new(TestTransport::default());
    {
        let _client = IsolatedBrokerClient::new(
            binding(),
            CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
            BrokerChannelMac::from_secret([7; 32]),
            transport.clone(),
        )
        .expect("client");
    }
    assert_eq!(transport.closes.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s04_client_closes_ambiguous_sequence_after_transport_unavailable() {
    let transport = Arc::new(UnavailableTransport::default());
    let client = IsolatedBrokerClient::new(
        binding(),
        CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
        BrokerChannelMac::from_secret([7; 32]),
        transport.clone(),
    )
    .expect("client");

    assert_eq!(
        client.dispatch_openai_responses("/v1/responses"),
        Err(BrokerClientError::Unavailable)
    );
    assert_eq!(
        client.dispatch_openai_responses("/v1/responses"),
        Err(BrokerClientError::Closed)
    );
    assert_eq!(transport.dispatches.load(Ordering::SeqCst), 1);
    assert_eq!(transport.closes.load(Ordering::SeqCst), 1);
}
