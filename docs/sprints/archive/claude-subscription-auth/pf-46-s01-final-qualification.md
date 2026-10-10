---
sprint_id: "PF-46-S01"
title: "Claude subscription authentication final qualification"
status: completed
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-46"
execution_order: 5
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "MODULE.bazel.lock, codex-rs/Cargo.lock, codex-rs/Cargo.toml, codex-rs/cli/Cargo.toml, codex-rs/cli/src/claude_oauth.rs, codex-rs/cli/src/main.rs, codex-rs/login/Cargo.toml, codex-rs/login/src/auth/auth_tests.rs, codex-rs/login/src/auth/external_bearer.rs, codex-rs/login/src/auth/manager.rs, codex-rs/login/src/auth/mod.rs, codex-rs/login/src/lib.rs, codex-rs/model-provider/src/auth.rs, codex-rs/model-provider-info/src/lib.rs, codex-rs/model-provider-info/src/model_provider_info_tests.rs, codex-rs/vault/Cargo.toml, codex-rs/vault/src/capability.rs, codex-rs/vault/src/capability_tests.rs, codex-rs/vault/src/claude_auth.rs, codex-rs/vault/src/claude_auth_tests.rs, codex-rs/vault/src/lib.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/app_event.rs, codex-rs/tui/src/chatwidget/claude_code_login.rs, codex-rs/tui/src/chatwidget/claude_code_login_tests.rs, codex-rs/tui/src/chatwidget/provider_credentials.rs, codex-rs/tui/src/chatwidget/snapshots/, codex-rs/tui/src/chatwidget/vault_menu.rs, codex-rs/tui/src/claude_panes/command_plan.rs, codex-rs/tui/src/claude_panes/execution.rs, codex-rs/tui/src/claude_panes/tests.rs, codex-rs/tui/src/claude_panes/turn_types.rs, codex-rs/tui/tests/support/tmux.rs, codex-rs/tui/tests/suite/claude_auth.rs, codex-rs/tui/tests/suite/mod.rs, docs/authentication.md, docs/features/claude-headless-panes.md, docs/features/claude-plan-authentication.md, docs/features/model-providers.md, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/, docs/sprints/archive/claude-subscription-auth/, docs/sprints/index.md, mkdocs.yml, qa/claude-subscription-auth/"
integration_gate: "Jim Ricketts verifies the isolated final candidate from frozen base 8ae13e, closes structured and Corbanu/Opus 5 Max reviews, archives PF-46-S01, and pushes only feat/claude-subscription-auth-isolated."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
branch: "feat/claude-subscription-auth-isolated"
base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
depends_on: "PF-45-S01"
created: 2026-08-30
updated: 2026-08-31
---

# CSA-05 / PF-46-S01 — Final qualification

## Execution mandate

- Deliver: final-tree cross-platform, adversarial, true-TUI, documentation, and
  review evidence for the isolated Claude subscription-auth feature branch.
- Excludes: release/tag/merge/PR decisions and fabricated external human or
  live-account acceptance.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/unified-provider-auth.md).
- Feature: `PF-46` (plan alias `CSA-05`).
- Acceptance advanced: a review-clean branch can be evaluated without exposing
  a credential or silently changing source, account, or billing path.

## Code boundaries

- Existing: CSA-01 through CSA-04 implementation and archived sprint evidence.
- Planned: final fixes only within declared auth surfaces, finished feature
  documentation, compact QA/review evidence, and remote branch verification.
- Tests: affected Rust crates, typed tmux state-machine suite, secret scans,
  structured branch autoreview, and Corbanu Terminal Opus 5 Max review.
- Coordination-only: pre-sprint commit `f36c28770` mirrors P0 lifecycle state
  solely so global allocation accounting matches committed main. Its paths are
  intentionally excluded from PF-46's feature scope because adding the active
  P0 records would overlap PF-35's live literal scope. This isolated branch does
  not contain or re-qualify PF-13-S06 or PF-41-S03 product code/evidence; those
  archive claims become valid only when integrated onto the main lineage that
  already contains their implementation.

## Preconditions

- [x] Plan is active.
- [x] PF-42-S01 through PF-45-S01 are completed and archived.
- [x] Worktree, isolated branch, and frozen base commit match the plan.
- [x] Owner, serial lane, literal scope, and integration gate are recorded.

## Done

- [x] Sprint record created and linked to PF-46.
- [x] Finished the user/operator authentication guide and reconciled native-pane
  credential-boundary language with the behavior actually under Corbanu control.
- [x] Closed the final bounded-output, missing-environment-source, transient
  health classification, compatibility-token custody, generic vault custody,
  source-change bearer-cache, stale navigation, and literal-scope findings.
- [x] Ran retry-disabled affected tests and the required typed Tmux state-machine
  suite against the isolated final implementation candidate.
- [x] Dispositioned structured and Corbanu/Opus 5 Max findings without weakening
  the runtime executable-binding or custom-provider compatibility contracts.
- [x] Recorded automated evidence and honest external applicability gaps.

## Remaining

None for PF-46's automated delivery scope. Named human/live Anthropic account,
TensorCash, Isometric Game, physical Linux/Windows release-host, target-release,
tag, merge, and release-ledger decisions remain open on the active plan.

## Verification

- [x] `just fix -p codex-cli`, `just fix -p codex-tui`, `just fmt`, and
  `git diff --check` passed around the final retry-disabled affected tests.
- [x] Required typed tmux run used the candidate binary, trace logs, isolated
  artifacts, real key flows, and a clean canary scan.
- [x] Plan and sprint checkers pass on the final tree.
- [x] TUI applicability: required; snapshots plus private-tmux key and visible
  checkpoint evidence are recorded.

## Exit evidence

- [x] Implementation commits through `b96326f01` and the frozen base `8ae13e`
  are recorded in `qa/claude-subscription-auth/sprints/PF-46-S01/evidence.md`.
- [x] Final-tree test, documentation, review protocol, and remote push gate are linked.
- [x] Done and Remaining ledgers reflect reality without claiming external signoff.
- [x] Completed record moved to `docs/sprints/archive/claude-subscription-auth/`.
