---
sprint_id: "PF-84-S04"
title: "Account management in /providers and onboarding"
status: ready
plan_file: "docs/plans/active/unified-provider-auth.md"
plan_feature: "PF-84"
execution_order: 26
owner: "Codex PF-84 lane owner"
parallel_lane: "account-ui"
write_scope: "codex-rs/tui/, codex-rs/protocol/, codex-rs/analytics/src/analytics_client_tests.rs, codex-rs/exec/tests/event_processor_with_json_output.rs, codex-rs/Cargo.lock, codex-rs/provider-auth/, codex-rs/app-server/, codex-rs/app-server-protocol/, codex-rs/core/, codex-rs/login/, codex-rs/vault/, codex-rs/cli/, codex-rs/model-provider/, codex-rs/model-provider-info/, qa/provider-auth/pf-84/s04-gate.md, qa/demos/specs/pf84-add-account.toml, qa/demos/specs/pf84-switch-account.toml, qa/demos/specs/pf84-no-duplicate-rows.toml, qa/demos/index/PF-84-S04.md, docs/sprints/current/unified-provider-auth/pf-84-s04-provider-account-management-ui.md, docs/sprints/current/unified-provider-auth/index.md"
integration_gate: "Codex PF-84 lane owner merges to main after just test -p codex-tui -p codex-provider-auth (plus touched crates), Linux clippy -D warnings, the tmux/GLM run and one Opus 5.5 High review"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf84-s04-providers"
branch: "feat/pf-84-s04-provider-accounts-ui"
base_commit: "0558c52cc320422d1fc187f4a98afff7a1530286"
depends_on: "PF-84-S03"
created: 2026-10-08
updated: 2026-10-11
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

- Plan: [Unified provider onboarding and management](../../../plans/active/unified-provider-auth.md)
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

- [x] Plan is active.
- [x] Dependencies are completed (PF-84-S03 merged behind `named_accounts`, `0558c52cc3`).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Sprint record created and linked to one plan feature.
- [x] Regression test `configured_claude_plan_has_exactly_one_row` (both rows on the base),
  then one onboarding row per configured provider with use / add account / replace behind it.
- [x] `/providers` account rows (kind, salted fingerprint, session/default markers) and add,
  rename, remove (replacement first when in use), use-for-session and make-default.
- [x] Onboarding "Add another account"; account in `/status` and the spawn line; S03 follow-up
  tests. Gate: [s04-gate.md](../../../../qa/provider-auth/pf-84/s04-gate.md).
- [x] Named ChatGPT sign-ins and AWS profiles stay refused here; Travis's D4 (2026-10-11) moves them to PF-84-S06.

## Remaining

- [ ] Code-blind functional design and execution (independent acceptance step, not the implementer).

## Verification

- [x] Focused: `just test -p codex-tui -p codex-provider-auth` (snapshots plus unit tests).
- [x] tmux SOP: in a disposable home, add a second Claude Plan account and a second
  ZAI key, switch the session account, restart, and check durability. Cancel is
  inert. Send text and Enter separately.
- [x] GLM run: `-m glm-5.3-flash -c model_provider="zai"` with the default ZAI account
  replying and the session switched to a fake second key giving a 401.
- [x] Videos: `pf84-add-account.toml`, `pf84-switch-account.toml`, `pf84-no-duplicate-rows.toml`.
- [x] One independent review (Opus 5.5 High), plus one scoped follow-up check.
- [ ] Independent code-blind functional execution (acceptance step, not the implementer).

## Security notes

- Entry is masked. Values never appear in the viewport, scrollback, snapshots or tmux artifacts (canary scan).
- Remove deletes only that account's labels, and the vault index proves it.

## Exit evidence

- [x] Implementation commit and PR recorded (branch `feat/pf-84-s04-provider-accounts-ui`, code at `7a1ab9bc9d`).
- [x] Snapshots, tmux logs, canary scan and video paths linked (gate file).
- [ ] Code-blind handoff checker passes, or limited-testing agreement recorded.
- [x] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
