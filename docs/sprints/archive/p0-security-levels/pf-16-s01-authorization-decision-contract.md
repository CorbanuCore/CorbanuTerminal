---
sprint_id: "PF-16-S01"
title: "Authorization decision contract reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-16"
execution_order: 2
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-15-S01"
created: 2026-08-24
updated: 2026-08-25
---

# PF-16-S01 — Authorization decision contract reconciliation

## Execution mandate

- Deliver: verify one deterministic, secret-free authorization request and decision contract.
- Excludes: grants, mandates, persistence, enforcement adapters, and user interface.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-16`
- Acceptance advanced: protected operations use subject, resource, action, context, and typed allow/deny.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{authorization,digest,lib}.rs`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [x] PF-15-S01 is completed and archived.
- [x] Worktree, branch, and base commit still match the active plan.
- [x] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Record creation

- [x] Sprint record is linked to PF-16.
- [x] Commit `d183036cb0` added actor chains, protected resources, actions, contexts, requests, decisions, and canonical digests.

## Done

- [x] Review the existing types for complete human, agent, session, task, purpose, operation, destination, and resource bindings.
- [x] Prove malformed or incomplete inputs deny without exposing protected values.
- [x] Remove parallel policy concepts or widenings not authorized by PF-16.
- [x] Record any corrective commit and final public API.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy authorization`.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [x] TUI applicability: none; later PF-24/PF-25 sprints prove interactive use.

## Exit evidence

- [x] Implementation and corrective commits recorded.
- [x] Test output and reviewed API surface linked under `qa/security-levels/sprints/PF-16-S01/`.
- [x] Ledgers reflect reality and the completed record is archived.
