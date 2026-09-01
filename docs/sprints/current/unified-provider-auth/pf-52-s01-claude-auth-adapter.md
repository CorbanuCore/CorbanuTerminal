---
sprint_id: "PF-52-S01"
title: "Claude auth adapter"
status: draft
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-52"
execution_order: 11
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-51-S01"
created: 2026-09-01
updated: 2026-09-01
---

# PF-52-S01 — Claude auth adapter

## Execution mandate

- Deliver: the merged Claude subscription choice/recovery backend behind the shared controller.
- Excludes: rewriting Claude credential discovery, source priority, or token custody.

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-52`.
- Acceptance advanced: onboarding and `/providers` invoke exactly the qualified Claude flow.

## Code boundaries

- Existing: `claude_code_login.rs`, vault Claude auth, CLI helper, login external bearer.
- Planned: typed adapter from shared actions/effects to the existing Claude backend.
- Tests: managed-token, Claude Code login, cancel, conflict, 401, replace, restart, and redaction.

## Preconditions

- [ ] Plan is active.
- [ ] PF-51-S01 is completed and archived.
- [ ] Exact serial allocation matches the plan.
- [ ] PF-42–PF-47 invariants and evidence are reviewed before changing adapters.

## Done

- [x] Draft sprint record created and linked to PF-52.

## Remaining

- [ ] Adapt method choice, managed-token entry, Claude Code login, retry, replace, and cancellation.
- [ ] Preserve selected source/account and deterministic 401 recovery without fallback.
- [ ] Reject stale process/output events and keep setup-token output outside host history.
- [ ] Resolve status through PF-49 and remove duplicated host-owned Claude orchestration.
- [ ] Rerun inherited focused and typed state-machine regressions on the changed boundary.

## Verification

- [ ] Focused test: Claude vault, CLI, login, provider, and TUI adapter filters with zero retries.
- [ ] Integration test: inherited provider-auth command/401 and restart/resume tests.
- [ ] TUI applicability resolved: typed Claude state harness passes; final host PTY matrix remains later.

## Exit evidence

- [ ] Implementation commit and PF-42–PF-47 compatibility audit recorded.
- [ ] Final-tree tests and canary scan linked.
- [ ] Any invalidated historical evidence is explicitly identified.
- [ ] `Done` and `Remaining` reflect reality.
- [ ] Completed record archived.
