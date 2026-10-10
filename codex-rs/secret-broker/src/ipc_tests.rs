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
    let frame = signer
        .sign(binding(), /*sequence*/ 1, operation())
        .expect("frame");
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
    let frame = signer
        .sign(binding(), /*sequence*/ 1, operation())
        .expect("frame");
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
    for path in [
        "/v1/response body",
        "/v1/response\tbody",
        "/v1/response\u{7f}body",
        "/v1/réponses",
        "/v1/responses\\child",
    ] {
        assert_eq!(
            OpenAiResponsesOperation::new(path),
            Err(BrokerFrameError::UnsupportedOperation),
            "path must use bounded visible ASCII origin-form bytes: {path:?}"
        );
    }
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
            .sign(binding(), /*sequence*/ 1, invalid_operation)
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
        ObservedPeer::from_os("worker principal", /*process_id*/ 1),
        Err(BrokerFrameError::InvalidIdentity)
    );
    assert_eq!(
        ObservedPeer::from_os("worker-uid-501", /*process_id*/ 0),
        Err(BrokerFrameError::InvalidPeer)
    );
}

#[test]
fn pf_27_s04_pf_27_s01_debug_output_is_redacted() {
    let mac = BrokerChannelMac::from_secret(KEY);
    let frame = mac
        .sign(binding(), /*sequence*/ 1, operation())
        .expect("frame");
    assert_eq!(format!("{mac:?}"), "BrokerChannelMac(<redacted>)");
    assert_eq!(format!("{frame:?}"), "SignedBrokerFrame(<authenticated>)");
}

fn provider_request() -> ProviderRequestOperation {
    ProviderRequestOperation::new(
        "api.github.com",
        /*port*/ 443,
        "GET",
        "/user?per_page=1",
    )
    .expect("provider request")
}

fn reference() -> CredentialReference {
    CredentialReference::from_sha256_hex(REFERENCE).expect("reference")
}

#[test]
fn pf_27_s04_pf_27_s01_provider_frame_round_trip_is_typed_and_secret_free() {
    let mac = BrokerChannelMac::from_secret(KEY);
    let frame = mac
        .sign_provider_request(
            binding(),
            /*sequence*/ 3,
            reference(),
            provider_request(),
        )
        .expect("frame");
    let verified = mac.verify_provider_request(&frame).expect("verified");

    assert_eq!(
        verified,
        VerifiedProviderRequest {
            sequence: 3,
            binding: binding(),
            credential: reference(),
            request: provider_request(),
        }
    );
    let wire = String::from_utf8_lossy(frame.as_bytes()).to_ascii_lowercase();
    assert!(!wire.contains("authorization"));
    assert!(!wire.contains("bearer"));
}

#[test]
fn pf_27_s04_pf_27_s01_provider_and_openai_frames_are_domain_separated() {
    let mac = BrokerChannelMac::from_secret(KEY);
    let provider = mac
        .sign_provider_request(
            binding(),
            /*sequence*/ 1,
            reference(),
            provider_request(),
        )
        .expect("provider frame");
    let openai = mac
        .sign(binding(), /*sequence*/ 1, operation())
        .expect("openai frame");

    assert_eq!(
        mac.verify(&provider).err(),
        Some(BrokerFrameError::MalformedFrame)
    );
    assert_eq!(
        mac.verify_provider_request(&openai).err(),
        Some(BrokerFrameError::MalformedFrame)
    );
}

