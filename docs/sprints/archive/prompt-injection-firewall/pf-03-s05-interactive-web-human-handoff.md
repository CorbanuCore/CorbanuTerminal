---
sprint_id: "PF-03-S05"
title: "Interactive web human handoff"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-03"
execution_order: 18
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-03-S04"
created: 2026-08-23
updated: 2026-08-24
---

# PF-03-S05 — Interactive web human handoff

## Execution mandate

- Deliver: Provide a separate trusted path for login, CAPTCHA, MFA, passkey, and interactive pages.
- Excludes: implementation owned by any plan feature other than `PF-03`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-03` — Isolated public-web retrieval
- Acceptance advanced: Provide a separate trusted path for login, CAPTCHA, MFA, passkey, and interactive pages.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/network-proxy/src/`; `codex-rs/core/src/spawn.rs`
- Planned: `codex-rs/ext/web-search/src/isolated_fetch.rs`; `resources/web-retriever/`
- Tests: planned `codex-rs/ext/web-search/tests/isolated_fetch.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-03-S04`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Define a human-only handoff state that grants no browser handle to the agent.
- [ ] Show origin, requested task, data exposure, and return conditions before opening.
- [ ] Return a redacted completion receipt or cancellation without page text becoming authority.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension isolated_fetch`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: key-driven handoff, cancel, completion, and resume flow.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-03` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
