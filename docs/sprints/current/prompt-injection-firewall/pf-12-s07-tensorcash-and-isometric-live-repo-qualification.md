---
sprint_id: "PF-12-S07"
title: "TensorCash and Isometric live-repo qualification"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 71
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-12-S06"
created: 2026-08-23
updated: 2026-08-23
---

# PF-12-S07 — TensorCash and Isometric live-repo qualification

## Execution mandate

- Deliver: Qualify security behavior while agents perform unconstrained work in both standard live repositories.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Qualify security behavior while agents perform unconstrained work in both standard live repositories.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-12-S06`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Run representative TensorCash development through Corbanu Terminal with hostile data-source fixtures.
- [ ] Run representative Isometric Game visual work through the same profiles and attack conditions.
- [ ] Record task success, policy interventions, runtime, spend, regressions, and recoverable repository diffs.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: both repositories are driven through the real TUI with keys sent.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
