---
sprint_id: "PF-22-S01"
title: "Runtime policy and agent inheritance"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-22"
execution_order: 8
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-19-S01, PF-20-S01, PF-21-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-22-S01 — Runtime policy and agent inheritance

## Execution mandate

- Deliver: one Core-owned effective-policy state inherited by active and newly spawned agents.
- Excludes: individual protected-surface adapters, credential resolution, TUI, and qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-22`
- Acceptance advanced: no agent, tool, project input, hook, plugin, connector, or MCP server can weaken the human-selected level.

## Code boundaries

- Existing: `codex-rs/core/src/config/mod.rs`; `codex-rs/core/src/agent/{control,registry}.rs`
- Planned: `codex-rs/core/src/security/{mod,effective_policy}.rs`
- Tests: planned `codex-rs/core/src/security/effective_policy_tests.rs`

## Preconditions

- [x] PF-19-S01, PF-20-S01, and PF-21-S01 are completed and archived.
- [x] Read `codex-rs/AGENTS.md` and `codex-rs/core/AGENTS.md`.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked to PF-22.
- [x] Runtime composition shipped in `9711bcd94c`; auxiliary-agent inheritance was corrected in `2a0f3abfd0`.
- [x] Final-tree evidence was committed in `00d1ea2039`.

## Remaining

- [x] Build effective policy only from persisted human state and existing lower-level policies.
- [x] Make Permissive compose to the existing decision unchanged; Moderate/Aggressive may only narrow.
- [x] Propagate level, actor chain, task/session identity, revocation generation, and kill state to child creation.
- [x] Recompute active-session state atomically after a confirmed level change; reject unknown/corrupt state.
- [x] Add paraphrase/adjacent cases proving model and project content cannot route to a policy mutation.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-core`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-core effective_policy`.
- [x] Spawn regression: `cd codex-rs && just test -p codex-core security_inheritance`.
- [x] TUI applicability: none; PF-24 and PF-25 exercise the state through the TUI.

## Exit evidence

- [x] Implementation commit and final changed paths recorded.
- [x] Test output linked under `qa/security-levels/sprints/PF-22-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
