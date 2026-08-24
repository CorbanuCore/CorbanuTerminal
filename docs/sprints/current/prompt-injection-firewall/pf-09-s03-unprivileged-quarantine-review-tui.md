---
sprint_id: "PF-09-S03"
title: "Unprivileged quarantine review TUI"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-09"
execution_order: 54
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-09-S02"
created: 2026-08-23
updated: 2026-08-23
---

# PF-09-S03 — Unprivileged quarantine review TUI

## Execution mandate

- Deliver: Let the human inspect source and reason without granting content authority.
- Excludes: implementation owned by any plan feature other than `PF-09`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-09` — Quarantine and pre-model rejection
- Acceptance advanced: Let the human inspect source and reason without granting content authority.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/tui/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/security-policy/src/ingress_outcome.rs`; `codex-rs/tui/src/quarantine/`
- Tests: planned `codex-rs/core/tests/quarantine.rs`; `codex-rs/tui/src/quarantine/tests.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-09-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Render metadata and raw content in a non-agent, non-tool, non-copying review surface.
- [ ] Offer skip, delete, retry safe source, retry detector, and close actions.
- [ ] Test keys, cancellation, resizing, large content, escape sequences, and no transcript insertion.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-core quarantine && just test -p codex-tui quarantine`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: inspect, skip, delete, retry, cancel, and resume with keys sent.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-09` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
