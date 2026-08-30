//! Fail-closed contract between content sanitization, classification, and quarantine.
//!
//! This module is deliberately free of Core, provider, tool, and policy adapters.
//! It binds opaque provenance supplied by the provenance owner to complete,
//! transformed content and releases bytes only after a fresh, matching verdict.
//!
//! Verdicts are in-process values, not authenticated messages. Any future wire
//! format must deserialize through validating `TryFrom` implementations and
//! the constructors below; deriving `Deserialize` directly on these types would
//! bypass their invariants. Callers must provide monotonic elapsed time. Buffered
//! external content is dropped, but not zeroized, on failure or cancellation.
//! Public enums are deliberately exhaustive in contract v1; adding a variant
//! requires a contract-version bump rather than an implicit behavior change.

use sha2::Digest as _;
use sha2::Sha256;
use std::fmt;

pub const SCREENING_CONTRACT_VERSION: u32 = 1;
pub const SCREENING_FIXTURE_SCHEMA_VERSION: u32 = 1;
pub const MAX_SCREENED_CONTENT_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_SCREENING_SEGMENTS: u32 = 4_096;
pub const MAX_SCREENING_ELAPSED_MS: u64 = 60 * 60 * 1_000;
pub const MAX_VERDICT_AGE_MS: u64 = 5 * 60 * 1_000;
const MAX_ID_BYTES: usize = 128;

/// A SHA-256 identity used to bind content and immutable configuration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ContentDigest([u8; 32]);

impl ContentDigest {
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    const fn is_zero(self) -> bool {
        let mut index = 0;
        while index < self.0.len() {
            if self.0[index] != 0 {
                return false;
            }
            index += 1;
        }
        true
    }

    pub fn to_hex(self) -> String {
        const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut hex = String::with_capacity(64);
        for byte in self.0 {
            hex.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
            hex.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
        }
        hex
    }
}

/// A bounded, display-safe identifier that cannot carry control characters.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ContractId(String);

impl ContractId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_ID_BYTES
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(ContractError::InvalidIdentifier);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Opaque provenance identity supplied by PF-30; this crate never derives it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceBinding {
    opaque_id: ContentDigest,
    schema_version: u32,
}

impl SourceBinding {
    pub fn from_trusted_provenance(
        opaque_id: ContentDigest,
        schema_version: u32,
    ) -> Result<Self, ContractError> {
        if schema_version == 0 || opaque_id.is_zero() {
            return Err(ContractError::InvalidVersion);
        }
        Ok(Self {
            opaque_id,
            schema_version,
        })
    }

    pub const fn opaque_id(&self) -> ContentDigest {
        self.opaque_id
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

/// Immutable transformation chain from raw input through rendering to sanitization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformationBinding {
    pipeline_id: ContractId,
    pipeline_version: u32,
    raw_digest: ContentDigest,
    rendered_digest: ContentDigest,
    sanitized_digest: ContentDigest,
}

impl TransformationBinding {
    pub fn new(
        pipeline_id: ContractId,
        pipeline_version: u32,
        raw_digest: ContentDigest,
        rendered_digest: ContentDigest,
        sanitized_digest: ContentDigest,
    ) -> Result<Self, ContractError> {
        if pipeline_version == 0
            || raw_digest.is_zero()
            || rendered_digest.is_zero()
            || sanitized_digest.is_zero()
        {
            return Err(ContractError::InvalidVersion);
        }
        Ok(Self {
            pipeline_id,
            pipeline_version,
            raw_digest,
            rendered_digest,
            sanitized_digest,
        })
    }

    pub fn pipeline_id(&self) -> &ContractId {
        &self.pipeline_id
    }

    pub const fn pipeline_version(&self) -> u32 {
        self.pipeline_version
    }

    pub const fn raw_digest(&self) -> ContentDigest {
        self.raw_digest
    }

    pub const fn rendered_digest(&self) -> ContentDigest {
        self.rendered_digest
    }

    pub const fn sanitized_digest(&self) -> ContentDigest {
        self.sanitized_digest
    }
}

/// Complete immutable identity for one sanitized content item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentBinding {
    contract_version: u32,
    source: SourceBinding,
    transformation: TransformationBinding,
}

impl ContentBinding {
    pub fn new(source: SourceBinding, transformation: TransformationBinding) -> Self {
        Self {
            contract_version: SCREENING_CONTRACT_VERSION,
            source,
            transformation,
        }
    }

