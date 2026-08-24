---
sprint_id: "PF-01-S01"
title: "Protected-data taxonomy and types"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-01"
execution_order: 1
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-01-S01 — Protected-data taxonomy and types

## Execution mandate

- Deliver: Define the canonical protected-data classes and serialization rules.
- Excludes: implementation owned by any plan feature other than `PF-01`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-01` — Secretless agent context
- Acceptance advanced: Define the canonical protected-data classes and serialization rules.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs`; `codex-rs/protocol/src/models.rs`; `codex-rs/core/src/tools/`
- Planned: `codex-rs/security-policy/src/protected_data.rs`; `codex-rs/security-policy/src/secret_broker.rs`
- Tests: `codex-rs/vault/src/tests.rs`; planned `codex-rs/security-policy/tests/protected_data.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Add typed classes for secrets, credentials, wallet material, financial state, and identity-linked data.
- [ ] Define model-visible, broker-only, audit-redacted, and derived-view dispositions for every class.
- [ ] Add exhaustive serialization and unknown-class fail-closed tests.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-vault && just test -p codex-security-policy protected_data`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-01` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
