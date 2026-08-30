---
sprint_id: "PF-34-S04"
title: "Screening segment contract and fixtures"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-34"
execution_order: 19
owner: "Codex ingress/classifier lane"
parallel_lane: "ingress-classifier"
write_scope: "codex-rs/content-security/src/contract.rs, codex-rs/content-security/src/contract_tests.rs, qa/security-levels/ingress-contract/, qa/security-levels/sprints/PF-34-S04/, docs/sprints/current/p0-security-levels/pf-34-s04-screening-contract-and-fixtures.md"
integration_gate: "Jim Ricketts receives the PF-34-S04 candidate at G1/G2, audits the literal scope, performs serialized content-security crate/workspace/Cargo/Bazel/lock registration, reruns governance and the complete content-security suite on the combined tree, then archives PF-34-S04 before PF-35-S01 allocation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-ingress-classifier"
branch: "feat/p0-security-ingress-classifier"
base_commit: "6a35712cd5731b191d875e8c6468f1abe23eb66e"
depends_on: "none"
created: 2026-08-28
updated: 2026-08-28
---

# PF-34-S04 — Screening segment contract and fixtures

## Execution mandate

- Deliver: Freeze bounded segment, sanitization and verdict contracts so classifier preparation can proceed independently.
- Excludes: protected-mode activation, adjacent feature implementation and Permissive behavior changes.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-34).
- Feature: `PF-34`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [accepted architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md).
- Source input: [OpenClaw source review](../../../plans/openclaw-source-review-2026-08-28.md) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference behavior is not candidate acceptance.

## Code boundaries

- Planned: codex-rs/content-security/src/{contract,contract_tests}.rs; qa/security-levels/ingress-contract/
- Existing integration paths are read-only until the named consumer sprint; shared manifests/lockfiles require serialized ownership.

## Preconditions

- [x] Plan active; dependencies in front matter are `none`.
- [x] Named execution owner and exact plan-matching worktree/branch/base assigned; governance checkers pass before readiness.
- [x] Root and nearest implementation AGENTS.md read; literal disjoint write scope and Jim Ricketts receiving integration gate reserved.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.
- [x] Ingress/classifier lane allocated from dispatch base `6a35712cd5731b191d875e8c6468f1abe23eb66e`; all caches are rooted under `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`.

## Remaining

- [ ] Define immutable content digest, opaque source binding, transformation/version, segment index/count and reassembly identity; source authority remains owned by PF-30, never duplicated or inferred from caller strings.
- [ ] Define allow/suspicious/hostile/unavailable outcomes, model/threshold identities, size/time budgets and safe diagnostics; missing, malformed or mismatched verdicts are unavailable, not allow.
- [ ] Create versioned raw/rendered/sanitized fixtures, quarantine transition fixtures and cross-segment attacks; the fixture seam cannot authorize tools, clear taint or ship as protected ingestion.
- [ ] Define full-content decision and cancellation semantics; no unexamined prefix release. Any future incremental protocol needs separate proof and cannot waive reassembled-content screening.
- [ ] Freeze schema compatibility and change ownership for sanitizer, classifier and quarantine consumers; code uses pure constructors and fixtures, not live Core/provider adapters.

## Verification

- [ ] Run affected format/fix tools before final tests; record exact commands and actual test counts.
- [ ] Run planned content-security contract tests including malformed/partial/duplicate segments, digest/version mismatch, timeout and forced-allow safety assertions at the interface.
- [ ] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [ ] Verify no runtime route or profile becomes available from fixture-only preparation.

## Exit evidence

- [ ] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-34-S04/`.
- [ ] PF-34-S01 and PF-35-S03 must test actual rendering, PF-30 provenance and deterministic policy; fixture completion is not detector or ingestion qualification.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
