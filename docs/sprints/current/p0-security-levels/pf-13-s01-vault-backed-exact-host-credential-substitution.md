---
sprint_id: "PF-13-S01"
title: "Typed credential capability and bounded store"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 16
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-16-S01, PF-17-S01, PF-19-S01, PF-22-S01"
created: 2026-08-24
updated: 2026-08-28
---

# PF-13-S01 — Typed credential capability and bounded store

## Execution mandate

- Deliver: one Core-owned, secret-free capability request and bounded lifecycle store.
- Excludes: vault resolution, proxy injection, provider behavior, legacy helper gating, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Reconciliation: [source decisions and archive mapping](../../../plans/security-source-reconciliation.md).
- Product citation: **Required trust boundaries** — “Credentials are referenced by label and resolved only inside a trusted execution boundary.”
- Acceptance advanced: an opaque capability binds exact authority before any secret is resolved.

## Code boundaries

- Existing: `codex-rs/security-policy/src/{authorization,grant,revocation,mandate}.rs`
- Planned: `codex-rs/security-policy/src/credential.rs`; `codex-rs/core/src/security/credential_capability.rs`
- Build/tests: affected Cargo/Bazel files; planned sibling `credential*_tests.rs`

## Preconditions

- [ ] PF-16-S01, PF-17-S01, PF-19-S01, and PF-22-S01 are completed and archived.
- [ ] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-13.

## Remaining

- [ ] Bind per-request and aggregate use/count/token/byte/spend limits to the existing BoundedGrant reservation and operation identity; unknown authoritative usage cannot silently become unlimited.

- [ ] Reuse `ActorChain`, `AuthorizationRequest`, `BoundedGrant`, `RevocationState`, and `ActionReceipt`; do not create parallel policy types.
- [ ] Define safe metadata for human, agent/session, task, purpose, operation, HTTP method, normalized destination, vault label/scope, issue/expiry, and revocation generation.
- [ ] Keep an unguessable `CapabilityToken` inside trusted runtime state, with no model/public serialization; PF-27 may transport bounded authority only over authenticated broker IPC. Expose only a digest `CapabilityId` in decisions/receipts, never a secret-returning API.
- [ ] Implement a hard-bounded concurrent store with expiry/revocation cleanup and fail-closed clock, poison, capacity, unknown-field, and malformed-input behavior.
- [ ] Add full-object, concurrency, forgery, expiry, revocation, wrong-actor, wrong-purpose, wrong-operation, wrong-method, wrong-host, and broader-scope tests.
- [ ] Update Cargo, lock, Bazel, and module exports together.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-security-policy && just fix -p codex-core`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused tests: `cd codex-rs && just test -p codex-security-policy credential && just test -p codex-core credential_capability`.
- [ ] Dependency parity from repository root: `just bazel-lock-update` when manifests change.
- [ ] TUI applicability: none; PF-26-S02 owns true-TUI qualification.

## Exit evidence

- [ ] Implementation commit, changed paths, and public API recorded.
- [ ] Final-tree output linked under `qa/security-levels/sprints/PF-13-S01/`.
- [ ] Secret-surface review confirms metadata cannot carry a credential value.
- [ ] Ledgers reflect reality and the completed record is archived.
