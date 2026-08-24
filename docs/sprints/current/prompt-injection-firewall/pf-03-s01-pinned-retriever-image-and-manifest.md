---
sprint_id: "PF-03-S01"
title: "Pinned retriever image and manifest"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-03"
execution_order: 14
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-03-S01 — Pinned retriever image and manifest

## Execution mandate

- Deliver: Define the reviewed Scrapling-class runtime artifact and capability manifest.
- Excludes: implementation owned by any plan feature other than `PF-03`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-03` — Isolated public-web retrieval
- Acceptance advanced: Define the reviewed Scrapling-class runtime artifact and capability manifest.

## Code boundaries

- Existing: `codex-rs/ext/web-search/src/`; `codex-rs/network-proxy/src/`; `codex-rs/core/src/spawn.rs`
- Planned: `codex-rs/ext/web-search/src/isolated_fetch.rs`; `resources/web-retriever/`
- Tests: planned `codex-rs/ext/web-search/tests/isolated_fetch.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Pin image digest, entrypoint, exposed operations, SBOM, and update policy.
- [ ] Declare no workspace, vault, wallet, browser-profile, clipboard, or host-socket access.
- [ ] Add manifest validation and digest-drift failure tests.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-web-search-extension isolated_fetch`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-03` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
