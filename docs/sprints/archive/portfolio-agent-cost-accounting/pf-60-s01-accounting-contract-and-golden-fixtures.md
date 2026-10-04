---
sprint_id: "PF-60-S01"
title: "Accounting contract and golden fixtures"
status: completed
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

Closed locally September 11: reviewed fixtures/corrections, approved defaults
and independently reviewed S02 handoff accepted by Codex management. This is
contract/fixture completion, not accounting runtime, release or live enablement.

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
- [x] Exact [S02 handoff](../../../research/agent-cost-accounting/s02-allocation.md) reviewed with no findings; receiving owner accepts the bounded S01 artifact.

## Remaining

None in S01; runtime implementation and qualification remain in S02–S04.

## Verification

- [x] Focused: 48 tests on combined baseline 0415a00dc; reviewer findings independently reproduced and corrected. No runtime proof claimed.
- [x] Combined baseline and reviewed allocation passed both governance checkers and whitespace; lifecycle transition rechecked by the manager before dispatch.
- [x] Astra High reviews and parent reproductions cover source/calculations and synthetic success/failure/replay; native UI and real billing are not qualified.
- [x] Exact candidates, test results and review artifacts recorded in the linked manager handoff and private manager-continuation receipt; no new collector or product automation tested.

## Exit evidence

- [x] Fixture candidate 1c5978690 integrated at 0415a00dc; S02 handoff independently reviewed on receiving allocation base 265bf0c3e.
- [x] Travis approved v1 defaults; Codex management accepts the bounded synthetic artifact and handoff, not runtime or release.
- [x] Handoff names exact future scope, upstream ancestry, unknowns, test-only boundary and later native/human gates.
- [x] Accepted S01 moved to archive; plan backlinks/navigation and one-sprint reservation reconciled.
