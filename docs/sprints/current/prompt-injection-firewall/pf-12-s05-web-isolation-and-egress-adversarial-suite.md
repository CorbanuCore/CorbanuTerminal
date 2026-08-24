---
sprint_id: "PF-12-S05"
title: "Web isolation and egress adversarial suite"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-12"
execution_order: 69
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-03-S04, PF-04-S12, PF-05-S05, PF-12-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-12-S05 — Web isolation and egress adversarial suite

## Execution mandate

- Deliver: Exercise browser, provider, redirect, DNS, and native-web bypass attacks end to end.
- Excludes: implementation owned by any plan feature other than `PF-12`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-12` — Security regression and red-team harness
- Acceptance advanced: Exercise browser, provider, redirect, DNS, and native-web bypass attacks end to end.

## Code boundaries

- Existing: `codex-rs/core/tests/`; `codex-rs/tui/tests/`; `benchmarks/`
- Planned: `scripts/security/`; `qa/security/`; `qa/release/security/`
- Tests: `scripts/security/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-03-S04, PF-04-S12, PF-05-S05, PF-12-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Serve hostile pages and provider responses covering redirects, rebinding, hidden text, and oversized bodies.
- [ ] Attempt direct host browser, native web, loopback, metadata, private-network, and raw-socket bypasses.
- [ ] Assert isolated execution, normalized output, audit records, and zero protected-data exposure.

## Verification

- [ ] Focused final-tree command: `python3 -m unittest discover -s scripts/security/tests -v`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-12` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
