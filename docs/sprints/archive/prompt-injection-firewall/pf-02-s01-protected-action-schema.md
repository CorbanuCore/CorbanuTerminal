---
sprint_id: "PF-02-S01"
title: "Protected-action schema"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-02"
execution_order: 8
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-24
---

# PF-02-S01 — Protected-action schema

## Execution mandate

- Deliver: Define exact schemas for sensitive disclosure and financial operations.
- Excludes: implementation owned by any plan feature other than `PF-02`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-02` — Deterministic sensitive-tool executor
- Acceptance advanced: Define exact schemas for sensitive disclosure and financial operations.

## Code boundaries

- Existing: `codex-rs/tui/src/bottom_pane/approval_overlay.rs`; `codex-rs/protocol/src/models.rs`; `codex-rs/core/src/tools/`
- Planned: `codex-rs/security-policy/src/protected_action.rs`; `codex-rs/security-policy/src/executor.rs`
- Tests: planned `codex-rs/security-policy/tests/protected_action.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Add typed operations for reads, order construction, approval, signing, broadcast, and protected disclosure.
- [ ] Require venue, account reference, asset, side, size, limits, expiry, and idempotency fields where applicable.
- [ ] Reject unknown, missing, ambiguous, free-form shell, JavaScript, and arbitrary-template operations.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy protected_action`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-02` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
