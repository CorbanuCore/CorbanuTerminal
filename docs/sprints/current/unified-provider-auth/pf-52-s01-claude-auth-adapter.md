---
sprint_id: "PF-52-S01"
title: "Claude auth adapter"
status: in_progress
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-52"
execution_order: 11
owner: "GPT-5.6 Sol high implementation agent"
parallel_lane: "provider-auth-serial"
write_scope: "MODULE.bazel.lock; codex-rs/Cargo.lock; codex-rs/provider-auth/src/lib.rs; codex-rs/provider-auth/src/auth_flow.rs; codex-rs/provider-auth/src/auth_flow_tests.rs; codex-rs/provider-auth/src/claude_account_flow.rs; codex-rs/provider-auth/src/claude_account_controller.rs; codex-rs/provider-auth/src/claude_account_settlement.rs; codex-rs/provider-auth/src/claude_account_flow_tests.rs; codex-rs/tui/Cargo.toml; codex-rs/tui/BUILD.bazel; codex-rs/tui/src/chatwidget.rs; codex-rs/tui/src/chatwidget/claude_auth_adapter.rs; codex-rs/tui/src/chatwidget/claude_auth_adapter_tests.rs; codex-rs/tui/src/chatwidget/claude_code_login.rs; codex-rs/tui/src/chatwidget/claude_code_login_tests.rs; codex-rs/tui/src/onboarding/auth.rs; docs/sprints/current/unified-provider-auth/pf-52-s01-claude-auth-adapter.md"
integration_gate: "PF-52 typed Claude controller, thin TUI adapter, surgical qualified-backend reauthorization/callback seams, and onboarding's existing authorization-code send wrapped in the zeroizing backend input only; no onboarding behavior or /providers host migration, vault/login/CLI discovery rewrite, app-server protocol change, runtime bearer change, credential removal, or PF-53+ work"
worktree: "/home/pfrpc/repos/worktrees/corbanu-main-f7356a94e0"
branch: "feat/unified-provider-auth"
base_commit: "f7356a94e032234022a462d65b576a7de2854859"
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

- [x] Plan is active.
- [x] PF-51-S01 is completed and archived at `bd3b011a23`.
- [x] Exact serial allocation matches the plan.
- [x] PF-42–PF-47 invariants and evidence are reviewed before changing adapters.

## Done

- [x] Added the renderer-independent Claude account controller for method choice,
      managed-token entry, Claude Code login, retry, replace, cancellation, and PF-49 reconciliation.
- [x] Bound UnauthorizedRecovery to the qualified backend's selected Managed,
      Environment, or Claude Code source without fallback; PreserveSelected binds source and authority.
- [x] Made code submission the commit point; correlated timeout, stale, late,
      transport-loss, status, and adapter Ready/Finished ordering races are typed and tested.
- [x] Kept managed tokens, challenges, codes, backend text, and process output out
      of snapshots/history; the legacy onboarding send only adds the zeroizing input wrapper.
- [x] Added Cargo/Bazel registration and preserved PF-42–PF-47 discovery, priority,
      custody, atomic-selection, runtime-bearer, and metadata-only invariants.

## Remaining

- [ ] Primary reviews/integrates the final tree, records the implementation commit,
      marks the sprint completed, and archives it; no implementation item remains.

## Verification

- [x] `just bazel-lock-update`, scoped `just fix`, escalated `just fmt`, and
      `git diff --check` passed. Sandboxed fmt alone was environment-blocked by read-only uv cache.
- [x] Zero-retry suites passed: provider-auth 49; TUI adapter 5; full Claude backend
      30; vault Claude 21; login bearer cache 2; login UnauthorizedRecovery 1; CLI health 5.
- [x] `CARGO_INCREMENTAL=0 cargo check -p codex-tui` passed. The first CLI filter
      selected zero tests; corrected `platform_fixture_health` passed all five binaries.
- [x] Governance passed: plans 2/2 active, sprints 63 current/107 archived,
      portable skills 25; production canary scan returned no matches.
- [x] Final production lines stay below 500: auth_flow 494, lib 491, Claude flow
      423, controller 484, settlement 421, TUI adapter 285.
- [ ] Primary confirms the recorded final-tree evidence after integration.

## Exit evidence

- [x] Final changed implementation paths are Cargo.lock; provider-auth lib/auth_flow
      plus Claude flow/controller/settlement/tests; TUI Cargo/BUILD, chatwidget
      adapter/backend/tests, and onboarding's wrapper-only send.
- [x] PF-42–PF-47 compatibility is covered by the vault/login/CLI suites above;
      no historical evidence was invalidated.
- [x] `Done` and `Remaining` reflect the primary-review-ready tree.
- [ ] Implementation commit and archived completed record remain primary-owned.
