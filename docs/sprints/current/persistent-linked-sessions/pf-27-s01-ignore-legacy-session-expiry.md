---
sprint_id: "PF-27-S01"
title: "Ignore legacy active-session expiry"
status: in_progress
plan_file: "docs/plans/active/persistent-linked-sessions.md"
plan_feature: "PF-27"
execution_order: 1
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-session-persistence"
branch: "codex/session-persistence"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "none"
created: 2026-08-29
updated: 2026-08-29
---

# PF-27-S01 — Ignore legacy active-session expiry

## Execution mandate

- Deliver: active encrypted linked credentials remain usable until server revocation.
- Excludes: one-time link-request TTLs, unrelated provider auth, and release publication.

## Plan linkage

- Plan: [Persistent linked sessions](../../../plans/active/persistent-linked-sessions.md)
- Feature: `PF-27`
- Acceptance advanced: a past legacy timestamp cannot force browser relinking.

## Code boundaries

- Existing: `codex-rs/tasknode-session/src/lib.rs::ActiveSession`
- Existing: `codex-rs/cli/src/tasknode_cmd.rs::require_active_session`
- Existing: `codex-rs/tui/src/chatwidget/tasknode_menu.rs::ensure_tasknode_session`
- Tests: `codex-rs/tasknode-session/src/tests.rs` and focused CLI/TUI tests

## Preconditions

- [x] Plan is active.
- [x] Dependencies are completed.
- [x] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked only to PF-27.

## Remaining

- [ ] Make legacy expiry metadata informational and never locally authoritative.
- [ ] Remove expiry-only CLI and TUI branches while preserving server rejection.
- [ ] Add generalized regressions for past, future, absent, and malformed legacy metadata.
- [ ] Run formatting, focused tests, and true-TUI qualification.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-tasknode-session`.
- [ ] Format: `cd codex-rs && just fmt`; inspect final diff.
- [ ] Focused test: `cd codex-rs && just test -p codex-tasknode-session`.
- [ ] Affected CLI/TUI tests pass.
- [ ] TUI applicability: required; keys and checkpoints recorded.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test and true-TUI artifacts linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/persistent-linked-sessions/`.
