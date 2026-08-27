---
sprint_id: "PF-24-S02"
title: "Security confirm, cancel, and downgrade"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-24"
execution_order: 25
owner: "Jim Ricketts"
lane: "inspector"
write_scope: "codex-rs/tui/src/app/config_persistence.rs, codex-rs/tui/src/bottom_pane/approval_overlay.rs, codex-rs/tui/src/security/confirm.rs, codex-rs/tui/src/security/events.rs, codex-rs/tui/src/security/mod.rs, codex-rs/core/src/security/ui_events.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-23-S02, PF-23-S03, PF-24-S01"
created: 2026-08-24
updated: 2026-08-27
---

# PF-24-S02 — Security confirm, cancel, and downgrade

## Execution mandate

- Deliver: the trusted TUI confirms, cancels, persists, and reports security-level transitions.
- Excludes: temporary-grant editor, kill switch, protected-action preview, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-24`
- Acceptance advanced: Esc changes nothing; confirmation applies immediately; downgrade shows removed protections first.

## Code boundaries

- Existing: `tui/src/app/config_persistence.rs`; `tui/src/bottom_pane/approval_overlay.rs`
- Planned: `tui/src/security/{confirm,events,mod}.rs`; `core/src/security/ui_events.rs`; shared grant/revoke view registrations.
- Tests: sibling tests, app event tests, and reviewed snapshots

## Preconditions

- [ ] Every listed dependency is completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

- [ ] Allocate lane/worktree/base in the plan and validate disjoint write scopes before readiness.

## Done

- [x] Sprint record is linked only to PF-24.

## Remaining

- [ ] Land shared overlay/module registrations and trusted grant/revoke event adapters to completed Core APIs; PF-25 sprints own separate view implementations only.
- [ ] Preserve unavailable placeholders until each view lands; do not expose an unimplemented action or accept self-declared human provenance.

- [ ] Show exact profile differences before confirmation and a protection-removal warning for downgrades.
- [ ] Send one typed human-origin transition event; expose no model/tool route to the event.
- [ ] On Enter, call the Core transition API and show success or actionable failure; on Esc, restore the prior state.
- [ ] Keep the view open/recoverable after persistence failure and reflect the effective level only after commit.
- [ ] Add confirm, cancel, downgrade, write-failure, restart, unknown-state, and agent-attempt regressions with snapshots.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-tui && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Tests: `cd codex-rs && just test -p codex-tui security_confirm && just test -p codex-core security_transition`.
- [ ] Snapshot review: inspect and intentionally accept only PF-24 output.
- [ ] Run applicable success/cancel/failure/recovery keys in a true PTY before completion; PF-26 repeats final integrated qualification.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-24-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
