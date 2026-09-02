---
sprint_id: "PF-55-S01"
title: "Startup and custom-provider convergence"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-55"
execution_order: 14
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "codex-rs/provider-auth/src/lib.rs; codex-rs/provider-auth/src/runtime_selection.rs; codex-rs/provider-auth/src/runtime_selection_tests.rs; codex-rs/tui/src/startup_provider.rs; codex-rs/tui/src/lib.rs; codex-rs/tui/src/app.rs; codex-rs/tui/src/app_server_session.rs; codex-rs/tui/src/app/event_dispatch.rs; codex-rs/tui/src/app/thread_routing.rs; codex-rs/tui/src/app/background_requests.rs; codex-rs/tui/src/app/provider_management_status.rs; codex-rs/tui/src/app/session_lifecycle.rs; codex-rs/tui/src/app/test_support.rs; codex-rs/tui/src/app/tests.rs; codex-rs/tui/src/config_update.rs; codex-rs/tui/src/config_update_tests.rs; codex-rs/tui/src/model_catalog.rs; codex-rs/tui/src/model_catalog_tests.rs; codex-rs/tui/src/chatwidget.rs; codex-rs/tui/src/chatwidget/provider_model_policy.rs; codex-rs/tui/src/chatwidget/model_popups.rs; codex-rs/tui/src/chatwidget/settings.rs; codex-rs/tui/src/chatwidget/slash_dispatch.rs; codex-rs/tui/src/chatwidget/tests/; codex-rs/tui/src/chatwidget/snapshots/; codex-rs/tui/src/spawn_orchestration.rs; codex-rs/tui/src/provider_status_host.rs; codex-rs/tui/src/provider_status_host_tests.rs; codex-rs/tui/tests/suite/provider_convergence.rs; codex-rs/tui/tests/suite/mod.rs; docs/sprints/current/unified-provider-auth/pf-55-s01-startup-provider-convergence.md"
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
- [x] Expanded scope to redirect thread-session `ListSkills` refreshes through the existing nonblocking request-handle/result-event path after true-TMUX proved the blocking RPC starved terminal input before `UserTurn` dispatch.
- [x] Expanded scope to delete the zero-caller crate-private `AppServerSession::skills_list` wrapper made obsolete by that request-handle redirect; server-side skills APIs remain unchanged.
- [x] Expanded scope to `tui/src/app/provider_management_status.rs` after true-TMUX proved manager open rebuilt the status host and discarded secret-free command runtime authorization; reuse the shared host and retain config reconstruction only as an unchecked fallback.

- [x] Replaced startup, current-model, picker, resume, and spawn heuristics with the shared typed provider catalog/status/runtime-authorization policy while preserving exact provider identity.
- [x] Preserved existing usable current selection, first-success fresh-install default, profile/config layering, and session-specific resumed replacement.
- [x] Excluded inactive providers from normal selection while retaining management visibility and explicit recovery without silent switching.
- [x] Converged custom environment, managed-key, and command-auth providers across startup, provider management, model picking, requests, restart, resume, and native child use.
- [x] Preserved real command authorization in provider-management status without persisting or logging tokens or raw command output.

## Remaining

- [x] Primary created signed implementation commit `21cf3199f2`, pushed it to `origin/feat/unified-provider-auth`, and archived this completed sprint.

## Verification

- [x] Focused suites: provider status 9/9, runtime selection 6/6, startup 3/3, model catalog 5/5, shared picker policy 1/1, spawn guards 5/5, inactive selection 1/1, and manager authorization reuse/fallback 1/1.
- [x] Integration: real loopback runtime requests passed for existing, environment, managed, replacement, resumed, native-child, command-auth, and duplicate-slug exact-provider paths.
- [x] TUI: 12/12 serial typed-TMUX journeys passed on one exact rebuilt candidate.
- [x] Primary verified the signed implementation commit and completed-sprint archive.

## Exit evidence

- [x] Signed implementation commit `21cf3199f2` records the converged provider runtime-selection policy, startup/picker/resume/native-use adapters, nonblocking skills refresh, exact custom model identity, command-auth status propagation, and associated regressions.
- [x] TMUX candidate SHA256 `d5c51203779224704487a6596213062b77a15db6e0b55172adfe71cae2dba944`, size `1379543376`, mtime `2026-09-02 19:52:42.853149984 +0000`; success artifacts `pf55-upgrade`, `pf55-freshfirstsuccess`, `pf55-environment`, `pf55-managedrestart`, `pf55-inactivenoncurrent`, `pf55-inactivecurrentcancel`, `pf55-exactreplacement`, `pf55-missingcurrent`, `pf55-resume`, `pf55-nativespawn`, `pf55-commandauth`, and `pf55-duplicateslug` each record that SHA.
- [x] Seven generated credential canaries were absent from terminal evidence, non-custody files, and success artifacts.
- [x] Existing-install upgrade, restart retention, and first-success migration evidence passed.
- [x] `Done` and `Remaining` reflect reality.
- [x] Primary integration audit found no blocking issue; this completed record is archived and the branch is backed up on origin.
