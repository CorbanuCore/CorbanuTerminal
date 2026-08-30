use super::*;
use pretty_assertions::assert_eq;

const BENIGN_RAW: &[u8] =
    include_bytes!("../../../qa/security-levels/ingress-contract/fixtures/benign-v1/raw.txt");
const BENIGN_RENDERED: &[u8] =
    include_bytes!("../../../qa/security-levels/ingress-contract/fixtures/benign-v1/rendered.txt");
const BENIGN_SANITIZED: &[u8] =
    include_bytes!("../../../qa/security-levels/ingress-contract/fixtures/benign-v1/sanitized.txt");
const CROSS_SEGMENT_RAW: &[u8] = include_bytes!(
    "../../../qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/raw.txt"
);
const CROSS_SEGMENT_RENDERED: &[u8] = include_bytes!(
    "../../../qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/rendered.txt"
);
const CROSS_SEGMENT_SANITIZED: &[u8] = include_bytes!(
    "../../../qa/security-levels/ingress-contract/fixtures/cross-segment-hostile-v1/sanitized.txt"
);

fn id(value: &str) -> ContractId {
    ContractId::new(value).expect("test identifier should be valid")
}

fn source(seed: &[u8]) -> SourceBinding {
    SourceBinding::from_trusted_provenance(ContentDigest::of(seed), /*schema_version*/ 1)
        .expect("test source binding should be valid")
}

fn binding_for(
    source_seed: &[u8],
    raw: &[u8],
    rendered: &[u8],
    sanitized: &[u8],
    pipeline_version: u32,
) -> ContentBinding {
    let transformation = TransformationBinding::new(
        id("render-sanitize"),
        pipeline_version,
        ContentDigest::of(raw),
        ContentDigest::of(rendered),
        ContentDigest::of(sanitized),
    )
    .expect("test transformation should be valid");
    ContentBinding::new(source(source_seed), transformation)
}

fn target_for(content: &[u8], count: u32) -> ScreeningTarget {
    let (source_seed, raw, rendered) = if content == CROSS_SEGMENT_SANITIZED {
        (
            b"pf34-fixture-source:cross-segment-hostile-v1".as_slice(),
            CROSS_SEGMENT_RAW,
            CROSS_SEGMENT_RENDERED,
        )
    } else if content == BENIGN_SANITIZED {
        (
            b"pf34-fixture-source:benign-v1".as_slice(),
            BENIGN_RAW,
            BENIGN_RENDERED,
        )
    } else {
        (
            b"pf34-fixture-source:synthetic".as_slice(),
            content,
            content,
        )
    };
    ScreeningTarget::new(
        binding_for(
            source_seed,
            raw,
            rendered,
            content,
            /*pipeline_version*/ 1,
        ),
        ContentDigest::of(content),
        count,
    )
    .expect("test target should be valid")
}

fn budget() -> ScreeningBudget {
    ScreeningBudget {
        max_content_bytes: 1_024,
        max_segment_bytes: 512,
        max_segments: 8,
        max_elapsed_ms: 500,
        max_verdict_age_ms: 100,
    }
}

fn model() -> ModelIdentity {
    ModelIdentity::new(
        id("fixture-detector"),
        id("1.0.0"),
        ContentDigest::of(b"fixture-model-artifact"),
    )
    .expect("test model should be valid")
}

fn threshold() -> ThresholdIdentity {
    ThresholdIdentity::new(
        id("moderate"),
        /*profile_version*/ 1,
        ContentDigest::of(b"fixture-thresholds"),
    )
    .expect("test threshold should be valid")
}

fn identity() -> VerdictIdentity {
    VerdictIdentity::new(model(), threshold())
}

fn verdict(target: &ScreeningTarget, kind: VerdictKind, issued_at_ms: u64) -> ClassifierVerdict {
    ClassifierVerdict::new(target.clone(), kind, identity(), issued_at_ms)
}

fn session(
    target: ScreeningTarget,
    budget: ScreeningBudget,
) -> Result<ScreeningSession, ContractError> {
    ScreeningSession::new(target, budget, identity())
}

