---
sprint_id: "PF-10-S02"
title: "Hosted-service opt-in and disclosure"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-10"
execution_order: 57
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-10-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-10-S02 — Hosted-service opt-in and disclosure

## Execution mandate

- Deliver: Require trusted human opt-in with retention, region, cost, and privacy disclosure.
- Excludes: implementation owned by any plan feature other than `PF-10`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-10` — Optional hosted classifier service
- Acceptance advanced: Require trusted human opt-in with retention, region, cost, and privacy disclosure.

## Code boundaries

- Existing: `codex-rs/model-provider-info/src/lib.rs`; `codex-rs/vault/src/lib.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/hosted.rs`; `codex-rs/config/src/security.rs`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/hosted.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-10-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Persist opt-in separately from provider credentials and security level.
- [ ] Show exactly which content classes may leave the device.
- [ ] Test cancel, revoke, provider change, policy change, restart, and no silent default enablement.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier hosted`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Required: opt in, cancel, revoke, provider change, and restart with keys sent.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-10` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
