---
sprint_id: "PF-05-S04"
title: "DNS-rebinding and connection pinning"
status: cancelled
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-05"
execution_order: 34
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-05-S03"
created: 2026-08-23
updated: 2026-08-24
---

# PF-05-S04 — DNS-rebinding and connection pinning

## Execution mandate

- Deliver: Bind approved resolution to the actual network connection.
- Excludes: implementation owned by any plan feature other than `PF-05`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-05` — Network and IP gate
- Acceptance advanced: Bind approved resolution to the actual network connection.

## Code boundaries

- Existing: `codex-rs/network-proxy/src/policy.rs`; `codex-rs/config/src/permissions_toml.rs`
- Planned: `codex-rs/network-proxy/src/web_gate.rs`
- Tests: planned `codex-rs/network-proxy/tests/web_gate.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-05-S03`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Pin the allowed address set or revalidate the connected peer before data transfer.
- [ ] Expire resolution decisions and prevent connection-pool reuse across changed policy.
- [ ] Test rebinding between resolution and connect, dual-stack races, retries, and pooled connections.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-network-proxy web_gate`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-05` work.
- [x] Cancelled unstarted and archived by product direction on 2026-08-24.
