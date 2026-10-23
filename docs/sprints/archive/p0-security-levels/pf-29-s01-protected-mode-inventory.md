---
sprint_id: "PF-29-S01"
title: "Protected-mode inventory and activation preflight"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-29"
execution_order: 35
owner: "first-free lane worker (codex, 2026-10-06)"
parallel_lane: "tui"
write_scope: "codex-rs/core/src/security/inventory.rs, codex-rs/core/src/security/preflight.rs, codex-rs/core/src/security/pf_29_s01_tests.rs, codex-rs/core/src/lib.rs, codex-rs/core/src/exec_env.rs, codex-rs/core/config.schema.json, codex-rs/features/src/lib.rs, codex-rs/app-server-client/src/lib.rs, codex-rs/tui/src/security/preflight.rs, codex-rs/tui/src/security/preflight_tests.rs, codex-rs/tui/src/security/mod.rs, codex-rs/tui/src/security/level.rs, codex-rs/tui/src/security/launch.rs, codex-rs/tui/src/security/aggressive.rs, codex-rs/tui/src/security/view.rs, codex-rs/tui/src/bottom_pane/security_level_picker.rs, codex-rs/tui/src/bottom_pane/security_level_picker_tests.rs, codex-rs/tui/src/bottom_pane/security_view.rs, codex-rs/tui/src/bottom_pane/security_view_tests.rs, codex-rs/tui/src/slash_command.rs, codex-rs/tui/src/lib.rs, codex-rs/tui/src/app/session_lifecycle.rs, qa/security-levels/sprints/PF-29-S01/, docs/plans/completed/main-2026-10-08-p0-security-levels.md, docs/sprints/current/p0-security-levels/index.md, docs/sprints/archive/p0-security-levels/pf-28-s02-reflected-secret-response-scrubbing.md, qa/demos/index/PF-29-S01.md, docs/sprints/archive/p0-security-levels/pf-29-s01-protected-mode-inventory.md"
integration_gate: "PR to main under the per-sprint gate (sec-common decision 5): focused tests, GLM 5.2 tmux run, one Opus 5.5 High review, SOP videos; behaviour behind the new default-off protected_mode_preflight flag (only with security_levels). Shared hunks kept small: two module lines in core/src/security/mod.rs (PF-27-S02 scope), the /security view argument in tui/src/chatwidget/slash_dispatch.rs (PF-60-S03 scope), one module re-export each in core/src/lib.rs and app-server-client/src/lib.rs, one helper in core/src/exec_env.rs, the flag in features/src/lib.rs and core/config.schema.json, the resume hooks in tui/src/lib.rs and app/session_lifecycle.rs, the merged_behind_flag note in the PF-28-S02 record and the index row; new demo specs qa/demos/specs/pf29s01-*.toml under the directory PF-23-S01 reserves (new files only)."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf-29-s01-20261006"
branch: "pf-29-s01-20261006"
base_commit: "4b42012daa8e532d7ae2f9a62b6829f55b044b0b"
depends_on: "PF-28-S02, PF-20-S02"
merged_behind_flag: "protected_mode_preflight"
gate_evidence: "qa/security-levels/sprints/PF-29-S01/README.md"
created: 2026-08-28
updated: 2026-10-08
---

# PF-29-S01 — Protected-mode inventory and activation preflight

