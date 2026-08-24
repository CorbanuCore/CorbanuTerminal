---
sprint_id: "PF-14-S06"
title: "Isolated bounded review runner"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 6
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-14-S02, PF-14-S04, PF-14-S05"
created: 2026-08-24
updated: 2026-08-24
---

# PF-14-S06 — Isolated bounded review runner

## Execution mandate

- Deliver: one cancellable exact-runtime reviewer that receives only scanned packets and returns one validated advisory report.
- Excludes: provider setup, code mutation, multi-model panels, and final documentation.

## Plan linkage

- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: exact-runtime success, malformed-result failure, cancellation, and durable result inspection.

## Code boundaries

- Existing: `codex-rs/core/src/tasks/review.rs::start_review_conversation`
- Existing: `codex-rs/core/src/exec_env.rs::remove_provider_auth_env_vars`
- Planned: `codex-rs/core/src/autoreview/runner.rs`; `codex-rs/core/src/autoreview/report.rs`
- Tests: `codex-rs/core/tests/suite/arbitrary_model_autoreview.rs::runner_*`

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Create the reviewer with `fork_turns=none`, an empty workspace, sanitized environment, read-only/no-approval policy, and no shell, web, image, MCP, skill, plugin, collaboration, or subagent tools.
- [ ] Provide only the scanned packet, fixed review rubric, threshold, and strict output schema to the selected provider/model.
- [ ] Run one call for normal input or one call per complete bounded packet; never nest, rerun, or switch runtime automatically.
- [ ] Emit start, heartbeat, packet progress, usage, cancel, completion, interruption, and failure events with provider/model identity.
- [ ] Validate severity, path, line range, title, explanation, confidence, and overall result; reject malformed or contradictory output.
- [ ] Merge packet reports deterministically, deduplicate identical findings, retain packet provenance, and never turn parse failure into “no findings.”
- [ ] Persist redacted request metadata and report for inspection without packet contents, auth values, or raw provider payloads.
- [ ] Prove the runner cannot read or mutate repository files and that cancellation prevents remaining provider calls.

## Verification

- [ ] Focused test: `cargo test -p codex-core arbitrary_model_autoreview_runner`
- [ ] Isolation test: `cargo test -p codex-core autoreview_isolation`
- [ ] Result test: `cargo test -p codex-core autoreview_report`
- [ ] TUI applicability resolved; event checkpoints are consumed by PF-14-S07.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree mock-provider and isolation output linked.
- [ ] Cancellation and malformed-output artifacts linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
