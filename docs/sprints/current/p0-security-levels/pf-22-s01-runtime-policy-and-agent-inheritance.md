---
sprint_id: "PF-22-S01"
title: "Runtime policy and agent inheritance"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-22"
execution_order: 8
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-19-S01, PF-20-S01, PF-21-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-22-S01 — Runtime policy and agent inheritance

## Execution mandate

- Deliver: one Core-owned effective-policy state inherited by active and newly spawned agents.
- Excludes: individual protected-surface adapters, credential resolution, TUI, and qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-22`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: no agent, tool, project input, hook, plugin, connector, or MCP server can weaken the human-selected level.

## Code boundaries

- OpenClaw adoption reference: [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing: `codex-rs/core/src/config/mod.rs`; `codex-rs/core/src/agent/{control,registry}.rs`
- Planned: `codex-rs/core/src/security/{mod,effective_policy}.rs`
- Tests: planned `codex-rs/core/src/security/effective_policy_tests.rs`

## Preconditions

- [ ] PF-19-S01, PF-20-S01, and PF-21-S01 are completed and archived.
- [ ] Read `codex-rs/AGENTS.md` and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked to PF-22.

## Remaining

- [ ] Record configured versus creator-required/effective containment and enforce inherited restrictions across every supported child/provider runtime; missing backend or identity must not silently downgrade a protected session.

- [ ] Build effective policy only from persisted human state and existing lower-level policies.
- [ ] Make Permissive compose to the existing decision unchanged; Moderate/Aggressive may only narrow.
- [ ] Propagate level, actor chain, task/session identity, revocation generation, and kill state to child creation.
- [ ] Recompute active-session state atomically after a confirmed level change; reject unknown/corrupt state.
- [ ] Add paraphrase/adjacent cases proving model and project content cannot route to a policy mutation.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused final-tree test: `cd codex-rs && just test -p codex-core effective_policy`.
- [ ] Spawn regression: `cd codex-rs && just test -p codex-core security_inheritance`.
- [ ] TUI applicability: none; PF-24 and PF-25 exercise the state through the TUI.

## Exit evidence

- [ ] Implementation commit and final changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-22-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
