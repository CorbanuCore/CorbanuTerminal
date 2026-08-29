---
sprint_id: "PF-23-S01"
title: "Moderate ingress and disclosure enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 40
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-13-S05, PF-22-S02, PF-30-S03"
created: 2026-08-24
updated: 2026-08-28
---

# PF-23-S01 — Moderate ingress and disclosure enforcement

## Execution mandate

- Deliver: Moderate deterministically blocks untrusted requests for secrets, protected financial data, policy changes, or protected actions.
- Excludes: Aggressive defaults, financial signing implementation, classifier training, TUI, and browser isolation.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: normal analysis continues while hostile instructions cannot gain authority or protected data.

## Code boundaries

- Existing: `codex-rs/core/src/tools/{router,registry}.rs`; `core/src/mcp_tool_call.rs`; `core/src/exec.rs`
- Planned: `codex-rs/core/src/security/{protected_surface,protected_surface_tests}.rs`
- Tests: affected tool, MCP, exec, context, vault, and policy suites

## Preconditions

- [ ] PF-13-S05, PF-22-S02, PF-30-S03 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Implement the plan's action/profile usability matrix with conservative data and control-flow ancestry; a model selecting clean-looking human values after hostile content is not trusted reconstruction. No runtime Moderate activation until all required subsystems qualify.

- [ ] Integrate PF-30 durable provenance and post-taint checks; detector output never supplies authority, and a new turn or summary never clears taint.
- [ ] Register required protected-mode subsystems and deny unsupported/unready routes; final activation requires the full plan readiness matrix, not this dispatch slice alone.

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
