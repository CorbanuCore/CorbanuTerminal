---
sprint_id: "PF-60-S06"
title: "Side-conversation and temporary-fork attribution"
status: completed
plan_file: "docs/plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 1
owner: "Codex worker pf60-s06; Travis Good accepts"
parallel_lane: "accounting"
write_scope: "codex-rs/core/src/session/mod.rs, codex-rs/core/src/accounting.rs, codex-rs/core/src/agent/control.rs, codex-rs/core/src/session/tests/guardian_tests.rs, codex-rs/core/tests/suite/, codex-rs/protocol/src/protocol.rs, codex-rs/tui/src/app_server_session.rs, docs/sprints/current/portfolio-agent-cost-accounting-followups/pf-60-s06-side-and-fork-attribution.md, qa/portfolio/agent-cost-accounting/pf-60-s06/"
integration_gate: "One PR to main: focused just test with and without developer-accounting, macOS and Linux clippy -D warnings, independent Opus 5.5 High review to APPROVE, tmux check on GLM 5.3 Flash in a disposable home, SOP video; Travis accepts"
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf60-s06"
branch: "pf60-s06-side-attribution"
base_commit: "ecb9069c4b3e0bdb20cb88d520fc8b54745cea34"
depends_on: "PF-60-S04"
created: 2026-10-10
updated: 2026-10-16
---

# PF-60-S06 — Side-conversation and temporary-fork attribution

**Status: completed** (accepted by Travis as limited testing 2026-10-14; code-blind run in PF-60-S10 accepted 2026-10-16;
see Closure).

## Closure — 2026-10-16

Completed. Travis accepted S06 as limited testing on 2026-10-14 (comment on #400). PF-60-S10's isolated code-blind run ([record](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/README.md), PR #909, merged `223927bc26`) was accepted by Travis on 2026-10-16 as recorded, 29 passed, 1 failed, 7 blocked of 37 (decision D24, "all as recommended"; comment on [#898](https://github.com/CorbanuCore/CorbanuTerminal/issues/898)). With it he decided D25 (the S10 gaps are re-checked in #898's acceptance run, not a full S10 re-run) and D26 (the record's last evidence check, r3, failed on wording only; accepted without another round).

S10 results for this sprint ([dispositions](../../../../qa/portfolio/agent-cost-accounting/followups-acceptance-20261016/dispositions.md)):

- Passed: ACC-01 (`/cost` navigation, empty day), ACC-02 and ACC-03 (`/side` work and the automatic reviewer inside
  `/side` recorded once under the saved conversation as `side:` and `side:review:`), ACC-05 (a sub-agent counted once
  in the parent's total), ACC-07 (`exec --ephemeral` stays out).
- Failed: ACC-06, a `/review` code review's requests were recorded under the review's own thread and shown nowhere in
  `/cost` (#898). Fixed by PR #917 (merged `d4831b7326`): they are now listed under the conversation as `code-review:`
  and counted in the total. The live re-check is in #898's acceptance run (D25).
- Blocked (advisory): ACC-04, the product's model declined to start a sub-agent inside `/side`, which the design
  allows; AC2 rests on the test, as before.

#400 and #401 close with this archive.

## Execution mandate

- Deliver: items (i) and (ii) from Travis's 2026-10-10 decision ([PF-60-S04 Closure](../../archive/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md)).
  - (i) #400: a guardian review of a `/side` conversation, and a sub-agent started from one, are recorded. Today
    they are excluded because the side conversation itself isn't persisted.
  - (ii) #401: the `side:` label names only TUI `/side`. Every other kind of temporary fork of a saved conversation
    gets a label naming what it is.
- Excludes: price changes, `/cost` layout changes beyond the new labels, recording `exec --ephemeral` runs.

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/completed/main-2026-10-20-portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: every request appears once under the parent conversation, labelled with what it was.

## Code boundaries

- Existing: `codex-rs/core/src/session/mod.rs::accounting_owner` and `accounting_owner_elsewhere`;
  `codex-rs/core/src/accounting.rs` (`AccountingOwner`, turn labels); `codex-rs/core/src/guardian/review_session.rs`;
  sub-agent spawning under `codex-rs/core/src/agent/`.
- Planned: owner resolution that follows a temporary fork up to its persisted conversation; a label per fork kind.
- Tests: `codex-rs/core/src/accounting_extensions_tests.rs`, `codex-rs/core/tests/suite/accounting_responses.rs`;
  literal list fixed at readiness.

## Preconditions

- [x] Plan is active (Travis, 2026-10-14; slot 2).
- [x] Dependencies are completed (PF-60-S04 archived 2026-10-10).
- [x] Worktree, branch, and base commit are exact and match the plan.
- [x] Not parallel: the plan runs one sprint at a time.

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.
- [x] Allocated 2026-10-14 (worker `pf60-s06`) when Travis started the follow-ups.
- [x] Audit: [coverage audit](../../../../qa/portfolio/agent-cost-accounting/pf-60-s06/coverage-audit.md), listing every
      session with no thread of its own, its owner and its label.
- [x] AC1: a guardian review inside `/side` is recorded once under the saved conversation, as `side:review:`. Shown by
      a test, in a live tmux run and in the video.
- [x] AC2: a sub-agent from `/side` is recorded under the saved conversation, as `side:agent:`. The test checks exact
      rows and the day total. It wasn't shown live, because `/side` tells the model not to spawn agents (see the gate).
- [x] AC3: `side:` only for TUI `/side` (thread source `side`); every other temporary fork is `fork:`; sub-agents and
      `/review` tasks without a thread are `agent:` and `code-review:`.
- [x] AC4: the ledger format is unchanged; the labels are ordinary turn labels, so older rows read as before.
- [x] AC5: `exec --ephemeral`, and work done for it, stay excluded, with one log line per session (existing
      `an_uncollected_session_is_named_once` plus a new sub-agent guard).
- [x] Gate: [gate.md](../../../../qa/portfolio/agent-cost-accounting/pf-60-s06/gate.md); PR #793 at code head
      `604fe9c89d`; plan activation in #790.
- [x] Travis accepted S06 as limited testing (2026-10-14). #400 and #401 stayed open until S10's code-blind run
      (accepted 2026-10-16; Closure).

## Remaining

- [x] Code-blind functional design frozen before test-result disclosure: PF-60-S10 froze it at `220c99ed59`
      ([S10](pf-60-s10-isolated-code-blind-acceptance.md)).

## Verification

- [x] Focused `just test -p codex-core -E 'test(accounting) | test(guardian) | test(side) | test(spawn)'`: 523/523 with
      `developer-accounting`, 518/518 without; `codex-tui` `test(side)` 94/94.
- [x] macOS and Linux clippy `-D warnings --tests`, with and without `developer-accounting`.
- [x] TUI: tmux run with real keys (`/side` plus a guardian review), the ledger read back, and the
      [demo video](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-60-s06-pf60s06-side-guardian-review-cost-604fe9c89da3-2026-10-14.mp4).
- [x] Code-blind cases executed independently in PF-60-S10 under the enforced gate (Closure).

## Exit evidence

- [x] Implementation commits recorded: `1bb8e8d834`, `604fe9c89d` (PR #793).
- [x] Final-tree test output linked (gate).
- [x] Limited-testing agreement recorded: Travis, 2026-10-14 (comment on #400), accepted S06 as limited testing; the
      code-blind cases run in PF-60-S10, which may still reopen it. No automatic human acceptance.
- [x] `Done` and `Remaining` ledgers reflect reality; #400 and #401 closed with this archive.
- [x] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/` (2026-10-16).