fn split(content: &[u8], at: usize) -> [Vec<u8>; 2] {
    [content[..at].to_vec(), content[at..].to_vec()]
}

fn ingest_all(session: &mut ScreeningSession, target: &ScreeningTarget, parts: &[Vec<u8>]) {
    for (index, part) in parts.iter().enumerate() {
        session
            .ingest(
                SegmentEnvelope::new(target, index as u32, part.clone()),
                /*elapsed_ms*/ 1,
            )
            .expect("test segment should be accepted");
    }
}

fn unavailable_reason(decision: ScreeningDecision) -> Option<UnavailableReason> {
    match decision {
        ScreeningDecision::Withhold(withheld) => withheld.reason,
        ScreeningDecision::Release(_) => panic!("content unexpectedly released"),
    }
}

#[test]
fn pf_34_s04_content_digest_hex_encoding_is_stable() {
    assert_eq!(
        ContentDigest::from_bytes([0x00; 32]).to_hex(),
        "00".repeat(32)
    );
    assert_eq!(
        ContentDigest::from_bytes([0xff; 32]).to_hex(),
        "ff".repeat(32)
    );

    let mut ascending = [0_u8; 32];
    for (value, byte) in ascending.iter_mut().enumerate() {
        *byte = value as u8;
    }
    assert_eq!(
        ContentDigest::from_bytes(ascending).to_hex(),
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
    );
}

#[test]
fn pf_34_s04_releases_only_complete_reassembled_content() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 2);
    let parts = split(BENIGN_SANITIZED, BENIGN_SANITIZED.len() / 2);
    let debug_segment = SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    assert_eq!(
        format!("{debug_segment:?}"),
        "SegmentEnvelope { len: 99, sha256: \"c26af282d0752aa49a5fbba56c6151a7091ccb9f2018f78e3115af2bb685ce5f\" }"
    );
    let mut session = session(target.clone(), budget()).unwrap();

    let second = session
        .ingest(
            SegmentEnvelope::new(&target, /*index*/ 1, parts[1].clone()),
            /*elapsed_ms*/ 1,
        )
        .unwrap();
    assert_eq!(
        second,
        ScreeningProgress {
            received_segments: 1,
            expected_segments: 2,
            received_bytes: parts[1].len(),
            complete: false,
        }
    );
    session
        .ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, parts[0].clone()),
            /*elapsed_ms*/ 2,
        )
        .unwrap();

    let ScreeningDecision::Release(content) = session.finish(
        Some(verdict(
            &target,
            VerdictKind::Allow,
            /*issued_at_ms*/ 10,
        )),
        /*now_ms*/ 20,
        /*elapsed_ms*/ 3,
    ) else {
        panic!("matching allow verdict should release complete content");
    };
    assert_eq!(
        format!("{:?}", content.bytes()),
        "UntrustedBytes { len: 99, sha256: \"c26af282d0752aa49a5fbba56c6151a7091ccb9f2018f78e3115af2bb685ce5f\" }"
    );
    assert_eq!(
        format!("{content:?}"),
        "ScreenedContent { len: 99, sha256: \"c26af282d0752aa49a5fbba56c6151a7091ccb9f2018f78e3115af2bb685ce5f\" }"
    );
    assert_eq!(content.bytes().into_raw_untrusted(), BENIGN_SANITIZED);
    assert_eq!(content.taint(), ContentTaint::Untrusted);
    assert_eq!(content.authority(), ContentAuthority::None);
    assert_eq!(content.verdict_identity(), &identity());
}

#[test]
fn pf_34_s04_partial_input_never_releases_a_prefix() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 2);
    let parts = split(BENIGN_SANITIZED, BENIGN_SANITIZED.len() / 2);
    let mut session = session(target.clone(), budget()).unwrap();
    let progress = session
        .ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, parts[0].clone()),
            /*elapsed_ms*/ 1,
        )
        .unwrap();

    assert!(!progress.complete);
    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 10
            )),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::PartialSegments)
    );
}

