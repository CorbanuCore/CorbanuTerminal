---
sprint_id: "PF-04-S01"
title: "web.run compatibility facade"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-04"
execution_order: 19
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-24
---

# PF-04-S01 — web.run compatibility facade

## Execution mandate

- Deliver: Preserve the current web.run command schema while inserting a broker boundary.
- Excludes: implementation owned by any plan feature other than `PF-04`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-04` — Search-provider broker and stable web-tool facade
- Acceptance advanced: Preserve the current web.run command schema while inserting a broker boundary.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/codex-api/src/search.rs`; `codex-rs/core/src/client.rs`
- Planned: `codex-rs/ext/web-search/src/broker/`; `codex-rs/protocol/src/web_research.rs`
- Tests: `codex-rs/core/tests/suite/web_search.rs`; planned `codex-rs/ext/web-search/tests/broker.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Freeze search_query, image_query, open, click, find, screenshot, and response compatibility fixtures.
- [ ] Replace direct provider dispatch with one internal broker request interface.
- [ ] Prove Permissive requests and history events remain byte- or snapshot-compatible.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension && just test -p codex-core web_search`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: existing web.run query and result history render through real TUI.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-04` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
