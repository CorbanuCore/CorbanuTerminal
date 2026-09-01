---
sprint_id: "PF-50-S01"
title: "Shared API-key flow controller"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-50"
execution_order: 9
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-49-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-50-S01 — Shared API-key flow controller

## Execution mandate

- Deliver: renderer-independent typed provider-auth transitions and the API-key adapter.
- Excludes: OpenAI, Claude, Corbanu Plan, and full host rendering.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-50`.
- Acceptance advanced: API-key setup behaves identically from either host.

## Code boundaries

- Existing: onboarding API-key states and `chatwidget/provider_credentials.rs` vault entry.
- Planned: provider-auth flow state/actions/effects plus thin TUI event adapters.
- Tests: transition tables, cancellation races, stale completion, storage failure, and masking.

## Preconditions

- [ ] Plan is active.
- [ ] PF-49-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] App-server/vault persistence is the sole raw-secret effect boundary.

## Done

- [x] Draft sprint record created and linked to PF-50.

## Remaining

- [ ] Define typed start, input, submit, success, failure, cancel, retry, replace, and completion effects.
- [ ] Move API-key policy from both renderers into the shared controller and approved storage adapter.
- [ ] Reject stale results after cancellation or a newer attempt.
- [ ] Return metadata-only status and provider identity after successful persistence.
- [ ] Cover built-in and custom `env_key` providers without provider-specific branches.
- [ ] Add generalized regression tests for adjacent API-key providers and paraphrase-free protocol events.

## Verification

- [ ] Focused test: provider-auth controller/API-key transition suite.
- [ ] Integration test: vault/app-server save and runtime resolution tests.
- [ ] TUI applicability resolved: a minimal typed host harness proves effects; full PTY flows remain PF-53/PF-54.

## Exit evidence

- [ ] Implementation commit and controller protocol version recorded.
- [ ] Final-tree tests and secret-canary results linked.
- [ ] Cancellation and stale-result evidence recorded.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
