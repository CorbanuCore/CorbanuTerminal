---
sprint_id: "PF-24-S01"
title: "Security command and profile view"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-24"
execution_order: 34
owner: "/root/security_ui"
parallel_lane: "security-view"
write_scope: "codex-rs/tui/src/security/, codex-rs/tui/src/bottom_pane/security_view.rs, codex-rs/tui/src/bottom_pane/security_view_tests.rs, codex-rs/tui/src/bottom_pane/snapshots/, codex-rs/tui/src/slash_command.rs, codex-rs/tui/src/chatwidget/slash_dispatch.rs, codex-rs/tui/src/status/, codex-rs/tui/tests/suite/security_profiles.rs, codex-rs/tui/tests/suite/mod.rs, qa/security-levels/sprints/PF-24-S01/, docs/sprints/current/p0-security-levels/pf-24-s01-security-command-and-profile-view.md"
integration_gate: "Codex /root owns shared module exports and documentation, audits observation-only requested/effective UX, reruns TUI snapshots and actual-key TMUX on RTX after formatting, and coordinates Astra High/Fable 5.1 High reviews (maximum five per lane). No protected activation or policy mutation."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/security-round5-ui"
branch: "feat/security-round5-ui"
base_commit: "07791288b6feeccfaee5a57c12452359cc666957"
depends_on: "PF-20-S02, PF-22-S02"
created: 2026-08-24
updated: 2026-09-04
---

# PF-24-S01 — Security command and profile view

## Execution mandate

- Deliver: `/security` opens a focused, keyboard-navigable view of the current level and three profiles.
- Excludes: applying changes, downgrade confirmation, temporary grants, kill switch, and release TUI qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-24`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: the user can understand current protection and profile differences without internal policy vocabulary.

## Code boundaries

- Existing: `tui/src/slash_command.rs`; `tui/src/chatwidget/slash_dispatch.rs`
- Planned: `tui/src/security/{mod,view}.rs`; `tui/src/bottom_pane/security_view.rs`
- Tests: sibling tests and `insta` snapshots under the owning modules

## Preconditions

- [ ] PF-20-S02 and PF-22-S02 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, `codex-rs/tui/AGENTS.md`, and `tui/styles.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-24.

## Remaining

- [ ] Distinguish requested level from effective readiness; unfinished/degraded protected subsystems show blocked status, not a healthy Moderate/Aggressive badge. PF-41 owns detailed runtime inspection.

- [ ] Register and route `/security` without changing `/permissions`.
- [ ] Render current level, concise protection summary, and Permissive/Moderate/Aggressive choices using existing TUI patterns.
- [ ] Support Up/Down or equivalent configured keys, Enter intent, and Esc cancel with no mutation in this slice.
- [ ] Keep narrow terminals readable and expose the level in session status.
- [ ] Add focused behavior tests and reviewed snapshots for default, each level, narrow width, and unknown-state error.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-tui`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Final-tree test: `cd codex-rs && just test -p codex-tui security_view`.
- [ ] Snapshot review: inspect every `*.snap.new`; accept only intended output.
- [ ] Actual-key TMUX: open, explore profiles, inert Enter, Esc, narrow/error states and /status on the final RTX candidate; PF-26-S02 retains final composed qualification.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and deferred key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-24-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