    pub const fn contract_version(&self) -> u32 {
        self.contract_version
    }

    pub const fn source(&self) -> SourceBinding {
        self.source
    }

    pub const fn transformation(&self) -> &TransformationBinding {
        &self.transformation
    }
}

/// Classifier artifact identity; an ID without its artifact digest is insufficient.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelIdentity {
    model_id: ContractId,
    model_version: ContractId,
    artifact_digest: ContentDigest,
}

impl ModelIdentity {
    pub fn new(
        model_id: ContractId,
        model_version: ContractId,
        artifact_digest: ContentDigest,
    ) -> Result<Self, ContractError> {
        if artifact_digest.is_zero() {
            return Err(ContractError::MissingIdentityDigest);
        }
        Ok(Self {
            model_id,
            model_version,
            artifact_digest,
        })
    }

    pub fn model_id(&self) -> &ContractId {
        &self.model_id
    }

    pub fn model_version(&self) -> &ContractId {
        &self.model_version
    }

    pub const fn artifact_digest(&self) -> ContentDigest {
        self.artifact_digest
    }
}

/// Threshold profile identity; profile name, version, and bytes are all bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThresholdIdentity {
    profile_id: ContractId,
    profile_version: u32,
    config_digest: ContentDigest,
}

impl ThresholdIdentity {
    pub fn new(
        profile_id: ContractId,
        profile_version: u32,
        config_digest: ContentDigest,
    ) -> Result<Self, ContractError> {
        if profile_version == 0 {
            return Err(ContractError::InvalidVersion);
        }
        if config_digest.is_zero() {
            return Err(ContractError::MissingIdentityDigest);
        }
        Ok(Self {
            profile_id,
            profile_version,
            config_digest,
        })
    }

    pub fn profile_id(&self) -> &ContractId {
        &self.profile_id
    }

    pub const fn profile_version(&self) -> u32 {
        self.profile_version
    }

    pub const fn config_digest(&self) -> ContentDigest {
        self.config_digest
    }
}

/// Exact classifier artifact and threshold configuration required by policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerdictIdentity {
    model: ModelIdentity,
    threshold: ThresholdIdentity,
}

impl VerdictIdentity {
    pub fn new(model: ModelIdentity, threshold: ThresholdIdentity) -> Self {
        Self { model, threshold }
    }

    pub const fn model(&self) -> &ModelIdentity {
        &self.model
    }

    pub const fn threshold(&self) -> &ThresholdIdentity {
        &self.threshold
    }
}

/// Bounded resource and freshness limits selected by deterministic policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreeningBudget {
    pub max_content_bytes: usize,
    pub max_segment_bytes: usize,
    pub max_segments: u32,
    pub max_elapsed_ms: u64,
    pub max_verdict_age_ms: u64,
}

impl ScreeningBudget {
    pub fn validate(self) -> Result<Self, ContractError> {
        let max_segments =
            usize::try_from(self.max_segments).map_err(|_| ContractError::InvalidBudget)?;
        if self.max_content_bytes == 0
            || self.max_segment_bytes == 0
            || self.max_segment_bytes > self.max_content_bytes
            || self.max_content_bytes > MAX_SCREENED_CONTENT_BYTES
            || self.max_segments == 0
            || self.max_segments > MAX_SCREENING_SEGMENTS
            || max_segments > self.max_content_bytes
            || self.max_elapsed_ms == 0
            || self.max_elapsed_ms > MAX_SCREENING_ELAPSED_MS
            || self.max_verdict_age_ms == 0
            || self.max_verdict_age_ms > MAX_VERDICT_AGE_MS
        {
            return Err(ContractError::InvalidBudget);
        }
        Ok(self)
    }
}

/// Exact target a verdict must cover.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreeningTarget {
    binding: ContentBinding,
    reassembly_digest: ContentDigest,
    segment_count: u32,
}

impl ScreeningTarget {
    pub fn new(
        binding: ContentBinding,
        reassembly_digest: ContentDigest,
        segment_count: u32,
    ) -> Result<Self, ContractError> {
        if segment_count == 0 || segment_count > MAX_SCREENING_SEGMENTS {
            return Err(ContractError::InvalidSegmentCount);
        }
        if binding.transformation().sanitized_digest() != reassembly_digest {
            return Err(ContractError::ReassemblyDigestMismatch);
        }
        Ok(Self {
            binding,
            reassembly_digest,
            segment_count,
        })
    }

