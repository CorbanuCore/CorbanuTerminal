---
sprint_id: "PF-36-S02"
title: "Hosted detector bakeoff and safe local fallback"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-36"
execution_order: 60
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-36-S01"
created: 2026-08-28
updated: 2026-08-28
---

# PF-36-S02 — Hosted detector bakeoff and safe local fallback

## Execution mandate

- Deliver: Optional hosted detection either qualifies explicitly or remains disabled; its failure never weakens local enforcement.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-36).
- Feature: `PF-36`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Optional hosted detection either qualifies explicitly or remains disabled; its failure never weakens local enforcement.
- Sources and archive disposition: [PF-36 reconciliation](../../../plans/security-source-reconciliation.md#pf-36).

## Code boundaries

- OpenClaw adoption reference: [OC-4](../../../plans/openclaw-source-review-2026-08-28.md#oc-4), [OC-10](../../../plans/openclaw-source-review-2026-08-28.md#oc-10) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-35 evaluator and local qualification; PF-36 hosted interface.
- Planned: scripts/security-hosted-detector-eval; qa/security-levels/classifier/hosted-qualification.json.
- Tests: planned colocated Rust test modules prefixed `pf_36_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-36-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Record provider-specific evaluation evidence separately from source-reference tests; verify hosted failure falls back only to the qualified local gate or pause, preserving provenance and query/privacy limits.

- [ ] Compare candidate services on the same evaluator-owned blind corpus and operational privacy/cost/latency targets; record API/version/date and contractual evidence.
- [ ] Require product approval of a named service and data terms before real payloads, paid tests or deployment; no qualifying service leaves the lane explicitly disabled, not silently assumed complete.
- [ ] Implement fallback only to the already-qualified local detector with equal-or-stricter profile policy; if local is unavailable, pause ingestion.
- [ ] Test outages, partial responses, revocation during retry, spending caps, no double-send and unchanged Aggressive disclosure constraints using fixtures.
- [ ] Record enabled-qualified or disabled-no-qualified-vendor disposition with evidence; do not claim that premium detection is stronger without measured benefit.
- [ ] Add named `pf_36_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-content-security pf_36_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-36-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
