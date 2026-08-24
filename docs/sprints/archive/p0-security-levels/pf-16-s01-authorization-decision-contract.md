---
sprint_id: "PF-16-S01"
title: "Authorization decision contract reconciliation"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-16"
execution_order: 2
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-15-S01"
created: 2026-08-24
updated: 2026-08-24
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

## Done

- [x] Sprint record is linked to PF-16.
- [x] Commit `d183036cb0` added actor chains, protected resources, actions, contexts, requests, decisions, and canonical digests.
- [x] Reviewed all required identity and action bindings; corrective commit `6af50d0a5f` added bounded session, task, purpose, and operation fields to the canonical request digest.
- [x] Added malformed and incomplete request tests that fail without echoing a protected canary value.
- [x] Confirmed the public surface contains one authorization contract without unauthorized widening.
- [x] Recorded corrective paths and public API in evidence commit `940680e8e6`.

## Remaining

None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy` passed.
- [x] Format: `cd codex-rs && just fmt`; final diff inspected.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy authorization` passed 3 tests.
- [x] Regression: `cd codex-rs && just test -p codex-security-policy` passed 12 tests.
- [x] TUI applicability: none; later PF-24/PF-25 sprints prove interactive use.

## Exit evidence

- [x] Original `d183036cb0`, corrective `6af50d0a5f`, and evidence `940680e8e6` commits recorded.
- [x] Test output and reviewed API surface recorded at `qa/security-levels/sprints/PF-16-S01/evidence.md`.
- [x] Ledgers reflect reality and the completed record is archived.
