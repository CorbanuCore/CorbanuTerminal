---
sprint_id: "PF-60-S09"
title: "Verification paths for unverified flows"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 4
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

# PF-60-S09 — Verification paths for unverified flows

## Execution mandate

- Deliver: item (iv) from Travis's 2026-10-10 decision (#402). Four flows were NOT VERIFIABLE in PF-60-S05 and
  PF-60-S04: ChatGPT login, image generation, realtime and TensorCash. Build a real way to verify each, run it once,
  and record PASS or FAIL.
- Excludes: changing how these flows are priced or recorded (a FAIL becomes an issue, not a fix here).

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/proposed/portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: no accounting flow is left unverified without Travis's recorded decision.

## Code boundaries

- Existing: `codex-rs/core/src/realtime_conversation.rs`; `codex-rs/ext/image-generation/src/backend.rs`;
  `codex-rs/core/src/tools/spec_plan.rs` (image tool offered to OpenAI API keys since #370); the tmux harness
  (`docs/tmuxHarness.md`) and the PF-60-S04 acceptance tools under `qa/portfolio/agent-cost-accounting/pf-60-s04/`.
- Planned: test drivers and fixtures under `qa/portfolio/agent-cost-accounting/`; vault labels for any new test account.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] Travis provides or approves: a dedicated ChatGPT test account in the vault (his own login is never swapped), and
      TensorCash repository access for the acceptance account (or rules it not applicable, with a reason).

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.

## Remaining

- [ ] AC1 ChatGPT login: with the test account in a disposable home, ChatGPT-login turns (including an uncatalogued
      model and the priority tier) show "Subscription", never spend, "no price" or "check the bill".
- [ ] AC2 Image generation: a real image request through the TUI on an OpenAI API key is recorded at OpenAI's published
      image rates and recomputes exactly; the ChatGPT-login image path is recorded as subscription (with AC1's account).
- [ ] AC3 Realtime: a scripted driver (for example the app-server realtime API with recorded audio) runs a real realtime
      session; its requests are recorded with their basis and, on an API key, recompute exactly.
- [ ] AC4 TensorCash: with access resolved, one accounting journey runs against the live repository and its `/cost`
      totals reconcile; otherwise Travis's not-applicable decision is recorded.
- [ ] Each driver is documented so the PF-60-S10 executor can rerun it without reading source.
- [ ] Code-blind functional design frozen before test-result disclosure.

## Verification

- [ ] Each AC has captures, raw provider usage and a recomputation (or Travis's recorded decision); no NOT VERIFIABLE left.
- [ ] Key scan of all captures: 0 hits; disposable homes with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- [ ] Integration: `python3 docs/plans/check.py; python3 docs/sprints/check.py`; `git diff --check`.
- [ ] Code-blind cases executed independently (in PF-60-S10 under the enforced gate).

## Exit evidence

- [ ] Driver commit and evidence directory recorded.
- [ ] Per-flow PASS/FAIL linked; any FAIL filed as an issue.
- [ ] Code-blind handoff checker passes, or explicit limited-testing agreement/blocker recorded; no automatic human acceptance.
- [ ] Parallel handoff, if applicable: commit, contract versions, scope audit and combined-tree test evidence recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality; #402 closed.
- [ ] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/`.
