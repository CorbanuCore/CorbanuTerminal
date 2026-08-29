---
sprint_id: "PF-41-S03"
title: "Durable security event and recovery foundation"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-41"
execution_order: 25
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-19-S02, PF-20-S02"
created: 2026-08-28
updated: 2026-08-28
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

- [ ] Plan active; dependencies in front matter completed and archived.
- [ ] Assign a named execution owner and exact plan-matching worktree/branch/base; run the sprint checker before readiness.
- [ ] Read root and nearest implementation AGENTS.md; reserve disjoint write scope and receiving integration gate if parallel.

## Done

- [x] Bounded preparation/foundation mandate created from the accepted review; no implementation or platform acceptance claimed.

## Remaining

- [ ] Reuse PF-16–20 actor, request, mandate and revocation types. Define versioned event/decision/action/reservation IDs, causal parents, policy/run generation, producer ownership and deduplication; no raw content, secrets or reusable capabilities.
- [ ] Implement bounded append/acknowledgment, integrity-chain/checkpoint and recovery interfaces with a reference durable store; separate physical stores are allowed with explicit cross-store commit/recovery protocol, not assumed distributed atomicity.
- [ ] Define reservation/intent-before-dispatch and completion/unknown receipts; disk-full, timeout, failed acknowledgment and ambiguous commit cannot become silent success or trigger replay. Required record failure blocks new protected dispatch.
- [ ] Emergency kill/restriction must still fence new dispatch immediately when audit persistence fails; retain fail-closed startup/recovery state and expose the gap. Never wait for uncertain financial settlement or claim an unrecorded stop survived restart without evidence.
- [ ] Test append/crash boundaries, duplicate IDs, rollback/truncation/rotation, queue saturation, missing keys and concurrent writers. Protect integrity roots through PF-20's controller-owned store; a local hash chain alone is not host-compromise resistance.
- [ ] Publish consumer contract fixtures and exact durability/ownership guarantees; broker, quarantine, financial and Sweep sprints test their real adapters, while PF-41-S02 retains joined inspection/export and end-to-end chain checks.

## Verification

- [ ] Run affected format/fix tools before final tests; record exact commands and actual test counts.
- [ ] Run planned security-audit unit/integration fault-injection tests; record crash points and restart outcomes, including immediate stop with unavailable audit storage.
- [ ] TUI applicability: none for this pure preparation/foundation boundary; user-facing consumer sprints retain true-TUI proof.
- [ ] Verify no runtime route or profile becomes available from fixture-only preparation.

## Exit evidence

- [ ] Commit, contract/fixture versions, owner review and final-tree outputs under `qa/security-levels/sprints/PF-41-S03/`.
- [ ] Consumers depend on this completed foundation; final PF-41-S02 and PF-26 revalidate combined-tree audit/recovery rather than accepting fake producer fixtures.
- [ ] Record integration handoff and scope audit; complete all ledgers before archive and update plan/navigation.
