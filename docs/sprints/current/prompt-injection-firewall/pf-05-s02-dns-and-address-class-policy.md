---
sprint_id: "PF-05-S02"
title: "DNS and address-class policy"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-05"
execution_order: 32
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "PF-05-S01"
created: 2026-08-23
updated: 2026-08-23
---

# PF-05-S02 — DNS and address-class policy

## Execution mandate

- Deliver: Resolve A and AAAA records and reject every non-public destination class.
- Excludes: implementation owned by any plan feature other than `PF-05`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-05` — Network and IP gate
- Acceptance advanced: Resolve A and AAAA records and reject every non-public destination class.

## Code boundaries

- Existing: `codex-rs/network-proxy/src/policy.rs`; `codex-rs/config/src/permissions_toml.rs`
- Planned: `codex-rs/network-proxy/src/web_gate.rs`
- Tests: planned `codex-rs/network-proxy/tests/web_gate.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] Dependencies are completed: `PF-05-S01`.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Classify loopback, private, link-local, multicast, documentation, CGNAT, and cloud-metadata ranges.
- [ ] Evaluate all resolved addresses rather than the first answer.
- [ ] Test mixed public/private answers, IPv4-mapped IPv6, metadata names, and resolver errors.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-network-proxy web_gate`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-05` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
