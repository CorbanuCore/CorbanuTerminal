---
sprint_id: "PF-25-S01"
title: "Temporary grant TUI"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-25"
execution_order: 26
owner: "Jim Ricketts"
lane: "grant-ui"
write_scope: "codex-rs/tui/src/security/grant_view.rs, codex-rs/tui/src/security/grant_tests.rs, codex-rs/tui/src/security/snapshots/grant"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-17-S01, PF-23-S02, PF-24-S02"
created: 2026-08-24
updated: 2026-08-27
---

# PF-25-S01 — Temporary grant TUI

## Execution mandate

- Deliver: Aggressive users can inspect and confirm one narrow, expiring grant on a trusted surface.
- Excludes: kill switch, revocation management, arbitrary policy editing, financial signing adapters, and release qualification.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-25`
- Acceptance advanced: the exact actor, action, resource, destination, limits, and expiry are visible before human confirmation.

## Code boundaries

- Existing (read-only): PF-24-S02 overlay/events and `security-policy/src/grant.rs`.
- Planned: `tui/src/security/{grant_view,grant_tests}.rs`; grant-only snapshots; consume registered Core events.
- Tests: grant-only behavior tests and snapshots; run existing Core event tests without editing shared files.

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Every listed dependency is completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-25.

## Remaining

- [ ] Render canonical actor/action/resource/destination/limit/expiry fields and the adjacent access that remains denied.
- [ ] Require explicit human confirmation; prevent agent, prompt, tool, or project content from creating the event.
- [ ] Use the completed Core grant API to persist only the canonical, human-bound secret-free grant record and show active scope/expiry in `/security`.
- [ ] Support Esc cancel and visible validation/persistence errors without creating authority.
- [ ] Add mutation, adjacent-scope, child, expiry, cancel, failure, and agent-attempt tests with snapshots.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Fix: `cd codex-rs && just fix -p codex-tui && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Tests: `cd codex-rs && just test -p codex-tui security_grant && just test -p codex-core bounded_grant`.
- [ ] Snapshot review: inspect and intentionally accept only PF-25 grant output.
- [ ] Run applicable success/cancel/failure/recovery keys in a true PTY before completion; PF-26 repeats final integrated qualification.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-25-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
