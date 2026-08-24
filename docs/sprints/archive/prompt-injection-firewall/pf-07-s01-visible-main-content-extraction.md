---
sprint_id: "PF-07-S01"
title: "Visible main-content extraction"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-07"
execution_order: 41
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-24
---

# PF-07-S01 — Visible main-content extraction

## Execution mandate

- Deliver: Extract bounded visible main-body content from isolated retrieval output.
- Excludes: implementation owned by any plan feature other than `PF-07`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-07` — Render-aware content cleaner
- Acceptance advanced: Extract bounded visible main-body content from isolated retrieval output.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/output.rs`; `codex-rs/ext/web-search/src/tool.rs`
- Planned: `codex-rs/web-sanitize/src/`; `codex-rs/ext/web-search/src/extraction.rs`
- Tests: planned `codex-rs/web-sanitize/tests/fixtures.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Define accepted HTML/text inputs and normalized Markdown output.
- [ ] Select main article/body regions while retaining source and link provenance.
- [ ] Add fixtures for articles, tables, filings, transcripts, dynamic shells, and empty pages.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-sanitize`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-07` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
