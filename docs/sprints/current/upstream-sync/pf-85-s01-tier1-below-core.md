---
sprint_id: "PF-85-S01"
title: "Tier 1: crates below core"
status: in_progress
plan_file: "docs/plans/active/upstream-sync-rust-v0.167.0.md"
plan_feature: "PF-85"
execution_order: 2
owner: "Coordinator, contributor integration #2106"
parallel_lane: "upstream-sync"
write_scope: "codex-rs/codex-mcp/, codex-rs/rmcp-client/, docs/plans/active/upstream-sync-rust-v0.167.0.md, docs/sprints/current/upstream-sync/, docs/sprints/archive/upstream-sync/, docs/management/upstreamSync/, qa/release/unreleased/notes/2101-mcp-discovery-budget.md"
integration_gate: "Coordinator receives #2101 into sync/upstream-rust-v0.167.0 only after independent exact-head review, isolated affected-crate tests and applicable CI; the final main merge still requires PF-85-S05 and SG1-SG3. No parallel slice implementation is allocated."
worktree: "/Volumes/CorbanuDrive/Corbanu/.codex-work/contrib-1111-2101"
branch: "maint/contrib-2101-1111"
base_commit: "787027b83ffa53875f49e40a2886284d26557075"
depends_on: "PF-85-S00"
created: 2026-11-07
updated: 2026-11-19
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
- [x] S3a Tier 1 sandbox crates resolved (with all of S3a, 2026-11-14; see PF-85-S02 and #1628).
- [x] Reconciled 2026-11-19 at sync `91efec006d` (sync-integrator): every Tier 1 ledger row is resolved (S1a 80, S1b 102, S1c 849, S1d 493). #2101 merged into sync on 2026-11-14 (`f5fa123c8e`) after independent Opus review, keeping its process-lifecycle tests and ledger attribution. Tier 1 security-path rows (S3a) were resolved with security review (#1628).
- [x] D319 fixtures from `main` `18fa372a` were committed before any `age`/`zip` change (#1712); the security owner decided D345/D346.
- [x] Migration renumbering (0054–0069, `codex-rs/state/MIGRATIONS.md`); `tests/state_db_upgrade.rs` upgrades the 0.1.48 release DB and a `main` 1–53 DB. There is no 0.1.49 fixture yet (1–53 + 0070, #2812); upgrade and rollback evidence belong to PF-85-S05.
- [x] Non-user-facing N/A: Tier 1 crates have no TUI flow of their own; PF-85-S05 keeps the final functional qualification.

## Remaining

The #2101 allocation is finished. The one open item is #2122's MCP helper qualification. Re-allocating S01 to #2122's worktree, or recording it under PF-85-S05, is the coordinator's call (plan `implementation_worktrees`).

- [ ] #2122 (full CLI helpers for `codex-mcp`/`codex-rmcp-client`) qualified on all three platforms, reviewed and merged (owner: sync test-isolation maintainer).

## Verification

- [ ] Isolated `just test -p codex-mcp` and `just test -p codex-rmcp-client` with full CLI helpers on all three platforms (#2122; four MCP cases were CLI-blocked until #2274). Later Tier 1 crates keep their platform/check/clippy gates.
- [x] TUI applicability: none.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/upstream-sync/`.
