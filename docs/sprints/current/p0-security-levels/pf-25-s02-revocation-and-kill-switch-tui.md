---
sprint_id: "PF-25-S02"
title: "Revocation and kill-switch TUI"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-25"
execution_order: 45
owner: "tui lane worker (round 8, 2026-10-07)"
parallel_lane: "tui"
write_scope: "codex-rs/core/src/security/revocation_change.rs, codex-rs/core/src/security/revocation_change_tests.rs, codex-rs/core/src/security/level_change.rs, codex-rs/core/src/security/aggressive.rs, codex-rs/core/src/security/mod.rs, codex-rs/core/src/lib.rs, codex-rs/app-server-client/src/lib.rs, codex-rs/tui/src/security/revocation_view.rs, codex-rs/tui/src/security/revocation_view_tests.rs, codex-rs/tui/src/security/mod.rs, codex-rs/tui/src/security/snapshots/, codex-rs/tui/src/bottom_pane/security_view.rs, codex-rs/tui/src/bottom_pane/security_level_picker.rs, qa/security-levels/sprints/PF-25-S02/, qa/demos/specs/, qa/demos/index/PF-25-S02.md"
integration_gate: "Per-sprint gate of 2026-10-06 behind security_levels; the view is offered when Core enforces a protected level, the kill switch is on, or grants are held."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-25-s02-20261007"
branch: "feat/pf-25-s02-revocation-tui"
base_commit: "c5fa36b31f"
depends_on: "PF-19-S02, PF-23-S03, PF-25-S01"
merged_behind_flag: "security_levels"
gate_evidence: "qa/security-levels/sprints/PF-25-S02/README.md"
created: 2026-08-24
updated: 2026-10-07
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

- [x] PF-19-S02, PF-23-S03 archived; PF-25-S01 (grant TUI) is this branch's base.
- [x] Read root, Rust, Core, TUI, and TUI style instructions.
- [x] Worktree coordinates above.

## Done

Gate: [qa/security-levels/sprints/PF-25-S02/README.md](../../../../qa/security-levels/sprints/PF-25-S02/README.md).

- [x] Sprint record is linked only to PF-25.
- [x] `/security`, `g` (offered when Core enforces a protected level, the kill switch is on, or grants are held)
  opens "Grants and kill switch" (`tui/src/security/revocation_view.rs`): Core's level, the kill switch, each grant
  held now (command, runs left, expiry; no protected values), "Revoke all active authority", and the kill switch.
- [x] Every choice has a review; only Enter there commits, off the UI thread. Esc changes nothing. Turning the kill
  switch off (the one choice that removes protection) opens on "Back" and needs an arrow key first.
- [x] One grant ends now (`security_revocation::revoke_grant`; grants are memory-only). Revoke all and the kill
  switch go through PF-23-S03's transition (`core/src/security/revocation_change.rs`): they apply now to this
  session's policy tree and the others of the process on the home (grants, "for session" approvals, broker
  channels end) and are saved in `security_state.json`, so they hold after a restart; `r` restarts to check.
  Only the kill switch the person saw can be released, and releasing keeps the level.
- [x] A state changed since the review is refused with "review it again"; a save failure is reported.
- [x] Tests: Core (kill switch now, other trees, saved, next start, release keeps the level, release of an off switch
  refused, no session, revoke all ends grants and moves the epoch, changed state refused, one grant); TUI (list and
  review snapshots, Esc, on then off with the Back guard, revoke all, changed state, restart key, scan that only
  this view calls Core's revocation entry points).

## Remaining

- [ ] Gate: Opus review, Linux clippy, GLM videos, merge, archive.
- Not here: a fake financial effect under the kill switch (PF-38-S03/PF-26); mandates (none are issued yet, so none
  are listed); scoped revocation of one agent (Core supports `Actor`, no UI yet).

## Verification

- [x] `just fix -p codex-core -p codex-tui -p codex-app-server-client`; `just fmt`.
- [x] `just test -p codex-core` (`security_revocation`, `transition`, `level_change`, `grant_offer`);
  `just test -p codex-tui` (`pf_25_s02`, `security`).
- [x] Snapshots reviewed and accepted (PF-25 revocation output only).
- [ ] GLM 5.2 tmux run and videos with revoke/kill/restart/recovery keys; Linux clippy on the RTX box.

## Exit evidence

- [ ] Commit, snapshots, changed paths, and key script recorded.
- [ ] Test output linked under `qa/security-levels/sprints/PF-25-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
