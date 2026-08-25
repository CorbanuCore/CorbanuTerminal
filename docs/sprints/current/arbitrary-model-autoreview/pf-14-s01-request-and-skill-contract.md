---
sprint_id: "PF-14-S01"
title: "Explicit request and skill contract"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 1
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-24
updated: 2026-08-24
---

# PF-14-S01 — Explicit request and skill contract

## Execution mandate

- Deliver: one explicit `$autoreview` skill route with a typed host request.
- Excludes: packet construction, provider dispatch, reviewer execution, and TUI result rendering.

## Plan linkage

- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: missing provider/model opens host selection; no model guesses a route.

## Code boundaries

- Existing: `.codex/skills/*/SKILL.md`; `codex-rs/core/src/tools/spec_plan.rs`
- Planned: `.codex/skills/autoreview/SKILL.md`; `codex-rs/core/src/autoreview/request.rs`; `codex-rs/core/src/tools/handlers/autoreview.rs`
- Tests: `codex-rs/core/tests/suite/arbitrary_model_autoreview.rs`

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Create an original thin skill that requires explicit `$autoreview`, cites upstream commit `128a4ea6` and MIT provenance, and calls only the native Core handler.
- [ ] Define serde request types for `local|branch|commit`, ref, provider, model, effort, threshold, repository-relative prompt, and repository-relative datasets.
- [ ] Reject implicit invocation, missing or ambiguous runtime selection, invalid refs, absolute paths, traversal, and conflicting targets with stable typed errors.
- [ ] Keep Autoreview naming and events distinct from `/review` and Guardian `auto_review`.
- [ ] Register the structured handler without exposing raw shell or arbitrary command arguments.
- [ ] Add paraphrase and adjacent-case tests for explicit selection, missing selection, target validation, and non-Autoreview review requests.

## Verification

- [ ] Focused test: `cargo test -p codex-core arbitrary_model_autoreview_request`
- [ ] Integration test: `cargo test -p codex-core tool_spec_autoreview`
- [ ] TUI applicability resolved; request-selection keys and checkpoints recorded for PF-14-S07.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
