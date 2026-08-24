---
sprint_id: "PF-07-S03"
title: "Unicode, control, link, and size normalization"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-07"
execution_order: 43
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-07-S02"
created: 2026-08-23
updated: 2026-08-23
---

# PF-07-S03 — Unicode, control, link, and size normalization

## Execution mandate

- Deliver: Normalize hostile encodings and bound content without destroying provenance.
- Excludes: implementation owned by any plan feature other than `PF-07`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-07` — Render-aware content cleaner
- Acceptance advanced: Normalize hostile encodings and bound content without destroying provenance.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/output.rs`; `codex-rs/ext/web-search/src/tool.rs`
- Planned: `codex-rs/web-sanitize/src/`; `codex-rs/ext/web-search/src/extraction.rs`
- Tests: planned `codex-rs/web-sanitize/tests/fixtures.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-07-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Normalize Unicode and line endings; strip terminal/control sequences.
- [ ] Represent links as labeled data and cap nesting, nodes, bytes, tokens, and link counts.
- [ ] Test homoglyphs, bidi controls, zero-width text, data URLs, deep DOM, and oversized pages.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-sanitize`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-07` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
