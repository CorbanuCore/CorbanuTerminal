---
sprint_id: "PF-70-S04"
title: "Flag-gated index API candidate"
status: draft
plan_file: "docs/plans/proposed/portfolio-acceleration-index.md"
plan_feature: "PF-70"
execution_order: 4
owner: "Implementation owner pending S03"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-70-S03"
created: 2026-09-10
updated: 2026-09-10
---

# PF-70-S04 — Flag-gated index API candidate

## Execution mandate

- Deliver: one disabled-by-default, non-trading index API candidate reusing Alex's identified prototype and the approved S01 contract.
- Excludes: creator claims/payouts/leaderboards, live trades, public launch, unapproved data publication and unrelated pipeline rebuilds.
- Proposed timebox: 2–4 builder-days plus independent QA after discovery; re-slice before readiness if the resolved scope exceeds this bound.

## Plan linkage

- Plan: [Index-creation API, replayability and creator ecosystem](../../../plans/proposed/portfolio-acceleration-index.md)
- Feature: `PF-70`; acceptance: create/retrieve/recover one index without duplicate creation/charge, with explicit provenance and supported replay claims.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-acceleration-index.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing: prototype and API ownership discovered at S01; exact repository/files are unresolved and block readiness.
- Planned: S03 must replace this boundary with literal API handler, request-state, pricing/auth reuse, flag, docs and test paths in the owning repository; no guessed endpoint or second billing/auth system.
- Evidence: `qa/portfolio/acceleration-index/api-candidate.md` and `qa/portfolio/acceleration-index/pf-70-s04/`.

## Preconditions

- [ ] Plan active; PF-70-S03 completed/archived with explicit API scope/spec decision.
- [ ] Exact worktree/branch/base and literal write scope match the plan, including separately authorized external-repository coordinates if needed.
- [ ] S03 resolves contracts, nonzero test commands, integration owner, reviewer, human tester, input rights and approved test budget.
- [ ] All tests use isolated authorized fixtures; live trades, public deployment and creator financial actions remain excluded.

## Done

- [x] Draft linked to PF-70; no runtime implementation or allocation completed.

## Remaining

- [ ] Reuse approved prototype/native auth and accounting; implement bounded creation, status/retrieval and cancellation with versioned inputs/results.
- [ ] Enforce feature-off at discovery and actual execution entry points; direct invocation cannot create work or charges while disabled.
- [ ] Persist request identity and reconcile uncertain execution/billing before retry; report partial/failure states and cost limits clearly.
- [ ] Attach packet/model/methodology/output provenance and only S02-supported replay claims; fail visibly on missing or changed inputs.
- [ ] Add final-candidate API docs/examples, migration/rollback notes and an independent human plan; no unsupported capability enters finished docs.

## Verification

- [ ] Registered nonzero tests cover valid/invalid input, authorization, duplicate/timeout/restart, cancellation, stale data, budget exhaustion and feature-off bypass.
- [ ] Repeat one request across failure/restart; verify no duplicate index or debit and independently reconstruct the supported result.
- [ ] Run actual-key Terminal success/failure/cancel/recovery/resume if integrated; otherwise record S03's API-only applicability decision and equivalent client evidence.
- [ ] Named tester uses only documented controls; record binary/service commit, fixture digests, expected/actual results and applicable live-repository evidence.
- [ ] Final-tree focused/integration tests, `git diff --check`, plan and sprint checks run; inherited errors are not an execution waiver.

## Exit evidence

- [ ] Implementation commit, final test commands/counts, request/recovery evidence and named human acceptance linked.
- [ ] Flag remains off; receiving integration/release/benchmark gates and rollback are recorded without claiming public launch.
- [ ] Done/Remaining accurate; accepted sprint archived and plan backlink updated.
