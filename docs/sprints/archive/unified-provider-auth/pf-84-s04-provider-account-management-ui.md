---
sprint_id: "PF-84-S04"
title: "Account management in /providers and onboarding"
status: completed
plan_file: "docs/plans/completed/main-2026-10-14-unified-provider-auth.md"
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
merged_behind_flag: "named_accounts"
gate_evidence: "qa/provider-auth/pf-84/s04-gate.md"
---

# PF-84-S04 — Account management in /providers and onboarding

## Closure — 2026-10-11

Completed. Travis **accepted** PF-84-S04 on 2026-10-11 behind the default-off `named_accounts` flag
([comment on #465](https://github.com/CorbanuCore/CorbanuTerminal/pull/465)); the flag stays off, except for the
duplicate-row fix, which is not behind it. The independent run was code-blind on a debug build, not the full isolated
code-blind VM run; Travis accepted on that evidence, which is recorded here as the limited-testing agreement.
Archived by the PF-84-S06 worker.

Evidence:

- [Gate evidence](../../../../qa/provider-auth/pf-84/s04-gate.md) (tests, tmux/GLM 5.3 Flash runs, review, videos).
- [S04 independent acceptance](../../../../qa/provider-auth/pf-84/independent-acceptance-s04-20261011/README.md)
  (PR #465, [review](../../../../qa/provider-auth/pf-84/independent-acceptance-s04-20261011/REVIEW.md)): 14 pass;
  S04-8 and the onboarding half of S04-12 failed on #461; S04-2b, S04-16b and S04-17 not verifiable.
- [#461/#462 fixes](../../../../qa/provider-auth/pf-84/fix-461-462-20261011/README.md) (PR #472): `q` no longer quits
  onboarding text fields; #462 (pre-existing) fixed too. Not re-run by an independent tester.
- PRs: #430 (implementation), #465 (acceptance), #472 (#461, #462 fixes).

Known limits (gate): one session selection covers one provider at a time; *Use for this session* starts a new session;
`corbanu account add` replaces an existing account; multi-agent v2 spawns do not show the account; a Claude Code login
directory is accepted without checking it holds a login (S04-2b). Named ChatGPT sign-ins and AWS profiles move to
PF-84-S06 (D4).

## Execution mandate

- Deliver: `/providers` shows exactly one row per (provider, account). On that
  row you can add an account (masked key or token, ChatGPT or Claude login,
  Claude Code config dir), rename it, remove its credential, choose "use for this
  session", and make it the default. It reuses the shared PF-50 controller.
- Fix the duplicate-row bug found by multiacct1: after Claude Plan setup, both
  "Anthropic Claude Account" and "Claude Account" showed "Configured · active · ready".
- Excludes: storage/resolution (S02), propagation (S03), usage (S05).

## Plan linkage

- Plan: [Unified provider onboarding and management](../../../plans/completed/main-2026-10-14-unified-provider-auth.md)
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

- [x] Independent acceptance: a code-blind run on a debug build (PR #465) instead of the isolated VM run; Travis accepted it 2026-10-11 as limited testing.

## Verification

- [x] Focused: `just test -p codex-tui -p codex-provider-auth` (snapshots plus unit tests).
- [x] tmux SOP: in a disposable home, add a second Claude Plan account and a second
  ZAI key, switch the session account, restart, and check durability. Cancel is
  inert. Send text and Enter separately.
- [x] GLM run: `-m glm-5.3-flash -c model_provider="zai"` with the default ZAI account
  replying and the session switched to a fake second key giving a 401.
- [x] Videos: `pf84-add-account.toml`, `pf84-switch-account.toml`, `pf84-no-duplicate-rows.toml`.
- [x] One independent review (Opus 5.5 High), plus one scoped follow-up check.
- [x] Independent code-blind functional execution linked (PR #465); Travis signed off 2026-10-11.

## Security notes

- Entry is masked. Values never appear in the viewport, scrollback, snapshots or tmux artifacts (canary scan).
- Remove deletes only that account's labels, and the vault index proves it.

## Exit evidence

- [x] Implementation commit and PR recorded (branch `feat/pf-84-s04-provider-accounts-ui`, code at `7a1ab9bc9d`).
- [x] Snapshots, tmux logs, canary scan and video paths linked (gate file).
- [x] Limited-testing agreement recorded (Closure, 2026-10-11).
- [x] `Done` and `Remaining` ledgers reflect reality.
- [x] Completed record moved to `docs/sprints/archive/unified-provider-auth/`.
