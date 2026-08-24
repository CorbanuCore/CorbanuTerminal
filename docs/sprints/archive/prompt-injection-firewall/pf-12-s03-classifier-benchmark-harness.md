---
sprint_id: "PF-12-S03"
title: "Classifier benchmark harness"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 67
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-08-S05, PF-12-S01"
created: 2026-08-23
updated: 2026-08-24
---

# PF-12-S03 — Classifier benchmark harness

## Execution mandate

- Deliver: Measure local classifier quality, latency, memory, and package size on release hardware.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Measure local classifier quality, latency, memory, and package size on release hardware.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-08-S05, PF-12-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Report recall, precision, false-positive rate, calibration, and per-modality confusion slices.
- [ ] Measure cold/warm CPU latency, peak memory, model size, and throughput on named hardware.
- [ ] Define versioned thresholds and emit machine-readable artifacts that block regression.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not applicable; this is a deterministic benchmark unit.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
