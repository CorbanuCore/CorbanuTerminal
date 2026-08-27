---
sprint_id: "PF-14-S03"
title: "Shared Core provider readiness"
status: draft
plan_file: "docs/plans/proposed/arbitrary-model-autoreview.md"
plan_feature: "PF-14"
execution_order: 3
owner: "Jim Ricketts"
lane: "autoreview"
write_scope: "UNALLOCATED"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-14-S01, PF-13-S05, PF-27-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-14-S03 — Shared Core provider readiness

## Execution mandate

- Deliver: one Core-owned authorization and authentication preflight for every native child route.
- Excludes: transport selection, review packets, and reviewer output.

## Plan linkage

- Upstream: [plan touch record](../../../plans/proposed/arbitrary-model-autoreview.md#native-lifecycle-and-upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [Arbitrary-model Autoreview](../../../plans/proposed/arbitrary-model-autoreview.md)
- Feature: `PF-14`
- Acceptance advanced: unauthorized or unauthenticated selection fails before child creation.

## Code boundaries

- Existing: `codex-rs/tui/src/spawn_orchestration.rs::ensure_native_spawn_provider_ready`
- Existing: `codex-rs/core/src/tools/handlers/multi_agents_common.rs::ensure_spawn_provider_authorized`
- Planned: `codex-rs/core/src/agent/provider_readiness.rs`
- Tests: `codex-rs/core/src/tools/handlers/multi_agents_tests.rs`; `codex-rs/tui/src/spawn_orchestration.rs`

## Preconditions

- [ ] Allocate literal implementation/test/registration scopes and check cross-plan collisions before readiness.
- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Plan is active.
- [ ] Dependencies are completed.
- [ ] Worktree, branch, and base commit are exact and match the plan.

## Done

- [x] Sprint record created and linked to PF-14.

## Remaining

- [ ] Move provider existence, catalog eligibility, operator allowlist, account-auth, env-key, stored-key, and provider auth-command validation behind one Core API.
- [ ] Return typed, actionable errors containing provider identity and recovery action but no credential value.
- [ ] Call the Core preflight from V2 native/plaintext spawn and the planned Autoreview path before AgentControl creates a child.
- [ ] Replace TUI-owned duplicate logic with the Core result while preserving current user-facing recovery messages.
- [ ] Verify custom, OpenAI account, Claude Plan, API-key, cloud, and local providers use the appropriate readiness branch.
- [ ] Add regressions proving missing auth creates no thread, pane, mailbox, provider request, or secret-bearing log.
- [ ] Preserve `agents.provider_allowlist` as operator authorization; availability never expands it.
- [ ] Reuse qualified PF-13 broker and PF-27 effective-policy contracts; readiness must not resolve secrets into generic child/model-visible state or weaken stronger-mode helper restrictions.

## Verification

- [ ] Run fix/format before final tests; execute Rust commands below from `codex-rs`.
- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Focused test: `just test -p codex-core provider_readiness`
- [ ] Spawn regression: `just test -p codex-core spawn_agent_provider`
- [ ] TUI regression: `just test -p codex-tui native_spawn_provider`
- [ ] If this sprint changes an interactive path, complete its actual-key success/failure/recovery proof before completion; otherwise record why internal-only. S07 repeats integrated proof.

## Exit evidence

- [ ] Implementation commit recorded.
- [ ] Final-tree Core and TUI test output linked.
- [ ] Zero-child/zero-request assertions linked.
- [ ] `Done` and `Remaining` ledgers reflect reality.
- [ ] Completed record moved to `docs/sprints/archive/arbitrary-model-autoreview/`.
