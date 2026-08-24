---
sprint_id: "PF-12-S08"
title: "Release security ledger and human acceptance"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 72
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-12-S03, PF-12-S06, PF-12-S07"
created: 2026-08-23
updated: 2026-08-23
---

# PF-12-S08 — Release security ledger and human acceptance

## Execution mandate

- Deliver: Publish the final security evidence packet and capture human release acceptance.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Publish the final security evidence packet and capture human release acceptance.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-12-S03, PF-12-S06, PF-12-S07`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Assemble classifier, policy, red-team, true-TUI, live-repo, runtime, and spend artifacts by commit.
- [ ] Verify every dependency sprint is archived completed and every open exception has an owner and expiry.
- [ ] Record human acceptance or rejection; block release when evidence, thresholds, or approval are absent.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: the human reviews evidence from real-TUI qualification before accepting release.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
