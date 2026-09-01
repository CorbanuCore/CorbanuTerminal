---
sprint_id: "PF-54-S01"
title: "Provider management and eligibility"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-54"
execution_order: 13
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
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

- [ ] Plan is active.
- [ ] PF-53-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] Credential removal and eligibility mutations are distinct typed effects.

## Done

- [x] Draft sprint record created and linked to PF-54.

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
