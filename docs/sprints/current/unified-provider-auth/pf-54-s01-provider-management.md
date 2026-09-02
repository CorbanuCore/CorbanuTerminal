---
sprint_id: "PF-54-S01"
title: "Provider management and eligibility"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-54"
execution_order: 13
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/provider-auth/src/lib.rs; codex-rs/provider-auth/src/management.rs; codex-rs/provider-auth/src/management_tests.rs; codex-rs/login/src/auth/manager.rs; codex-rs/login/src/auth/provider_key_vault.rs; codex-rs/login/src/auth/auth_tests.rs; codex-rs/login/src/lib.rs; codex-rs/tui/src/lib.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/provider_status_host_tests.rs; codex-rs/tui/src/provider_management_host.rs; codex-rs/tui/src/provider_management_host_tests.rs; codex-rs/tui/src/provider_account_auth_host.rs; codex-rs/tui/src/provider_account_auth_host_tests.rs; codex-rs/tui/src/chatwidget.rs; codex-rs/tui/src/chatwidget/slash_dispatch.rs; codex-rs/tui/src/chatwidget/provider_credentials.rs; codex-rs/tui/src/chatwidget/provider_manager.rs; codex-rs/tui/src/chatwidget/wallet_menu.rs; codex-rs/tui/src/chatwidget/snapshots/; codex-rs/tui/src/slash_command.rs; codex-rs/tui/src/app.rs; codex-rs/tui/src/app/provider_management.rs; codex-rs/tui/src/app/provider_management_auth.rs; codex-rs/tui/src/app/provider_management_status.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/app/test_support.rs; codex-rs/tui/src/app/tests.rs; codex-rs/tui/src/app_event.rs; codex-rs/tui/tests/suite/provider_management.rs; codex-rs/tui/tests/suite/mod.rs; docs/sprints/current/unified-provider-auth/pf-54-s01-provider-management.md"
integration_gate: "PF-54 only: replace the static /providers menu with the PF-48/PF-49 shared catalog and status host; reuse PF-50–PF-52 typed setup and recovery adapters; persist explicit eligibility without credential deletion; require a user-selected usable model/provider replacement before current-provider deactivation; keep cancellation and stale completion inert; accurately describe externally and environment-managed credential removal; preserve explicit inactivity through recovery; and prove restart persistence. Excludes PF-55 startup/model-picker convergence and heuristic removal, all credential deletion, PF-56 final documentation/release work, and provider wire-protocol or credential-custody changes."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-53-S01"
created: 2026-09-01
updated: 2026-09-02
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
- [x] Authorized the existing `provider_account_auth_host.rs` and adjacent tests for the narrow typed Claude RecoveryRequired-to-method-choice presentation repair.
- [x] Replaced the static `/providers` menu and direct per-provider reads with the PF-48 catalog, PF-49 status host, and one bulk metadata refresh.
- [x] Routed API-key, OpenAI account, and Claude account setup/recovery through the PF-50–PF-52 typed adapters with exact attempt correlation.
- [x] Added activate/deactivate persistence without credential deletion; configured providers default active and explicit inactivity survives recovery and restart.
- [x] Required an exact user-selected ready/active replacement to persist before current-provider deactivation; cancellation and stale completion are inert.
- [x] Preserved the selected provider by stable catalog identity across status refresh, with conservative fallback when the provider disappears.
- [x] Derived Claude recovery only from typed RecoveryRequired status plus a configured recoverable credential source; unconfigured and external/nonrecoverable cases do not guess.
- [x] Added snapshot, reducer, host, App-correlation, nonblocking resolver, persistence, redaction, restart, and eight true-TMUX product journeys.
- [x] Completed the final formatted-candidate fix, focused, cap, scope, governance, artifact-hash, and canary gates.

## Remaining

- [ ] Primary owner must create the implementation commit and archive this completed implementation record; this execution owner was explicitly instructed not to commit or archive.
- [ ] PF-55 startup/model-picker/custom-provider convergence and PF-56 integrated documentation/release evidence remain outside PF-54.

## Verification

- [x] `just fix -p codex-provider-auth -p codex-login -p codex-tui`, `just fmt-check`, and `git diff --check` passed on the final tree.
- [x] Focused tests passed: management reducer 6/6; bulk login metadata 3/3; status host 6/6; management host 2/2; account host 3/3; provider-manager/App correlation 7/7; shared-auth stack 1/1; nonblocking status job 1/1; embedded save-to-bulk-refresh boundary 1/1.
- [x] Final serial true-TMUX matrix passed 8/8 in 505.30s: catalog parity; PF-50 API setup/recovery; PF-51 OpenAI cancel/retry; PF-52 Claude recovery/cancel/retry; noncurrent deactivate/reactivate/restart/request retention; current cancel; exact replacement/restart/retention; environment-backed copy/deactivate/reactivate/no deletion.
- [x] Governance passed: `python3 docs/plans/check.py` reported active 2/2 with 0 available slots; `python3 docs/sprints/check.py` reported 61 current and 109 archived before this record changes status or location.
- [x] Production caps passed: status host 393; App management 444; status split 142; auth split 155; management host 200; provider manager 414; account host 356.
- [ ] Primary-owned post-commit governance and clean-tree verification remains pending because this execution owner was instructed not to commit.

## Exit evidence

- [ ] Implementation commit remains intentionally pending for the primary owner; no commit or archive was created by this execution owner.
- [x] Exact formatted candidate: SHA-256 `7bc575da5072b3d959b5474d942972aadddcb841579267bfd1dc40bbe71a037d`; mtime `2026-09-02 00:16:12.214938821 +0000`; size 1,379,184,024 bytes.
- [x] Final success artifacts: `pf54-catalog-parity`, `pf54-api-setup-recovery`, `pf54-openai-cancel-retry`, `pf54-claude-recovery-cancel`, `pf54-noncurrent-inactive`, `pf54-noncurrent-retention-request`, `pf54-current-cancel`, `pf54-current-replacement`, `pf54-current-replacement-restart`, and `pf54-environment-copy`.
- [x] Every final artifact records the exact candidate hash and binary metadata; generated secret canaries were absent from viewport, scrollback, logs, and emitted success artifacts and existed nowhere outside authorized credential custody.
- [x] Credential preservation is proven through restart and a captured authorization-bearing request; environment-backed management copy never claims or performs deletion.
- [x] Status parity is proven for the shared built-in/custom catalog through the same metadata-only host, including post-save managed readiness and environment Present/Invalid/Missing precedence.
- [x] `Done` and `Remaining` reflect the uncommitted but fully qualified implementation state.
- [ ] Completed record archival remains with the primary owner after committing the accepted tree.
