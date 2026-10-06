---
sprint_id: "PF-30-S02"
title: "Persistent taint across summaries and memory"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 38
owner: "untrusted-content lane"
parallel_lane: "untrusted-content"
write_scope: "codex-rs/protocol/src/provenance.rs, codex-rs/protocol/src/protocol.rs, codex-rs/core/src/security/ingress/, codex-rs/core/src/client.rs, codex-rs/core/src/compact.rs, codex-rs/core/src/codex_thread.rs, codex-rs/core/src/session/mod.rs, codex-rs/core/src/session/session.rs, codex-rs/core/src/session/rollout_reconstruction.rs, codex-rs/core/src/session/tests.rs, codex-rs/core/src/agent/control.rs, codex-rs/core/src/agent/control/spawn.rs, codex-rs/core/src/agent/control_tests.rs, codex-rs/core/src/tools/handlers/multi_agents/send_input.rs, codex-rs/core/src/memory_stage_one_tests.rs, codex-rs/core/src/client_tests.rs, codex-rs/core/src/thread_rollout_truncation.rs, codex-rs/core/tests/suite/provenance.rs, codex-rs/rollout/src/, codex-rs/state/src/extract.rs, codex-rs/state/src/runtime/threads.rs, codex-rs/thread-store/src/thread_metadata_sync.rs, codex-rs/app-server-protocol/src/protocol/thread_history.rs, codex-rs/app-server-protocol/src/protocol/thread_history_projection.rs, codex-rs/app-server-protocol/schema/, codex-rs/app-server/tests/suite/v2/thread_fork.rs, codex-rs/external-agent-migration/src/sessions/export.rs, codex-rs/memories/write/src/phase1.rs, codex-rs/ext/extension-api/src/contributors/prompt.rs, codex-rs/ext/memories/src/extension.rs, codex-rs/ext/memories/src/tests.rs, qa/security-levels/sprints/PF-30-S02/, qa/demos/specs/, qa/demos/index/PF-30-S02.md, docs/sprints/current/p0-security-levels/pf-30-s02-persistent-taint-and-memory.md"
integration_gate: "Per-sprint gate of 2026-10-06: focused tests, GLM 5.2 tmux demos, one independent Opus 5.5 High review; merge behind source_envelopes."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/pf30-s02b-20261006"
branch: "feat/pf-30-s02-slice2"
base_commit: "476f42dbe3d478f3b75b3d7016552e36632d212f"
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
- Slice 1 (PR #190): origin records in the rollout, restore on resume/fork, compaction, memory read path.
- Slice 2 (worktree above): signed records (`ingress/origin_key.rs`), agent hand-offs (`agent/control.rs`,
  `spawn.rs`, `send_input.rs`, `codex_thread.rs`), MCP token-budget hint (`session/mod.rs`).
- Tests: colocated `pf_30_s02` modules; fixtures use synthetic content and fake providers only.

## Preconditions

- [x] Active plan; PF-30-S01 completed and archived (PR #178, `b96b23344b`). Decisions of 2026-10-06 apply.

## Done

- [x] Slice 1, PR #190 (`7b2a04ea41`): origins persist (digests only), restore on resume and fork, are
  restated at compaction checkpoints; compaction and memory keep their taint. Gate in
  [persistent-origins-gate.md](../../../../qa/security-levels/sprints/PF-30-S02/persistent-origins-gate.md).
- [x] Agent lineage: a task one agent hands another (spawn or `send_input`) is never human input in the
  receiver. It is host text only while the sender's whole history had standing, otherwise
  `source=child_agent` data. A V1 child's result reaches its parent as agent data. Forks carry signed records.
- [x] Export/import: records are version 2 and carry an HMAC-SHA256 tag under a per-home key
  (`$CODEX_HOME/source-origin.key`, 0600). A session file opened in another home, an edited record, an
  unsigned version-1 record or a home without a key restores nothing, so the content is labelled. Imported
  external-agent sessions never contain records; thread-history projections drop them.
- [x] Memory summarisation bound to the session level: delivered by
  [PF-30-S04](../../archive/p0-security-levels/pf-30-s04-policy-bound-memory-dispatch.md) (stage one denies
  under Moderate/Aggressive). New test: turning on `source_envelopes` does not reopen it.
- [x] Capacity and digest-on-read: restore and journal overflow leave later content labelled and never
  upgrade; content edited after recording no longer matches its digest and is labelled.
- [x] One-off approval: approving one exact command keeps its output `source=tool` on that and later turns;
  approval notes are recorded without standing.
- [x] Real paginated / referenced-fork resume: app-server test forks a compacted paginated thread
  (reference-backed) and cold-resumes the fork; human prompts and model answers keep standing, the
  tool-derived summary stays labelled.
- [x] Token-budget MCP thread hint: with the flag, it leaves the host developer message and is sent as its own
  `source=mcp` data message.
- [x] Gate for slice 2: [slice-2-gate.md](../../../../qa/security-levels/sprints/PF-30-S02/slice-2-gate.md);
  videos in [qa/demos/index/PF-30-S02.md](../../../../qa/demos/index/PF-30-S02.md).

## Remaining

Nothing. Two items were moved to the sprints that own them:

- Positive protected memory extraction (stage one under Moderate using labelled input) is new protected
  inference, not persistence: moved to [PF-23-S01](../../current/p0-security-levels/pf-23-s01-moderate-ingress-and-disclosure-enforcement.md).
- Authority checks on tainted follow-on actions are [PF-30-S03](../../current/p0-security-levels/pf-30-s03-post-taint-authority-checks.md).

## Verification

- [x] `just fmt`; `just fix -p` for each changed crate.
- [x] Focused: `just test -p codex-core pf_30_s0`, `just test -p codex-app-server pf_30_s02`,
  `just test -p codex-external-agent-migration pf_30_s02`.
- [x] Real TUI: GLM 5.2 tmux runs and demos for each slice.
- [x] One independent Opus 5.5 High review per slice.

## Exit evidence

- [x] Gate records under `qa/security-levels/sprints/PF-30-S02/`; videos in `qa/demos/index/PF-30-S02.md`.
- [x] Both slices merged to main behind `source_envelopes`; record archived.
