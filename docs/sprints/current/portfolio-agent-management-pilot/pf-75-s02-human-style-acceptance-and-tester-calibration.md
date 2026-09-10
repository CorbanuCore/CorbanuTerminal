---
sprint_id: "PF-75-S02"
title: "Human-style acceptance and tester calibration"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-management-pilot.md"
plan_feature: "PF-75"
execution_order: 2
owner: "Jim Ricketts (integration lead, proposed)"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-75-S01"
created: 2026-09-09
updated: 2026-09-09
---

# PF-75-S02 — Human-style acceptance and tester calibration

## Execution mandate

- Deliver: docs/research/agent-management-pilot/acceptance-contract.md; All four seeded failures are detected; a previously qualified positive recovery control passes on the recorded candidate without privileged assistance.
- Excludes: A second scheduler, unattended merges/releases, new task systems, broad credentials, automatic Task Node posting or the claim that testing is bulletproof.
- Budget proposal: 0.5–2 analyst-days after inputs are available; not a commitment. Stop and re-slice if the bound is exceeded.

## Plan linkage

- Plan: [Native agent management and independent acceptance pilot](../../../plans/proposed/portfolio-agent-management-pilot.md)
- Feature: `PF-75`; acceptance: All four seeded failures are detected; a previously qualified positive recovery control passes on the recorded candidate without privileged assistance.
- Upstream/allocation: [plan record](../../../plans/proposed/portfolio-agent-management-pilot.md#native-lifecycle-and-upstream-touch-record).

## Code boundaries

- Existing, read before work: `codex-rs/core/src/agent/control/mailbox.rs`; `codex-rs/core/src/agent/control/execution.rs`; `codex-rs/core/src/agent/registry.rs`; `codex-rs/tasknode-session/src/lib.rs`; `docs/tmuxHarness.md`.
- Planned output: `docs/research/agent-management-pilot/acceptance-contract.md`; no edits outside the allocated scope.
- Tests/evidence: `qa/portfolio/agent-management-pilot/pf-75-s02/` (planned); final literal test/registration scope required before readiness.

## Preconditions

- [ ] Plan active after explicit authority decision; global WIP and occupied lanes reconciled.
- [ ] Dependencies completed and archived; Accepted PF-75-S01 output and its explicit go/no-go; no unresolved decision that changes this mandate.
- [ ] Exact worktree/branch/40-character base match plan; literal scopes and receiving owner are allocated.
- [ ] Inputs, disclosure rights and spend/time limits approved; external writes and live financial actions remain excluded.
- [ ] Resolve exact changed files, native compatibility and nonempty test commands before any code sprint starts.

- [ ] S01 supplies evidence for a previously qualified positive recovery control on this exact candidate. A missing auth implementation is a prerequisite failure, not evidence of an uncalibrated tester; add the receiving auth sprint dependency if using unfinished auth as the positive control.

## Done

- [x] Draft sprint created and linked to one feature; no implementation completed.

## Remaining

- [ ] Write user-only journeys for the four seeded negative cases and the S01-selected qualified positive control. Expired-login negatives test defect detection; they do not qualify unfinished auth.
- [ ] Separate fixture operator from black-box tester; forbid shell repair, hidden profile edits and reading implementation to find recovery controls.
- [ ] Seed unrecoverable login, silently substituted model, double-counted spend and stale-binary evidence; require tester rejection of each before pilot.
- [ ] Record actual outputs, counterexamples, remaining limitations and a concrete next-sprint handoff; stop on changed scope.

## Verification

- [ ] Focused: Use actual-key Corbanu TUI fixtures for applicable journeys with RUST_LOG=trace, isolated homes and final binary hashes.
- [ ] Integration: from repo root, `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Independently inspect source provenance, calculations and the plan's success/failure/recovery table; document-only checks do not qualify product UI.
- [ ] Record reviewer, artifact digest, expected versus actual results and observed cases; identify automation as not applicable when none ran. Never invent a test count or pass.

## Exit evidence

- [ ] Output commit/digest and input provenance recorded; checks linked to that final artifact/tree.
- [ ] Named human/receiving owner accepts the bounded output; needed go/no-go decision is recorded.
- [ ] Handoff includes changed scope, contracts, known gaps and required combined-tree evidence.
- [ ] Done/Remaining ledgers updated honestly; archive accepted record under `docs/sprints/archive/portfolio-agent-management-pilot/` and update plan backlinks.
