---
sprint_id: "PF-55-S01"
title: "Startup and custom-provider convergence"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-55"
execution_order: 14
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/provider-auth/src/lib.rs; codex-rs/provider-auth/src/runtime_selection.rs; codex-rs/provider-auth/src/runtime_selection_tests.rs; codex-rs/tui/src/startup_provider.rs; codex-rs/tui/src/lib.rs; codex-rs/tui/src/app.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/app/session_lifecycle.rs; codex-rs/tui/src/app/test_support.rs; codex-rs/tui/src/app/tests.rs; codex-rs/tui/src/config_update.rs; codex-rs/tui/src/config_update_tests.rs; codex-rs/tui/src/model_catalog.rs; codex-rs/tui/src/model_catalog_tests.rs; codex-rs/tui/src/chatwidget.rs; codex-rs/tui/src/chatwidget/provider_model_policy.rs; codex-rs/tui/src/chatwidget/model_popups.rs; codex-rs/tui/src/chatwidget/settings.rs; codex-rs/tui/src/chatwidget/slash_dispatch.rs; codex-rs/tui/src/chatwidget/tests/; codex-rs/tui/src/chatwidget/snapshots/; codex-rs/tui/src/spawn_orchestration.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/provider_status_host_tests.rs; codex-rs/tui/tests/suite/provider_convergence.rs; codex-rs/tui/tests/suite/mod.rs; docs/sprints/current/unified-provider-auth/pf-55-s01-startup-provider-convergence.md"
integration_gate: "PF-55 only: converge startup, layered current selection, model picking, resume, and native/child provider use on PF-48 catalog plus PF-49 metadata/eligibility; preserve exact provider identity and first-fresh-success behavior; list any successfully authorized provider regardless of account/API-key/environment/command mechanism; keep failed command auth visible as status/recovery without inventing enrollment; block unusable current selections until explicit repair or replacement; keep resumed-thread replacement session-specific unless the user explicitly changes the global default; and prove custom-provider/restart behavior. Excludes app-server/provider wire-protocol changes, login/vault custody or storage changes, Telegram, interactive command-auth enrollment, PF-56 documentation/release work, and silent provider switching."
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
depends_on: "PF-54-S01"
created: 2026-09-01
updated: 2026-09-02
---

# PF-55-S01 — Startup and custom-provider convergence

## Execution mandate

- Deliver: startup/current-model gating and custom-provider behavior from shared state.
- Excludes: adding interactive setup to custom command-auth providers without an adapter.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-55`.
- Acceptance advanced: startup, both hosts, model selection, and runtime agree after restart.

## Code boundaries

- Existing: `tui/src/app.rs`, config updates, model picker, spawn auth guard, runtime resolver.
- Planned: shared usable-provider/current-selection policy and custom-provider host coverage.
- Tests: upgrades, first-success default, inactive/current mismatch, env custom, command custom, child use.

## Preconditions

- [x] Plan is active.
- [x] PF-54-S01 is completed at signed implementation commit `3db6321294` and archived at `31894e4f1a`.
- [x] Exact serial allocation matches the plan and uses the continuing GPT-5.6 Sol high owner.
- [x] Runtime resolution remains authoritative; startup cannot infer from catalog presence.

## Done

- [x] Draft sprint record created and linked to PF-55.
- [x] Allocated serially in the plan worktree at post-PF-54 base `31894e4f1a` with literal production, test, and TMUX scope.
- [x] Froze provider-use authority as layered exact selection → PF-48 identity → PF-49 status/eligibility → runtime confirmation.
- [x] Froze command-auth parity: a successful real authorization check makes the provider normally visible/selectable regardless of mechanism; failures remain visible for status/recovery with no invented enrollment UI.
- [x] Froze explicit recovery: cancelling an unusable current-provider recovery leaves requests/spawns blocked and never silently switches.
- [x] Froze resumed replacement as session-specific unless the user separately changes the global current selection.

## Remaining

- [ ] Replace `requires_openai_auth`/environment presence heuristics with shared configured/active status.
- [ ] Preserve existing usable current selection and implement first-success fresh-install default.
- [ ] Exclude inactive providers from eligible model choices while retaining management visibility.
- [ ] Auto-list custom `env_key` providers in both hosts and keep unsupported command-auth status-only.
- [ ] Make missing/inactive current provider recovery explicit rather than silently switching.
- [ ] Cover restart/resume, profile/config layering, child/native panes, and generalized custom providers.
- [ ] Add true-TMUX upgrade, custom-provider, recovery, and resumed-request flows.

## Verification

- [ ] Focused test: startup/config/model-provider/spawn guard and custom-provider suites.
- [ ] Integration test: real runtime credential resolution and request selection.
- [ ] TUI: typed-TMUX existing install, custom env provider, inactive current recovery, restart/resume pass.

## Exit evidence

- [ ] Implementation commit and removed heuristic inventory recorded.
- [ ] TMUX binary hash, keys, artifacts, and canary scan linked.
- [ ] Existing-user migration evidence recorded.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
