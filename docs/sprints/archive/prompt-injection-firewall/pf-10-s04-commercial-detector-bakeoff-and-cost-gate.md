---
sprint_id: "PF-10-S04"
title: "Commercial detector bakeoff and cost gate"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-10"
execution_order: 59
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-10-S03"
created: 2026-08-23
updated: 2026-08-24
---

# PF-10-S04 — Commercial detector bakeoff and cost gate

## Execution mandate

- Deliver: Compare hosted candidates against the local baseline on blind identical inputs.
- Excludes: implementation owned by any plan feature other than `PF-10`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-10` — Optional hosted classifier service
- Acceptance advanced: Compare hosted candidates against the local baseline on blind identical inputs.

## Code boundaries

- Existing: `codex-rs/model-provider-info/src/lib.rs`; `codex-rs/vault/src/lib.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/hosted.rs`; `codex-rs/config/src/security.rs`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/hosted.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-10-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Use one adapter harness for accuracy, OOD, false positives, latency, privacy, and cost.
- [ ] Keep vendor tuning separate from final blind holdouts.
- [ ] Require material measured advantage and approved data policy before product enablement.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier hosted`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-10` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
