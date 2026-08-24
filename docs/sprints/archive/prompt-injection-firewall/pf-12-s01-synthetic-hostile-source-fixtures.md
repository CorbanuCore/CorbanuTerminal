---
sprint_id: "PF-12-S01"
title: "Synthetic hostile-source fixtures"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 65
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-24
---

# PF-12-S01 — Synthetic hostile-source fixtures

## Execution mandate

- Deliver: Create versioned hostile and benign source fixtures spanning supported ingress channels.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Create versioned hostile and benign source fixtures spanning supported ingress channels.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Cover visible, hidden, encoded, fragmented, multilingual, role-spoofed, and tool-output injections.
- [ ] Pair attacks with clean adjacent cases and explicit expected policy, classifier, and audit outcomes.
- [ ] Version provenance and fixture licenses; add schema validation and duplicate/leakage checks.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; real-TUI qualification is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
