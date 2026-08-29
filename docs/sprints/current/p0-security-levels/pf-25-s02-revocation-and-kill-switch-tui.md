---
sprint_id: "PF-25-S02"
title: "Revocation and kill-switch TUI"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-25"
execution_order: 45
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-19-S02, PF-23-S03, PF-25-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-25-S02 — Revocation and kill-switch TUI

## Execution mandate

- Deliver: the human can revoke scoped authority or activate the kill switch and verify recovery after restart.
- Excludes: changing security levels, new protected surfaces, automatic model escalation, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-25`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: revocation immediately dominates grants, mandates, approvals, cached decisions, and resumed work.

## Code boundaries

- Existing: `security-policy/src/revocation.rs`; `tui/src/bottom_pane/approval_overlay.rs`
- Planned: `tui/src/security/revocation_view.rs`; Core revocation/kill events and persistence adapter
- Tests: sibling behavior tests, Core recovery tests, and reviewed snapshots

## Preconditions

- [ ] PF-19-S02, PF-23-S03, and PF-25-S01 are completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-25.

## Remaining

- [ ] Exercise immediate kill while a fake financial effect is submitted/unknown; show future authority revoked and the prior effect still uncertain. Full financial integration is repeated in PF-38-S03/PF-26, not claimed from a UI fixture.

- [ ] List active secret-free grants/mandates and their exact scopes without protected values.
- [ ] Require trusted human confirmation for revoke-all, scoped revoke, and kill-switch activation.
- [ ] Apply and persist revocation before another protected operation can start; show durable active state after restart.
- [ ] Provide an explicit human recovery path that cannot silently weaken the selected level.
- [ ] Add race, cancel, persistence-failure, restart/resume, child, cached-decision, and agent-attempt tests with snapshots.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-tui && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Tests: `cd codex-rs && just test -p codex-tui security_revocation && just test -p codex-core security_recovery`.
- [ ] Snapshot review: inspect and intentionally accept only PF-25 revocation output.
- [ ] TUI qualification deferred to PF-26-S02 with revoke/kill/restart/recovery keys.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-25-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
