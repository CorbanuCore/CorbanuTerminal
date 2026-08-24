---
sprint_id: "PF-10-S01"
title: "Hosted-classifier adapter contract"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-10"
execution_order: 56
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-08-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-10-S01 — Hosted-classifier adapter contract

## Execution mandate

- Deliver: Define a provider-neutral remote detector request and response boundary.
- Excludes: implementation owned by any plan feature other than `PF-10`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-10` — Optional hosted classifier service
- Acceptance advanced: Define a provider-neutral remote detector request and response boundary.

## Code boundaries

- Existing: `codex-rs/model-provider-info/src/lib.rs`; `codex-rs/vault/src/lib.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/hosted.rs`; `codex-rs/config/src/security.rs`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/hosted.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-08-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Send only allowed untrusted text plus non-sensitive classifier metadata.
- [ ] Resolve vendor credentials by reference and enforce endpoint, timeout, size, and region policy.
- [ ] Normalize verdicts and test malformed responses, drift, auth, quota, latency, and cancellation.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier hosted`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-10` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
