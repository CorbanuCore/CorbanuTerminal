---
sprint_id: "PF-08-S05"
title: "CPU packaging and artifact verification"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-08"
execution_order: 50
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-08-S04"
created: 2026-08-23
updated: 2026-08-24
---

# PF-08-S05 — CPU packaging and artifact verification

## Execution mandate

- Deliver: Package the selected model for a pinned local ONNX-class runtime.
- Excludes: implementation owned by any plan feature other than `PF-08`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-08` — Local prompt-injection classifier
- Acceptance advanced: Package the selected model for a pinned local ONNX-class runtime.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/`; `qa/security/classifier/`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-08-S04`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Quantize and export model/tokenizer with hashes, SBOM, signature, and version metadata.
- [ ] Enforce artifact size, peak memory, p50/p95 latency, and cold-start measurements.
- [ ] Reject signature, hash, tokenizer, runtime, and architecture mismatches before inference.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-08` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
