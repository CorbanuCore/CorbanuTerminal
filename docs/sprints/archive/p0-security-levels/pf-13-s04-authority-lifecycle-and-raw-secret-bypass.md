---
sprint_id: "PF-13-S04"
title: "Credential authority lifecycle and raw-secret bypass closure"
status: completed
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 12
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-pf13-s02"
branch: "feat/pf-13-s02-scoped-vault-resolver"
base_commit: "1bdc515bff48a4d9048dae7d06c6214e884265bc"
depends_on: "PF-13-S03"
created: 2026-08-24
updated: 2026-08-26
---

# PF-13-S04 — Credential authority lifecycle and raw-secret bypass closure

## Execution mandate

- Deliver: enforce revocation/replay and prevent Moderate/Aggressive agents from using raw-secret escape paths.
- Excludes: changing Permissive behavior, new providers, general protected-data policy, and TUI.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md)
- Feature: `PF-13`
- Acceptance advanced: credential use produces a secret-free decision/receipt and revoked authority cannot be replayed.

## Code boundaries

- CLI/Vault: `cli/src/main.rs::run_vault_auth_helper`; `vault/src/{lib,capability}.rs`
- Policy/Core: `security-policy/src/{lib,mandate}.rs`; `core/src/config/network_proxy_credential.rs`; `core/src/security/credential_capability.rs`
- Tests: `cli/tests/vault.rs`; Vault and security-policy unit tests; Core credential capability and resolver integration tests

## Preconditions

- [x] PF-13-S03 is completed and archived.
- [x] Read root, `codex-rs/AGENTS.md`, and `codex-rs/core/AGENTS.md`.
- [x] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record identifies the existing raw `auth-helper` and broker storage bypasses.
- [x] Consumed exact capabilities atomically so replay and concurrent duplicate use fail before another resolution.
- [x] Emitted secret-free `ActionReceipt` metadata for capability id, policy reason, operation, destination, and outcome.
- [x] Preserved `vault auth-helper` in Permissive and rejected it before vault access in Moderate/Aggressive, including attempted CLI downgrade.
- [x] Kept raw credentials out of child environment, tool output, stdout/command substitution, logs, audit metadata, errors, and serialization.
- [x] Added revocation-before-resolve, revoke-during-use, duplicate, replay, helper, environment, cancellation, and restart regressions.
- [x] Kept future finished-doc edits out of this sprint; PF-26-S03 retains that ownership.

## Remaining

- None.

## Verification

- [x] Fix: affected `codex-security-policy`, `codex-vault`, `codex-network-proxy`, `codex-core`, and `codex-cli` crates.
- [x] Format: `cd codex-rs && just fmt`; then inspected the final diff.
- [x] Focused tests: `cd codex-rs && just test -p codex-cli vault && just test -p codex-vault capability && just test -p codex-network-proxy credential_broker`.
- [x] Full affected non-Core suites: `codex-security-policy`, `codex-vault`, and `codex-network-proxy`.
- [x] Security integration: targeted Core `credential_capability` and `credential_authority` suites.
- [x] TUI applicability: none; PF-26-S02 owns the user-visible proof.

## Exit evidence

- [x] Implementation commit and changed paths recorded in `qa/security-levels/sprints/PF-13-S04/evidence.md`.
- [x] Denial/receipt output linked under `qa/security-levels/sprints/PF-13-S04/`.
- [x] Review proves Moderate/Aggressive have no supported raw-secret path.
- [x] Ledgers reflect reality and the completed record is archived.
