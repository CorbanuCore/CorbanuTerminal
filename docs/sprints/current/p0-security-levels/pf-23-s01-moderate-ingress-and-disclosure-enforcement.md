---
sprint_id: "PF-23-S01"
title: "Moderate ingress and disclosure enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 21
owner: "Jim Ricketts"
lane: "enforcement"
write_scope: "codex-rs/core/src/tools/router.rs, codex-rs/core/src/security/protected_surface.rs, codex-rs/core/src/security/protected_surface_tests.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-13-S05, PF-22-S01, PF-27-S01, PF-28-S01, PF-29-S02, PF-30-S02"
created: 2026-08-24
updated: 2026-08-27
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

- Existing: `codex-rs/core/src/tools/router.rs`; completed PF-28/PF-29/PF-30 adapters are read-only consumers.
- Planned: `codex-rs/core/src/security/{protected_surface,protected_surface_tests}.rs`
- Tests: affected tool, MCP, exec, context, vault, and policy suites

## Preconditions

- [ ] Every listed dependency is completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Consume PF-27 effective health/epoch contracts, PF-28 sink protections, PF-29 source taint, and PF-30 acquisition status; do not reimplement those boundaries.
- [ ] Reauthorize protected actions after untrusted reads; deny unknown provenance, unavailable controls, and stale epochs without changing Permissive.
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
- [ ] TUI applicability: true-PTY hostile-source/protected-action denial and safe retry before completion; PF-26 repeats final proof.

## Exit evidence

- [ ] Commit, typed surface matrix, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
