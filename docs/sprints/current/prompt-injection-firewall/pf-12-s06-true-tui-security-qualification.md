---
sprint_id: "PF-12-S06"
title: "True-TUI security qualification"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 70
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-12-S02, PF-12-S04, PF-12-S05"
created: 2026-08-23
updated: 2026-08-23
---

# PF-12-S06 — True-TUI security qualification

## Execution mandate

- Deliver: Run the security profiles and adversarial workflows in a real terminal with keys sent.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Run the security profiles and adversarial workflows in a real terminal with keys sent.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-12-S02, PF-12-S04, PF-12-S05`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Cover `/security` selection, restart persistence, web retrieval, quarantine, approval, denial, and recovery.
- [ ] Drive the real binary through a PTY and retain key transcripts, screen captures, logs, and receipts.
- [ ] Prove permissive, moderate, and aggressive behavior plus cancellation, timeout, and malformed-state paths.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: all acceptance paths run against the real TUI; exec-only proof is rejected.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
