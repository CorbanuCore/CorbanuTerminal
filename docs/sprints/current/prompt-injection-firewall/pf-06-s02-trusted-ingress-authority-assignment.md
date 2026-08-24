---
sprint_id: "PF-06-S02"
title: "Trusted ingress authority assignment"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-06"
execution_order: 37
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-06-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-06-S02 — Trusted ingress authority assignment

## Execution mandate

- Deliver: Assign authority only at trusted user, system, external, tool, agent, and broker boundaries.
- Excludes: implementation owned by any plan feature other than `PF-06`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-06` — Typed source and authority labels
- Acceptance advanced: Assign authority only at trusted user, system, external, tool, agent, and broker boundaries.

## Code boundaries

- Existing: `codex-rs/protocol/src/models.rs`; `codex-rs/ext/extension-api/src/`; `codex-rs/core/src/context_manager/`
- Planned: `codex-rs/protocol/src/source_envelope.rs`; `codex-rs/core/src/source_ingress.rs`
- Tests: planned `codex-rs/core/tests/source_envelope.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-06-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Map every current ingress route to one non-forgeable authority value.
- [ ] Ignore labels printed inside content and reject attempts to set authority from tool arguments.
- [ ] Fail closed for unclassified ingress and add source-specific regressions.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-protocol source_envelope && just test -p codex-core source_ingress`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-06` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
