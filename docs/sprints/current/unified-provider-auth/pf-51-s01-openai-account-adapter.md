---
sprint_id: "PF-51-S01"
title: "OpenAI account auth adapter"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-51"
execution_order: 10
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-50-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-51-S01 — OpenAI account auth adapter

## Execution mandate

- Deliver: OpenAI browser/device account login as an adapter on the shared controller.
- Excludes: changing app-server login protocol, account policy, or API-key behavior.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-51`.
- Acceptance advanced: both hosts share OpenAI login, cancel, completion, and recovery semantics.

## Code boundaries

- Existing: `onboarding/auth.rs` browser/device login and app-server notifications.
- Planned: thin app-server login effect adapter and renderer-neutral status mapping.
- Tests: request correlation, cancellation, stale notifications, device/browser success, and restart.

## Preconditions

- [ ] Plan is active.
- [ ] PF-50-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] Existing app-server request/cancel semantics are preserved.

## Done

- [x] Draft sprint record created and linked to PF-51.

## Remaining

- [ ] Map OpenAI browser and device login into shared typed actions/effects.
- [ ] Correlate request IDs and ignore stale completion after cancel or replacement.
- [ ] Produce metadata-only configured/recovery status through PF-49.
- [ ] Preserve forced-login and existing-account compatibility.
- [ ] Add generalized tests across browser, device-code, cancel, timeout, error, and restart cases.

## Verification

- [ ] Focused test: provider-auth OpenAI adapter and app-server login tests.
- [ ] Integration test: affected codex-login/app-server-client/TUI state tests.
- [ ] TUI applicability resolved: typed event harness passes; visual PTY proof remains with host sprints.

## Exit evidence

- [ ] Implementation commit and app-server adapter contract recorded.
- [ ] Final-tree tests linked.
- [ ] Cancel/stale notification evidence recorded.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
