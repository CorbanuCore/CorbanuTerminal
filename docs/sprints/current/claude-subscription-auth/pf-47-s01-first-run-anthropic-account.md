---
sprint_id: "PF-47-S01"
title: "First-run Anthropic-account onboarding"
status: in_progress
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-47"
execution_order: 6
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/chatwidget/claude_code_login.rs, codex-rs/tui/src/chatwidget/claude_code_login_tests.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/tui/src/config_update.rs, codex-rs/tui/src/config_update_tests.rs, codex-rs/tui/src/lib.rs, codex-rs/tui/src/onboarding/auth.rs, codex-rs/tui/src/onboarding/onboarding_screen.rs, codex-rs/tui/src/onboarding/snapshots/, codex-rs/tui/tests/suite/claude_auth.rs, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/, docs/sprints/archive/claude-subscription-auth/, docs/sprints/index.md, qa/claude-subscription-auth/"
integration_gate: "Jim Ricketts verifies the isolated final candidate from frozen base 8ae13e, closes structured and Corbanu/Opus 5 Max reviews, archives PF-47-S01, and pushes only feat/claude-subscription-auth-isolated."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
branch: "feat/claude-subscription-auth-isolated"
base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
depends_on: "PF-46-S01"
created: 2026-08-31
updated: 2026-08-31
---

# CSA-06 / PF-47-S01 — First-run Anthropic-account onboarding

## Execution mandate

- Deliver: first-run onboarding offers an Anthropic Claude account and routes
  it into the existing explicit stable-token or Claude Code login flow.
- Excludes: changing Anthropic API-key behavior, provider wire contracts,
  credential custody, or unrelated onboarding providers.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/claude-subscription-auth.md).
- Feature: `PF-47` (plan alias `CSA-06`).
- Acceptance advanced: a fresh user can enroll an eligible personal, Team, or
  Enterprise Anthropic account without first navigating to `/providers`.

## Code boundaries

- Existing: `onboarding::auth::SignInOption`, `OnboardingResult`, and the
  qualified Claude auth-choice and persistence flow under `chatwidget`.
- Planned: a typed onboarding handoff that reuses the existing Claude flow and
  commits `claude-plan` only after successful account-method selection.
- Tests: onboarding unit/snapshot coverage and retry-disabled typed-Tmux startup
  flows for recommended, compatibility, cancel, persistence, and restart.

## Preconditions

- [x] Plan is active.
- [x] PF-46-S01 is completed and archived.
- [x] Worktree, isolated branch, and frozen base commit match the plan.
- [x] Owner, serial lane, literal scope, and integration gate are recorded.

## Done

- [x] Sprint record created and linked to PF-47.

## Remaining

- [ ] Add a distinct Anthropic-account option to first-run onboarding.
- [ ] Route selection through the existing explicit Claude authentication flow.
- [ ] Persist provider selection only after success; keep cancel inert.
- [ ] Add onboarding unit, snapshot, and typed-Tmux regression coverage.
- [ ] Run affected tests, Opus 5 Max autoreview, push, rebuild, and relaunch.

## Verification

- [ ] Focused test: `CARGO_INCREMENTAL=0 just test -p codex-tui onboarding -j 1 --retries 0`.
- [ ] Integration test: retry-disabled `codex-tui` affected suite on the final tree.
- [ ] TUI applicability resolved; required with typed keys, trace logs, isolated
  artifacts, exact visible checkpoints, and zero harness retries.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output and Tmux artifacts linked.
- [ ] Opus 5 Max review disposition and remote SHA recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/claude-subscription-auth/`.
