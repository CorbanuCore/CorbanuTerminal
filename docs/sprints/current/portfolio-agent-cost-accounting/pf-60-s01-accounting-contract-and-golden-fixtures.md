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
- [x] Combined local baseline 0415a00dc passed all 48 fixture tests; no production or migration code in S01.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.
- [x] Native source trace, proposed contract, hand-calculated fixtures and two reviewed corrections returned at 1c5978690 and integrated locally at 0415a00dc.
- [x] Manager independently verified corrections and 48 combined-tree tests; review/commit evidence is retained in the handoff.
- [x] Travis explicitly approved the v1 defaults in this task; [decision record](../../../plans/workstream-manager-handoff-2026-09-11.md#accounting-decision). No billing/live collection enabled.

## Remaining

- [ ] Finish the exact S02 native adapter/allocation handoff and reconcile final receiving evidence; do not start S02 before acceptance/archive.

## Verification

- [x] Focused: 48 tests on combined baseline 0415a00dc; reviewer findings independently reproduced and corrected. No runtime proof claimed.
- [ ] Integration: final allocation/policy candidate must pass `python3 docs/plans/check.py`, `python3 docs/sprints/check.py` and `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-cost-accounting/` and update plan backlinks.
