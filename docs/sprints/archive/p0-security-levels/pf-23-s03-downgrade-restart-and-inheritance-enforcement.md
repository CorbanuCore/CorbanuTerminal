---
sprint_id: "PF-23-S03"
title: "Downgrade, restart, and inheritance enforcement"
status: completed
plan_file: "docs/plans/completed/main-2026-10-08-p0-security-levels.md"
plan_feature: "PF-23"
execution_order: 42
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/core/src/security/transition.rs, codex-rs/core/src/security/transition_tests.rs, codex-rs/core/src/security/recovery.rs, codex-rs/core/src/security/recovery_tests.rs, codex-rs/core/src/security/effective_policy.rs, codex-rs/core/src/security/trusted_requests.rs, codex-rs/core/src/security/aggressive.rs, codex-rs/core/src/agent/control.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/handlers.rs, codex-rs/core/src/session/turn_context.rs, codex-rs/core/src/state/service.rs, codex-rs/core/src/tools/sandboxing.rs, codex-rs/core/src/tools/network_approval.rs, codex-rs/core/src/mcp_tool_call.rs, codex-rs/core/src/memory_stage_one.rs, codex-rs/memories/write/src/phase1.rs, codex-rs/security-policy/src/revocation.rs, codex-rs/protocol/src/protocol.rs, codex-rs/network-proxy/src/proxy.rs, qa/security-levels/sprints/PF-23-S03/, qa/demos/index/PF-23-S03.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review, Linux clippy on the RTX box. Not reserved: codex-rs/core/src/config/mod.rs."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/sec-pf23s03"
branch: "feat/pf-23-s03-memory"
base_commit: "6b1c8b8873b7c35ba5f030f56bcf519c1948e216"
depends_on: "PF-19-S02, PF-20-S02, PF-23-S02"
created: 2026-08-24
updated: 2026-10-07
---

# PF-23-S03 — Downgrade, restart, and inheritance enforcement

## Execution mandate

- Deliver: confirmed level changes atomically invalidate incompatible authority and survive restart without a weaker interval.
- Excludes: selection TUI, grant/kill-switch TUI, content classifiers, and release qualification.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/completed/main-2026-10-08-p0-security-levels.md)
- Feature: `PF-23`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **P0 `/security` levels** — “Existing approval, sandbox, vault, wallet, tool, network, and agent policies are unchanged.”
- Acceptance advanced: no old grant, mandate, cached decision, child state, or pending approval can be replayed after change/restart.

## Code boundaries

- `core/src/security/transition.rs`: prepare/commit/cancel for confirmed level, revocation and kill-switch requests.
- `core/src/security/recovery.rs`: `security_state.json` (level floor + revocations), recovery at every start, the
  locked merge store.
- Consumers: `agent/control.rs` (recovered init), `config/mod.rs` (layer floor), `session/` (sink, run end, turn
  level), `state/service.rs` + `tools/sandboxing.rs` + `mcp_tool_call.rs` (approval-cache fence),
  `memory_stage_one.rs` + `memories/write` (past-session level).

## Preconditions

- [x] PF-23-S02 archived; PF-19-S02/PF-20-S02 archived earlier. Root, Rust and Core instructions read.

## Done

Merged 2026-10-07: #246 (recovery), #247 (transitions), #248 (fan-out), #249 (memory). Gate:
[qa/security-levels/sprints/PF-23-S03/gate.md](../../../../qa/security-levels/sprints/PF-23-S03/gate.md).

- [x] Prepare/commit/cancel: a confirmed human request is bound to the policy epoch; stricter protected levels need
  the PF-29 probes (`ProbeOutcome`); grant requests are refused; cancel changes nothing.
- [x] Commit merges into the stored state under a file lock (union of revocations; a stored stricter level is never
  lowered without a confirmed downgrade), saves, then advances epoch and revocation generation: grants, pending
  post-taint approvals, "for session" approval caches and child snapshots are invalidated; revoking an actor
  stops its agents. Restrictive changes and the kill switch apply now, also when the save fails (reported);
  downgrades apply at the next start; a kill-switch release is only for the switch the session shows.
- [x] Recovery: at config load and every session start the stricter of the configured and stored level applies with
  the stored revocations; unreadable state enforces Aggressive and the kill switch with a warning. Each config
  layer's level is a floor (a repository cannot lower it).
- [x] Broker revocation trigger (from PF-27-S04): restrictive commits, the kill switch and run end call
  `revoke_brokered_credentials` for every session of the tree (and other trees of the process on the same home).
- [x] The level a past session ran under (from PF-23-S01): recorded per turn; memory labels such sessions and never
  summarizes one that ran under Aggressive.
- [x] Merging step and summaries under Aggressive (from PF-23-S01): kept off (stage one denies, consolidation is
  skipped above Permissive); now also off for sessions that ran under Aggressive. Product decision pending
  (gate record).
- [x] GLM 5.2 tmux functional run and videos (2026-10-07, Z.AI balance cleared): four demos in
  `qa/demos/index/PF-23-S03.md`, each under 90 s with clean leak scans; focused tests re-run on candidate
  `a230f2082141` (`security_transition security_recovery` 26/26, `revocation` 11/11).

## Remaining

- Handed to PF-24-S02 (first production caller of `commit_transition`): call it off the async runtime (up to 2 s
  lock wait); reconcile `security_level.toml` with `security_state.json`; a way out after `StoredLevelChanged` /
  `StoredStateChanged` and for `UnreadableState`'s session-long kill switch; warn when a project or managed layer
  sets a level above a requested downgrade. Known limits are in the gate record.

## Verification

- [x] `just fix -p codex-core`, `just fmt`; `just test -p codex-core security_transition security_recovery` and
  the wider affected sets per PR; full `just test` of core and touched crates (gate record).
- [x] Linux clippy (`-D warnings`) on the RTX box; Opus 5.5 High review (three rounds, APPROVE).
- [x] GLM 5.2 tmux functional run and videos (Z.AI balance cleared 2026-10-07).
- [x] Full isolated code-blind VM run and human sign-off are milestone gates only (not required per sprint; deferred to the Aggressive/Moderate ship and flag removal).

## Exit evidence

- [x] Videos (four demos, `qa/demos/index/PF-23-S03.md`); state diagram and changed paths in the gate record; record archived.
