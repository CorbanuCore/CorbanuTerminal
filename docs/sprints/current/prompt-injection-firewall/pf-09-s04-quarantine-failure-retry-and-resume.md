---
sprint_id: "PF-09-S04"
title: "Quarantine failure, retry, and resume"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-09"
execution_order: 55
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-09-S03"
created: 2026-08-23
updated: 2026-08-23
---

# PF-09-S04 — Quarantine failure, retry, and resume

## Execution mandate

- Deliver: Recover safely from detector, storage, provider, and restart failures.
- Excludes: implementation owned by any plan feature other than `PF-09`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-09` — Quarantine and pre-model rejection
- Acceptance advanced: Recover safely from detector, storage, provider, and restart failures.

## Code boundaries

- Existing: `codex-rs/core/src/`; `codex-rs/tui/src/`; `codex-rs/protocol/src/models.rs`
- Planned: `codex-rs/security-policy/src/ingress_outcome.rs`; `codex-rs/tui/src/quarantine/`
- Tests: planned `codex-rs/core/tests/quarantine.rs`; `codex-rs/tui/src/quarantine/tests.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-09-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Persist pending state before showing recovery choices.
- [ ] Retry only the failed safe stage without reusing stale classification or authority.
- [ ] Test crash, restart, classifier outage, provider outage, deletion, and resumed agent context.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-core quarantine && just test -p codex-tui quarantine`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-09` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
