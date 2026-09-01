---
sprint_id: "PF-49-S01"
title: "Provider status and eligibility"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-49"
execution_order: 8
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-48-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-49-S01 — Provider status and eligibility

## Execution mandate

- Deliver: one metadata-only configured/status resolver and durable active/inactive policy.
- Excludes: renderer migration, credential setup execution, and credential deletion.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-49`.
- Acceptance advanced: both hosts and startup can read identical provider state.

## Code boundaries

- Existing: `codex-login::provider_api_key_from_auth_storage`, Claude/Plan status loaders, config edits.
- Planned: provider-auth status snapshots plus typed eligibility persistence/migration.
- Tests: source/status matrices, config round trips, migration, restart, and redaction.

## Preconditions

- [ ] Plan is active.
- [ ] PF-48-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] Persistence and credential boundaries are reviewed before edits.

## Done

- [x] Draft sprint record created and linked to PF-49.

## Remaining

- [ ] Define configured, active/inactive, current, checking, unavailable, and recovery-required states without raw values.
- [ ] Resolve API-key environment/vault, OpenAI account, Claude source, Corbanu Plan, local, and command-auth metadata through typed adapters.
- [ ] Persist explicit inactive provider identities; treat configured providers with no override as active.
- [ ] Migrate existing configured providers to active without changing the current model or forcing login.
- [ ] Make environment-backed removal semantics explicit and preserve config layering.
- [ ] Add failure, ambiguity, partial-state, restart, and secret-canary tests.

## Verification

- [ ] Focused test: provider-auth status/eligibility and config migration suites.
- [ ] Integration test: affected login, vault, config, schema, and model-provider tests.
- [ ] TUI applicability resolved: snapshots are deferred to host sprints; typed status fixtures recorded.

## Exit evidence

- [ ] Implementation commit and state schema recorded.
- [ ] Final-tree test output and migration fixtures linked.
- [ ] No raw credential in debug, error, snapshot, or serialized state artifacts.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
