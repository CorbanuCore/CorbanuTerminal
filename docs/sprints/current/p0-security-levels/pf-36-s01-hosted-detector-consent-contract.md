---
sprint_id: "PF-36-S01"
title: "Optional hosted detector consent contract"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-36"
execution_order: 59
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-35-S03, PF-33-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-36-S01 — Optional hosted detector consent contract

## Execution mandate

- Deliver: Hosted screening cannot disclose protected data or activate without exact human consent.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-36).
- Feature: `PF-36`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Hosted screening cannot disclose protected data or activate without exact human consent.
- Sources and archive disposition: [PF-36 reconciliation](../../../plans/security-source-reconciliation.md#pf-36).

## Code boundaries

- OpenClaw adoption reference: [OC-3](../../../plans/openclaw-source-review-2026-08-28.md#oc-3), [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-35 classifier interface; PF-28 disclosure gate.
- Planned: codex-rs/content-security/src/hosted.rs; codex-rs/tui/src/bottom_pane/hosted_detector_consent.rs.
- Tests: planned colocated Rust test modules prefixed `pf_36_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-35-S03, PF-33-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Treat hosted-detector payloads as protected outbound disclosure with explicit data/cost consent and complete secret removal; no OpenClaw wrapper or logging helper is evidence of a qualified hosted detector.

- [ ] Define a disabled-by-default provider-neutral hosted detector interface and deterministic local fake service; do not invent a vendor contract or premium entitlement.
- [ ] Require human opt-in to exact provider/data categories/retention/region/cost cap and revocable consent; Aggressive additionally requires a narrow disclosure/egress grant.
- [ ] Transmit only sanitized policy-allowed segments after secret/protected-data gating; no raw artifacts, hidden history or financial records.
- [ ] Treat vendor scores as detection only; map timeout, rate limit, malformed response and consent expiry to explicit unavailable states.
- [ ] Test opt-in/cancel/revoke, prohibited disclosure, cost cap, malicious responses and fake-service contract; mark real providers unavailable until PF-36-S02 qualifies them.
- [ ] Add named `pf_36_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_36_s01 && just test -p codex-tui pf_36_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: inspect privacy/cost disclosure → Esc unchanged → opt in to fake service → revoke → retry denied.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-36-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
