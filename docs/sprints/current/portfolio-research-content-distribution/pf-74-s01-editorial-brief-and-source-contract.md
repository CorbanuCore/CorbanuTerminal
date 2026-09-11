---
sprint_id: "PF-74-S01"
title: "Editorial brief and source contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-research-content-distribution.md"
plan_feature: "PF-74"
execution_order: 1
owner: "Alex Good (research/editorial lead, proposed)"
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

# PF-74-S01 — Editorial brief and source contract

## Execution mandate

- Deliver: docs/research/research-content-distribution/brief.md; The brief permits a negative conclusion and does not promise a preferred stock outcome.
- Excludes: Publishing, investment recommendations tailored to the user, scraping restricted data without rights, growth spam or claims about unshipped features.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Stock research and content distribution workflow](../../../plans/proposed/portfolio-research-content-distribution.md)
- Feature: `PF-74`; acceptance: The brief permits a negative conclusion and does not promise a preferred stock outcome.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-research-content-distribution.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/corbanu-product-spec.md`; `docs/benchmarking/index.md`.
- Planned output: `docs/research/research-content-distribution/brief.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/research-content-distribution/pf-74-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Alex selects topic/audience; data rights, conflicts and financial-content review must be resolved before any public release.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Select one bounded exemplar from the September 10 register with its owner: Travis/SSD; Alex/materials-robotics; Alex/Apple-hedge. Confirm timing/budget and private-memo versus editorial audience; no automatic dispatch of all three.
- [ ] Define thesis/counterthesis, rejection criteria and Terminal learning outcome; Apple research must quantify the local-AI-beneficiary counterargument and allow no trade.
- [ ] Specify approved primary sources, dates, licensing, material-claim ledger and conflict disclosures.
- [ ] Write acceptance and publication authority criteria before drafting.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Reviewer checks source rights and confirms every planned claim can be evidenced.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-research-content-distribution/` and update plan backlinks.
