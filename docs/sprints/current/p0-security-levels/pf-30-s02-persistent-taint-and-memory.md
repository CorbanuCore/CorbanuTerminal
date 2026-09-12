---
sprint_id: "PF-30-S02"
title: "Persistent taint across summaries and memory"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-30"
execution_order: 38
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-30-S01"
created: 2026-08-28
updated: 2026-09-04
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

- OpenClaw adoption reference: [OC-5](../../../plans/openclaw-source-review-2026-08-28.md#oc-5), [OC-11](../../../plans/openclaw-source-review-2026-08-28.md#oc-11) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/core/src/context_manager/{history,normalize,updates}.rs; codex-rs/memories/{read,write}/src; codex-rs/state/src/runtime/memories.rs.
- Import/export adapters: `codex-rs/external-agent-migration/src/{memory_import.rs,sessions/export.rs}`; child propagation in `codex-rs/core/src/agent/{control,registry}.rs`.
- Planned: codex-rs/core/src/security/taint.rs; codex-rs/state/src/runtime/provenance.rs.
- Tests: planned colocated Rust test modules prefixed `pf_30_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-30-S01 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Resolve the accepted round-five stage-one memory policy-binding gap using the [scoped follow-up](../../../../qa/security-levels/sprints/PF-30-S01-typed-source-envelope/memory-stage-one-follow-up.md). Decide the host-owned inherited/live binding contract before allocating worker/Core public API edits; a config-only floor is insufficient. This draft handoff grants no implementation authority or protected-memory readiness.

- [ ] Test provenance-store capacity rejection before wrapped file commit, filename/path aliases including memory/dreaming, and out-of-band changed content against read-time digests. Do not encode the outside review's disproven OpenClaw capacity exploit as an observed fact.

- [ ] Reuse the persistent writer/store/index chain, not only turn-taint-state: test canonical workspace aliases, sticky least-trusted origin, reservation-owned rollback, capacity rejection and content identity on read. Missing records, user-turn boundaries and dreaming/memory filenames must not clear protected ancestry.

- [ ] Propagate the conservative union of source authority and taint through compaction, summaries, memory write/read, retrieval, cache, export/import and transcript replay.
- [ ] Carry lineage through agent spawn, mailbox replies, task artifacts, delegation and resume; no laundering by a more privileged summarizer or a new turn.
- [ ] Version persistent envelopes and integrity digests; old/missing/corrupt provenance is untrusted or quarantined, never silently trusted.
- [ ] Keep source taint sticky across exact-action approvals. An explicitly clean context must exclude or quarantine contaminated ancestry, not relabel it; approval authorizes only the specific action and never erases source taint.
- [ ] Add multi-turn, restart, old-store migration, nested-agent and poisoned-memory tests, including a benign summary that hides its hostile source.
- [ ] Add named `pf_30_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-core pf_30_s02 && just test -p codex-state pf_30_s02`; confirm tests actually ran.
- [ ] Memory/import regression suites: `cd codex-rs && just test -p codex-memories-read && just test -p codex-memories-write`; run the affected external-agent-migration suite too.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-30-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
