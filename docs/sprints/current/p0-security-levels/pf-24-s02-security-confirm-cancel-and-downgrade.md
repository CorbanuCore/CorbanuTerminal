---
sprint_id: "PF-24-S02"
title: "Security confirm, cancel, and downgrade"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-24"
execution_order: 43
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-23-S03, PF-24-S01, PF-29-S02"
created: 2026-08-24
updated: 2026-08-28
---

# PF-24-S02 — Security confirm, cancel, and downgrade

## Execution mandate

- Deliver: the trusted TUI confirms, cancels, persists, and reports security-level transitions.
- Excludes: temporary-grant editor, kill switch, protected-action preview, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-24`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: Esc changes nothing; confirmation applies immediately; downgrade shows removed protections first.

## Code boundaries

- Existing: `tui/src/app/config_persistence.rs`; `tui/src/bottom_pane/approval_overlay.rs`
- Planned: `tui/src/security/{confirm,events}.rs`; Core transition event wiring
- Tests: sibling tests, app event tests, and reviewed snapshots

## Preconditions

- [ ] PF-23-S03, PF-24-S01, PF-29-S02 are completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-24.

## Remaining

- [ ] Run PF-29 preflight/migration before protected activation; show newly added broker, environment, web, browser, financial and disclosure restrictions, and do not apply an incomplete transition after restart.

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
- [ ] TUI qualification deferred to PF-26-S02 with exact success/cancel/failure/restart keys.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-24-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
