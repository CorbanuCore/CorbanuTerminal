---
sprint_id: "PF-55-S01"
title: "Startup and custom-provider convergence"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-55"
execution_order: 14
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-54-S01"
created: 2026-09-01
updated: 2026-09-01
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

- [ ] Plan is active.
- [ ] PF-54-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] Runtime resolution remains authoritative; startup cannot infer from catalog presence.

## Done

- [x] Draft sprint record created and linked to PF-55.

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
