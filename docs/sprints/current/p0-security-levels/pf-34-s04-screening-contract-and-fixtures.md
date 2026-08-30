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
updated: 2026-08-30
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
- [x] Rebased onto current `main` at `1a5562738cb3d53bd4d0b6668761cfe76bd4b93e` so the mandated review used the corrected TMUX/provider behavior while preserving the immutable creation coordinate.
- [x] Immutable opaque-source, raw/rendered/sanitized transformation, complete-reassembly, segment index/count and contract-version bindings implemented.
- [x] Typed allow/suspicious/hostile/unavailable decisions bind exact model/threshold identities, safe diagnostics, size/time/freshness budgets and sticky fail-closed state.
- [x] Versioned benign, cross-segment hostile and quarantine-transition fixtures freeze hashes, schema, taint/no-authority and forbidden transitions.
- [x] Full-content one-shot release and cancellation semantics prevent partial/prefix release and forced allow after any terminal fault.
- [x] Pure constructors and fixture verifier add no Core/provider/runtime route; PF-30 source authority and PF-34/PF-35 change ownership remain separate.
- [x] Lane-local formatting, Clippy, argument-comment lint, 20 named contract tests, 13 verifier regressions, seven-fixture/schema verification, governance and `git diff --check` pass at Opus-remediated implementation commit `74e97148701ef541ff9ef2d0a9194ba472b2801c`.
- [x] Supplemental structured Codex review verified four in-scope findings, drove bounded remediation and finished clean on the complete committed branch.

## Remaining

- [ ] Record the final immutable Claude Opus 5.0 Max follow-up verdict and rerun outputs after every accepted fix.
- [ ] Hand the candidate, contract/fixture identities, scope audit and exact shared registration patch to Jim Ricketts for serialized G1/G2 integration.

## Verification

- [ ] Run affected format/fix tools before final tests; record exact commands and actual test counts.
- [ ] Run planned content-security contract tests including malformed/partial/duplicate segments, digest/version mismatch, timeout and forced-allow safety assertions at the interface.
- [x] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [x] Verified no runtime route or profile becomes available: the lane does not add `lib.rs`, Cargo/Bazel registration, Core/provider adapters or profile changes.

## Exit evidence

- [ ] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-34-S04/`.
- [x] PF-34-S01 and PF-35-S03 remain assigned actual rendering, PF-30 provenance and deterministic policy; fixture completion is not detector or ingestion qualification.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
