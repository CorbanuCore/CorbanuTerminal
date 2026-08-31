use crate::ipc::*;
use pretty_assertions::assert_eq;

const KEY: [u8; 32] = [7; 32];
const REFERENCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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

fn operation() -> BrokerOperation {
    BrokerOperation::OpenAiResponses {
        credential: CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
        request: OpenAiResponsesOperation::new("/v1/responses").expect("operation"),
    }
}

#[test]
fn pf_27_s04_pf_27_s01_authenticated_frame_round_trip_preserves_only_typed_metadata() {
    let signer = BrokerChannelMac::from_secret(KEY);
    let verifier = BrokerChannelMac::from_secret(KEY);
    let frame = signer.sign(binding(), 1, operation()).expect("frame");
    let verified = verifier.verify(&frame).expect("verified");

    assert_eq!(verified.binding, binding());
    assert_eq!(verified.sequence, 1);
    assert_eq!(verified.operation, operation());
    let wire = String::from_utf8_lossy(frame.as_bytes());
    assert!(!wire.contains("secret"));
    assert!(!wire.contains("authorization"));
}

#[test]
fn pf_27_s04_pf_27_s01_wrong_key_and_tampering_fail_authentication() {
    let signer = BrokerChannelMac::from_secret(KEY);
    let wrong = BrokerChannelMac::from_secret([8; 32]);
    let frame = signer.sign(binding(), 1, operation()).expect("frame");
    assert_eq!(
        wrong.verify(&frame).err(),
        Some(BrokerFrameError::AuthenticationFailed)
    );

    let mut tampered = frame.as_bytes().to_vec();
    let index = tampered.len() / 2;
    tampered[index] ^= 1;
    let tampered = SignedBrokerFrame::from_bytes(tampered).expect("bounded frame");
    assert_eq!(
        signer.verify(&tampered).err(),
        Some(BrokerFrameError::AuthenticationFailed)
    );
}

#[test]
fn pf_27_s04_pf_27_s01_bounds_and_operation_shape_fail_closed() {
    assert_eq!(
        OpenAiResponsesOperation::new("http://api.openai.com/v1/responses"),
        Err(BrokerFrameError::UnsupportedOperation)
    );
    assert_eq!(
        OpenAiResponsesOperation::new("/v1/../credentials"),
        Err(BrokerFrameError::UnsupportedOperation)
    );
    assert_eq!(
        OpenAiResponsesOperation::new("/v1/responses?token=x"),
        Err(BrokerFrameError::UnsupportedOperation)
    );
    assert_eq!(
        CredentialReference::from_sha256_hex("A".repeat(64)),
        Err(BrokerFrameError::InvalidCredentialReference)
    );
    assert_eq!(
        SignedBrokerFrame::from_bytes(vec![0; MAX_FRAME_BYTES + 1]).err(),
        Some(BrokerFrameError::FrameTooLarge)
    );

    // Serde is part of the wire boundary. Even a locally deserialized opaque
    // reference must be revalidated before it can be authenticated.
    let invalid_reference: CredentialReference =
        serde_json::from_str("\"not-a-sha256-reference\"").expect("wire reference");
    let invalid_operation = BrokerOperation::OpenAiResponses {
        credential: invalid_reference,
        request: OpenAiResponsesOperation::new("/v1/responses").expect("operation"),
    };
    assert_eq!(
        BrokerChannelMac::from_secret(KEY)
            .sign(binding(), 1, invalid_operation)
            .err(),
        Some(BrokerFrameError::InvalidCredentialReference)
    );
}

#[test]
fn pf_27_s04_pf_27_s01_binding_and_peer_require_canonical_observed_identity() {
    let mut invalid = binding();
    invalid.run_generation = 0;
    assert_eq!(
        invalid.validate(),
        Err(BrokerFrameError::InvalidRunGeneration)
    );
    assert_eq!(
        ObservedPeer::from_os("worker principal", 1),
        Err(BrokerFrameError::InvalidIdentity)
    );
    assert_eq!(
        ObservedPeer::from_os("worker-uid-501", 0),
        Err(BrokerFrameError::InvalidPeer)
    );
}

#[test]
fn pf_27_s04_pf_27_s01_debug_output_is_redacted() {
    let mac = BrokerChannelMac::from_secret(KEY);
    let frame = mac.sign(binding(), 1, operation()).expect("frame");
    assert_eq!(format!("{mac:?}"), "BrokerChannelMac(<redacted>)");
    assert_eq!(format!("{frame:?}"), "SignedBrokerFrame(<authenticated>)");
}
