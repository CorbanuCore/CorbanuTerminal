---
sprint_id: "PF-23-S02"
title: "Aggressive deny and grant enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 41
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-17-S01, PF-23-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-23-S02 — Aggressive deny and grant enforcement

## Execution mandate

- Deliver: Aggressive denies every named sensitive surface unless one matching human grant is active.
- Excludes: grant TUI, signing adapters, new tools/providers, downgrade flow, and qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: one narrow grant cannot authorize an adjacent actor, resource, destination, operation, child, or post-expiry use.

## Code boundaries

- Existing: `codex-rs/core/src/config/permissions.rs`; `core/src/tools/router.rs`; `network-proxy/src/policy.rs`
- Planned: `codex-rs/core/src/security/aggressive.rs`
- Tests: planned sibling `aggressive_tests.rs`; affected permissions/network tests

## Preconditions

- [ ] PF-17-S01 and PF-23-S01 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Apply narrow grants to every registered broker, retrieval, browser-login, derived-data and disclosure operation; later adapters must use this common enforcement point, not add a second policy engine.

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
- [ ] TUI applicability: none; PF-25/PF-26 own interactive proof.

## Exit evidence

- [ ] Commit, denied-surface matrix, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
