---
sprint_id: "PF-43-S01"
title: "Managed Claude subscription token lifecycle"
status: completed
plan_file: "docs/plans/active/claude-subscription-auth.md"
plan_feature: "PF-43"
execution_order: 2
owner: "Jim Ricketts"
parallel_lane: "claude-auth-serial"
write_scope: "codex-rs/vault/src/claude_auth.rs, codex-rs/vault/src/claude_auth_tests.rs, codex-rs/vault/src/lib.rs, codex-rs/vault/src/capability.rs, docs/plans/active/claude-subscription-auth.md, docs/sprints/current/claude-subscription-auth/"
integration_gate: "Jim Ricketts verifies the encrypted token lifecycle and adversarial redaction tests, archives PF-43-S01, then allocates the platform adapter without exposing token material."
worktree: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
branch: "feat/claude-subscription-auth-isolated"
base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
depends_on: "PF-42-S01"
created: 2026-08-30
updated: 2026-08-30
---

# CSA-02 / PF-43-S01 — Managed token lifecycle

## Execution mandate

- Deliver: encrypted add/replace/status/resolve/remove lifecycle for one long-lived Claude subscription token.
- Excludes: provider routing, platform credential discovery, setup-token TUI orchestration, and migration UX.

## Plan linkage

- Plan: [Reliable Claude subscription authentication](../../../plans/active/claude-subscription-auth.md).
- Feature: `PF-43` (plan alias `CSA-02`).
- Acceptance advanced: store the recommended method only in an approved secret backend and inspect metadata without revealing it.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs`; CSA-01 typed contract.
- Planned: managed-token operations and tests in `codex-rs/vault/src/claude_auth*.rs`.

## Preconditions

- [x] Plan is active.
- [x] PF-42-S01 is completed and archived.
- [x] Worktree, branch, and base commit match the plan.
- [x] Serial scope and integration gate are recorded.

## Done

- [x] Sprint record created and linked to PF-43.
- [x] Added bounded token validation plus encrypted add/replace and metadata-only status.
- [x] Added a dedicated short provider callback and exact local removal without changing Claude-owned stores.
- [x] Denied generic reveal, programmatic export, and scoped-capability resolution for the provider-managed token.
- [x] Proved token values stay out of metadata, errors, debug formatting, and encrypted-state inspection.

## Remaining

- [x] None.

## Verification

- [x] `cd codex-rs && just fix -p codex-vault && just fmt` preceded final tests.
- [x] `cd codex-rs && just test -p codex-vault --retries 0` passed 44 of 44 tests.
- [x] TUI applicability: none in this lifecycle-only sprint.

## Exit evidence

- [x] Implementation commit `6e4069ea1` and final-tree tests are recorded above.
- [x] Official setup-token eligibility/lifetime/limitations citation is recorded in the plan.
- [x] Done and Remaining reflect reality; completed record is archived.
