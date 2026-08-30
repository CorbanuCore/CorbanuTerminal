---
sprint_id: "PF-34-S04"
title: "Screening segment contract and fixtures"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-34"
execution_order: 19
owner: "Codex ingress/classifier lane"
parallel_lane: "ingress-classifier"
write_scope: "codex-rs/content-security/, codex-rs/Cargo.toml, codex-rs/Cargo.lock, BUILD.bazel, MODULE.bazel.lock, .github/workflows/security-ingress-contract.yml, qa/security-levels/ingress-contract/, qa/security-levels/sprints/PF-34-S04/, docs/sprints/current/p0-security-levels/pf-34-s04-screening-contract-and-fixtures.md, docs/sprints/archive/p0-security-levels/pf-34-s04-screening-contract-and-fixtures.md, docs/sprints/current/p0-security-levels/index.md, docs/sprints/index.md, docs/plans/active/p0-security-levels.md, mkdocs.yml"
integration_gate: "Codex ingress/classifier lane owns the user-authorized G1/G2 transfer: combine current main at 3232f5e65bae60bc86122a5495ebb4c280f7c8fb, audit the literal scope, serialize content-security crate/workspace/Cargo/Bazel/lock/CI registration, rerun governance and the complete content-security suite, obtain an independent TMUX plus Corbanu Terminal plus Claude Opus 5.0 Max review, then archive PF-34-S04 before PF-35-S01 allocation."
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
- [x] Root and nearest implementation AGENTS.md read; literal disjoint write scope is reserved and the user transferred G1/G2 integration ownership from unavailable Jim Ricketts to this lane.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.
- [x] Ingress/classifier lane allocated from dispatch base `6a35712cd5731b191d875e8c6468f1abe23eb66e`; all caches are rooted under `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-ingress-classifier/`.
- [x] Rebased onto current `main` at `1a5562738cb3d53bd4d0b6668761cfe76bd4b93e` so the mandated review used the corrected TMUX/provider behavior while preserving the immutable creation coordinate.
- [x] Immutable opaque-source, raw/rendered/sanitized transformation, complete-reassembly, segment index/count and contract-version bindings implemented.
- [x] Typed allow/suspicious/hostile/unavailable decisions bind exact model/threshold identities, safe diagnostics, size/time/freshness budgets and sticky fail-closed state.
- [x] Versioned benign, cross-segment hostile and quarantine-transition fixtures freeze hashes, schema, taint/no-authority and forbidden transitions.
- [x] Full-content one-shot release and cancellation semantics prevent partial/prefix release and forced allow after any terminal fault.
- [x] Pure constructors and fixture verifier add no Core/provider/runtime route; PF-30 source authority and PF-34/PF-35 change ownership remain separate.
- [x] Lane-local formatting, Clippy, argument-comment lint, 20 named contract tests, 14 verifier regressions, seven-fixture/schema verification, governance and `git diff --check` pass at final Opus-remediated implementation commit `a75efecc0a37d5544e123ad19d57867cac360a68`.
- [x] Supplemental structured Codex review verified four in-scope findings, drove bounded remediation and finished clean on the complete committed branch.
- [x] Independent review ran through TMUX and rebased Corbanu Terminal using provider `claude-plan`, route `claude-opus-5-plan` (provider-reported Claude Opus 5.0), and effort `max`; four immutable passes ended `clean` with 0 new P0/P1/P2.
- [x] Current `main` at `1907d99aed9714f05a5f54fca1703658017d616c` was combined without rewriting the immutable reviewed commits; PF-13-S05 archive and plan updates are preserved.
- [x] Current `main` advanced to `3232f5e65bae60bc86122a5495ebb4c280f7c8fb` and was reconciled in merge `158b9b0ebe4b06a81c98be6a58a0d1c7919a0d08`; PF-31/PF-33 remediation remains intact.
- [x] Transferred G1 registered the crate, Cargo lock, Bazel fixture visibility and recurring Linux/macOS/Windows contract CI; the unused dependency alias was removed after `cargo shear` rejected it.
- [x] Combined-tree `just fix`, `just fmt`, strict Clippy, 21 focused/full tests, Bazel parity, fixture/schema checks and targeted argument-comment lint pass. The only repository-wide lint failures are two pre-existing `security-policy` argument comments outside this sprint.

## Remaining

- [x] Closed the integration review's stale-evidence P2 with checksum-verified packet `dec900c90c5b7a0e649eef942b4dda12f605f0bc751aa708036102067188d829`; the follow-up returned `clean` with 0 P0/P1/P2 and explicitly authorized archive.

## Verification

- [x] Ran affected lane-local format/lint tools before final tests and recorded exact commands and actual test counts; repository `just fix` then ran after transferred G1 registration.
- [x] Run planned content-security contract tests including malformed/partial/duplicate segments, digest/version mismatch, timeout and forced-allow safety assertions at the interface.
- [x] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [x] Verified no runtime route or profile becomes available: G1 adds only a private-module `lib.rs`, Cargo/Bazel registration and CI; it adds no consumer dependency, Core/provider adapter or profile change.
- [x] At G1, registered-crate `just fix`, `just fmt`, named/full crate tests, lock regeneration, Bazel parity and recurring-CI contract checks ran on the combined tree.
- [x] Confirmed the evidence-only remediation through the checksum-verified TMUX/Corbanu Terminal/Claude Opus 5.0 Max follow-up.

## Exit evidence

- [x] Commit contract/fixture versions, independent owner review, and lane-candidate outputs under `qa/security-levels/sprints/PF-34-S04/`; combined-tree outputs are now owned by this lane under the user-authorized transfer.
- [x] PF-34-S01 and PF-35-S03 remain assigned actual rendering, PF-30 provenance and deterministic policy; fixture completion is not detector or ingestion qualification.
- [x] Record the exact integration handoff and lane scope audit; archive and navigation changes remain after G1 combined-tree acceptance.
- [x] Record the transferred integration commit and combined-tree source hashes.
- [x] Archive PF-34-S04 after the clean follow-up review; PF-35-S01 remains unallocated here.
