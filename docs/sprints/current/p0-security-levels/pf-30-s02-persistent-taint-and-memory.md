---
sprint_id: "PF-30-S02"
title: "Persistent taint across summaries and memory"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 38
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/protocol/src/provenance.rs, codex-rs/protocol/src/protocol.rs, codex-rs/core/src/security/ingress/, codex-rs/core/src/client.rs, codex-rs/core/src/compact.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/rollout_reconstruction.rs, codex-rs/core/src/session/tests.rs, codex-rs/core/src/agent/control/spawn.rs, codex-rs/core/src/agent/control_tests.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/thread_rollout_truncation.rs, codex-rs/core/tests/suite/provenance.rs, codex-rs/rollout/src/, codex-rs/state/src/extract.rs, codex-rs/state/src/runtime/threads.rs, codex-rs/thread-store/src/thread_metadata_sync.rs, codex-rs/app-server-protocol/src/protocol/thread_history.rs, codex-rs/app-server-protocol/src/protocol/thread_history_projection.rs, codex-rs/app-server-protocol/schema/, codex-rs/memories/write/src/phase1.rs, codex-rs/ext/extension-api/src/contributors/prompt.rs, codex-rs/ext/memories/src/extension.rs, codex-rs/ext/memories/src/tests.rs, qa/security-levels/sprints/PF-30-S02/, qa/demos/specs/, qa/demos/index/PF-30-S02.md, docs/sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s02-20261006"
branch: "feat/pf-30-s02-persistent-taint"
base_commit: "b96b23344ba68e8a484b5e68834b6a62e392e007"
depends_on: "PF-30-S01"
created: 2026-08-28
updated: 2026-10-06
---

# PF-30-S02 — Persistent taint across summaries and memory

## Execution mandate

- Deliver: Taint and provenance survive every memory/summary/agent hop rather than resetting at turn completion.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-30).
- Feature: `PF-30`.
- Product citation: **Non-negotiable controls** — “Classify instruction intent and provenance before external content can influence tools or financial actions.”
- Acceptance advanced: Taint and provenance survive every memory/summary/agent hop rather than resetting at turn completion.
- Sources and archive disposition: [PF-30 reconciliation](../../../plans/security-source-reconciliation.md#pf-30).

## Code boundaries

- OpenClaw adoption reference: [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference tests are not candidate evidence.
- This slice: origin records in the rollout (`protocol/src/provenance.rs`, `RolloutItem`), restore on
  resume/fork (`core/src/security/ingress/`, `session/mod.rs`), compaction (`compact.rs`) and the memory
  read path (`ext/memories`, `ext/extension-api`). Every exhaustive `RolloutItem` match is in `write_scope`.
- Tests: colocated `pf_30_s02` modules; fixtures use synthetic content and fake providers only.

## Preconditions

- [x] Active plan; PF-30-S01 completed and archived (PR #178, `b96b23344b`). Read root and `codex-rs`
  AGENTS.md; worktree registered in the plan front matter. Decisions of 2026-10-06 apply (flagged merge).

## Done

- [x] Allocated 2026-10-06 to the untrusted-content lane after PF-30-S01 closed.
- [x] Origins persist in the rollout (`source_origin`, version 1, digests only) and are restored on resume and
  fork, and restated at compaction checkpoints. Missing, old, unknown-version or malformed records stay untrusted;
  a record never upgrades a live registration.
- [x] Compaction: retained human messages keep human standing; the summary is host only when every input had
  standing. Memory context reaches protected requests as `source=memory`. Window-reset context registered as host.
- [x] Flag-off Moderate names the disabled feature (separate commit `836ec41e04`).
- [x] Gate at `01a3a88330`: [persistent-origins-gate.md](../../../../qa/security-levels/sprints/PF-30-S02/persistent-origins-gate.md);
  videos in [qa/demos/index/PF-30-S02.md](../../../../qa/demos/index/PF-30-S02.md).

## Remaining

- [ ] Later slices: agent spawn/mailbox lineage, export/import, memory stage-one policy binding
  ([follow-up](../../../../qa/security-levels/sprints/PF-30-S01-typed-source-envelope/memory-stage-one-follow-up.md)),
  provenance-store capacity and read-time digest tests, sticky taint across exact-action approvals,
  a real paginated/referenced-fork resume test, and the token-budget MCP thread hint (still host context).

## Verification

- [x] `just fmt`; `just fix -p` for each changed crate.
- [x] Focused: `just test -p codex-core pf_30_s0` (70 pass), plus the protocol, rollout, state and memories suites.
- [x] Integration: nine affected crates, 5,052/5,056; the four failures are recorded as baseline or flake.
- [x] Real TUI: GLM 5.2 tmux run and four demos (resume, memory, compaction, flag-off message).
- [x] One independent Opus 5.5 High review: round 1 changes requested, round 2 APPROVE.
- [ ] Remaining slices get the same gate; milestone VM run and sign-off when Moderate ships.

## Exit evidence

- [x] Gate record under `qa/security-levels/sprints/PF-30-S02/`; videos in `qa/demos/index/PF-30-S02.md`.
- [ ] This slice merged to main behind `source_envelopes`; later slices merged and the record archived.
