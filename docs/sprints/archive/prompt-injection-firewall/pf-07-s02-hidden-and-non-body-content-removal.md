---
sprint_id: "PF-07-S02"
title: "Hidden and non-body content removal"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-07"
execution_order: 42
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-07-S01"
created: 2026-08-23
updated: 2026-08-24
---

# PF-07-S02 — Hidden and non-body content removal

## Execution mandate

- Deliver: Remove scripts, styles, comments, hidden nodes, overlays, and non-task metadata.
- Excludes: implementation owned by any plan feature other than `PF-07`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-07` — Render-aware content cleaner
- Acceptance advanced: Remove scripts, styles, comments, hidden nodes, overlays, and non-task metadata.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/output.rs`; `codex-rs/ext/web-search/src/tool.rs`
- Planned: `codex-rs/web-sanitize/src/`; `codex-rs/ext/web-search/src/extraction.rs`
- Tests: planned `codex-rs/web-sanitize/tests/fixtures.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-07-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Implement render-aware visibility and non-content filters in the isolated lane.
- [ ] Record removal counts and transform version without exposing removed text to the model.
- [ ] Test CSS hiding, aria-hidden, zero-size, off-screen, comments, SVG, and metadata injection.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-sanitize`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-07` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
