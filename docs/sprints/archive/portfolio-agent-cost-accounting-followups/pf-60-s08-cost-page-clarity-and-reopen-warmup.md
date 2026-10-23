---
sprint_id: "PF-60-S08"
title: "/cost page clarity and reopen warm-up cost"
status: completed
plan_file: "docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 3
owner: "Codex worker pf60-next; Travis Good accepts"
parallel_lane: "accounting"
write_scope: "codex-rs/core/src/session_startup_prewarm.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/accounting.rs, codex-rs/core/src/config/mod.rs, codex-rs/config/src/config_toml.rs, codex-rs/core/config.schema.json, codex-rs/core/src/client.rs, codex-rs/core/src/session_startup_prewarm_tests.rs, codex-rs/core/src/accounting_tests.rs, codex-rs/thread-manager-sample/src/main.rs, codex-rs/core/tests/suite/accounting_responses_ws_recovery.rs, codex-rs/tui/src/chatwidget/tokens.rs, codex-rs/tui/src/chatwidget/tokens_tests.rs, docs/config.md, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s08-cost-page-clarity-and-reopen-warmup.md, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s07-price-catalogue-refresh.md, docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md, qa/portfolio/agent-cost-accounting/pf-60-s08/, qa/demos/index/PF-60-S08.md, qa/demos/specs/pf60s08-reopen-warmup-cost.toml"
integration_gate: "One PR to main: fail-before/pass-after tests, focused just test with and without developer-accounting, macOS and Linux clippy -D warnings, independent Opus 5.5 High review to APPROVE on the exact head, tmux check with a real OpenAI key in a disposable home, SOP video; Travis accepts"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s08"
branch: "pf60-s08/cost-clarity-reopen-warmup"
base_commit: "45916ab64ef7faf4a5525a8dbdbfd4f706dd9f79"
depends_on: "PF-60-S04"
created: 2026-10-10
updated: 2026-10-16
---

# PF-60-S08 — `/cost` page clarity and reopen warm-up cost

**Status: completed** (accepted by Travis 2026-10-15; code-blind run in PF-60-S10 accepted 2026-10-16; see Closure).

## Closure — 2026-10-16

