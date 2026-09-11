---
sprint_id: "PF-64-S03"
title: "Opt-in pilot readiness decision"
status: draft
plan_file: "docs/plans/proposed/portfolio-rollout-support.md"
plan_feature: "PF-64"
execution_order: 3
owner: "Alex Good (adoption) / Jim Ricketts (support), proposed"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-64-S02"
created: 2026-09-09
updated: 2026-09-09
---

# PF-64-S03 — Opt-in pilot readiness decision

## Execution mandate

- Deliver: docs/research/rollout-support/decision.md; The decision identifies a release candidate and human support coverage; no pilot is claimed to have run.
- Excludes: Sending invitations, issuing subsidized Plans, posting marketing, collecting unconsented telemetry or announcing unfinished security.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Controlled rollout, support ownership and adoption](../../../plans/proposed/portfolio-rollout-support.md)
- Feature: `PF-64`; acceptance: The decision identifies a release candidate and human support coverage; no pilot is claimed to have run.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-rollout-support.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `docs/features/tasknode.md`; `docs/features/wallet-plan.md`; `docs/features/model-providers.md`.
- Planned output: `docs/research/rollout-support/decision.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/rollout-support/pf-64-s03/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-64-S02 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Assemble release evidence, support rehearsal, draft user message and budget request without sending them.
- [ ] Define a bounded pilot measurement window and rollback/stop rules; require approval for actual participants and spend.
- [ ] Have product and support owners accept or defer readiness; document exactly which evidence remains pending.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Cross-check all advertised capabilities against final candidate evidence and user-facing docs.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-rollout-support/` and update plan backlinks.
