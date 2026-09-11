---
sprint_id: "PF-75-S04"
title: "Two-week pilot operating decision"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-management-pilot.md"
plan_feature: "PF-75"
execution_order: 4
owner: "Jim Ricketts (integration lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-75-S03"
created: 2026-09-09
updated: 2026-09-10
---

# PF-75-S04 — Two-week pilot operating decision

## Execution mandate

- Deliver: docs/research/agent-management-pilot/decision.md; The human selects whether to activate a pilot; proposed scheduling does not create an automation or start workers.
- Excludes: A second scheduler, unattended merges/releases, new task systems, broad credentials, automatic Task Node posting or the claim that testing is bulletproof.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Native agent management and independent acceptance pilot](../../../plans/proposed/portfolio-agent-management-pilot.md)
- Feature: `PF-75`; acceptance: The human selects whether to activate a pilot; proposed scheduling does not create an automation or start workers.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-agent-management-pilot.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/core/src/agent/control/mailbox.rs`; `codex-rs/core/src/agent/control/execution.rs`; `codex-rs/core/src/agent/registry.rs`; `codex-rs/tasknode-session/src/lib.rs`; `docs/tmuxHarness.md`.
- Planned output: `docs/research/agent-management-pilot/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-management-pilot/pf-75-s04/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-75-S03 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Compare rehearsal results with one-hour review capacity and current WIP; do not add two builders on top of occupied lanes.
- [ ] Recommend native-as-is, one bounded missing adapter or stop; any scheduler replacement requires separate product approval.
- [ ] Name the native/adapter ownership seam, coverage gaps, stable retry policy and exact Task Node action/account/consent gate; keep writeback disabled until a separately authorized live acceptance passes.
- [ ] Propose a two-week schedule and criteria: zero accepted seeded critical defects, no unauthorized external writes, <=60 daily review minutes on at least four of five observed days.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Review all evidence gaps and classify deep human reviews separately from the daily one-hour target.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-management-pilot/` and update plan backlinks.
