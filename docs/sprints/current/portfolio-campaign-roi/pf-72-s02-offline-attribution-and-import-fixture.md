---
sprint_id: "PF-72-S02"
title: "Offline attribution and import fixture"
status: draft
plan_file: "docs/plans/proposed/portfolio-campaign-roi.md"
plan_feature: "PF-72"
execution_order: 2
owner: "Alex Good (campaign owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-72-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-72-S02 — Offline attribution and import fixture

## Execution mandate

- Deliver: docs/research/campaign-roi/fixture.md; Duplicate re-import leaves totals unchanged; unknown cost and unverified revenue cannot create numeric ROI.
- Excludes: Replacing Alex's tracker, copying raw logs to external models, claiming causal ROI from activity counts or deploying another dashboard.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Campaign Tracker and agent ROI integration](../../../plans/proposed/portfolio-campaign-roi.md)
- Feature: `PF-72`; acceptance: Duplicate re-import leaves totals unchanged; unknown cost and unverified revenue cannot create numeric ROI.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-campaign-roi.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/tasknode-session/src/lib.rs`; `docs/features/tasknode.md`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`.
- Planned output: `docs/research/campaign-roi/fixture.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/campaign-roi/pf-72-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-72-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Create a synthetic/redacted export and deterministic transformation specification using existing tracker fields.
- [ ] Exercise duplicate task IDs, parent/child costs, delayed billing, rejected work and corrected outcomes.
- [ ] Demonstrate idempotent replay and a digest-only evidence link; write no production tracker records.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Independent recomputation of the three campaign cases plus replay-twice check.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-campaign-roi/` and update plan backlinks.
