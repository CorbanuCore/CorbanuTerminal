---
sprint_id: "PF-13-S02"
title: "Scoped vault resolver"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-13"
execution_order: 10
owner: "Jim Ricketts"
worktree: "/home/pfrpc/repos/CorbanuTerminal-security-levels"
branch: "feat/p0-security-levels"
base_commit: "3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb"
depends_on: "PF-13-S01"
created: 2026-08-24
updated: 2026-08-24
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
- Planned: `codex-rs/vault/src/capability.rs::{VaultCredentialRef,Vault::with_scoped_credential}`
- Tests: planned `codex-rs/vault/src/capability_tests.rs`; affected Cargo/Bazel files

## Preconditions

- [ ] PF-13-S01 is completed and archived.
- [ ] Read root and `codex-rs/AGENTS.md`.
- [ ] Exact worktree coordinates match the active plan.

## Done

- [x] Sprint record is linked only to PF-13.

## Remaining

- [ ] Accept only the typed approved capability and vault label supplied by the trusted Core adapter.
- [ ] Resolve into `Zeroizing` temporary material inside a non-serializable callback; do not return `String`, bytes, or a displayable wrapper, and use constant-time comparison wherever secret material is compared.
- [ ] Make callback success, error, panic containment, cancellation, and cleanup erase temporary material.
- [ ] Reject missing, deleted, wrong-type, wrong-scope, expired, revoked, and mismatched-label references with stable secret-free errors.
- [ ] Add redaction tests for `Debug`, `Display`, errors, tracing, serialization, and temporary cleanup.
- [ ] Keep `reveal_for_programmatic_use` unchanged in this slice; PF-13-S04 owns profile-aware gating.

## Verification

- [ ] Fix: `cd codex-rs && just fix -p codex-vault`.
- [ ] Format: `cd codex-rs && just fmt`; then inspect the final diff.
- [ ] Focused final-tree test: `cd codex-rs && just test -p codex-vault capability`.
- [ ] Regression: `cd codex-rs && just test -p codex-vault`.
- [ ] TUI applicability: none; no interactive surface changes.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree output linked under `qa/security-levels/sprints/PF-13-S02/`.
- [ ] Memory/redaction review records the exact temporary-secret lifetime.
- [ ] Ledgers reflect reality and the completed record is archived.
