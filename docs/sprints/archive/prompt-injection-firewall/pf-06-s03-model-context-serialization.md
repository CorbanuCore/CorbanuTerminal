---
sprint_id: "PF-06-S03"
title: "Model-context serialization"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-06"
execution_order: 38
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-06-S02"
created: 2026-08-23
updated: 2026-08-24
---

# PF-06-S03 — Model-context serialization

## Execution mandate

- Deliver: Preserve instruction/data separation when envelopes enter provider requests.
- Excludes: implementation owned by any plan feature other than `PF-06`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-06` — Typed source and authority labels
- Acceptance advanced: Preserve instruction/data separation when envelopes enter provider requests.

## Code boundaries

- Existing: `codex-rs/protocol/src/models.rs`; `codex-rs/ext/extension-api/src/`; `codex-rs/core/src/context_manager/`
- Planned: `codex-rs/protocol/src/source_envelope.rs`; `codex-rs/core/src/source_ingress.rs`
- Tests: planned `codex-rs/core/tests/source_envelope.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-06-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Serialize system policy, human authority, and untrusted data through distinct typed paths.
- [ ] Keep provenance visible enough for reasoning without turning external text into instructions.
- [ ] Test fake role headers, conversation markup, delimiter injection, and provider translations.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-protocol source_envelope && just test -p codex-core source_ingress`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-06` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
