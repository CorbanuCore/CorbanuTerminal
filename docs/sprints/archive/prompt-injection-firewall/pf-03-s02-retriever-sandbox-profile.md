---
sprint_id: "PF-03-S02"
title: "Retriever sandbox profile"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-03"
execution_order: 15
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-03-S01"
created: 2026-08-23
updated: 2026-08-24
---

# PF-03-S02 — Retriever sandbox profile

## Execution mandate

- Deliver: Enforce filesystem, process, capability, and resource isolation for the web workload.
- Excludes: implementation owned by any plan feature other than `PF-03`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-03` — Isolated public-web retrieval
- Acceptance advanced: Enforce filesystem, process, capability, and resource isolation for the web workload.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/network-proxy/src/`; `codex-rs/core/src/spawn.rs`
- Planned: `codex-rs/ext/web-search/src/isolated_fetch.rs`; `resources/web-retriever/`
- Tests: planned `codex-rs/ext/web-search/tests/isolated_fetch.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-03-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Run as non-root with read-only rootfs, bounded tmpfs, dropped capabilities, and resource limits.
- [ ] Mount no host paths or credentials and deny privileged container features.
- [ ] Test mount, socket, process escape, resource exhaustion, and teardown behavior.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension isolated_fetch`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-03` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
