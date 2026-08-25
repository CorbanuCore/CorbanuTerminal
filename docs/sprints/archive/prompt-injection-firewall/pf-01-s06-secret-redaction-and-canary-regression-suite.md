---
sprint_id: "PF-01-S06"
title: "Secret redaction and canary regression suite"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-01"
execution_order: 6
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-01-S05"
created: 2026-08-23
updated: 2026-08-24
---

# PF-01-S06 — Secret redaction and canary regression suite

## Execution mandate

- Deliver: Prove every model-visible and evidence channel stays secret-free.
- Excludes: implementation owned by any plan feature other than `PF-01`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-01` — Secretless agent context
- Acceptance advanced: Prove every model-visible and evidence channel stays secret-free.

## Code boundaries

- Existing: `codex-rs/vault/src/lib.rs`; `codex-rs/protocol/src/models.rs`; `codex-rs/core/src/tools/`
- Planned: `codex-rs/security-policy/src/protected_data.rs`; `codex-rs/security-policy/src/secret_broker.rs`
- Tests: `codex-rs/vault/src/tests.rs`; planned `codex-rs/security-policy/tests/protected_data.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-01-S05`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Create synthetic canary credentials for each protected class.
- [ ] Scan transcripts, audit events, tool payloads, crash output, and QA artifacts for canaries.
- [ ] Make any canary occurrence a failing security test with the leaking boundary identified.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-vault && just test -p codex-security-policy protected_data`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-01` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
