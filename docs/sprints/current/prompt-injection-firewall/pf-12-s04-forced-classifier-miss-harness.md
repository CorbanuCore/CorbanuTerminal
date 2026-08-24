---
sprint_id: "PF-12-S04"
title: "Forced classifier-miss harness"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 68
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-12-S01, PF-12-S02"
created: 2026-08-23
updated: 2026-08-23
---

# PF-12-S04 — Forced classifier-miss harness

## Execution mandate

- Deliver: Prove that downstream controls contain attacks when the classifier deliberately misses.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Prove that downstream controls contain attacks when the classifier deliberately misses.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-12-S01, PF-12-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Inject a deterministic allow verdict for hostile fixtures without changing downstream policy.
- [ ] Assert provenance, sanitization, broker policy, approvals, and canary controls still prevent harm.
- [ ] Produce per-layer containment evidence and fail when classifier output becomes sole authority.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
