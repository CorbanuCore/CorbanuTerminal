---
sprint_id: "PF-03-S04"
title: "No host-browser fallback"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-03"
execution_order: 17
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-03-S03"
created: 2026-08-23
updated: 2026-08-23
---

# PF-03-S04 — No host-browser fallback

## Execution mandate

- Deliver: Make retriever failure terminal for Moderate and Aggressive web fetches.
- Excludes: implementation owned by any plan feature other than `PF-03`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-03` — Isolated public-web retrieval
- Acceptance advanced: Make retriever failure terminal for Moderate and Aggressive web fetches.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/network-proxy/src/`; `codex-rs/core/src/spawn.rs`
- Planned: `codex-rs/ext/web-search/src/isolated_fetch.rs`; `resources/web-retriever/`
- Tests: planned `codex-rs/ext/web-search/tests/isolated_fetch.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-03-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Apply security-level routing before any browser tool registration or fallback decision.
- [ ] Return typed unavailable, timeout, and policy-denied results with safe retry choices.
- [ ] Add regressions proving failure never invokes host browser content, navigation, or eval.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension isolated_fetch`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-03` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
