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
    SourceBinding::from_trusted_provenance(ContentDigest::of(seed), 1)
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
        binding_for(source_seed, raw, rendered, content, 1),
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
    ThresholdIdentity::new(id("moderate"), 1, ContentDigest::of(b"fixture-thresholds"))
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
            .ingest(SegmentEnvelope::new(target, index as u32, part.clone()), 1)
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
fn pf_34_s04_releases_only_complete_reassembled_content() {
    let target = target_for(BENIGN_SANITIZED, 2);
    let parts = split(BENIGN_SANITIZED, BENIGN_SANITIZED.len() / 2);
    let mut session = session(target.clone(), budget()).unwrap();

    let second = session
        .ingest(SegmentEnvelope::new(&target, 1, parts[1].clone()), 1)
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
        .ingest(SegmentEnvelope::new(&target, 0, parts[0].clone()), 2)
        .unwrap();

    let ScreeningDecision::Release(content) =
        session.finish(Some(verdict(&target, VerdictKind::Allow, 10)), 20, 3)
    else {
        panic!("matching allow verdict should release complete content");
    };
    assert_eq!(content.bytes(), BENIGN_SANITIZED);
    assert_eq!(content.taint(), ContentTaint::Untrusted);
    assert_eq!(content.authority(), ContentAuthority::None);
}

#[test]
fn pf_34_s04_partial_input_never_releases_a_prefix() {
    let target = target_for(BENIGN_SANITIZED, 2);
    let parts = split(BENIGN_SANITIZED, BENIGN_SANITIZED.len() / 2);
    let mut session = session(target.clone(), budget()).unwrap();
    let progress = session
        .ingest(SegmentEnvelope::new(&target, 0, parts[0].clone()), 1)
        .unwrap();

    assert_eq!(progress.complete, false);
    assert_eq!(
        unavailable_reason(session.finish(Some(verdict(&target, VerdictKind::Allow, 10)), 20, 2,)),
        Some(UnavailableReason::PartialSegments)
    );
}

#[test]
fn pf_34_s04_duplicate_segment_is_sticky_against_forced_allow() {
    let target = target_for(BENIGN_SANITIZED, 2);
    let parts = split(BENIGN_SANITIZED, BENIGN_SANITIZED.len() / 2);
    let mut session = session(target.clone(), budget()).unwrap();
    let first = SegmentEnvelope::new(&target, 0, parts[0].clone());
    session.ingest(first.clone(), 1).unwrap();

    assert_eq!(
        session.ingest(first, 2),
        Err(UnavailableReason::DuplicateSegment)
    );
    assert_eq!(
        unavailable_reason(session.finish(Some(verdict(&target, VerdictKind::Allow, 10)), 20, 3,)),
        Some(UnavailableReason::DuplicateSegment)
    );
}

#[test]
fn pf_34_s04_contract_version_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut segment = SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec());
    segment.contract_version += 1;
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, 1),
        Err(UnavailableReason::ContractVersionMismatch)
    );
}

#[test]
fn pf_34_s04_source_binding_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let other_target = ScreeningTarget::new(
        binding_for(
            b"different-source",
            BENIGN_RAW,
            BENIGN_RENDERED,
            BENIGN_SANITIZED,
            1,
        ),
        ContentDigest::of(BENIGN_SANITIZED),
        1,
    )
    .unwrap();
    let segment = SegmentEnvelope::new(&other_target, 0, BENIGN_SANITIZED.to_vec());
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, 1),
        Err(UnavailableReason::SourceBindingMismatch)
    );
}

#[test]
fn pf_34_s04_transformation_version_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let other_target = ScreeningTarget::new(
        binding_for(
            b"pf34-fixture-source:benign-v1",
            BENIGN_RAW,
            BENIGN_RENDERED,
            BENIGN_SANITIZED,
            2,
        ),
        ContentDigest::of(BENIGN_SANITIZED),
        1,
    )
    .unwrap();
    let segment = SegmentEnvelope::new(&other_target, 0, BENIGN_SANITIZED.to_vec());
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, 1),
        Err(UnavailableReason::TransformationBindingMismatch)
    );
}

