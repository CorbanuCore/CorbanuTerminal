---
sprint_id: "PF-24-S01"
title: "Security command and profile view"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-24"
execution_order: 24
owner: "Jim Ricketts"
lane: "inspector"
write_scope: "codex-rs/tui/src/slash_command.rs, codex-rs/tui/src/chatwidget/slash_dispatch.rs, codex-rs/tui/src/security/view.rs, codex-rs/tui/src/bottom_pane/security_view.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-20-S01, PF-22-S01, PF-27-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-24-S01 — Security command and profile view

## Execution mandate

- Deliver: `/security` opens a focused, keyboard-navigable view of the current level and three profiles.
- Excludes: applying changes, downgrade confirmation, temporary grants, kill switch, and release TUI qualification.

## Plan linkage

- Upstream: [plan touch record](../../../plans/active/p0-security-levels.md#upstream-touch-record); resolve this sprint's adapter rows.
- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-24`
- Acceptance advanced: the user can understand current protection and profile differences without internal policy vocabulary.

## Code boundaries

- Existing: `tui/src/slash_command.rs`; `tui/src/chatwidget/slash_dispatch.rs`
- Planned: `tui/src/security/view.rs` (module registration owned by PF-27); `tui/src/bottom_pane/security_view.rs`
- Tests: sibling tests and `insta` snapshots under the owning modules

## Preconditions

- [ ] Plan upstream baseline, adapter ownership, and exact contract tests are resolved before readiness.
- [ ] Every listed dependency is completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, `codex-rs/tui/AGENTS.md`, and `tui/styles.md`.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-24.

## Remaining

- [ ] Consume PF-27 read-only snapshots: requested/effective level, backend, network posture, active authority, recent denials, tainted turns, and degraded controls.
- [ ] Show Browser Isolation and External Content Firewall health separately; unknown/unavailable controls cannot appear active or selectable as qualified.

- [ ] Register and route `/security` without changing `/permissions`.
- [ ] Render current level, concise protection summary, and Permissive/Moderate/Aggressive choices using existing TUI patterns.
- [ ] Support Up/Down or equivalent configured keys, Enter intent, and Esc cancel with no mutation in this slice.
- [ ] Keep narrow terminals readable and expose the level in session status.
- [ ] Add focused behavior tests and reviewed snapshots for default, each level, narrow width, and unknown-state error.

## Verification

- [ ] Record applicable upstream adapter evidence or justified non-applicability; structural checks alone are not qualification.
- [ ] Fix: `cd codex-rs && just fix -p codex-tui`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Final-tree test: `cd codex-rs && just test -p codex-tui security_view`.
- [ ] Snapshot review: inspect every `*.snap.new`; accept only intended output.
- [ ] Run applicable success/cancel/failure/recovery keys in a true PTY before completion; PF-26 repeats final integrated qualification.

## Exit evidence

- [ ] Commit, snapshots, changed paths, actual keys, and visible checkpoints recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-24-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