#[test]
fn pf_34_s04_duplicate_segment_is_sticky_against_forced_allow() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 2);
    let parts = split(BENIGN_SANITIZED, BENIGN_SANITIZED.len() / 2);
    let mut session = session(target.clone(), budget()).unwrap();
    let first = SegmentEnvelope::new(&target, /*index*/ 0, parts[0].clone());
    session.ingest(first.clone(), /*elapsed_ms*/ 1).unwrap();

    assert_eq!(
        session.ingest(first, /*elapsed_ms*/ 2),
        Err(UnavailableReason::DuplicateSegment)
    );
    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 10
            )),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 3,
        )),
        Some(UnavailableReason::DuplicateSegment)
    );
}

#[test]
fn pf_34_s04_contract_version_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut segment = SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    segment.contract_version += 1;
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, /*elapsed_ms*/ 1),
        Err(UnavailableReason::ContractVersionMismatch)
    );
}

#[test]
fn pf_34_s04_source_binding_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let other_target = ScreeningTarget::new(
        binding_for(
            b"different-source",
            BENIGN_RAW,
            BENIGN_RENDERED,
            BENIGN_SANITIZED,
            /*pipeline_version*/ 1,
        ),
        ContentDigest::of(BENIGN_SANITIZED),
        /*segment_count*/ 1,
    )
    .unwrap();
    let segment = SegmentEnvelope::new(&other_target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, /*elapsed_ms*/ 1),
        Err(UnavailableReason::SourceBindingMismatch)
    );
}

#[test]
fn pf_34_s04_transformation_version_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let other_target = ScreeningTarget::new(
        binding_for(
            b"pf34-fixture-source:benign-v1",
            BENIGN_RAW,
            BENIGN_RENDERED,
            BENIGN_SANITIZED,
            /*pipeline_version*/ 2,
        ),
        ContentDigest::of(BENIGN_SANITIZED),
        /*segment_count*/ 1,
    )
    .unwrap();
    let segment = SegmentEnvelope::new(&other_target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, /*elapsed_ms*/ 1),
        Err(UnavailableReason::TransformationBindingMismatch)
    );
}

#[test]
fn pf_34_s04_reassembly_digest_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut segment = SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    segment.reassembly_digest = ContentDigest::of(b"different-content");
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, /*elapsed_ms*/ 1),
        Err(UnavailableReason::ReassemblyDigestMismatch)
    );
}

#[test]
fn pf_34_s04_segment_count_and_index_are_bound() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut wrong_count =
        SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    wrong_count.count = 2;
    let mut count_session = session(target.clone(), budget()).unwrap();
    assert_eq!(
        count_session.ingest(wrong_count, /*elapsed_ms*/ 1),
        Err(UnavailableReason::SegmentCountMismatch)
    );

    let mut wrong_index =
        SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec());
    wrong_index.index = 1;
    let mut index_session = session(target, budget()).unwrap();
    assert_eq!(
        index_session.ingest(wrong_index, /*elapsed_ms*/ 1),
        Err(UnavailableReason::SegmentOutOfRange)
    );
}

#[test]
fn pf_34_s04_malformed_empty_segment_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut session = session(target.clone(), budget()).unwrap();

    assert_eq!(
        session.ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, Vec::new()),
            /*elapsed_ms*/ 1
        ),
        Err(UnavailableReason::EmptySegment)
    );
}