#[test]
fn pf_34_s04_reassembly_digest_mismatch_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut segment = SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec());
    segment.reassembly_digest = ContentDigest::of(b"different-content");
    let mut session = session(target, budget()).unwrap();

    assert_eq!(
        session.ingest(segment, 1),
        Err(UnavailableReason::ReassemblyDigestMismatch)
    );
}

#[test]
fn pf_34_s04_segment_count_and_index_are_bound() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut wrong_count = SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec());
    wrong_count.count = 2;
    let mut count_session = session(target.clone(), budget()).unwrap();
    assert_eq!(
        count_session.ingest(wrong_count, 1),
        Err(UnavailableReason::SegmentCountMismatch)
    );

    let mut wrong_index = SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec());
    wrong_index.index = 1;
    let mut index_session = session(target, budget()).unwrap();
    assert_eq!(
        index_session.ingest(wrong_index, 1),
        Err(UnavailableReason::SegmentOutOfRange)
    );
}

#[test]
fn pf_34_s04_malformed_empty_segment_is_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut session = session(target.clone(), budget()).unwrap();

    assert_eq!(
        session.ingest(SegmentEnvelope::new(&target, 0, Vec::new()), 1),
        Err(UnavailableReason::EmptySegment)
    );
}

#[test]
fn pf_34_s04_size_and_segment_budgets_fail_closed() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut small_segment_budget = budget();
    small_segment_budget.max_segment_bytes = 4;
    let mut segment_session = session(target.clone(), small_segment_budget).unwrap();
    assert_eq!(
        segment_session.ingest(
            SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
            1,
        ),
        Err(UnavailableReason::SegmentTooLarge)
    );

    let bounded_content = b"123456789012";
    let bounded_target = target_for(bounded_content, 2);
    let bounded_parts = split(bounded_content, 6);
    let mut small_content_budget = budget();
    small_content_budget.max_content_bytes = 8;
    small_content_budget.max_segment_bytes = 8;
    let mut content_session = session(bounded_target.clone(), small_content_budget).unwrap();
    content_session
        .ingest(
            SegmentEnvelope::new(&bounded_target, 0, bounded_parts[0].clone()),
            1,
        )
        .unwrap();
    assert_eq!(
        content_session.ingest(
            SegmentEnvelope::new(&bounded_target, 1, bounded_parts[1].clone()),
            2,
        ),
        Err(UnavailableReason::ContentTooLarge)
    );

    let too_many_target = target_for(BENIGN_SANITIZED, 3);
    let mut few_segments_budget = budget();
    few_segments_budget.max_segments = 2;
    let too_many_session = session(too_many_target.clone(), few_segments_budget).unwrap();
    assert_eq!(
        unavailable_reason(too_many_session.finish(
            Some(verdict(&too_many_target, VerdictKind::Allow, 10)),
            20,
            2,
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
}

#[test]
fn pf_34_s04_timeout_is_sticky_against_late_allow() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut session = session(target.clone(), budget()).unwrap();
    assert_eq!(
        session.ingest(
            SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
            501,
        ),
        Err(UnavailableReason::TimedOut)
    );
    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(&target, VerdictKind::Allow, 500)),
            500,
            500,
        )),
        Some(UnavailableReason::TimedOut)
    );
}

#[test]
fn pf_34_s04_cancellation_erases_segments_and_blocks_allow() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
            1,
        )
        .unwrap();
    session.cancel();

    assert_eq!(
        unavailable_reason(session.finish(Some(verdict(&target, VerdictKind::Allow, 10)), 20, 2,)),
        Some(UnavailableReason::Cancelled)
    );
}

