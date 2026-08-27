---
sprint_id: "PF-23-S02"
title: "Aggressive deny and grant enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 22
owner: "Jim Ricketts"
lane: "enforcement"
write_scope: "codex-rs/core/src/config/permissions.rs, codex-rs/core/src/tools/router.rs, codex-rs/network-proxy/src/policy.rs, codex-rs/core/src/security/aggressive.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-17-S01, PF-23-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-23-S02 — Aggressive deny and grant enforcement

## Execution mandate

- Deliver: Aggressive denies every named sensitive surface unless one matching human grant is active.
- Excludes: grant TUI, signing adapters, new tools/providers, downgrade flow, and qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Acceptance advanced: one narrow grant cannot authorize an adjacent actor, resource, destination, operation, child, or post-expiry use.

## Code boundaries

- Existing: `codex-rs/core/src/config/permissions.rs`; `core/src/tools/router.rs`; `network-proxy/src/policy.rs`
- Planned: `codex-rs/core/src/security/aggressive.rs`
- Tests: planned sibling `aggressive_tests.rs`; affected permissions/network tests

## Preconditions

- [ ] Every listed dependency is completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Default sensitive tools, accounts, credentials, protected data, financial actions, arbitrary egress, clipboard, and export to deny.
- [ ] Compose with existing permission/network policies so the security level can narrow but never override an existing denial.
- [ ] Admit only a valid `BoundedGrant` matching actor chain, action, resource, destination, limits, and expiry.
- [ ] Prevent grant inheritance from widening and deny unknown/unclassified sensitive surfaces.
- [ ] Add adjacent-surface, child-agent, expiry, limit, existing-deny, and unknown-state regressions.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-core && just fix -p codex-network-proxy`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused test: `cd codex-rs && just test -p codex-core aggressive`.
- [ ] Network/permission regressions: `cd codex-rs && just test -p codex-core permissions && just test -p codex-network-proxy policy`.
- [ ] TUI applicability: true-PTY default denial and expiry/recovery using the completed dispatch boundary; PF-25/PF-26 add final UI flows.

## Exit evidence

- [ ] Commit, denied-surface matrix, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
