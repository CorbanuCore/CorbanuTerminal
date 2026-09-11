---
sprint_id: "PF-74-S02"
title: "One evidence-backed draft packet"
status: draft
plan_file: "docs/plans/proposed/portfolio-research-content-distribution.md"
plan_feature: "PF-74"
execution_order: 2
owner: "Alex Good (research/editorial lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-74-S01"
created: 2026-09-09
updated: 2026-09-10
---

# PF-74-S02 — One evidence-backed draft packet

## Execution mandate

- Deliver: docs/research/research-content-distribution/draft-packet.md; An independent reviewer can follow each material claim to evidence and reproduce any calculation.
- Excludes: Publishing, investment recommendations tailored to the user, scraping restricted data without rights, growth spam or claims about unshipped features.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Stock research and content distribution workflow](../../../plans/proposed/portfolio-research-content-distribution.md)
- Feature: `PF-74`; acceptance: An independent reviewer can follow each material claim to evidence and reproduce any calculation.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-research-content-distribution.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/benchmarking/index.md`.
- Planned output: `docs/research/research-content-distribution/draft-packet.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/research-content-distribution/pf-74-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-74-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Research using current primary sources and record conflicting evidence; no factual claim from model memory alone.
- [ ] Draft the one S01-selected private research memo or article/demo outline with reproducible analysis; separate facts, interpretations, valuation assumptions and uncertainty. Other call tickets remain separately owned.
- [ ] Verify every shown Terminal capability against a released candidate; retain source timestamps and corrections.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Line-by-line claim audit and counterthesis review; no publication tool calls.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-research-content-distribution/` and update plan backlinks.
