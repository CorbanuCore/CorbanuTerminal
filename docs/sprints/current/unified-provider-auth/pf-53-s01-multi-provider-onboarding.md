---
sprint_id: "PF-53-S01"
title: "Multi-provider onboarding"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-53"
execution_order: 12
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/provider-auth/src/api_key_flow.rs; codex-rs/provider-auth/src/api_key_flow_tests.rs; codex-rs/provider-auth/src/auth_flow.rs; codex-rs/provider-auth/src/auth_flow_tests.rs; codex-rs/provider-auth/src/lib.rs; codex-rs/tui/src/provider_auth_effect_executor.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/onboarding/provider_setup.rs; codex-rs/tui/src/onboarding/provider_setup_tests.rs; codex-rs/tui/src/onboarding/mod.rs; codex-rs/tui/src/onboarding/auth.rs; codex-rs/tui/src/onboarding/onboarding_screen.rs; codex-rs/tui/src/onboarding/snapshots/; codex-rs/tui/src/config_update.rs; codex-rs/tui/src/lib.rs; codex-rs/tui/src/app.rs; codex-rs/tui/src/app_event.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/chatwidget/wallet_menu.rs; codex-rs/tui/tests/suite/multi_provider_onboarding.rs; codex-rs/tui/tests/suite/mod.rs; docs/sprints/current/unified-provider-auth/pf-53-s01-multi-provider-onboarding.md"
integration_gate: "PF-53 configure-many onboarding, reusable API-key/status host adapters, typed App-level deferred Corbanu continuation, shared in-App provider-list return host, and focused true-TMUX tests only; protected wallet/Plan implementation is reused without duplication; no /providers deactivation UI, startup-wide heuristic removal, credential removal, PF-54+, or untyped secret/event history"
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-52-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-53-S01 — Multi-provider onboarding

## Execution mandate

- Deliver: configure-many onboarding with **Done** and deferred Corbanu Plan execution.
- Excludes: later deactivation controls and startup-wide heuristic removal.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-53`.
- Acceptance advanced: fresh users can set up multiple providers before optional Plan onboarding.

## Code boundaries

- Existing: onboarding screen/auth renderers, config selection edits, app events, wallet/Plan menu.
- Planned: onboarding host adapter, setup queue, deferred Plan handoff, and snapshots/TMUX tests.
- Tests: multiple success orderings, cancel/failure, only-Plan return, default selection, and restart.

## Preconditions

- [x] Plan is active.
- [x] PF-52-S01 is completed at `a723374834` and archived at `1fe8d4ffb9`.
- [x] Exact serial allocation matches the plan.
- [x] App-level continuation and shared in-App provider-list host are authorized;
      wallet/Plan flow remains behind its existing protected boundary.

## Done

- [x] Draft sprint record created and linked to PF-53.

## Remaining

- [ ] Render the shared catalog/status and return to it after every provider attempt.
- [ ] Execute shared API-key effects generically, including mapping OpenAI
      `ApiKeyStorage::OpenAiAuth` to the existing app-server API-key request;
      expose the same executor for PF-54 reuse.
- [ ] Add **Done** and enforce a usable-provider or queued-Plan completion condition.
- [ ] Queue Corbanu without configuring it; run Plan flow only after **Done**.
- [ ] On Plan success activate Corbanu without overriding an existing current provider.
- [ ] On Plan cancel continue when another provider is usable, otherwise return to the list.
- [ ] Preserve existing current selection; on fresh profiles select the first successful provider default.
- [ ] Add snapshots, race tests, and focused true-TMUX success/cancel/recovery/restart scenarios.

## Verification

- [ ] Focused test: onboarding/controller/config and snapshot suites.
- [ ] Integration test: wallet/Plan handoff, persistence, and provider request smoke.
- [ ] TUI: typed-TMUX multi-provider, deferred success, Escape/cancel, only-Plan return, and restart pass.

## Exit evidence

- [ ] Implementation commit and visible checkpoints recorded.
- [ ] TMUX binary hash, keys, artifacts, and canary scan linked.
- [ ] Primary integration review pass recorded within the four-review budget.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
