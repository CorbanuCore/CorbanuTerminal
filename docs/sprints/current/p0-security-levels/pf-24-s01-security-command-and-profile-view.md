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

- [x] PF-20-S02 and PF-22-S02 are completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, `codex-rs/tui/AGENTS.md`, and `tui/styles.md`.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-24.

- [x] Distinguish requested configuration from unavailable effective protection; no healthy protected badge is manufactured. PF-41 owns authenticated detailed runtime inspection.
- [x] Register and route `/security` without changing `/permissions`.
- [x] Render requested level, concise profile summaries, and Permissive/Moderate/Aggressive exploration.
- [x] Support configured Up/Down equivalents, inert Enter inspection, and Esc cancellation without mutation.
- [x] Keep 40-column terminals readable and expose the requested/unverified state in `/status`.
- [x] Add focused behavior tests and reviewed snapshots for every profile, narrow width, and unknown-state error.

## Remaining

- [ ] Complete the whole-repository formatter after the coordinator supplies missing remote `uv`.
- [ ] Receive coordinator-numbered Astra High and Fable 5.1 High review and resolve any accepted findings.
- [ ] Integration owner completes combined-tree gates and archive; full protected activation and PF-26 qualification remain separate.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-tui`; final source passed on RTX.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Final-tree focused TUI test: 235/235 security-view/status/slash-command cases passed; exact filter and run ID in lane evidence.
- [x] Snapshot review: four new profile and 21 affected status snapshots inspected and accepted.
- [x] Actual-key TMUX: 2/2 tests passed covering three configuration levels, 120/40/80 columns, inert Enter, Esc/reopen, unknown-state startup failure and `/status`; PF-26-S02 retains final composed qualification.

## Exit evidence

- [x] Source commit `0ecb19969`, snapshots, candidate hash, changed paths and actual key script recorded.
- [x] Test output and safe captures linked under `qa/security-levels/sprints/PF-24-S01/`.
- [ ] Ledgers reflect reality and the completed record is archived.
