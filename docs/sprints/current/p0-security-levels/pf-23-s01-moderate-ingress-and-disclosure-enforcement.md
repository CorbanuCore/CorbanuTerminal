---
sprint_id: "PF-23-S01"
title: "Moderate ingress and disclosure enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 14
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-13-S05, PF-22-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-23-S01 — Moderate ingress and disclosure enforcement

## Execution mandate

- Deliver: Moderate deterministically blocks untrusted requests for secrets, protected financial data, policy changes, or protected actions.
- Excludes: Aggressive defaults, financial signing implementation, classifier training, TUI, and browser isolation.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Acceptance advanced: normal analysis continues while hostile instructions cannot gain authority or protected data.

## Code boundaries

- Existing: `codex-rs/core/src/tools/{router,registry}.rs`; `core/src/mcp_tool_call.rs`; `core/src/exec.rs`
- Planned: `codex-rs/core/src/security/{protected_surface,protected_surface_tests}.rs`
- Tests: affected tool, MCP, exec, context, vault, and policy suites

## Preconditions

- [ ] PF-13-S05 and PF-22-S01 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Classify protected surfaces by typed resource/action at the shared Core dispatch boundary.
- [ ] Treat project text, tool/MCP output, hooks, plugins, connectors, and external content as non-authoritative inputs.
- [ ] Deny vault enumeration/extraction, protected-financial-data disclosure, policy mutation, approval bypass, and value transfer without matching authority.
- [ ] Preserve non-protected analysis and existing Permissive behavior.
- [ ] Add paraphrase and adjacent-case regressions; literal prompt matching is not the primary router.
- [ ] Emit stable secret-free decisions and audit metadata.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-core protected_surface`.
- [ ] Boundary regressions: `cd codex-rs && just test -p codex-core tools:: && just test -p codex-core mcp_tool_call`.
- [ ] TUI applicability: none; PF-26-S02 owns interactive proof.

## Exit evidence

- [ ] Commit, typed surface matrix, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
