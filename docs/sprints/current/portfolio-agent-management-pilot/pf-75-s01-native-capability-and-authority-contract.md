---
sprint_id: "PF-75-S01"
title: "Native capability and authority contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-management-pilot.md"
plan_feature: "PF-75"
execution_order: 1
owner: "Jim Ricketts (integration lead, proposed)"
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

# PF-75-S01 — Native capability and authority contract

## Execution mandate

- Deliver: docs/research/agent-management-pilot/contract.md; One source owns execution state; Task Node is a coordination/evidence projection, not a second dispatcher.
- Excludes: A second scheduler, unattended merges/releases, new task systems, broad credentials, automatic Task Node posting or the claim that testing is bulletproof.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Native agent management and independent acceptance pilot](../../../plans/proposed/portfolio-agent-management-pilot.md)
- Feature: `PF-75`; acceptance: One source owns execution state; Task Node is a coordination/evidence projection, not a second dispatcher.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-agent-management-pilot.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/core/src/agent/control/mailbox.rs`; `codex-rs/core/src/agent/control/execution.rs`; `codex-rs/core/src/agent/registry.rs`; `codex-rs/tasknode-session/src/lib.rs`; `docs/tmuxHarness.md`.
- Planned output: `docs/research/agent-management-pilot/contract.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-management-pilot/pf-75-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Reconcile active work before allocating any pilot; choose one permitted workload, spend cap, named acceptance tester and live Task Node write policy.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Map existing orchestration, mailbox/resume, worktree allocation and Task Node evidence surfaces; reproduce current integration gaps read-only.
- [ ] Select an already qualified positive recovery journey on the pinned candidate and link its evidence for S02; if unavailable, stop calibration setup or declare the receiving auth prerequisite after baseline reconciliation. PF-75 does not repair auth.
- [ ] Define planner, builder, independent tester, reviewer and human roles; distinguish task state, review findings and actual authorization.
- [ ] Specify two-builder ceiling within the global three reserved-sprint limit, a review queue cap of two and at most three daily human decisions.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Tabletop duplicate dispatch, stopped worker, stale evidence, denied auth and exhausted quota.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-management-pilot/` and update plan backlinks.
