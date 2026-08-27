---
sprint_id: "PF-14-S01"
title: "Explicit request and skill contract"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 1
owner: "Jim Ricketts"
lane: "autoreview"
write_scope: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-27-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-14-S01 — Explicit request and skill contract

## Execution mandate

- Deliver: one explicit `$autoreview` skill route with a typed host request.
- Excludes: packet construction, provider dispatch, reviewer execution, and TUI result rendering.

## Plan linkage

- Upstream: [plan touch record](../../../plans/proposed/arbitrary-model-autoreview.md#native-lifecycle-and-upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: missing provider/model opens host selection; no model guesses a route.

## Code boundaries

- Existing: `.codex/skills/*/SKILL.md`; `codex-rs/core/src/tools/spec_plan.rs`
- Planned: `.codex/skills/autoreview/SKILL.md`; `codex-rs/core/src/autoreview/request.rs`; `codex-rs/core/src/tools/handlers/autoreview.rs`
- Tests: `codex-rs/core/tests/suite/arbitrary_model_autoreview.rs`

## Preconditions

- [ ] Allocate literal implementation/test/registration scopes and check cross-plan collisions before readiness.
- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Create an original thin skill that requires explicit `$autoreview`, cites upstream commit `128a4ea6` and MIT provenance, and calls only the native Core handler.
- [ ] Mirror the repository skill byte-for-byte in `.agents/skills/` and run `python3 scripts/check_portable_skills.py`; allocate both paths before readiness.
- [ ] Define serde request types for `local|branch|commit`, ref, provider, model, effort, threshold, repository-relative prompt, and repository-relative datasets.
- [ ] Reject implicit invocation, missing or ambiguous runtime selection, invalid refs, absolute paths, traversal, and conflicting targets with stable typed errors.
- [ ] Keep Autoreview naming and events distinct from `/review` and Guardian `auto_review`.
- [ ] Register the structured handler without exposing raw shell or arbitrary command arguments.
- [ ] Add paraphrase and adjacent-case tests for explicit selection, missing selection, target validation, and non-Autoreview review requests.

## Verification

- [ ] Run fix/format before final tests; execute Rust commands below from `codex-rs`.
- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Focused test: `just test -p codex-core arbitrary_model_autoreview_request`
- [ ] Integration test: `just test -p codex-core tool_spec_autoreview`
- [ ] If this sprint changes an interactive path, complete its actual-key success/failure/recovery proof before completion; otherwise record why internal-only. S07 repeats integrated proof.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree test output linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