    pub const fn binding(&self) -> &ContentBinding {
        &self.binding
    }

    pub const fn reassembly_digest(&self) -> ContentDigest {
        self.reassembly_digest
    }

    pub const fn segment_count(&self) -> u32 {
        self.segment_count
    }
}

/// Untrusted wire representation. Validation occurs inside [`ScreeningSession`].
#[derive(Clone, Eq, PartialEq)]
pub struct SegmentEnvelope {
    pub contract_version: u32,
    pub binding: ContentBinding,
    pub reassembly_digest: ContentDigest,
    pub index: u32,
    pub count: u32,
    pub payload: Vec<u8>,
}

impl fmt::Debug for SegmentEnvelope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        redacted_bytes("SegmentEnvelope", &self.payload, formatter)
    }
}

impl SegmentEnvelope {
    pub fn new(target: &ScreeningTarget, index: u32, payload: Vec<u8>) -> Self {
        Self {
            contract_version: SCREENING_CONTRACT_VERSION,
            binding: target.binding.clone(),
            reassembly_digest: target.reassembly_digest,
            index,
            count: target.segment_count,
            payload,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerdictKind {
    Allow,
    Suspicious,
    Hostile,
    Unavailable,
}

/// Safe diagnostic categories. Free-form source or model text is never retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    ClassifierFlagged,
    ClassifierUnavailable,
}

/// A typed classifier verdict bound to exact content and immutable identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifierVerdict {
    target: ScreeningTarget,
    kind: VerdictKind,
    identity: VerdictIdentity,
    issued_at_ms: u64,
    diagnostic: Option<DiagnosticCode>,
}

impl ClassifierVerdict {
    pub fn new(
        target: ScreeningTarget,
        kind: VerdictKind,
        identity: VerdictIdentity,
        issued_at_ms: u64,
    ) -> Self {
        let diagnostic = match kind {
            VerdictKind::Allow => None,
            VerdictKind::Suspicious | VerdictKind::Hostile => {
                Some(DiagnosticCode::ClassifierFlagged)
            }
            VerdictKind::Unavailable => Some(DiagnosticCode::ClassifierUnavailable),
        };
        Self {
            target,
            kind,
            identity,
            issued_at_ms,
            diagnostic,
        }
    }

    pub const fn target(&self) -> &ScreeningTarget {
        &self.target
    }

    pub const fn kind(&self) -> VerdictKind {
        self.kind
    }

    pub const fn identity(&self) -> &VerdictIdentity {
        &self.identity
    }

    pub const fn issued_at_ms(&self) -> u64 {
        self.issued_at_ms
    }

    pub const fn diagnostic(&self) -> Option<DiagnosticCode> {
        self.diagnostic
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnavailableReason {
    MissingVerdict,
    ContractVersionMismatch,
    ContentBindingMismatch,
    SourceBindingMismatch,
    TransformationBindingMismatch,
    ReassemblyDigestMismatch,
    SegmentCountMismatch,
    SegmentOutOfRange,
    EmptySegment,
    DuplicateSegment,
    PartialSegments,
    TooManySegments,
    SegmentTooLarge,
    ContentTooLarge,
    ArithmeticOverflow,
    TimedOut,
    Cancelled,
    StaleVerdict,
    FutureVerdict,
    VerdictBindingMismatch,
    VerdictIdentityMismatch,
    ResourceExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    InvalidIdentifier,
    InvalidVersion,
    InvalidBudget,
    InvalidSegmentCount,
    ReassemblyDigestMismatch,
    MissingIdentityDigest,
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid screening contract: {self:?}")
    }
}

impl std::error::Error for ContractError {}

/// Progress intentionally exposes counters only, never an unexamined prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreeningProgress {
    pub received_segments: u32,
    pub expected_segments: u32,
    pub received_bytes: usize,
    pub complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentTaint {
    Untrusted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentAuthority {
    None,
}

/// A view that preserves the untrusted-content marker at the API boundary.
///
/// Raw bytes require the explicit, greppable [`Self::into_raw_untrusted`]
/// escape hatch. This type intentionally implements neither `Deref` nor
/// `AsRef<[u8]>`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UntrustedBytes<'a>(&'a [u8]);

impl<'a> UntrustedBytes<'a> {
    pub const fn into_raw_untrusted(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for UntrustedBytes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        redacted_bytes("UntrustedBytes", self.0, formatter)
    }
}

/// Complete content released only by a matching `Allow` verdict.
#[derive(Clone, Eq, PartialEq)]
pub struct ScreenedContent {
    bytes: Vec<u8>,
    target: ScreeningTarget,
    identity: VerdictIdentity,
}

impl fmt::Debug for ScreenedContent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        redacted_bytes("ScreenedContent", &self.bytes, formatter)
    }
}

fn redacted_bytes(name: &str, bytes: &[u8], formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
        .debug_struct(name)
        .field("len", &bytes.len())
        .field("sha256", &ContentDigest::of(bytes).to_hex())
        .finish()
}

impl ScreenedContent {
    pub fn bytes(&self) -> UntrustedBytes<'_> {
        UntrustedBytes(&self.bytes)
    }