#[test]
fn pf_34_s04_missing_stale_and_future_verdicts_are_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let make_session = || {
        let mut session = session(target.clone(), budget()).unwrap();
        session
            .ingest(
                SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
                1,
            )
            .unwrap();
        session
    };

    assert_eq!(
        unavailable_reason(make_session().finish(None, 200, 2)),
        Some(UnavailableReason::MissingVerdict)
    );
    assert_eq!(
        unavailable_reason(make_session().finish(
            Some(verdict(&target, VerdictKind::Allow, 99)),
            200,
            2,
        )),
        Some(UnavailableReason::StaleVerdict)
    );
    assert_eq!(
        unavailable_reason(make_session().finish(
            Some(verdict(&target, VerdictKind::Allow, 201)),
            200,
            2,
        )),
        Some(UnavailableReason::FutureVerdict)
    );
}

#[test]
fn pf_34_s04_mismatched_verdict_cannot_authorize_release() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let other_target = ScreeningTarget::new(
        binding_for(
            b"different-source",
            BENIGN_RAW,
            BENIGN_RENDERED,
            BENIGN_SANITIZED,
            1,
        ),
        ContentDigest::of(BENIGN_SANITIZED),
        1,
    )
    .unwrap();
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
            1,
        )
        .unwrap();

    assert_eq!(
        unavailable_reason(session.finish(
            Some(verdict(&other_target, VerdictKind::Allow, 10)),
            20,
            2,
        )),
        Some(UnavailableReason::VerdictBindingMismatch)
    );
}

#[test]
fn pf_34_s04_mismatched_model_or_threshold_cannot_authorize_release() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
            1,
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
    let mismatched = ClassifierVerdict::new(target, VerdictKind::Allow, other_identity, 10);

    assert_eq!(
        unavailable_reason(session.finish(Some(mismatched), 20, 2)),
        Some(UnavailableReason::VerdictIdentityMismatch)
    );
}

#[test]
fn pf_34_s04_suspicious_hostile_and_unavailable_all_withhold() {
    let target = target_for(BENIGN_SANITIZED, 1);
    for kind in [
        VerdictKind::Suspicious,
        VerdictKind::Hostile,
        VerdictKind::Unavailable,
    ] {
        let mut session = session(target.clone(), budget()).unwrap();
        session
            .ingest(
                SegmentEnvelope::new(&target, 0, BENIGN_SANITIZED.to_vec()),
                1,
            )
            .unwrap();
        let ScreeningDecision::Withhold(withheld) =
            session.finish(Some(verdict(&target, kind, 10)), 20, 2)
        else {
            panic!("non-allow verdict unexpectedly released content");
        };
        assert_eq!(withheld.kind, kind);
    }
}

#[test]
fn pf_34_s04_corrupt_reassembled_bytes_are_unavailable() {
    let target = target_for(BENIGN_SANITIZED, 1);
    let mut session = session(target.clone(), budget()).unwrap();
    session
        .ingest(
            SegmentEnvelope::new(&target, 0, b"corrupt-but-bound-envelope".to_vec()),
            1,
        )
        .unwrap();

    assert_eq!(
        unavailable_reason(session.finish(Some(verdict(&target, VerdictKind::Allow, 10)), 20, 2,)),
        Some(UnavailableReason::ReassemblyDigestMismatch)
    );
}

#[test]
fn pf_34_s04_cross_segment_attack_is_screened_only_after_reassembly() {
    let target = target_for(CROSS_SEGMENT_SANITIZED, 2);
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

    let ScreeningDecision::Withhold(withheld) =
        session.finish(Some(verdict(&target, VerdictKind::Hostile, 10)), 20, 2)
    else {
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
        SourceBinding::from_trusted_provenance(ContentDigest::of(b"source"), 0),
        Err(ContractError::InvalidVersion)
    );
    assert_eq!(
        ThresholdIdentity::new(id("moderate"), 0, ContentDigest::of(b"config")),
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
        ThresholdIdentity::new(id("moderate"), 1, ContentDigest::from_bytes([0; 32]),),
        Err(ContractError::MissingIdentityDigest)
    );
}

#[test]
fn pf_34_s04_fixture_schema_and_content_hashes_are_frozen() {
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
        ContentDigest::of(CROSS_SEGMENT_SANITIZED).to_hex(),
        "d544d6af194cd4efe7e1203c9ec91d4498a9c20aaf35d09b19446b2b130ce386"
    );
}