#[test]
fn pf_34_s04_size_and_segment_budgets_fail_closed() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut small_segment_budget = budget();
    small_segment_budget.max_segment_bytes = 4;
    let mut segment_session = session(target.clone(), small_segment_budget).unwrap();
    assert_eq!(
        segment_session.ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
            /*elapsed_ms*/ 1,
        ),
        Err(UnavailableReason::SegmentTooLarge)
    );

    let bounded_content = b"123456789012";
    let bounded_target = target_for(bounded_content, /*count*/ 2);
    let bounded_parts = split(bounded_content, /*at*/ 6);
    let mut small_content_budget = budget();
    small_content_budget.max_content_bytes = 8;
    small_content_budget.max_segment_bytes = 8;
    let mut content_session = session(bounded_target.clone(), small_content_budget).unwrap();
    content_session
        .ingest(
            SegmentEnvelope::new(&bounded_target, /*index*/ 0, bounded_parts[0].clone()),
            /*elapsed_ms*/ 1,
        )
        .unwrap();
    assert_eq!(
        content_session.ingest(
            SegmentEnvelope::new(&bounded_target, /*index*/ 1, bounded_parts[1].clone()),
            /*elapsed_ms*/ 2,
        ),
        Err(UnavailableReason::ContentTooLarge)
    );

    let too_many_target = target_for(BENIGN_SANITIZED, /*count*/ 3);
    let mut few_segments_budget = budget();
    few_segments_budget.max_segments = 2;
    let too_many_session = session(too_many_target.clone(), few_segments_budget).unwrap();
    assert_eq!(
        unavailable_reason(too_many_session.finish(
            Some(verdict(
                &too_many_target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 10
            )),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::TooManySegments)
    );

    let mut unsafe_budget = budget();
    unsafe_budget.max_content_bytes = MAX_SCREENED_CONTENT_BYTES + 1;
    unsafe_budget.max_segment_bytes = MAX_SCREENED_CONTENT_BYTES + 1;
    assert_eq!(unsafe_budget.validate(), Err(ContractError::InvalidBudget));
    unsafe_budget = budget();
    unsafe_budget.max_segments = MAX_SCREENING_SEGMENTS + 1;
    assert_eq!(unsafe_budget.validate(), Err(ContractError::InvalidBudget));
    unsafe_budget = budget();
    unsafe_budget.max_elapsed_ms = MAX_SCREENING_ELAPSED_MS + 1;
    assert_eq!(unsafe_budget.validate(), Err(ContractError::InvalidBudget));
    unsafe_budget = budget();
    unsafe_budget.max_verdict_age_ms = MAX_VERDICT_AGE_MS + 1;
    assert_eq!(unsafe_budget.validate(), Err(ContractError::InvalidBudget));

    let binding = binding_for(
        b"pf34-fixture-source:benign-v1",
        BENIGN_RAW,
        BENIGN_RENDERED,
        BENIGN_SANITIZED,
        /*pipeline_version*/ 1,
    );
    assert_eq!(
        ScreeningTarget::new(
            binding,
            ContentDigest::of(BENIGN_SANITIZED),
            MAX_SCREENING_SEGMENTS + 1,
        ),
        Err(ContractError::InvalidSegmentCount)
    );
}

#[test]
fn pf_34_s04_timeout_is_sticky_against_late_allow() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut session = session(target.clone(), budget()).unwrap();
    assert_eq!(
        session.ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
            /*elapsed_ms*/ 501,
        ),
        Err(UnavailableReason::TimedOut)
    );
    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 500
            )),
            /*now_ms*/ 500,
            /*elapsed_ms*/ 500,
        )),
        Some(UnavailableReason::TimedOut)
    );
}

#[test]
fn pf_34_s04_cancellation_erases_segments_and_blocks_allow() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
            /*elapsed_ms*/ 1,
        )
        .unwrap();
    session.cancel();
    assert_eq!(session.received_segments, 0);
    assert_eq!(session.received_bytes, 0);

    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 10
            )),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::Cancelled)
    );
}

#[test]
fn pf_34_s04_missing_stale_and_future_verdicts_are_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let make_session = || {
        let mut session = session(target.clone(), budget()).unwrap();
        session
            .ingest(
                SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
                /*elapsed_ms*/ 1,
            )
            .unwrap();
        session
    };

    assert_eq!(
        unavailable_reason(make_session().finish(
            /*verdict*/ None, /*now_ms*/ 200, /*elapsed_ms*/ 2
        )),
        Some(UnavailableReason::MissingVerdict)
    );
    assert_eq!(
        unavailable_reason(make_session().finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 99
            )),
            /*now_ms*/ 200,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::StaleVerdict)
    );
    assert_eq!(
        unavailable_reason(make_session().finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 201
            )),
            /*now_ms*/ 200,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::FutureVerdict)
    );
}

