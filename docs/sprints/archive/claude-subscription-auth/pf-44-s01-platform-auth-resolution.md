---
sprint_id: "PF-44-S01"
title: "Platform-authoritative Claude authentication resolution"
status: completed
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-44"
execution_order: 3
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/cli/src/claude_oauth.rs, codex-rs/cli/src/main.rs, codex-rs/model-provider-info/src/lib.rs, codex-rs/model-provider-info/src/model_provider_info_tests.rs, codex-rs/vault/src/claude_auth.rs, codex-rs/vault/src/lib.rs, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/"
integration_gate: "Jim Ricketts verifies exact-source provider resolution and platform fixtures, archives PF-44-S01, then allocates the migration and recovery UX without exposing credential values."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
branch: "feat/claude-subscription-auth-isolated"
base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
depends_on: "PF-42-S01"
created: 2026-08-30
updated: 2026-08-30
---

# CSA-03 / PF-44-S01 — Platform auth resolution

## Execution mandate

- Deliver: authoritative macOS Keychain and Linux/Windows credentials-file adapters plus exact persisted-source provider resolution.
- Excludes: method-choice TUI, legacy migration decisions, and final release qualification.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/claude-subscription-auth.md).
- Feature: `PF-44` (plan alias `CSA-03`).
- Acceptance advanced: a persisted source controls every request without silent fallback or account/billing-path changes.

## Code boundaries

- Existing: `codex-rs/cli/src/claude_oauth.rs` and the internal provider-auth command in `codex-rs/cli/src/main.rs`.
- Planned: platform-neutral fixture seams, current store precedence, source health, and exact-source resolution.

## Preconditions

- [x] Plan is active.
- [x] PF-42-S01 is completed and archived.
- [x] Worktree, branch, and frozen base commit match the plan.
- [x] Official Keychain/file precedence and `CLAUDE_CONFIG_DIR` behavior are recorded before product code.

## Done

- [x] Sprint record created and linked to PF-44.
- [x] Made macOS Keychain authoritative and Linux/Windows credentials files authoritative, with exact current service/path names.
- [x] Bound managed, environment, and Claude-login selections to their persisted source ID without fallback; selection-less installs retain the historical env-first behavior.
- [x] Classified missing, malformed, blank-refresh, stale, and unavailable credential records without emitting credential values.
- [x] Raised only the Claude provider's outer auth-command budget above its documented refresh and vault-unlock operations.
- [x] Added platform fixtures proving Keychain/file authority, service names, health, source binding, refresh rotation, and redaction.

## Remaining

- [x] None.

## Verification

- [x] `cd codex-rs && just fix -p codex-model-provider-info && just fix -p codex-cli && just fmt` preceded final tests; `git diff --check` passed.
- [x] `cd codex-rs && just test -p codex-model-provider-info --retries 0` passed 57 of 57 tests.
- [x] `cd codex-rs && just test -p codex-cli claude_oauth --retries 0` passed 60 of 60 tests across all five CLI binaries.
- [x] A bounded retry-disabled full CLI run passed 1,438 of 1,443 tests. The two PID-tracker races and two slow entrypoint timeouts passed independently with retries disabled; the unrelated pre-existing `sandbox_fetches_and_enforces_cloud_managed_permission_profile` host-sandbox assertion remained reproducible independently. All 60 auth tests passed within the bounded full run.
- [x] TUI applicability: none in this provider-adapter sprint.

## Exit evidence

- [x] Implementation commit `9382b0e7c` and final-tree tests are recorded above.
- [x] Platform authority behavior and upstream compatibility note are recorded in the plan.
- [x] Done and Remaining reflect reality; completed record is archived.
