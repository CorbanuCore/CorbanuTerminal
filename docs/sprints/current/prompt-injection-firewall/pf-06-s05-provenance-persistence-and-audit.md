---
sprint_id: "PF-06-S05"
title: "Provenance persistence and audit"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-06"
execution_order: 40
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-06-S04"
created: 2026-08-23
updated: 2026-08-23
---

# PF-06-S05 — Provenance persistence and audit

## Execution mandate

- Deliver: Persist source lineage and transformations without persisting protected contents.
- Excludes: implementation owned by any plan feature other than `PF-06`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-06` — Typed source and authority labels
- Acceptance advanced: Persist source lineage and transformations without persisting protected contents.

## Code boundaries

- Existing: `codex-rs/protocol/src/models.rs`; `codex-rs/ext/extension-api/src/`; `codex-rs/core/src/context_manager/`
- Planned: `codex-rs/protocol/src/source_envelope.rs`; `codex-rs/core/src/source_ingress.rs`
- Tests: planned `codex-rs/core/tests/source_envelope.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-06-S04`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Store envelope ids, digests, policy decisions, transform versions, and redacted origin.
- [ ] Link search, fetch, classifier, quarantine, and action receipts by ids.
- [ ] Test restart, compaction, replay, deletion, and secret-free audit export.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-protocol source_envelope && just test -p codex-core source_ingress`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-06` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
