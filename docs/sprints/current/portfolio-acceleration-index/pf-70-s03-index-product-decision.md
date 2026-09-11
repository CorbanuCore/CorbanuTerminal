---
sprint_id: "PF-70-S03"
title: "Index product decision"
status: draft
plan_file: "docs/plans/proposed/portfolio-acceleration-index.md"
plan_feature: "PF-70"
execution_order: 3
owner: "Alex Good (index owner, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-70-S02"
created: 2026-09-09
updated: 2026-09-10
---

# PF-70-S03 — Index product decision

## Execution mandate

- Deliver: docs/research/acceleration-index/decision.md; The next step has a defined audience and authority boundary, or the idea is parked.
- Excludes: Trading, launching tokens, claiming neutrality without a defined hedge, or inventing Alex's holdings and weights.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Index-creation API, replayability and creator ecosystem](../../../plans/proposed/portfolio-acceleration-index.md)
- Feature: `PF-70`; acceptance: The next step has a defined audience and authority boundary, or the idea is parked.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-acceleration-index.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/acceleration-index/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/acceleration-index/pf-70-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-70-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Summarize thesis evidence, tracking error, implementation gaps, economic costs and reasons to reject.
- [ ] Separate research publication, paper portfolio and potential financial product as distinct future scopes.
- [ ] Obtain Alex/Travis decision and legal review requirements for distribution; no launch.
- [ ] Approve or reject general-purpose API delivery using S01/S02; amend product scope/spec before readiness, retaining creator implementation/payouts as excluded.
- [ ] Freeze exact owning repository/commit, paths/tests, API identity/price/budget/retry rules, supported replay guarantees and feature flag; record off-by-default entry-point enforcement and rollback.
- [ ] Allocate S04 only after named implementation/independent acceptance owners, required API/TUI/live-repository tests and release/benchmark applicability are resolved; otherwise leave it draft.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Check every performance statement against the reproducible report and disclosure requirements.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-acceleration-index/` and update plan backlinks.
