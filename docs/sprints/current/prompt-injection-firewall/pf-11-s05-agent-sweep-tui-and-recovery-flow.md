---
sprint_id: "PF-11-S05"
title: "Agent Sweep TUI and recovery flow"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-11"
execution_order: 64
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-11-S02, PF-11-S03, PF-11-S04"
created: 2026-08-23
updated: 2026-08-23
---

# PF-11-S05 — Agent Sweep TUI and recovery flow

## Execution mandate

- Deliver: Ship the key-driven operator workflow for findings, escalation, and recovery.
- Excludes: implementation owned by any plan feature other than `PF-11`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-11` — Agent Sweep behavioral monitor
- Acceptance advanced: Ship the key-driven operator workflow for findings, escalation, and recovery.

## Code boundaries

- Existing: `codex-rs/core/src/guardian/`; `codex-rs/protocol/src/models.rs`; `codex-rs/tui/src/`
- Planned: `codex-rs/security-policy/src/agent_sweep.rs`; `codex-rs/tui/src/agent_sweep/`
- Tests: planned `codex-rs/security-policy/tests/agent_sweep.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-11-S02, PF-11-S03, PF-11-S04`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Render finding source, severity, evidence window, applied containment, and current authority state.
- [ ] Provide keys for inspect, acknowledge, keep paused, revoke, and initiate bounded recovery.
- [ ] Capture true-TUI key transcripts for detection, containment, restart, and recovery failure paths.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy agent_sweep`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: exercise the complete Agent Sweep workflow in a real PTY with keys sent.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-11` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
