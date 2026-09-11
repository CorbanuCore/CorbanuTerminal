---
sprint_id: "PF-79-S02"
title: "Public beta assignment, evidence and recurring triage pilot"
status: draft
plan_file: "docs/plans/active/initiative-delivery-control.md"
plan_feature: "PF-79"
execution_order: 3
owner: "UNALLOCATED — Task Node integration owner assigns before ready"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-79-S01"
created: 2026-09-11
updated: 2026-09-11
---

# PF-79-S02 — Public beta assignment, evidence and recurring triage pilot

## Execution mandate

- Deliver: one bounded native public-assignment adapter and two-week manual-testing pilot with evidence triage.
- Excludes: new scheduler/marketplace, auto acceptance/rewards/releases, general recruitment and unbounded task generation.
- Estimate: 3–5 setup days plus two-week observation; use existing Task Node/public-card infrastructure or re-slice.

## Plan linkage

- [Workstream 3](../../../plans/active/initiative-delivery-control.md), feature PF-79.
- [Detailed contract](../../../plans/tasknode-beta-program.md), recurring workflow and acceptance matrix.
- Product citation: **Internal delivery control — TO BUILD**, “ongoing public manual-testing beta program”.

## Code boundaries

- Existing: `scripts/initiative_control/control.py::collect/safe_markdown` and `scripts/initiative_control/tasknode.py`; native `codex-rs/cli/src/tasknode_cmd.rs` contracts.
- Planned: `scripts/initiative_control/beta.py`, `test_beta.py`; `qa/beta-program/` pilot manifests/receipts and operator runbook.
- Reuse S01 candidate/card schemas and supported scoped board command; no direct DB writes or broad worker session copies.
- Allocate Desktop or native CLI edits explicitly if an existing adapter cannot satisfy the frozen contract; do not infer unlimited scope.

## Preconditions

- [ ] S01 completed/archived; exact candidate, safe cards, native scope and public/evidence destinations verified.
- [ ] Assign one manager/publisher, independent reviewer, builder and human escalation owner with exact worktree/scope/integration gate.
- [ ] Board owner approves beta routing lane; Travis approves cohort, budget/reward semantics, public payloads, retention and downstream lifecycle.
- [ ] Agree pilot cap: one OS, three cases × two humans × two weekly candidates = at most 12 assignments; no cadence starts by planning alone.

## Done

- [x] Drafted pilot caps, approval boundary, evidence rules, queue budget and stop/exit criteria.

## Remaining

- [ ] Implement default-OFF manifest-to-native-request adapter, validating S01 schema and explicit public-payload approval.
- [ ] Persist idempotency before send; reconcile durable request/generated task/assignee receipts across timeouts and restarts.
- [ ] Keep two deliberate tester slots separate; handle claim conflicts, no-shows and stale/superseded builds without silent cancellation or duplicate creation.
- [ ] Verify signed-out public card visibility and generated wording before cohort handoff; reject truncated/changed safety requirements.
- [ ] Separate service lifecycle, evidence review, product outcome and economic outcome; respect native verification/reward contracts without extra authority.
- [ ] Quarantine untrusted evidence, check candidate identity and privacy, then queue independent human review; missing evidence is blocked, not a failed tester.
- [ ] Project counts/candidate/blockers/public-card links/next reviewer onto private dashboard; worker run logs remain separate and redacted.
- [ ] Rehearse locally first; obtain explicit launch approval and publish only individually approved assignments within cap.
- [ ] Operate approved two-week pilot: top-three daily decisions, at most 20 minutes/day normal triage; pause new offers for overdue review or exhausted budget.
- [ ] Record defect ownership, supported user recovery, review load and approve/hold operating decision; no automatic continuation beyond pilot.

## Verification

- [ ] Unit/integration tests: send timeout/restart/retry uses same key; claim race, no-show, duplicate and changed payload conflicts are visible.
- [ ] Seed wrong-build/false-pass, unsafe attachment, inaccessible evidence and broken auth recovery; independent reviewer detects each.
- [ ] Verify wrong-board/expired credentials fail closed; relink/re-provision only through supported operator flow, never secret-file repair.
- [ ] Verify OFF prevents new sends, in-flight outcomes reconcile and already-public obligations remain visible.
- [ ] Real GUI signed-out public card + signed-in tester flow and exact Desktop candidate; true-TUI checks where applicable, no fixture-only acceptance.
- [ ] Record actual commands for `test_beta.py`, affected control tests, governance checks and final candidate regression tests after allocation.
- [ ] Reconcile all 12 assignment slots including blocked/declined/no-show; independent review of critical cases, no critical security/data-loss defect open.

## Exit evidence

- [ ] Implementation commit, contract/candidate digests, final tests and native public-task receipts linked without private evidence or credentials.
- [ ] Publication approval, scope/budget receipts, human outcomes and review-load/defect report recorded.
- [ ] Travis records continue/change/hold decision; missing coverage is blocked and beta results do not waive stable release gates.
- [ ] Update Done/Remaining and archive only after required acceptance; preserve unresolved issues and public obligations.
