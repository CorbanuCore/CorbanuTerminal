---
sprint_id: "PF-02-S04"
title: "Approval binding and trusted preview"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-02"
execution_order: 11
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-02-S03"
created: 2026-08-23
updated: 2026-08-23
---

# PF-02-S04 — Approval binding and trusted preview

## Execution mandate

- Deliver: Bind human approval to the exact canonical action shown in the TUI.
- Excludes: implementation owned by any plan feature other than `PF-02`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-02` — Deterministic sensitive-tool executor
- Acceptance advanced: Bind human approval to the exact canonical action shown in the TUI.

## Code boundaries

- Existing: `codex-rs/tui/src/bottom_pane/approval_overlay.rs`; `codex-rs/protocol/src/models.rs`; `codex-rs/core/src/tools/`
- Planned: `codex-rs/security-policy/src/protected_action.rs`; `codex-rs/security-policy/src/executor.rs`
- Tests: planned `codex-rs/security-policy/tests/protected_action.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-02-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Render a complete effect preview from canonical fields on the trusted surface.
- [ ] Store approval digest, actor, scope, expiry, and one-use state outside model control.
- [ ] Reject mutation, replay, stale approval, duplicate submission, and mismatched preview.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy protected_action`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: preview, approve, cancel, mutate, expire, and resume with keys sent.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-02` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
