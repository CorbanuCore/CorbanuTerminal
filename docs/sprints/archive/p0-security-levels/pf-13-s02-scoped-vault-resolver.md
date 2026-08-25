---
sprint_id: "PF-13-S02"
title: "Scoped vault resolver"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 10
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-pf13-s02"
branch: "feat/pf-13-s02-scoped-vault-resolver"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-13-S01"
created: 2026-08-24
updated: 2026-08-25
---

# PF-13-S02 — Scoped vault resolver

## Execution mandate

- Deliver: resolve one approved vault label only inside a zeroizing trusted callback.
- Excludes: HTTP/provider injection, legacy CLI gating, receipt persistence, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: the model and child environment receive an opaque reference, never the credential.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs::Vault::reveal_for_programmatic_use`
- Added: `codex-rs/vault/src/capability.rs::{VaultCredentialRef,Vault::with_scoped_credential}`
- Trusted adapter: `codex-rs/core/src/security/credential_capability.rs::AuthorizedCredentialCapability::into_vault_ref`
- Tests: `codex-rs/vault/src/capability_tests.rs`; `codex-rs/core/src/security/credential_capability_tests.rs`; affected Cargo and lock files

## Preconditions

- [x] PF-13-S01 is completed and archived.
- [x] Read root and `codex-rs/AGENTS.md` before implementation begins.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-13.
- [x] Accept only the typed approved capability and vault label supplied by the trusted Core adapter.
- [x] Resolve into `Zeroizing` temporary material inside a non-serializable callback; do not return `String`, bytes, or a displayable wrapper, and use constant-time comparison wherever secret material is compared.
- [x] Make callback success, error, panic containment, cancellation, and cleanup erase temporary material.
- [x] Reject missing, deleted, wrong-type, wrong-scope, expired, revoked, and mismatched-label references with stable secret-free errors.
- [x] Add redaction tests for `Debug`, `Display`, errors, tracing, serialization, and temporary cleanup.
- [x] Keep `reveal_for_programmatic_use` unchanged in this slice; PF-13-S04 owns profile-aware gating.

## Remaining

- None.

## Verification

- [x] Fix: `cd codex-rs && just fix -p codex-vault && just fix -p codex-core`.
- [x] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [x] Focused final-tree test: `cd codex-rs && just test -p codex-vault capability`.
- [x] Regression: `cd codex-rs && just test -p codex-vault`.
- [x] Core adapter: `cd codex-rs && just test -p codex-core credential_capability`.
- [x] Dependency parity: `just bazel-lock-update && just bazel-lock-check`.
- [x] TUI applicability: none; no interactive surface changes.

## Exit evidence

- [x] Implementation commit and changed paths recorded.
- [x] Final-tree output linked under `qa/security-levels/sprints/PF-13-S02/`.
- [x] Memory/redaction review records the exact temporary-secret lifetime.
- [x] Ledgers reflect reality and the completed record is archived.
