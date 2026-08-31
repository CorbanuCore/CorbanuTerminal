---
sprint_id: "PF-45-S01"
title: "Claude authentication choice, migration, and recovery"
status: completed
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-45"
execution_order: 4
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/vault/src/claude_auth.rs, codex-rs/vault/src/claude_auth_tests.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/chatwidget/claude_code_login.rs, codex-rs/tui/src/chatwidget/claude_code_login_tests.rs, codex-rs/tui/src/chatwidget/provider_credentials.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/tui/tests/support/tmux.rs, codex-rs/tui/tests/suite/claude_auth.rs, codex-rs/tui/tests/suite/mod.rs, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/"
integration_gate: "Jim Ricketts verifies explicit choice, secure enrollment, reversible migration, cancel/failure/retry/resume snapshots, archives PF-45-S01, then allocates final qualification."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
branch: "feat/claude-subscription-auth-isolated"
base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
depends_on: "PF-43-S01, PF-44-S01"
created: 2026-08-30
updated: 2026-08-30
---

# CSA-04 / PF-45-S01 — Auth choice and recovery

## Execution mandate

- Deliver: an explicit, concise method choice with official long-lived token enrollment recommended by default and Claude Code login as the compatibility alternative.
- Deliver: reversible selection migration plus clear success, cancel, failure, recovery, retry, and resumed-session behavior.
- Excludes: final live-repository qualification, durable operator docs, human acceptance, and release decisions.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/claude-subscription-auth.md).
- Feature: `PF-45` (plan alias `CSA-04`).
- Acceptance advanced: the user explicitly chooses the persisted source and never experiences silent identity or billing-path fallback.

## Code boundaries

- Existing: provider credentials menu, Claude Code login orchestration, masked vault entry, and CSA-01 through CSA-03 vault/provider contracts.
- Planned: choice view, setup-token handoff, atomic selection commit/rollback in the established vault, source status, and recovery snapshots within the existing TUI seam.

## Preconditions

- [x] Plan is active.
- [x] PF-43-S01 and PF-44-S01 are completed and archived.
- [x] Worktree, branch, and frozen base commit match the plan.
- [x] Serial scope and integration gate are recorded.

## Done

- [x] Sprint record created and linked to PF-45.
- [x] Presents the recommended long-lived `claude setup-token` flow first and
  the existing Claude Code login as an explicit compatibility alternative.
- [x] Uses a masked, zeroizing handoff into the encrypted vault and commits the
  exact selected source only after successful enrollment or verified login.
- [x] Preserves the previous source and unrelated credentials across cancel,
  setup failure, masked-entry cancel, and injected vault partial failures.
- [x] Provides source-specific status, retry, choose-method, and keep-current
  recovery actions without automatic fallback.
- [x] Covers choice and recovery with stable snapshots and typed private-tmux
  success, cancel, failure, retry, compatibility, and resumed-session flows.

## Remaining

- None for PF-45.

Final cross-platform/live-repository qualification, documentation, external
human acceptance, and release decisions belong to PF-46 / CSA-05.

## Verification

- [x] `just fix -p codex-tui`, `just fmt`, and `git diff --check` passed before
  the final targeted tests.
- [x] `just test -p codex-tui claude_code_login -j 1 --retries 0` passed 12/12
  without snapshot updates; `provider_credentials` passed 9/9.
- [x] `CORBANU_TMUX_REQUIRED=1
  CORBANU_TMUX_ARTIFACT_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-subscription-auth/tmux-csa04-run-20260830-1
  just test -p codex-tui --test all tmux_claude_auth -j 1 --retries 0`
  passed 2/2 (compatibility 23.317s; managed state machine 55.470s).
- [x] User-facing TUI applicability: required; inspected snapshots cover the
  method choice and source-specific recovery menu.
- [x] Candidate reported `corbanu 0.1.35`; binary SHA-256 was
  `21d05d33822c42e7e9b9e70ec1f96936bc37f9c045205cff3155fef4fa081816`.
- [x] The harness asserted its exact generated canary was absent from viewport,
  scrollback, isolated home/logs, and artifacts; an independent artifact scan
  found no `synthetic-claude-oauth-` marker. A successful run emitted no
  failure artifacts.
- [x] Lifecycle checkers passed after the implementation commit and archive.

## Exit evidence

- [x] Implementation commits `0913d38ed` and `446806526` plus typed-tmux
  harness commit `dff693e46` are recorded on the isolated feature branch.
- [x] Migration/rollback behavior and recovery copy are recorded in the plan.
- [x] Done and Remaining reflect reality; completed record is archived.
