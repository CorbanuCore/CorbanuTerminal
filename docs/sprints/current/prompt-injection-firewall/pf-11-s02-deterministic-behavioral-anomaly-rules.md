---
sprint_id: "PF-11-S02"
title: "Deterministic behavioral anomaly rules"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-11"
execution_order: 61
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-11-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-11-S02 — Deterministic behavioral anomaly rules

## Execution mandate

- Deliver: Implement explainable rules for suspicious financial-agent behavior.
- Excludes: implementation owned by any plan feature other than `PF-11`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-11` — Agent Sweep behavioral monitor
- Acceptance advanced: Implement explainable rules for suspicious financial-agent behavior.

## Code boundaries

- Existing: `codex-rs/core/src/guardian/`; `codex-rs/protocol/src/models.rs`; `codex-rs/tui/src/`
- Planned: `codex-rs/security-policy/src/agent_sweep.rs`; `codex-rs/tui/src/agent_sweep/`
- Tests: planned `codex-rs/security-policy/tests/agent_sweep.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-11-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Detect action bursts, unexplained asset accumulation, repeated denials, approval probing, and policy drift.
- [ ] Make thresholds profile-bound, deterministic, inspectable, and stable across process restarts.
- [ ] Emit rule identifiers and evidence windows; test benign, suspicious, and boundary sequences.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy agent_sweep`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; the monitor TUI is owned by PF-11-S05.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-11` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
