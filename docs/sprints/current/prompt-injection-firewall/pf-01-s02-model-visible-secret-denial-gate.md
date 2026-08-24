---
sprint_id: "PF-01-S02"
title: "Model-visible secret denial gate"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-01"
execution_order: 2
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-01-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-01-S02 — Model-visible secret denial gate

## Execution mandate

- Deliver: Reject protected values before model input, tool output, logs, or artifacts.
- Excludes: implementation owned by any plan feature other than `PF-01`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-01` — Secretless agent context
- Acceptance advanced: Reject protected values before model input, tool output, logs, or artifacts.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs`; `codex-rs/protocol/src/models.rs`; `codex-rs/core/src/tools/`
- Planned: `codex-rs/security-policy/src/protected_data.rs`; `codex-rs/security-policy/src/secret_broker.rs`
- Tests: `codex-rs/vault/src/tests.rs`; planned `codex-rs/security-policy/tests/protected_data.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-01-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Insert protected-data checks at model-input and tool-result materialization boundaries.
- [ ] Return typed denial metadata without echoing the protected value.
- [ ] Add canary tests for transcript, tool result, log, trace, and artifact paths.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-vault && just test -p codex-security-policy protected_data`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-01` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
