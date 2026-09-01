---
sprint_id: "PF-53-S01"
title: "Multi-provider onboarding"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-53"
execution_order: 12
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
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

- [ ] Plan is active.
- [ ] PF-52-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] Wallet/Plan flow is invoked through its existing protected boundary.

## Done

- [x] Draft sprint record created and linked to PF-53.

## Remaining

- [ ] Render the shared catalog/status and return to it after every provider attempt.
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
