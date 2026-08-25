---
sprint_id: "PF-23-S03"
title: "Downgrade, restart, and inheritance enforcement"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 16
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-19-S01, PF-20-S01, PF-23-S02"
created: 2026-08-24
updated: 2026-08-25
---

# PF-23-S03 — Downgrade, restart, and inheritance enforcement

## Execution mandate

- Deliver: confirmed level changes atomically invalidate incompatible authority and survive restart without a weaker interval.
- Excludes: selection TUI, grant/kill-switch TUI, content classifiers, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-23`
- Acceptance advanced: no old grant, mandate, cached decision, child state, or pending approval can be replayed after change/restart.

## Code boundaries

- Existing: `codex-rs/core/src/config/{mod,edit}.rs`; `core/src/agent/control.rs`
- Planned: `codex-rs/core/src/security/{transition,recovery}.rs`
- Tests: planned sibling transition/recovery tests; config and agent-control suites

## Preconditions

- [ ] PF-19-S01, PF-20-S01, and PF-23-S02 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-23.

## Remaining

- [ ] Define prepare/commit/cancel transition state so only trusted human confirmation commits a level change.
- [ ] Atomically advance revocation generation and invalidate cached decisions, grants, mandates, approvals, and incompatible child authority.
- [ ] Persist level and restrictive revocation/kill state before protected work resumes.
- [ ] Recover restart/crash state without transient Permissive fallback or stale approval restoration.
- [ ] Add cancel, persistence failure, crash boundary, downgrade, child-resume, and concurrent protected-action regressions.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-core security_transition`.
- [ ] Recovery regressions: `cd codex-rs && just test -p codex-core security_recovery`.
- [ ] TUI applicability: none; PF-24-S02 and PF-25-S02 consume this API.

## Exit evidence

- [ ] Commit, transition state diagram, and changed paths recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-23-S03/`.
- [ ] Ledgers reflect reality and the completed record is archived.
