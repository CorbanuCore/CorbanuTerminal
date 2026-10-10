---
sprint_id: "PF-60-S08"
title: "/cost page clarity and reopen warm-up cost"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 3
owner: "Codex accounting lane (proposed); Travis Good accepts"
parallel_lane: "UNALLOCATED"
write_scope: "UNALLOCATED"
integration_gate: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-60-S04"
created: 2026-10-10
updated: 2026-10-10
---

# PF-60-S08 — `/cost` page clarity and reopen warm-up cost

## Execution mandate

- Deliver: items (v), (vi) and (vii) from Travis's 2026-10-10 decision.
  - (v) #396 item 1: an unpriced request page shows "Cache write cost: $0.000000" for 0 tokens.
  - (vi) #403: every reopen of an OpenAI conversation (`/resume`, restart, `/side`) sends a paid warm-up request,
    about $0.025 on `gpt-5.4`. It is recorded, but users don't expect a charge for reopening. Make it visible or avoidable.
  - (vii) #396 item 2: with a declared basis the line reads "(pay per use (set in your config))".
- Excludes: price changes (PF-60-S07); #352's other display nits unless Travis adds them.

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/proposed/portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: reopening shows no surprise charge; unpriced pages show no money figure.

## Code boundaries

- Existing: `codex-rs/tui/src/chatwidget/tokens.rs` (request page, basis line); `codex-rs/core/src/session_startup_prewarm.rs`;
  `codex-rs/core/src/accounting.rs::prewarm_turn_label`.
- Tests: `codex-rs/tui/src/chatwidget/tokens_tests.rs`, `codex-rs/tui/src/chatwidget/tokens/scope_tests.rs`; literal list at readiness.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] Travis chooses the warm-up option for AC3. Recommendation: (B) show it, plus a config switch to turn reopen
      warm-up off; (A) alone trades the charge for a slower first reply.

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.

## Remaining

- [ ] AC1 (#396.1): on an unpriced request page no component shows a USD figure; a 0-token component reads
      "none — 0 tokens" (or "unknown — rate unavailable"); priced pages are unchanged.
- [ ] AC2 (#396.2): the declared-basis line has no nested parentheses (for example "records only its billing basis:
      pay per use, set in your config"); snapshot tests updated.
- [ ] AC3 (#403), Travis's option: (A) **avoid** — reopening sends no paid request until the user sends a message;
      or (B) **show** — `/cost` lists it as "Reopen warm-up" in plain words with its cost, and reopening says it was sent.
      Either way: earlier totals are unchanged and the warm-up is never hidden or counted twice.
- [ ] Code-blind functional design frozen before test-result disclosure.

## Verification

- [ ] Focused: `just test -p codex-tui --features developer-accounting -E 'test(tokens) | test(cost)'`, without the feature too; core tests for AC3.
- [ ] Linux clippy `-D warnings` with and without `developer-accounting`.
- [ ] TUI: tmux run with a real OpenAI key covering an unpriced (Fast tier) page, a declared-basis route, and `/resume`
      and restart; demo video per change.
- [ ] Code-blind cases executed independently (in PF-60-S10 under the enforced gate).

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] Code-blind handoff checker passes, or explicit limited-testing agreement/blocker recorded; no automatic human acceptance.
- [ ] Parallel handoff, if applicable: commit, contract versions, scope audit and combined-tree test evidence recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality; #396 and #403 closed.
- [ ] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/`.
