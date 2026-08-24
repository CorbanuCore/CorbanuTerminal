---
sprint_id: "PF-04-S06"
title: "SearXNG metasearch adapter"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-04"
execution_order: 24
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-04-S05"
created: 2026-08-23
updated: 2026-08-24
---

# PF-04-S06 — SearXNG metasearch adapter

## Execution mandate

- Deliver: Implement an isolated maintained SearXNG provider lane.
- Excludes: implementation owned by any plan feature other than `PF-04`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-04` — Search-provider broker and stable web-tool facade
- Acceptance advanced: Implement an isolated maintained SearXNG provider lane.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/codex-api/src/search.rs`; `codex-rs/core/src/client.rs`
- Planned: `codex-rs/ext/web-search/src/broker/`; `codex-rs/protocol/src/web_research.rs`
- Tests: `codex-rs/core/tests/suite/web_search.rs`; planned `codex-rs/ext/web-search/tests/broker.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-04-S05`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Define service health, JSON search contract, upstream engine policy, and version pin.
- [ ] Constrain the local service endpoint and its outbound network separately.
- [ ] Normalize results and test CAPTCHA degradation, bad engines, partial results, and service restart.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension && just test -p codex-core web_search`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-04` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
