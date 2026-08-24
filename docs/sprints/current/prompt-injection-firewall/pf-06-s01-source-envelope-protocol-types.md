---
sprint_id: "PF-06-S01"
title: "Source-envelope protocol types"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-06"
execution_order: 36
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-06-S01 — Source-envelope protocol types

## Execution mandate

- Deliver: Define immutable provenance and authority fields for every model-visible segment.
- Excludes: implementation owned by any plan feature other than `PF-06`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-06` — Typed source and authority labels
- Acceptance advanced: Define immutable provenance and authority fields for every model-visible segment.

## Code boundaries

- Existing: `codex-rs/protocol/src/models.rs`; `codex-rs/ext/extension-api/src/`; `codex-rs/core/src/context_manager/`
- Planned: `codex-rs/protocol/src/source_envelope.rs`; `codex-rs/core/src/source_ingress.rs`
- Tests: planned `codex-rs/core/tests/source_envelope.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Add source id, kind, authority, origin, retrieval time, content type, transforms, digest, and classifier fields.
- [ ] Define versioning and forward-compatible unknown-field behavior.
- [ ] Test canonical serialization, missing fields, unknown authority, and digest mismatch.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-protocol source_envelope && just test -p codex-core source_ingress`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-06` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
