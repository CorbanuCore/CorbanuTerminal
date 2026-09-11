---
sprint_id: "PF-60-S01"
title: "Accounting contract and golden fixtures"
status: in_progress
plan_file: "docs/plans/active/portfolio-agent-cost-accounting.md"
plan_feature: "PF-60"
execution_order: 1
owner: "Astra High accounting kickoff"
parallel_lane: "accounting-contract"
write_scope: "docs/research/agent-cost-accounting/, qa/portfolio/agent-cost-accounting/pf-60-s01/"
integration_gate: "Codex management reviews only the worker's research/fixture diff, independently recomputes totals, runs fixture and governance tests on the combined tree, and requests Travis's contract acceptance before S02; worker cannot edit shared plans or merge."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/accounting-pf60-s01-20260911"
branch: "workstream/accounting-pf60-s01-20260911"
base_commit: "295aed26e53b17f919f7199ae1c9748b1b1250ba"
depends_on: "none"
created: 2026-09-09
updated: 2026-09-11
---

# PF-60-S01 — Accounting contract and golden fixtures

Allocated before post-merge dispatch. Status reserves the lane, not proof an
agent has started; manager receipt records native subagent ID and actual HEAD.

## Execution mandate

- Deliver: docs/research/agent-cost-accounting/contract.md; Every fixture has raw inputs, expected totals and provenance; no unknown value is rendered as zero.
- Excludes: Changing prices, rebilling historical customers, collecting prompts, restoring legacy Plan allowances, or silently converting allowance to cash.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Unified agent cost and usage accounting](../../../plans/active/portfolio-agent-cost-accounting.md)
- Feature: `PF-60`; acceptance: Every fixture has raw inputs, expected totals and provenance; no unknown value is rendered as zero.
- Upstream/allocation: [plan record](../../../plans/active/portfolio-agent-cost-accounting.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/state/migrations/0041_provider_request_cache_usage.sql`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`; `codex-rs/tui/src/chatwidget/usage.rs`; `codex-rs/tui/src/token_usage.rs`.
- Planned output: `docs/research/agent-cost-accounting/contract.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-cost-accounting/pf-60-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [x] Travis authorized S01 kickoff; active plan, disjoint worker lane and global three-sprint reservation reconciled.
- [x] No sprint dependencies; synthetic contract preparation is authorized, with vocabulary/retention approval gating S02 runtime.
- [x] Exact worktree/branch/base, literal research/QA paths and receiving owner allocated; fast-forward to main planning merge before dispatch.
- [x] Only public source/synthetic inputs; one requested Astra High agent, no extra agents, paid data, private logs or external writes.
- [ ] Run nonempty fixture validation below before handoff; no production or migration code in S01.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Trace existing persisted usage, replay and parent/child IDs; reconcile receiving worktrees and private Corbanu API balance accounting read-only, without restoring legacy Plan entitlements.
- [ ] Specify request identity, retry identity, hierarchy aggregation, currency precision, cached/reasoning tokens, estimates versus invoices and effective-dated prices.
- [ ] Create a hand-calculated fixture matrix for three providers, two children, retries, partial streams, unknown prices and historical records.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: `python3 -m unittest discover -s qa/portfolio/agent-cost-accounting/pf-60-s01 -p 'test_*.py'`; add and run nonzero tests. Reviewer independently recomputes fixture totals and checks fields against native state.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
