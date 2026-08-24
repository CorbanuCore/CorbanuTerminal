---
sprint_id: "PF-13-S04"
title: "Deterministic exact-runtime dispatch"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-13"
execution_order: 4
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-13-S03"
created: 2026-08-24
updated: 2026-08-24
---

# PF-13-S04 — Deterministic exact-runtime dispatch

## Execution mandate

- Deliver: one host-owned child dispatcher that reaches the exact requested provider/model without probe-and-retry routing.
- Excludes: review packet content, full-history policy, and report rendering.

## Plan linkage

- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-13`
- Acceptance advanced: observed reviewer route exactly matches the explicit request.

## Code boundaries

- Existing: `codex-rs/core/src/tools/spec_plan.rs::spec_for_model_request`
- Existing: `codex-rs/core/src/tools/handlers/multi_agents_v2/spawn.rs::handle_spawn_agent`
- Planned: `codex-rs/core/src/agent/runtime_dispatch.rs::dispatch_exact_runtime_child`
- Tests: `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`

## Preconditions

- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-13.

## Remaining

- [ ] Add a non-reserved exact-runtime request schema carrying provider, model, effort, service tier, task identity, and assignment.
- [ ] Resolve and validate the complete runtime before choosing collaboration message encoding.
- [ ] Select provider-native encoding only when source and target are compatible; otherwise select the plaintext mailbox adapter directly without a failed native call.
- [ ] Route existing V2 spawn and Autoreview through the shared dispatcher while preserving graph, mailbox, and canonical task-name semantics.
- [ ] Require PF-13-S03 readiness and runtime eligibility before dispatch; forbid model, provider, or service-tier fallback.
- [ ] Remove retry-only guidance that asks a model to infer transport after failure.
- [ ] Prove OpenAI-parent to Anthropic, OpenRouter, Kimi, Z.AI, local, and custom configured targets preserves exact assignment bytes and creates one child.
- [ ] Preserve same-provider native behavior and fail closed on unknown or mismatched provider/model pairs.

## Verification

- [ ] Focused test: `cargo test -p codex-core spawn_agent_explicit_runtime_supports_required_multimodel_pairs`
- [ ] Cross-provider test: `cargo test -p codex-core exact_runtime_dispatch`
- [ ] Schema test: `cargo test -p codex-core openai_reserved_collaboration_schema`
- [ ] TUI applicability resolved; exact route evidence is surfaced in PF-13-S07.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree route matrix output linked.
- [ ] One-child/no-probe assertions linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
