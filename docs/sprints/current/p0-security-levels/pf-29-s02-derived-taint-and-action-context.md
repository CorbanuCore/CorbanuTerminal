---
sprint_id: "PF-29-S02"
title: "Derived taint and protected-action context"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-29"
execution_order: 18
owner: "Jim Ricketts"
lane: "content"
write_scope: "codex-rs/core/src/security/taint, codex-rs/core/src/compact.rs, codex-rs/core/src/compact_remote.rs, codex-rs/core/src/memories, codex-rs/core/src/agent/control.rs, codex-rs/core/src/rollout, codex-rs/core/src/session/rollout_reconstruction.rs"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-29-S01"
created: 2026-08-27
updated: 2026-08-27
---

# PF-29-S02 — Derived taint and protected-action context

## Execution mandate

- Deliver: Preserve source taint across derivation and supply deterministic context for post-read action checks.
- Excludes: Browser isolation, classifier training, shared schema redesign, and PF-23 dispatch integration.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-29`; see the plan's adopted contract and requirement traceability.
- Acceptance advanced: Preserve source taint across derivation and supply deterministic context for post-read action checks.

## Code boundaries

- Existing (paths below `codex-rs/`): `core/src/compact{,_remote}.rs`; `core/src/memories/`; `core/src/agent/control.rs`; rollout reconstruction.
- Planned: `core/src/security/taint/` derivation, persistence, and action-context tests.
- Tests: sibling unit/integration regressions and the PF-26 fixture matrix.

## Preconditions

- [ ] Listed dependencies are completed and archived.
- [ ] Read applicable root and nested AGENTS instructions; plan remains active.
- [ ] Allocate exact owner/worktree/branch/base and literal write scope in the plan; check lane/slot conflicts.

## Done

- [x] Approved feature contract decomposed into this single-feature draft.

## Remaining

- [ ] Propagate a conservative union of source provenance through summaries, local/remote compaction, delegated output, and child contexts.
- [ ] Persist derivation metadata with memory and rollout writes; restore it on resume before protected work can start.
- [ ] Treat missing, corrupt, or unknown lineage as untrusted for protected use; generated text cannot clear its own taint.
- [ ] Supply immutable current taint and authority epoch to PF-23 action checks; a pre-fetch approval cannot authorize a changed post-fetch action.
- [ ] Test multi-hop derivation, memory replay, restart, child narrowing, and spoofed trust resets while preserving Permissive.

## Verification

- [ ] Run `just fix -p <affected-crate>` and `just fmt` in `codex-rs` before final tests.
- [ ] Run focused and affected integration tests with `just test -p <affected-crate>`; never direct `cargo test`.
- [ ] Record exact dependency commits, candidate, commands, artifacts, and missing coverage; no pass by code presence.
- [ ] Run compaction, memory, delegation, and resume integration fixtures with canary-bearing attack instructions.
- [ ] TUI applicability: actual-key ingest, summarize/delegate, restart/resume, and attempted protected-action flow.

## Exit evidence

- [ ] Final candidate and dependency commits, commands, platform results, and artifact paths recorded.
- [ ] Applicable actual-key proof is linked; no missing test is relabeled a pass.
- [ ] Ledgers reflect reality; completed record is archived and plan evidence linked.
