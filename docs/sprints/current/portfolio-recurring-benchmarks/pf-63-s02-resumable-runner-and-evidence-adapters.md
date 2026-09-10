---
sprint_id: "PF-63-S02"
title: "Resumable runner and evidence adapters"
status: draft
plan_file: "docs/plans/proposed/portfolio-recurring-benchmarks.md"
plan_feature: "PF-63"
execution_order: 2
owner: "Jim Ricketts (proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-63-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-63-S02 — Resumable runner and evidence adapters

## Execution mandate

- Deliver: benchmarks/coding/runner.py and benchmarks/coding/tests/test_runner.py; An interrupted/resumed fixture campaign has the same logical run set as an uninterrupted campaign.
- Excludes: Declaring a harness best without comparable evidence, changing production models, or relaxing existing every-three-releases benchmark policy.
- Budget proposal: 2–4 builder-days plus independent testing; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Repeatable cross-harness quality, runtime and cost benchmarks](../../../plans/proposed/portfolio-recurring-benchmarks.md)
- Feature: `PF-63`; acceptance: An interrupted/resumed fixture campaign has the same logical run set as an uninterrupted campaign.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-recurring-benchmarks.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/benchmarking/index.md`; `docs/benchmarking/coding-tests.md`; `scripts/native-provider-tui-benchmark`; `scripts/hammer-reduction-benchmark`; `benchmarks/README.md`; `benchmarks/coding/`.
- Planned output: `benchmarks/coding/runner.py and benchmarks/coding/tests/test_runner.py`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/recurring-benchmarks/pf-63-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-63-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Extend the canonical benchmarks/coding/runner.py and its tests with missing manifest, attempt identity, restart and artifact seams; scripts/ diagnostics remain diagnostics. Preserve fresh-workspace/source-integrity checks; resumed attempts use new isolated workspaces with an explicit reconciliation manifest.
- [ ] Keep comparator invocation isolated with separate homes/worktrees; collect measured spend or explicit unavailable markers.
- [ ] Add deterministic fixtures for timeout, duplicate resume, malformed result, missing binary, partial log and cap reached.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: From repo root: python3 -m unittest discover -s benchmarks/coding/tests; python3 benchmarks/coding/runner.py --config benchmarks/coding/configs/all-tasks.example.json plan. The plan command makes no paid calls.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-recurring-benchmarks/` and update plan backlinks.
