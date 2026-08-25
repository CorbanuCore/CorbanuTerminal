---
sprint_id: "PF-04-S04"
title: "Exa search adapter"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-04"
execution_order: 22
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-04-S03"
created: 2026-08-23
updated: 2026-08-24
---

# PF-04-S04 — Exa search adapter

## Execution mandate

- Deliver: Implement Exa search and approved fetch capabilities behind the provider contract.
- Excludes: implementation owned by any plan feature other than `PF-04`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-04` — Search-provider broker and stable web-tool facade
- Acceptance advanced: Implement Exa search and approved fetch capabilities behind the provider contract.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/codex-api/src/search.rs`; `codex-rs/core/src/client.rs`
- Planned: `codex-rs/ext/web-search/src/broker/`; `codex-rs/protocol/src/web_research.rs`
- Tests: `codex-rs/core/tests/suite/web_search.rs`; planned `codex-rs/ext/web-search/tests/broker.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-04-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Resolve the Exa credential by reference inside the request executor.
- [ ] Map query, domain, recency, locale, and result limits to the adapter.
- [ ] Normalize results and add malformed, hostile-snippet, quota, timeout, and auth tests.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension && just test -p codex-core web_search`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-04` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
