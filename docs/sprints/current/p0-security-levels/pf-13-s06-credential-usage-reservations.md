---
sprint_id: "PF-13-S06"
title: "Credential usage reservations"
status: in_progress
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 22
owner: "Pauli — credential reservations lane"
parallel_lane: "credential-reservations"
write_scope: "codex-rs/security-policy/src/, codex-rs/core/src/security/credential_capability.rs, codex-rs/core/src/security/credential_capability_tests.rs, qa/security-levels/sprints/PF-13-S06/, docs/sprints/current/p0-security-levels/pf-13-s06-credential-usage-reservations.md"
integration_gate: "The Codex ingress/classifier integration lane receives PF-13-S06, verifies it extends rather than replaces BoundedGrant and the opaque capability, serializes shared exports or manifests, reruns complete security-policy and focused Core capability suites plus governance checks, records consumer handoff, and archives without activating a transport."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/p0-security-credential-reservations"
branch: "feat/p0-security-credential-reservations"
base_commit: "9d08b15fa94676c1383ee1605b77e7cc7218dcc4"
depends_on: "PF-13-S01, PF-17-S01"
created: 2026-08-28
updated: 2026-08-30
---

# PF-13-S06 — Credential usage reservations

## Execution mandate

- Deliver: Add per-request and aggregate usage accounting to the accepted opaque credential capability.
- Excludes: vault resolution, live transport injection, broker IPC implementation and TUI.

## Plan linkage

- Plan: [P0 security levels](../../../plans/active/p0-security-levels.md).
- Feature: `PF-13`.
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: [architecture refinements](../../../plans/security-architecture-refinements-2026-08-28.md); preserve the completed S01 and its historical evidence, with only added guarantees in this follow-up.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{credential,grant,bounded}.rs`; `codex-rs/core/src/security/credential_capability.rs` and sibling tests.
- Transport consumers: PF-13-S03 and PF-27; keep their implementation outside this unit.

## Preconditions

- [x] Plan active; all dependencies completed and archived.
- [x] Assign a named execution owner and exact plan-matching worktree/branch/base; reserve disjoint scopes and integration gate if parallel.
- [x] Read root and nearest implementation AGENTS.md; run the sprint checker before readiness.

## Done

- [x] Follow-up separated from the accepted upstream foundation; no new implementation or qualification claimed.
- [x] Bound request, token, byte, and spend limits plus exact model/resource/operation/destination identity to the existing `BoundedGrant` and opaque capability authority.
- [x] Added atomic worst-case reservation and trusted settlement accounting for completion, partial response, cancellation, retry, expiry/revocation, and unknown outcomes.
- [x] Preserved private unguessable capability ownership and added a non-serializable, redacted, zeroizing reservation bearer for the later authenticated broker boundary; no vault resolution or transport was activated.
- [x] Added deterministic coverage for per-request/aggregate exhaustion, concurrent over-reservation, changed authority, forged/excess metering, partial and duplicate settlement, cancellation/retry charging, revocation/expiry, and fail-closed unknown outcomes.

## Remaining

- [ ] Integration owner exports the usage schema constant on the shared policy surface, reruns the combined-lane gates, records downstream PF-13-S03/PF-27 consumer handoff, updates shared navigation/plan state, and archives this sprint without activating transport.

## Verification

- [x] `cd codex-rs && just fix -p codex-security-policy && just fix -p codex-core && just fmt` before final affected tests.
- [x] `cd codex-rs && just test -p codex-security-policy credential && just test -p codex-security-policy grant && just test -p codex-core credential_capability`.
- [x] TUI applicability: none for this accounting contract; an exact-candidate Corbanu TMUX startup/input/response smoke was still captured, while PF-13-S03/PF-27 integrate transport and PF-26-S02 proves the visible flow.
- [ ] Integration owner reruns combined-lane policy/Core/governance gates after the shared export and before archive.

## Exit evidence

- [x] Record implementation commit, changed paths, contract version and exact final-tree commands/results under `qa/security-levels/sprints/PF-13-S06/`.
- [x] Preserve S01 archive/evidence unchanged; do not relabel historical passes as proof of these new cases.
- [ ] Integration owner records the consumer handoff and shared export, completes combined ledgers, and updates plan/navigation while archiving.
