---
sprint_id: "PF-76-S01"
title: "Provider profile persistence"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-76"
execution_order: 83
owner: "Alex Good profile-persistence lane"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-09-01
updated: 2026-09-10
---

# PF-76-S01 — Provider profile persistence

Renumbered from P0 PF-43-S01; see the [identity reconciliation](../../identity-reconciliation-2026-09-10.md). Historical evidence and completion state are retained.

Before allocation, reconcile this historical draft with the unified-provider plan's shipped evidence; do not duplicate already completed provider work.

## Execution mandate

- Deliver: Persist provider selection and provider authorization in the active
  named Corbanu profile, with restart durability and cross-profile isolation.
- Excludes: Provider protocol changes, credential disclosure, implicit copying
  from another profile, and unrelated Task Node or security-level work.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-76`
- Acceptance advanced: A provider selected or linked while a named profile is
  active is restored only for that profile after restart.

## Code boundaries

- Existing: `codex-rs/tui/src/lib.rs`, `codex-rs/tui/src/onboarding/auth.rs`,
  `codex-rs/tui/src/onboarding/auth/headless_chatgpt_login.rs`
- Existing: `codex-rs/app-server/src/request_processors/account_processor.rs`,
  `codex-rs/core/src/auth.rs`, `codex-rs/login/src/device_code_auth.rs`
- Planned: shared validated provider-auth profile scope in the config/auth
  boundary, consumed by both TUI and CLI login/status/logout paths
- Tests: focused config, login, app-server, CLI, and TUI profile persistence
  regressions plus true-PTY restart/isolation evidence

## Preconditions

- [x] Plan is active.
- [x] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Product authority defined named-profile provider selection and authorization as durable profile-owned state.
- [x] Existing failure anchored: provider selection persisted while ChatGPT authorization had no durable profile artifact after restart.

## Remaining

- [ ] Release one existing reserved sprint slot, allocate exact worktree coordinates and pass `python3 docs/sprints/check.py`.
- [ ] Add one validated profile-auth scope shared by TUI onboarding and CLI login/status/logout.
- [ ] Persist OpenAI device-code auth and API-key provider credentials under that scope; do not treat transient external auth as a durable link.
- [ ] Restore provider selection and authorization for the same profile after process restart.
- [ ] Deny cross-profile read, use, refresh, status, and logout; keep the unprofiled compatibility scope separate.
- [ ] Surface canceled, failed, missing, or corrupt persistence without advancing onboarding as linked.
- [ ] Add adjacent two-profile, two-provider, cancellation, restart, logout, and unprofiled-fallback regression cases.
- [ ] Run final formatting, focused tests, and a true-PTY two-profile restart flow on the final built binary.

## Verification

- [ ] Focused test: `just test -p codex-login`
- [ ] Integration test: `just test -p codex-app-server`
- [ ] Integration test: `just test -p codex-tui`
- [ ] Integration test: `just test -p codex-cli`
- [ ] TUI applicability resolved; record provider selection, device-code success,
  restart, second-profile isolation, scoped logout, and recovery checkpoints.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] Parallel handoff, if applicable: commit, contract versions, scope audit and combined-tree test evidence recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/p0-security-levels/`.
