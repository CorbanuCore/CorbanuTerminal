---
sprint_id: "PF-54-S01"
title: "Provider management and eligibility"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-54"
execution_order: 13
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/provider-auth/src/lib.rs; codex-rs/provider-auth/src/management.rs; codex-rs/provider-auth/src/management_tests.rs; codex-rs/login/src/auth/manager.rs; codex-rs/login/src/auth/provider_key_vault.rs; codex-rs/login/src/auth/auth_tests.rs; codex-rs/login/src/lib.rs; codex-rs/tui/src/lib.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/provider_status_host_tests.rs; codex-rs/tui/src/provider_management_host.rs; codex-rs/tui/src/provider_management_host_tests.rs; codex-rs/tui/src/chatwidget.rs; codex-rs/tui/src/chatwidget/slash_dispatch.rs; codex-rs/tui/src/chatwidget/provider_credentials.rs; codex-rs/tui/src/chatwidget/provider_manager.rs; codex-rs/tui/src/chatwidget/wallet_menu.rs; codex-rs/tui/src/chatwidget/snapshots/; codex-rs/tui/src/slash_command.rs; codex-rs/tui/src/app.rs; codex-rs/tui/src/app/provider_management.rs; codex-rs/tui/src/app/provider_management_auth.rs; codex-rs/tui/src/app/provider_management_status.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/app/test_support.rs; codex-rs/tui/src/app/tests.rs; codex-rs/tui/src/app_event.rs; codex-rs/tui/tests/suite/provider_management.rs; codex-rs/tui/tests/suite/mod.rs; docs/sprints/current/unified-provider-auth/pf-54-s01-provider-management.md"
integration_gate: "PF-54 only: replace the static /providers menu with the PF-48/PF-49 shared catalog and status host; reuse PF-50–PF-52 typed setup and recovery adapters; persist explicit eligibility without credential deletion; require a user-selected usable model/provider replacement before current-provider deactivation; keep cancellation and stale completion inert; accurately describe externally and environment-managed credential removal; preserve explicit inactivity through recovery; and prove restart persistence. Excludes PF-55 startup/model-picker convergence and heuristic removal, all credential deletion, PF-56 final documentation/release work, and provider wire-protocol or credential-custody changes."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-53-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-54-S01 — Provider management and eligibility

## Execution mandate

- Deliver: `/providers` as the shared setup host plus safe activate/deactivate management.
- Excludes: implicit credential deletion and silent current-model replacement.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-54`.
- Acceptance advanced: returning users see the same providers/status and control eligibility safely.

## Code boundaries

- Existing: `chatwidget/provider_credentials.rs`, bottom-pane selection, app events, model picker.
- Planned: management host adapter, eligibility actions, replacement handoff, and removal separation.
- Tests: status parity, activate/deactivate, current replacement/cancel, env-backed removal, restart.

## Preconditions

- [x] Plan is active.
- [x] PF-53-S01 is completed and archived.
- [x] Exact serial allocation matches the plan.
- [x] Credential removal and eligibility mutations are distinct typed effects.

## Done

- [x] Draft sprint record created and linked to PF-54.
- [x] Allocated serially to the GPT-5.6 Sol high implementation owner in the plan worktree.
- [x] Froze eligibility recovery, replacement, cancellation, and credential-retention policy before implementation.
- [x] Authorized `app/provider_management_auth.rs` as a <=220-line correlated auth coordinator split after the main App module exceeded its readability cap.
- [x] Authorized `provider_status_host_tests.rs` to keep regression coverage out of the <400-line production status host.
- [x] Authorized `app/provider_management_status.rs` as a <=220-line nonblocking, generation-correlated status coordinator after TMUX exposed event-loop vault blocking.
- [x] Authorized a secret-free bulk login/vault metadata snapshot after TMUX proved full-catalog status resolution reopened unavailable OS keyring custody once per provider.
- [x] Authorized `chatwidget/wallet_menu.rs` for one stable shared-account-auth view slot after deterministic TMUX proved phase transitions stacked stale pending/challenge views.

## Remaining

- [ ] Replace the static menu and direct status reads with PF-48/PF-49 services.
- [ ] Route all supported setup/recovery actions through PF-50–PF-52.
- [ ] Add deactivate/reactivate without touching credential material.
- [ ] Require an explicit usable replacement before deactivating the current provider.
- [ ] Keep cancel inert and explain environment-backed credential removal accurately.
- [ ] Add snapshots and substantial true-TMUX management, replacement, cancel, and restart tests.

## Verification

- [ ] Focused test: provider menu/controller/eligibility/model-picker suites and snapshots.
- [ ] Integration test: config persistence, runtime eligibility, and retained credential use.
- [ ] TUI: typed-TMUX setup parity, deactivate/reactivate, replacement, cancel, and restart pass.

## Exit evidence

- [ ] Implementation commit and status-parity matrix recorded.
- [ ] TMUX binary hash, keys, artifacts, and canary scan linked.
- [ ] Credential-preservation and explicit replacement evidence recorded.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
