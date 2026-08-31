---
sprint_id: "PF-45-S01"
title: "Claude authentication choice, migration, and recovery"
status: in_progress
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-45"
execution_order: 4
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/vault/src/claude_auth.rs, codex-rs/vault/src/claude_auth_tests.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/chatwidget/claude_code_login.rs, codex-rs/tui/src/chatwidget/claude_code_login_tests.rs, codex-rs/tui/src/chatwidget/provider_credentials.rs, codex-rs/tui/src/chatwidget/snapshots/, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/"
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

## Remaining

- [ ] Present the recommended long-lived setup-token flow first/default with a concise eligibility/lifetime explanation and Claude login alternative.
- [ ] Store enrollment output only through the encrypted vault, then persist the chosen exact source after success.
- [ ] Preserve the prior selection and all unrelated credentials on cancel or partial failure; expose explicit replacement/switch actions.
- [ ] Add deterministic success, cancel, failure, retry, conflict, and resumed-session tests plus user-visible snapshots.

## Verification

- [ ] `cd codex-rs && just fix -p codex-tui && just fmt` precedes final tests.
- [ ] Focused retry-disabled TUI tests and snapshots pass.
- [ ] User-facing TUI applicability: required; snapshots cover choice and recovery states.

## Exit evidence

- [ ] Implementation commit and final-tree tests recorded.
- [ ] Migration/rollback behavior and recovery copy recorded in the plan.
- [ ] Done and Remaining reflect reality; completed record is archived.