**October 6:** merged (PR #228, `b751254169`) behind `protected_mode_preflight`; returned to `draft` for its open items.

## Execution mandate

- Deliver: Protected-mode activation cannot claim a clean boundary while a known raw-secret route or contaminated resume remains usable.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md#pf-29).
- Feature: `PF-29`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: Protected-mode activation cannot claim a clean boundary while a known raw-secret route or contaminated resume remains usable.
- Sources and archive disposition: [PF-29 reconciliation](../../../plans/security-source-reconciliation.md#pf-29).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-6](../../../plans/openclaw-source-review-2026-08-28.md#oc-6) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Core: `core/src/security/inventory.rs` (findings, classes, keyed dry-run manifest), `core/src/security/preflight.rs`
  (readiness, blockers, drift), exported as `codex_core::protected_preflight` (and `legacy_core::protected_preflight`).
- TUI: `tui/src/security/preflight.rs`, the picker, `security/launch.rs` and the resume chokepoints. Flag
  `protected_mode_preflight` (default off); without it, or without `security_levels`, nothing changes.

## Preconditions

- [x] Active plan. PF-20-S02 completed and archived. PF-28-S02 merged behind `secret_output_gate` (#216) with gate
  evidence but is not archived; started on that basis (PF-28-S01/PF-33-S02 precedent), recorded as
  `merged_behind_flag` in the PF-28-S02 record so the checker accepts it.
- [x] Read root and nearest AGENTS.md; coordinates recorded above and in the plan; both checkers pass.
- [x] Module paths as planned (`core/src/security/{inventory,preflight}.rs`). No new crate, Cargo or lockfile change.
  Protected mode today is the flagged Aggressive level (PF-24-S03); Moderate is not offered, so the preflight gates
  Aggressive.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input.
- [x] Readiness: OS sandbox, `secretless_agent_launch`, `isolated_credential_broker`, `secret_output_gate`, vault store
  (unreadable or damaged = incomplete; the vault is never opened), and every provider that signs in through an `auth`
  command (skipped, consent required = incomplete). Any gap blocks the transition.
- [x] Inventory by stable finding ID, location only, never a value: Corbanu stores (vault, sign-in, `.env`, wallet,
  sessions, history, snapshots, logs, state databases, memories), fixed `$HOME` credential files (CLI tokens, cloud,
  kube, `~/.ssh` private keys, gnupg), shell profiles (`export`/fish `set -gx`), project env files, browser profiles,
  keychains, the process environment, and every config layer including shadowed values (MCP `env`/headers/bearer,
  provider bearer tokens and hidden auth headers, `shell_environment_policy.set`, hooks, notify, writable roots).
- [x] Classes: managed/unmanaged secret, custody key, financial record, historic content, reference, ordinary env,
  launch route; limits shown with every preflight. Dry-run manifest: path, scope, symlink target, `(dev, inode)`,
  size, mode, keyed digest, supported/unsupported/unreadable. Fixed locations only; nothing written.
- [x] Transition: the Aggressive review runs the preflight; blockers (readiness gaps, profile or config literals,
  key-shaped values in innocent variables, secrets in memories) keep it unsaved. Confirm reruns it and refuses on
  drift. A pass writes `$CODEX_HOME/security_preflight.toml` (finding IDs and times only).
- [x] Isolation (covers the PF-27-S02 known limit): while Aggressive is stored after a preflight, launch denies agent
  reads of every credential file found (for example `~/.ssh/id_ed25519`, CLI tokens, browser profiles, Corbanu
  history and state) and verifies each denial before Aggressive is shown active.
- [x] Launch re-audit: a boundary that is no longer clean (for example a new profile export) is reported as not
  clean in a startup warning and `/status`, never claimed. Conversations recorded before activation cannot be resumed
  or forked (UUIDv7 creation time against the receipt); a missing or corrupt receipt refuses every resume.

## Remaining

Nothing left in this sprint: merged behind its flag and archived when Travis closed the P0 plan on 2026-10-08. These open items moved to the [P1 plan](../../../plans/active/p1-security-hardening.md#carried-forward-from-p0):

- MCP servers, hooks and notify run outside the sandbox: listed as "not contained", not blocked, pending Travis's
  PF-27-S02 decision. Claude panes are not inventoried.
- Consent flow for exec-provider sign-in commands (today: remove `auth` or block). Migration: PF-29-S02.
- Launch isolation uses the start folder; a `.env` in another `-C` folder is reported not clean, not denied.

## Verification

- [x] `just fix -p codex-features -p codex-core -p codex-app-server-client -p codex-tui`, `just fmt`; diff inspected.
- [x] Focused `pf_29_s01`: core 14, tui 8. Suites, five GLM 5.2 videos, three Opus reviews (all findings handled).
- [x] PR CI green; merged as #228. Details in the [evidence](../../../../qa/security-levels/sprints/PF-29-S01/README.md).
- Moved to P1: PF-26 final-candidate requalification (milestone gate).

## Exit evidence

- [x] Commits, commands, outcomes, videos and review under `qa/security-levels/sprints/PF-29-S01/`.
- Moved to P1: PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [x] Ledgers reflect reality; open items moved to the [P1 plan](../../../plans/active/p1-security-hardening.md#carried-forward-from-p0) and the record archived with the P0 close (Travis, 2026-10-08).
