---
sprint_id: "PF-09-S02"
title: "Quarantine store and retention"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-09"
execution_order: 53
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-09-S01"
created: 2026-08-23
updated: 2026-08-24
---

# PF-09-S02 — Quarantine store and retention

## Execution mandate

- Deliver: Keep suspicious content outside model context with bounded encrypted metadata.
- Excludes: implementation owned by any plan feature other than `PF-09`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-09` — Quarantine and pre-model rejection
- Acceptance advanced: Keep suspicious content outside model context with bounded encrypted metadata.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/tui/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/security-policy/src/ingress_outcome.rs`; `codex-rs/tui/src/quarantine/`
- Tests: planned `codex-rs/core/tests/quarantine.rs`; `codex-rs/tui/src/quarantine/tests.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-09-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Persist source metadata, reason, digests, expiry, and sealed artifact reference.
- [ ] Expose no raw-content API to agents or ordinary tool calls.
- [ ] Test restart, expiry, deletion, concurrent decisions, corruption, and unauthorized reads.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-core quarantine && just test -p codex-tui quarantine`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-09` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