Completed. Travis accepted S08 on 2026-10-15 (20:55Z, "1-6 as recommended"; comment on #400). PF-60-S10's isolated code-blind run ([record](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/README.md), PR #909, merged `223927bc26`) was accepted by Travis on 2026-10-16 as recorded, 29 passed, 1 failed, 7 blocked of 37 (decision D24, "all as recommended"; comment on [#898](https://github.com/CorbanuCore/CorbanuTerminal/issues/898)). With it he decided D25 (the S10 gaps are re-checked in #898's acceptance run, not a full S10 re-run) and D26 (the record's last evidence check, r3, failed on wording only; accepted without another round).

S10 results for this sprint ([dispositions](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/dispositions.md)):

- Passed: ACC-10 and ACC-13 (a new conversation shows no reopen label or notice but still warms up), ACC-12 and
  ACC-36 (`skip_reopen_warmup` by flag and by `config.toml`), ACC-16 (editing an earlier message sends a labelled,
  announced reopen warm-up), ACC-17 (reviewers and sub-agents are not reopens), ACC-27 (unpriced and basis-only
  wording, #396), ACC-29 (no warm-up on a non-WebSocket reopen).
- Blocked: ACC-11, ACC-14 and ACC-18 (restart and `/resume`): every observed result passed, but no earlier Request ID
  was re-read after the reopen, and ACC-18 checked per-conversation sums for one conversation only. ACC-15 (`/side`):
  everything passed except the `side:` label on the side turn, which no captured screen shows. These are re-checked in
  #898's acceptance run (D25).
- Observations filed: #912 (a conversation holding only a warm-up request is billed but can't be opened from
  `/resume`) and #913 ("Codex Auto Review" has no price on the OpenAI route).

#396 and #403 close with this archive.

## Execution mandate

- Deliver: items (v), (vi) and (vii) from Travis's 2026-10-10 decision.
  - (v) #396 item 1: an unpriced request page shows "Cache write cost: $0.000000" for 0 tokens.
  - (vi) #403: every reopen of an OpenAI conversation (`/resume`, restart, `/side`) sends a paid warm-up request,
    about $0.025 on `gpt-5.4`. It is recorded, but users don't expect a charge for reopening. Make it visible or avoidable.
  - (vii) #396 item 2: with a declared basis the line reads "(pay per use (set in your config))".
- Excludes: price changes (PF-60-S07); #352's other display nits unless Travis adds them.

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: reopening shows no surprise charge; unpriced pages show no money figure.

## Code boundaries

- Existing: `codex-rs/tui/src/chatwidget/tokens.rs` (request page, basis line); `codex-rs/core/src/session_startup_prewarm.rs`;
  `codex-rs/core/src/accounting.rs::prewarm_turn_label`.
- Tests: `codex-rs/tui/src/chatwidget/tokens_tests.rs`, `codex-rs/tui/src/chatwidget/tokens/scope_tests.rs`; literal list at readiness.

## Preconditions

- [x] Plan is active (Travis, 2026-10-14; slot 2).
- [x] Dependencies are completed (PF-60-S04 archived 2026-10-10).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Travis chooses the warm-up option for AC3 (2026-10-14): (C), show the reopen warm-up in `/cost` and on
      reopen, plus a setting to skip it, off by default so behaviour stays as today.

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.
- [x] AC1 (#396.1): a component with no rate states no money; a 0-token one reads "none — 0 tokens". Components with
      a rate keep their figure, zero included.
- [x] AC2 (#396.2): the basis-only price line reads "records only its billing basis: pay per use; set in your
      config"; the `Billing basis:` line keeps its parentheses.
- [x] AC3 (#403, option C): a conversation the user resumes or forks labels its warm-up `prewarm:reopen:<id>`
      (still a `prewarm:` label; sub-agents and reviewers are not reopens); once it has gone out the session shows a
      notice naming `skip_reopen_warmup`, not written to the conversation, so a later reopen doesn't replay it. `/cost` says "Reopen warm-up: N request(s), <cost>" on the
      first screen and "Reopen warm-up" on the request link and page. `skip_reopen_warmup = true` (default false)
      sends no warm-up on reopen; a new conversation still warms up. Earlier totals are unchanged; nothing is hidden
      or counted twice. [Gate](../../../../qa/portfolio/agent-cost-accounting/pf-60-s08/gate.md).

- [x] Merged: PR #821 as `4d0d1597a9` (head `a4ef32068d`); review APPROVE on that head.

## Remaining

- [x] Code-blind functional design frozen before test-result disclosure: PF-60-S10 froze it at `220c99ed59`
      ([S10](pf-60-s10-isolated-code-blind-acceptance.md)).
- [x] Travis accepted S08 (2026-10-15); #396 and #403 closed with this archive.

## Verification

- [x] Focused: core `test(accounting) | test(spawn) | test(prewarm) | test(websocket) | test(config) | test(reopen)`
      1115/1115 with `developer-accounting`, 985/985 without; codex-tui `test(tokens) | test(charge) | test(cost) |
      test(resume)` 308/309 (the local-only `cost_scope` tmux test).
- [x] macOS and Linux clippy `-D warnings --tests`, with and without `developer-accounting`.
- [x] TUI: tmux with a real OpenAI key (restart reopen, `/resume`, `/side`, skip setting, Fast tier, declared
      basis) and the demo video; see the gate.
- [x] Code-blind cases executed independently in PF-60-S10 under the enforced gate (Closure).

## Exit evidence

- [x] Implementation commit recorded: PR #821 (`4d0d1597a9`, head `a4ef32068d`).
- [x] Final-tree test output linked (gate).
- [x] Code-blind handoff checker: it stops at S10's first unresolved case, as expected; Travis's acceptance of S10 as
      recorded (2026-10-16, D24) is the recorded agreement. No automatic human acceptance.
- [x] Parallel handoff: not applicable (the plan ran one sprint at a time).
- [x] `Done` and `Remaining` ledgers reflect reality; #396 and #403 closed with this archive.
- [x] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/` (2026-10-16).
