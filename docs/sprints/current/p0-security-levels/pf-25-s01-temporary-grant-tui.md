---
sprint_id: "PF-25-S01"
title: "Temporary grant TUI"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-25"
execution_order: 19
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-17-S01, PF-23-S02, PF-24-S02"
created: 2026-08-24
updated: 2026-08-24
---

# PF-25-S01 — Temporary grant TUI

## Execution mandate

- Deliver: Aggressive users can inspect and confirm one narrow, expiring grant on a trusted surface.
- Excludes: kill switch, revocation management, arbitrary policy editing, financial signing adapters, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-25`
- Acceptance advanced: the exact actor, action, resource, destination, limits, and expiry are visible before human confirmation.

## Code boundaries

- Existing: `tui/src/bottom_pane/approval_overlay.rs`; `security-policy/src/grant.rs`
- Planned: `tui/src/security/grant_view.rs`; typed Core grant event
- Tests: sibling behavior tests, Core event tests, and reviewed snapshots

## Preconditions

- [ ] PF-17-S01, PF-23-S02, and PF-24-S02 are completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-25.

## Remaining

- [ ] Render canonical actor/action/resource/destination/limit/expiry fields and the adjacent access that remains denied.
- [ ] Require explicit human confirmation; prevent agent, prompt, tool, or project content from creating the event.
- [ ] Persist only the signed secret-free grant record and show active scope/expiry in `/security`.
- [ ] Support Esc cancel and visible validation/persistence errors without creating authority.
- [ ] Add mutation, adjacent-scope, child, expiry, cancel, failure, and agent-attempt tests with snapshots.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-tui && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Tests: `cd codex-rs && just test -p codex-tui security_grant && just test -p codex-core bounded_grant`.
- [ ] Snapshot review: inspect and intentionally accept only PF-25 grant output.
- [ ] TUI qualification deferred to PF-26-S02 with grant/cancel/expiry keys.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-25-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
