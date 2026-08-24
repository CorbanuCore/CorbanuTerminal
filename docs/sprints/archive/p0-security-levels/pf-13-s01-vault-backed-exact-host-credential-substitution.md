---
sprint_id: "PF-13-S01"
title: "Typed credential capability and bounded store"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 9
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-16-S01, PF-17-S01, PF-19-S01, PF-22-S01"
created: 2026-08-24
updated: 2026-08-24
---

# PF-13-S01 — Typed credential capability and bounded store

## Execution mandate

- Deliver: one Core-owned, secret-free capability request and bounded lifecycle store.
- Excludes: vault resolution, proxy injection, provider behavior, legacy helper gating, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Product citation: **Required trust boundaries** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: an opaque capability binds exact authority before any secret is resolved.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{authorization,grant,revocation,mandate}.rs`
- Planned: `codex-rs/security-policy/src/credential.rs`; `codex-rs/core/src/security/credential_capability.rs`
- Build/tests: affected Cargo/Bazel files; planned sibling `credential*_tests.rs`

## Preconditions

- [x] PF-16-S01, PF-17-S01, PF-19-S01, and PF-22-S01 are completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-13.
- [x] Policy contract shipped in `0f62930018`; the bounded Core store shipped in `61afdd805c`.
- [x] Final-tree evidence was committed in `d18c318f22`.

## Remaining

- [x] Reuse `ActorChain`, `AuthorizationRequest`, `BoundedGrant`, `RevocationState`, and `ActionReceipt`; do not create parallel policy types.
- [x] Define safe metadata for human, agent/session, task, purpose, operation, HTTP method, normalized destination, vault label/scope, issue/expiry, and revocation generation.
- [x] Keep an unguessable non-serializable `CapabilityToken` inside trusted runtime state; expose only a separate digest `CapabilityId` in decisions/receipts and no secret-returning API.
- [x] Implement a hard-bounded concurrent store with expiry/revocation cleanup and fail-closed clock, poison, capacity, unknown-field, and malformed-input behavior.
- [x] Add full-object, concurrency, forgery, expiry, revocation, wrong-actor, wrong-purpose, wrong-operation, wrong-method, wrong-host, and broader-scope tests.
- [x] Update Cargo, lock, Bazel, and module exports together.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-security-policy && just fix -p codex-core`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused tests: `cd codex-rs && just test -p codex-security-policy credential && just test -p codex-core credential_capability`.
- [x] Dependency parity from repository root: `just bazel-lock-update` when manifests change.
- [x] TUI applicability: none; PF-26-S02 owns true-TUI qualification.

## Exit evidence

- [x] Implementation commit, changed paths, and public API recorded.
- [x] Final-tree output linked under `qa/security-levels/sprints/PF-13-S01/`.
- [x] Secret-surface review confirms metadata cannot carry a credential value.
- [x] Ledgers reflect reality and the completed record is archived.
