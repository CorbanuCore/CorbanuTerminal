---
sprint_id: "PF-72-S01"
title: "Existing tracker and outcome contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-campaign-roi.md"
plan_feature: "PF-72"
execution_order: 1
owner: "Alex Good (campaign owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-09-09
updated: 2026-09-10
---

# PF-72-S01 — Existing tracker and outcome contract

## Execution mandate

- Deliver: docs/research/campaign-roi/contract.md; A complete field map identifies existing fields, gaps and human acceptance; no new dashboard is assumed.
- Excludes: Replacing Alex's tracker, copying raw logs to external models, claiming causal ROI from activity counts or deploying another dashboard.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Campaign Tracker and agent ROI integration](../../../plans/proposed/portfolio-campaign-roi.md)
- Feature: `PF-72`; acceptance: A complete field map identifies existing fields, gaps and human acceptance; no new dashboard is assumed.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-campaign-roi.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/tasknode-session/src/lib.rs`; `docs/features/tasknode.md`; `codex-rs/app-server/src/request_processors/token_usage_replay.rs`.
- Planned output: `docs/research/campaign-roi/contract.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/campaign-roi/pf-72-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Alex identifies the existing tracker/schema and allowed export; use PF-60 accounting definitions before any spend-derived ROI.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Inspect native `tasknode-session/src/tracker.rs`, `chatwidget/campaign_tracker.rs`, `cli/src/tasknode_cmd.rs` and September 7 tracker QA; ask Alex only for unresolved backend/operator coordinates and export permission.
- [ ] Define outcome versus activity, attribution window, accepted evidence, reversals and unknown-cost handling.
- [ ] Map native Task Node/run IDs and outcome fields; document missing cost semantics for S02 rather than block basic capture/mapping on PF-60. Reuse PF-75's operational boundary and minimize exported context.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Trace three synthetic campaigns including one failed and one unverified outcome.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-campaign-roi/` and update plan backlinks.
