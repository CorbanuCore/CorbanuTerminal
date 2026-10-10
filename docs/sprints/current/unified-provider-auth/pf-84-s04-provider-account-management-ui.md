---
sprint_id: "PF-84-S04"
title: "Account management in /providers and onboarding"
status: draft
plan_file: "docs/plans/proposed/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 26
owner: "UNALLOCATED"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-84-S03"
created: 2026-10-08
updated: 2026-10-08
---

# PF-84-S04 — Account management in /providers and onboarding

## Execution mandate

- Deliver: `/providers` shows exactly one row per (provider, account). On that
  row you can add an account (masked key or token, ChatGPT or Claude login,
  Claude Code config dir), rename it, remove its credential, choose "use for this
  session", and make it the default. It reuses the shared PF-50 controller.
- Fix the duplicate-row bug found by multiacct1: after Claude Plan setup, both
  "Anthropic Claude Account" and "Claude Account" showed "Configured · active · ready".
- Excludes: storage/resolution (S02), propagation (S03), usage (S05).

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/proposed/unified-provider-auth.md)
- Feature: `PF-84`
- Acceptance advanced: "Add second account", "Session account switch", "One row per account".

## Code boundaries

Paths are under `codex-rs/`, at `63ea3d0cbd`.

- Duplicate rows: `tui/src/onboarding/auth.rs:370-400` maps the catalog
  `ClaudeAccount` capability to the fixed `SignInOption::AnthropicAccount`
  (rendered as "Provider: Anthropic Claude Account" at `:989-1000`).
  `provider-auth/src/lib.rs:571-575` names the same catalog entry "Claude Account".
  Find the second source of the row (catalog entry vs. runtime or status-only
  option) and dedupe by (provider id, account).
- Hosts: `tui/src/onboarding/auth.rs:141-150, 559-569`;
  `tui/src/chatwidget/provider_credentials.rs`;
  `tui/src/chatwidget/claude_code_login.rs:125-151` (enrolment takes an account).
- Existing tmux tests to update: `tui/tests/suite/claude_auth.rs:60-62, 310, 348`;
  `tui/tests/suite/provider_management.rs:380-431`.
- Tests: snapshot tests for the row list; tmux scenarios.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.

## Remaining

- [ ] Regression first: a snapshot showing two rows for one configured Claude Plan; then fix.
- [ ] Account rows: name, kind, 12-hex fingerprint or login email hint, active/current markers.
- [ ] Add, rename, remove, session-select and set-default actions. Removing the
  current account needs a replacement first, as in PF-54.
- [ ] Onboarding offers "Add another account" after a provider succeeds.
- [ ] Code-blind functional design frozen before test-result disclosure, or non-user-facing N/A reason recorded.

## Verification

- [ ] Focused: `just test -p codex-tui -p codex-provider-auth` (snapshots plus unit tests).
- [ ] tmux SOP: in a disposable home, add a second Claude Plan account and a second
  ZAI key, switch the session account, restart, and check durability. Cancel is
  inert. Send text and Enter separately.
- [ ] GLM run: `-m glm-5.3-flash -c model_provider="zai"` with the default ZAI account
  replying and the session switched to a fake second key giving a 401.
- [ ] Videos: `pf84-add-account.toml`, `pf84-switch-account.toml`, `pf84-no-duplicate-rows.toml`.
- [ ] One independent review (Opus 5.5 High).

## Security notes

- Entry is masked. Values never appear in the viewport, scrollback, snapshots or tmux artifacts (canary scan).
- Remove deletes only that account's labels, and the vault index proves it.

## Exit evidence

- [ ] Implementation commit and PR recorded.
- [ ] Snapshots, tmux logs, canary scan and video paths linked.
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
