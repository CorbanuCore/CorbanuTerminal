---
sprint_id: "PF-60-S06"
title: "Side-conversation and temporary-fork attribution"
status: draft
plan_file: "docs/plans/proposed/portfolio-agent-cost-accounting-followups.md"
plan_feature: "PF-60"
execution_order: 1
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

# PF-60-S06 — Side-conversation and temporary-fork attribution

## Execution mandate

- Deliver: items (i) and (ii) from Travis's 2026-10-10 decision ([PF-60-S04 Closure](../../archive/portfolio-agent-cost-accounting/pf-60-s04-cost-accounting-acceptance-and-handoff.md)).
  - (i) #400: a guardian review of a `/side` conversation, and a sub-agent started from one, are recorded. Today
    they are excluded because the side conversation itself isn't persisted.
  - (ii) #401: the `side:` label names only TUI `/side`. Every other kind of temporary fork of a saved conversation
    gets a label naming what it is.
- Excludes: price changes, `/cost` layout changes beyond the new labels, recording `exec --ephemeral` runs.

## Plan linkage

- Plan: [Accounting follow-ups after PF-60](../../../plans/proposed/portfolio-agent-cost-accounting-followups.md); feature `PF-60`.
- Acceptance advanced: every request appears once under the parent conversation, labelled with what it was.

## Code boundaries

- Existing: `codex-rs/core/src/session/mod.rs::accounting_owner` and `accounting_owner_elsewhere`;
  `codex-rs/core/src/accounting.rs` (`AccountingOwner`, turn labels); `codex-rs/core/src/guardian/review_session.rs`;
  sub-agent spawning under `codex-rs/core/src/agent/`.
- Planned: owner resolution that follows a temporary fork up to its persisted conversation; a label per fork kind.
- Tests: `codex-rs/core/src/accounting_extensions_tests.rs`, `codex-rs/core/tests/suite/accounting_responses.rs`;
  literal list fixed at readiness.

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.
- [ ] If parallel, owner/lane/scopes are disjoint and the receiving integration gate is recorded.

## Done

- [x] Draft sprint created 2026-10-10 from Travis's decision; linked to one plan feature.

## Remaining

- [ ] Audit every code path that creates a session without its own saved thread; list each fork kind and its owner.
- [ ] AC1: a guardian review of a tool call inside `/side` is recorded under the parent conversation, labelled so it
      reads as a review of side work (for example `side:review:`); its tokens are counted once.
- [ ] AC2: a sub-agent spawned from `/side` is recorded under the parent conversation as a descendant, with its own
      label; each of its requests is counted once and `/cost` totals reconcile.
- [ ] AC3: each fork kind found by the audit has its own label; `side:` appears only on TUI `/side` work; an unknown
      kind gets a neutral label (for example `fork:`), never `side:`.
- [ ] AC4: existing ledger rows are unchanged and still read; any format change is versioned.
- [ ] AC5: `exec --ephemeral` stays excluded with exactly one `accounting.excluded` warning; the coverage audit is updated.
- [ ] Code-blind functional design frozen before test-result disclosure.

## Verification

- [ ] Focused: `just test -p codex-core --features developer-accounting -E 'test(accounting) | test(guardian)'`, and without the feature.
- [ ] Linux clippy `-D warnings` with and without `developer-accounting`.
- [ ] TUI: tmux run with real keys (`/side`, a guardian review and a sub-agent from it); captures and demo video.
- [ ] Code-blind cases executed independently (in PF-60-S10 under the enforced gate).

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] Code-blind handoff checker passes, or explicit limited-testing agreement/blocker recorded; no automatic human acceptance.
- [ ] Parallel handoff, if applicable: commit, contract versions, scope audit and combined-tree test evidence recorded.
- [ ] `Done` and `Remaining` ledgers reflect reality; #400 and #401 closed.
- [ ] Completed record moved to `docs/sprints/archive/portfolio-agent-cost-accounting-followups/`.
