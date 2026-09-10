---
sprint_id: "PF-63-S03"
title: "First bounded campaign and gate rehearsal"
status: draft
plan_file: "docs/plans/proposed/portfolio-recurring-benchmarks.md"
plan_feature: "PF-63"
execution_order: 3
owner: "Jim Ricketts (proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-63-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-63-S03 — First bounded campaign and gate rehearsal

## Execution mandate

- Deliver: qa/portfolio/recurring-benchmarks/qualification.md; A deliberately regressed candidate is rejected; every accepted score has reproducible evidence.
- Excludes: Declaring a harness best without comparable evidence, changing production models, or relaxing existing every-three-releases benchmark policy.
- Budget proposal: 1–2 tester-days after candidate readiness; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Repeatable cross-harness quality, runtime and cost benchmarks](../../../plans/proposed/portfolio-recurring-benchmarks.md)
- Feature: `PF-63`; acceptance: A deliberately regressed candidate is rejected; every accepted score has reproducible evidence.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-recurring-benchmarks.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/benchmarking/index.md`; `docs/benchmarking/coding-tests.md`; `scripts/native-provider-tui-benchmark`; `scripts/hammer-reduction-benchmark`; `benchmarks/README.md`; `benchmarks/coding/`.
- Planned output: `qa/portfolio/recurring-benchmarks/qualification.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/recurring-benchmarks/pf-63-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-63-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Run the approved campaign within the spend cap using pinned candidates; preserve failed attempts and settings.
- [ ] Re-score blinded outputs; compare correctness, latency and cost separately and document uncertainty rather than a single unsupported rank.
- [ ] Inject one regression and prove the release decision rejects it; prepare the next cadence entry without scheduling new jobs.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Re-run failed fixture classes and calculate campaign totals from the manifest independently.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Resolve TUI applicability against the plan; record actual-key success, failure/cancel, recovery/resume and final binary evidence for every affected interactive path.
- [ ] Record expected versus actual results and nonzero test counts; no unchecked assumption is converted into a pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-recurring-benchmarks/` and update plan backlinks.
