---
sprint_id: "PF-85-S01"
title: "Tier 1: crates below core"
status: in_progress
plan_file: "docs/plans/active/upstream-sync-rust-v0.167.0.md"
plan_feature: "PF-85"
execution_order: 2
owner: "Coordinator, contributor integration #2106"
parallel_lane: "upstream-sync"
write_scope: "codex-rs/codex-mcp/, codex-rs/rmcp-client/, docs/plans/active/upstream-sync-rust-v0.167.0.md, docs/sprints/current/upstream-sync/, docs/sprints/archive/upstream-sync/, docs/management/upstreamSync/, qa/release/unreleased/notes/"
integration_gate: "Coordinator receives #2101 into sync/upstream-rust-v0.167.0 only after independent exact-head review, isolated affected-crate tests and applicable CI; the final main merge still requires PF-85-S05 and SG1-SG3. No parallel slice implementation is allocated."
worktree: "/Volumes/CorbanuDrive/Corbanu/.codex-work/contrib-1111-2101"
branch: "maint/contrib-2101-1111"
base_commit: "787027b83ffa53875f49e40a2886284d26557075"
depends_on: "PF-85-S00"
created: 2026-11-07
updated: 2026-11-11
---

# PF-85-S01 — Tier 1: crates below core

## Execution mandate

- Deliver: Every Tier 1 ledger row resolved (S1a protocol, S1b config/providers/auth, S1c other crates incl. migration renumbering, S1d tooling, S3a Tier 1 sandbox crates); main merge #2 (S1 exit; #1 was an earlier catch-up).
- Excludes: core and crates above it.
- Tracking issue: #1628. Details: [program plan §6](../../../management/upstreamSyncPlan.md).

## Plan linkage

- Plan: [Upstream Codex sync to rust-v0.167.0](../../../plans/active/upstream-sync-rust-v0.167.0.md).
- Feature: `PF-85`.
- Product citation: **Product principles** — “Maintain continuous Codex parity without removing Corbanu-specific behavior.”

## Code boundaries

- Write scope: Ledger rows with tier 1 (one slice owner per path).
- Branch target: `sync/upstream-rust-v0.167.0` (PRs only).

## Preconditions

- [x] Plan is active.
- [x] S00 historical exit verified on #1627 and its completed record archived.
- [x] Exact coordinator worktree, branch and base match the plan.
- [x] Single coordinator allocation; other slice work remains queued.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Main merge #2 (S1 exit): #2012/#2014 (#1 was #1915/#1922).
- [x] #2101: pending same-owner and pool compatibility use the effective capped discovery budget; ready connections do not compare it.
- [x] Three isolated discovery-budget regressions passed, covering differing budgets, equivalent/default/capped budgets and ready reuse (nextest `b763c88a-413b-4f12-8ca4-062e3ff98b39`). An initial test compilation failed on a missing `mut`; corrected and freshly rerun.
- [x] Internal connection-lifecycle scope: code-blind TUI design is not applicable to this slice; PF-85-S05 retains final sync functional qualification.

## Remaining

Current mechanical assignment is limited to #2101; later slice items below stay queued.

- [ ] Preserve contributor process-lifecycle tests, ledger attribution and independent final review.
- [ ] Slice PRs per the ledger; security-owner review on security paths.
- [ ] D319 fixtures (plan §6 S1) generated from `main` and committed before any `age`/`zip` change; then the security owner's decision.
- [ ] Migration renumbering with an upgrade test from the latest released state DB.
- [ ] Code-blind functional design frozen before test-result disclosure, or non-user-facing N/A reason recorded.

## Verification

- [ ] Isolated `just test -p codex-mcp` and `just test -p codex-rmcp-client` on the final #2101 tree; later Tier 1 crates retain their platform/check/clippy gates.
- [ ] TUI applicability: none.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/upstream-sync/`.
