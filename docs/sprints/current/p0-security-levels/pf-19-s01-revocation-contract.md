---
sprint_id: "PF-19-S01"
title: "Revocation contract reconciliation"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-19"
execution_order: 5
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-17-S01, PF-18-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-19-S01 — Revocation contract reconciliation

## Execution mandate

- Deliver: verify durable revocation semantics that override grants, mandates, and cached decisions.
- Excludes: persistence wiring, kill-switch TUI, credential broker integration, and runtime adapters.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-19`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: revoked authority cannot start or resume another protected operation.

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing: `codex-rs/security-policy/src/{revocation,grant,mandate,lib}.rs`
- Tests: `codex-rs/security-policy/src/security_policy_tests.rs`

## Preconditions

- [ ] PF-17-S01 and PF-18-S01 are completed and archived.
- [ ] Exact worktree coordinates match the plan.
- [ ] Read `codex-rs/AGENTS.md` before corrective Rust work.

## Done

- [x] Sprint record is linked to PF-19.
- [x] Commit `8a3b416c26` added revocation events/state and policy contract tests.

## Remaining

- [ ] Specify revocation semantics for admitted operations, open channels and queued work, including the point after which no further protected dispatch is allowed; reject stale run generations without claiming already-completed effects can be undone.

- [ ] Review target, generation, ordering, idempotency, and cache-invalidation semantics.
- [ ] Prove revocation dominates active grants, pending mandates, and replay state.
- [ ] Add or correct race, duplicate-event, unknown-target, and rollback cases.
- [ ] Record any corrective commit and final public API.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-security-policy`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused final-tree test: `cd codex-rs && just test -p codex-security-policy revocation`.
- [ ] Regression: `cd codex-rs && just test -p codex-security-policy`.
- [ ] TUI applicability: none; PF-25 owns revocation and kill-switch interaction.

## Exit evidence

- [ ] Commits and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-19-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
