---
sprint_id: "PF-11-S01"
title: "Behavior-event schema and redaction"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-11"
execution_order: 60
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-24
---

# PF-11-S01 — Behavior-event schema and redaction

## Execution mandate

- Deliver: Define the sanitized behavioral event stream available to the monitor.
- Excludes: implementation owned by any plan feature other than `PF-11`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-11` — Agent Sweep behavioral monitor
- Acceptance advanced: Define the sanitized behavioral event stream available to the monitor.

## Code boundaries

- Existing: `codex-rs/core/src/guardian/`; `codex-rs/protocol/src/models.rs`; `codex-rs/tui/src/`
- Planned: `codex-rs/security-policy/src/agent_sweep.rs`; `codex-rs/tui/src/agent_sweep/`
- Tests: planned `codex-rs/security-policy/tests/agent_sweep.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Define allowlisted event types for actions, approvals, policy decisions, exposures, and position changes.
- [ ] Exclude prompts, secrets, raw tool payloads, and protected financial data from the monitor boundary.
- [ ] Add versioned serialization plus redaction and forbidden-field regression tests.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy agent_sweep`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; the monitor TUI is owned by PF-11-S05.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-11` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
