---
sprint_id: "PF-70-S01"
title: "Existing prototype and index API contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-acceleration-index.md"
plan_feature: "PF-70"
execution_order: 1
owner: "Alex Good (index owner, proposed)"
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

# PF-70-S01 — Existing prototype and index API contract

## Execution mandate

- Deliver: docs/research/acceleration-index/methodology.md; inventory the existing prototype and freeze a general-purpose index API contract, including methodology and replay claims to test.
- Excludes: Trading, launching tokens, claiming neutrality without a defined hedge, or inventing Alex's holdings and weights.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Index-creation API, replayability and creator ecosystem](../../../plans/proposed/portfolio-acceleration-index.md)
- Feature: `PF-70`; acceptance: existing pipeline ownership and bounded request/result/error/recovery contract are explicit; no unsupported determinism promise.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-acceleration-index.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`.
- Planned output: `docs/research/acceleration-index/methodology.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/acceleration-index/pf-70-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Alex supplies prototype/cache/price-pipeline coordinates, read permissions and data rights; approve a sample objective/universe without requiring live stock access.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Obtain Alex's intended thesis/universe; distinguish proposed long-only index and optional hedge hypothesis.
- [ ] Inspect the existing index/SEC/transcript/price code, commits and licenses; identify reusable components and gaps before proposing new code.
- [ ] Define create/retrieve/status/cancel behavior, identity, request ID, input schema, cost ceiling, failures, versioning and retention; resolve exact owning repository and planned code/test paths.
- [ ] Specify hashed cached input packets, model/prompt/runtime/output provenance and the desired replay guarantee; public IPFS upload needs separate rights/approval.
- [ ] Define inclusion, weighting, rebalance, corporate actions, delisting and missing-data rules.
- [ ] Record exposure/access unknowns without blocking the API contract on PF-68; add stock-access diligence before any tradability claim.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Walk valid/invalid/duplicate/interrupted API requests and one sample rebalance against the contract; reproducibility itself is measured in S02.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-acceleration-index/` and update plan backlinks.
