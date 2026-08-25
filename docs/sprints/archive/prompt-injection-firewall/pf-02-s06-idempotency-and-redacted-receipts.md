---
sprint_id: "PF-02-S06"
title: "Idempotency and redacted receipts"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-02"
execution_order: 13
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-02-S05"
created: 2026-08-23
updated: 2026-08-24
---

# PF-02-S06 — Idempotency and redacted receipts

## Execution mandate

- Deliver: Prevent duplicate execution and return durable secret-free results.
- Excludes: implementation owned by any plan feature other than `PF-02`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-02` — Deterministic sensitive-tool executor
- Acceptance advanced: Prevent duplicate execution and return durable secret-free results.

## Code boundaries

- Existing: `codex-rs/tui/src/bottom_pane/approval_overlay.rs`; `codex-rs/protocol/src/models.rs`; `codex-rs/core/src/tools/`
- Planned: `codex-rs/security-policy/src/protected_action.rs`; `codex-rs/security-policy/src/executor.rs`
- Tests: planned `codex-rs/security-policy/tests/protected_action.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-02-S05`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Persist idempotency keys and canonical action digests before side effects.
- [ ] Return typed approved, denied, executed, failed, and unknown-outcome receipts.
- [ ] Test retry, crash recovery, duplicate delivery, partial failure, and audit redaction.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-security-policy protected_action`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-02` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
