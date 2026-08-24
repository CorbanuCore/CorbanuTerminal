---
sprint_id: "PF-04-S08"
title: "Normalized results and stable ids"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-04"
execution_order: 26
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-04-S07"
created: 2026-08-23
updated: 2026-08-24
---

# PF-04-S08 — Normalized results and stable ids

## Execution mandate

- Deliver: Convert every provider response to bounded result records with session-stable ids.
- Excludes: implementation owned by any plan feature other than `PF-04`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-04` — Search-provider broker and stable web-tool facade
- Acceptance advanced: Convert every provider response to bounded result records with session-stable ids.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/codex-api/src/search.rs`; `codex-rs/core/src/client.rs`
- Planned: `codex-rs/ext/web-search/src/broker/`; `codex-rs/protocol/src/web_research.rs`
- Tests: `codex-rs/core/tests/suite/web_search.rs`; planned `codex-rs/ext/web-search/tests/broker.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-04-S07`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Define title, snippet, canonical URL, timestamps, digests, provider, and classifier fields.
- [ ] Store result-id to canonical-URL binding outside model control.
- [ ] Test cross-provider collisions, duplicate URLs, tampered ids, pagination, and size limits.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension && just test -p codex-core web_search`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-04` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
