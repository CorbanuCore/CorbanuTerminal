---
sprint_id: "PF-08-S06"
title: "Calibration, blind evaluation, and runtime gate"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-08"
execution_order: 51
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-08-S05"
created: 2026-08-23
updated: 2026-08-24
---

# PF-08-S06 — Calibration, blind evaluation, and runtime gate

## Execution mandate

- Deliver: Calibrate thresholds and enforce classifier decisions before main-agent ingestion.
- Excludes: implementation owned by any plan feature other than `PF-08`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-08` — Local prompt-injection classifier
- Acceptance advanced: Calibrate thresholds and enforce classifier decisions before main-agent ingestion.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/`; `qa/security/classifier/`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-08-S05`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Report fixed-FPR, known-family, OOD, surface-heuristic, language, latency, and memory gates.
- [ ] Integrate chunk and aggregate classification at trusted ingress before context materialization.
- [ ] Run shadow mode, enforce approved thresholds, and fail safely when the runtime is unavailable.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-08` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
