---
sprint_id: "PF-14-S05"
title: "Fail-closed full-history invariant"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 5
owner: "Jim Ricketts"
lane: "autoreview"
write_scope: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-14-S04"
created: 2026-08-24
updated: 2026-08-27
---

# PF-14-S05 — Fail-closed full-history invariant

## Execution mandate

- Deliver: deterministic enforcement that full-history children inherit their complete parent runtime.
- Excludes: history truncation behavior, provider readiness, and reviewer execution.

## Plan linkage

- Upstream: [plan touch record](../../../plans/proposed/arbitrary-model-autoreview.md#native-lifecycle-and-upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: every runtime override paired with `fork_turns=all` is rejected before child creation.

## Code boundaries

- Existing: `codex-rs/core/src/tools/handlers/multi_agents_common.rs::reject_full_fork_agent_type_override`
- Existing: `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs::handle_spawn_agent`
- Planned: `codex-rs/core/src/tools/handlers/multi_agents_common.rs::reject_full_fork_runtime_overrides`
- Tests: `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`

## Preconditions

- [ ] Allocate literal implementation/test/registration scopes and check cross-plan collisions before readiness.
- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Validate fork mode before applying role, provider, model, effort, or service-tier mutations.
- [ ] Reject each individual override and every adjacent combination with one stable recovery message: use `fork_turns=none|N` or inherit the full runtime.
- [ ] Apply the guard to provider-native, plaintext-adapter, and legacy-compatible spawn entry points.
- [ ] Prove a full-history request with no overrides inherits provider, model, effort, service tier, permissions, and history.
- [ ] Prove `none` and finite-turn forks still accept authorized explicit runtime selection.
- [ ] Hard-code Autoreview's internal fork mode to `none` and make any conflicting request unrepresentable.
- [ ] Add regressions showing rejected requests create no child, mailbox, pane, or provider call.

## Verification

- [ ] Run fix/format before final tests; execute Rust commands below from `codex-rs`.
- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Focused test: `just test -p codex-core full_history_runtime_override`
- [ ] Inheritance test: `just test -p codex-core spawn_agent_full_history`
- [ ] Cross-provider regression: `just test -p codex-core exact_runtime_dispatch`
- [ ] If this sprint changes an interactive path, complete its actual-key success/failure/recovery proof before completion; otherwise record why internal-only. S07 repeats integrated proof.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree override matrix output linked.
- [ ] Zero-child assertions linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
