---
sprint_id: "PF-08-S01"
title: "Classifier adapter and result schema"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-08"
execution_order: 46
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-08-S01 — Classifier adapter and result schema

## Execution mandate

- Deliver: Define one local detector interface and versioned structured verdict.
- Excludes: implementation owned by any plan feature other than `PF-08`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-08` — Local prompt-injection classifier
- Acceptance advanced: Define one local detector interface and versioned structured verdict.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/`; `qa/security/classifier/`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Add benign, suspicious, and hostile verdicts plus attack classes, confidence, model version, and input digest.
- [ ] Bound input length, batching, timeout, and cancellation semantics.
- [ ] Test malformed model output, version mismatch, NaN scores, timeout, and unavailable runtime.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-08` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
