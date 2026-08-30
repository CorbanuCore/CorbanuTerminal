---
sprint_id: "PF-41-S03"
title: "Durable security event and recovery foundation"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-41"
execution_order: 25
owner: "Huygens — durable events lane"
parallel_lane: "durable-events"
write_scope: "codex-rs/security-audit/, qa/security-levels/audit-foundation/, qa/security-levels/sprints/PF-41-S03/, docs/sprints/current/p0-security-levels/pf-41-s03-durable-security-event-foundation.md"
integration_gate: "The Codex ingress/classifier integration lane receives PF-41-S03, audits secret-free event and ambiguous-commit semantics, alone registers the new crate in shared Cargo/Bazel/lock surfaces, reruns full security-audit fault-injection and consumed PF-19/PF-20 suites plus governance checks, and archives without enabling a producer or protected profile."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-durable-events"
branch: "feat/p0-security-durable-events"
base_commit: "9d08b15fa94676c1383ee1605b77e7cc7218dcc4"
depends_on: "PF-19-S02, PF-20-S02"
created: 2026-08-28
updated: 2026-08-30
---

# PF-41-S03 — Durable security event and recovery foundation

## Execution mandate

- Deliver: Provide shared durable secret-free event identities and commit/failure semantics before quarantine, broker and financial consumers.
- Excludes: protected-mode activation, adjacent feature implementation and Permissive behavior changes.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md#pf-41).
- Feature: `PF-41`.
- Product citation: **Reconciled security scope — TO BUILD** — “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”
- Acceptance advanced: [accepted architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md).
- Source input: [OpenClaw source review](../../../plans/openclaw-source-review-2026-08-28.md) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; reference behavior is not candidate acceptance.

## Code boundaries

- Planned: codex-rs/security-audit/src/{event,journal,recovery}.rs; codex-rs/security-audit/tests/
- Existing integration paths are read-only until the named consumer sprint; shared manifests/lockfiles require serialized ownership.

## Preconditions

- [x] Plan active; dependencies in front matter completed and archived.
- [x] Named owner, exact worktree/branch/base and disjoint literal write scope allocated; both governance checkers passed before implementation.
- [x] Read root and nearest implementation AGENTS.md plus the exact product, active-plan, round-three and lane handoff requirements.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.
- [x] Reused PF-16–20 actor, request, mandate, receipt and revocation types; added versioned digest identities, causal parents, policy/run/owner generations and secret-free correlation records without reusable authority.
- [x] Added bounded segmented append, acknowledgment, hash-chain, protected-checkpoint and recovery contracts with an explicit record-first, protected-root-last cross-store commit protocol.
- [x] Added durable intent-before-dispatch permits and completed/unknown terminal receipts. Action/dedup retries remain idempotent across clock, session, task and generation changes. Disk full, protected-root timeout, ambiguous commit, saturation, missing keys and concurrent writers fail closed without automatic replay; duplicate errors preserve the original acknowledgement identity.
- [x] Added validated recovery tail caching and explicit exact-event operator reconciliation for one-record ambiguous commits; restart/root changes invalidate the cache, reconciliation grants no permit and the journal requires recovery plus unknown resolution before dispatch.
- [x] Emergency restriction applies the PF-19 fence before persistence; audit failure remains visible and blocks restart recovery while the PF-20 state and reconstructed ledger differ.
- [x] Covered append/crash boundaries, duplicate IDs, rollback, truncation, mutation, rotation, saturation, key loss, owner rotation, malformed storage and concurrent writers.
- [x] Published fixture-only consumer contract v1 and exact durability/ownership guarantees without registering a producer, consumer, runtime route or protected profile.

## Remaining

- [ ] Integration owner audits the literal scope, registers the crate and shared dependencies on the combined tree, reruns consumed PF-19/PF-20 and security-audit suites, updates shared navigation/plan evidence and archives the sprint without activating a consumer.

## Verification

- [x] Final Rust formatting, full-workspace-deny Clippy, 34 unit/fault tests, one public integration test, three fixture tests, 23 governance checker tests and both live governance checkers pass; exact commands and artifact hashes are recorded in evidence.
- [x] Fault tests cover before-write failure, post-sync crash, post-publish ambiguity and a real protected-root timeout, including immediate emergency fencing with unavailable audit storage. Transport acknowledgement loss is outside the in-process journal and uses the stable duplicate-retry contract.
- [x] TUI applicability remains indirect for this unregistered foundation; the exact candidate passed the round-three TMUX/Corbanu `/status` and clean-exit smoke with trace logging.
- [x] Contract fixture asserts `runtime_activation: false` with empty producer and consumer registrations; the lane changes no existing runtime route or profile.
- [ ] Complete the fresh final read-only Claude Opus 5 Plan Max review in the TMUX/Corbanu harness; all nine first-review and six second-review findings are remediated and retested.

## Exit evidence

- [ ] Record the clean independent review and final-tree digests under `qa/security-levels/sprints/PF-41-S03/`; exact-candidate TMUX smoke is complete.
- [ ] Consumers depend on this completed foundation; final PF-41-S02 and PF-26 revalidate combined-tree audit/recovery rather than accepting fake producer fixtures.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
