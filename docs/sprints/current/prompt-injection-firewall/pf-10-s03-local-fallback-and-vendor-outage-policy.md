---
sprint_id: "PF-10-S03"
title: "Local fallback and vendor-outage policy"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-10"
execution_order: 58
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-10-S02"
created: 2026-08-23
updated: 2026-08-23
---

# PF-10-S03 — Local fallback and vendor-outage policy

## Execution mandate

- Deliver: Keep Moderate and Aggressive screening safe during remote failures.
- Excludes: implementation owned by any plan feature other than `PF-10`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-10` — Optional hosted classifier service
- Acceptance advanced: Keep Moderate and Aggressive screening safe during remote failures.

## Code boundaries

- Existing: `codex-rs/model-provider-info/src/lib.rs`; `codex-rs/vault/src/lib.rs`
- Planned: `codex-rs/prompt-injection-classifier/src/hosted.rs`; `codex-rs/config/src/security.rs`
- Tests: planned `codex-rs/prompt-injection-classifier/tests/hosted.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-10-S02`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Route to a qualified local detector when hosted service is unavailable or disallowed.
- [ ] Prevent fallback from changing thresholds, provenance, or quarantine semantics silently.
- [ ] Test timeout, partial response, regional outage, rate limit, revoked opt-in, and local failure.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-prompt-injection-classifier hosted`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-10` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
