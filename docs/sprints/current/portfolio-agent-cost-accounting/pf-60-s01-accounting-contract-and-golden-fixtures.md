---
sprint_id: "PF-60-S01"
title: "Accounting contract and golden fixtures"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 1
owner: "Jim Ricketts (proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-09-09
updated: 2026-09-09
---

# PF-60-S01 — Accounting contract and golden fixtures

## Execution mandate

- Deliver: docs/research/agent-cost-accounting/contract.md; Every fixture has raw inputs, expected totals and provenance; no unknown value is rendered as zero.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/proposed/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Every fixture has raw inputs, expected totals and provenance; no unknown value is rendered as zero.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Planned output: `docs/research/agent-cost-accounting/contract.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-cost-accounting/pf-60-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Approve the cost vocabulary, retention window, currency/price source and historical unknown policy; reconcile current branch before selecting the next migration number.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Trace existing persisted usage, replay and parent/child IDs; reconcile receiving worktrees and private Corbanu API balance accounting read-only, without restoring legacy Plan entitlements.
- [ ] Specify request identity, retry identity, hierarchy aggregation, currency precision, cached/reasoning tokens, estimates versus invoices and effective-dated prices.
- [ ] Create a hand-calculated fixture matrix for three providers, two children, retries, partial streams, unknown prices and historical records.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Reviewer independently recomputes all fixture totals; inspect every proposed schema field against existing state.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