    pub const fn target(&self) -> &ScreeningTarget {
        &self.target
    }

    pub const fn taint(&self) -> ContentTaint {
        ContentTaint::Untrusted
    }

    pub const fn authority(&self) -> ContentAuthority {
        ContentAuthority::None
    }

    pub const fn verdict_identity(&self) -> &VerdictIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WithheldContent {
    pub kind: VerdictKind,
    pub reason: Option<UnavailableReason>,
    pub diagnostic: Option<DiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScreeningDecision {
    Release(Box<ScreenedContent>),
    Withhold(WithheldContent),
}

/// One-shot, fail-closed reassembly session.
pub struct ScreeningSession {
    target: ScreeningTarget,
    budget: ScreeningBudget,
    expected_identity: VerdictIdentity,
    segments: Vec<Option<Vec<u8>>>,
    received_segments: u32,
    received_bytes: usize,
    failure: Option<UnavailableReason>,
}

impl ScreeningSession {
    pub fn new(
        target: ScreeningTarget,
        budget: ScreeningBudget,
        expected_identity: VerdictIdentity,
    ) -> Result<Self, ContractError> {
        let budget = budget.validate()?;
        let too_many_segments = target.segment_count > budget.max_segments;
        let mut segments = Vec::new();
        let allocation_failed = !too_many_segments
            && segments
                .try_reserve_exact(target.segment_count as usize)
                .is_err();
        if !too_many_segments && !allocation_failed {
            segments.resize_with(target.segment_count as usize, || None);
        }
        let failure = if too_many_segments {
            Some(UnavailableReason::TooManySegments)
        } else if allocation_failed {
            Some(UnavailableReason::ResourceExhausted)
        } else {
            None
        };
        Ok(Self {
            target,
            budget,
            expected_identity,
            segments,
            received_segments: 0,
            received_bytes: 0,
            failure,
        })
    }

    pub fn ingest(
        &mut self,
        segment: SegmentEnvelope,
        elapsed_ms: u64,
    ) -> Result<ScreeningProgress, UnavailableReason> {
        if let Some(reason) = self.failure {
            return Err(reason);
        }
        if elapsed_ms > self.budget.max_elapsed_ms {
            return self.fail(UnavailableReason::TimedOut);
        }
        if segment.contract_version != SCREENING_CONTRACT_VERSION {
            return self.fail(UnavailableReason::ContractVersionMismatch);
        }
        // Constructors make this unreachable in v1. Keep the guard for future
        // validated wire decoders, which may accept multiple contract versions.
        if segment.binding.contract_version != self.target.binding.contract_version {
            return self.fail(UnavailableReason::ContentBindingMismatch);
        }
        if segment.binding.source != self.target.binding.source {
            return self.fail(UnavailableReason::SourceBindingMismatch);
        }
        if segment.binding.transformation != self.target.binding.transformation {
            return self.fail(UnavailableReason::TransformationBindingMismatch);
        }
        if segment.reassembly_digest != self.target.reassembly_digest {
            return self.fail(UnavailableReason::ReassemblyDigestMismatch);
        }
        if segment.count != self.target.segment_count {
            return self.fail(UnavailableReason::SegmentCountMismatch);
        }
        if segment.count > self.budget.max_segments {
            return self.fail(UnavailableReason::TooManySegments);
        }
        if segment.index >= segment.count {
            return self.fail(UnavailableReason::SegmentOutOfRange);
        }
        if segment.payload.is_empty() {
            return self.fail(UnavailableReason::EmptySegment);
        }
        if segment.payload.len() > self.budget.max_segment_bytes {
            return self.fail(UnavailableReason::SegmentTooLarge);
        }
        let Some(next_bytes) = self.received_bytes.checked_add(segment.payload.len()) else {
            return self.fail(UnavailableReason::ArithmeticOverflow);
        };
        if next_bytes > self.budget.max_content_bytes {
            return self.fail(UnavailableReason::ContentTooLarge);
        }
        let Some(slot) = self.segments.get_mut(segment.index as usize) else {
            return self.fail(UnavailableReason::SegmentOutOfRange);
        };
        if slot.is_some() {
            return self.fail(UnavailableReason::DuplicateSegment);
        }
        *slot = Some(segment.payload);
        self.received_segments += 1;
        self.received_bytes = next_bytes;
        Ok(ScreeningProgress {
            received_segments: self.received_segments,
            expected_segments: self.target.segment_count,
            received_bytes: self.received_bytes,
            complete: self.received_segments == self.target.segment_count,
        })
    }

    pub fn cancel(&mut self) {
        if self.failure.is_none() {
            self.failure = Some(UnavailableReason::Cancelled);
            self.segments.iter_mut().for_each(|segment| *segment = None);
            self.received_segments = 0;
            self.received_bytes = 0;
        }
    }

    pub fn finish(
        mut self,
        verdict: Option<ClassifierVerdict>,
        now_ms: u64,
        elapsed_ms: u64,
    ) -> ScreeningDecision {
        if let Some(reason) = self.failure {
            return unavailable(reason);
        }
        if elapsed_ms > self.budget.max_elapsed_ms {
            return unavailable(UnavailableReason::TimedOut);
        }
        if self.received_segments != self.target.segment_count {
            return unavailable(UnavailableReason::PartialSegments);
        }
        let mut bytes = Vec::new();
        if bytes.try_reserve_exact(self.received_bytes).is_err() {
            return unavailable(UnavailableReason::ResourceExhausted);
        }
        for segment in &mut self.segments {
            let Some(segment) = segment.take() else {
                return unavailable(UnavailableReason::PartialSegments);
            };
            bytes.extend_from_slice(&segment);
        }
        if ContentDigest::of(&bytes) != self.target.reassembly_digest {
            return unavailable(UnavailableReason::ReassemblyDigestMismatch);
        }
        let Some(verdict) = verdict else {
            return unavailable(UnavailableReason::MissingVerdict);
        };
        if verdict.target != self.target {
            return unavailable(UnavailableReason::VerdictBindingMismatch);
        }
        if verdict.identity != self.expected_identity {
            return unavailable(UnavailableReason::VerdictIdentityMismatch);
        }
        if verdict.issued_at_ms > now_ms {
            return unavailable(UnavailableReason::FutureVerdict);
        }
        if now_ms - verdict.issued_at_ms > self.budget.max_verdict_age_ms {
            return unavailable(UnavailableReason::StaleVerdict);
        }
        match verdict.kind {
            VerdictKind::Allow => ScreeningDecision::Release(Box::new(ScreenedContent {
                bytes,
                target: self.target,
                identity: verdict.identity,
            })),
            VerdictKind::Suspicious | VerdictKind::Hostile | VerdictKind::Unavailable => {
                ScreeningDecision::Withhold(WithheldContent {
                    kind: verdict.kind,
                    reason: None,
                    diagnostic: verdict.diagnostic,
                })
            }
        }
    }

    fn fail<T>(&mut self, reason: UnavailableReason) -> Result<T, UnavailableReason> {
        self.failure = Some(reason);
        self.segments.iter_mut().for_each(|segment| *segment = None);
        self.received_segments = 0;
        self.received_bytes = 0;
        Err(reason)
    }
}

fn unavailable(reason: UnavailableReason) -> ScreeningDecision {
    ScreeningDecision::Withhold(WithheldContent {
        kind: VerdictKind::Unavailable,
        reason: Some(reason),
        diagnostic: Some(DiagnosticCode::ClassifierUnavailable),
    })
}

#[cfg(test)]
#[path = "contract_tests.rs"]
mod tests;
