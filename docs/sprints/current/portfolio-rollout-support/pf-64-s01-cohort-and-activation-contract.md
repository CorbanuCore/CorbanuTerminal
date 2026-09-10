---
sprint_id: "PF-64-S01"
title: "Cohort and activation contract"
status: draft
plan_file: "docs/plans/proposed/portfolio-rollout-support.md"
plan_feature: "PF-64"
execution_order: 1
owner: "Alex Good (adoption) / Jim Ricketts (support), proposed"
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

# PF-64-S01 — Cohort and activation contract

## Execution mandate

- Deliver: docs/research/rollout-support/cohort.md; Every metric has an event definition, consent basis and owner; no commercial target is invented.
- Excludes: Sending invitations, issuing subsidized Plans, posting marketing, collecting unconsented telemetry or announcing unfinished security.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Controlled rollout, support ownership and adoption](../../../plans/proposed/portfolio-rollout-support.md)
- Feature: `PF-64`; acceptance: Every metric has an event definition, consent basis and owner; no commercial target is invented.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-rollout-support.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/features/tasknode.md`; `docs/features/wallet-plan.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/rollout-support/cohort.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/rollout-support/pf-64-s01/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Choose cohort size, support owner, consent wording and any subsidy budget; live rollout requires a separate explicit decision.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Define one persona and one useful first-task outcome; separate download/signup from genuine activation.
- [ ] Specify proposed cohort limit, subsidy cap, opt-in events, retention period and privacy-preserving identifiers.
- [ ] Map blockers to existing security/auth work and name the human decisions required before any invitation.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Review each event for prompt/credential/financial-data disclosure; validate funnel on synthetic events.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-rollout-support/` and update plan backlinks.
