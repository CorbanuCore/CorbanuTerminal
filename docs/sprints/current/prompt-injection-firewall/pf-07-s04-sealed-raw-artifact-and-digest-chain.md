---
sprint_id: "PF-07-S04"
title: "Sealed raw artifact and digest chain"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-07"
execution_order: 44
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-07-S03"
created: 2026-08-23
updated: 2026-08-23
---

# PF-07-S04 — Sealed raw artifact and digest chain

## Execution mandate

- Deliver: Retain investigation evidence outside model context under explicit policy.
- Excludes: implementation owned by any plan feature other than `PF-07`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-07` — Render-aware content cleaner
- Acceptance advanced: Retain investigation evidence outside model context under explicit policy.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/output.rs`; `codex-rs/ext/web-search/src/tool.rs`
- Planned: `codex-rs/web-sanitize/src/`; `codex-rs/ext/web-search/src/extraction.rs`
- Tests: planned `codex-rs/web-sanitize/tests/fixtures.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-07-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Hash raw input, each transform, and final extracted content.
- [ ] Store sealed raw artifacts with retention, access, and deletion policy.
- [ ] Test digest-chain verification, unauthorized access, expiry, corruption, and redacted exports.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-sanitize`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-07` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
