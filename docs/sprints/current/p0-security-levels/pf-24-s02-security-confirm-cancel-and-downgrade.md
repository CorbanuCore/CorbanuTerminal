---
sprint_id: "PF-24-S02"
title: "Security confirm, cancel, and downgrade"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-24"
execution_order: 43
owner: "tui lane worker (round 7, 2026-10-07)"
parallel_lane: "tui"
write_scope: "codex-rs/tui/src/security/{confirm,restart,launch,level,mod}.rs, codex-rs/tui/src/bottom_pane/security_{level_picker,view}.rs and tests/snapshots, codex-rs/tui/src/{app,app_event,lib,main}.rs, codex-rs/tui/src/app/event_dispatch.rs, codex-rs/tui/src/chatwidget/slash_dispatch.rs, codex-rs/cli/src/main.rs, codex-rs/core/src/security/level_change.rs (new), core/src/security/{transition,recovery,preflight,mod}.rs, codex-rs/core/src/lib.rs, codex-rs/app-server-client/src/lib.rs, codex-rs/security-level/, qa/security-levels/sprints/PF-24-S02/, qa/demos/specs/pf24s02-*.toml, qa/demos/index/PF-24-S02.md"
integration_gate: "Per-sprint gate of 2026-10-06 behind security_levels; Core's level is raised only after the protected_mode_preflight preflight passes."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-24-s02-20261007"
branch: "feat/pf-24-s02-security-confirm"
base_commit: "4f09d7af99"
depends_on: "PF-24-S03, PF-23-S03, PF-24-S01, PF-29-S02"
merged_behind_flag: "security_levels"
gate_evidence: "qa/security-levels/sprints/PF-24-S02/README.md"
created: 2026-08-24
updated: 2026-10-07
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

- [ ] PF-24-S03 (flagged picker), PF-23-S03, PF-24-S01, PF-29-S02 are completed and archived.
- [ ] Read root, Rust, Core, TUI, and TUI style instructions.
- [ ] Exact worktree coordinates match the active plan.

## Done

Gate: [qa/security-levels/sprints/PF-24-S02/README.md](../../../../qa/security-levels/sprints/PF-24-S02/README.md).

- [x] Sprint record is linked only to PF-24.
- [x] PF-29 preflight before protected activation: Core's level is raised only after the preflight passed and was
  rechecked on Enter. The preflight now also requires `source_envelopes` (Core's protected levels refuse
  unlabelled external content). A save whose Core step did not finish is reported at the next start; nothing
  weaker applies.
- [x] Differences first: the Aggressive review shows each row now and under Aggressive, the controls that turn on,
  and what Core's level does now; the downgrade review lists every protection removed at the next start (red
  heading) and Core's timing, and warns when a project, profile, `-c` or managed layer keeps a stricter level.
- [x] One typed human-origin event: the review's accept key builds one `TransitionRequest`
  (`tui/src/security/confirm.rs`), committed on its own thread through `codex_core::security_level_change` into
  PF-23-S03's `commit_transition`. No `Op`, app-server method or tool reaches it.
- [x] Enter commits and shows success or an actionable failure; Esc changes nothing. A stricter Core level applies
  to this process's sessions now; a downgrade saves now and applies at the next start (its revocation applies now).
- [x] A failed save restores the previous files, keeps the view open (Enter reviews again, Esc goes back) and shows
  the level only after the commit. A state changed elsewhere since the review is refused with "review it again".
- [x] Regressions with snapshots: confirm, cancel, downgrade, write failure, changed state, threaded save, unknown
  Core state (repaired by a confirmed level), agent rewrite of the level file, restart.
- [x] PF-24-S03 follow-ups: tamper check (a level file weaker than Core's confirmed record reads as invalid, so
  Aggressive stays enforced, also for nested-launch checks); "restart now" (`r` after a save; same program,
  arguments and folder, minus an initial prompt); a second Permissive launch no longer removes the rule file or
  registry entry while an Aggressive process of the same home holds `security_level.lock`.
- [x] PF-23-S03 hand-offs: the commit runs off the async runtime; `security_level.toml` and `security_state.json`
  are reconciled (above); `StoredLevelChanged`/`StoredStateChanged` lead back to a fresh review; an unreadable
  state is repaired by a confirmed level and its session kill switch ends with "restart now"; a downgrade
  rewrites `config.toml` only when it sets a stricter level.

## Remaining

- [ ] GLM 5.2 tmux run and model videos: Z.AI has no balance; the gate used model-free runs (no model turn).
- [ ] Merge, then archive this record.
- Known limits are in the gate record.

## Verification

- [x] `just fix -p codex-tui -p codex-core -p codex-security-level -p codex-cli`; `just fmt`.
- [x] `just test -p codex-tui` (security, pf_24, pf_29 sets), `just test -p codex-core -p codex-security-level`
  (`security_transition`, `security_recovery`, `security_confirm`, `pf_29`).
- [x] Snapshots reviewed and accepted (PF-24 output only).
- [x] Linux clippy (`-D warnings`) on the RTX box.
- [ ] TUI qualification deferred to PF-26-S02 (keys: Enter confirm, Esc cancel, `r` restart, Enter retry).

## Exit evidence

- [x] Commit, snapshots, changed paths and key script in the gate record.
- [x] Test output under `qa/security-levels/sprints/PF-24-S02/`.
- [ ] Ledgers reflect reality and the completed record is archived.
