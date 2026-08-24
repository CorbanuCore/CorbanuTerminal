---
sprint_id: "PF-11-S04"
title: "Pause revoke and kill escalation"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-11"
execution_order: 63
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-11-S02"
created: 2026-08-23
updated: 2026-08-24
---

# PF-11-S04 — Pause revoke and kill escalation

## Execution mandate

- Deliver: Connect monitor findings to bounded pause, revoke, and kill controls.
- Excludes: implementation owned by any plan feature other than `PF-11`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-11` — Agent Sweep behavioral monitor
- Acceptance advanced: Connect monitor findings to bounded pause, revoke, and kill controls.

## Code boundaries

- Existing: `codex-rs/core/src/guardian/`; `codex-rs/protocol/src/models.rs`; `codex-rs/tui/src/`
- Planned: `codex-rs/security-policy/src/agent_sweep.rs`; `codex-rs/tui/src/agent_sweep/`
- Tests: planned `codex-rs/security-policy/tests/agent_sweep.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-11-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Define deterministic severity-to-action policy and a fail-closed escalation path.
- [ ] Revoke pending authority and halt new sensitive actions without exposing secret material.
- [ ] Persist the decision, affected sessions, recovery prerequisites, and idempotent retry behavior.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy agent_sweep`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; the operator workflow is owned by PF-11-S05.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-11` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