#[test]
fn pf_34_s04_mismatched_verdict_cannot_authorize_release() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let other_target = ScreeningTarget::new(
        binding_for(
            b"different-source",
            BENIGN_RAW,
            BENIGN_RENDERED,
            BENIGN_SANITIZED,
            /*pipeline_version*/ 1,
        ),
        ContentDigest::of(BENIGN_SANITIZED),
        /*segment_count*/ 1,
    )
    .unwrap();
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
            /*elapsed_ms*/ 1,
        )
        .unwrap();

    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(
                &other_target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 10
            )),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::VerdictBindingMismatch)
    );
}

#[test]
fn pf_34_s04_mismatched_model_or_threshold_cannot_authorize_release() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
            /*elapsed_ms*/ 1,
        )
        .unwrap();
    let other_identity = VerdictIdentity::new(
        ModelIdentity::new(
            id("different-detector"),
            id("1.0.0"),
            ContentDigest::of(b"different-artifact"),
        )
        .unwrap(),
        threshold(),
    );
    let mismatched = ClassifierVerdict::new(
        target,
        VerdictKind::Allow,
        other_identity,
        /*issued_at_ms*/ 10,
    );

    assert_eq!(
        unavailable_reason(session.finish(
            Some(mismatched),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2
        )),
        Some(UnavailableReason::VerdictIdentityMismatch)
    );
}

#[test]
fn pf_34_s04_suspicious_hostile_and_unavailable_all_withhold() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    for kind in [
        VerdictKind::Suspicious,
        VerdictKind::Hostile,
        VerdictKind::Unavailable,
    ] {
        let mut session = session(target.clone(), budget()).unwrap();
        session
            .ingest(
                SegmentEnvelope::new(&target, /*index*/ 0, BENIGN_SANITIZED.to_vec()),
                /*elapsed_ms*/ 1,
            )
            .unwrap();
        let ScreeningDecision::Withhold(withheld) = session.finish(
            Some(verdict(&target, kind, /*issued_at_ms*/ 10)),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2,
        ) else {
            panic!("non-allow verdict unexpectedly released content");
        };
        assert_eq!(withheld.kind, kind);
    }
}

#[test]
fn pf_34_s04_corrupt_reassembled_bytes_are_unavailable() {
    let target = target_for(BENIGN_SANITIZED, /*count*/ 1);
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(
                &target,
                /*index*/ 0,
                b"corrupt-but-bound-envelope".to_vec(),
            ),
            /*elapsed_ms*/ 1,
        )
        .unwrap();

    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(
                &target,
                VerdictKind::Allow,
                /*issued_at_ms*/ 10
            )),
            /*now_ms*/ 20,
            /*elapsed_ms*/ 2,
        )),
        Some(UnavailableReason::ReassemblyDigestMismatch)
    );
}

#[test]
fn pf_34_s04_cross_segment_attack_is_screened_only_after_reassembly() {
    let target = target_for(CROSS_SEGMENT_SANITIZED, /*count*/ 2);
    let marker = CROSS_SEGMENT_SANITIZED
        .windows(b"ignore previous".len())
        .position(|window| window == b"ignore previous")
        .expect("fixture should contain cross-segment marker");
    let split_at = marker + b"ignore ".len();
    let parts = split(CROSS_SEGMENT_SANITIZED, split_at);
    assert!(
        !parts[0]
            .windows(b"ignore previous".len())
            .any(|window| window == b"ignore previous")
    );
    assert!(
        !parts[1]
            .windows(b"ignore previous".len())
            .any(|window| window == b"ignore previous")
    );
    let mut session = session(target.clone(), budget()).unwrap();
    ingest_all(&mut session, &target, &parts);

    let ScreeningDecision::Withhold(withheld) = session.finish(
        Some(verdict(
            &target,
            VerdictKind::Hostile,
            /*issued_at_ms*/ 10,
        )),
        /*now_ms*/ 20,
        /*elapsed_ms*/ 2,
    ) else {
        panic!("hostile cross-segment fixture unexpectedly released");
    };
    assert_eq!(withheld.kind, VerdictKind::Hostile);
}

