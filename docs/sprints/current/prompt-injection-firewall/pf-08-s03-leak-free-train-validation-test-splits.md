---
sprint_id: "PF-08-S03"
title: "Leak-free train/validation/test splits"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-08"
execution_order: 48
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-08-S02"
created: 2026-08-23
updated: 2026-08-23
---

# PF-08-S03 — Leak-free train/validation/test splits

## Execution mandate

- Deliver: Group splits by source, base document, template, attack family, and semantic duplicate.
- Excludes: implementation owned by any plan feature other than `PF-08`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-08` — Local prompt-injection classifier
- Acceptance advanced: Group splits by source, base document, template, attack family, and semantic duplicate.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/`; `qa/security/classifier/`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-08-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Build train, calibration, validation, known-family test, and blind OOD holdouts.
- [ ] Keep transformed variants and paraphrases of one base item in one split.
- [ ] Report source, topic, language, position, and evasion coverage with leakage failures.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-08` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