#[test]
fn pf_27_s04_pf_27_s01_provider_frame_rejects_forgery_and_untyped_requests() {
    let frame = BrokerChannelMac::from_secret(KEY)
        .sign_provider_request(
            binding(),
            /*sequence*/ 1,
            reference(),
            provider_request(),
        )
        .expect("frame");
    assert_eq!(
        BrokerChannelMac::from_secret([8; 32])
            .verify_provider_request(&frame)
            .err(),
        Some(BrokerFrameError::AuthenticationFailed)
    );

    let mut tampered = frame.as_bytes().to_vec();
    let index = tampered.len() / 2;
    tampered[index] ^= 1;
    assert_eq!(
        BrokerChannelMac::from_secret(KEY)
            .verify_provider_request(&SignedBrokerFrame::from_bytes(tampered).expect("bounded"))
            .err(),
        Some(BrokerFrameError::AuthenticationFailed)
    );

    for (host, port, method, path) in [
        ("API.GITHUB.COM", 443, "GET", "/user"),
        ("api.github.com", 0, "GET", "/user"),
        ("api.github.com", 443, "CONNECT", "/user"),
        ("api.github.com", 443, "GET", "user"),
        ("api.github.com", 443, "GET", "/user#fragment"),
        ("api.github.com", 443, "GET", "/user name"),
        ("api.github.com/evil", 443, "GET", "/user"),
    ] {
        assert_eq!(
            ProviderRequestOperation::new(host, port, method, path).err(),
            Some(BrokerFrameError::UnsupportedOperation),
            "{host} {port} {method} {path}"
        );
    }
    assert_eq!(
        BrokerChannelMac::from_secret(KEY)
            .sign_provider_request(
                binding(),
                /*sequence*/ 0,
                reference(),
                provider_request()
            )
            .err(),
        Some(BrokerFrameError::InvalidSequence)
    );
}

#[test]
fn pf_33_s02_provider_frame_carries_authenticated_pins() {
    let addrs: Vec<std::net::IpAddr> = vec![
        "93.184.216.34".parse().expect("ipv4"),
        "2606:2800:220:1::248".parse().expect("ipv6"),
    ];
    let pinned = provider_request()
        .with_pinned_addrs(addrs.clone())
        .expect("pinned");
    assert_eq!(pinned.pinned_addrs(), addrs.as_slice());
    let mac = BrokerChannelMac::from_secret(KEY);
    let frame = mac
        .sign_provider_request(binding(), /*sequence*/ 1, reference(), pinned.clone())
        .expect("frame");
    assert_eq!(
        mac.verify_provider_request(&frame)
            .expect("verified")
            .request,
        pinned
    );
    // The pins are inside the MAC: changing one byte of an address fails.
    let wire = frame.as_bytes().to_vec();
    let needle = b"93.184.216.34";
    let at = wire
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("pin on the wire");
    let mut tampered = wire;
    tampered[at] = b'1';
    assert_eq!(
        mac.verify_provider_request(&SignedBrokerFrame::from_bytes(tampered).expect("bounded"))
            .err(),
        Some(BrokerFrameError::AuthenticationFailed)
    );
    // Unpinned frames keep their previous wire shape.
    let unpinned = mac
        .sign_provider_request(
            binding(),
            /*sequence*/ 2,
            reference(),
            provider_request(),
        )
        .expect("frame");
    assert!(!String::from_utf8_lossy(unpinned.as_bytes()).contains("pinned_addrs"));

    assert_eq!(
        provider_request().with_pinned_addrs([]).err(),
        Some(BrokerFrameError::UnsupportedOperation)
    );
    let too_many =
        (0..=MAX_PINNED_ADDRS as u8).map(|octet| std::net::IpAddr::from([93, 184, 216, octet]));
    assert_eq!(
        provider_request().with_pinned_addrs(too_many).err(),
        Some(BrokerFrameError::UnsupportedOperation)
    );
}

#[test]
fn sec_390_pipe_peer_proof_binds_key_challenge_and_both_processes() {
    let mac = BrokerChannelMac::from_secret(KEY);
    let challenge = [3_u8; PIPE_CHALLENGE_BYTES];
    let proof = mac.pipe_peer_proof(&challenge, 10, 20);
    assert_eq!(proof.len(), PIPE_PROOF_BYTES);
    assert!(mac.verify_pipe_peer_proof(&challenge, 10, 20, &proof));

    let other_key = BrokerChannelMac::from_secret([8; 32]);
    assert!(!other_key.verify_pipe_peer_proof(&challenge, 10, 20, &proof));
    assert!(!mac.verify_pipe_peer_proof(&[4; PIPE_CHALLENGE_BYTES], 10, 20, &proof));
    // A relaying process is the broker's client, so the ids differ.
    assert!(!mac.verify_pipe_peer_proof(&challenge, 11, 20, &proof));
    assert!(!mac.verify_pipe_peer_proof(&challenge, 10, 21, &proof));
    assert!(!mac.verify_pipe_peer_proof(&challenge, 10, 20, &proof[..31]));
    assert!(!mac.verify_pipe_peer_proof(&challenge, 10, 20, &[]));
}
