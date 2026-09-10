---
sprint_id: "PF-63-S01"
title: "Comparable campaign specification"
status: draft
plan_file: "docs/plans/proposed/portfolio-recurring-benchmarks.md"
plan_feature: "PF-63"
execution_order: 1
owner: "Jim Ricketts (proposed)"
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

# PF-63-S01 — Comparable campaign specification

## Execution mandate

- Deliver: docs/research/recurring-benchmarks/contract.md; An evaluator can score identical artifacts without knowing the producing model; required release catalog remains included.
- Excludes: Declaring a harness best without comparable evidence, changing production models, or relaxing existing every-three-releases benchmark policy.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Repeatable cross-harness quality, runtime and cost benchmarks](../../../plans/proposed/portfolio-recurring-benchmarks.md)
- Feature: `PF-63`; acceptance: An evaluator can score identical artifacts without knowing the producing model; required release catalog remains included.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-recurring-benchmarks.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/benchmarking/index.md`; `docs/benchmarking/coding-tests.md`; `scripts/native-provider-tui-benchmark`; `scripts/hammer-reduction-benchmark`; `benchmarks/README.md`; `benchmarks/coding/`.
- Planned output: `docs/research/recurring-benchmarks/contract.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/recurring-benchmarks/pf-63-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Approve campaign spend/runtime caps and current comparator versions; existing release thresholds remain binding.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Inventory the checked-in catalog and historical thresholds; select representative trading, coding and recovery tasks without weakening required coverage.
- [ ] Pin harness/model/settings and comparable tool permissions; define repeated-run seeds, outcome rubric and independent evaluator.
- [ ] Specify run identity, timeout, failed attempts, unknown spend, environmental confounders and stop budget.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Dry-score one good and one deliberately broken artifact for every task class.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-recurring-benchmarks/` and update plan backlinks.
