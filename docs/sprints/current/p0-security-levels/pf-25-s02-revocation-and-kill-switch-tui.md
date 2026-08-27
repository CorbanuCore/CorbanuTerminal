---
sprint_id: "PF-25-S02"
title: "Revocation and kill-switch TUI"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-25"
execution_order: 27
owner: "Jim Ricketts"
lane: "revoke-ui"
write_scope: "codex-rs/tui/src/security/revocation_view.rs, codex-rs/tui/src/security/revocation_tests.rs, codex-rs/tui/src/security/snapshots/revocation"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-19-S01, PF-23-S03, PF-24-S02"
created: 2026-08-24
updated: 2026-08-27
---

# PF-25-S02 — Revocation and kill-switch TUI

## Execution mandate

- Deliver: the human can revoke scoped authority or activate the kill switch and verify recovery after restart.
- Excludes: changing security levels, new protected surfaces, automatic model escalation, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-25`
- Acceptance advanced: revocation immediately dominates grants, mandates, approvals, cached decisions, and resumed work.

## Code boundaries

- Existing (read-only): `security-policy/src/revocation.rs`; PF-24-S02 overlay/events.
- Planned: `tui/src/security/{revocation_view,revocation_tests}.rs`; revocation-only snapshots; consume registered Core events.
- Tests: revocation-only behavior tests and snapshots; run existing Core recovery tests without editing shared files.

## Preconditions

- [ ] Every listed dependency is completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-25.

## Remaining

- [ ] List active secret-free grants/mandates and their exact scopes without protected values.
- [ ] Require trusted human confirmation for revoke-all, scoped revoke, and kill-switch activation.
- [ ] Use the completed Core API to apply and persist revocation before another protected operation can start; show durable active state after restart.
- [ ] Provide an explicit human recovery path that cannot silently weaken the selected level.
- [ ] Add race, cancel, persistence-failure, restart/resume, child, cached-decision, and agent-attempt tests with snapshots.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-tui && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Tests: `cd codex-rs && just test -p codex-tui security_revocation && just test -p codex-core security_recovery`.
- [ ] Snapshot review: inspect and intentionally accept only PF-25 revocation output.
- [ ] Run applicable success/cancel/failure/recovery keys in a true PTY before completion; PF-26 repeats final integrated qualification.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-25-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
