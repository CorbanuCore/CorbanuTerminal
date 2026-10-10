---
sprint_id: "PF-47-S01"
title: "First-run Anthropic-account onboarding"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-47"
execution_order: 6
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/Cargo.lock, codex-rs/Cargo.toml, codex-rs/cli/, codex-rs/login/, codex-rs/model-provider-info/, codex-rs/model-provider/, codex-rs/tui/, codex-rs/vault/, docs/authentication.md, docs/features/claude-headless-panes.md, docs/features/claude-plan-authentication.md, docs/features/model-providers.md, docs/plans/active/claude-subscription-auth.md, docs/plans/active/index.md, docs/plans/index.md, docs/sprints/archive/claude-subscription-auth/, docs/sprints/current/claude-subscription-auth/, docs/sprints/index.md, mkdocs.yml, qa/claude-subscription-auth/"
integration_gate: "Jim Ricketts verified the recovered candidate based on remote main 9ec532ed, reran retry-disabled focused and typed-Tmux qualification, archived PF-47-S01, and authorized direct integration to main."
worktree: "/Users/Neo/.codex/worktrees/claude-subscription-auth-recovered"
branch: "feat/claude-subscription-auth-recovered"
base_commit: "9ec532ed144ff041cae32592414e9e21873df6fe"
depends_on: "PF-46-S01"
created: 2026-08-31
updated: 2026-08-31
---

# CSA-06 / PF-47-S01 — First-run Anthropic-account onboarding

## Execution mandate

- Delivered: first-run onboarding offers an Anthropic Claude account and
  routes it into the existing explicit stable-token or Claude Code login flow.
- Excluded: changing Anthropic API-key behavior, provider wire contracts,
  credential custody, or unrelated onboarding providers.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-47` (plan alias `CSA-06`).
- Acceptance advanced: a fresh user can enroll an eligible personal, Team, or
  Enterprise Anthropic account without first navigating to `/providers`.

## Code boundaries

- Reused `onboarding::auth::SignInOption`, `OnboardingResult`, and the qualified
  Claude auth-choice and persistence flow under `chatwidget`.
- Added a typed onboarding handoff that commits `claude-plan` only after a
  successful account-method selection and ignores stale completion events.
- Added unit, snapshot, cancellation-race, and typed-Tmux startup coverage.

## Preconditions

- [x] Plan is active.
- [x] PF-46-S01 is completed and archived.
- [x] Recovered worktree, branch, and remote-main base are recorded.
- [x] Owner, serial lane, literal scope, and integration gate are recorded.

## Done

- [x] Added a distinct Anthropic-account option to first-run onboarding.
- [x] Routed selection through the existing explicit Claude authentication flow.
- [x] Persisted provider selection only after success and kept cancel inert.
- [x] Removed pasted CR/LF characters from managed tokens before validation.
- [x] Rejected stale auth completion after cancellation and killed the child flow.
- [x] Added onboarding unit, snapshot, race, and typed-Tmux regression coverage.
- [x] Reconciled the recovered candidate with remote `main` and final review findings.

## Remaining

- None for PF-47-S01. Live-account, live-repository, physical release-host, and
  release/tag gates remain explicitly owned by the active plan.

## Verification

- [x] `just fix -p codex-tui`, `just fmt`, and `git diff --check` passed.
- [x] Retry-disabled focused race/default tests passed 3/3.
- [x] `CARGO_INCREMENTAL=0 just test -p codex-tui onboarding -j 1 --retries 0`
  passed 46/46 (Nextest `ca1a51b7-7893-4697-888b-3c6942c47bd1`).
- [x] `CARGO_INCREMENTAL=0 just test -p codex-tui claude_code_login -j 1
  --retries 0` passed 27/27 (Nextest `5b308f0e-81b1-4610-a916-d1970095946e`).
- [x] Required typed-Tmux tests passed 3/3 with `CORBANU_TMUX_REQUIRED=1`,
  `RUST_LOG=trace`, serial execution, and zero retries.

## Exit evidence

- [x] Final implementation and recovery boundary recorded in the PF-47 evidence ledger.
- [x] Final-tree test output, binary hash, and Tmux run IDs recorded.
- [x] Twelve formal review runs and all accepted Opus findings are dispositioned;
  no additional review was required.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/claude-subscription-auth/`.