#[test]
fn pf_34_s04_identifiers_and_versions_reject_malformed_metadata() {
    assert_eq!(ContractId::new(""), Err(ContractError::InvalidIdentifier));
    assert_eq!(
        ContractId::new("attacker supplied\ntext"),
        Err(ContractError::InvalidIdentifier)
    );
    assert_eq!(
        SourceBinding::from_trusted_provenance(
            ContentDigest::of(b"source"),
            /*schema_version*/ 0
        ),
        Err(ContractError::InvalidVersion)
    );
    assert_eq!(
        ThresholdIdentity::new(
            id("moderate"),
            /*profile_version*/ 0,
            ContentDigest::of(b"config")
        ),
        Err(ContractError::InvalidVersion)
    );
    assert_eq!(
        ModelIdentity::new(
            id("fixture-detector"),
            id("1.0.0"),
            ContentDigest::from_bytes([0; 32]),
        ),
        Err(ContractError::MissingIdentityDigest)
    );
    assert_eq!(
        ThresholdIdentity::new(
            id("moderate"),
            /*profile_version*/ 1,
            ContentDigest::from_bytes([0; 32]),
        ),
        Err(ContractError::MissingIdentityDigest)
    );
}

#[test]
fn pf_34_s04_fixture_schema_and_content_hashes_are_frozen() {
    assert_eq!(MAX_SCREENED_CONTENT_BYTES, 67_108_864);
    assert_eq!(MAX_SCREENING_SEGMENTS, 4_096);
    assert_eq!(MAX_SCREENING_ELAPSED_MS, 3_600_000);
    assert_eq!(MAX_VERDICT_AGE_MS, 300_000);
    assert_eq!(SCREENING_FIXTURE_SCHEMA_VERSION, 1);
    assert_eq!(
        ContentDigest::of(BENIGN_RAW).to_hex(),
        "b64866d85fdce7ac143f6940497c5d9851c7dc8fcf35e8566f53d6a70e342002"
    );
    assert_eq!(
        ContentDigest::of(BENIGN_RENDERED).to_hex(),
        "870b15813879f5c002c628d97a19728348cd22627f7129e737fa3f492408ab77"
    );
    assert_eq!(
        ContentDigest::of(BENIGN_SANITIZED).to_hex(),
        "c26af282d0752aa49a5fbba56c6151a7091ccb9f2018f78e3115af2bb685ce5f"
    );
    assert_eq!(
        ContentDigest::of(CROSS_SEGMENT_RAW).to_hex(),
        "50584c7fd9d80bf166f5f1b1a2236ad33e02aec53fd2dcb55937ed00737fb410"
    );
    assert_eq!(
        ContentDigest::of(CROSS_SEGMENT_RENDERED).to_hex(),
        "d4b9c4fe86395844ff4b4fd4c75d0a90575baf6f37b21c11bc930ce82370fe07"
    );
    assert_eq!(
        ContentDigest::of(CROSS_SEGMENT_SANITIZED).to_hex(),
        "d544d6af194cd4efe7e1203c9ec91d4498a9c20aaf35d09b19446b2b130ce386"
    );
    assert_eq!(
        source(b"pf34-fixture-source:benign-v1")
            .opaque_id()
            .to_hex(),
        "5ae8798cfea72dc9b41953f59fb302eae50ff71da5b10538bd8f9ec678aefd62"
    );
    assert_eq!(
        source(b"pf34-fixture-source:cross-segment-hostile-v1")
            .opaque_id()
            .to_hex(),
        "35ee161076c896f0767b96409664ce12741ce8d3aff1ec043280e7f21a113fc1"
    );
}
