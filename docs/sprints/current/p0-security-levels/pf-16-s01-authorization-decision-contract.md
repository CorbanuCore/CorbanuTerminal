---
sprint_id: "PF-16-S01"
title: "Authorization decision contract reconciliation"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-16"
execution_order: 2
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-15-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-16-S01 — Authorization decision contract reconciliation

## Execution mandate

- Deliver: verify one deterministic, secret-free authorization request and decision contract.
- Excludes: grants, mandates, persistence, enforcement adapters, and user interface.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-16`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: protected operations use subject, resource, action, context, and typed allow/deny.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{authorization,digest,lib}.rs`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [ ] PF-15-S01 is completed and archived.
- [ ] Worktree, branch, and base commit still match the active plan.
- [ ] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Done

- [x] Sprint record is linked to PF-16.
- [x] Commit `d183036cb0` added actor chains, protected resources, actions, contexts, requests, decisions, and canonical digests.

## Remaining

- [ ] Review the existing types for complete human, agent, session, task, purpose, operation, destination, and resource bindings.
- [ ] Prove malformed or incomplete inputs deny without exposing protected values.
- [ ] Remove parallel policy concepts or widenings not authorized by PF-16.
- [ ] Record any corrective commit and final public API.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy authorization`.
- [ ] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [ ] TUI applicability: none; later PF-24/PF-25 sprints prove interactive use.

## Exit evidence

- [ ] Implementation and corrective commits recorded.
- [ ] Test output and reviewed API surface linked under `qa/security-levels/sprints/PF-16-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
