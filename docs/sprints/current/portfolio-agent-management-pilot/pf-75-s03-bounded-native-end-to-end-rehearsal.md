---
sprint_id: "PF-75-S03"
title: "Bounded native end-to-end rehearsal"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-management-pilot.md"
plan_feature: "PF-75"
execution_order: 3
owner: "Jim Ricketts (integration lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-75-S02"
created: 2026-09-09
updated: 2026-09-10
---

# PF-75-S03 — Bounded native end-to-end rehearsal

## Execution mandate

- Deliver: qa/portfolio/agent-management-pilot/rehearsal.md; No duplicate execution or false completion; new work stops when queue/budget caps are reached; all evidence traces to one candidate.
- Excludes: A second scheduler, unattended merges/releases, new task systems, broad credentials, automatic Task Node posting or the claim that testing is bulletproof.
- Budget proposal: 1–2 tester-days after candidate readiness; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Native agent management and independent acceptance pilot](../../../plans/proposed/portfolio-agent-management-pilot.md)
- Feature: `PF-75`; acceptance: No duplicate execution or false completion; new work stops when queue/budget caps are reached; all evidence traces to one candidate.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-agent-management-pilot.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/core/src/agent/control/mailbox.rs`; `codex-rs/core/src/agent/control/execution.rs`; `codex-rs/core/src/agent/registry.rs`; `codex-rs/tasknode-session/src/lib.rs`; `docs/tmuxHarness.md`.
- Planned output: `qa/portfolio/agent-management-pilot/rehearsal.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-management-pilot/pf-75-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-75-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Use a disposable authorized fixture workload, existing native agents and one independent reviewer; no unrelated product implementation.
- [ ] Exercise worker/coordinator interruption, duplicate handoff, blocked approval, model unavailability and two queued review packets; retain native IDs.
- [ ] Measure human minutes, rework, escaped seeded defects and spend; use a local dry-run Task Node projection until posting is explicitly approved.
- [ ] Rehearse mapped native task status and evidence lifecycle against fixtures, duplicate/uncertain-write reconciliation and restart; progress must never imply human/server acceptance.
- [ ] Compare visible activity against a known action ledger for TUI, CLI, Desktop and remote-worker lanes; record supported/partial/missing coverage, never assume complete capture.
- [ ] Verify one publisher renders machine/blocker/log/test-plan/sprint links and both timestamps; offline or stale reports remain visible and worker reports never overwrite one another.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: From codex-rs: just test -p codex-core agent; just test -p codex-tasknode-session; plus actual-key rehearsal checkpoints.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-management-pilot/` and update plan backlinks.
