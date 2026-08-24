---
sprint_id: "PF-05-S01"
title: "URL canonicalization gate"
status: draft
plan_file: "docs/plans/proposed/prompt-injection-firewall.md"
plan_feature: "PF-05"
execution_order: 31
owner: "Jim Ricketts"
worktree: "UNALLOCATED"
branch: "UNALLOCATED"
base_commit: "UNALLOCATED"
depends_on: "none"
created: 2026-08-23
updated: 2026-08-23
---

# PF-05-S01 — URL canonicalization gate

## Execution mandate

- Deliver: Canonicalize and validate every retrieval URL before routing.
- Excludes: implementation owned by any plan feature other than `PF-05`.

## Plan linkage

- Plan: [Prompt-injection firewall and brokered authority](../../../plans/proposed/prompt-injection-firewall.md)
- Feature: `PF-05` — Network and IP gate
- Acceptance advanced: Canonicalize and validate every retrieval URL before routing.

## Code boundaries

- Existing: `codex-rs/network-proxy/src/policy.rs`; `codex-rs/config/src/permissions_toml.rs`
- Planned: `codex-rs/network-proxy/src/web_gate.rs`
- Tests: planned `codex-rs/network-proxy/tests/web_gate.rs`

## Preconditions

- [ ] The linked plan is active.
- [ ] No sprint dependencies.
- [ ] Worktree, branch, and base commit are exact and match the active plan.

## Done

- [x] Sprint record created, bounded, and linked to one plan feature.

## Remaining

- [ ] Accept only supported schemes and reject embedded credentials, fragments used as transport tricks, and invalid hosts.
- [ ] Normalize IDNA, ports, path, percent encoding, and IPv4/IPv6 representations once.
- [ ] Test alternate encodings, parser differentials, userinfo, empty hosts, and oversized URLs.

## Verification

- [ ] Focused final-tree command: `cd codex-rs && just test -p codex-network-proxy web_gate`
- [ ] Regression fixtures for this sprint pass.
- [ ] TUI applicability: Not standalone; integrated key-driven proof is owned by PF-12-S06.

## Exit evidence

- [ ] Implementation commit and changed paths recorded.
- [ ] Final-tree test output and failure artifacts linked.
- [ ] Scope diff contains only `PF-05` work.
- [ ] Completed record moved to `docs/sprints/archive/prompt-injection-firewall/`.
